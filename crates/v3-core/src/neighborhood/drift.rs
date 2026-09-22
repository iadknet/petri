//! Mutation-only lineage depth observations, isolated from ecological state.

use super::mesh_execution::{indices_for_node_ids, MeshExecutionReading, MeshExecutionSets};
use super::recruitment::{BirthObservation, RecruitmentCheckpoint, RecruitmentTracker};
use super::{births, Battery, BirthResult, EvalContext};
use crate::config::MutationConfig;
use crate::contracts::NodeId;
use crate::creature::genome::{analysis::mesh_reachable_nodes, CreatureGenome};
use crate::mutation::reachability::ParentExecuted;
use crate::mutation::MutationEngine;
use rand::{rngs::SmallRng, SeedableRng};
use rayon::prelude::*;
use std::collections::BTreeSet;

pub const VERSION: &str = "drift-depth-v3";
/// How the walk keeps each lineage's executed node set current (T11.F17).
pub const EXECUTED_SOURCE: &str = "battery hop records (mesh-execution-v1), node ids";
pub const EXECUTED_REFRESH: &str =
    "walk: depth 0 and every 10 generations; births: derived at each checkpoint";
/// Generations between executed-set refreshes along the walk.
pub const EXECUTED_REFRESH_INTERVAL: u64 = 10;
pub const WALK_SEED_BASE: u64 = 90_000;
pub const BIRTH_OFFSET_BASE: u64 = 7_000_000;
pub const BIRTH_LINEAGE_MULTIPLIER: u64 = 1_000;

/// Fixed production experiment; smaller explicit values support internal fixtures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DriftSizes {
    pub lineages: u32,
    pub birth_lineages: u32,
    pub births: u32,
    pub checkpoints: &'static [u64],
}
impl DriftSizes {
    pub const PRODUCTION: Self = Self {
        lineages: 50,
        birth_lineages: 20,
        births: 100,
        checkpoints: &[0, 22, 250, 1_000, 2_000],
    };
}
impl Default for DriftSizes {
    fn default() -> Self {
        Self {
            lineages: 2,
            birth_lineages: 1,
            births: 2,
            checkpoints: &[0, 2],
        }
    }
}

/// Integer totals over whole lineages, never VM instructions or graph internals.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MeshTotals {
    pub backends: super::mesh_execution::MeshBackendCounts,
    pub lineages: u32,
    pub total_nodes: u64,
    pub reachable_nodes: u64,
    pub executed_nodes: u64,
    pub knockout_nodes: u64,
    pub route_varying_lineages: u32,
    /// Battery executions by tick reason, summed over lineages (T19.F04).
    pub tick_reasons: super::mesh_execution::TickReasonCounts,
    /// Passes summed over lineages (T19.F04).
    pub passes: u64,
    /// `Decide`-ended passes summed over lineages (T19.F04).
    pub decided_passes: u64,
    /// Capped passes summed over lineages (T19.F02).
    pub pass_cap_hits: u64,
    /// Lineages whose reachable mesh contains a cycle (T19.F02).
    pub cycle_carrying_lineages: u32,
    /// Lineages that dispatched some node more than once in one execution (T19.F02).
    pub revisiting_lineages: u32,
    /// Lineages with a dispatched cycle node in a non-`NoOp` execution (T19.F02).
    pub productive_cycle_lineages: u32,
}
impl MeshTotals {
    fn record(&mut self, reading: MeshExecutionReading) {
        for (total, sample) in [
            (&mut self.backends.graph, reading.backends.graph),
            (&mut self.backends.vm, reading.backends.vm),
        ] {
            total.total += sample.total;
            total.executed += sample.executed;
            total.contributing += sample.contributing;
        }
        self.lineages += 1;
        self.total_nodes += reading.total_node_count as u64;
        self.reachable_nodes += reading.reachable_node_count as u64;
        self.executed_nodes += reading.executed_node_count as u64;
        self.knockout_nodes += reading.knockout_count as u64;
        self.route_varying_lineages += u32::from(reading.route_varies_with_input);
        self.tick_reasons.add(&reading.tick_reasons);
        self.passes += reading.passes as u64;
        self.decided_passes += reading.decided_passes as u64;
        self.pass_cap_hits += reading.pass_cap_hits as u64;
        self.cycle_carrying_lineages += u32::from(reading.cycle_carrying);
        self.revisiting_lineages += u32::from(reading.revisiting);
        self.productive_cycle_lineages += u32::from(reading.productive_cycle);
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Checkpoint {
    pub depth: u64,
    pub mesh: MeshTotals,
    pub births: BirthResult,
}

/// One walk's readings: the unchanged depth checkpoints, and beside them the
/// module recruitment reading taken at the same depths (T13.F01).
#[derive(Debug, Clone, PartialEq)]
pub struct DriftWalk {
    pub checkpoints: Vec<Checkpoint>,
    pub recruitment: Vec<RecruitmentCheckpoint>,
}

/// The supply rule and values a walk over `mutation` runs, for the report's
/// `supply_rule` metadata.
#[must_use]
pub fn supply_rule(mutation: &MutationConfig) -> String {
    format!(
        "legacy per-birth rule (per_unit_supply_enabled forced false): \
         mutation_probability {}, events {} to {}, continuation {}",
        mutation.mutation_probability,
        mutation.per_birth_mutation_events_min,
        mutation.per_birth_mutation_events_max,
        mutation.per_birth_mutation_event_continuation_probability,
    )
}

/// Retain every production birth unconditionally. Checkpoint reads borrow genomes
/// and use separate trial RNGs; they never consume the persistent walk streams.
///
/// Every walk birth and checkpoint birth runs the legacy per-birth supply rule
/// (see [`supply_rule`]); the config's per-unit fields are ignored here.
#[must_use]
pub fn observe(
    founder: &CreatureGenome,
    battery: &Battery,
    mutation: &MutationConfig,
    context: &EvalContext,
    sizes: DriftSizes,
) -> DriftWalk {
    assert!(sizes.birth_lineages <= sizes.lineages);
    assert!(sizes.checkpoints.windows(2).all(|pair| pair[0] < pair[1]));
    // The walk stays the fixed-count control whatever supply rule production
    // selects (T11.F19); nothing else is overridden.
    let mutation = &mutation.clone().with_legacy_supply();
    let mut genomes: Vec<_> = (0..sizes.lineages).map(|_| founder.clone()).collect();
    let mut rngs: Vec<_> = (0..sizes.lineages)
        .map(|index| SmallRng::seed_from_u64(WALK_SEED_BASE + u64::from(index)))
        .collect();
    let mut depth = 0;
    // The recruitment observation reads the genomes and summaries the walk
    // already produces; it consumes no RNG and runs no extra battery pass.
    let mut tracker = RecruitmentTracker::new(sizes.lineages);
    for lineage in 0..sizes.lineages {
        tracker.seed_founder(lineage, &founder.nodes);
    }
    // Each lineage's executed nodes, by node id so the set survives the index
    // shuffling of intervening births. Refreshed on the predeclared cadence:
    // between refreshes removed nodes drop out and added nodes wait.
    let mut executed_ids = refresh_executed_ids(&genomes, battery, context);
    record_dispatch(&mut tracker, &executed_ids, depth);
    let mut readings = Vec::with_capacity(sizes.checkpoints.len());
    let mut recruitment = Vec::with_capacity(sizes.checkpoints.len());
    for &checkpoint in sizes.checkpoints {
        while depth < checkpoint {
            if depth > 0 && depth.is_multiple_of(EXECUTED_REFRESH_INTERVAL) {
                executed_ids = refresh_executed_ids(&genomes, battery, context);
                record_dispatch(&mut tracker, &executed_ids, depth);
            }
            let born = depth + 1;
            for (lineage, ((genome, rng), ids)) in genomes
                .iter_mut()
                .zip(&mut rngs)
                .zip(&executed_ids)
                .enumerate()
            {
                let reachable = mesh_reachable_nodes(genome);
                let executed = indices_for_node_ids(genome, ids);
                let summary = MutationEngine::apply_mutations_with_food_type_count(
                    genome,
                    mutation,
                    &reachable,
                    ParentExecuted::Indices(&executed),
                    rng,
                    context.food_type_count,
                );
                tracker.record_birth(BirthObservation {
                    lineage: lineage as u32,
                    depth: born,
                    after: &genome.nodes,
                    summary: &summary,
                });
            }
            depth += 1;
        }
        // A checkpoint's births derive their own executed set from the battery
        // inside `births::per_birth_result`. The walk's own sets stay on the
        // fixed interval, so checkpoint placement never changes the walk.
        let (row, sets) = observe_checkpoint(&genomes, battery, mutation, context, sizes, depth);
        for (lineage, lineage_sets) in sets.iter().enumerate() {
            tracker.record_reading(
                lineage as u32,
                depth,
                &lineage_sets.executed,
                Some(&lineage_sets.contributing),
            );
        }
        readings.push(row);
        recruitment.push(tracker.checkpoint(depth));
    }
    DriftWalk {
        checkpoints: readings,
        recruitment,
    }
}

/// Fold an executed-set refresh into the module table. The walk's own executed
/// sets are untouched: this only dates each module's first dispatch.
fn record_dispatch(
    tracker: &mut RecruitmentTracker,
    executed_ids: &[BTreeSet<NodeId>],
    depth: u64,
) {
    for (lineage, ids) in executed_ids.iter().enumerate() {
        tracker.record_reading(lineage as u32, depth, ids, None);
    }
}

/// Re-read every lineage's executed node ids from the fixed battery. Pure per
/// lineage, so the parallel walk is deterministic.
fn refresh_executed_ids(
    genomes: &[CreatureGenome],
    battery: &Battery,
    context: &EvalContext,
) -> Vec<BTreeSet<NodeId>> {
    genomes
        .par_iter()
        .map(|genome| {
            battery.executed_node_ids(genome, context.runtime, context.shared_memory_decay_rate)
        })
        .collect()
}

/// A checkpoint row and, per lineage, the node ids behind its mesh counts:
/// the same single battery pass, read once (T13.F01).
fn observe_checkpoint(
    genomes: &[CreatureGenome],
    battery: &Battery,
    mutation: &MutationConfig,
    context: &EvalContext,
    sizes: DriftSizes,
    depth: u64,
) -> (Checkpoint, Vec<MeshExecutionSets>) {
    let mut row = Checkpoint {
        depth,
        ..Checkpoint::default()
    };
    let mut sets = Vec::with_capacity(genomes.len());
    for (index, genome) in genomes.iter().enumerate() {
        let reading =
            battery.mesh_execution_sets(genome, context.runtime, context.shared_memory_decay_rate);
        row.mesh.record(reading.reading);
        sets.push(reading);
        if index < sizes.birth_lineages as usize {
            let base = battery.signature(genome, context.runtime, context.shared_memory_decay_rate);
            row.births = row.births.merge(&births::per_birth_result(
                genome,
                &base,
                battery,
                mutation,
                context,
                sizes.births,
                BIRTH_OFFSET_BASE + BIRTH_LINEAGE_MULTIPLIER * (index as u64 + 1) + depth,
            ));
        }
    }
    (row, sets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FounderProfile, SimulationConfig};
    use crate::creature::founder::founder_genome;

    use proptest::prelude::*;

    /// The production config on the legacy supply rule the walk forces, so a
    /// hand replay of the walk's births draws the same counts.
    fn replay_config() -> SimulationConfig {
        let mut config = SimulationConfig::default();
        config.mutation = config.mutation.with_legacy_supply();
        config
    }

    /// Mirror [`observe`]'s executed-set cadence for a single-lineage replay:
    /// refresh at depth 0 and at every positive multiple of the interval,
    /// otherwise reuse the cached node ids mapped to the current genome.
    fn replay_executed(
        depth: u64,
        genome: &CreatureGenome,
        battery: &Battery,
        context: &EvalContext,
        ids: &mut BTreeSet<NodeId>,
    ) -> Vec<usize> {
        if depth.is_multiple_of(EXECUTED_REFRESH_INTERVAL) {
            *ids = battery.executed_node_ids(
                genome,
                context.runtime,
                context.shared_memory_decay_rate,
            );
        }
        indices_for_node_ids(genome, ids)
    }

    proptest! {
        #[test]
        fn mesh_pooling_sums_counts_and_preserves_bounds(rows in prop::collection::vec((0usize..20, 0usize..20, 0usize..20, 0usize..20, any::<bool>(), 0usize..81, any::<[bool; 3]>()), 0..50)) {
            let readings: Vec<_> = rows.into_iter().map(|(total, reach, exec, knockout, varies, cap, cycles)| {
                let reach = reach.min(total);
                let exec = exec.min(reach);
                MeshExecutionReading { backends: super::super::mesh_execution::MeshBackendCounts {
                    graph: super::super::mesh_execution::BackendNodeCounts {total: (total / 2) as u64, executed: (exec / 2) as u64, contributing: ((exec-knockout.min(exec))/2) as u64},
                    vm: super::super::mesh_execution::BackendNodeCounts {total: (total-total/2) as u64, executed: (exec-exec/2) as u64, contributing: ((exec-knockout.min(exec))-(exec-knockout.min(exec))/2) as u64},
                }, total_node_count: total, reachable_node_count: reach, executed_node_count: exec, knockout_count: knockout.min(exec), route_varies_with_input: varies, route_destination_varies: varies, tick_reasons: super::super::mesh_execution::TickReasonCounts { no_decision: cap, ..Default::default() }, passes: 2 * cap, decided_passes: cap, pass_cap_hits: cap, cycle_carrying: cycles[0], revisiting: cycles[1], productive_cycle: cycles[2] }
            }).collect();
            let mut pooled = MeshTotals::default();
            for &reading in &readings { pooled.record(reading); }
            prop_assert_eq!(pooled.lineages as usize, readings.len());
            prop_assert_eq!(pooled.total_nodes, readings.iter().map(|r| r.total_node_count as u64).sum::<u64>());
            prop_assert_eq!(pooled.reachable_nodes, readings.iter().map(|r| r.reachable_node_count as u64).sum::<u64>());
            prop_assert_eq!(pooled.executed_nodes, readings.iter().map(|r| r.executed_node_count as u64).sum::<u64>());
            for (actual, graph) in [(&pooled.backends.graph, true), (&pooled.backends.vm, false)] {
                let samples: Vec<_> = readings.iter().map(|r| if graph { r.backends.graph } else { r.backends.vm }).collect();
                prop_assert_eq!(actual.total, samples.iter().map(|s| s.total).sum::<u64>());
                prop_assert_eq!(actual.executed, samples.iter().map(|s| s.executed).sum::<u64>());
                prop_assert_eq!(actual.contributing, samples.iter().map(|s| s.contributing).sum::<u64>());
            }
            prop_assert_eq!(pooled.knockout_nodes, readings.iter().map(|r| r.knockout_count as u64).sum::<u64>());
            prop_assert_eq!(pooled.route_varying_lineages as usize, readings.iter().filter(|r| r.route_varies_with_input).count());
            prop_assert_eq!(pooled.tick_reasons.total(), readings.iter().map(|r| r.tick_reasons.total()).sum::<usize>());
            prop_assert_eq!(pooled.passes, readings.iter().map(|r| r.passes as u64).sum::<u64>());
            prop_assert_eq!(pooled.decided_passes, readings.iter().map(|r| r.decided_passes as u64).sum::<u64>());
            prop_assert!(pooled.knockout_nodes <= pooled.executed_nodes && pooled.executed_nodes <= pooled.reachable_nodes && pooled.reachable_nodes <= pooled.total_nodes);
            prop_assert!(pooled.route_varying_lineages <= pooled.lineages);
            prop_assert!(pooled.pass_cap_hits <= pooled.passes);
            prop_assert_eq!(pooled.pass_cap_hits, readings.iter().map(|r| r.pass_cap_hits as u64).sum::<u64>());
            prop_assert_eq!(pooled.cycle_carrying_lineages as usize, readings.iter().filter(|r| r.cycle_carrying).count());
            prop_assert_eq!(pooled.revisiting_lineages as usize, readings.iter().filter(|r| r.revisiting).count());
            prop_assert_eq!(pooled.productive_cycle_lineages as usize, readings.iter().filter(|r| r.productive_cycle).count());
            prop_assert!(pooled.cycle_carrying_lineages.max(pooled.revisiting_lineages).max(pooled.productive_cycle_lineages) <= pooled.lineages);
        }
    }

    #[test]
    fn checkpoint_reads_preserve_lineage_genomes_and_do_not_restart_dead_parents() {
        use crate::contracts::WorldAction;
        use crate::creature::genome::BackendDef;
        let config = replay_config();
        let context = EvalContext::from_config(&config);
        let battery = Battery::generate(context.food_type_count);
        // The founder with every vote and parameter edge removed votes
        // nothing, so every execution is `NoOp`.
        let mut dead = founder_genome(FounderProfile::V3Alpha1);
        for node in &mut dead.nodes {
            if let BackendDef::Graph(graph) = &mut node.backend_def {
                for sink in &mut graph.output_sinks {
                    if matches!(
                        sink.kind,
                        crate::creature::genome::cgp::OutputSinkKind::ActionVote(_)
                            | crate::creature::genome::cgp::OutputSinkKind::ActionParam(_, _)
                    ) {
                        sink.inputs.clear();
                    }
                }
            }
        }
        let base = battery.signature(&dead, context.runtime, context.shared_memory_decay_rate);
        assert!(base
            .snapshots
            .iter()
            .chain(base.sequences.iter().flatten())
            .all(|actions| actions == &[WorldAction::NoOp]));
        let sizes = DriftSizes {
            lineages: 1,
            birth_lineages: 1,
            births: 3,
            checkpoints: &[0, 22],
        };
        let actual = observe(&dead, &battery, &config.mutation, &context, sizes).checkpoints;
        let mut genomes = vec![dead];
        let mut rng = SmallRng::seed_from_u64(90_000);
        let mut zero_births = 0;
        let mut applied_dead_births = 0;
        let mut ids = BTreeSet::new();
        for depth in 1..=22 {
            let reachable = mesh_reachable_nodes(&genomes[0]);
            let executed = replay_executed(depth - 1, &genomes[0], &battery, &context, &mut ids);
            let summary = MutationEngine::apply_mutations_with_food_type_count(
                &mut genomes[0],
                &config.mutation,
                &reachable,
                ParentExecuted::Indices(&executed),
                &mut rng,
                context.food_type_count,
            );
            zero_births += u32::from(summary.applied_events == 0);
            let signature = battery.signature(
                &genomes[0],
                context.runtime,
                context.shared_memory_decay_rate,
            );
            if summary.applied_events > 0
                && signature
                    .snapshots
                    .iter()
                    .chain(signature.sequences.iter().flatten())
                    .all(|actions| actions == &[WorldAction::NoOp])
            {
                applied_dead_births += 1;
            }
            if depth == 3 {
                let before = genomes.clone();
                let rng_before = rng.clone();
                let _ = observe_checkpoint(
                    &genomes,
                    &battery,
                    &config.mutation,
                    &context,
                    sizes,
                    depth,
                )
                .0;
                let _ = observe_checkpoint(
                    &genomes,
                    &battery,
                    &config.mutation,
                    &context,
                    DriftSizes { births: 7, ..sizes },
                    depth,
                );
                assert_eq!(genomes, before);
                // The next production offspring is identical with or without the observations.
                let mut without = before[0].clone();
                let mut with = genomes[0].clone();
                let next_executed = indices_for_node_ids(&genomes[0], &ids);
                MutationEngine::apply_mutations_with_food_type_count(
                    &mut without,
                    &config.mutation,
                    &mesh_reachable_nodes(&before[0]),
                    ParentExecuted::Indices(&next_executed),
                    &mut rng_before.clone(),
                    context.food_type_count,
                );
                MutationEngine::apply_mutations_with_food_type_count(
                    &mut with,
                    &config.mutation,
                    &mesh_reachable_nodes(&genomes[0]),
                    ParentExecuted::Indices(&next_executed),
                    &mut rng.clone(),
                    context.food_type_count,
                );
                assert_eq!(with, without);
            }
        }
        assert!(zero_births > 0);
        assert!(applied_dead_births > 0);
        assert_eq!(
            actual[1],
            observe_checkpoint(&genomes, &battery, &config.mutation, &context, sizes, 22).0
        );
    }

    #[test]
    fn checkpoints_replay_the_production_walk_and_fixed_birth_subset() {
        let config = replay_config();
        let founder = founder_genome(FounderProfile::V3Alpha1);
        let before = founder.clone();
        let battery = Battery::generate(2);
        let context = EvalContext::from_config(&config);
        let sizes = DriftSizes {
            lineages: 3,
            birth_lineages: 2,
            births: 4,
            checkpoints: &[0, 3, 22],
        };
        let walk = observe(&founder, &battery, &config.mutation, &context, sizes);
        let actual = walk.checkpoints;
        let mut expected = vec![Checkpoint::default(); 3];
        let mut applied = 0;
        for index in 0..3 {
            let mut genome = founder.clone();
            let mut rng = SmallRng::seed_from_u64(90_000 + index);
            let mut ids = BTreeSet::new();
            for depth in 0..=22 {
                if depth > 0 {
                    let reachable = mesh_reachable_nodes(&genome);
                    let executed =
                        replay_executed(depth - 1, &genome, &battery, &context, &mut ids);
                    applied += MutationEngine::apply_mutations_with_food_type_count(
                        &mut genome,
                        &config.mutation,
                        &reachable,
                        ParentExecuted::Indices(&executed),
                        &mut rng,
                        context.food_type_count,
                    )
                    .applied_events;
                }
                if let Some(slot) = sizes.checkpoints.iter().position(|&d| d == depth) {
                    let row = &mut expected[slot];
                    row.depth = depth;
                    row.mesh.record(battery.mesh_execution(
                        &genome,
                        context.runtime,
                        context.shared_memory_decay_rate,
                    ));
                    if index < 2 {
                        let base = battery.signature(
                            &genome,
                            context.runtime,
                            context.shared_memory_decay_rate,
                        );
                        row.births = row.births.clone().merge(&births::per_birth_result(
                            &genome,
                            &base,
                            &battery,
                            &config.mutation,
                            &context,
                            4,
                            7_000_000 + 1_000 * (index + 1) + depth,
                        ));
                    }
                }
            }
        }
        assert!(applied > 0);
        assert_eq!(actual, expected);
        assert_eq!(founder, before);
        assert!(actual
            .iter()
            .all(|r| r.births.births_total == 8 && r.mesh.lineages == 3));
    }

    /// The walk refreshes each lineage's executed node ids every
    /// `EXECUTED_REFRESH_INTERVAL` generations, not only at depth 0: a replay
    /// that keeps the founder's set for the whole walk diverges from
    /// [`observe`], while the replay on the real cadence reproduces it.
    #[test]
    fn the_walk_refreshes_executed_sets_along_the_way_not_only_at_depth_zero() {
        let config = replay_config();
        let founder = founder_genome(FounderProfile::V3Alpha1);
        let battery = Battery::generate(2);
        let context = EvalContext::from_config(&config);
        const DEPTH: u64 = 30;
        let sizes = DriftSizes {
            lineages: 8,
            birth_lineages: 0,
            births: 0,
            checkpoints: &[DEPTH],
        };
        let actual = observe(&founder, &battery, &config.mutation, &context, sizes).checkpoints;

        let replay = |refresh: bool| {
            (0..sizes.lineages)
                .map(|index| {
                    let mut genome = founder.clone();
                    let mut rng = SmallRng::seed_from_u64(WALK_SEED_BASE + u64::from(index));
                    let mut ids = BTreeSet::new();
                    for step in 0..DEPTH {
                        let executed = if refresh || step == 0 {
                            replay_executed(step, &genome, &battery, &context, &mut ids)
                        } else {
                            indices_for_node_ids(&genome, &ids)
                        };
                        let reachable = mesh_reachable_nodes(&genome);
                        MutationEngine::apply_mutations_with_food_type_count(
                            &mut genome,
                            &config.mutation,
                            &reachable,
                            ParentExecuted::Indices(&executed),
                            &mut rng,
                            context.food_type_count,
                        );
                    }
                    genome
                })
                .collect::<Vec<_>>()
        };

        let mut on_cadence = MeshTotals::default();
        let mut stale = MeshTotals::default();
        for (totals, genomes) in [(&mut on_cadence, replay(true)), (&mut stale, replay(false))] {
            for genome in &genomes {
                totals.record(battery.mesh_execution(
                    genome,
                    context.runtime,
                    context.shared_memory_decay_rate,
                ));
            }
        }
        assert_eq!(
            actual[0].mesh, on_cadence,
            "the walk follows the predeclared refresh cadence"
        );
        assert_ne!(
            on_cadence, stale,
            "refreshing mid-walk changes which nodes later births target"
        );
    }

    /// The recruitment observation rides along without changing the walk: a
    /// replay that never touches the tracker reproduces the checkpoints byte
    /// for byte, and the module table it produced describes exactly the nodes
    /// those replayed genomes carry.
    #[test]
    fn recruitment_readings_track_the_walk_without_changing_it() {
        let config = replay_config();
        let founder = founder_genome(FounderProfile::V3Alpha1);
        let battery = Battery::generate(2);
        let context = EvalContext::from_config(&config);
        const DEPTH: u64 = 22;
        let sizes = DriftSizes {
            lineages: 3,
            birth_lineages: 1,
            births: 2,
            checkpoints: &[0, 12, DEPTH],
        };
        let walk = observe(&founder, &battery, &config.mutation, &context, sizes);

        // The same walk with no observation of any kind.
        let mut replayed = Vec::new();
        for index in 0..sizes.lineages {
            let mut genome = founder.clone();
            let mut rng = SmallRng::seed_from_u64(WALK_SEED_BASE + u64::from(index));
            let mut ids = BTreeSet::new();
            for step in 0..DEPTH {
                let executed = replay_executed(step, &genome, &battery, &context, &mut ids);
                let reachable = mesh_reachable_nodes(&genome);
                MutationEngine::apply_mutations_with_food_type_count(
                    &mut genome,
                    &config.mutation,
                    &reachable,
                    ParentExecuted::Indices(&executed),
                    &mut rng,
                    context.food_type_count,
                );
            }
            replayed.push(genome);
        }
        assert_eq!(
            walk.checkpoints[2],
            observe_checkpoint(
                &replayed,
                &battery,
                &config.mutation,
                &context,
                sizes,
                DEPTH
            )
            .0
        );

        let last = &walk.recruitment[2];
        assert_eq!(last.depth, DEPTH);
        assert_eq!(
            last.founders.created,
            founder.nodes.len() as u64 * u64::from(sizes.lineages)
        );
        assert_eq!(
            last.founders.created,
            last.founders.present + last.founders.deleted
        );
        // Present modules are exactly the nodes the replayed genomes carry.
        assert_eq!(
            last.cohort.present + last.founders.present,
            replayed
                .iter()
                .map(|genome| genome.nodes.len() as u64)
                .sum::<u64>()
        );
        assert!(last.cohort.created > 0, "the walk creates modules");
        assert_eq!(
            last.cohort.created,
            last.cohort.present + last.cohort.deleted
        );
        assert_eq!(last.graph.created + last.vm.created, last.cohort.created);
        assert!(last.cohort.contributing <= last.cohort.dispatched());
        // Opportunities pool every birth of every lineage up to the checkpoint.
        assert_eq!(last.opportunities.births, DEPTH * u64::from(sizes.lineages));
        assert_eq!(
            last.opportunities.attempted,
            last.opportunities.applied + last.opportunities.skipped
        );
        assert!(last.opportunities.applied > 0);
        assert_eq!(last.lineage_rows.len(), sizes.lineages as usize);
        assert_eq!(last.lineage_opportunities.len(), sizes.lineages as usize);
        assert_eq!(walk.recruitment[0].retention, None);
        assert!(walk.recruitment[2].retention.is_some());
    }

    #[test]
    fn zero_mutation_and_observation_changes_leave_walk_readings_unchanged() {
        let mut config = SimulationConfig::default();
        let founder = founder_genome(FounderProfile::V3Alpha1);
        let battery = Battery::generate(2);
        let sizes = DriftSizes {
            lineages: 2,
            birth_lineages: 1,
            births: 2,
            checkpoints: &[0, 22],
        };
        let context = EvalContext::from_config(&config);
        let sparse = observe(&founder, &battery, &config.mutation, &context, sizes).checkpoints;
        let dense = observe(
            &founder,
            &battery,
            &config.mutation,
            &context,
            DriftSizes {
                checkpoints: &[0, 3, 22],
                births: 5,
                ..sizes
            },
        )
        .checkpoints;
        assert_eq!(sparse[1].mesh, dense[2].mesh);
        config.mutation.mutation_probability = 0.0;
        let context = EvalContext::from_config(&config);
        let zero = observe(&founder, &battery, &config.mutation, &context, sizes).checkpoints;
        assert_eq!(zero[0].mesh, zero[1].mesh);
        assert!(zero.iter().all(|r| r.births.zero_event_births == 2));
    }

    /// The walk is the fixed-count control (T11.F19): it runs the legacy
    /// per-birth rule whatever supply rule the config it receives selects, so
    /// its births and checkpoint rows are identical either way, and its
    /// metadata names the rule and values in force.
    #[test]
    fn the_walk_forces_the_legacy_supply_rule_whatever_the_config_selects() {
        let config = SimulationConfig::default();
        let founder = founder_genome(FounderProfile::V3Alpha1);
        let battery = Battery::generate(2);
        let sizes = DriftSizes {
            lineages: 3,
            birth_lineages: 2,
            births: 3,
            checkpoints: &[0, 5],
        };
        let context = EvalContext::from_config(&config);
        let mut production_mutation = config.mutation.clone();
        assert!(production_mutation.per_unit_supply_enabled);
        production_mutation.per_unit_rate = 1.0;
        let production = observe(&founder, &battery, &production_mutation, &context, sizes);
        let mut legacy_mutation = config.mutation.clone();
        legacy_mutation.per_unit_supply_enabled = false;
        let legacy = observe(&founder, &battery, &legacy_mutation, &context, sizes);
        assert_eq!(production.checkpoints, legacy.checkpoints);
        assert_eq!(production.recruitment, legacy.recruitment);
        assert_eq!(
            supply_rule(&production_mutation),
            "legacy per-birth rule (per_unit_supply_enabled forced false): \
             mutation_probability 0.44, events 1 to 10, continuation 0.2"
        );
    }

    /// The walk dates a module from the generation its birth produced, and it
    /// folds every executed-set refresh into the module table rather than
    /// waiting for the next checkpoint. Both show up in the cohort's
    /// time-to-first medians, which this deterministic walk pins.
    #[test]
    fn the_walk_dates_modules_from_their_birth_and_from_every_refresh() {
        use crate::neighborhood::recruitment::{CohortFact, TimeToFirst};
        let config = SimulationConfig::default();
        let context = EvalContext::from_config(&config);
        let battery = Battery::generate(context.food_type_count);
        let founder = founder_genome(FounderProfile::V3Alpha1);
        let walk = observe(
            &founder,
            &battery,
            &config.mutation,
            &context,
            DriftSizes {
                lineages: 8,
                birth_lineages: 2,
                births: 2,
                // Past the depth-10 refresh, so a dispatch date can predate
                // the closing checkpoint.
                checkpoints: &[1, 20],
            },
        );
        let reading = &walk.recruitment[1];
        assert_eq!(reading.depth, 20);
        // T13.F03 re-pin: applicability-first selection changes which
        // operator applies at each node-internal event, so this walk's
        // lineages differ from the pre-repair ones; the dating properties
        // the test exists for are asserted on the new walk. Re-pinned again
        // 2026-09-18 when the topology weight table was scaled ten-fold
        // around `ChangeEntryNode` (weight 1 of 211): every topology draw
        // shifts, so the walk's lineages diverge at their first topology
        // event. Re-pinned 2026-09-19 for the 25% large-copy weight default,
        // which again changes topology draws and downstream RNG history.
        // Re-pinned by T19.F04: the 97-unit vote founder draws fewer events
        // per birth and its graph decision node changes every
        // node-internal draw.
        assert_eq!(reading.cohort.created, 12);
        assert_eq!(reading.cohort.dispatched(), 5);
        // Seven cohort modules reached dispatch, the median four generations
        // after the birth that created them: later births, or a dispatch date
        // taken only at the closing checkpoint, would both read higher.
        assert_eq!(
            reading.time_to_first(CohortFact::Dispatch),
            &TimeToFirst {
                reached: 7,
                median_generations: Some(4),
                censored_deleted: 0,
                censored_present: 5,
            },
        );
        // T11.F22 re-pin: the within-kind `Swap` and consumer-preserving
        // `Prune` draw differently from the old `Swap`/`Remove`, so the walk's
        // lineages diverge at their first input-reference event; two cohort
        // modules reached an internal change. With the 25% copy default,
        // three reach an internal change, still a median five generations
        // after birth; on the T19.F04 vote founder six do, at the same
        // median.
        assert_eq!(
            reading.time_to_first(CohortFact::InternalChange),
            &TimeToFirst {
                reached: 6,
                median_generations: Some(5),
                censored_deleted: 1,
                censored_present: 5,
            },
        );
    }
}
