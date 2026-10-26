//! A creature brain's execution state: what the mesh loop reads and writes
//! across ticks, and the record of which mesh nodes it dispatched.

use crate::creature::genome::cgp::BirthSeeds;
use crate::creature::genome::{BackendDef, CreatureGenome};
use crate::runtime::plasticity::LearnedState;

/// Which mesh nodes a creature's brain dispatched, and how recently.
///
/// One entry per mesh node index, holding the creature age at which that node
/// last dispatched (`None` until it dispatches at all). The mesh executor
/// writes one entry per hop; the mutation engine reads the recent window at a
/// birth to bias the offspring's mutation targets toward executed structure
/// (T11.F17). Indices belong to the creature's own genome, so a newborn always
/// starts empty rather than inheriting its parent's record.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DispatchRecord {
    last_dispatch_age: Vec<Option<u64>>,
    /// Age of the tick currently being executed, set by `begin_tick`.
    age: u64,
}

impl DispatchRecord {
    /// Adopt the creature's current age for the dispatches of this tick.
    fn begin_tick(&mut self, age: u64) {
        self.age = age;
    }

    /// Record that mesh node `node_idx` dispatched in the current tick.
    pub fn record_dispatch(&mut self, node_idx: usize) {
        if node_idx >= self.last_dispatch_age.len() {
            self.last_dispatch_age.resize(node_idx + 1, None);
        }
        self.last_dispatch_age[node_idx] = Some(self.age);
    }

    /// Sorted ascending mesh node indices dispatched less than `window` ticks
    /// before `age`. An empty record yields an empty set.
    #[must_use]
    pub fn executed_indices(&self, age: u64, window: u64) -> Vec<usize> {
        self.last_dispatch_age
            .iter()
            .enumerate()
            .filter(|(_, last)| {
                last.is_some_and(|last| age.saturating_sub(last) < window && last <= age)
            })
            .map(|(index, _)| index)
            .collect()
    }

    /// Whether any dispatch has ever been recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.last_dispatch_age.iter().all(Option::is_none)
    }
}

/// Per-creature runtime state for Graph backends.
/// Groups all mutable state that graph evaluation reads/writes.
#[derive(Clone)]
pub struct GraphRuntimeState {
    /// Per-node stateful operator state. Indexed as `[mesh_node_idx][internal_node_idx]`.
    pub node_state: Vec<Vec<f32>>,
    /// Last committed compute outputs (the last successful visit, this tick
    /// or earlier), held when a module is unvisited.
    pub node_outputs: Vec<Vec<f32>>,
    /// The learned weights and traces of every plastic edge (T25.F06),
    /// built once at construction by [`Self::for_genome`]; read through
    /// [`Self::learned`].
    pub(crate) learned: LearnedState,
    /// The genomic weight of every sink edge, built once at construction by
    /// [`Self::for_genome`] and read by the sink loop (T25.F03). Indexed as
    /// `[mesh_node_idx][sink_idx][edge_idx]`; a VM node's row is empty.
    pub(crate) sink_weights: Vec<Vec<Box<[f32]>>>,
    /// Scratch: prev_outputs buffer reused across graph evaluations.
    pub(crate) scratch_prev: Vec<f32>,
    /// Scratch: curr_outputs buffer reused across graph evaluations.
    pub(crate) scratch_curr: Vec<f32>,
    /// Scratch: candidate operator state, committed only after affordability checks.
    pub(crate) scratch_backup: Vec<f32>,
    /// Scratch: weighted-inputs buffer reused across graph evaluations.
    pub(crate) scratch_w_inputs: Vec<f32>,
    /// Which mesh nodes this creature dispatched, and at what age (T11.F17).
    /// Lives here because it is the per-creature state the mesh loop already
    /// borrows mutably; observation clones therefore never touch the live one.
    pub dispatch_record: DispatchRecord,
}

impl GraphRuntimeState {
    /// Begin one world tick before any mesh visits at the creature's current
    /// age: the dispatch record's age and the once-per-world-tick eligibility
    /// decay. No internal state is snapshotted; visits read and commit the
    /// live state (T19.F02).
    pub fn begin_tick(&mut self, age: u64) {
        self.dispatch_record.begin_tick(age);
        self.learned.begin_tick();
    }

    /// The learned state's read-only view (T25.F06).
    #[must_use]
    pub fn learned(&self) -> &LearnedState {
        &self.learned
    }

    /// The only constructor (T25.F03, T25.F06): the runtime state of
    /// `genome`, with the sink weight table and the learned state built from
    /// the genome and `seeds` (mesh node `i`'s birth seeds at `seeds[i]`;
    /// empty for a genome without inheritance).
    #[must_use]
    pub fn for_genome(genome: &CreatureGenome, seeds: Vec<Option<BirthSeeds>>) -> Self {
        let sink_weights = genome
            .nodes
            .iter()
            .map(|node| match &node.backend_def {
                BackendDef::Graph(def) => def.sink_weight_rows(&node.input_refs),
                BackendDef::Vm(_) => Vec::new(),
            })
            .collect();
        Self {
            node_state: Vec::new(),
            node_outputs: Vec::new(),
            learned: LearnedState::for_genome(genome, &seeds),
            sink_weights,

            scratch_prev: Vec::new(),
            scratch_curr: Vec::new(),
            scratch_backup: Vec::new(),
            scratch_w_inputs: Vec::new(),
            dispatch_record: DispatchRecord::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_record_keeps_the_latest_age_per_node_within_the_window() {
        let mut record = DispatchRecord::default();
        record.begin_tick(10);
        record.record_dispatch(2);
        record.begin_tick(40);
        record.record_dispatch(0);
        // Node 2 last ran at age 10, node 0 at age 40.
        assert_eq!(record.executed_indices(40, 100), vec![0, 2]);
        assert_eq!(record.executed_indices(40, 31), vec![0, 2]);
        assert_eq!(record.executed_indices(40, 30), vec![0]);
        assert_eq!(record.executed_indices(140, 100), Vec::<usize>::new());
        assert!(!record.is_empty());
    }

    proptest::proptest! {
        /// The window is sorted, monotone in the window length, and, from the
        /// last recorded dispatch on, only ever loses nodes as the creature ages.
        #[test]
        fn dispatch_window_membership_is_monotone_in_age_and_window(
            dispatches in proptest::collection::vec((0usize..8, 0u64..200), 0..12),
            elapsed in 0u64..400,
            window in 1u64..150,
            growth in 0u64..400,
        ) {
            let mut record = DispatchRecord::default();
            let mut newest = 0;
            for (node, at) in dispatches {
                record.begin_tick(at);
                record.record_dispatch(node);
                newest = newest.max(at);
            }
            let age = newest + elapsed;
            let now = record.executed_indices(age, window);
            let later = record.executed_indices(age + growth, window);
            let wider = record.executed_indices(age, window + growth);
            proptest::prop_assert!(now.windows(2).all(|pair| pair[0] < pair[1]));
            for index in &later {
                proptest::prop_assert!(
                    now.contains(index),
                    "ageing past a dispatch never brings it back into the window"
                );
            }
            for index in &now {
                proptest::prop_assert!(
                    wider.contains(index),
                    "a wider window keeps every member of a narrower one"
                );
            }
        }
    }

    #[test]
    fn dispatch_record_ignores_dispatches_recorded_after_the_queried_age() {
        let mut record = DispatchRecord::default();
        record.begin_tick(50);
        record.record_dispatch(1);
        assert_eq!(record.executed_indices(50, 100), vec![1]);
        assert_eq!(record.executed_indices(49, 100), Vec::<usize>::new());
    }
}
