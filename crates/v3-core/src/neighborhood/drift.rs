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
    pub hop_cap_hits: u64,
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
        self.hop_cap_hits += reading.hop_cap_hits as u64;
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

/// Retain every production birth unconditionally. Checkpoint reads borrow genomes
/// and use separate trial RNGs; they never consume the persistent walk streams.
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
        fn mesh_pooling_sums_counts_and_preserves_bounds(rows in prop::collection::vec((0usize..20, 0usize..20, 0usize..20, 0usize..20, any::<bool>(), 0usize..81), 0..50)) {
            let readings: Vec<_> = rows.into_iter().map(|(total, reach, exec, knockout, varies, cap)| {
                let reach = reach.min(total);
                let exec = exec.min(reach);
                MeshExecutionReading { backends: super::super::mesh_execution::MeshBackendCounts {
                    graph: super::super::mesh_execution::BackendNodeCounts {total: (total / 2) as u64, executed: (exec / 2) as u64, contributing: ((exec-knockout.min(exec))/2) as u64},
                    vm: super::super::mesh_execution::BackendNodeCounts {total: (total-total/2) as u64, executed: (exec-exec/2) as u64, contributing: ((exec-knockout.min(exec))-(exec-knockout.min(exec))/2) as u64},
                }, total_node_count: total, reachable_node_count: reach, executed_node_count: exec, knockout_count: knockout.min(exec), route_varies_with_input: varies, hop_cap_hits: cap }
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
            prop_assert_eq!(pooled.hop_cap_hits, readings.iter().map(|r| r.hop_cap_hits as u64).sum::<u64>());
            prop_assert!(pooled.knockout_nodes <= pooled.executed_nodes && pooled.executed_nodes <= pooled.reachable_nodes && pooled.reachable_nodes <= pooled.total_nodes);
            prop_assert!(pooled.route_varying_lineages <= pooled.lineages);
            prop_assert!(pooled.hop_cap_hits <= u64::from(pooled.lineages) * 80);
        }
    }

    #[test]
    fn checkpoint_reads_preserve_lineage_genomes_and_do_not_restart_dead_parents() {
        use crate::contracts::WorldAction;
        use crate::creature::genome::{BackendDef, VmInstruction};
        let config = SimulationConfig::default();
        let context = EvalContext::from_config(&config);
        let battery = Battery::generate(context.food_type_count);
        let mut dead = founder_genome(FounderProfile::V3Alpha1);
        for node in &mut dead.nodes {
            if let BackendDef::Vm(vm) = &mut node.backend_def {
                vm.program = vec![VmInstruction::Halt];
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
        let config = SimulationConfig::default();
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
        let config = SimulationConfig::default();
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
        let config = SimulationConfig::default();
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
}
