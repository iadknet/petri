//! The bounded export queue with exact per-run accounting.
//!
//! Every record is attributed to its run when it is offered. From then on it is
//! in exactly one place: dropped (queue full or body over the cap), queued, in
//! the one batch in flight, or resolved as exported, failed or abandoned. A
//! run's report line is written once, when the run has ended and none of its
//! records is queued or in flight, or when a flush deadline abandons the rest.

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use opentelemetry::logs::AnyValue;
use opentelemetry::InstrumentationScope;
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::logs::{LogBatch, LogExporter, LogProcessor, SdkLogRecord};

use crate::{Limits, ReportSink, RUN_ID_KEY};

/// A run's key: its 128-bit ID.
pub(crate) type RunKey = u128;

struct Entry {
    run: RunKey,
    bytes: u64,
    item: Box<(SdkLogRecord, InstrumentationScope)>,
}

/// One run's counts. `accepted` records entered the queue; each of them ends as
/// exported, failed or abandoned.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RunCounts {
    pub(crate) accepted: u64,
    pub(crate) exported: u64,
    pub(crate) failed: u64,
    pub(crate) dropped: u64,
    pub(crate) abandoned: u64,
    /// Payload bytes (body plus attribute keys and values) of exported records.
    pub(crate) bytes: u64,
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

/// The batch the worker is exporting: records per run and their bytes.
#[derive(Default)]
struct InFlight {
    per_run: HashMap<RunKey, (u64, u64)>,
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
                    "telemetry: gap run={id} record={} body_bytes={body_bytes} cap={} dropped whole",
                    record.event_name().unwrap_or("unnamed"),
                    self.limits.max_body_bytes,
                ));
            }
            return;
        }
        entry.counts.accepted += 1;
        state.queued_bytes += bytes;
        state.queue.push_back(Entry {
            run,
            bytes,
            item: Box::new((record, scope)),
        });
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
        if let Some((count, _)) = state
            .in_flight
            .as_mut()
            .and_then(|flight| flight.per_run.remove(&run))
        {
            abandoned += count;
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
            ..
        } = entry.counts;
        self.sink.write(&format!(
            "telemetry: run={} exported={exported} failed={failed} dropped={dropped} abandoned={abandoned} bytes={bytes} self_time_us={} flush_ms={flush_ms}",
            entry.id,
            entry.self_time.as_micros(),
        ));
    }

    /// The worker's next batch, or `None` once the queue is closed.
    fn take_batch(&self) -> Option<Vec<Entry>> {
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
        let take = state.queue.len().min(self.limits.batch_records);
        let batch: Vec<Entry> = state.queue.drain(..take).collect();
        let mut flight = InFlight::default();
        for entry in &batch {
            state.queued_bytes -= entry.bytes;
            let slot = flight.per_run.entry(entry.run).or_default();
            slot.0 += 1;
            slot.1 += entry.bytes;
        }
        state.in_flight = Some(flight);
        Some(batch)
    }

    /// Resolves the in-flight batch; records a flush already abandoned are not
    /// counted twice.
    fn complete_batch(&self, exported: bool) {
        let mut state = self.lock();
        let Some(flight) = state.in_flight.take() else {
            return;
        };
        for (run, (count, bytes)) in flight.per_run {
            if let Some(entry) = state.runs.get_mut(&run) {
                if exported {
                    entry.counts.exported += count;
                    entry.counts.bytes += bytes;
                } else {
                    entry.counts.failed += count;
                }
            }
            self.report_if_resolved(&mut state, run);
        }
        self.changed.notify_all();
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

/// Exports batches until the queue closes. Owns the exporter so its HTTP client
/// is created, used and dropped off any async runtime.
pub(crate) fn run_worker<E: LogExporter>(shared: &Shared, exporter: E) {
    while let Some(batch) = shared.take_batch() {
        let refs: Vec<(&SdkLogRecord, &InstrumentationScope)> = batch
            .iter()
            .map(|entry| (&entry.item.0, &entry.item.1))
            .collect();
        let exported = futures_executor::block_on(exporter.export(LogBatch::new(&refs))).is_ok();
        shared.complete_batch(exported);
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
