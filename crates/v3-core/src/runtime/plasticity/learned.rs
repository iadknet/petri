//! Learned state (T25.F06): per-edge learned weights and eligibility traces,
//! built once at birth from the genome and the parent's seeds.
//!
//! [`LearnedState::for_genome`] lays out, per mesh node, the plastic-edge
//! slots over both surfaces in canonical order (`edge_sites`), a learned own
//! weight per slot, a trace per reward-modulated slot and a visited flag. A
//! graph without a plastic edge allocates nothing and runs the static loops.
//! A slot is initialized once its backend has been visited or when it was
//! seeded at birth; only initialized slots are updated by reward, captured
//! for a child and exposed by the view.

use super::rules::{activity, hebbian_delta, three_factor};
use super::OutcomeSignalBank;
use crate::contracts::InputReference;
use crate::creature::genome::cgp::{BirthSeeds, CgpGraphBackendDef, EdgeOccurrence, EdgeSurface};
use crate::creature::genome::{BackendDef, CreatureGenome, PlasticityConfig};
use crate::creature::plasticity_census::PlasticityKind;

/// One plastic edge of a backend's layout.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Slot {
    occurrence: EdgeOccurrence,
    config: PlasticityConfig,
    /// The genomic weight at birth (the edge's own weight: a plastic edge is
    /// never a member).
    genomic: f32,
}

/// One mesh node's learning layout and state. Empty for a VM node and for a
/// graph without a plastic edge.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct BackendLearning {
    /// Plastic edges in canonical order.
    slots: Box<[Slot]>,
    /// The learned own weight per slot: the clamped seed of a seeded
    /// Lamarckian slot, else the genomic weight until learning moves it.
    learned: Box<[f32]>,
    /// Whether each slot inherited its learned weight at birth.
    seeded: Box<[bool]>,
    /// Slot index of every reward-modulated slot, ascending.
    modulated: Box<[usize]>,
    /// One eligibility trace per entry of `modulated`.
    traces: Box<[f32]>,
    /// Set at the first visit's entry, before the pass-cost check, for life.
    visited: bool,
}

impl BackendLearning {
    fn from_seeds(
        def: &CgpGraphBackendDef,
        input_refs: &[InputReference],
        seeds: Option<&BirthSeeds>,
    ) -> Self {
        let slots: Box<[Slot]> = def
            .plastic_edges()
            .map(|(occurrence, config)| Slot {
                occurrence,
                config,
                genomic: def.genomic_weight(occurrence, input_refs),
            })
            .collect();
        let inherited = |slot: &Slot| {
            seeds
                .filter(|_| slot.config.lamarckian)
                .and_then(|seeds| seeds.value(slot.occurrence))
                .map(|seed| seed.clamp(-slot.config.weight_clamp, slot.config.weight_clamp))
        };
        let learned = slots
            .iter()
            .map(|slot| inherited(slot).unwrap_or(slot.genomic))
            .collect();
        let seeded = slots.iter().map(|slot| inherited(slot).is_some()).collect();
        let modulated: Box<[usize]> = slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.config.modulation.is_some())
            .map(|(index, _)| index)
            .collect();
        let traces = vec![0.0; modulated.len()].into_boxed_slice();
        Self {
            slots,
            learned,
            seeded,
            modulated,
            traces,
            visited: false,
        }
    }

    fn initialized(&self, slot: usize) -> bool {
        self.visited || self.seeded[slot]
    }

    /// Decay every trace once per world tick by its edge's retention.
    fn decay(&mut self) {
        for (trace, &slot) in self.traces.iter_mut().zip(self.modulated.iter()) {
            if let Some(modulation) = self.slots[slot].config.modulation {
                *trace *= modulation.trace_decay;
            }
        }
    }

    /// The three-factor update on every initialized modulated slot: edge
    /// assignments and assignments that change a stored weight.
    fn reward(&mut self, signals: &OutcomeSignalBank) -> (u32, u32) {
        let (mut updates, mut changes) = (0, 0);
        for (&trace, &slot) in self.traces.iter().zip(self.modulated.iter()) {
            if !self.initialized(slot) {
                continue;
            }
            let config = self.slots[slot].config;
            let Some(modulation) = config.modulation else {
                continue;
            };
            let old = self.learned[slot];
            self.learned[slot] = three_factor(
                old,
                config.learning_rate,
                signals.signals[modulation.reward_source as usize],
                trace,
                config.weight_clamp,
            );
            // Compare the final stored f32, including clamping and rounding.
            changes += u32::from(self.learned[slot] != old);
            updates += 1;
        }
        (updates, changes)
    }

    /// The seeds this backend hands a child: `None` when no plastic edge is
    /// initialized, else one row per compute node and per sink of `def`,
    /// a value per initialized plastic edge.
    fn capture(&self, def: &CgpGraphBackendDef) -> Option<BirthSeeds> {
        if !(0..self.slots.len()).any(|slot| self.initialized(slot)) {
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
            if !self.initialized(index) {
                continue;
            }
            let row = match slot.occurrence.surface {
                EdgeSurface::ComputeInput(i) => &mut compute[i],
                EdgeSurface::SinkInput(i) => &mut sinks[i],
            };
            row[slot.occurrence.index] = Some(self.learned[index]);
        }
        Some(BirthSeeds::Rows { compute, sinks })
    }
}

/// Per-slot samples of the visit in flight: runtime scratch.
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
    /// The genomic weight at birth.
    pub genomic: f32,
    /// The learned own weight, `Some` only for an initialized slot.
    pub learned: Option<f32>,
    /// The eligibility trace, `Some` only for an initialized
    /// reward-modulated slot.
    pub trace: Option<f32>,
    /// The edge's weight clamp.
    pub bound: f32,
    pub kind: PlasticityKind,
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

    /// Decay every trace once per world tick (T19.F02).
    pub(crate) fn begin_tick(&mut self) {
        for backend in &mut self.backends {
            backend.decay();
        }
    }

    /// Phase 2.5's three-factor update over every backend: edge assignments
    /// and assignments that change a stored weight.
    pub(crate) fn apply_reward(&mut self, signals: &OutcomeSignalBank) -> (u32, u32) {
        self.backends
            .iter_mut()
            .map(|backend| backend.reward(signals))
            .fold((0, 0), |(u, c), (bu, bc)| (u + bu, c + bc))
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
            cursor: 0,
            row_start: 0,
        })
    }

    /// The read-only view of mesh node `node_idx`'s plastic edges, in
    /// canonical order.
    pub fn edges(&self, node_idx: usize) -> impl Iterator<Item = LearnedEdge> + '_ {
        let backend = self.backends.get(node_idx);
        let slots = backend.map_or(&[][..], |backend| &backend.slots[..]);
        let mut modulated = 0;
        slots.iter().enumerate().map(move |(index, slot)| {
            let backend = backend.expect("a slot belongs to a backend");
            let initialized = backend.initialized(index);
            let trace = slot.config.modulation.map(|_| {
                let trace = backend.traces[modulated];
                modulated += 1;
                trace
            });
            LearnedEdge {
                occurrence: slot.occurrence,
                genomic: slot.genomic,
                learned: initialized.then_some(backend.learned[index]),
                trace: trace.filter(|_| initialized),
                bound: slot.config.weight_clamp,
                kind: PlasticityKind::of(&slot.config),
            }
        })
    }

    /// The weight in force on the edge at `occurrence` of mesh node
    /// `node_idx` (graph `def`): the learned own weight of a plastic edge,
    /// else the genomic weight. The only resolver.
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
                    .slots
                    .iter()
                    .position(|slot| slot.occurrence == occurrence)
                    .map(|slot| backend.learned[slot])
            })
            .unwrap_or_else(|| def.genomic_weight(occurrence, input_refs))
    }
}

#[cfg(test)]
impl LearnedState {
    /// A fixture: mark mesh node `node_idx`'s backend visited, set its slot
    /// `slot` (canonical order) to `learned` and, on a reward-modulated
    /// slot, its trace to `trace`.
    pub(crate) fn set_slot(&mut self, node_idx: usize, slot: usize, learned: f32, trace: f32) {
        let backend = &mut self.backends[node_idx];
        backend.visited = true;
        backend.learned[slot] = learned;
        if let Ok(position) = backend.modulated.binary_search(&slot) {
            backend.traces[position] = trace;
        }
    }
}

/// One visit's access to its backend's learned weights and samples. The
/// executor weighs every edge in canonical order through it, compute
/// surface first, then the sinks; [`Self::commit`] learns from the samples
/// once the visit's effects have run.
pub(crate) struct Learner<'a> {
    backend: &'a mut BackendLearning,
    samples: &'a mut Samples,
    /// The next plastic slot in canonical order.
    cursor: usize,
    /// The first slot of the container in flight.
    row_start: usize,
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
        self.cursor = 0;
        self.row_start = 0;
    }

    /// Start the next container (a compute node or a sink).
    #[inline]
    pub(crate) fn begin_row(&mut self) {
        self.row_start = self.cursor;
    }

    /// The weight of the next edge, which read `value`: the learned weight
    /// of a plastic edge, whose pre sample is recorded, else `genomic`.
    #[inline]
    pub(crate) fn weigh(&mut self, plastic: bool, value: f32, genomic: f32) -> f32 {
        if !plastic {
            return genomic;
        }
        let slot = self.cursor;
        self.cursor += 1;
        self.samples.pre[slot] = value;
        self.backend.learned[slot]
    }

    /// End the container in flight: `post` is the post sample of each of
    /// its plastic edges.
    #[inline]
    pub(crate) fn end_row(&mut self, post: f32) {
        self.samples.post[self.row_start..self.cursor].fill(post);
    }

    /// Learn from the committed visit's samples: each pure-Hebbian slot
    /// applies its rule's change bounded by its clamp, in canonical order,
    /// and each reward-modulated slot adds its activity to its trace.
    /// Returns pure-Hebbian edge assignments and the assignments that
    /// change a stored weight.
    pub(crate) fn commit(self) -> (u32, u32) {
        debug_assert_eq!(
            self.cursor,
            self.backend.slots.len(),
            "the visit weighed every plastic edge of the layout"
        );
        let backend = self.backend;
        let (mut updates, mut changes) = (0, 0);
        let mut modulated = 0;
        for (slot, config) in backend.slots.iter().map(|slot| slot.config).enumerate() {
            let (pre, post) = (self.samples.pre[slot], self.samples.post[slot]);
            let w = backend.learned[slot];
            if config.modulation.is_some() {
                backend.traces[modulated] += activity(config.rule, pre, post, w);
                modulated += 1;
                continue;
            }
            let dw = hebbian_delta(config.rule, config.learning_rate, pre, post, w);
            backend.learned[slot] = (w + dw).clamp(-config.weight_clamp, config.weight_clamp);
            // Exact stored-value inequality excludes deltas rounded or
            // clamped away.
            changes += u32::from(backend.learned[slot] != w);
            updates += 1;
        }
        (updates, changes)
    }
}
