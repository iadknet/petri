//! Mutation reach of the kind drives and the birth edge class (T11.F30):
//! the existing draws over the catalog reach every drive, `AddGraphEdge` and
//! a creation that makes one `Plain` bundle report the class of the sink
//! they pushed onto, and a birth's class is defined only for one applied
//! event of `AddGraphEdge` or a creation operator (T25.F11 invariant 8).

use std::collections::BTreeSet;

use proptest::prelude::*;
use rand::{rngs::SmallRng, SeedableRng};

use crate::contracts::votes::{DirectedVoteKind, VoteSink};
use crate::contracts::InputReference;
use crate::contracts::NodeId;
use crate::genome::cgp::{CgpGraphBackendDef, OutputSinkKind};
use crate::genome::{BackendDef, NodeGenome, VmInstruction};
use crate::mutation::graph::bundle::{add_sensor_by_law, BirthLaw};
use crate::mutation::graph::operators::add_edge;
use crate::mutation::graph::GraphOperator;
use crate::mutation::input_ref::InputRefOperator;
use crate::mutation::types::{BirthEdgeClass, MutationDomain, MutationOperator, MutationSummary};
use crate::mutation::vm::operators::random_vm_instruction;
use crate::mutation::vm::VmOperator;

/// The vote sink whose edge count grew between `before` and `after`.
fn grown_vote_sink(before: &CgpGraphBackendDef, after: &CgpGraphBackendDef) -> Option<VoteSink> {
    before
        .output_sinks()
        .iter()
        .zip(after.output_sinks())
        .find(|(old, new)| new.inputs.len() > old.inputs.len())
        .and_then(|(_, new)| match new.kind {
            OutputSinkKind::ActionVote(sink) => Some(sink),
            _ => None,
        })
}

/// A Graph node on a full sink catalog declaring nothing.
fn bare_graph_node() -> NodeGenome {
    NodeGenome::new(
        NodeId(0),
        Vec::new(),
        BackendDef::Graph(CgpGraphBackendDef::new_with_fixed_outputs()),
        Vec::new(),
    )
}

/// `AddSensorLive` of `reference` onto a bare Graph node with `seed`: the
/// node's def before and after, and the class the creation reports.
fn add_sensor_on_bare_node(
    reference: InputReference,
    seed: u64,
) -> (
    CgpGraphBackendDef,
    CgpGraphBackendDef,
    Option<BirthEdgeClass>,
) {
    let mut node = bare_graph_node();
    let BackendDef::Graph(before) = node.backend_def().clone() else {
        unreachable!()
    };
    let (_, class) = add_sensor_by_law(
        &mut node,
        reference,
        BirthLaw::Live,
        &crate::config::test_sensor_context(1),
        &mut SmallRng::seed_from_u64(seed),
    )
    .expect("a full catalog hosts every Plain bundle");
    let BackendDef::Graph(after) = node.backend_def() else {
        unreachable!()
    };
    (before, after.clone(), class)
}

fn drives() -> BTreeSet<usize> {
    DirectedVoteKind::ALL
        .map(|kind| kind.drive().index())
        .into_iter()
        .collect()
}

#[test]
fn add_graph_edge_and_add_sensor_reach_every_drive() {
    let refs = [InputReference::ActionVotes];
    let mut by_edge = BTreeSet::new();
    let mut by_input = BTreeSet::new();
    for seed in 0..4_096 {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        let before = def.clone();
        let class = add_edge(
            &mut def,
            crate::mutation::graph::operators::test_inputs(&refs),
            &mut SmallRng::seed_from_u64(seed),
        )
        .unwrap();
        if let Some(sink @ VoteSink::Drive(_)) = grown_vote_sink(&before, &def) {
            assert_eq!(class, Some(BirthEdgeClass::Drive));
            by_edge.insert(sink.index());
        }
        let (before, def, class) = add_sensor_on_bare_node(InputReference::UpstreamSlot(0), seed);
        if let Some(sink @ VoteSink::Drive(_)) = grown_vote_sink(&before, &def) {
            assert_eq!(class, Some(BirthEdgeClass::Drive));
            by_input.insert(sink.index());
        }
    }
    assert_eq!(by_edge, drives());
    assert_eq!(by_input, drives());
}

/// The Graph event dispatch reports the class `AddGraphEdge` pushed, not
/// only the operator itself (T25.F04 moved the report into the dispatch).
#[test]
fn a_dispatched_add_graph_edge_reports_its_vote_sink_class() {
    use crate::config::MutationConfig;
    use crate::creature::founder::v3alpha1_founder_genome;
    use crate::genome::BackendDef;
    use crate::mutation::graph::{GraphMutator, GraphOperator};
    use crate::mutation::reachability::TargetSelector;

    let base = v3alpha1_founder_genome();
    let all: Vec<usize> = (0..base.nodes.len()).collect();
    let config = MutationConfig::default();
    let mut classified = 0;
    for seed in 0..256 {
        let mut genome = base.clone();
        let mut targets = TargetSelector::reachable_only(&all, 0.0);
        let Ok((_, report)) = GraphMutator::apply_classified(
            &mut genome,
            GraphOperator::AddGraphEdge,
            &mut targets,
            &mut SmallRng::seed_from_u64(seed),
            &config,
            &crate::config::test_sensor_context(1),
        ) else {
            continue;
        };
        let pick = targets
            .first_pick()
            .expect("an applied event records its pick");
        let (BackendDef::Graph(before), BackendDef::Graph(after)) = (
            base.nodes[pick].backend_def(),
            genome.nodes[pick].backend_def(),
        ) else {
            panic!("AddGraphEdge picks a Graph node");
        };
        let expected = grown_vote_sink(before, after).and_then(BirthEdgeClass::of_sink);
        assert_eq!(report.class, expected, "seed {seed}");
        classified += usize::from(expected.is_some());
    }
    assert!(
        classified > 0,
        "some seed pushes onto a classified vote sink"
    );
}

#[test]
fn vm_add_vote_reaches_every_drive() {
    let mut rng = SmallRng::seed_from_u64(30);
    let mut reached = BTreeSet::new();
    for _ in 0..20_000 {
        if let VmInstruction::AddVote { sink, .. } = random_vm_instruction(&mut rng, 4, 1, 1) {
            assert!(usize::from(sink) < crate::contracts::votes::VOTE_SINK_COUNT);
            if let Some(VoteSink::Drive(kind)) = VoteSink::from_index(usize::from(sink)) {
                reached.insert(kind.drive().index());
            }
        }
    }
    assert_eq!(reached, drives());
}

proptest! {
    /// The class `add_edge` and a one-`Plain`-bundle `AddSensor*` report is
    /// the class of the vote sink they pushed onto, `None` when no vote sink
    /// grew; a sensor of several field sets reports none.
    #[test]
    fn the_reported_class_is_the_class_of_the_pushed_sink(seed in any::<u64>(), slot in 0usize..24) {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        let before = def.clone();
        let class = add_edge(&mut def, crate::mutation::graph::operators::test_inputs(&[InputReference::ActionVotes]), &mut SmallRng::seed_from_u64(seed)).unwrap();
        prop_assert_eq!(class, grown_vote_sink(&before, &def).and_then(BirthEdgeClass::of_sink));

        for reference in [InputReference::UpstreamSlot(slot), InputReference::ActionVotes] {
            let (before, def, class) = add_sensor_on_bare_node(reference, seed);
            let grown = grown_vote_sink(&before, &def);
            prop_assert!(grown.is_some(), "a full catalog carries every drawn sink");
            prop_assert_eq!(class, grown.and_then(BirthEdgeClass::of_sink));
        }
        let area = InputReference::World(crate::contracts::WorldInputKey::AreaBarrierSummary);
        let (_, _, class) = add_sensor_on_bare_node(area, seed);
        prop_assert_eq!(class, None, "two pair bundles carry no class");
        let ring = InputReference::World(crate::contracts::WorldInputKey::NeighborBarrierRing);
        let (_, _, class) = add_sensor_on_bare_node(ring, seed);
        prop_assert_eq!(class, None, "one geometric bundle carries no class");
    }
}

#[test]
fn the_class_of_every_sink_is_drive_direction_or_none() {
    for sink in VoteSink::all() {
        let expected = match sink {
            VoteSink::Drive(_) => Some(BirthEdgeClass::Drive),
            VoteSink::Directed(..) => Some(BirthEdgeClass::Direction),
            VoteSink::Eat | VoteSink::Terminate | VoteSink::Decide => None,
        };
        assert_eq!(BirthEdgeClass::of_sink(sink), expected, "{sink:?}");
    }
    assert_eq!(
        serde_json::to_string(&BirthEdgeClass::ALL).unwrap(),
        "[\"drive\",\"direction\"]"
    );
    let keys = BirthEdgeClass::ALL.map(BirthEdgeClass::as_key);
    assert_eq!(keys, ["drive", "direction"], "keys match the serde names");
}

fn summary(
    events: &[(MutationDomain, MutationOperator, Option<BirthEdgeClass>)],
) -> MutationSummary {
    let mut summary = MutationSummary::zero();
    for &(domain, operator, class) in events {
        summary.record_attempt(domain, operator);
        summary.record_applied(domain, operator);
        if let Some(class) = class {
            summary.record_vote_edge_class(class);
        }
    }
    summary
}

#[test]
fn a_birth_has_a_class_only_for_one_applied_edge_or_creation() {
    use MutationDomain::{Graph, InputRef, Vm};
    let drive = Some(BirthEdgeClass::Drive);
    let direction = Some(BirthEdgeClass::Direction);
    assert_eq!(
        summary(&[(
            Graph,
            MutationOperator::Graph(GraphOperator::AddGraphEdge),
            drive
        )])
        .birth_edge_class(),
        drive
    );
    assert_eq!(
        summary(&[(
            InputRef,
            MutationOperator::InputRef(InputRefOperator::AddSensorLive),
            direction
        )])
        .birth_edge_class(),
        direction
    );
    for operator in [
        MutationOperator::Graph(GraphOperator::AddBundleLive),
        MutationOperator::Graph(GraphOperator::AddBundleSilent),
    ] {
        assert_eq!(
            summary(&[(Graph, operator, direction)]).birth_edge_class(),
            direction
        );
    }
    assert_eq!(
        summary(&[(
            InputRef,
            MutationOperator::InputRef(InputRefOperator::AddSensorSilent),
            drive
        )])
        .birth_edge_class(),
        drive
    );
    // One applied event of any other operator has no class, even with a
    // recorded vote-sink push.
    assert_eq!(
        summary(&[(
            Graph,
            MutationOperator::Graph(GraphOperator::RemoveGraphEdge),
            drive
        )])
        .birth_edge_class(),
        None
    );
    // A compute-input edge, an `Eat` sink or a VM node: no vote-sink push.
    assert_eq!(
        summary(&[(
            Graph,
            MutationOperator::Graph(GraphOperator::AddGraphEdge),
            None
        )])
        .birth_edge_class(),
        None
    );
    // A repeated same-operator birth has no class, whatever both pushed.
    assert_eq!(
        summary(&[
            (
                Graph,
                MutationOperator::Graph(GraphOperator::AddGraphEdge),
                drive
            ),
            (
                Graph,
                MutationOperator::Graph(GraphOperator::AddGraphEdge),
                drive
            )
        ])
        .birth_edge_class(),
        None
    );
    // Any second applied event, of any operator, removes the class.
    assert_eq!(
        summary(&[
            (
                Graph,
                MutationOperator::Graph(GraphOperator::AddGraphEdge),
                drive
            ),
            (
                Graph,
                MutationOperator::Graph(GraphOperator::RemoveGraphEdge),
                None
            )
        ])
        .birth_edge_class(),
        None
    );
    assert_eq!(
        summary(&[
            (
                InputRef,
                MutationOperator::InputRef(InputRefOperator::AddSensorLive),
                direction
            ),
            (
                Vm,
                MutationOperator::Vm(VmOperator::VmDeleteInstruction),
                None
            )
        ])
        .birth_edge_class(),
        None
    );
    assert_eq!(summary(&[]).birth_edge_class(), None);
}
