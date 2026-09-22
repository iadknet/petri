//! One-edge census (T19.F04 reading): every unit-weight edge from an
//! input-leaf sub-value of one Graph node into each of the 27 vote sinks,
//! added one at a time and classified against the unedited genome on the
//! `neighborhood-v1` battery.
//!
//! The rows a single edge can land in are the transition contract's worked
//! cases, pinned exactly by the executor fixtures in
//! `runtime/pass_loop_tests.rs`: into an unused kind it adds one action (W4),
//! into a used kind's other sink it steers (W5), into a sink already voted on
//! it recounts and reorders (W6), at a larger weight it bursts (W7), and into
//! `Terminate` it can end a plan (W10). The census counts how often each
//! happens on a real genome; it asserts none of them.
//!
//! Observation only: nothing here changes a genome in the population.

use rayon::prelude::*;

use crate::config::MutationConfig;
use crate::creature::genome::cgp::{GraphEdge, GraphSource, OutputSinkKind};
use crate::creature::genome::vote::VoteSink;
use crate::creature::genome::{BackendDef, CreatureGenome};
use crate::mutation::compound::sub_value_count;
use crate::neighborhood::battery::Battery;
use crate::neighborhood::classify::{classify, Tally};
use crate::neighborhood::EvalContext;

/// One vote sink's tally over every input-leaf edge into it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SinkRow {
    pub sink: VoteSink,
    pub tally: Tally,
}

/// The census of one Graph node: one row per vote sink in catalog order, and
/// their merge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneEdgeCensus {
    /// Input-leaf sub-values of the node, so every row holds this many trials.
    pub sub_values: u32,
    pub rows: Vec<SinkRow>,
    pub total: Tally,
}

/// Every input-leaf source of `node`, in reference then sub-value order.
fn input_leaves(genome: &CreatureGenome, node: usize, config: &MutationConfig) -> Vec<GraphSource> {
    genome.nodes[node]
        .input_refs
        .iter()
        .enumerate()
        .flat_map(|(ref_idx, reference)| {
            let width = sub_value_count(reference, config).max(1);
            (0..width).map(move |sub_idx| GraphSource::InputLeaf {
                ref_idx: ref_idx as u16,
                sub_idx,
            })
        })
        .collect()
}

/// `genome` with one unit-weight edge from `source` into `sink` on `node`.
///
/// # Panics
///
/// If `node` is not a Graph node or lacks the `sink` vote sink; the census
/// is defined only on Graph nodes carrying the fixed sink catalog.
fn with_edge(
    genome: &CreatureGenome,
    node: usize,
    source: GraphSource,
    sink: VoteSink,
) -> CreatureGenome {
    let mut edited = genome.clone();
    let BackendDef::Graph(graph) = &mut edited.nodes[node].backend_def else {
        panic!("the one-edge census reads a Graph node");
    };
    graph
        .sink_mut(OutputSinkKind::ActionVote(sink))
        .expect("every Graph node carries the fixed vote sinks")
        .inputs
        .push(GraphEdge {
            source,
            weight: 1.0,
        });
    edited
}

/// Run the one-edge census on Graph node `node` of `genome`.
#[must_use]
pub fn one_edge_census(
    genome: &CreatureGenome,
    node: usize,
    battery: &Battery,
    mutation_config: &MutationConfig,
    context: &EvalContext,
) -> OneEdgeCensus {
    let base = battery.signature(genome, context.runtime, context.shared_memory_decay_rate);
    let leaves = input_leaves(genome, node, mutation_config);
    let rows: Vec<SinkRow> = VoteSink::all()
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|sink| {
            let tally = leaves.iter().fold(Tally::default(), |tally, &source| {
                let edited = with_edge(genome, node, source, sink);
                let signature =
                    battery.signature(&edited, context.runtime, context.shared_memory_decay_rate);
                tally.record(classify(&base, &signature))
            });
            SinkRow { sink, tally }
        })
        .collect();
    let total = rows
        .iter()
        .fold(Tally::default(), |total, row| total.merge(row.tally));
    OneEdgeCensus {
        sub_values: leaves.len() as u32,
        rows,
        total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FounderProfile;
    use crate::config::SimulationConfig;
    use crate::creature::founder::founder_genome;
    use crate::creature::genome::vote::VOTE_SINK_COUNT;

    fn founder_census() -> OneEdgeCensus {
        let config = SimulationConfig::default();
        let context = EvalContext::from_config(&config);
        let battery = Battery::generate(context.food_type_count);
        one_edge_census(
            &founder_genome(FounderProfile::V3Alpha1),
            1,
            &battery,
            &config.mutation,
            &context,
        )
    }

    /// Every sub-value of node 1 is tried once into every vote sink, the
    /// rows come in catalog order, and the total is their merge. No trial
    /// skips (the edge always applies) and a shape is only ever counted
    /// inside `Changed`.
    #[test]
    fn the_founder_census_covers_every_leaf_and_sink_once() {
        let census = founder_census();
        assert!(census.sub_values > 0);
        assert_eq!(census.rows.len(), VOTE_SINK_COUNT);
        for (index, row) in census.rows.iter().enumerate() {
            assert_eq!(row.sink.index(), index);
            assert_eq!(row.tally.trials, census.sub_values);
            assert_eq!(row.tally.skipped, 0);
            assert!(row.tally.reordered + row.tally.recount <= row.tally.changed);
        }
        assert_eq!(
            census.total.trials,
            census.sub_values * VOTE_SINK_COUNT as u32
        );
        assert_eq!(
            census.total.silent + census.total.changed + census.total.dead,
            census.total.trials
        );
    }

    /// A unit edge into the founder's unused `StealEnergy` kind never makes
    /// it dead: the edge only adds votes to a kind the founder never commits
    /// (W4's row), so the founder's own commits survive.
    #[test]
    fn a_unit_edge_into_an_unused_kind_never_kills_the_founder() {
        let census = founder_census();
        for row in census
            .rows
            .iter()
            .filter(|row| matches!(row.sink, VoteSink::StealEnergy(_)))
        {
            assert_eq!(row.tally.dead, 0, "{:?}", row.sink);
        }
    }

    /// Prints the readings-file census table. Run with
    /// `cargo test -p v3-core --release one_edge_census_print -- --ignored --nocapture`.
    #[test]
    #[ignore = "prints the readings-file table"]
    fn one_edge_census_print() {
        let census = founder_census();
        println!("sub_values {}", census.sub_values);
        println!("| Sink | Silent | Changed | Reordered | Recount | Dead |");
        println!("| --- | ---: | ---: | ---: | ---: | ---: |");
        for row in &census.rows {
            let t = row.tally;
            println!(
                "| {:?} | {} | {} | {} | {} | {} |",
                row.sink, t.silent, t.changed, t.reordered, t.recount, t.dead
            );
        }
        let t = census.total;
        println!(
            "| total | {} | {} | {} | {} | {} |",
            t.silent, t.changed, t.reordered, t.recount, t.dead
        );
    }
}
