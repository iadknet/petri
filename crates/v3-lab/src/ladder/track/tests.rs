use super::*;
use crate::arena::arena_config;
use crate::ladder::{Keyed, Status};
use crate::readings::Batteries;
use proptest::prelude::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use v3_core::contracts::{InputReference, Position, WorldInputKey};
use v3_core::creature::founder::founder_genome_with_age_gate;
use v3_core::creature::genome::cgp::{CgpGraphBackendDef, GraphEdge, GraphSource};
use v3_core::creature::genome::VmBackendDef;
use v3_core::mutation::reachability::ParentExecuted;
use v3_core::mutation::{
    MutationDomain, MutationEngine, MutationEventRecord, MutationOperator, MutationSkipReason,
    TargetReachability,
};

fn founder() -> CreatureGenome {
    let config = arena_config(48);
    founder_genome_with_age_gate(config.population.founder_profile, &config.energy.lifecycle)
}

fn node(id: u32, input_refs: Vec<InputReference>, backend_def: BackendDef) -> NodeGenome {
    NodeGenome {
        node_id: NodeId(id),
        input_refs,
        backend_def,
        targets: Vec::new(),
    }
}

fn vm(program: Vec<VmInstruction>) -> BackendDef {
    BackendDef::Vm(VmBackendDef {
        register_count: 1,
        constants: Vec::new(),
        program,
    })
}

fn one_node(node: NodeGenome) -> CreatureGenome {
    CreatureGenome {
        entry_node_id: node.node_id,
        nodes: vec![node],
    }
}

/// A graph node holding the full fixed sink catalog, every sink unconnected
/// unless `wired` names its kind; one non-relevant input reference.
fn graph_node(id: u32, wired: Option<OutputSinkKind>) -> NodeGenome {
    let mut graph = CgpGraphBackendDef::new_with_fixed_outputs();
    for sink in &mut graph.output_sinks {
        if Some(sink.kind) == wired {
            sink.inputs.push(GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                weight: 1.0,
            });
        }
    }
    node(
        id,
        vec![InputReference::World(WorldInputKey::NeighborOccupiedRing)],
        BackendDef::Graph(graph),
    )
}

#[test]
fn the_founder_has_a_sensor_site_and_a_motor_site() {
    let founder = founder();
    let sites = Sites::of(Assay::FoodSeeking, &founder);
    let ids: BTreeSet<NodeId> = founder.nodes.iter().map(|n| n.node_id).collect();
    assert_eq!(sites.sites, ids);
    assert_eq!(sites.reachable, 2);
    assert!(motor_node(&founder.nodes[1]), "the decision node votes");
}

#[test]
fn unconnected_vote_sinks_are_not_motor_sites_and_a_bare_start_has_no_site() {
    let bare = one_node(graph_node(0, None));
    assert!(!motor_node(&bare.nodes[0]));
    let sites = Sites::of(Assay::FoodSeeking, &bare);
    assert_eq!((sites.sites.len(), sites.reachable), (0, 1));
    for (kind, motor) in [
        (OutputSinkKind::ActionVote(VoteSink::Eat), true),
        (OutputSinkKind::ActionVote(VoteSink::Move(3)), true),
        (OutputSinkKind::ActionVote(VoteSink::Reproduce(0)), false),
        (OutputSinkKind::ActionVote(VoteSink::Decide), false),
        (OutputSinkKind::CustomOutput(0), false),
    ] {
        let genome = one_node(graph_node(0, Some(kind)));
        assert_eq!(motor_node(&genome.nodes[0]), motor, "{kind:?}");
        assert_eq!(
            Sites::of(Assay::FoodSeeking, &genome).sites.len(),
            usize::from(motor)
        );
    }
}

#[test]
fn a_vm_add_vote_on_eat_or_move_is_a_motor_site() {
    for (sink, motor) in [
        (0u8, true),
        (1, true),
        (8, true),
        (9, false),
        (26, false),
        (200, false),
    ] {
        let genome = one_node(node(
            4,
            Vec::new(),
            vm(vec![VmInstruction::AddVote { sink, src: 0 }]),
        ));
        assert_eq!(motor_node(&genome.nodes[0]), motor, "sink {sink}");
    }
}

#[test]
fn relevant_families_follow_the_assay() {
    for family in [
        Family::FoodHere(0),
        Family::NeighborFoodRing(2),
        Family::AreaFoodSummary(1),
    ] {
        assert!(relevant_family(Assay::FoodSeeking, family));
        assert!(relevant_family(Assay::BarrierNavigation, family));
    }
    for family in [Family::NeighborBarrierRing, Family::AreaBarrierSummary] {
        assert!(!relevant_family(Assay::FoodSeeking, family));
        assert!(relevant_family(Assay::BarrierNavigation, family));
    }
    for family in [
        Family::NeighborOccupiedRing,
        Family::EnergyCurrent,
        Family::UpstreamSlot,
    ] {
        assert!(!relevant_family(Assay::BarrierNavigation, family));
    }
    assert_eq!(relevant_family_labels(Assay::FoodSeeking).len(), 3);
    assert_eq!(relevant_family_labels(Assay::BarrierNavigation).len(), 5);
    // A barrier reader is a site only under barrier navigation.
    let reader = one_node(node(
        3,
        vec![InputReference::World(WorldInputKey::NeighborBarrierRing)],
        vm(vec![VmInstruction::ReadInput {
            dst: 0,
            ref_idx: 0,
            sub_idx: 0,
        }]),
    ));
    assert!(Sites::of(Assay::FoodSeeking, &reader).sites.is_empty());
    assert_eq!(Sites::of(Assay::BarrierNavigation, &reader).sites.len(), 1);
}

#[test]
fn an_unreachable_relevant_node_is_not_a_site() {
    let mut genome = founder();
    let mut stray = genome.nodes[0].clone();
    stray.node_id = NodeId(77);
    stray.targets.clear();
    genome.nodes.push(stray);
    let sites = Sites::of(Assay::FoodSeeking, &genome);
    assert!(!sites.sites.contains(&NodeId(77)));
    assert_eq!(sites.reachable, 2);
}

fn event(
    operator: Option<MutationOperator>,
    target: Option<u32>,
    outcome: MutationEventOutcome,
    discarded: Vec<(MutationOperator, Option<NodeId>)>,
) -> MutationEventRecord {
    MutationEventRecord {
        domain: MutationDomain::Graph,
        operator,
        target: target.map(NodeId),
        outcome,
        discarded,
    }
}

fn summary(events: Vec<MutationEventRecord>) -> MutationSummary {
    let mut summary = MutationSummary::zero();
    summary.applied_events =
        u32::try_from(events.iter().filter(|e| e.outcome.is_applied()).count()).unwrap();
    summary.events = events;
    summary
}

#[test]
fn a_birth_counts_events_on_the_parents_sites_by_operator_and_reason() {
    let parent = founder();
    let sites = Sites::of(Assay::FoodSeeking, &parent);
    let (n0, n1) = (parent.nodes[0].node_id.0, parent.nodes[1].node_id.0);
    let applied = MutationEventOutcome::Applied(TargetReachability::Reachable);
    let events = vec![
        event(
            Some(MutationOperator::GraphAddGraphEdge),
            Some(n0),
            applied,
            Vec::new(),
        ),
        event(
            None,
            Some(n1),
            MutationEventOutcome::Skipped(MutationSkipReason::NoApplicableTarget),
            vec![
                (MutationOperator::GraphRemoveGraphEdge, Some(NodeId(n0))),
                (MutationOperator::GraphSwapGraphOperator, None),
                (MutationOperator::GraphRetargetGraphEdge, Some(NodeId(99))),
            ],
        ),
        // Off the sites, or with no target: never targeted.
        event(
            Some(MutationOperator::GraphAddGraphEdge),
            Some(99),
            applied,
            Vec::new(),
        ),
        event(
            Some(MutationOperator::InputRefAdd),
            None,
            MutationEventOutcome::Skipped(MutationSkipReason::ParseabilityViolation),
            Vec::new(),
        ),
    ];
    let birth = Birth::of(
        Assay::FoodSeeking,
        &parent,
        &sites,
        &parent,
        &summary(events),
    );
    let map = |pairs: &[(&str, u64)]| -> Keyed {
        pairs.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect()
    };
    assert_eq!(
        birth.targeted,
        Targeted {
            requested: map(&[("GraphAddGraphEdge", 1), ("none", 1)]),
            applied: map(&[("GraphAddGraphEdge", 1)]),
            skipped: map(&[("none", 1)]),
            skipped_by_reason: map(&[("NoApplicableTarget", 1)]),
        }
    );
    assert_eq!(birth.discarded, 1);
    assert_eq!((birth.created, birth.removed), (0, 0));
    assert_eq!((birth.sites, birth.reachable, birth.applied), (2, 2, 2));
    assert!(birth.touching);

    // Skips and discards alone never touch.
    let quiet = Birth::of(
        Assay::FoodSeeking,
        &parent,
        &sites,
        &parent,
        &summary(vec![event(
            Some(MutationOperator::GraphAddGraphEdge),
            Some(n0),
            MutationEventOutcome::Skipped(MutationSkipReason::NumericProposalRejected),
            vec![(MutationOperator::GraphRemoveGraphEdge, Some(NodeId(n1)))],
        )]),
    );
    assert!(!quiet.touching);
    assert_eq!(quiet.discarded, 1);
}

#[test]
fn a_zero_site_start_is_touched_by_a_site_the_child_creates_or_the_parent_loses() {
    let bare = one_node(graph_node(0, None));
    let wired = one_node(graph_node(
        0,
        Some(OutputSinkKind::ActionVote(VoteSink::Eat)),
    ));
    let none = summary(Vec::new());
    let created = Birth::of(
        Assay::FoodSeeking,
        &bare,
        &Sites::of(Assay::FoodSeeking, &bare),
        &wired,
        &none,
    );
    assert_eq!(
        (created.created, created.removed, created.touching),
        (1, 0, true)
    );
    assert_eq!(created.sites, 0);
    let removed = Birth::of(
        Assay::FoodSeeking,
        &wired,
        &Sites::of(Assay::FoodSeeking, &wired),
        &bare,
        &none,
    );
    assert_eq!(
        (removed.created, removed.removed, removed.touching),
        (0, 1, true)
    );
    let unchanged = Birth::of(
        Assay::FoodSeeking,
        &bare,
        &Sites::of(Assay::FoodSeeking, &bare),
        &bare,
        &none,
    );
    assert!(!unchanged.touching);
}

fn junk(id: u32, tag: u8) -> NodeGenome {
    node(
        id,
        Vec::new(),
        vm(vec![VmInstruction::AddVote { sink: 26, src: tag }]),
    )
}

fn with(genome: &CreatureGenome, node: NodeGenome) -> CreatureGenome {
    let mut genome = genome.clone();
    genome.nodes.push(node);
    genome
}

fn without(genome: &CreatureGenome, id: u32) -> CreatureGenome {
    let mut genome = genome.clone();
    genome.nodes.retain(|n| n.node_id != NodeId(id));
    genome
}

#[test]
fn a_removed_highest_id_reused_by_an_unrelated_node_is_not_the_removed_node() {
    let parent = with(&founder(), junk(9, 1));
    let child = without(&parent, 9);
    let change = Change::between(&parent, &child);
    assert_eq!((change.added.len(), change.removed.len()), (0, 1));
    assert!(change.carried_by(&child) && !change.carried_by(&parent));
    // `next_node_id` reuses 9 for an unrelated node: still carried.
    assert!(change.carried_by(&with(&child, junk(9, 2))));
    // The same content back under the same id: the removal is undone.
    assert!(!change.carried_by(&with(&child, junk(9, 1))));
}

#[test]
fn a_later_edit_of_a_changed_node_or_a_moved_entry_breaks_the_carry() {
    let parent = founder();
    let child = with(&parent, junk(9, 1));
    let change = Change::between(&parent, &child);
    assert_eq!(change.added, [junk(9, 1)]);
    assert!(change.carried_by(&with(&child, junk(10, 0))));
    let mut edited = child.clone();
    edited.nodes.last_mut().unwrap().backend_def = vm(Vec::new());
    assert!(!change.carried_by(&edited));

    let mut moved = child.clone();
    moved.entry_node_id = NodeId(9);
    let entry = Change::between(&child, &moved);
    assert_eq!(entry.entry, Some(NodeId(9)));
    assert!(entry.added.is_empty() && entry.removed.is_empty());
    assert!(entry.carried_by(&moved) && !entry.carried_by(&child));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn a_child_carries_its_own_change_and_its_parent_only_when_the_change_is_empty(
        seed in any::<u64>(),
        rate in 0.0f64..0.3,
    ) {
        let parent = founder();
        let mut config = arena_config(48);
        config.mutation.per_unit_rate = rate;
        let mut child = parent.clone();
        MutationEngine::apply_mutations_with_food_type_count(
            &mut child,
            &config.mutation,
            &mesh_reachable_nodes(&parent),
            ParentExecuted::NONE,
            &mut SmallRng::seed_from_u64(seed),
            config.world.food.types.len(),
        );
        let change = Change::between(&parent, &child);
        prop_assert!(change.carried_by(&child));
        let empty = change == Change::default();
        prop_assert_eq!(change.carried_by(&parent), empty);
        if child == parent {
            prop_assert!(empty);
        }
    }
}

/// Genomes and births a fixture's members point into.
struct Lineages {
    base: CreatureGenome,
    /// `base` plus node 50: the selected change.
    changed: CreatureGenome,
    /// `changed` plus node 51: carries the change.
    deeper: CreatureGenome,
    /// `base` plus node 52: lacks the change.
    other: CreatureGenome,
    births: Vec<Birth>,
}

impl Lineages {
    fn new() -> Self {
        let base = founder();
        let changed = with(&base, junk(50, 1));
        let deeper = with(&changed, junk(51, 1));
        let other = with(&base, junk(52, 1));
        let birth = |parent: &CreatureGenome, child: &CreatureGenome| Birth {
            change: Arc::new(Change::between(parent, child)),
            ..Birth::default()
        };
        let births = vec![
            birth(&base, &changed),
            birth(&changed, &deeper),
            birth(&changed, &other),
            birth(&base, &other),
        ];
        Self {
            base,
            changed,
            deeper,
            other,
            births,
        }
    }
}

/// `(id, parent, carried, genome, applied, birth, scalar)`.
type Spec<'a> = (
    u64,
    Option<u64>,
    bool,
    &'a CreatureGenome,
    u64,
    Option<&'a Birth>,
    f64,
);

fn members<'a>(specs: &[Spec<'a>]) -> Vec<Member<'a>> {
    specs
        .iter()
        .map(
            |&(id, parent, carried, genome, applied, birth, scalar)| Member {
                id,
                parent,
                carried,
                genome,
                applied,
                birth,
                identical: false,
                scalar,
                scenes: &[],
                sequences: &[],
            },
        )
        .collect()
}

fn no_battery(_: &CreatureGenome) -> BatterySignature {
    unreachable!("no touching child")
}

fn step(tracker: &mut Tracker, specs: &[Spec<'_>], survivors: Option<&[u64]>) -> LadderRow {
    let pending = tracker.step(&members(specs), survivors, &no_battery);
    let row = pending.row().clone();
    tracker.commit(pending);
    row
}

/// Generation 1: carried parent 0 (`base`) and child 5 (`changed`), an
/// improvement that survives.
fn selected(tracker: &mut Tracker, l: &Lineages) {
    let row = step(
        tracker,
        &[
            (0, None, true, &l.base, 0, None, 1.0),
            (5, Some(0), false, &l.changed, 1, Some(&l.births[0]), 2.0),
        ],
        Some(&[5, 0]),
    );
    assert_eq!(
        (row.retention.selected, row.retention.selected_touching),
        (1, 0)
    );
    assert_eq!(row.children.other.children, 1);
    assert_eq!(row.children.other.outcomes.improved, 1);
    assert_eq!(row.supply.births, 1);
}

#[test]
fn a_carrying_survivor_at_each_depth_resolves_retained() {
    let l = Lineages::new();
    let mut tracker = Tracker::new(2);
    selected(&mut tracker, &l);
    let row = step(
        &mut tracker,
        &[
            (5, None, true, &l.changed, 1, None, 2.0),
            (0, None, true, &l.base, 0, None, 1.0),
            (9, Some(5), false, &l.deeper, 2, Some(&l.births[1]), 1.5),
        ],
        Some(&[9, 5]),
    );
    assert_eq!(row.retention.depths, [[1, 0, 0], [0, 0, 0]]);
    assert_eq!(row.retention.selected, 0, "not an improvement");
    let row = step(
        &mut tracker,
        &[
            (9, None, true, &l.deeper, 2, None, 1.5),
            (5, None, true, &l.changed, 1, None, 2.0),
            (12, Some(9), false, &l.deeper, 3, Some(&l.births[1]), 1.0),
        ],
        Some(&[12]),
    );
    assert_eq!(row.retention.depths, [[0, 0, 0], [1, 0, 0]]);
    tracker.censor();
    assert_eq!(tracker.pooled.retention.depths, [[1, 0, 0], [1, 0, 0]]);
    assert_eq!(tracker.pooled.retention.censored, [0, 0]);
}

#[test]
fn descendants_that_all_lack_the_change_resolve_deleted() {
    let l = Lineages::new();
    let mut tracker = Tracker::new(2);
    selected(&mut tracker, &l);
    // The change's own carried copy still carries it: open.
    let row = step(
        &mut tracker,
        &[
            (5, None, true, &l.changed, 1, None, 2.0),
            (0, None, true, &l.base, 0, None, 1.0),
            (9, Some(5), false, &l.other, 2, Some(&l.births[2]), 1.5),
        ],
        Some(&[9]),
    );
    assert_eq!(row.retention.depths, [[0, 0, 0], [0, 0, 0]]);
    let row = step(
        &mut tracker,
        &[
            (9, None, true, &l.other, 2, None, 1.5),
            (12, Some(9), false, &l.other, 3, Some(&l.births[3]), 1.0),
        ],
        Some(&[12]),
    );
    assert_eq!(row.retention.depths, [[0, 1, 0], [0, 1, 0]]);
}

#[test]
fn no_descendant_in_the_evaluated_population_resolves_lineage_loss() {
    let l = Lineages::new();
    let mut tracker = Tracker::new(2);
    selected(&mut tracker, &l);
    step(
        &mut tracker,
        &[
            (5, None, true, &l.changed, 1, None, 2.0),
            (0, None, true, &l.base, 0, None, 1.0),
            (9, Some(0), false, &l.other, 1, Some(&l.births[3]), 3.0),
        ],
        Some(&[9, 0]),
    );
    let row = step(
        &mut tracker,
        &[
            (9, None, true, &l.other, 1, None, 3.0),
            (0, None, true, &l.base, 0, None, 1.0),
        ],
        Some(&[9]),
    );
    assert_eq!(row.retention.depths, [[0, 0, 1], [0, 0, 1]]);
}

#[test]
fn a_non_carrying_branch_deeper_beside_a_carrying_branch_below_decides_nothing() {
    let l = Lineages::new();
    let mut tracker = Tracker::new(2);
    selected(&mut tracker, &l);
    // 9 carries at depth 1 but does not survive; 10 lacks it at depth 2
    // and survives, beside the carried copy 5.
    let row = step(
        &mut tracker,
        &[
            (5, None, true, &l.changed, 1, None, 2.0),
            (0, None, true, &l.base, 0, None, 1.0),
            (9, Some(5), false, &l.deeper, 2, Some(&l.births[1]), 1.0),
            (10, Some(5), false, &l.other, 3, Some(&l.births[2]), 1.0),
        ],
        Some(&[10, 5]),
    );
    assert_eq!(row.retention.depths, [[0, 0, 0], [0, 0, 0]]);
    tracker.censor();
    assert_eq!(tracker.pooled.retention.censored, [1, 1]);
    assert_eq!(tracker.pooled.retention.censored_touching, [0, 0]);
}

#[test]
fn a_refused_row_records_nothing_and_its_open_changes_are_censored() {
    let l = Lineages::new();
    let mut tracker = Tracker::new(2);
    selected(&mut tracker, &l);
    let before = tracker.pooled.clone();
    // The next row would resolve depth 1, but it is refused: no commit.
    let pending = tracker.step(
        &members(&[
            (5, None, true, &l.changed, 1, None, 2.0),
            (0, None, true, &l.base, 0, None, 1.0),
            (9, Some(5), false, &l.deeper, 2, Some(&l.births[1]), 1.5),
        ]),
        Some(&[9, 5]),
        &no_battery,
    );
    assert_eq!(pending.row().retention.depths[0], [1, 0, 0]);
    assert_eq!(tracker.pooled, before);
    tracker.censor();
    assert_eq!(tracker.pooled.retention.depths, [[0, 0, 0], [0, 0, 0]]);
    assert_eq!(tracker.pooled.retention.censored, [1, 1]);
}

#[test]
fn a_generation_without_survivors_selects_nothing_and_its_open_depths_are_censored() {
    let l = Lineages::new();
    let mut tracker = Tracker::new(2);
    selected(&mut tracker, &l);
    let row = step(
        &mut tracker,
        &[
            (5, None, true, &l.changed, 1, None, 2.0),
            (0, None, true, &l.base, 0, None, 1.0),
            (9, Some(5), false, &l.deeper, 2, Some(&l.births[1]), 3.0),
        ],
        None,
    );
    assert_eq!(row.retention.selected, 0, "no survivors, no selection");
    assert_eq!(row.retention.depths, [[0, 0, 0], [0, 0, 0]]);
    tracker.censor();
    assert_eq!(tracker.pooled.retention.censored, [1, 1]);
}

#[test]
fn touching_children_are_classed_on_the_battery_and_viable_needs_a_behavior_change() {
    let config = arena_config(48);
    let batteries = Batteries::new(&config);
    let signature = |genome: &CreatureGenome| batteries.signature(&config, genome);
    let parent = founder();
    let silent = with(&parent, junk(60, 1));
    let mut dead = parent.clone();
    for n in &mut dead.nodes {
        if let BackendDef::Graph(graph) = &mut n.backend_def {
            for sink in &mut graph.output_sinks {
                if matches!(sink.kind, OutputSinkKind::ActionVote(_)) {
                    sink.inputs.clear();
                }
            }
        }
    }
    let touching = |child: &CreatureGenome| Birth {
        touching: true,
        change: Arc::new(Change::between(&parent, child)),
        ..Birth::default()
    };
    let births = [touching(&silent), touching(&silent), touching(&dead)];
    let moved: Vec<Vec<Boundary>> = vec![vec![(Position { x: 1, y: 1 }, 1.0)]];
    let still: Vec<Vec<Boundary>> = vec![Vec::new()];
    let member = |id, genome, birth, scalar, sequences| Member {
        id,
        parent: (id > 0).then_some(0),
        carried: id == 0,
        genome,
        applied: u64::from(id > 0),
        birth,
        identical: false,
        scalar,
        scenes: &[],
        sequences,
    };
    let view = [
        member(0, &parent, None, 1.0, &still[..]),
        member(1, &silent, Some(&births[0]), 1.0, &still[..]),
        member(2, &silent, Some(&births[1]), 2.0, &moved[..]),
        member(3, &dead, Some(&births[2]), 0.0, &moved[..]),
    ];
    let row = Tracker::new(2).step(&view, None, &signature).row().clone();
    let t = &row.children.touching;
    assert_eq!((t.children, t.scene_changed), (3, 2));
    assert_eq!((t.silent, t.changed, t.dead), (2, 0, 1));
    assert_eq!(t.viable, 1, "only the silent child whose scenes changed");
    assert_eq!((t.outcomes.improved, t.outcomes.delta_max), (1, Some(1.0)));
    assert_eq!(row.supply.touching_births, 3);
    assert_eq!(row.children.other.children, 0);
    // The children placed alone at the defaults: inconclusive (n < 60).
    let mut pooled = Pooled::new(2);
    pooled.add(&row);
    assert_eq!(
        pooled.statuses(&crate::ladder::StallRates::default())[1].status,
        Status::Inconclusive
    );
}
