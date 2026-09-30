//! The bounded export queue with exact per-run accounting.
//!
//! The queue is one FIFO of items: log records, run snapshots (T21.F02) and
//! tick traces (T21.F03), each attributed to its run when it is offered. From
//! then on an item is in exactly one place: dropped (queue full or body over
//! the cap), queued, in the one batch in flight, or resolved as exported,
//! failed or abandoned. A batch holds records of one run only, or one
//! snapshot or trace, so what a collector's partial success rejects belongs
//! to that run. A run's report
//! line is written once, when the run has ended and none of its items is
//! queued or in flight, or when a flush deadline abandons the rest.

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use opentelemetry::logs::AnyValue;
use opentelemetry::InstrumentationScope;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogBatch, LogExporter, LogProcessor, SdkLogRecord};

use crate::export::Rejections;
use crate::{Limits, ReportSink, RUN_ID_KEY};

/// A run's key: its 128-bit ID.
pub(crate) type RunKey = u128;

/// The OTLP signal of an encoded request the worker posts itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Signal {
    /// A run snapshot: an OTLP metrics request for `/v1/metrics`.
    Metrics,
    /// A tick trace: an OTLP trace request for `/v1/traces`.
    Traces,
}

impl Signal {
    /// The item's name on a gap line.
    fn item_name(self) -> &'static str {
        match self {
            Self::Metrics => "snapshot",
            Self::Traces => "trace",
        }
    }
}

/// A queued item: a log record, or an encoded OTLP request.
enum Item {
    Record(Box<(SdkLogRecord, InstrumentationScope)>),
    Encoded(Signal, Vec<u8>),
}

struct Entry {
    run: RunKey,
    bytes: u64,
    item: Item,
}

/// What the worker exports next: leading records of one run, or one encoded
/// request.
enum Batch {
    Records(Vec<(SdkLogRecord, InstrumentationScope)>),
    Encoded(Signal, Vec<u8>),
}

/// One run's counts. `accepted` items entered the queue; each of them ends as
/// exported, failed or abandoned. `snapshots` and `traces` count those taken,
/// whether or not the queue accepted them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RunCounts {
    pub(crate) accepted: u64,
    pub(crate) exported: u64,
    pub(crate) failed: u64,
    pub(crate) dropped: u64,
    pub(crate) abandoned: u64,
    /// Payload bytes of exported items: a record's body plus attribute keys
    /// and values, a snapshot's or trace's encoded request.
    pub(crate) bytes: u64,
    pub(crate) snapshots: u64,
    pub(crate) traces: u64,
}

impl RunCounts {
    fn pending(&self) -> u64 {
        self.accepted - self.exported - self.failed - self.abandoned
    }
}

struct RunEntry {
    id: String,
    counts: RunCounts,
    self_time: Duration,
    ended_at: Option<Instant>,
}

/// The batch the worker is exporting: one run's records and their bytes.
struct InFlight {
    run: RunKey,
    records: u64,
    bytes: u64,
}

/// How the collector answered a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// Accepted, except this many items (records, a snapshot's data points
    /// or a trace's spans) a partial success rejected.
    Exported { rejected: u64 },
    /// The request failed; nothing of the batch was stored.
    Failed,
}

/// Posts one encoded request of `signal` and reports how the collector
/// answered.
pub(crate) trait PostEncoded {
    fn post(&self, signal: Signal, body: Vec<u8>) -> Outcome;
}

#[derive(Default)]
struct State {
    queue: VecDeque<Entry>,
    queued_bytes: u64,
    in_flight: Option<InFlight>,
    runs: HashMap<RunKey, RunEntry>,
    closed: bool,
    /// Test hook: while set, the worker takes no batch.
    held: bool,
}

pub(crate) struct Shared {
    state: Mutex<State>,
    changed: Condvar,
    limits: Limits,
    sink: ReportSink,
}

impl fmt::Debug for Shared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shared")
            .field("limits", &self.limits)
            .finish_non_exhaustive()
    }
}

/// Payload size of a value: string bytes, or eight for a scalar.
fn value_bytes(value: &AnyValue) -> u64 {
    match value {
        AnyValue::String(text) => text.as_str().len() as u64,
        AnyValue::Bytes(bytes) => bytes.len() as u64,
        _ => 8,
    }
}

impl Shared {
    pub(crate) fn new(limits: Limits, sink: ReportSink, held: bool) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State {
                held,
                ..State::default()
            }),
            changed: Condvar::new(),
            limits,
            sink,
        })
    }

    pub(crate) fn sink(&self) -> &ReportSink {
        &self.sink
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn register_run(&self, run: RunKey, id: String) {
        self.lock().runs.insert(
            run,
            RunEntry {
                id,
                counts: RunCounts::default(),
                self_time: Duration::ZERO,
                ended_at: None,
            },
        );
    }

    /// Queues one record for `run`, or drops and counts it.
    fn offer(&self, run: RunKey, record: SdkLogRecord, scope: InstrumentationScope) {
        let body_bytes = record.body().map_or(0, value_bytes);
        let bytes = body_bytes
            + record
                .attributes_iter()
                .map(|(key, value)| key.as_str().len() as u64 + value_bytes(value))
                .sum::<u64>();
        let name = record.event_name().unwrap_or("unnamed");
        self.enqueue(
            run,
            bytes,
            body_bytes,
            name,
            Item::Record(Box::new((record, scope))),
        );
    }

    /// Counts a snapshot or trace taken for `run` and queues its encoded
    /// request, or drops and counts it as a record would be.
    pub(crate) fn offer_encoded(&self, run: RunKey, signal: Signal, body: Vec<u8>) {
        if let Some(entry) = self.lock().runs.get_mut(&run) {
            match signal {
                Signal::Metrics => entry.counts.snapshots += 1,
                Signal::Traces => entry.counts.traces += 1,
            }
        }
        let bytes = body.len() as u64;
        self.enqueue(
            run,
            bytes,
            bytes,
            signal.item_name(),
            Item::Encoded(signal, body),
        );
    }

    fn enqueue(&self, run: RunKey, bytes: u64, body_bytes: u64, name: &'static str, item: Item) {
        let over_cap = body_bytes > self.limits.max_body_bytes;
        let mut state = self.lock();
        let full = state.closed
            || state.queue.len() >= self.limits.max_queue_records
            || state.queued_bytes + bytes > self.limits.max_queue_bytes;
        let Some(entry) = state.runs.get_mut(&run) else {
            return;
        };
        if over_cap || full {
            entry.counts.dropped += 1;
            let gap = over_cap.then(|| entry.id.clone());
            drop(state);
            if let Some(id) = gap {
                self.sink.write(&format!(
                    "telemetry: gap run={id} record={name} body_bytes={body_bytes} cap={} dropped whole",
                    self.limits.max_body_bytes,
                ));
            }
            return;
        }
        entry.counts.accepted += 1;
        state.queued_bytes += bytes;
        state.queue.push_back(Entry { run, bytes, item });
        drop(state);
        self.changed.notify_all();
    }

    pub(crate) fn add_self_time(&self, run: RunKey, elapsed: Duration) {
        if let Some(entry) = self.lock().runs.get_mut(&run) {
            entry.self_time += elapsed;
        }
    }

    /// Marks `run` ended; its line is written once nothing of it is pending.
    pub(crate) fn mark_ended(&self, run: RunKey) {
        let mut state = self.lock();
        if let Some(entry) = state.runs.get_mut(&run) {
            entry.ended_at.get_or_insert_with(Instant::now);
        }
        self.report_if_resolved(&mut state, run);
    }

    /// Waits until `run`'s line has been written, abandoning whatever of it is
    /// still pending when `limits.flush_timeout` has passed.
    pub(crate) fn flush_run(&self, run: RunKey) {
        let deadline = Instant::now() + self.limits.flush_timeout;
        let mut state = self.lock();
        while state.runs.contains_key(&run) {
            let now = Instant::now();
            if now >= deadline {
                Self::abandon(&mut state, run);
                self.report(&mut state, run);
                break;
            }
            state = self
                .changed
                .wait_timeout(state, deadline - now)
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .0;
        }
    }

    /// Waits for every run to resolve, abandons what remains at the flush
    /// deadline, writes every outstanding line and stops the worker.
    pub(crate) fn shutdown(&self) {
        let deadline = Instant::now() + self.limits.flush_timeout;
        let mut state = self.lock();
        loop {
            let pending = state.runs.values().any(|entry| entry.counts.pending() > 0);
            let now = Instant::now();
            if !pending || now >= deadline {
                break;
            }
            state = self
                .changed
                .wait_timeout(state, deadline - now)
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .0;
        }
        let mut runs: Vec<RunKey> = state.runs.keys().copied().collect();
        runs.sort_unstable();
        for run in runs {
            Self::abandon(&mut state, run);
            if let Some(entry) = state.runs.get_mut(&run) {
                entry.ended_at.get_or_insert_with(Instant::now);
            }
            self.report(&mut state, run);
        }
        state.closed = true;
        drop(state);
        self.changed.notify_all();
    }

    /// Resolves every queued or in-flight record of `run` as abandoned.
    fn abandon(state: &mut State, run: RunKey) {
        let mut abandoned = 0;
        let mut freed = 0;
        state.queue.retain(|entry| {
            if entry.run == run {
                abandoned += 1;
                freed += entry.bytes;
                false
            } else {
                true
            }
        });
        state.queued_bytes -= freed;
        if let Some(flight) = state.in_flight.take_if(|flight| flight.run == run) {
            abandoned += flight.records;
        }
        if let Some(entry) = state.runs.get_mut(&run) {
            entry.counts.abandoned += abandoned;
        }
    }

    fn report_if_resolved(&self, state: &mut State, run: RunKey) {
        let resolved = state
            .runs
            .get(&run)
            .is_some_and(|entry| entry.ended_at.is_some() && entry.counts.pending() == 0);
        if resolved {
            self.report(state, run);
        }
        self.changed.notify_all();
    }

    /// Writes `run`'s line and releases its counters.
    fn report(&self, state: &mut State, run: RunKey) {
        let Some(entry) = state.runs.remove(&run) else {
            return;
        };
        let flush_ms = entry.ended_at.map_or(0, |at| at.elapsed().as_millis());
        let RunCounts {
            exported,
            failed,
            dropped,
            abandoned,
            bytes,
            snapshots,
            traces,
            ..
        } = entry.counts;
        self.sink.write(&format!(
            "telemetry: run={} exported={exported} failed={failed} dropped={dropped} abandoned={abandoned} bytes={bytes} self_time_us={} flush_ms={flush_ms} snapshots={snapshots} traces={traces}",
            entry.id,
            entry.self_time.as_micros(),
        ));
    }

    /// The worker's next batch: the queue's leading records of one run, up to
    /// the batch bound, or its leading encoded request alone; `None` once the
    /// queue is closed.
    fn take_batch(&self) -> Option<Batch> {
        let mut state = self.lock();
        while !state.closed && (state.held || state.queue.is_empty()) {
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        if state.closed {
            return None;
        }
        let front = state.queue.front()?;
        let run = front.run;
        let take = if matches!(front.item, Item::Encoded(..)) {
            1
        } else {
            state
                .queue
                .iter()
                .take(self.limits.batch_records)
                .take_while(|entry| entry.run == run && matches!(entry.item, Item::Record(_)))
                .count()
        };
        let entries: Vec<Entry> = state.queue.drain(..take).collect();
        let bytes = entries.iter().map(|entry| entry.bytes).sum();
        state.queued_bytes -= bytes;
        state.in_flight = Some(InFlight {
            run,
            records: entries.len() as u64,
            bytes,
        });
        let mut records = Vec::with_capacity(entries.len());
        for entry in entries {
            match entry.item {
                Item::Record(record) => records.push(*record),
                Item::Encoded(signal, body) => return Some(Batch::Encoded(signal, body)),
            }
        }
        Some(Batch::Records(records))
    }

    /// Resolves the in-flight batch; records a flush already abandoned are not
    /// counted twice. A partial success names how many records it rejected,
    /// not which, so the bytes of a partly rejected batch are not counted.
    fn complete_batch(&self, outcome: Outcome) {
        let mut state = self.lock();
        let Some(InFlight {
            run,
            records,
            bytes,
        }) = state.in_flight.take()
        else {
            return;
        };
        if let Some(entry) = state.runs.get_mut(&run) {
            match outcome {
                Outcome::Exported { rejected: 0 } => {
                    entry.counts.exported += records;
                    entry.counts.bytes += bytes;
                }
                Outcome::Exported { rejected } => {
                    let rejected = rejected.min(records);
                    entry.counts.exported += records - rejected;
                    entry.counts.failed += rejected;
                }
                Outcome::Failed => entry.counts.failed += records,
            }
        }
        self.report_if_resolved(&mut state, run);
    }

    #[cfg(test)]
    pub(crate) fn release(&self) {
        self.lock().held = false;
        self.changed.notify_all();
    }

    #[cfg(test)]
    pub(crate) fn counts(&self, run: RunKey) -> Option<RunCounts> {
        self.lock().runs.get(&run).map(|entry| entry.counts.clone())
    }
}

/// Exports batches until the queue closes: records through the log exporter,
/// whose partial-success rejections `rejections` holds after each export, and
/// snapshots and traces through `encoded`. Owns both so their HTTP clients are
/// used and dropped off any async runtime.
pub(crate) fn run_worker<E: LogExporter, P: PostEncoded>(
    shared: &Shared,
    exporter: E,
    encoded: P,
    rejections: &Rejections,
) {
    while let Some(batch) = shared.take_batch() {
        let outcome = match batch {
            Batch::Encoded(signal, body) => encoded.post(signal, body),
            Batch::Records(records) => {
                let refs: Vec<(&SdkLogRecord, &InstrumentationScope)> =
                    records.iter().map(|item| (&item.0, &item.1)).collect();
                let result = futures_executor::block_on(exporter.export(LogBatch::new(&refs)));
                let rejected = rejections.take();
                match result {
                    Ok(()) => Outcome::Exported { rejected },
                    Err(_) => Outcome::Failed,
                }
            }
        };
        shared.complete_batch(outcome);
    }
    let _ = exporter.shutdown();
}

/// The SDK log processor that feeds [`Shared`]. Each record names its run in
/// its `petri.run_id` attribute.
#[derive(Debug)]
pub(crate) struct QueueProcessor {
    pub(crate) shared: Arc<Shared>,
}

impl LogProcessor for QueueProcessor {
    fn emit(&self, record: &mut SdkLogRecord, scope: &InstrumentationScope) {
        let run = record
            .attributes_iter()
            .find_map(|(key, value)| match value {
                AnyValue::String(id) if key.as_str() == RUN_ID_KEY => {
                    u128::from_str_radix(id.as_str(), 16).ok()
                }
                _ => None,
            });
        if let Some(run) = run {
            self.shared.offer(run, record.clone(), scope.clone());
        }
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }
}
