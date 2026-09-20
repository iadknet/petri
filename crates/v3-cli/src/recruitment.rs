//! The T13.F07 S0 recruitment panel: `v3-cli recruitment`.
//!
//! Runs the recruitment assay under production supply, one bounded lineage at
//! a time, streaming each lineage's compact record to the raw file as it
//! completes. The wall-clock and byte caps stop the run between lineages and
//! mark the record `incomplete`; a completed lineage is never dropped. The
//! summary carries the per-arm estimates, ladder and classification counts,
//! and the optional replay check that rebuilds every proposal from the record.

use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use v3_core::neighborhood::recruitment::Opportunities;
use v3_core::neighborhood::recruitment_paths::{
    summaries, Assay, CheckpointScalars, CompactLineage, HorizonOutcome, Ladder, LineageClass,
    LineageFacts, Panel, Policy, ReplayCheck, RetentionOutcome, Sizes, Summary, Task, S0_VERSION,
};

use crate::bench::artifacts::{output_paths_with_suffix, write_json};

/// The record kind the summary carries.
pub const SUMMARY_KIND: &str = "petri-recruitment-s0-summary";
/// Default wall-clock cap, seconds.
pub const DEFAULT_WALL_CAP_SECS: u64 = 7_200;
/// Default on-disk byte cap: 2 GiB.
pub const DEFAULT_BYTE_CAP: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct Options {
    pub feature: String,
    pub pilot: bool,
    /// Private rayon pool size; `None` uses the global pool.
    pub threads: Option<usize>,
    pub wall_cap: Duration,
    pub byte_cap: u64,
    pub replay_check: bool,
    /// Sizes other than the panel's fixed ones (tests only); still bounded by
    /// the S0 cap.
    pub sizes: Option<Sizes>,
    pub raw: Option<PathBuf>,
    pub summary: Option<PathBuf>,
    pub cwd: PathBuf,
    pub source_revision: String,
}

impl Options {
    #[must_use]
    pub fn new(feature: &str, cwd: PathBuf) -> Self {
        Self {
            feature: feature.into(),
            pilot: false,
            threads: None,
            wall_cap: Duration::from_secs(DEFAULT_WALL_CAP_SECS),
            byte_cap: DEFAULT_BYTE_CAP,
            replay_check: false,
            sizes: None,
            raw: None,
            summary: None,
            cwd,
            source_revision: "unknown".into(),
        }
    }

    fn panel(&self) -> Panel {
        Panel::s0(self.sizes.unwrap_or(if self.pilot {
            Sizes::S0_PILOT
        } else {
            Sizes::S0
        }))
    }
}

/// The raw record's header and footer, around the streamed `lineages` array.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawRecord {
    pub version: String,
    pub feature: String,
    pub pilot: bool,
    pub panel: Panel,
    pub supply_rule: String,
    pub source_revision: String,
    pub config_digest: String,
    pub threads: usize,
    pub lineages: Vec<CompactLineage>,
    pub incomplete: bool,
    pub stop_reason: Option<String>,
    pub lineage_count: u64,
    pub wall_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmSummary {
    pub arm: usize,
    pub start: String,
    pub policy: Policy,
    pub task: Task,
    pub lineages: u32,
    pub summary: Summary,
    pub batches: Vec<Summary>,
    pub opportunities: Opportunities,
    /// One compact row per completed lineage, in `(batch, lineage)` order.
    pub rows: Vec<LineageRow>,
}

/// One lineage's outcome as the summary keeps it: no module or proposal
/// payloads, only generations, outcomes and the final checkpoint scalars.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineageRow {
    pub batch: u32,
    pub lineage: u32,
    pub classification: LineageClass,
    pub ladder: Ladder,
    pub proposal_discovery: Option<u32>,
    pub retained_discovery: Option<u32>,
    pub specialized_discovery: Option<u32>,
    pub retention: Option<RetentionOutcome>,
    pub horizons: Vec<(u32, HorizonOutcome)>,
    pub specialized_proposals: u64,
    pub final_checkpoint: Option<CheckpointScalars>,
}

impl LineageRow {
    fn of(facts: &LineageFacts) -> Self {
        Self {
            batch: facts.batch,
            lineage: facts.lineage,
            classification: facts.classification,
            ladder: facts.ladder,
            proposal_discovery: facts.proposal_discovery,
            retained_discovery: facts.retained_discovery,
            specialized_discovery: facts.specialized_discovery,
            retention: facts.retention,
            horizons: facts.horizons.clone(),
            specialized_proposals: facts.specialized_proposals,
            final_checkpoint: facts.checkpoints.last().copied(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawIdentity {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

/// The committed summary of one S0 or pilot run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub kind: String,
    pub version: String,
    pub feature: String,
    pub pilot: bool,
    pub panel: Panel,
    pub supply_rule: String,
    pub source_revision: String,
    pub config_digest: String,
    pub threads: usize,
    pub wall_secs: f64,
    pub incomplete: bool,
    pub stop_reason: Option<String>,
    pub lineage_count: u64,
    pub proposal_count: u64,
    pub expected_proposals: u64,
    pub raw: RawIdentity,
    pub replay_check: Option<ReplayCheck>,
    pub arms: Vec<ArmSummary>,
    pub opportunities: Opportunities,
}

/// What a run produced.
#[derive(Debug, Clone)]
pub struct Outcome {
    pub raw: PathBuf,
    pub summary: PathBuf,
    pub incomplete: bool,
    pub lineage_count: u64,
    pub proposal_count: u64,
    pub bytes: u64,
    pub replay_check: Option<ReplayCheck>,
}

/// Counts and hashes every byte on its way to the file.
struct HashingWriter<W: Write> {
    inner: W,
    hasher: Sha256,
    bytes: u64,
}

impl<W: Write> Write for HashingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let written = self.inner.write(buf)?;
        self.hasher.update(&buf[..written]);
        self.bytes += written as u64;
        Ok(written)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

struct Stream {
    writer: HashingWriter<BufWriter<std::fs::File>>,
    written: u64,
    facts: Vec<(usize, LineageFacts, Opportunities)>,
    error: Option<String>,
}

impl Stream {
    fn append(&mut self, arm: usize, record: &CompactLineage) -> u64 {
        if self.error.is_some() {
            return self.writer.bytes;
        }
        let result = (|| {
            if self.written > 0 {
                self.writer.write_all(b",\n")?;
            }
            serde_json::to_writer(&mut self.writer, record)
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            self.writer.flush()
        })();
        match result {
            Ok(()) => {
                self.written += 1;
                self.facts.push((
                    arm,
                    LineageFacts::of_compact(record),
                    record.opportunities.clone(),
                ));
            }
            Err(e) => self.error = Some(format!("failed to write lineage record: {e}")),
        }
        self.writer.bytes
    }
}

fn io_error(path: &Path, e: &std::io::Error) -> String {
    format!("failed to write {}: {e}", path.display())
}

/// Run the panel, write both artifacts, and report what was produced.
///
/// # Errors
/// When an output path cannot be resolved or written.
pub fn run(options: &Options) -> Result<Outcome, String> {
    let panel = options.panel();
    let profile = if options.pilot {
        "recruitment-s0-pilot"
    } else {
        "recruitment-s0"
    };
    let suffix = if options.pilot { "-s0-pilot" } else { "-s0" };
    let paths = output_paths_with_suffix(
        &options.cwd,
        profile,
        suffix,
        Some(&options.feature),
        options.raw.as_deref(),
        options.summary.as_deref(),
    )?;
    let pool = options
        .threads
        .map(|threads| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads.max(1))
                .build()
                .map_err(|e| format!("cannot build a {threads}-thread pool: {e}"))
        })
        .transpose()?;
    let threads = pool.as_ref().map_or_else(
        rayon::current_num_threads,
        rayon::ThreadPool::current_num_threads,
    );
    let assay = Assay::new(panel);
    let config_digest =
        v3_core::config::config_digest(&v3_core::neighborhood::recruitment_paths::task_config());
    let header = RawRecord {
        version: S0_VERSION.into(),
        feature: options.feature.clone(),
        pilot: options.pilot,
        panel,
        supply_rule: panel.supply.rule().into(),
        source_revision: options.source_revision.clone(),
        config_digest: config_digest.clone(),
        threads,
        lineages: Vec::new(),
        incomplete: false,
        stop_reason: None,
        lineage_count: 0,
        wall_secs: 0.0,
    };

    let stream = open_stream(&paths.raw, &header)?;
    let started = Instant::now();
    let Execution {
        facts,
        stop_reason,
        incomplete,
    } = execute(options, &assay, pool.as_ref(), stream, started)?;
    let wall_secs = started.elapsed().as_secs_f64();
    let stream = facts.stream;
    let footer = serde_json::json!({
        "incomplete": incomplete,
        "stop_reason": stop_reason,
        "lineage_count": facts.records.len(),
        "wall_secs": wall_secs,
    });
    let (bytes, sha256) = close_stream(&paths.raw, stream, &footer)?;
    let replay_check = options
        .replay_check
        .then(|| replay(&paths.raw, &assay, pool.as_ref()))
        .transpose()?;
    let (arms, opportunities) = summarize_arms(&assay, &facts.records);
    let lineage_count = facts.records.len() as u64;
    let summary = RunSummary {
        kind: SUMMARY_KIND.into(),
        version: S0_VERSION.into(),
        feature: options.feature.clone(),
        pilot: options.pilot,
        panel,
        supply_rule: panel.supply.rule().into(),
        source_revision: options.source_revision.clone(),
        config_digest,
        threads,
        wall_secs,
        incomplete,
        stop_reason,
        lineage_count,
        proposal_count: opportunities.births,
        expected_proposals: panel.sizes.proposals(),
        raw: RawIdentity {
            path: paths.raw.display().to_string(),
            bytes,
            sha256,
        },
        replay_check,
        arms,
        opportunities,
    };
    write_json(&paths.summary, &summary)?;
    Ok(Outcome {
        raw: paths.raw,
        summary: paths.summary,
        incomplete,
        lineage_count,
        proposal_count: summary.proposal_count,
        bytes,
        replay_check,
    })
}

/// Create the raw file and write the record header up to the open
/// `lineages` array.
fn open_stream(raw: &Path, header: &RawRecord) -> Result<Stream, String> {
    if let Some(parent) = raw.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
    }
    let file = std::fs::File::create(raw).map_err(|e| io_error(raw, &e))?;
    let mut writer = HashingWriter {
        inner: BufWriter::new(file),
        hasher: Sha256::new(),
        bytes: 0,
    };
    // The header object minus its trailing fields, then the streamed array.
    let mut head = serde_json::to_value(header).map_err(|e| e.to_string())?;
    let object = head.as_object_mut().expect("record header is an object");
    for key in [
        "lineages",
        "incomplete",
        "stop_reason",
        "lineage_count",
        "wall_secs",
    ] {
        object.remove(key);
    }
    let mut head_text = serde_json::to_string(&head).map_err(|e| e.to_string())?;
    head_text.pop(); // the closing brace
    writeln!(writer, "{head_text},\"lineages\":[").map_err(|e| io_error(raw, &e))?;
    Ok(Stream {
        writer,
        written: 0,
        facts: Vec::new(),
        error: None,
    })
}

/// Close the `lineages` array, write the footer fields, and return the
/// file's byte count and SHA-256.
fn close_stream(
    raw: &Path,
    mut stream: Stream,
    footer: &serde_json::Value,
) -> Result<(u64, String), String> {
    let mut footer_text = serde_json::to_string(footer).map_err(|e| e.to_string())?;
    footer_text.remove(0); // the opening brace
    writeln!(stream.writer, "\n],{footer_text}").map_err(|e| io_error(raw, &e))?;
    stream.writer.flush().map_err(|e| io_error(raw, &e))?;
    let HashingWriter {
        inner,
        hasher,
        bytes,
    } = stream.writer;
    drop(inner);
    Ok((bytes, format!("{:x}", hasher.finalize())))
}

/// What the parallel loop leaves behind: the stream with its per-lineage
/// facts, and whether a cap stopped it.
struct Execution {
    facts: Completed,
    stop_reason: Option<String>,
    incomplete: bool,
}

struct Completed {
    stream: Stream,
    records: Vec<(usize, LineageFacts, Opportunities)>,
}

/// Run every `(arm, batch, lineage)` task on the pool, streaming each
/// completed record; a cap sets the stop flag checked before each task.
fn execute(
    options: &Options,
    assay: &Assay,
    pool: Option<&rayon::ThreadPool>,
    stream: Stream,
    started: Instant,
) -> Result<Execution, String> {
    let sizes = assay.panel().sizes;
    let tasks: Vec<(usize, u32, u32)> = (0..assay.arm_count())
        .flat_map(|arm| {
            (0..sizes.batches).flat_map(move |batch| {
                (0..sizes.lineages).map(move |lineage| (arm, batch, lineage))
            })
        })
        .collect();
    let stream = Mutex::new(stream);
    let stop = AtomicBool::new(false);
    let stop_reason = Mutex::new(None::<String>);
    let halt = |reason: &str| {
        stop.store(true, Ordering::SeqCst);
        stop_reason
            .lock()
            .expect("stop reason lock")
            .get_or_insert_with(|| reason.into());
    };
    let work = || {
        tasks.par_iter().for_each(|&(arm, batch, lineage)| {
            if stop.load(Ordering::SeqCst) {
                return;
            }
            if started.elapsed() >= options.wall_cap {
                halt("wall_cap");
                return;
            }
            let record = assay.compact_lineage(arm, batch, lineage);
            let bytes = stream.lock().expect("stream lock").append(arm, &record);
            if bytes >= options.byte_cap {
                halt("byte_cap");
            }
        });
    };
    match pool {
        Some(pool) => pool.install(work),
        None => work(),
    }
    let mut stream = stream.into_inner().expect("stream lock");
    if let Some(error) = stream.error.take() {
        return Err(error);
    }
    let records = std::mem::take(&mut stream.facts);
    let incomplete = records.len() < tasks.len();
    Ok(Execution {
        facts: Completed { stream, records },
        stop_reason: stop_reason.into_inner().expect("stop reason lock"),
        incomplete,
    })
}

/// Per-arm summaries over the completed lineages, and the pooled
/// opportunities (whose `births` count the proposals observed).
fn summarize_arms(
    assay: &Assay,
    records: &[(usize, LineageFacts, Opportunities)],
) -> (Vec<ArmSummary>, Opportunities) {
    let batches = assay.panel().sizes.batches;
    let mut arms = Vec::with_capacity(assay.arm_count());
    let mut opportunities = Opportunities::default();
    for arm in 0..assay.arm_count() {
        let mut arm_facts = Vec::new();
        let mut pooled = Opportunities::default();
        for (_, facts, lineage) in records.iter().filter(|(index, _, _)| *index == arm) {
            arm_facts.push(facts.clone());
            pooled.merge(lineage);
        }
        // Completion order depends on the thread count; the summary does not.
        arm_facts.sort_by_key(|facts| (facts.batch, facts.lineage));
        opportunities.merge(&pooled);
        let (summary, per_batch) = summaries(&arm_facts, batches);
        let (start, policy) = assay.arm(arm);
        arms.push(ArmSummary {
            arm,
            start: start.name.clone(),
            policy,
            task: start.task,
            lineages: arm_facts.len() as u32,
            summary,
            batches: per_batch,
            opportunities: pooled,
            rows: arm_facts.iter().map(LineageRow::of).collect(),
        });
    }
    (arms, opportunities)
}

/// Read a raw record back.
///
/// # Errors
/// When the file cannot be read or is not a raw record.
pub fn read_raw(path: &Path) -> Result<RawRecord, String> {
    let file =
        std::fs::File::open(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    serde_json::from_reader(std::io::BufReader::new(file))
        .map_err(|e| format!("{} is not a recruitment record: {e}", path.display()))
}

/// Rebuild every proposal of every lineage in the raw record from the
/// initial genome, the chosen chain and the seeds, comparing fingerprints.
fn replay(
    raw: &Path,
    assay: &Assay,
    pool: Option<&rayon::ThreadPool>,
) -> Result<ReplayCheck, String> {
    let record = read_raw(raw)?;
    let work = || {
        record
            .lineages
            .par_iter()
            .map(|lineage| assay.replay(lineage))
            .reduce(ReplayCheck::default, |mut left, right| {
                left.merge(right);
                left
            })
    };
    Ok(match pool {
        Some(pool) => pool.install(work),
        None => work(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "petri recruitment test {} {}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    const TINY: Sizes = Sizes {
        batches: 1,
        lineages: 1,
        discovery: 1,
        followup: 1,
    };

    fn options(dir: &Temp, name: &str, sizes: Sizes) -> Options {
        Options {
            sizes: Some(sizes),
            raw: Some(dir.0.join(format!("{name}-raw.json"))),
            summary: Some(dir.0.join(format!("{name}-summary.json"))),
            threads: Some(1),
            source_revision: "test-revision".into(),
            ..Options::new("t13-f07-test", dir.0.clone())
        }
    }

    fn read_summary(path: &Path) -> RunSummary {
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
    }

    /// A tiny complete run writes a readable raw record and a summary whose
    /// counts, identities and replay check agree with the record.
    #[test]
    fn a_complete_run_writes_the_record_summary_and_replay_check() {
        let dir = Temp::new();
        let mut options = options(&dir, "complete", TINY);
        options.replay_check = true;
        options.pilot = true;

        let outcome = run(&options).unwrap();

        assert!(!outcome.incomplete);
        let record = read_raw(&outcome.raw).unwrap();
        assert_eq!(record.version, S0_VERSION);
        assert_eq!(record.panel, Panel::s0(TINY));
        assert_eq!(record.supply_rule, Panel::s0(TINY).supply.rule());
        assert_eq!(record.source_revision, "test-revision");
        assert_eq!(record.threads, 1);
        assert!(record.pilot);
        assert!(!record.incomplete);
        assert_eq!(record.stop_reason, None);
        assert_eq!(record.lineages.len(), 27);
        assert_eq!(record.lineage_count, 27);
        assert_eq!(
            record
                .lineages
                .iter()
                .map(|lineage| lineage.arm)
                .collect::<Vec<_>>(),
            (0..27).collect::<Vec<_>>()
        );
        assert_eq!(record.lineages[0].start, "graph_blank");
        assert_eq!(record.lineages[0].policy, Policy::Drift);
        assert_eq!(record.lineages[0].proposals.len(), 4);
        let bytes = std::fs::read(&outcome.raw).unwrap();
        assert_eq!(bytes.len() as u64, outcome.bytes);

        let summary = read_summary(&outcome.summary);
        assert_eq!(summary.kind, SUMMARY_KIND);
        assert_eq!(summary.version, S0_VERSION);
        assert_eq!(summary.feature, "t13-f07-test");
        assert!(summary.pilot);
        assert_eq!(summary.lineage_count, 27);
        assert_eq!(summary.proposal_count, 27 * 4);
        assert_eq!(summary.expected_proposals, TINY.proposals());
        assert_eq!(summary.raw.bytes, outcome.bytes);
        assert_eq!(summary.raw.sha256, format!("{:x}", Sha256::digest(&bytes)));
        assert_eq!(summary.arms.len(), 27);
        assert_eq!(summary.arms[13].arm, 13);
        assert_eq!(summary.arms[13].lineages, 1);
        assert_eq!(summary.arms[13].summary.transitions.lineages, 1);
        assert_eq!(summary.arms[13].batches.len(), 1);
        assert_eq!(summary.arms[13].rows.len(), 1);
        let row = &summary.arms[13].rows[0];
        assert_eq!((row.batch, row.lineage), (0, 0));
        assert_eq!(row.classification, record.lineages[13].classification);
        assert_eq!(row.ladder, record.lineages[13].ladder);
        assert_eq!(row.final_checkpoint.map(|c| c.generation), Some(2));
        assert_eq!(summary.opportunities.births, 27 * 4);
        assert_eq!(summary.threads, 1);
        assert!(!summary.incomplete);
        let check = summary.replay_check.unwrap();
        assert_eq!(check, outcome.replay_check.unwrap());
        assert_eq!(
            (check.proposals, check.matched, check.first_mismatch),
            (27 * 4, 27 * 4, None)
        );
    }

    /// The pilot's lineages are a prefix of the panel: the same
    /// `(arm, batch, lineage)` record is byte-identical in both runs.
    #[test]
    fn pilot_lineages_reproduce_byte_identically_inside_the_panel() {
        let dir = Temp::new();
        let pilot = run(&options(&dir, "pilot", TINY)).unwrap();
        let panel = run(&options(
            &dir,
            "panel",
            Sizes {
                batches: 2,
                lineages: 2,
                ..TINY
            },
        ))
        .unwrap();
        let pilot = read_raw(&pilot.raw).unwrap();
        let panel = read_raw(&panel.raw).unwrap();
        assert_eq!(panel.lineages.len(), 27 * 4);
        for lineage in &pilot.lineages {
            let inside = panel
                .lineages
                .iter()
                .find(|candidate| {
                    candidate.arm == lineage.arm
                        && candidate.batch == lineage.batch
                        && candidate.lineage == lineage.lineage
                })
                .unwrap();
            assert_eq!(
                serde_json::to_vec(lineage).unwrap(),
                serde_json::to_vec(inside).unwrap()
            );
        }
    }

    /// Two threads complete every lineage and summarize exactly as one thread
    /// does: the record order may differ, the per-arm rows and estimates do
    /// not.
    #[test]
    fn a_two_thread_run_summarizes_exactly_as_one_thread() {
        let dir = Temp::new();
        let sizes = Sizes {
            batches: 2,
            lineages: 2,
            ..TINY
        };
        let single = run(&options(&dir, "one", sizes)).unwrap();
        let mut parallel = options(&dir, "two", sizes);
        parallel.threads = Some(2);
        let parallel = run(&parallel).unwrap();

        assert!(!parallel.incomplete);
        assert_eq!(parallel.lineage_count, 27 * 4);
        let one = read_summary(&single.summary);
        let two = read_summary(&parallel.summary);
        assert_eq!(two.threads, 2);
        assert_eq!(
            serde_json::to_value(&one.arms).unwrap(),
            serde_json::to_value(&two.arms).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&one.opportunities).unwrap(),
            serde_json::to_value(&two.opportunities).unwrap()
        );
        // Every record is byte-identical to its single-thread twin.
        let one = read_raw(&single.raw).unwrap();
        let two = read_raw(&parallel.raw).unwrap();
        for lineage in &two.lineages {
            let twin = one
                .lineages
                .iter()
                .find(|candidate| {
                    (candidate.arm, candidate.batch, candidate.lineage)
                        == (lineage.arm, lineage.batch, lineage.lineage)
                })
                .unwrap();
            assert_eq!(
                serde_json::to_vec(lineage).unwrap(),
                serde_json::to_vec(twin).unwrap()
            );
        }
        let mut capped = options(&dir, "capped-two", sizes);
        capped.threads = Some(2);
        capped.byte_cap = 1;
        let capped = run(&capped).unwrap();
        assert!(capped.incomplete);
        assert!(capped.lineage_count >= 1);
        assert!(read_summary(&capped.summary).incomplete);
    }

    /// A byte cap stops the run between lineages: the lineage that crossed
    /// it is kept, nothing after it runs, and both artifacts say so.
    #[test]
    fn a_byte_cap_stops_between_lineages_and_marks_the_record_incomplete() {
        let dir = Temp::new();
        let mut options = options(&dir, "capped", TINY);
        options.byte_cap = 1;

        let outcome = run(&options).unwrap();

        assert!(outcome.incomplete);
        assert_eq!(outcome.lineage_count, 1);
        let record = read_raw(&outcome.raw).unwrap();
        assert!(record.incomplete);
        assert_eq!(record.stop_reason.as_deref(), Some("byte_cap"));
        assert_eq!(record.lineages.len(), 1);
        assert_eq!(record.lineage_count, 1);
        let summary = read_summary(&outcome.summary);
        assert!(summary.incomplete);
        assert_eq!(summary.stop_reason.as_deref(), Some("byte_cap"));
        assert_eq!(summary.lineage_count, 1);
        assert_eq!(summary.proposal_count, 4);
        assert_eq!(summary.arms[0].lineages, 1);
        assert_eq!(summary.arms[1].lineages, 0);
        assert_eq!(summary.arms[1].summary.transitions.lineages, 0);
    }

    /// A zero wall cap stops before the first lineage: the record is empty
    /// and incomplete, and nothing is dropped because nothing completed.
    #[test]
    fn a_wall_cap_stops_before_any_lineage_starts() {
        let dir = Temp::new();
        let mut options = options(&dir, "wall", TINY);
        options.wall_cap = Duration::ZERO;

        let outcome = run(&options).unwrap();

        assert!(outcome.incomplete);
        assert_eq!(outcome.lineage_count, 0);
        let record = read_raw(&outcome.raw).unwrap();
        assert!(record.lineages.is_empty());
        assert_eq!(record.stop_reason.as_deref(), Some("wall_cap"));
        assert!(read_summary(&outcome.summary).incomplete);
    }

    #[test]
    fn default_paths_follow_the_feature_and_pilot_naming() {
        let root = crate::bench::artifacts::output_paths_with_suffix(
            Path::new("."),
            "recruitment-s0-pilot",
            "-s0-pilot",
            Some("t13-f07-x"),
            None,
            None,
        )
        .unwrap();
        assert!(root
            .raw
            .ends_with(".bench-artifacts/t13-f07-x/recruitment-s0-pilot.json"));
        assert!(root
            .summary
            .ends_with("docs/progress/features/t13-f07-x-s0-pilot.json"));
        let panel = crate::bench::artifacts::output_paths_with_suffix(
            Path::new("."),
            "recruitment-s0",
            "-s0",
            Some("t13-f07-x"),
            None,
            None,
        )
        .unwrap();
        assert!(panel
            .raw
            .ends_with(".bench-artifacts/t13-f07-x/recruitment-s0.json"));
        assert!(panel
            .summary
            .ends_with("docs/progress/features/t13-f07-x-s0.json"));
        assert_eq!(
            Options::new("x", PathBuf::from(".")).panel().sizes,
            Sizes::S0
        );
        assert_eq!(
            Options {
                pilot: true,
                ..Options::new("x", PathBuf::from("."))
            }
            .panel()
            .sizes,
            Sizes::S0_PILOT
        );
    }
}
