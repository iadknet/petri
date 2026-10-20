//! Learned state (T25.F06, T25.F07): per-edge learned weights, per-group
//! learned scales and eligibility traces, built once at birth from the
//! genome and the parent's seeds.
//!
//! [`LearnedState::for_genome`] lays out, per mesh node, the learners (the
//! plastic edges over both surfaces in canonical order, `edge_sites`), the
//! scale slots (one per group with a group-scope learner, in `GroupId`
//! order) and the passive members (static members of a group with a scale
//! slot), a learned own quantity per ungrouped or edge-scope learner, a
//! learned scale per scale slot, a trace per reward-modulated learner and a
//! visited flag. A graph without a plastic edge allocates nothing and runs
//! the static loops. A learner or a scale slot is initialized once its
//! backend has been visited or when it was seeded at birth; only
//! initialized ones are updated by reward, captured for a child and exposed
//! by the view.
//!
//! Every learned own quantity and learned scale starts at its birth value:
//! the clamped seed when seeded, else its genomic value clamped to its
//! bound (T25.F09 invariant 2). Once per world tick each initialized one
//! whose retention is below 1 relaxes toward it (invariant 3).

use super::rules::{activity, hebbian_delta, relax, three_factor};
use super::{OutcomeSignalBank, PlasticityKind};
use crate::contracts::InputReference;
use crate::creature::genome::cgp::{
    compose, BirthSeeds, CgpGraphBackendDef, EdgeOccurrence, EdgeSurface, GroupId, Scope,
};
use crate::creature::genome::{BackendDef, CreatureGenome, PlasticityConfig};

/// Where a member's group term reads its scale.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ScaleSource {
    /// The group has no scale slot: its genomic scale.
    Genomic(f32),
    /// The group's scale slot.
    Slot(usize),
}

/// A member's group term: its pattern and its scale's source.
#[derive(Debug, Clone, Copy, PartialEq)]
struct GroupTerm {
    pattern: f32,
    scale: ScaleSource,
}

/// One learner (plastic edge) of a backend's layout.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Slot {
    occurrence: EdgeOccurrence,
    config: PlasticityConfig,
    /// The genomic own weight: an ungrouped edge's weight, a member's
    /// correction.
    genomic: f32,
    /// The stored own quantity at construction: the clamped seed or the
    /// clamped genomic own weight; a group learner's raw genomic entry.
    birth: f32,
    /// The member's scope, `None` for an ungrouped edge.
    scope: Option<Scope>,
    /// The member's group term, `None` for an ungrouped edge.
    term: Option<GroupTerm>,
}

impl Slot {
    /// The scale slot a group learner credits, `None` for an ungrouped or
    /// edge-scope learner.
    fn credited_scale(&self) -> Option<(usize, f32)> {
        match (self.scope, self.term) {
            (
                Some(Scope::Group),
                Some(GroupTerm {
                    pattern,
                    scale: ScaleSource::Slot(scale),
                }),
            ) => Some((scale, pattern)),
            _ => None,
        }
    }
}

/// One group's learned scale.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ScaleSlot {
    group: GroupId,
    /// The genomic scale.
    genomic: f32,
    /// The stored scale at construction: the clamped seed or the clamped
    /// genomic scale.
    birth: f32,
    /// The smallest `weight_clamp` among the group's group learners.
    bound: f32,
    inherit_scale: bool,
    /// The group's retention, read from the record at construction.
    retention: f32,
}

/// One static member of a group with a scale slot: weighed through the
/// learned scale, with no samples and no stored value.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Passive {
    own: f32,
    term: GroupTerm,
}

/// A dynamic edge: a learner or a passive member, by index.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Dynamic {
    Learner(usize),
    Passive(usize),
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

/// One mesh node's learning layout and state. Empty for a VM node and for a
/// graph without a plastic edge.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct BackendLearning {
    /// Learners in canonical order.
    slots: Box<[Slot]>,
    /// Per learner, the learned own quantity of an ungrouped or edge-scope
    /// learner, starting at its birth value. A group learner stores none:
    /// its entry holds its raw genomic own weight, is never written, and the
    /// view and `capture` expose `None` for it.
    learned: Box<[f32]>,
    /// Learner index of every ungrouped or edge-scope learner whose
    /// retention is below 1, ascending.
    decaying: Box<[usize]>,
    /// Whether each learner inherited its own quantity at birth.
    seeded: Box<[bool]>,
    /// Learner index of every reward-modulated learner, ascending.
    modulated: Box<[usize]>,
    /// One eligibility trace per entry of `modulated`.
    traces: Box<[f32]>,
    /// The scale slots, in `GroupId` order.
    scales: Box<[ScaleSlot]>,
    /// The learned scale per scale slot.
    scale: Box<[f32]>,
    /// Whether each scale slot inherited its scale at birth.
    scale_seeded: Box<[bool]>,
    /// Every scale slot whose group's retention is below 1, ascending.
    decaying_scales: Box<[usize]>,
    /// Static members of groups with a scale slot, in canonical order.
    passive: Box<[Passive]>,
    /// Every dynamic edge in canonical order.
    order: Box<[(EdgeOccurrence, Dynamic)]>,
    /// Per scale slot, the event's summed contribution and its number of
    /// contributors: zero between events.
    credit: Box<[(f32, u32)]>,
    /// Set at the first visit's entry, before the pass-cost check, for life.
    visited: bool,
}

impl BackendLearning {
    fn from_seeds(
        def: &CgpGraphBackendDef,
        input_refs: &[InputReference],
        seeds: Option<&BirthSeeds>,
    ) -> Self {
        let groups = def.groups();
        // The scale slots: one per group with a group learner, bounded by
        // the smallest clamp among them.
        let mut bounds: Vec<Option<f32>> = vec![None; groups.len()];
        for (occurrence, config) in def.plastic_edges() {
            if let Some(membership) = def.edge_at(occurrence).membership() {
                if membership.scope == Scope::Group {
                    let bound = &mut bounds[membership.group.index()];
                    *bound =
                        Some(bound.map_or(config.weight_clamp, |b| b.min(config.weight_clamp)));
                }
            }
        }
        let mut slot_of: Vec<Option<usize>> = vec![None; groups.len()];
        let mut scales = Vec::new();
        let mut scale_seeded = Vec::new();
        for (index, bound) in bounds.iter().enumerate() {
            if let Some(bound) = *bound {
                slot_of[index] = Some(scales.len());
                let group = GroupId::new(u16::try_from(index).expect("groups below u16::MAX"));
                let genomic = groups[index].scale();
                let inherit_scale = groups[index].inherit_scale();
                // The birth value (invariant 2): the clamped seed, else the
                // clamped genomic scale.
                let seed = seeds
                    .filter(|_| inherit_scale)
                    .and_then(|seeds| seeds.group_value(group));
                scales.push(ScaleSlot {
                    group,
                    genomic,
                    birth: seed.unwrap_or(genomic).clamp(-bound, bound),
                    bound,
                    inherit_scale,
                    retention: groups[index].retention(),
                });
                scale_seeded.push(seed.is_some());
            }
        }
        let scale = scales.iter().map(|slot| slot.birth).collect();
        let decaying_scales = (0..scales.len())
            .filter(|&index| decays(scales[index].retention))
            .collect();

        let mut slots = Vec::new();
        let mut seeded = Vec::new();
        let mut passive = Vec::new();
        let mut order = Vec::new();
        for occurrence in def.edge_sites() {
            let edge = def.edge_at(occurrence);
            let term = edge.membership().map(|membership| GroupTerm {
                pattern: def.pattern(occurrence, input_refs),
                scale: slot_of[membership.group.index()].map_or(
                    ScaleSource::Genomic(groups[membership.group.index()].scale()),
                    ScaleSource::Slot,
                ),
            });
            match (edge.plasticity(), term) {
                (Some(config), _) => {
                    order.push((occurrence, Dynamic::Learner(slots.len())));
                    let scope = edge.membership().map(|membership| membership.scope);
                    let genomic = edge.own_weight();
                    // The birth value (invariant 2): a group learner's raw
                    // entry, else the clamped seed of a seeded Lamarckian
                    // learner (a group learner ignores any seed on its
                    // row), else the clamped genomic own weight.
                    let seed = seeds
                        .filter(|_| config.lamarckian && scope != Some(Scope::Group))
                        .and_then(|seeds| seeds.value(occurrence));
                    let birth = if scope == Some(Scope::Group) {
                        genomic
                    } else {
                        seed.unwrap_or(genomic)
                            .clamp(-config.weight_clamp, config.weight_clamp)
                    };
                    seeded.push(seed.is_some());
                    slots.push(Slot {
                        occurrence,
                        config,
                        genomic,
                        birth,
                        scope,
                        term,
                    });
                }
                (
                    None,
                    Some(
                        term @ GroupTerm {
                            scale: ScaleSource::Slot(_),
                            ..
                        },
                    ),
                ) => {
                    order.push((occurrence, Dynamic::Passive(passive.len())));
                    passive.push(Passive {
                        own: edge.own_weight(),
                        term,
                    });
                }
                _ => {}
            }
        }
        let learned = slots.iter().map(|slot| slot.birth).collect();
        let decaying = slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.scope != Some(Scope::Group) && decays(slot.config.retention))
            .map(|(index, _)| index)
            .collect();
        let modulated: Box<[usize]> = slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.config.modulation.is_some())
            .map(|(index, _)| index)
            .collect();
        let traces = vec![0.0; modulated.len()].into_boxed_slice();
        let credit = vec![(0.0, 0); scales.len()].into_boxed_slice();
        Self {
            slots: slots.into_boxed_slice(),
            learned,
            decaying,
            seeded: seeded.into_boxed_slice(),
            modulated,
            traces,
            scales: scales.into_boxed_slice(),
            scale,
            scale_seeded: scale_seeded.into_boxed_slice(),
            decaying_scales,
            passive: passive.into_boxed_slice(),
            order: order.into_boxed_slice(),
            credit,
            visited: false,
        }
    }

    fn initialized(&self, slot: usize) -> bool {
        self.visited || self.seeded[slot]
    }

    fn scale_initialized(&self, scale: usize) -> bool {
        self.visited || self.scale_seeded[scale]
    }

    /// The scale a group term reads.
    fn term_scale(&self, term: GroupTerm) -> f32 {
        match term.scale {
            ScaleSource::Genomic(scale) => scale,
            ScaleSource::Slot(slot) => self.scale[slot],
        }
    }

    /// `own + scale × pattern` through the scale `term` reads.
    fn composed(&self, own: f32, term: GroupTerm) -> f32 {
        compose(own, self.term_scale(term), term.pattern)
    }

    /// The applied weight of learner `slot`: `own + scale × pattern`, its
    /// own quantity alone when ungrouped (invariant 3).
    fn learner_applied(&self, slot: usize) -> f32 {
        let own = self.learned[slot];
        self.slots[slot]
            .term
            .map_or(own, |term| self.composed(own, term))
    }

    /// The applied weight of a dynamic edge, the one composition the
    /// executor, the view and both resolvers use.
    fn applied(&self, dynamic: Dynamic) -> f32 {
        match dynamic {
            Dynamic::Learner(slot) => self.learner_applied(slot),
            Dynamic::Passive(index) => {
                let passive = self.passive[index];
                self.composed(passive.own, passive.term)
            }
        }
    }

    /// Add one group learner's contribution to its scale's accumulator.
    fn credit(&mut self, scale: usize, contribution: f32) {
        let (sum, contributors) = &mut self.credit[scale];
        *sum += contribution;
        *contributors += 1;
    }

    /// Write every scale with a contribution once, `clamp(scale + Σ,
    /// ±bound)`, in slot order, and clear the accumulators: changes count
    /// every contributor of a moved scale.
    fn write_scales(&mut self, counts: &mut LearningCounts) {
        for (index, credit) in self.credit.iter_mut().enumerate() {
            let (sum, contributors) = std::mem::take(credit);
            if contributors == 0 {
                continue;
            }
            let bound = self.scales[index].bound;
            let old = self.scale[index];
            let new = (old + sum).clamp(-bound, bound);
            // Exact stored-value inequality excludes sums rounded or
            // clamped away.
            if new != old {
                counts.changes += contributors;
            }
            self.scale[index] = new;
        }
    }

    /// Decay every trace once per world tick by its edge's retention.
    fn decay(&mut self) {
        for (trace, &slot) in self.traces.iter_mut().zip(self.modulated.iter()) {
            if let Some(modulation) = self.slots[slot].config.modulation {
                *trace *= modulation.trace_decay;
            }
        }
    }

    /// Relax every initialized decaying own quantity and scale once per
    /// world tick toward its birth value (T25.F09 invariant 3). A retention
    /// of 1 is never listed, an uninitialized one holds its birth value and
    /// is skipped, and a group learner has no own decay.
    fn relax(&mut self) {
        for &slot in self.decaying.iter() {
            if self.visited || self.seeded[slot] {
                let birth = self.slots[slot].birth;
                let retention = self.slots[slot].config.retention;
                self.learned[slot] = relax(self.learned[slot], birth, retention);
            }
        }
        for &index in self.decaying_scales.iter() {
            if self.visited || self.scale_seeded[index] {
                let ScaleSlot {
                    birth, retention, ..
                } = self.scales[index];
                self.scale[index] = relax(self.scale[index], birth, retention);
            }
        }
    }

    /// The three-factor update: each initialized modulated ungrouped or
    /// edge learner updates its own quantity, each modulated group learner
    /// whose scale slot is initialized credits its scale, then each credited
    /// scale is written once.
    fn reward(&mut self, signals: &OutcomeSignalBank) -> LearningCounts {
        let mut counts = LearningCounts::default();
        for position in 0..self.modulated.len() {
            let (trace, slot) = (self.traces[position], self.modulated[position]);
            let config = self.slots[slot].config;
            let Some(modulation) = config.modulation else {
                continue;
            };
            let signal = signals.signals[modulation.reward_source as usize];
            if let Some((scale, pattern)) = self.slots[slot].credited_scale() {
                if !self.scale_initialized(scale) {
                    continue;
                }
                self.credit(scale, pattern * config.learning_rate * signal * trace);
            } else {
                if !self.initialized(slot) {
                    continue;
                }
                let old = self.learned[slot];
                self.learned[slot] = three_factor(
                    old,
                    config.learning_rate,
                    signal,
                    trace,
                    config.weight_clamp,
                );
                // Compare the final stored f32, including clamping and rounding.
                counts.changes += u32::from(self.learned[slot] != old);
            }
            counts.updates += 1;
        }
        self.write_scales(&mut counts);
        counts
    }

    /// The seeds this backend hands a child: `None` when no learner and no
    /// scale slot is initialized, else one row per compute node and per sink
    /// of `def` with the own quantity of each initialized ungrouped or edge
    /// learner, and one group slot per group record with the learned scale
    /// of each initialized scale slot.
    fn capture(&self, def: &CgpGraphBackendDef) -> Option<BirthSeeds> {
        let any_slot = (0..self.slots.len()).any(|slot| self.initialized(slot));
        let any_scale = (0..self.scales.len()).any(|scale| self.scale_initialized(scale));
        if !any_slot && !any_scale {
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
        for (index, slot) in self.slots.iter().enumerate() {
            if !self.initialized(index) || slot.scope == Some(Scope::Group) {
                continue;
            }
            let row = match slot.occurrence.surface {
                EdgeSurface::ComputeInput(i) => &mut compute[i],
                EdgeSurface::SinkInput(i) => &mut sinks[i],
            };
            row[slot.occurrence.index] = Some(self.learned[index]);
        }
        let mut groups = vec![None; def.groups().len()];
        for (index, slot) in self.scales.iter().enumerate() {
            if self.scale_initialized(index) {
                groups[slot.group.index()] = Some(self.scale[index]);
            }
        }
        Some(BirthSeeds::Rows {
            compute,
            sinks,
            groups,
        })
    }
}

/// Per-learner samples of the visit in flight: runtime scratch.
#[derive(Debug, Clone, Default)]
struct Samples {
    pre: Vec<f32>,
    post: Vec<f32>,
}

/// A creature's learned state, indexed by mesh node. Empty for a genome
/// without a plastic edge.
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

/// One plastic edge as the readers see it, in canonical order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LearnedEdge {
    pub occurrence: EdgeOccurrence,
    /// The genomic own weight: an ungrouped edge's weight, a member's
    /// correction (not its composed weight).
    pub genomic: f32,
    /// The own quantity's birth value (T25.F09); a group learner's raw
    /// entry, which no reading uses.
    pub birth: f32,
    /// The edge's stored retention.
    pub retention: f32,
    /// The learned own quantity, `Some` only for an initialized ungrouped
    /// or edge-scope learner; always `None` for a group learner.
    pub learned: Option<f32>,
    /// The weight in force: `own + scale × pattern`.
    pub applied: f32,
    /// The eligibility trace, `Some` only for an initialized
    /// reward-modulated learner.
    pub trace: Option<f32>,
    /// The edge's weight clamp.
    pub bound: f32,
    pub kind: PlasticityKind,
    /// The member's scope, `None` for an ungrouped edge.
    pub scope: Option<Scope>,
}

/// One group's learned scale as the readers see it, in `GroupId` order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LearnedScale {
    pub group: GroupId,
    /// The genomic scale.
    pub genomic: f32,
    /// The scale's birth value (T25.F09).
    pub birth: f32,
    /// The group's retention.
    pub retention: f32,
    /// The learned scale, `Some` only for an initialized scale slot.
    pub learned: Option<f32>,
    /// The smallest clamp among the group's group learners.
    pub bound: f32,
    pub inherit_scale: bool,
}

impl LearnedState {
    /// The layout and initial state of `genome`'s plastic edges, the seeds
    /// of mesh node `i` at `seeds[i]` (absent or `None`: no inheritance).
    #[must_use]
    pub(crate) fn for_genome(genome: &CreatureGenome, seeds: &[Option<BirthSeeds>]) -> Self {
        let plastic = genome.nodes.iter().any(|node| {
            matches!(&node.backend_def, BackendDef::Graph(def) if def.plastic_edges().next().is_some())
        });
        if !plastic {
            return Self::default();
        }
        let backends = genome
            .nodes
            .iter()
            .enumerate()
            .map(|(idx, node)| match &node.backend_def {
                BackendDef::Graph(def) => BackendLearning::from_seeds(
                    def,
                    &node.input_refs,
                    seeds.get(idx).and_then(Option::as_ref),
                ),
                BackendDef::Vm(_) => BackendLearning::default(),
            })
            .collect();
        Self {
            backends,
            samples: Samples::default(),
        }
    }

    /// Whether any backend carries a plastic edge.
    #[must_use]
    pub fn has_learning(&self) -> bool {
        !self.backends.is_empty()
    }

    /// Whether any backend carries a reward-modulated edge.
    #[must_use]
    pub(crate) fn has_reward_modulated(&self) -> bool {
        self.backends
            .iter()
            .any(|backend| !backend.modulated.is_empty())
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
    /// without a plastic edge.
    pub(crate) fn learner(&mut self, node_idx: usize) -> Option<Learner<'_>> {
        let backend = self.backends.get_mut(node_idx)?;
        if backend.slots.is_empty() {
            return None;
        }
        Some(Learner {
            backend,
            samples: &mut self.samples,
            next: 0,
            cursor: 0,
            row_start: 0,
            row: EdgeSurface::ComputeInput(0),
            index: 0,
        })
    }

    /// The read-only view of mesh node `node_idx`'s plastic edges, in
    /// canonical order.
    pub fn edges(&self, node_idx: usize) -> impl Iterator<Item = LearnedEdge> + '_ {
        self.backends.get(node_idx).into_iter().flat_map(|backend| {
            let mut modulated = 0;
            backend.slots.iter().enumerate().map(move |(index, slot)| {
                let initialized = backend.initialized(index);
                let trace = slot.config.modulation.map(|_| {
                    let trace = backend.traces[modulated];
                    modulated += 1;
                    trace
                });
                LearnedEdge {
                    occurrence: slot.occurrence,
                    genomic: slot.genomic,
                    birth: slot.birth,
                    retention: slot.config.retention,
                    learned: (initialized && slot.scope != Some(Scope::Group))
                        .then_some(backend.learned[index]),
                    applied: backend.learner_applied(index),
                    trace: trace.filter(|_| initialized),
                    bound: slot.config.weight_clamp,
                    kind: PlasticityKind::of(&slot.config),
                    scope: slot.scope,
                }
            })
        })
    }

    /// The read-only view of mesh node `node_idx`'s scale slots, in
    /// `GroupId` order.
    pub fn scales(&self, node_idx: usize) -> impl Iterator<Item = LearnedScale> + '_ {
        self.backends.get(node_idx).into_iter().flat_map(|backend| {
            backend
                .scales
                .iter()
                .enumerate()
                .map(move |(index, slot)| LearnedScale {
                    group: slot.group,
                    genomic: slot.genomic,
                    birth: slot.birth,
                    retention: slot.retention,
                    learned: backend
                        .scale_initialized(index)
                        .then_some(backend.scale[index]),
                    bound: slot.bound,
                    inherit_scale: slot.inherit_scale,
                })
        })
    }

    /// `(occurrence, applied)` over every dynamic edge of mesh node
    /// `node_idx` (every learner and passive member), in canonical order.
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
                    .iter()
                    .find(|(at, _)| *at == occurrence)
                    .map(|&(_, dynamic)| backend.applied(dynamic))
            })
            .unwrap_or_else(|| def.genomic_weight(occurrence, input_refs))
    }
}

#[cfg(test)]
impl LearnedState {
    /// A fixture: mark mesh node `node_idx`'s backend visited, set its
    /// learner `slot` (canonical order) to `learned` and, on a
    /// reward-modulated learner, its trace to `trace`.
    pub(crate) fn set_slot(&mut self, node_idx: usize, slot: usize, learned: f32, trace: f32) {
        let backend = &mut self.backends[node_idx];
        backend.visited = true;
        backend.learned[slot] = learned;
        if let Ok(position) = backend.modulated.binary_search(&slot) {
            backend.traces[position] = trace;
        }
    }

    /// A fixture: mark mesh node `node_idx`'s backend visited and set its
    /// scale slot `slot` (`GroupId` order) to `scale`.
    pub(crate) fn set_scale(&mut self, node_idx: usize, slot: usize, scale: f32) {
        let backend = &mut self.backends[node_idx];
        backend.visited = true;
        backend.scale[slot] = scale;
    }
}

/// Whether a retention decays: below 1, compared exactly so that 1 is an
/// exact no-op (T25.F09 invariant 3).
fn decays(retention: f32) -> bool {
    retention < 1.0
}

/// One visit's access to its backend's learned state and samples. The
/// executor weighs every edge in canonical order through it, compute
/// surface first, then the sinks, each container opened by
/// [`Self::begin_row`]; the learner, not the executor, decides from each
/// edge's occurrence whether it is a learner, a passive member or static
/// (R1). [`Self::commit`] learns from the samples once the visit's effects
/// have run.
pub(crate) struct Learner<'a> {
    backend: &'a mut BackendLearning,
    samples: &'a mut Samples,
    /// The next dynamic edge in canonical order.
    next: usize,
    /// The next learner in canonical order.
    cursor: usize,
    /// The first learner of the container in flight.
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
        let n = self.backend.slots.len();
        self.samples.pre.clear();
        self.samples.pre.resize(n, 0.0);
        self.samples.post.clear();
        self.samples.post.resize(n, 0.0);
        self.next = 0;
        self.cursor = 0;
        self.row_start = 0;
    }

    /// Start container `surface` (a compute node or a sink).
    #[inline]
    pub(crate) fn begin_row(&mut self, surface: EdgeSurface) {
        self.row = surface;
        self.index = 0;
        self.row_start = self.cursor;
    }

    /// The weight of the container's next edge, which read `value`: the
    /// applied weight of a dynamic edge (a learner's pre sample recorded),
    /// else `genomic`.
    #[inline]
    pub(crate) fn weigh(&mut self, value: f32, genomic: f32) -> f32 {
        let occurrence = EdgeOccurrence::new(self.row, self.index);
        self.index += 1;
        match self.backend.order.get(self.next) {
            Some(&(at, dynamic)) if at == occurrence => {
                self.next += 1;
                if let Dynamic::Learner(slot) = dynamic {
                    self.samples.pre[slot] = value;
                    self.cursor += 1;
                }
                self.backend.applied(dynamic)
            }
            _ => genomic,
        }
    }

    /// End the container in flight: `post` is the post sample of each of
    /// its learners.
    #[inline]
    pub(crate) fn end_row(&mut self, post: f32) {
        self.samples.post[self.row_start..self.cursor].fill(post);
    }

    /// Learn from the committed visit's samples (invariant 4), in canonical
    /// order with visit-start values: each pure-Hebbian ungrouped or edge
    /// learner applies its rule's change to its own quantity bounded by its
    /// clamp, each pure-Hebbian group learner credits `pattern × Δ` to its
    /// scale, each reward-modulated learner adds its activity to its trace;
    /// then each credited scale is written once.
    pub(crate) fn commit(self) -> LearningCounts {
        debug_assert_eq!(
            self.next,
            self.backend.order.len(),
            "the visit weighed every dynamic edge of the layout"
        );
        let backend = self.backend;
        let mut counts = LearningCounts::default();
        let mut modulated = 0;
        for slot in 0..backend.slots.len() {
            let config = backend.slots[slot].config;
            let (pre, post) = (self.samples.pre[slot], self.samples.post[slot]);
            // Own quantities change only at their own slot and scales after
            // the loop, so this is the visit-start applied weight.
            let w = backend.learner_applied(slot);
            if config.modulation.is_some() {
                backend.traces[modulated] += activity(config.rule, pre, post, w);
                modulated += 1;
                continue;
            }
            let dw = hebbian_delta(config.rule, config.learning_rate, pre, post, w);
            if let Some((scale, pattern)) = backend.slots[slot].credited_scale() {
                backend.credit(scale, pattern * dw);
            } else {
                let old = backend.learned[slot];
                backend.learned[slot] = (old + dw).clamp(-config.weight_clamp, config.weight_clamp);
                // Exact stored-value inequality excludes deltas rounded or
                // clamped away.
                counts.changes += u32::from(backend.learned[slot] != old);
            }
            counts.updates += 1;
        }
        backend.write_scales(&mut counts);
        counts
    }
}
