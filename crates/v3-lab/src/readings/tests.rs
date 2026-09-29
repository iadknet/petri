use super::*;
use crate::arena::arena_config;
use crate::calibration::draw_scenes;
use crate::scene::SceneSpec;
use proptest::prelude::*;
use v3_core::creature::founder::founder_genome_with_age_gate;
use v3_core::creature::genome::cgp::{ComputeNode, ComputeNodeKind, GraphEdge};
use v3_core::creature::genome::VmBackendDef;

fn setup(lifetime: u32, start_energy: f32) -> Setup {
    Setup::new(arena_config(48), start_energy, lifetime)
}

fn founder(setup: &Setup) -> CreatureGenome {
    founder_genome_with_age_gate(
        setup.config.population.founder_profile,
        &setup.config.energy.lifecycle,
    )
}

/// What the scoring pass keeps for one genome on `scenes`.
struct Run {
    scores: Vec<SceneScore>,
    frozens: Vec<Frozen>,
    sequences: Vec<Vec<Boundary>>,
    scalar: f64,
}

fn observe(setup: &Setup, genome: &CreatureGenome, scenes: &[Scene]) -> Run {
    let mut run = Run {
        scores: Vec::new(),
        frozens: Vec::new(),
        sequences: Vec::new(),
        scalar: 0.0,
    };
    for scene in scenes {
        let (score, frozen, sequence) = evaluate_genome_observed(setup, genome, scene);
        run.scores.push(score);
        run.frozens.push(frozen);
        run.sequences.push(sequence);
    }
    let values: Vec<f64> = run.scores.iter().map(|s| s.score).collect();
    run.scalar = crate::stats::mean(&values).unwrap();
    run
}

fn observed<'a>(
    genome: &'a CreatureGenome,
    ancestry: &'a Ancestry,
    scenes: &'a [Scene],
    run: &'a Run,
) -> Observed<'a> {
    Observed {
        genome,
        ancestry,
        scenes,
        scores: &run.scores,
        frozens: &run.frozens,
        sequences: &run.sequences,
        scalar: run.scalar,
    }
}

fn scenes(count: u32) -> Vec<Scene> {
    draw_scenes(5, count, &SceneSpec::sparse(48, 0.06, 5)).unwrap()
}

fn shape_of(setup: &Setup, batteries: &Batteries, elite: &Observed<'_>) -> Shape {
    read(setup, batteries, elite, None).0.shape
}

fn labels(shape: &Shape) -> Vec<&str> {
    shape.families.iter().map(|f| f.family.as_str()).collect()
}

#[test]
fn the_founder_shape_reads_its_food_energy_and_age_inputs() {
    let setup = setup(100, 100.0);
    let genome = founder(&setup);
    let scenes = scenes(2);
    let run = observe(&setup, &genome, &scenes);
    let ancestry = Ancestry::default();
    let elite = observed(&genome, &ancestry, &scenes, &run);
    let shape = shape_of(&setup, &Batteries::new(&setup.config), &elite);

    // The founder's food inputs are the local ones; `AreaFoodSummary:0` is
    // the built-in comparator's (below). Its declared `NeighborOccupiedRing`
    // has no consumer.
    assert_eq!(
        labels(&shape),
        [
            "FoodHere:0",
            "NeighborFoodRing:0",
            "AgeTicks",
            "EnergyCurrent",
            "ActionQueue",
            "UpstreamSlot"
        ]
    );
    let comparator = crate::comparator::built_in(crate::scene::Assay::FoodSeeking, &genome);
    let comparator_run = observe(&setup, &comparator, &scenes);
    let comparator_shape = shape_of(
        &setup,
        &Batteries::new(&setup.config),
        &observed(&comparator, &ancestry, &scenes, &comparator_run),
    );
    assert!(labels(&comparator_shape).contains(&"AreaFoodSummary:0"));
    let executed = executed_union(&run.frozens);
    let reachable: BTreeSet<usize> = mesh_reachable_nodes(&genome).into_iter().collect();
    assert!(!executed.is_empty() && executed.is_subset(&reachable));
    assert_eq!(shape.executed, executed.len());
    assert_eq!(shape.reachable, reachable.len());
    assert_eq!(
        (shape.genome_size, shape.nodes, shape.deaths),
        (genome.genome_size(), genome.nodes.len(), 0)
    );
    assert_eq!(shape.functional_complexity, functional_complexity(&genome));
    assert_eq!(shape.ancestry, Ancestry::default());
    assert!(shape.families.iter().all(|f| f.structural));
    let food = shape
        .families
        .iter()
        .find(|f| f.family == "NeighborFoodRing:0")
        .unwrap();
    assert_eq!((food.executed_node, food.live), (true, Some(true)));
    let energy = shape
        .families
        .iter()
        .find(|f| f.family == "EnergyCurrent")
        .unwrap();
    assert_eq!(energy.live, None, "the census does not cover introspection");
    assert_eq!(shape.steering.scenarios, 48);
    // Catalog order.
    let mut sorted = shape.families.clone();
    let order = |label: &str| {
        consumers(&genome)
            .keys()
            .position(|family| family.label() == label)
            .unwrap()
    };
    sorted.sort_by_key(|f| order(&f.family));
    assert_eq!(sorted, shape.families);
}

#[test]
fn a_first_tick_death_leaves_an_empty_executed_union_and_counts_the_death() {
    let setup = setup(50, 0.01);
    let genome = founder(&setup);
    let scenes = scenes(2);
    let run = observe(&setup, &genome, &scenes);
    assert!(run.sequences.iter().all(Vec::is_empty));
    let ancestry = Ancestry::default();
    let shape = shape_of(
        &setup,
        &Batteries::new(&setup.config),
        &observed(&genome, &ancestry, &scenes, &run),
    );
    assert_eq!((shape.executed, shape.deaths), (0, 2));
    assert!(shape.families.iter().all(|f| !f.executed_node));
}

#[test]
fn a_child_adds_its_birth_to_its_parents_ancestry() {
    let birth = |requested, applied, operator: &str| Events {
        requested,
        applied,
        applied_by_operator: BTreeMap::from([(operator.to_owned(), applied)]),
        offspring: 1,
        ..Events::default()
    };
    let parent = Ancestry::default().child(&birth(3, 2, "a"));
    let child = parent.child(&birth(4, 1, "b"));
    assert_eq!(
        child,
        Ancestry {
            births: 2,
            requested: 7,
            applied: 3,
            applied_by_operator: BTreeMap::from([("a".to_owned(), 2), ("b".to_owned(), 1)]),
        }
    );
}

/// The founder plus a dead compute node reading a fresh
/// `NearbyCreatureIdentity` reference on the entry node: structural but
/// never read.
fn with_unread_family(genome: &CreatureGenome) -> CreatureGenome {
    let mut genome = genome.clone();
    let entry = genome
        .nodes
        .iter()
        .position(|node| node.node_id == genome.entry_node_id)
        .unwrap();
    let node = &mut genome.nodes[entry];
    let ref_idx = u16::try_from(node.input_refs.len()).unwrap();
    node.input_refs.push(InputReference::World(
        v3_core::contracts::WorldInputKey::NearbyCreatureIdentity,
    ));
    let BackendDef::Graph(graph) = &mut node.backend_def else {
        panic!("the founder entry is a graph node");
    };
    graph.compute_nodes.push(ComputeNode {
        kind: ComputeNodeKind::Add,
        inputs: vec![GraphEdge {
            source: GraphSource::InputLeaf {
                ref_idx,
                sub_idx: 0,
            },
            weight: 1.0,
        }],
        plasticity: None,
    });
    genome
}

#[test]
fn ablation_separates_read_from_unread_families() {
    let setup = setup(100, 100.0);
    let genome = with_unread_family(&founder(&setup));
    let scenes = scenes(2);
    let run = observe(&setup, &genome, &scenes);
    assert!(run.scores.iter().any(|s| s.food_eaten > 0), "it eats");
    let ancestry = Ancestry::default();
    let elite = observed(&genome, &ancestry, &scenes, &run);
    let families = Wiring::of(&genome).families;
    let (Some(causal), ticks) = causal(&setup, &elite, &families) else {
        panic!("supported");
    };
    assert!(ticks > 0);
    let find = |label: &str| causal.iter().find(|c| c.family == label).unwrap();
    let unread = find("NearbyCreatureIdentity");
    assert_eq!((unread.causal, unread.score_delta), (false, 0.0));
    assert!(find("NeighborFoodRing:0").causal, "the food ring steers");
    for reading in &causal {
        assert!(reading.score_delta == 0.0 || reading.causal, "{reading:?}");
    }
    assert!(causal.iter().all(|c| !c.family.starts_with("SharedMemory")));
}

#[test]
fn ablation_keeps_size_and_leaves_no_in_range_consumer_of_the_family() {
    let setup = setup(1, 100.0);
    let genome = with_unread_family(&founder(&setup));
    for family in consumers(&genome).into_keys() {
        let ablated = ablate(&genome, family).unwrap();
        assert_eq!(ablated.genome_size(), genome.genome_size());
        // One reference per family on each founder node: the complexity
        // charge is unchanged too.
        assert_eq!(ablated.complexity(), genome.complexity(), "{family:?}");
        let left = consumers(&ablated);
        assert_eq!(
            left.contains_key(&family),
            family.is_shared_memory(),
            "{family:?}"
        );
    }
}

/// `genome` with one more node of `refs` references and the given backend.
fn with_wide_node(genome: &CreatureGenome, refs: usize, vm: bool) -> CreatureGenome {
    let mut genome = genome.clone();
    let mut node = genome.nodes[0].clone();
    node.node_id = NodeId::new(u32::MAX);
    node.targets.clear();
    node.input_refs = vec![InputReference::ActionQueue; refs];
    if vm {
        node.backend_def = BackendDef::Vm(VmBackendDef {
            register_count: 1,
            constants: Vec::new(),
            program: vec![VmInstruction::ReadInput {
                dst: 0,
                ref_idx: 0,
                sub_idx: 0,
            }],
        });
    }
    genome.nodes.push(node);
    genome
}

#[test]
fn a_node_with_65535_references_is_refused_at_load_and_unsupported_in_a_run() {
    let mut setup = setup(20, 100.0);
    setup.config.mutation.per_unit_rate = 0.0;
    let founder = founder(&setup);
    let scenes = scenes(1);
    let run = observe(&setup, &founder, &scenes);
    let ancestry = Ancestry::default();
    let dir = std::env::temp_dir().join(format!("petri-lab-readings-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for vm in [false, true] {
        let wide = with_wide_node(&founder, usize::from(ABLATED_REF), vm);
        assert!(!ablation_supported(&wide));
        assert!(ablate(&wide, Family::ActionQueue).is_none());
        let elite = observed(&wide, &ancestry, &scenes, &run);
        assert_eq!(causal(&setup, &elite, &[Family::ActionQueue]).0, None);
        // The row's signature: causal null and labelled, mutants still read
        // (identical at rate 0, so nothing is evaluated).
        let (readings, _) = read(
            &setup,
            &Batteries::new(&setup.config),
            &elite,
            Some((2, seed())),
        );
        let signature = readings.signature.unwrap();
        assert_eq!(signature.causal, None);
        assert_eq!(
            signature.unsupported.as_deref(),
            Some(UNSUPPORTED_INPUT_REFS)
        );
        assert_eq!(signature.mutants.n, 2);
        let path = dir.join(format!("wide-{vm}.json"));
        std::fs::write(
            &path,
            serde_json::to_vec(&crate::GenomeFile::new(wide)).unwrap(),
        )
        .unwrap();
        assert!(crate::GenomeFile::load(&path).is_err(), "vm {vm}");

        let narrow = with_wide_node(&founder, usize::from(ABLATED_REF) - 1, vm);
        assert!(ablation_supported(&narrow));
        std::fs::write(
            &path,
            serde_json::to_vec(&crate::GenomeFile::new(narrow)).unwrap(),
        )
        .unwrap();
        assert!(crate::GenomeFile::load(&path).is_ok(), "vm {vm}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

fn seed() -> MutantSeed<'static> {
    MutantSeed {
        observation: 11,
        arm: "native",
        generation: 3,
    }
}

#[test]
fn mutation_off_mutants_are_identical_silent_and_scored_by_copy() {
    let mut setup = setup(60, 100.0);
    setup.config.mutation.per_unit_rate = 0.0;
    let genome = founder(&setup);
    let scenes = scenes(2);
    let run = observe(&setup, &genome, &scenes);
    let ancestry = Ancestry::default();
    let elite = observed(&genome, &ancestry, &scenes, &run);
    let (mutants, ticks) = mutants(
        &setup,
        &Batteries::new(&setup.config),
        &elite,
        &mesh_reachable_nodes(&genome),
        5,
        &seed(),
    );
    assert_eq!(ticks, 0, "nothing evaluated");
    assert_eq!(
        (mutants.n, mutants.identical, mutants.silent, mutants.equal),
        (5, 5, 5, 5)
    );
    assert!(mutants.scores.iter().all(|s| *s == run.scalar));
    assert_eq!(mutants.mean_delta, 0.0);
}

#[test]
fn mutants_follow_their_seed_and_classes_sum_to_n() {
    let mut setup = setup(60, 100.0);
    setup.config.mutation.per_unit_rate = 0.05;
    let genome = founder(&setup);
    let scenes = scenes(2);
    let run = observe(&setup, &genome, &scenes);
    let ancestry = Ancestry::default();
    let elite = observed(&genome, &ancestry, &scenes, &run);
    let batteries = Batteries::new(&setup.config);
    let reachable = mesh_reachable_nodes(&genome);
    let (a, ticks) = mutants(&setup, &batteries, &elite, &reachable, 6, &seed());
    assert!(ticks > 0 && a.identical < 6);
    assert_eq!(a.silent + a.changed + a.dead, 6);
    assert_eq!(a.improved + a.equal + a.worse, 6);
    assert!(a.scores.windows(2).all(|w| w[0] <= w[1]));
    assert_eq!(
        a,
        mutants(&setup, &batteries, &elite, &reachable, 6, &seed()).0
    );
    let other = MutantSeed {
        generation: 4,
        ..seed()
    };
    assert_ne!(
        a,
        mutants(&setup, &batteries, &elite, &reachable, 6, &other).0
    );
}

#[test]
fn projections_keep_the_first_shape_scalars_and_the_last_aggregates() {
    let setup = setup(40, 100.0);
    let genome = founder(&setup);
    let scenes = scenes(1);
    let run = observe(&setup, &genome, &scenes);
    let ancestry = Ancestry::default();
    let elite = observed(&genome, &ancestry, &scenes, &run);
    let batteries = Batteries::new(&setup.config);
    let (plain, _) = read(&setup, &batteries, &elite, None);
    let (signed, _) = read(&setup, &batteries, &elite, Some((3, seed())));
    let mut projections = ReplicateReadings::default();
    projections.record(&plain);
    projections.record(&signed);
    assert_eq!(projections.first, Some(FirstProjection::of(&plain.shape)));
    let last = projections.last.unwrap();
    assert_eq!(last.shape, signed.shape);
    let aggregates = last.signature.unwrap();
    let mutants = &signed.signature.as_ref().unwrap().mutants;
    assert_eq!(aggregates.n, 3);
    assert_eq!(aggregates.score_min, mutants.scores[0]);
    assert_eq!(aggregates.score_max, mutants.scores[2]);
    assert_eq!(aggregates.score_median, mutants.scores[1]);
    assert!(aggregates.unsupported.is_none());
}

#[test]
fn two_references_of_one_family_keep_distinct_sentinels() {
    let setup = setup(1, 100.0);
    let mut genome = founder(&setup);
    let node = &mut genome.nodes[0];
    let duplicate = u16::try_from(node.input_refs.len()).unwrap();
    node.input_refs.push(node.input_refs[0].clone());
    let BackendDef::Graph(graph) = &mut node.backend_def else {
        panic!("the founder's node 0 is a graph node");
    };
    // Wire the duplicate onto a surface that already reads reference 0.
    let sink = graph
        .output_sinks
        .iter_mut()
        .find(|sink| {
            sink.inputs
                .iter()
                .any(|edge| matches!(edge.source, GraphSource::InputLeaf { ref_idx: 0, .. }))
        })
        .unwrap();
    sink.inputs.push(GraphEdge {
        source: GraphSource::InputLeaf {
            ref_idx: duplicate,
            sub_idx: 0,
        },
        weight: 0.0,
    });
    let family = Family::of(&genome.nodes[0].input_refs[0]);
    let ablated = ablate(&genome, family).unwrap();
    // One shared sentinel would count one consumed index where there were two.
    assert_eq!(ablated.complexity(), genome.complexity());
    assert!(!consumers(&ablated).contains_key(&family));
}

#[test]
fn a_node_without_sentinel_headroom_is_unsupported() {
    let setup = setup(1, 100.0);
    let founder = founder(&setup);
    // Five distinct `ActionQueue` indices on 65,534 references: two
    // sentinels are free.
    let tight = with_wide_node(&founder, usize::from(ABLATED_REF) - 1, false);
    assert!(ablation_supported(&tight));
    assert!(ablate(&tight, Family::ActionQueue).is_none());
    assert!(ablate(&tight, Family::AgeTicks).is_some());
}

fn class() -> impl Strategy<Value = Class> {
    prop_oneof![Just(Class::Silent), Just(Class::Changed), Just(Class::Dead)]
}

proptest! {
    #[test]
    fn folded_mutants_partition_n_and_sort_their_scores(
        results in prop::collection::vec((class(), -5.0f64..5.0, any::<bool>()), 0..40),
        elite in -5.0f64..5.0,
    ) {
        let folded = fold_mutants(elite, &results);
        let n = u32::try_from(results.len()).unwrap();
        prop_assert_eq!(folded.n, n);
        prop_assert_eq!(folded.silent + folded.changed + folded.dead, n);
        prop_assert_eq!(folded.improved + folded.equal + folded.worse, n);
        prop_assert!(folded.scores.windows(2).all(|w| w[0] <= w[1]));
        let mut expected: Vec<f64> = results.iter().map(|r| r.1).collect();
        expected.sort_by(f64::total_cmp);
        prop_assert_eq!(&folded.scores, &expected);
    }

    #[test]
    fn ancestry_accumulates_every_birth_along_the_path(
        births in prop::collection::vec((0u64..50, 0u64..50, 0usize..3), 0..20),
    ) {
        let mut ancestry = Ancestry::default();
        for &(requested, applied, operator) in &births {
            ancestry = ancestry.child(&Events {
                requested,
                applied,
                applied_by_operator: BTreeMap::from([(operator.to_string(), applied)]),
                ..Events::default()
            });
        }
        prop_assert_eq!(ancestry.births, births.len() as u64);
        prop_assert_eq!(ancestry.requested, births.iter().map(|b| b.0).sum::<u64>());
        prop_assert_eq!(ancestry.applied, births.iter().map(|b| b.1).sum::<u64>());
        prop_assert_eq!(ancestry.applied_by_operator.values().sum::<u64>(), ancestry.applied);
    }
}
