use super::assay::{draw_incumbents, run_replicate, seed_arms};
use super::controllers::controller;
use super::discovery::{self, discovery, feasibility, vm_concern, FOUNDER_VM_TRANSLATION_STEPS};
use super::fixtures::competence;
use super::*;
use crate::config::{FounderProfile, SimulationConfig};
use crate::creature::founder::{founder_genome, vm_decision_founder_genome};
use crate::neighborhood::Battery;

fn founder() -> crate::creature::genome::CreatureGenome {
    founder_genome(FounderProfile::V3Alpha1)
}

#[test]
fn every_controller_is_adequate_and_competent() {
    let config = SimulationConfig::default();
    let battery = Battery::generate(2);
    for family in Family::ALL {
        let reading = competence(
            &founder(),
            family,
            &battery,
            &config.runtime,
            config.shared_memory.decay_rate,
        );
        assert!(reading.adequacy.passed, "{family:?}: {reading:?}");
        assert_eq!(reading.violations, 0, "{family:?}: {reading:?}");
        assert_eq!(reading.battery_violations, 0, "{family:?}: {reading:?}");
        assert!(reading.battery_zero_signal > 0, "{family:?}");
        assert!(reading.signal_changed > 0, "{family:?}: {reading:?}");
    }
}

/// Diagonal ring cells do not gate `A_vector`: with the cardinal ring empty
/// and the nearest primary food on a diagonal ring cell, the gate opens and
/// the authored controller leads with a move toward one of its two
/// components in every diagonal, which its zero-weight twin does not.
#[test]
fn diagonal_ring_food_does_not_close_the_vector_gate() {
    use super::fixtures::{run, Context, Local};
    use crate::contracts::{Direction, WorldAction};
    let config = SimulationConfig::default();
    let (authored, _) = controller(&founder(), Family::Vector, false);
    let (inert, _) = controller(&founder(), Family::Vector, true);
    let mut changed = 0;
    for diagonal in [1u8, 3, 5, 7] {
        let scenario = Context {
            local: Local::None,
            ring_food: Some(diagonal),
            eligible: false,
            barrier: None,
            diagonal_barriers: false,
            far_food: None,
        }
        .scenario();
        let [a, z] =
            [&authored, &inert].map(|genome| run(genome, &scenario, &config.runtime).actions);
        let toward = [diagonal - 1, (diagonal + 1) % 8]
            .map(|cardinal| WorldAction::Move(Direction::ALL[usize::from(cardinal)]));
        assert!(
            a.first().is_some_and(|first| toward.contains(first)),
            "diagonal {diagonal}: {a:?}"
        );
        changed += u32::from(a != z);
    }
    assert!(changed > 0);
}

/// The adequacy fixture's premise in production visibility: barriers on the
/// four diagonal ring cells never hide a cardinal ring cell.
#[test]
fn diagonal_barriers_leave_every_cardinal_ring_cell_visible() {
    use crate::contracts::{Direction, Position};
    use crate::sensors::visibility::{compute_visible_cells, get_visibility_table};
    let mut world = crate::kernel::WorldState::new(20, 20, crate::config::WorldEdgeMode::Wrap);
    let origin = Position::new(10, 10);
    for diagonal in [Direction::NE, Direction::SE, Direction::SW, Direction::NW] {
        let (dx, dy) = diagonal.delta();
        let cell = |offset: i32| u16::try_from(10 + offset).expect("inside the world");
        world.set_barrier(Position::new(cell(dx), cell(dy)), true);
    }
    let visible = compute_visible_cells(origin, &world, get_visibility_table(5));
    for cardinal in [Direction::N, Direction::E, Direction::S, Direction::W] {
        let (dx, dy) = cardinal.delta();
        assert!(
            visible.iter().any(|cell| (cell.dx, cell.dy) == (dx, dy)),
            "{cardinal:?}"
        );
    }
}

#[test]
fn authored_and_inert_pairs_match_in_size_and_differ_only_in_weights() {
    for family in Family::ALL {
        let (authored, structure) = controller(&founder(), family, false);
        let (inert, same) = controller(&founder(), family, true);
        assert_eq!(structure, same);
        assert_eq!(authored.genome_size(), inert.genome_size(), "{family:?}");
        assert!(authored.genome_size() > founder().genome_size());
        assert_ne!(authored, inert);
        assert_eq!(authored.nodes[0], founder().nodes[0]);
    }
}

#[test]
fn applicability_names_the_two_null_references() {
    assert!(!applicable(Family::Ring, ORCHARDS));
    assert!(!applicable(Family::Scalar, CANYON));
    assert!(applicable(Family::Vector, ORCHARDS) && applicable(Family::Vector, CANYON));
    assert!(Family::ALL.into_iter().all(|family| applicable(family, 2)));
    assert_eq!(run_seed(2, 7), 26_002_007);
}

fn small_world() -> SimulationConfig {
    let mut config = SimulationConfig::default();
    config.world.width = 24;
    config.world.height = 24;
    config.population.initial_creatures = 40;
    replicate_config(&config)
}

#[test]
fn arms_follow_creature_order_and_descendants_inherit_them() {
    let config = small_world();
    let genomes = ArmGenomes::new(
        founder(),
        vec![controller(&founder(), Family::Ring, true).0],
    );
    let sim = seed_arms(&config, run_seed(0, 0), &genomes);
    for (k, (_, creature)) in sim.creatures.iter().enumerate() {
        let arm = Arm::ALL[k % 8];
        assert_eq!(Arm::of_lineage(creature.identity.lineage_id), arm);
        let expected = match arm {
            Arm::Founder => &genomes.founder,
            Arm::Incumbent => &genomes.incumbents[0],
            Arm::Authored(family) => &genomes.authored[family.index() as usize],
            Arm::Inert(family) => &genomes.inert[family.index() as usize],
        };
        assert_eq!(&creature.genome, expected);
        assert_eq!(creature.cached_genome_size, expected.genome_size());
    }

    let run = run_replicate(&config, run_seed(0, 0), &genomes, 150);
    assert_eq!(run.arms.iter().map(|arm| arm.founders).sum::<u32>(), 40);
    assert_eq!(run.arms[0].founders, 5);
    let births: u64 = run.arms.iter().map(|arm| arm.births).sum();
    assert!(births > 0, "the small world reproduces within 150 ticks");
    assert_eq!(births + run.unattributed_births, run.births_total);
    for arm in &run.arms {
        assert_eq!(arm.living.len(), (150 / assay::SAMPLE_EVERY) as usize);
    }
    // Mutation is off and arms pass by lineage: replaying the run and
    // counting each arm's creatures beyond its founders matches its births
    // whenever no newborn died in its birth tick.
    if run.unattributed_births == 0 {
        let mut replay = seed_arms(&config, run_seed(0, 0), &genomes);
        let mut seen: std::collections::BTreeSet<_> = replay.creatures.keys().collect();
        let mut per_arm = [0u64; 8];
        for _ in 0..150 {
            crate::simulation::run_tick(&mut replay, &mut None);
            for (id, creature) in &replay.creatures {
                if seen.insert(id) {
                    per_arm[Arm::of_lineage(creature.identity.lineage_id).index()] += 1;
                    assert_eq!(
                        creature.genome,
                        *match Arm::of_lineage(creature.identity.lineage_id) {
                            Arm::Founder => &genomes.founder,
                            Arm::Incumbent => &genomes.incumbents[0],
                            Arm::Authored(family) => &genomes.authored[family.index() as usize],
                            Arm::Inert(family) => &genomes.inert[family.index() as usize],
                        },
                        "a newborn carries its arm's genome"
                    );
                }
            }
        }
        let counted: Vec<u64> = run.arms.iter().map(|arm| arm.births).collect();
        assert_eq!(counted, per_arm);
    }
    for (index, arm) in Arm::ALL.into_iter().enumerate() {
        let exposure = run.arms[index].exposure;
        assert_eq!(
            exposure.sampled > 0,
            arm.family().is_some() && run.arms[index].living[0] > 0
        );
        assert!(exposure.applied <= exposure.sampled && exposure.exposed <= exposure.sampled);
    }
    // Inert arms never change actions when ablated.
    for family in Family::ALL {
        assert_eq!(run.arms[Arm::Inert(family).index()].exposure.applied, 0);
    }
}

#[test]
fn reduced_replicates_are_byte_identical_across_runs_and_thread_counts() {
    let config = small_world();
    let genomes = ArmGenomes::new(founder(), Vec::new());
    let read =
        || serde_json::to_vec(&run_replicate(&config, run_seed(1, 3), &genomes, 120)).unwrap();
    let pool = |threads| {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
    };
    let one = pool(1).install(read);
    assert_eq!(one, pool(4).install(read));
    assert_eq!(one, read());
}

#[test]
fn incumbents_are_a_seeded_draw_of_living_genomes() {
    let config = small_world();
    let sim = crate::simulation::seed_simulation(config.clone(), 11);
    let drawn = draw_incumbents(&sim, 11);
    assert_eq!(drawn.len(), assay::INCUMBENTS);
    assert_eq!(drawn, draw_incumbents(&sim, 11));
    let mut empty = crate::simulation::seed_simulation(config, 11);
    let ids: Vec<_> = empty.creatures.keys().collect();
    for id in ids {
        empty.remove_creature(id);
    }
    assert!(draw_incumbents(&empty, 11).is_empty());
    assert!(ArmGenomes::new(founder(), Vec::new()).founder_fallback());
}

#[test]
fn discovery_steps_count_declarations_and_connections() {
    let config = SimulationConfig::default();
    for family in Family::ALL {
        let reading = discovery(&founder(), family, &config, 400);
        assert_eq!(reading.declare.proposals, 400);
        assert!(reading.declare.hits <= 400 && reading.connect.hits <= reading.connect.proposals);
        assert!(reading.connect.hits <= reading.connect.beside);
        assert!((reading.events_per_birth - 0.485).abs() < 1e-9);
        assert_eq!(
            reading.implied_births.is_some(),
            reading.declare.hits > 0 && reading.connect.hits > 0
        );
        assert_eq!(reading, discovery(&founder(), family, &config, 400));
    }
}

#[test]
fn feasibility_and_vm_concerns_are_recorded() {
    let config = SimulationConfig::default();
    for family in Family::ALL {
        let record = feasibility(&founder(), family, &config);
        assert!(record.feasible, "{family:?}");
        assert!(record.added_genome_units > 0);
        assert_eq!(record.extended_perception, family == Family::Vector);
        assert_eq!(
            record.founder_work.executions,
            record.authored_work.executions
        );
        let vm = vm_concern(family, &config, FOUNDER_VM_TRANSLATION_STEPS);
        assert!(vm.instructions > 0);
        assert!(vm.charge_after_founder >= vm.charge_standalone);
    }
    let crate::creature::genome::BackendDef::Vm(vm) =
        &vm_decision_founder_genome().nodes[1].backend_def
    else {
        panic!("the VM founder's vote node is a VM node");
    };
    assert_eq!(vm.program.len(), FOUNDER_VM_TRANSLATION_STEPS as usize);
}

/// Prints the readings-file discovery, feasibility and VM tables. Run with
/// `cargo test -p v3-core --release --lib opportunity_readings_print -- --ignored --nocapture`.
#[test]
#[ignore = "prints the readings-file tables"]
fn opportunity_readings_print() {
    let config = SimulationConfig::default();
    let battery = Battery::generate(2);
    for family in Family::ALL {
        let d = discovery(&founder(), family, &config, discovery::DISCOVERY_PROPOSALS);
        let f = feasibility(&founder(), family, &config);
        let v = vm_concern(family, &config, FOUNDER_VM_TRANSLATION_STEPS);
        let c = competence(
            &founder(),
            family,
            &battery,
            &config.runtime,
            config.shared_memory.decay_rate,
        );
        println!(
            "{family:?}\n  discovery {d:?}\n  feasibility {f:?}\n  vm {v:?}\n  competence {c:?}"
        );
    }
}
