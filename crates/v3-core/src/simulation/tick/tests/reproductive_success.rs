use super::support::*;
use crate::contracts::{Direction, NodeId};
use crate::creature::genome::cgp::{
    CgpGraphBackendDef, ComputeNode, ComputeNodeKind, GraphEdge, GraphSource,
};
use crate::creature::genome::{
    BackendDef, CreatureGenome, HebbianRule, PlasticityConfig, VmInstruction,
};
use crate::mutation::MutationOperator;
use crate::simulation::actions::{apply_reproduce, apply_steal_energy, ReproductionActionResult};
use crate::simulation::energy_accounting::DeathCause;
use crate::simulation::reproductive_success::{
    CognitiveClass, ReproductiveSuccessByCognitiveClass, ReproductiveSuccessTotals,
};
use crate::simulation::tick::run_phase_0;
use rand::{rngs::SmallRng, SeedableRng};

fn genome_for(class: CognitiveClass) -> CreatureGenome {
    let mut genome = vm_program_genome(vec![VmInstruction::Halt]);
    match class {
        CognitiveClass::None => {}
        CognitiveClass::SharedMemory => {
            genome = vm_program_genome(vec![VmInstruction::StoreSlotImm {
                slot_idx: 0,
                src: 0,
            }]);
        }
        CognitiveClass::Stateful | CognitiveClass::Plasticity => {
            let mut graph = CgpGraphBackendDef::new_with_fixed_outputs();
            graph.compute_nodes.push(ComputeNode {
                kind: ComputeNodeKind::DecayIntegrator(0.5),
                inputs: vec![GraphEdge {
                    source: GraphSource::SharedMemory {
                        slot: 0,
                        previous: true,
                    },
                    weight: 1.0,
                }],
                plasticity: (class == CognitiveClass::Plasticity).then_some(PlasticityConfig {
                    rule: HebbianRule::Classic,
                    learning_rate: 0.1,
                    weight_clamp: 1.0,
                    lamarckian: false,
                    modulation: None,
                }),
            });
            genome.nodes[0].backend_def = BackendDef::Graph(graph);
        }
    }
    genome
}

#[test]
fn reproductive_success_records_every_class_cause_and_generation_once() {
    for class in CognitiveClass::ALL {
        for (index, cause) in DeathCause::ALL.into_iter().enumerate() {
            let (mut sim, id) = make_sim_with_custom_genome(10.0, genome_for(class));
            let creature = &mut sim.creatures[id];
            creature.age = 23;
            creature.offspring_spawned_count = 7;
            creature.generation = index as u64;
            if index % 2 != 0 {
                creature.birth_mutation_operators =
                    vec![MutationOperator::TopologyAddNode].into_boxed_slice();
            }
            if cause != DeathCause::ExternalRemoval {
                creature.energy = 0.0;
                creature.pending_death_cause = (cause != DeathCause::Unattributed).then_some(cause);
            }

            sim.remove_creature(id);
            sim.remove_creature(id);
            sim.stats.reset_tick_counters();

            assert_eq!(sim.stats.mortality.count(cause), 1);
            let totals = sim.stats.reproductive_success_by_cognitive_class;
            assert_eq!(
                totals.totals(class),
                ReproductiveSuccessTotals {
                    creatures_observed_total: 1,
                    offspring_spawned_sum: 7,
                    survival_ticks_sum: 23,
                }
            );
            assert_eq!(
                CognitiveClass::ALL
                    .into_iter()
                    .map(|c| totals.totals(c).creatures_observed_total)
                    .sum::<u64>(),
                sim.stats.mortality.deaths_total
            );
            for other in CognitiveClass::ALL
                .into_iter()
                .filter(|other| *other != class)
            {
                assert_eq!(totals.totals(other), ReproductiveSuccessTotals::default());
            }
        }
    }
}

#[test]
fn reproductive_success_ignores_unreachable_cognitive_structure() {
    let mut genome = genome_for(CognitiveClass::None);
    let mut unreachable = genome_for(CognitiveClass::Plasticity).nodes.remove(0);
    unreachable.node_id = NodeId::new(1);
    genome.nodes.push(unreachable);
    let (mut sim, id) = make_sim_with_custom_genome(10.0, genome);
    sim.remove_creature(id);
    assert_eq!(
        sim.stats
            .reproductive_success_by_cognitive_class
            .totals(CognitiveClass::None)
            .creatures_observed_total,
        1
    );
    assert_eq!(
        sim.stats
            .reproductive_success_by_cognitive_class
            .totals(CognitiveClass::Plasticity),
        ReproductiveSuccessTotals::default()
    );
}

#[test]
fn reproductive_success_preserves_phase_zero_and_predation_removal_ages() {
    let (mut sim, id) = make_sim_with_custom_genome(1.0, genome_for(CognitiveClass::None));
    sim.creatures[id].age = 11;
    sim.creatures[id].offspring_spawned_count = 3;
    sim.config.energy.lifecycle.energy_decay_per_tick = 2.0;
    sim.config.energy.lifecycle.genome_carry_cost_per_unit = 0.0;
    run_phase_0(&mut sim);
    assert_eq!(sim.stats.mortality.count(DeathCause::LifecycleDecay), 1);
    assert_eq!(
        sim.stats
            .reproductive_success_by_cognitive_class
            .totals(CognitiveClass::None),
        ReproductiveSuccessTotals {
            creatures_observed_total: 1,
            offspring_spawned_sum: 3,
            survival_ticks_sum: 12,
        }
    );

    let (mut sim, attacker, victim) = make_sim_two_creatures(10.0, 2.0);
    sim.creatures[victim].genome = genome_for(CognitiveClass::SharedMemory);
    sim.creatures[victim].age = 19;
    sim.creatures[victim].offspring_spawned_count = 5;
    sim.config.predation.steal_cost_rate = 0.0;
    sim.config.predation.kill_complexity_bonus_multiplier = 0.0;
    apply_steal_energy(attacker, &mut sim, Direction::N, 10.0);
    assert_eq!(sim.stats.mortality.count(DeathCause::Predation), 1);
    assert!(sim.creatures.contains_key(attacker));
    assert_eq!(
        sim.stats
            .reproductive_success_by_cognitive_class
            .totals(CognitiveClass::SharedMemory),
        ReproductiveSuccessTotals {
            creatures_observed_total: 1,
            offspring_spawned_sum: 5,
            survival_ticks_sum: 19,
        }
    );
}

#[test]
fn reproductive_success_counts_successful_births_when_the_parent_is_removed() {
    let (mut sim, parent) = make_sim_with_custom_genome(100.0, genome_for(CognitiveClass::None));
    sim.creatures[parent].age = 8;
    sim.config.energy.lifecycle.min_reproduce_age = 0;
    sim.config.energy.lifecycle.min_reproduce_energy = 0.0;
    let mut rng = SmallRng::seed_from_u64(17);
    assert_eq!(
        apply_reproduce(parent, &mut sim, Direction::N, 4.0, &mut rng),
        ReproductionActionResult::Spawned
    );
    assert_eq!(
        apply_reproduce(parent, &mut sim, Direction::N, 4.0, &mut rng),
        ReproductionActionResult::RejectedInvalidTarget
    );
    assert_eq!(sim.creatures[parent].offspring_spawned_count, 1);
    assert_eq!(
        sim.stats.reproductive_success_by_cognitive_class,
        ReproductiveSuccessByCognitiveClass::default()
    );

    let child = sim.creatures.keys().find(|id| *id != parent).unwrap();
    assert_eq!(sim.creatures[child].age, 0);
    assert_eq!(sim.creatures[child].generation, 1);
    // The actual newborn's genome can mutate; its exit is still one zero-age,
    // zero-offspring observation, while its successful parent remains alive.
    sim.remove_creature(child);
    let after_child = sim.stats.reproductive_success_by_cognitive_class;
    assert_eq!(
        CognitiveClass::ALL
            .into_iter()
            .map(|c| after_child.totals(c).creatures_observed_total)
            .sum::<u64>(),
        1
    );
    assert_eq!(
        CognitiveClass::ALL
            .into_iter()
            .map(|c| after_child.totals(c).offspring_spawned_sum)
            .sum::<u64>(),
        0
    );
    assert_eq!(
        CognitiveClass::ALL
            .into_iter()
            .map(|c| after_child.totals(c).survival_ticks_sum)
            .sum::<u64>(),
        0
    );

    sim.remove_creature(parent);
    let before = after_child.totals(CognitiveClass::None);
    assert_eq!(
        sim.stats
            .reproductive_success_by_cognitive_class
            .totals(CognitiveClass::None),
        ReproductiveSuccessTotals {
            creatures_observed_total: before.creatures_observed_total + 1,
            offspring_spawned_sum: 1,
            survival_ticks_sum: 8,
        }
    );
    assert_eq!(sim.stats.mortality.deaths_total, 2);
}

#[test]
fn reproductive_success_is_unchanged_by_observational_reexecution() {
    let (mut sim, id, _) = make_sim_two_creatures(10.0, 10.0);
    sim.remove_creature(id);
    let totals = sim.stats.reproductive_success_by_cognitive_class;
    crate::simulation::observe_final_actions(&sim);
    crate::simulation::observe_temporal_actions(&sim);
    assert_eq!(sim.stats.reproductive_success_by_cognitive_class, totals);
}
