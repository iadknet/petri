//! Learned state (T25.F06, T25.F07, T25.F12): per-learner learned values and
//! eligibility traces, built once at birth from the genome's learning layout
//! and the parent's seeds.
//!
//! [`LearnedState::for_genome`] builds, per mesh node, the node's
//! [`LearningLayout`] once and stores per edge learner and per scale learner
//! its configuration, value, birth value, trace and seeded flag, with the
//! dynamic edges (every edge learner and every member reading a learned
//! scale) in canonical order. A genome whose layouts are all empty allocates
//! nothing and runs the static loops. A learner is initialized once its
//! backend has been visited or when it was seeded at birth; only initialized
//! ones are updated by reward, captured for a child and exposed by the view.
//!
//! Every learned value starts at its birth value: the clamped seed when
//! seeded, else its genomic value clamped to its bound (T25.F09 invariant
//! 2). Once per world tick each initialized one whose retention is below 1
//! relaxes toward it (invariant 3).

use super::rules::{activity, hebbian_delta, relax, three_factor};
use super::{OutcomeSignalBank, PlasticityKind};
use crate::contracts::InputReference;
use crate::genome::cgp::{
    compose, BirthSeeds, BundleId, CgpGraphBackendDef, EdgeOccurrence, EdgeSurface,
};
use crate::genome::learning::{LearnerSource, LearningLayout, MemberTerm};
use crate::genome::{retention_decays, BackendDef, CreatureGenome, Learning};

/// Where a member's bundle term reads its scale.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ScaleRead {
    /// The bundle has no scale learner: its genomic scale.
    Genomic(f32),
    /// The bundle's scale learner, by index.
    Learned(usize),
}

/// A member's bundle term: its pattern and its scale's source.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Term {
    pattern: f32,
    scale: ScaleRead,
}

/// One edge learner's configuration and state.
#[derive(Debug, Clone, Copy, PartialEq)]
struct EdgeLearner {
    occurrence: EdgeOccurrence,
    config: Learning,
    source: LearnerSource,
    /// The genomic own weight: a lone edge's weight, a member's modifier.
    genomic: f32,
    /// The clamped seed or the clamped genomic own weight.
    birth: f32,
    /// The learned own quantity.
    value: f32,
    /// The eligibility trace; 0 and unused when pure Hebbian.
    trace: f32,
    seeded: bool,
    /// The member's bundle term, `None` for a lone edge.
    term: Option<Term>,
    /// The learner's position among the dynamic edges (its samples).
    dynamic: usize,
}

/// One member feeding a scale: its pattern and its position among the
/// dynamic edges (its samples and its applied weight).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Contributor {
    pattern: f32,
    dynamic: usize,
}

/// One scale learner's configuration and state.
#[derive(Debug, Clone, PartialEq)]
struct ScaleLearner {
    bundle: BundleId,
    config: Learning,
    /// The genomic scale.
    genomic: f32,
    /// The clamped seed or the clamped genomic scale.
    birth: f32,
    /// The learned scale.
    value: f32,
    /// The eligibility trace; 0 and unused when pure Hebbian.
    trace: f32,
    seeded: bool,
    /// The members feeding the scale, in canonical order; possibly none.
    contributors: Box<[Contributor]>,
}

/// A dynamic edge: an edge learner by index, or a member weighed through a
/// learned scale with its fixed modifier.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Dynamic {
    Learner(usize),
    Fixed { own: f32, term: Term },
}

/// What one learning event did: learner assignments and assignments whose
/// stored quantity changed (T25.F07 invariant 9).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct LearningCounts {
    pub updates: u32,
    pub changes: u32,
}

impl std::ops::AddAssign for LearningCounts {
    fn add_assign(&mut self, other: Self) {
        self.updates += other.updates;
        self.changes += other.changes;
    }
}

/// One mesh node's learners and state. Empty for a VM node and for a graph
/// whose layout is empty.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct BackendLearning {
    /// Edge learners in canonical order.
    edges: Box<[EdgeLearner]>,
    /// Scale learners in `BundleId` order.
    scales: Box<[ScaleLearner]>,
    /// Every dynamic edge in canonical order.
    order: Box<[(EdgeOccurrence, Dynamic)]>,
    /// Set at the first visit's entry, before the pass-cost check, for life.
    visited: bool,
}

/// `value` bounded by `config`'s clamp.
fn bounded(value: f32, config: &Learning) -> f32 {
    value.clamp(-config.weight_clamp, config.weight_clamp)
}

/// One learner's scalar update from its visit's aggregate `change`: a
/// reward-modulated learner adds it to its trace; a pure-Hebbian one adds it
/// to its value, bounded once, and counts the update and whether the stored
/// value moved (exact inequality excludes changes rounded or clamped away).
fn update(
    config: &Learning,
    value: &mut f32,
    trace: &mut f32,
    change: f32,
    counts: &mut LearningCounts,
) {
    if config.modulation.is_some() {
        *trace += change;
        return;
    }
    let old = *value;
    *value = bounded(old + change, config);
    counts.changes += u32::from(*value != old);
    counts.updates += 1;
}

impl BackendLearning {
    fn from_layout(
        def: &CgpGraphBackendDef,
        layout: LearningLayout,
        seeds: Option<&BirthSeeds>,
    ) -> Self {
        let seed_if = |lamarckian: bool, read: &dyn Fn(&BirthSeeds) -> Option<f32>| {
            seeds.filter(|_| lamarckian).and_then(read)
        };
        let term = |member: MemberTerm| Term {
            pattern: member.pattern,
            scale: layout.scale_of(member.bundle).map_or_else(
                || ScaleRead::Genomic(def.bundles()[member.bundle.index()].scale()),
                ScaleRead::Learned,
            ),
        };

        // The dynamic edges: every edge learner, then every other member
        // reading a learned scale, sorted into canonical order.
        let mut order: Vec<(EdgeOccurrence, Dynamic)> = layout
            .edges
            .iter()
            .enumerate()
            .map(|(index, learner)| (learner.occurrence, Dynamic::Learner(index)))
            .collect();
        let learns = |at: EdgeOccurrence| {
            layout
                .edges
                .binary_search_by_key(&at, |learner| learner.occurrence)
                .is_ok()
        };
        for scale in &layout.scales {
            for &member in scale.members.iter().filter(|m| !learns(m.occurrence)) {
                let own = def.edge_at(member.occurrence).own_weight();
                order.push((
                    member.occurrence,
                    Dynamic::Fixed {
                        own,
                        term: term(member),
                    },
                ));
            }
        }
        order.sort_by_key(|&(at, _)| at);
        let dynamic = |at: EdgeOccurrence| {
            order
                .binary_search_by_key(&at, |&(occurrence, _)| occurrence)
                .expect("every learner and every member of a learning scale is dynamic")
        };

        let edges = layout
            .edges
            .iter()
            .map(|learner| {
                let genomic = def.edge_at(learner.occurrence).own_weight();
                let seed = seed_if(learner.learning.lamarckian, &|seeds| {
                    seeds.value(learner.occurrence)
                });
                let birth = bounded(seed.unwrap_or(genomic), &learner.learning);
                EdgeLearner {
                    occurrence: learner.occurrence,
                    config: learner.learning,
                    source: learner.source,
                    genomic,
                    birth,
                    value: birth,
                    trace: 0.0,
                    seeded: seed.is_some(),
                    term: learner.member.map(term),
                    dynamic: dynamic(learner.occurrence),
                }
            })
            .collect();
        let scales = layout
            .scales
            .iter()
            .map(|scale| {
                let genomic = def.bundles()[scale.bundle.index()].scale();
                let seed = seed_if(scale.learning.lamarckian, &|seeds| {
                    seeds.bundle_value(scale.bundle)
                });
                let birth = bounded(seed.unwrap_or(genomic), &scale.learning);
                ScaleLearner {
                    bundle: scale.bundle,
                    config: scale.learning,
                    genomic,
                    birth,
                    value: birth,
                    trace: 0.0,
                    seeded: seed.is_some(),
                    contributors: scale
                        .contributors
                        .iter()
                        .map(|contributor| Contributor {
                            pattern: contributor.pattern,
                            dynamic: dynamic(contributor.occurrence),
                        })
                        .collect(),
                }
            })
            .collect();
        Self {
            edges,
            scales,
            order: order.into_boxed_slice(),
            visited: false,
        }
    }

    fn is_empty(&self) -> bool {
        self.edges.is_empty() && self.scales.is_empty()
    }

    fn initialized(&self, seeded: bool) -> bool {
        self.visited || seeded
    }

    /// The scale a bundle term reads.
    fn term_scale(&self, term: Term) -> f32 {
        match term.scale {
            ScaleRead::Genomic(scale) => scale,
            ScaleRead::Learned(index) => self.scales[index].value,
        }
    }

    /// The applied weight of edge learner `index`: its own quantity alone
    /// when lone, `own + scale × pattern` for a member.
    fn learner_applied(&self, index: usize) -> f32 {
        let learner = &self.edges[index];
        learner.term.map_or(learner.value, |term| {
            compose(learner.value, self.term_scale(term), term.pattern)
        })
    }

    /// The applied weight of a dynamic edge, the one composition the
    /// executor, the view and both resolvers use.
    fn applied(&self, dynamic: Dynamic) -> f32 {
        match dynamic {
            Dynamic::Learner(index) => self.learner_applied(index),
            Dynamic::Fixed { own, term } => compose(own, self.term_scale(term), term.pattern),
        }
    }

    /// Decay every trace once per world tick by its learner's trace decay.
    fn decay(&mut self) {
        for learner in self.edges.iter_mut() {
            if let Some(modulation) = learner.config.modulation {
                learner.trace *= modulation.trace_decay;
            }
        }
        for scale in self.scales.iter_mut() {
            if let Some(modulation) = scale.config.modulation {
                scale.trace *= modulation.trace_decay;
            }
        }
    }

    /// Relax every initialized learner whose retention is below 1 once per
    /// world tick toward its birth value (T25.F09 invariant 3). A retention
    /// of 1 is an exact no-op and an uninitialized learner holds its birth
    /// value.
    fn relax(&mut self) {
        let visited = self.visited;
        for learner in self.edges.iter_mut() {
            if retention_decays(learner.config.retention) && (visited || learner.seeded) {
                learner.value = relax(learner.value, learner.birth, learner.config.retention);
            }
        }
        for scale in self.scales.iter_mut() {
            if retention_decays(scale.config.retention) && (visited || scale.seeded) {
                scale.value = relax(scale.value, scale.birth, scale.config.retention);
            }
        }
    }

    /// The one reward step (T25.F12 invariant 6): `three_factor` per
    /// initialized reward-modulated learner, an edge's or a scale's, at its
    /// own source, rate and clamp.
    fn reward(&mut self, signals: &OutcomeSignalBank) -> LearningCounts {
        let mut counts = LearningCounts::default();
        let visited = self.visited;
        let learners = self
            .edges
            .iter_mut()
            .map(|learner| {
                (
                    &learner.config,
                    learner.seeded,
                    &mut learner.value,
                    learner.trace,
                )
            })
            .chain(
                self.scales
                    .iter_mut()
                    .map(|scale| (&scale.config, scale.seeded, &mut scale.value, scale.trace)),
            );
        for (config, seeded, value, trace) in learners {
            let Some(modulation) = config.modulation else {
                continue;
            };
            if !(visited || seeded) {
                continue;
            }
            let signal = signals.signals[modulation.reward_source as usize];
            let old = *value;
            *value = three_factor(
                old,
                config.learning_rate,
                signal,
                trace,
                config.weight_clamp,
            );
            // Compare the final stored f32, including clamping and rounding.
            counts.changes += u32::from(*value != old);
            counts.updates += 1;
        }
        counts
    }

    /// The one update law over a committed visit's samples (T25.F12
    /// invariant 6), from visit-start state. Aggregation: an edge learner
    /// takes its rule on its own `pre`, `post` and applied weight; a scale
    /// learner sums its rule over its contributors times each pattern.
    /// Update: pure Hebbian adds the aggregate at the learner's rate,
    /// bounded by its clamp, written once; reward-modulated adds it to the
    /// learner's trace.
    fn commit(&mut self, samples: &Samples) -> LearningCounts {
        let mut counts = LearningCounts::default();
        // Aggregate every scale first: contributors read visit-start
        // modifiers and scales.
        let aggregates: Vec<Option<f32>> = self
            .scales
            .iter()
            .map(|scale| {
                (!scale.contributors.is_empty()).then(|| {
                    let config = scale.config;
                    scale.contributors.iter().fold(0.0f32, |sum, contributor| {
                        let at = contributor.dynamic;
                        let w = self.applied(self.order[at].1);
                        let (pre, post) = (samples.pre[at], samples.post[at]);
                        let change = if config.modulation.is_some() {
                            activity(config.rule, pre, post, w)
                        } else {
                            hebbian_delta(config.rule, config.learning_rate, pre, post, w)
                        };
                        sum + contributor.pattern * change
                    })
                })
            })
            .collect();
        for index in 0..self.edges.len() {
            let learner = self.edges[index];
            let at = learner.dynamic;
            let (pre, post) = (samples.pre[at], samples.post[at]);
            // Scales are written after this loop, so this is the visit-start
            // applied weight.
            let w = self.learner_applied(index);
            let config = learner.config;
            let change = if config.modulation.is_some() {
                activity(config.rule, pre, post, w)
            } else {
                hebbian_delta(config.rule, config.learning_rate, pre, post, w)
            };
            let learner = &mut self.edges[index];
            update(
                &config,
                &mut learner.value,
                &mut learner.trace,
                change,
                &mut counts,
            );
        }
        for (scale, aggregate) in self.scales.iter_mut().zip(aggregates) {
            if let Some(change) = aggregate {
                let config = scale.config;
                update(
                    &config,
                    &mut scale.value,
                    &mut scale.trace,
                    change,
                    &mut counts,
                );
            }
        }
        counts
    }

    /// The seeds this backend hands a child: `None` when no learner is
    /// initialized, else one row per compute node and per sink of `def`
    /// with each initialized edge learner's value, and one bundle slot per
    /// bundle record with each initialized scale's value.
    fn capture(&self, def: &CgpGraphBackendDef) -> Option<BirthSeeds> {
        let any_edge = self
            .edges
            .iter()
            .any(|learner| self.initialized(learner.seeded));
        let any_scale = self
            .scales
            .iter()
            .any(|scale| self.initialized(scale.seeded));
        if !any_edge && !any_scale {
            return None;
        }
        let mut compute: Vec<Vec<Option<f32>>> = def
            .compute_nodes()
            .iter()
            .map(|node| vec![None; node.inputs.len()])
            .collect();
        let mut sinks: Vec<Vec<Option<f32>>> = def
            .output_sinks()
            .iter()
            .map(|sink| vec![None; sink.inputs.len()])
            .collect();
        for learner in self.edges.iter() {
            if !self.initialized(learner.seeded) {
                continue;
            }
            let row = match learner.occurrence.surface {
                EdgeSurface::ComputeInput(i) => &mut compute[i],
                EdgeSurface::SinkInput(i) => &mut sinks[i],
            };
            row[learner.occurrence.index] = Some(learner.value);
        }
        let mut bundles = vec![None; def.bundles().len()];
        for scale in self.scales.iter() {
            if self.initialized(scale.seeded) {
                bundles[scale.bundle.index()] = Some(scale.value);
            }
        }
        Some(BirthSeeds::Rows {
            compute,
            sinks,
            bundles,
        })
    }

    /// The view of edge learner `index`.
    fn edge_view(&self, index: usize) -> LearnedEdge {
        let learner = &self.edges[index];
        let initialized = self.initialized(learner.seeded);
        LearnedEdge {
            occurrence: learner.occurrence,
            genomic: learner.genomic,
            birth: learner.birth,
            retention: learner.config.retention,
            learned: initialized.then_some(learner.value),
            applied: self.learner_applied(index),
            trace: learner
                .config
                .modulation
                .filter(|_| initialized)
                .map(|_| learner.trace),
            bound: learner.config.weight_clamp,
            kind: PlasticityKind::of(&learner.config),
            learning: learner.config,
            source: learner.source,
            initialized,
            inherits: learner.config.lamarckian,
        }
    }
}

/// Per-dynamic-edge samples of the visit in flight: runtime scratch.
#[derive(Debug, Clone, Default)]
struct Samples {
    pre: Vec<f32>,
    post: Vec<f32>,
}

/// A creature's learned state, indexed by mesh node. Empty for a genome
/// that carries no learning.
#[derive(Debug, Clone, Default)]
pub struct LearnedState {
    backends: Vec<BackendLearning>,
    samples: Samples,
}

// Scratch samples are outside the state's identity.
impl PartialEq for LearnedState {
    fn eq(&self, other: &Self) -> bool {
        self.backends == other.backends
    }
}

/// One edge learner as the readers see it, in canonical order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LearnedEdge {
    pub occurrence: EdgeOccurrence,
    /// The genomic own weight: a lone edge's weight, a member's modifier
    /// (not its composed weight).
    pub genomic: f32,
    /// The own quantity's birth value (T25.F09).
    pub birth: f32,
    /// The effective learning's retention.
    pub retention: f32,
    /// The learned own quantity, `Some` once initialized.
    pub learned: Option<f32>,
    /// The weight in force: `own + scale × pattern`.
    pub applied: f32,
    /// The eligibility trace, `Some` only for an initialized
    /// reward-modulated learner.
    pub trace: Option<f32>,
    /// The effective learning's clamp.
    pub bound: f32,
    pub kind: PlasticityKind,
    /// The effective learning the learner runs.
    pub learning: Learning,
    /// Where the effective learning is stored.
    pub source: LearnerSource,
    /// Whether the learner is initialized (visited or seeded).
    pub initialized: bool,
    /// Whether a child inherits what it learns: the effective learning's
    /// `lamarckian`, as `capture` reads it.
    pub inherits: bool,
}

/// One bundle's learned scale as the readers see it, in `BundleId` order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LearnedScale {
    pub bundle: BundleId,
    /// The genomic scale.
    pub genomic: f32,
    /// The scale's birth value (T25.F09).
    pub birth: f32,
    /// The scale's retention.
    pub retention: f32,
    /// The learned scale, `Some` only when initialized.
    pub learned: Option<f32>,
    /// The scale's clamp.
    pub bound: f32,
    /// Whether a child inherits the learned scale (its learning's
    /// `lamarckian`).
    pub lamarckian: bool,
}

impl LearnedState {
    /// The learners and initial state of `genome`, built from each graph
    /// node's learning layout once, the seeds of mesh node `i` at
    /// `seeds[i]` (absent or `None`: no inheritance).
    #[must_use]
    pub(crate) fn for_genome(genome: &CreatureGenome, seeds: &[Option<BirthSeeds>]) -> Self {
        let backends: Vec<BackendLearning> = genome
            .nodes
            .iter()
            .enumerate()
            .map(|(idx, node)| match &node.backend_def {
                BackendDef::Graph(def) => BackendLearning::from_layout(
                    def,
                    LearningLayout::of(def, &node.input_refs),
                    seeds.get(idx).and_then(Option::as_ref),
                ),
                BackendDef::Vm(_) => BackendLearning::default(),
            })
            .collect();
        if backends.iter().all(BackendLearning::is_empty) {
            return Self::default();
        }
        Self {
            backends,
            samples: Samples::default(),
        }
    }

    /// Whether any backend carries a learner.
    #[must_use]
    pub fn has_learning(&self) -> bool {
        !self.backends.is_empty()
    }

    /// Whether any backend carries a reward-modulated learner.
    #[must_use]
    pub(crate) fn has_reward_modulated(&self) -> bool {
        self.backends.iter().any(|backend| {
            backend
                .edges
                .iter()
                .any(|learner| learner.config.modulation.is_some())
                || backend
                    .scales
                    .iter()
                    .any(|scale| scale.config.modulation.is_some())
        })
    }

    /// Decay every trace, then relax every decaying learned quantity, once
    /// per world tick before any visit (T19.F02, T25.F09).
    pub(crate) fn begin_tick(&mut self) {
        for backend in &mut self.backends {
            backend.decay();
            backend.relax();
        }
    }

    /// Phase 2.5's three-factor update over every backend.
    pub(crate) fn apply_reward(&mut self, signals: &OutcomeSignalBank) -> LearningCounts {
        let mut counts = LearningCounts::default();
        for backend in &mut self.backends {
            counts += backend.reward(signals);
        }
        counts
    }

    /// Mesh node `node_idx`'s seeds for a child before mutation (`def` is
    /// that node's graph): `None` when nothing there is initialized.
    #[must_use]
    pub(crate) fn capture(&self, node_idx: usize, def: &CgpGraphBackendDef) -> Option<BirthSeeds> {
        self.backends
            .get(node_idx)
            .and_then(|backend| backend.capture(def))
    }

    /// The learner of mesh node `node_idx`'s visit, `None` for a backend
    /// without a learner.
    pub(crate) fn learner(&mut self, node_idx: usize) -> Option<Learner<'_>> {
        let backend = self.backends.get_mut(node_idx)?;
        if backend.is_empty() {
            return None;
        }
        Some(Learner {
            backend,
            samples: &mut self.samples,
            next: 0,
            row_start: 0,
            row: EdgeSurface::ComputeInput(0),
            index: 0,
        })
    }

    /// The read-only view of mesh node `node_idx`'s learning edges, in
    /// canonical order.
    pub fn edges(&self, node_idx: usize) -> impl Iterator<Item = LearnedEdge> + '_ {
        self.backends
            .get(node_idx)
            .into_iter()
            .flat_map(|backend| (0..backend.edges.len()).map(|index| backend.edge_view(index)))
    }

    /// The read-only view of mesh node `node_idx`'s scale learners, in
    /// `BundleId` order.
    pub fn scales(&self, node_idx: usize) -> impl Iterator<Item = LearnedScale> + '_ {
        self.backends.get(node_idx).into_iter().flat_map(|backend| {
            backend.scales.iter().map(move |scale| LearnedScale {
                bundle: scale.bundle,
                genomic: scale.genomic,
                birth: scale.birth,
                retention: scale.config.retention,
                learned: backend.initialized(scale.seeded).then_some(scale.value),
                bound: scale.config.weight_clamp,
                lamarckian: scale.config.lamarckian,
            })
        })
    }

    /// `(occurrence, applied)` over every dynamic edge of mesh node
    /// `node_idx` (every learner and every member reading a learned scale),
    /// in canonical order.
    pub fn applied_weights(
        &self,
        node_idx: usize,
    ) -> impl Iterator<Item = (EdgeOccurrence, f32)> + '_ {
        self.backends.get(node_idx).into_iter().flat_map(|backend| {
            backend
                .order
                .iter()
                .map(move |&(occurrence, dynamic)| (occurrence, backend.applied(dynamic)))
        })
    }

    /// The weight in force on the edge at `occurrence` of mesh node
    /// `node_idx` (graph `def`): the applied weight of a dynamic edge, else
    /// the genomic weight. The only single-edge resolver.
    #[must_use]
    pub fn applied_weight(
        &self,
        node_idx: usize,
        def: &CgpGraphBackendDef,
        input_refs: &[InputReference],
        occurrence: EdgeOccurrence,
    ) -> f32 {
        self.backends
            .get(node_idx)
            .and_then(|backend| {
                backend
                    .order
                    .binary_search_by_key(&occurrence, |&(at, _)| at)
                    .ok()
                    .map(|position| backend.applied(backend.order[position].1))
            })
            .unwrap_or_else(|| def.genomic_weight(occurrence, input_refs))
    }
}

#[cfg(test)]
impl LearnedState {
    /// A fixture: mark mesh node `node_idx`'s backend visited, set its
    /// edge learner `slot` (canonical order) to `learned` and, on a
    /// reward-modulated learner, its trace to `trace`.
    pub(crate) fn set_slot(&mut self, node_idx: usize, slot: usize, learned: f32, trace: f32) {
        let backend = &mut self.backends[node_idx];
        backend.visited = true;
        let learner = &mut backend.edges[slot];
        learner.value = learned;
        if learner.config.modulation.is_some() {
            learner.trace = trace;
        }
    }

    /// A fixture: mark mesh node `node_idx`'s backend visited and set its
    /// scale learner `slot` (`BundleId` order) to `scale`.
    pub(crate) fn set_scale(&mut self, node_idx: usize, slot: usize, scale: f32) {
        let backend = &mut self.backends[node_idx];
        backend.visited = true;
        backend.scales[slot].value = scale;
    }

    /// The trace of mesh node `node_idx`'s scale learner `slot`.
    pub(crate) fn scale_trace(&self, node_idx: usize, slot: usize) -> f32 {
        self.backends[node_idx].scales[slot].trace
    }
}

/// One visit's access to its backend's learned state and samples. The
/// executor weighs every edge in canonical order through it, compute
/// surface first, then the sinks, each container opened by
/// [`Self::begin_row`]; the learner, not the executor, decides from each
/// edge's occurrence whether it is dynamic or static (R1).
/// [`Self::commit`] learns from the samples once the visit's effects have
/// run.
pub(crate) struct Learner<'a> {
    backend: &'a mut BackendLearning,
    samples: &'a mut Samples,
    /// The next dynamic edge in canonical order.
    next: usize,
    /// The first dynamic edge of the container in flight.
    row_start: usize,
    /// The container in flight and the index of its next edge.
    row: EdgeSurface,
    index: usize,
}

impl Learner<'_> {
    /// Enter the visit: the backend counts as visited from here on, a
    /// failed visit included.
    pub(crate) fn enter(&mut self) {
        self.backend.visited = true;
        let n = self.backend.order.len();
        self.samples.pre.clear();
        self.samples.pre.resize(n, 0.0);
        self.samples.post.clear();
        self.samples.post.resize(n, 0.0);
        self.next = 0;
        self.row_start = 0;
    }

    /// Start container `surface` (a compute node or a sink).
    #[inline]
    pub(crate) fn begin_row(&mut self, surface: EdgeSurface) {
        self.row = surface;
        self.index = 0;
        self.row_start = self.next;
    }

    /// The weight of the container's next edge, which read `value`: the
    /// applied weight of a dynamic edge (its pre sample recorded), else
    /// `genomic`.
    #[inline]
    pub(crate) fn weigh(&mut self, value: f32, genomic: f32) -> f32 {
        let occurrence = EdgeOccurrence::new(self.row, self.index);
        self.index += 1;
        match self.backend.order.get(self.next) {
            Some(&(at, dynamic)) if at == occurrence => {
                self.samples.pre[self.next] = value;
                self.next += 1;
                self.backend.applied(dynamic)
            }
            _ => genomic,
        }
    }

    /// End the container in flight: `post` is the post sample of each of
    /// its dynamic edges.
    #[inline]
    pub(crate) fn end_row(&mut self, post: f32) {
        self.samples.post[self.row_start..self.next].fill(post);
    }

    /// Learn from the committed visit's samples through the one update law.
    pub(crate) fn commit(self) -> LearningCounts {
        debug_assert_eq!(
            self.next,
            self.backend.order.len(),
            "the visit weighed every dynamic edge of the layout"
        );
        self.backend.commit(self.samples)
    }
}
