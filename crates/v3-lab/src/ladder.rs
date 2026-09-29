//! The why-not ladder (T22.F04): for every evolving arm, five rungs —
//! exposure, supply, viability, benefit, retention — each `pass`, `fail` or
//! `inconclusive` within a replicate, then a tally verdict over completed
//! replicates naming the first rung that is not `pass` and the track that
//! owns it. The ladder draws no random number and never feeds selection.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::arena::{Policy, Role};
use crate::summary::{label, Calibration, GenomeRecord, GenomeSource, Summary, Verdict};

pub mod track;

pub use track::{
    motor_node, relevant_family, relevant_family_labels, Birth, Change, Member, Pending, Sites,
    Tracker,
};

/// `--retention-depth` default.
pub const RETENTION_DEPTH: u32 = 2;
/// `--retention-depth` bounds.
pub const RETENTION_DEPTHS: std::ops::RangeInclusive<u32> = 1..=8;

/// The predeclared stall rates `ρ` of the counted rungs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StallRates {
    pub supply: f64,
    pub viability: f64,
    pub benefit: f64,
    pub retention: f64,
}

impl Default for StallRates {
    fn default() -> Self {
        Self {
            supply: 0.01,
            viability: 0.05,
            benefit: 0.05,
            retention: 0.20,
        }
    }
}

impl StallRates {
    /// `supply,viability,benefit,retention`, each in (0, 1).
    #[must_use]
    pub fn from_slice(rates: &[f64]) -> Option<Self> {
        let &[supply, viability, benefit, retention] = rates else {
            return None;
        };
        let rates = Self {
            supply,
            viability,
            benefit,
            retention,
        };
        rates.valid().then_some(rates)
    }

    /// Every rate in (0, 1).
    #[must_use]
    pub fn valid(&self) -> bool {
        [self.supply, self.viability, self.benefit, self.retention]
            .iter()
            .all(|rate| *rate > 0.0 && *rate < 1.0)
    }

    /// `ρ` of a counted rung; `None` for exposure.
    #[must_use]
    pub fn rate(&self, rung: Rung) -> Option<f64> {
        match rung {
            Rung::Exposure => None,
            Rung::Supply => Some(self.supply),
            Rung::Viability => Some(self.viability),
            Rung::Benefit => Some(self.benefit),
            Rung::Retention => Some(self.retention),
        }
    }
}

/// A rung of the ladder, in ladder order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rung {
    Exposure,
    Supply,
    Viability,
    Benefit,
    Retention,
}

impl Rung {
    /// The rungs read from counts, in ladder order.
    pub const COUNTED: [Self; 4] = [
        Self::Supply,
        Self::Viability,
        Self::Benefit,
        Self::Retention,
    ];

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Exposure => "exposure",
            Self::Supply => "supply",
            Self::Viability => "viability",
            Self::Benefit => "benefit",
            Self::Retention => "retention",
        }
    }

    /// The track that owns a stall here.
    #[must_use]
    pub fn route(self) -> &'static str {
        match self {
            Self::Exposure => "the assay (T22): instrument",
            Self::Supply => "T11 / T13",
            Self::Viability => "T11 / T17",
            Self::Benefit => "the scorer (T22) or, in the world, the pressure tracks",
            Self::Retention => "T13 / T14",
        }
    }
}

/// A rung's status within a replicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pass,
    Fail,
    Inconclusive,
}

impl Status {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Inconclusive => "inconclusive",
        }
    }
}

/// The rule-of-three adequacy minimum `⌈3 / ρ⌉`: the count at which zero
/// successes bound the rate below `ρ`.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn adequacy(rate: f64) -> u64 {
    (3.0 / rate).ceil() as u64
}

/// `inconclusive` below the adequacy minimum, else `pass` when `s / n ≥ ρ`
/// and `fail` below it. Counts are descriptive, never interval samples.
#[must_use]
pub fn status(successes: u64, trials: u64, rate: f64) -> Status {
    if trials < adequacy(rate) {
        Status::Inconclusive
    } else {
        #[allow(clippy::cast_precision_loss)]
        let share = successes as f64 / trials as f64;
        if share >= rate {
            Status::Pass
        } else {
            Status::Fail
        }
    }
}

/// Counts keyed by operator (`none` for an event no operator owned) or by
/// skip reason; only keys that fired.
pub type Keyed = BTreeMap<String, u64>;

fn add_keyed(into: &mut Keyed, from: &Keyed) {
    for (key, count) in from {
        *into.entry(key.clone()).or_default() += count;
    }
}

/// Events whose first target is a relevant site of the parent.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Targeted {
    pub requested: Keyed,
    pub applied: Keyed,
    pub skipped: Keyed,
    pub skipped_by_reason: Keyed,
}

impl Targeted {
    pub fn add(&mut self, other: &Self) {
        add_keyed(&mut self.requested, &other.requested);
        add_keyed(&mut self.applied, &other.applied);
        add_keyed(&mut self.skipped, &other.skipped);
        add_keyed(&mut self.skipped_by_reason, &other.skipped_by_reason);
    }
}

/// The supply rung's counts over births.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Supply {
    pub births: u64,
    pub touching_births: u64,
    /// Summed parent relevant sites and reachable nodes (means divide by
    /// `births`).
    pub sites: u64,
    pub reachable: u64,
    /// `Σ applied_b × sites_b / reachable_b`, a uniform-targeting reference
    /// (targeting is executed-biased); null with a zero-reachable parent or
    /// no birth.
    pub uniform_reference: Option<f64>,
    pub targeted: Targeted,
    pub discarded: u64,
    pub created: u64,
    pub removed: u64,
}

impl Supply {
    /// Pool another block in.
    pub fn add(&mut self, other: &Self) {
        if other.births == 0 {
            return;
        }
        self.uniform_reference = if self.births == 0 {
            other.uniform_reference
        } else {
            self.uniform_reference
                .zip(other.uniform_reference)
                .map(|(a, b)| a + b)
        };
        self.births += other.births;
        self.touching_births += other.touching_births;
        self.sites += other.sites;
        self.reachable += other.reachable;
        self.targeted.add(&other.targeted);
        self.discarded += other.discarded;
        self.created += other.created;
        self.removed += other.removed;
    }
}

/// Child-versus-parent outcomes on the same scenes.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Outcomes {
    /// `scalar_c > scalar_p`.
    pub improved: u64,
    /// Mean `progress` strictly greater.
    pub progress_improved: u64,
    pub worse: u64,
    /// `Σ (scalar_c − scalar_p)`.
    pub delta_sum: f64,
    /// Largest delta among the improved.
    pub delta_max: Option<f64>,
}

impl Outcomes {
    /// Fold one child in.
    pub fn add(&mut self, child: f64, parent: f64, progress_improved: bool) {
        let delta = child - parent;
        if child > parent {
            self.improved += 1;
            self.delta_max = Some(self.delta_max.map_or(delta, |max| max.max(delta)));
        }
        self.worse += u64::from(child < parent);
        self.progress_improved += u64::from(progress_improved);
        self.delta_sum += delta;
    }

    fn merge(&mut self, other: &Self) {
        self.improved += other.improved;
        self.progress_improved += other.progress_improved;
        self.worse += other.worse;
        self.delta_sum += other.delta_sum;
        self.delta_max = match (self.delta_max, other.delta_max) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        };
    }
}

/// Touching non-identical children: battery classes over all, outcomes
/// over the viable ones.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TouchingChildren {
    pub children: u64,
    pub scene_changed: u64,
    pub silent: u64,
    pub changed: u64,
    pub dead: u64,
    /// Class not `dead` and (`changed` or `scene_changed`).
    pub viable: u64,
    /// Over the viable children.
    #[serde(flatten)]
    pub outcomes: Outcomes,
}

/// Non-touching non-identical children: context only.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OtherChildren {
    pub children: u64,
    pub scene_changed: u64,
    #[serde(flatten)]
    pub outcomes: Outcomes,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Children {
    pub touching: TouchingChildren,
    pub other: OtherChildren,
}

impl Children {
    fn add(&mut self, other: &Self) {
        let (t, o) = (&mut self.touching, &other.touching);
        t.children += o.children;
        t.scene_changed += o.scene_changed;
        t.silent += o.silent;
        t.changed += o.changed;
        t.dead += o.dead;
        t.viable += o.viable;
        t.outcomes.merge(&o.outcomes);
        self.other.children += other.other.children;
        self.other.scene_changed += other.other.scene_changed;
        self.other.outcomes.merge(&other.other.outcomes);
    }
}

/// `[retained, deleted, lineage_loss]`.
pub type Resolved = [u64; 3];

/// A row's retention: selected improvements this generation and the depths
/// resolved in it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionRow {
    pub selected: u64,
    pub selected_touching: u64,
    /// Per depth `1..=D`.
    pub depths: Vec<Resolved>,
    pub depths_touching: Vec<Resolved>,
}

impl RetentionRow {
    #[must_use]
    pub fn new(depth: usize) -> Self {
        Self {
            depths: vec![[0; 3]; depth],
            depths_touching: vec![[0; 3]; depth],
            ..Self::default()
        }
    }
}

/// A row's `ladder` block.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LadderRow {
    pub supply: Supply,
    pub children: Children,
    pub retention: RetentionRow,
}

/// Pooled retention, with the depths censored at the stop.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Retention {
    pub selected: u64,
    pub selected_touching: u64,
    pub depths: Vec<Resolved>,
    pub depths_touching: Vec<Resolved>,
    pub censored: Vec<u64>,
    pub censored_touching: Vec<u64>,
}

fn add_resolved(into: &mut [Resolved], from: &[Resolved]) {
    for (into, from) in into.iter_mut().zip(from) {
        for (a, b) in into.iter_mut().zip(from) {
            *a += b;
        }
    }
}

fn add_counts(into: &mut [u64], from: &[u64]) {
    for (a, b) in into.iter_mut().zip(from) {
        *a += b;
    }
}

/// Pooled counts: over a replicate's written rows, or over an arm's
/// completed replicates.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Pooled {
    pub supply: Supply,
    pub children: Children,
    pub retention: Retention,
}

impl Pooled {
    #[must_use]
    pub fn new(depth: usize) -> Self {
        Self {
            retention: Retention {
                depths: vec![[0; 3]; depth],
                depths_touching: vec![[0; 3]; depth],
                censored: vec![0; depth],
                censored_touching: vec![0; depth],
                ..Retention::default()
            },
            ..Self::default()
        }
    }

    /// Fold a written row in.
    pub fn add(&mut self, row: &LadderRow) {
        self.supply.add(&row.supply);
        self.children.add(&row.children);
        let r = &mut self.retention;
        r.selected += row.retention.selected;
        r.selected_touching += row.retention.selected_touching;
        add_resolved(&mut r.depths, &row.retention.depths);
        add_resolved(&mut r.depths_touching, &row.retention.depths_touching);
    }

    /// Pool another replicate in.
    pub fn merge(&mut self, other: &Self) {
        self.supply.add(&other.supply);
        self.children.add(&other.children);
        let (r, o) = (&mut self.retention, &other.retention);
        r.selected += o.selected;
        r.selected_touching += o.selected_touching;
        add_resolved(&mut r.depths, &o.depths);
        add_resolved(&mut r.depths_touching, &o.depths_touching);
        add_counts(&mut r.censored, &o.censored);
        add_counts(&mut r.censored_touching, &o.censored_touching);
    }

    /// `(s, n)` of a counted rung; retention reads depth `D`.
    #[must_use]
    pub fn counts(&self, rung: Rung) -> Option<(u64, u64)> {
        let t = &self.children.touching;
        match rung {
            Rung::Exposure => None,
            Rung::Supply => Some((self.supply.touching_births, self.supply.births)),
            Rung::Viability => Some((t.viable, t.children)),
            Rung::Benefit => Some((t.outcomes.improved, t.viable)),
            Rung::Retention => self
                .retention
                .depths
                .last()
                .map(|[retained, deleted, lost]| (*retained, retained + deleted + lost)),
        }
    }

    /// The counted rungs' statuses.
    #[must_use]
    pub fn statuses(&self, rates: &StallRates) -> Vec<RungStatus> {
        Rung::COUNTED
            .iter()
            .map(|&rung| {
                let (successes, trials) = self.counts(rung).unwrap_or((0, 0));
                let rate = rates.rate(rung).expect("counted rungs have a rate");
                RungStatus {
                    rung,
                    status: status(successes, trials, rate),
                    successes,
                    trials,
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RungStatus {
    pub rung: Rung,
    pub status: Status,
    pub successes: u64,
    pub trials: u64,
}

/// One replicate's ladder (exposure passed: a campaign ran).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplicateLadder {
    pub replicate: u32,
    pub incomplete: bool,
    pub pooled: Pooled,
    pub statuses: Vec<RungStatus>,
    /// The first rung whose status is not `pass`; null when all pass.
    pub first_not_pass: Option<Rung>,
}

impl ReplicateLadder {
    #[must_use]
    pub fn new(replicate: u32, incomplete: bool, pooled: Pooled, rates: &StallRates) -> Self {
        let statuses = pooled.statuses(rates);
        let first_not_pass = statuses
            .iter()
            .find(|s| s.status != Status::Pass)
            .map(|s| s.rung);
        Self {
            replicate,
            incomplete,
            pooled,
            statuses,
            first_not_pass,
        }
    }

    fn status(&self, rung: Rung) -> Option<Status> {
        self.statuses
            .iter()
            .find(|s| s.rung == rung)
            .map(|s| s.status)
    }
}

/// Per rung: the replicate-status tally over completed replicates, with
/// `wilson_95` on the `fail` share (the replicate is the interval sample).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tally {
    pub rung: Rung,
    pub pass: u32,
    pub fail: u32,
    pub inconclusive: u32,
    pub fail_wilson_95: Option<[f64; 2]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerdictKind {
    /// More than half the replicates naming the verdict rung `fail` there.
    Stalls,
    /// At the verdict rung without a `fail` majority, or no completed
    /// replicate (`rung` null).
    Inconclusive,
    /// Every completed replicate passes every rung.
    NoStall,
}

/// The arm's tally verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmVerdict {
    pub kind: VerdictKind,
    pub rung: Option<Rung>,
    /// Among the completed replicates naming `rung`.
    pub fail: u32,
    pub inconclusive: u32,
    pub of: u32,
    /// `no stall` only: completed replicates that reached.
    pub reached: Option<u32>,
    pub completed: u32,
    pub incomplete: u32,
}

fn count_u32(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

impl ArmVerdict {
    /// From the replicates and, per replicate, whether it reached.
    #[must_use]
    pub fn of(replicates: &[ReplicateLadder], reached: &[Option<bool>]) -> Self {
        let completed: Vec<(&ReplicateLadder, Option<bool>)> = replicates
            .iter()
            .zip(reached.iter().copied().chain(std::iter::repeat(None)))
            .filter(|(r, _)| !r.incomplete)
            .collect();
        let incomplete = count_u32(replicates.len() - completed.len());
        let base = Self {
            kind: VerdictKind::Inconclusive,
            rung: None,
            fail: 0,
            inconclusive: 0,
            of: 0,
            reached: None,
            completed: count_u32(completed.len()),
            incomplete,
        };
        if completed.is_empty() {
            return base;
        }
        let mut modal: Option<(Rung, usize)> = None;
        for rung in Rung::COUNTED {
            let naming = completed
                .iter()
                .filter(|(r, _)| r.first_not_pass == Some(rung))
                .count();
            if naming > 0 && modal.is_none_or(|(_, best)| naming > best) {
                modal = Some((rung, naming));
            }
        }
        let Some((rung, _)) = modal else {
            return Self {
                kind: VerdictKind::NoStall,
                reached: Some(count_u32(
                    completed.iter().filter(|(_, r)| *r == Some(true)).count(),
                )),
                ..base
            };
        };
        let naming: Vec<&ReplicateLadder> = completed
            .iter()
            .map(|(r, _)| *r)
            .filter(|r| r.first_not_pass == Some(rung))
            .collect();
        let fail = count_u32(
            naming
                .iter()
                .filter(|r| r.status(rung) == Some(Status::Fail))
                .count(),
        );
        let of = count_u32(naming.len());
        Self {
            kind: if 2 * fail > of {
                VerdictKind::Stalls
            } else {
                VerdictKind::Inconclusive
            },
            rung: Some(rung),
            fail,
            inconclusive: of - fail,
            of,
            ..base
        }
    }

    /// `stalls at benefit: fail 3 / inconclusive 1 of 4`, …
    #[must_use]
    pub fn text(&self) -> String {
        let partial = if self.incomplete > 0 {
            format!(", partial ({} incomplete)", self.incomplete)
        } else {
            String::new()
        };
        match (self.kind, self.rung) {
            (VerdictKind::NoStall, _) => {
                let reached = self.reached.unwrap_or(0);
                let censored = if reached == 0 {
                    ", censored at the horizon"
                } else {
                    ""
                };
                format!(
                    "no stall: reached {reached}/{}{censored}{partial}",
                    self.completed
                )
            }
            (_, None) => format!("inconclusive, partial ({} incomplete)", self.incomplete),
            (kind, Some(rung)) => format!(
                "{} at {}: fail {} / inconclusive {} of {}{partial}",
                if kind == VerdictKind::Stalls {
                    "stalls"
                } else {
                    "inconclusive"
                },
                rung.name(),
                self.fail,
                self.inconclusive,
                self.of
            ),
        }
    }

    /// The owning track of the verdict rung.
    #[must_use]
    pub fn route(&self) -> Option<&'static str> {
        match self.kind {
            VerdictKind::NoStall => None,
            _ => self.rung.map(Rung::route),
        }
    }
}

/// One evolving arm's ladder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArmLadder {
    pub name: String,
    pub role: Role,
    pub policy: Policy,
    /// A `policy-deviation` arm: printed as a diagnostic, never as the
    /// capability's verdict.
    pub diagnostic: bool,
    pub replicates: Vec<ReplicateLadder>,
    pub tallies: Vec<Tally>,
    pub verdict: ArmVerdict,
    /// Over completed replicates; descriptive, no interval.
    pub pooled: Pooled,
    pub incomplete_replicates: u32,
}

impl ArmLadder {
    #[must_use]
    pub fn new(
        name: String,
        role: Role,
        policy: Policy,
        replicates: Vec<ReplicateLadder>,
        reached: &[Option<bool>],
        depth: usize,
    ) -> Self {
        let completed: Vec<&ReplicateLadder> =
            replicates.iter().filter(|r| !r.incomplete).collect();
        let tallies = Rung::COUNTED
            .iter()
            .map(|&rung| {
                let count = |status: Status| {
                    count_u32(
                        completed
                            .iter()
                            .filter(|r| r.status(rung) == Some(status))
                            .count(),
                    )
                };
                let (pass, fail, inconclusive) = (
                    count(Status::Pass),
                    count(Status::Fail),
                    count(Status::Inconclusive),
                );
                Tally {
                    rung,
                    pass,
                    fail,
                    inconclusive,
                    fail_wilson_95: crate::stats::wilson_95(fail, pass + fail + inconclusive),
                }
            })
            .collect();
        let mut pooled = Pooled::new(depth);
        for replicate in &completed {
            pooled.merge(&replicate.pooled);
        }
        let verdict = ArmVerdict::of(&replicates, reached);
        Self {
            name,
            role,
            policy,
            diagnostic: policy == Policy::PolicyDeviation,
            incomplete_replicates: verdict.incomplete,
            replicates,
            tallies,
            verdict,
            pooled,
        }
    }
}

/// A calibration point that failed a check, with the counts behind it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailingPoint {
    pub food_fraction: Option<f64>,
    pub scale: Option<u8>,
    pub lifetime: u32,
    pub scenes: u32,
    /// Scenes drawn before the infeasible draw (all when exposed).
    pub exposure_scenes: u32,
    pub exposure: bool,
    pub competence: bool,
    pub comparator_wins: Option<u32>,
    pub wins_needed: u32,
    /// `comparator − floor` against the margin.
    pub comparator_gap: Option<f64>,
    pub sensitivity: bool,
    /// `half − floor`, `oracle − half`, comparator − floor progress.
    pub sensitivity_gaps: Option<[f64; 3]>,
    pub validation_competence: Option<bool>,
}

/// The selected calibration point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedPoint {
    pub food_fraction: Option<f64>,
    pub scale: Option<u8>,
    pub lifetime: u32,
}

/// The exposure rung, from the calibration block alone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Exposure {
    pub status: Status,
    pub verdict: Verdict,
    /// Whether a campaign ran (`--calibrate-only` stops here).
    pub campaign: bool,
    pub selected: Option<SelectedPoint>,
    /// Uncalibrated only: every point with the checks it failed.
    pub failing: Vec<FailingPoint>,
    pub margin: f64,
}

impl Exposure {
    #[must_use]
    pub fn of(calibration: &Calibration, campaign: bool) -> Self {
        let calibrated = calibration.verdict == Verdict::Calibrated;
        let failing = if calibrated {
            Vec::new()
        } else {
            calibration
                .points
                .iter()
                .map(|point| {
                    let means = point.means.as_ref();
                    let drawn = count_u32(point.redraws.len());
                    FailingPoint {
                        food_fraction: point.food_fraction,
                        scale: point.scale,
                        lifetime: point.lifetime,
                        scenes: point.scenes,
                        exposure_scenes: if point.exposure {
                            drawn
                        } else {
                            drawn.saturating_sub(1)
                        },
                        exposure: point.exposure,
                        competence: point.competence,
                        comparator_wins: means.map(|m| m.comparator_wins),
                        wins_needed: (3 * point.scenes).div_ceil(4),
                        comparator_gap: means.map(|m| m.comparator - m.floor),
                        sensitivity: point.sensitivity,
                        sensitivity_gaps: means.map(|m| {
                            [
                                m.half - m.floor,
                                m.oracle - m.half,
                                m.comparator_progress - m.floor_progress,
                            ]
                        }),
                        validation_competence: point.validation_competence,
                    }
                })
                .collect()
        };
        Self {
            status: if calibrated {
                Status::Pass
            } else {
                Status::Fail
            },
            verdict: calibration.verdict,
            campaign,
            selected: calibration.selected.as_ref().map(|s| SelectedPoint {
                food_fraction: s.food_fraction,
                scale: s.scale,
                lifetime: s.lifetime,
            }),
            failing,
            margin: calibration.margin,
        }
    }
}

/// The summary's top-level `ladder`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ladder {
    pub exposure: Exposure,
    /// Evolving arms only.
    pub arms: Vec<ArmLadder>,
}

fn fmt_opt(value: Option<f64>) -> String {
    value.map_or_else(|| "null".to_owned(), |v| format!("{v:.3}"))
}

/// `s/n = share`, `null` at `n = 0`.
fn share(successes: u64, trials: u64) -> String {
    if trials == 0 {
        format!("{successes}/0 = null")
    } else {
        #[allow(clippy::cast_precision_loss)]
        let value = successes as f64 / trials as f64;
        format!("{successes}/{trials} = {value:.3}")
    }
}

fn keyed(map: &Keyed) -> String {
    if map.is_empty() {
        return "{}".to_owned();
    }
    let parts: Vec<String> = map.iter().map(|(k, v)| format!("{k} {v}")).collect();
    format!("{{{}}}", parts.join(", "))
}

#[allow(clippy::cast_precision_loss)]
fn mean_of(sum: f64, n: u64) -> Option<f64> {
    (n > 0).then(|| sum / n as f64)
}

fn axis(food_fraction: Option<f64>, scale: Option<u8>) -> String {
    match (food_fraction, scale) {
        (Some(fraction), _) => format!("fraction {fraction}"),
        (_, Some(scale)) => format!("scale {scale}"),
        _ => "layout".to_owned(),
    }
}

fn exposure_lines(out: &mut String, exposure: &Exposure) {
    match (&exposure.selected, exposure.status) {
        (Some(selected), Status::Pass) => {
            let _ = writeln!(
                out,
                "- exposure: pass (calibrated at {} lifetime {}){}",
                axis(selected.food_fraction, selected.scale),
                selected.lifetime,
                if exposure.campaign {
                    ""
                } else {
                    "; no campaign"
                }
            );
        }
        _ => {
            let _ = writeln!(out, "- exposure: fail (uncalibrated)");
            for point in &exposure.failing {
                let mut failed = Vec::new();
                if !point.exposure {
                    failed.push(format!(
                        "exposure ({}/{} scenes drawn)",
                        point.exposure_scenes, point.scenes
                    ));
                }
                if point.exposure && !point.competence {
                    failed.push(format!(
                        "competence (comparator wins {}/{} needed {}, gap {} vs margin {})",
                        point
                            .comparator_wins
                            .map_or("null".into(), |w| w.to_string()),
                        point.scenes,
                        point.wins_needed,
                        fmt_opt(point.comparator_gap),
                        exposure.margin
                    ));
                }
                if point.exposure && !point.sensitivity {
                    let gaps = point.sensitivity_gaps.map_or("null".into(), |[a, b, c]| {
                        format!("half−floor {a:.3}, oracle−half {b:.3}, progress {c:.3}")
                    });
                    failed.push(format!("sensitivity ({gaps})"));
                }
                if point.validation_competence == Some(false) {
                    failed.push("validation competence".to_owned());
                }
                let _ = writeln!(
                    out,
                    "  - {} lifetime {}: {}",
                    axis(point.food_fraction, point.scale),
                    point.lifetime,
                    if failed.is_empty() {
                        "no check failed".to_owned()
                    } else {
                        failed.join("; ")
                    }
                );
            }
        }
    }
}

fn rung_context(pooled: &Pooled, rung: Rung) -> String {
    let t = &pooled.children.touching;
    let o = &pooled.children.other;
    let s = &pooled.supply;
    let r = &pooled.retention;
    #[allow(clippy::cast_precision_loss)]
    let mean = |sum: u64| mean_of(sum as f64, s.births);
    match rung {
        Rung::Exposure => String::new(),
        Rung::Supply => format!(
            "targeted requested {}, applied {}, skipped {}, skipped by reason {}; discarded {}, created {}, removed {}; mean sites {}, mean reachable {}; uniform reference {}",
            keyed(&s.targeted.requested),
            keyed(&s.targeted.applied),
            keyed(&s.targeted.skipped),
            keyed(&s.targeted.skipped_by_reason),
            s.discarded,
            s.created,
            s.removed,
            fmt_opt(mean(s.sites)),
            fmt_opt(mean(s.reachable)),
            fmt_opt(s.uniform_reference),
        ),
        Rung::Viability => format!(
            "silent {}, changed {}, dead {}, scene_changed {}",
            t.silent, t.changed, t.dead, t.scene_changed
        ),
        Rung::Benefit => format!(
            "progress_improved {}, worse {}, delta_mean {}, delta_max {}; non-touching improved {}",
            t.outcomes.progress_improved,
            t.outcomes.worse,
            fmt_opt(mean_of(t.outcomes.delta_sum, t.viable)),
            fmt_opt(t.outcomes.delta_max),
            share(o.outcomes.improved, o.children),
        ),
        Rung::Retention => {
            let depths = |rows: &[Resolved], censored: &[u64]| -> String {
                rows.iter()
                    .zip(censored)
                    .enumerate()
                    .map(|(d, ([a, b, c], x))| {
                        format!(
                            "d{} retained {a} deleted {b} lineage_loss {c} censored {x}",
                            d + 1
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            };
            format!(
                "selected {} ({} touching); all: {}; touching: {}",
                r.selected,
                r.selected_touching,
                depths(&r.depths, &r.censored),
                depths(&r.depths_touching, &r.censored_touching),
            )
        }
    }
}

/// The start genome's label: `founder`, or a file's basename and SHA-256.
/// An arm's own genome file (`arm:<name>`) labels that arm; otherwise the
/// run's `start`.
fn start_label(summary: &Summary, arm: Option<&str>) -> String {
    let genomes = &summary.provenance.genomes;
    let named = |name: &str| genomes.iter().find(|g| g.name == name);
    let Some(start) = arm
        .and_then(|arm| named(&GenomeRecord::arm_name(arm)))
        .or_else(|| named("start"))
    else {
        return "unknown".to_owned();
    };
    match (start.source, &start.file) {
        (Some(GenomeSource::File), Some(file)) => format!("{file} (sha256 {})", start.sha256),
        (Some(GenomeSource::Founder), _) => format!("founder (sha256 {})", start.sha256),
        _ => format!("unknown (sha256 {})", start.sha256),
    }
}

/// The ladder section of a v4 summary: exposure, then per evolving arm one
/// line per rung and the verdict line. Empty for older summaries.
#[must_use]
pub fn render(summary: &Summary) -> String {
    let mut out = String::new();
    let Some(ladder) = &summary.ladder else {
        return out;
    };
    let rates = summary.provenance.sizes.stall_rates.unwrap_or_default();
    let assay = summary.assay.name();
    let arena = summary.provenance.arena.spec["id"]
        .as_str()
        .unwrap_or("unknown")
        .to_owned();
    let start = start_label(summary, None);
    let _ = writeln!(out, "\n## Why-not ladder\n");
    exposure_lines(&mut out, &ladder.exposure);
    if ladder.exposure.status != Status::Pass {
        let _ = writeln!(
            out,
            "\nverdict: {assay}, arena {arena}, start {start}: stalls at exposure (uncalibrated) -> route: {}",
            Rung::Exposure.route()
        );
        return out;
    }
    if !ladder.exposure.campaign {
        let _ = writeln!(
            out,
            "\nverdict: {assay}, arena {arena}, start {start}: no campaign"
        );
        return out;
    }
    for arm in &ladder.arms {
        let note = if arm.diagnostic {
            " (diagnostic: policy-deviation, not the capability's verdict)"
        } else {
            ""
        };
        let _ = writeln!(out, "\n### {}{note}\n", arm.name);
        let _ = writeln!(out, "- exposure: pass");
        for tally in &arm.tallies {
            let rung = tally.rung;
            let rate = rates.rate(rung).unwrap_or(0.0);
            let (s, n) = arm.pooled.counts(rung).unwrap_or((0, 0));
            let statuses: Vec<String> = arm
                .replicates
                .iter()
                .map(|r| {
                    let status = r.status(rung).map_or("-", Status::name);
                    if r.incomplete {
                        format!("r{} {status} (incomplete)", r.replicate)
                    } else {
                        format!("r{} {status}", r.replicate)
                    }
                })
                .collect();
            let _ = writeln!(
                out,
                "- {}: pooled {} (ρ {rate}, adequacy n ≥ {}); replicates {}; tally pass {} / fail {} / inconclusive {}, fail share Wilson 95% {}; {}",
                rung.name(),
                share(s, n),
                adequacy(rate),
                statuses.join(", "),
                tally.pass,
                tally.fail,
                tally.inconclusive,
                tally
                    .fail_wilson_95
                    .map_or("-".into(), |[lo, hi]| format!("[{lo:.3}, {hi:.3}]")),
                rung_context(&arm.pooled, rung),
            );
        }
        let _ = writeln!(
            out,
            "\nverdict: {assay}, arena {arena}, start {}, arm {}, role {}, policy {}{}: {} -> route: {}",
            start_label(summary, Some(&arm.name)),
            arm.name,
            label(arm.role),
            label(arm.policy),
            if arm.diagnostic { " (diagnostic)" } else { "" },
            arm.verdict.text(),
            arm.verdict.route().unwrap_or("none"),
        );
    }
    out
}

#[cfg(test)]
mod tests;
