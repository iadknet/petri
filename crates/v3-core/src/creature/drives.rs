//! The drive census (`drives-v1`, T11.F30): which creatures carry an edge
//! into a kind's drive, and how often a kind's direction channel repeats one
//! source across directions (a fan-out, the duplication a drive writes once).
//!
//! Reachable nodes only, both backends. A drive path for kind `K` is a Graph
//! edge with `|w| > 1e-3` into `Drive(K)`, signed by `w`, or a VM `AddVote`
//! into `Drive(K)`, unsigned. A fan-out is, on one reachable Graph node and
//! one kind, a `(source, sign)` with such edges into at least two of the
//! kind's eight direction sinks; its edges are fan-out edges.
//!
//! Pure structure: it reads genomes and cached reachability and executes no
//! brain.

use std::collections::{BTreeSet, HashMap};

use crate::creature::genome::cgp::{CgpGraphBackendDef, GraphSource, OutputSinkKind};
use crate::creature::genome::vote::{DirectedVoteKind, VoteSink};
use crate::creature::genome::{BackendDef, CreatureGenome, VmInstruction};

/// The census version the goal summary records.
pub const DRIVES_VERSION: &str = "drives-v1";

/// The weight magnitude above which an edge carries a path.
pub const PATH_WEIGHT_FLOOR: f32 = 1e-3;

const KINDS: usize = DirectedVoteKind::ALL.len();

/// One directed kind's readings over a population.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KindDrives {
    /// Creatures with at least one drive path for the kind.
    pub drive_carriers: u64,
    /// Graph edges into the drive with `|w| > PATH_WEIGHT_FLOOR`.
    pub drive_edges: u64,
    pub drive_edges_positive: u64,
    pub drive_edges_negative: u64,
    /// VM `AddVote` instructions into the drive.
    pub drive_vm_paths: u64,
    /// Creatures with at least one fan-out of the kind.
    pub fanout_creatures: u64,
    pub fanouts: u64,
    pub fanout_edges: u64,
}

/// The census of one population, per kind in `DirectedVoteKind::ALL` order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DriveCensus {
    pub creatures: u64,
    pub kinds: [KindDrives; KINDS],
}

fn carries(weight: f32) -> bool {
    weight.abs() > PATH_WEIGHT_FLOOR
}

/// One Graph node's fan-outs of `kind`: `(fanouts, fanout_edges)`. A group's
/// members are not a fan-out (T25.F03): the census reads independent edges.
fn node_fanouts(def: &CgpGraphBackendDef, kind: DirectedVoteKind) -> (u64, u64) {
    let mut by_source: HashMap<(GraphSource, bool), (BTreeSet<u8>, u64)> = HashMap::new();
    for sink in def.output_sinks() {
        let OutputSinkKind::ActionVote(vote) = sink.kind else {
            continue;
        };
        let Some((sink_kind, d)) = DirectedVoteKind::of_sink(vote) else {
            continue;
        };
        if sink_kind != kind {
            continue;
        }
        for edge in sink
            .inputs
            .iter()
            .filter(|edge| edge.membership().is_none() && carries(edge.own_weight()))
        {
            let entry = by_source
                .entry((edge.source(), edge.own_weight() > 0.0))
                .or_default();
            entry.0.insert(d);
            entry.1 += 1;
        }
    }
    by_source
        .values()
        .filter(|(directions, _)| directions.len() >= 2)
        .fold((0, 0), |(fanouts, edges), (_, count)| {
            (fanouts + 1, edges + count)
        })
}

impl DriveCensus {
    /// Census `creatures`, each a genome and its reachable node indices.
    pub fn observe<'a>(
        creatures: impl IntoIterator<Item = (&'a CreatureGenome, &'a [usize])>,
    ) -> Self {
        let mut census = Self::default();
        for (genome, reachable) in creatures {
            census.observe_one(genome, reachable);
        }
        census
    }

    fn observe_one(&mut self, genome: &CreatureGenome, reachable: &[usize]) {
        self.creatures += 1;
        let mut one = [KindDrives::default(); KINDS];
        for node in reachable.iter().filter_map(|&idx| genome.nodes.get(idx)) {
            match &node.backend_def {
                BackendDef::Graph(def) => {
                    for kind in DirectedVoteKind::ALL {
                        let row = &mut one[kind.index()];
                        if let Some(sink) = def.sink(OutputSinkKind::ActionVote(kind.drive())) {
                            for edge in sink.inputs.iter().filter(|edge| carries(edge.own_weight()))
                            {
                                row.drive_edges += 1;
                                if edge.own_weight() > 0.0 {
                                    row.drive_edges_positive += 1;
                                } else {
                                    row.drive_edges_negative += 1;
                                }
                            }
                        }
                        let (fanouts, edges) = node_fanouts(def, kind);
                        row.fanouts += fanouts;
                        row.fanout_edges += edges;
                    }
                }
                BackendDef::Vm(vm) => {
                    for instruction in &vm.program {
                        let VmInstruction::AddVote { sink, .. } = instruction else {
                            continue;
                        };
                        if let Some(VoteSink::Drive(kind)) =
                            VoteSink::from_index(usize::from(*sink))
                        {
                            one[kind.index()].drive_vm_paths += 1;
                        }
                    }
                }
            }
        }
        for (total, row) in self.kinds.iter_mut().zip(one) {
            total.drive_carriers += u64::from(row.drive_edges + row.drive_vm_paths > 0);
            total.drive_edges += row.drive_edges;
            total.drive_edges_positive += row.drive_edges_positive;
            total.drive_edges_negative += row.drive_edges_negative;
            total.drive_vm_paths += row.drive_vm_paths;
            total.fanout_creatures += u64::from(row.fanouts > 0);
            total.fanouts += row.fanouts;
            total.fanout_edges += row.fanout_edges;
        }
    }

    /// Fan-outs and fan-out edges summed over the kinds.
    #[must_use]
    pub fn fanout_totals(&self) -> (u64, u64) {
        self.kinds.iter().fold((0, 0), |(fanouts, edges), row| {
            (fanouts + row.fanouts, edges + row.fanout_edges)
        })
    }
}

#[cfg(test)]
#[path = "drives_tests.rs"]
mod tests;
