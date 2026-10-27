//! T25.F11: the one birth policy and the bundle edits the operators report.
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::config::test_sensor_context;
use crate::contracts::votes::DirectedVoteKind;
use crate::contracts::{FieldSet, InputReference, SensorDescriptor, WorldInputKey, MEMORY_BANK};
use crate::genome::cgp::{
    plans, test_support::edit_sinks, BundleSource, CgpGraphBackendDef, GroupId, OutputSinkKind,
    Wiring,
};
use crate::genome::{BackendDef, NodeGenome};
use crate::mutation::graph::group::{draw_birth_learning, draw_births, BirthLaw};
use crate::mutation::graph::operators::{remove_edge, retarget_edge, test_inputs};
use crate::mutation::types::{BundleEdits, MutationSkipReason};

fn area_sets() -> Vec<(u8, FieldSet)> {
    let area = InputReference::World(WorldInputKey::AreaBarrierSummary);
    SensorDescriptor::of(&area, &test_sensor_context(1))
        .field_sets()
        .enumerate()
        .map(|(index, set)| (u8::try_from(index).unwrap(), set))
        .collect()
}

/// A silent birth takes no scale draw; a live one takes exactly one, after
/// its plan draws, and each wiring is admissible for its set.
#[test]
fn the_birth_law_draws_a_scale_only_when_live() {
    let def = CgpGraphBackendDef::new_with_fixed_outputs();
    let sets = area_sets();
    for seed in 0..32 {
        let mut silent_rng = SmallRng::seed_from_u64(seed);
        let silent = draw_births(&def, &sets, BirthLaw::Silent, &mut silent_rng).unwrap();
        let mut live_rng = SmallRng::seed_from_u64(seed);
        let live = draw_births(&def, &sets[..1], BirthLaw::Live, &mut live_rng).unwrap();
        assert!(silent.iter().all(|birth| birth.scale == 0.0));
        assert_eq!(silent.len(), 2);
        for birth in &silent {
            let set = sets[usize::from(birth.field_set)].1;
            assert!(plans(&def, &set).contains(&birth.wiring));
        }
        // Set 0's plan draws are the same under both laws; the live scale
        // is the next draw of the stream.
        assert_eq!(live[0].wiring, silent[0].wiring);
        let mut replay = SmallRng::seed_from_u64(seed);
        replay.gen_range(0..DirectedVoteKind::ALL.len());
        replay.gen_range(0..8u8);
        assert_eq!(live[0].scale, replay.gen_range(-1.0f32..=1.0));
        assert!((-1.0..=1.0).contains(&live[0].scale));
    }
}

/// A `Plain` set's plan is one draw over the present vote sinks in catalog
/// order; a geometric set's the kind over the hostable kinds, then the
/// offset.
#[test]
fn the_plan_draws_follow_the_admissible_order() {
    let def = CgpGraphBackendDef::new_with_fixed_outputs();
    let plain = plans(&def, &MEMORY_BANK);
    for seed in 0..32 {
        let mut rng = SmallRng::seed_from_u64(seed);
        let births = draw_births(&def, &[(0, MEMORY_BANK)], BirthLaw::Silent, &mut rng).unwrap();
        let mut replay = SmallRng::seed_from_u64(seed);
        assert_eq!(births[0].wiring, plain[replay.gen_range(0..plain.len())]);

        let mut rng = SmallRng::seed_from_u64(seed);
        let births = draw_births(&def, &[(0, FieldSet::Ring)], BirthLaw::Silent, &mut rng).unwrap();
        let mut replay = SmallRng::seed_from_u64(seed);
        let kind = DirectedVoteKind::ALL[replay.gen_range(0..DirectedVoteKind::ALL.len())];
        let offset = replay.gen_range(0..8u8);
        assert_eq!(births[0].wiring, Wiring::Ring { kind, offset });
    }
}

/// A set with no admissible plan skips the event.
#[test]
fn a_set_without_an_admissible_plan_skips() {
    let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
    edit_sinks(&mut def, |sinks| {
        sinks.retain(|sink| !matches!(sink.kind, OutputSinkKind::ActionVote(_)));
    });
    let mut rng = SmallRng::seed_from_u64(0);
    assert_eq!(
        draw_births(&def, &[(0, MEMORY_BANK)], BirthLaw::Live, &mut rng),
        Err(MutationSkipReason::NoApplicableTarget)
    );
}

/// The learning draws run per bundle after materialization: the form coin,
/// then `draw_learning` in that form; the report counts each creation by
/// wiring and form.
#[test]
fn birth_learning_reports_each_creation_by_wiring_and_form() {
    let context = test_sensor_context(1);
    for seed in 0..16 {
        let mut node = NodeGenome {
            node_id: crate::contracts::NodeId::new(0),
            input_refs: vec![InputReference::World(WorldInputKey::NeighborBarrierRing)],
            backend_def: BackendDef::Graph(CgpGraphBackendDef::new_with_fixed_outputs()),
            targets: vec![],
        };
        let wiring = Wiring::Ring {
            kind: DirectedVoteKind::Move,
            offset: 2,
        };
        let source = BundleSource::Sensor {
            ref_idx: 0,
            field_set: 0,
        };
        let birth = crate::genome::cgp::BundleBirth {
            field_set: 0,
            wiring,
            scale: 0.0,
        };
        let id = node.add_bundle(source, birth, &context).unwrap();
        let BackendDef::Graph(def) = &mut node.backend_def else {
            unreachable!()
        };
        let mut rng = SmallRng::seed_from_u64(seed);
        let edits = draw_birth_learning(def, &[id], &[wiring], &mut rng);
        let mut replay = SmallRng::seed_from_u64(seed);
        let randomized = replay.gen_bool(0.5);
        assert_eq!(edits, BundleEdits::creation(&wiring, randomized));
        assert_eq!(edits.created[1].iter().sum::<u8>(), 1);
        assert_eq!(id, GroupId::new(0));
    }
}

/// `RemoveGraphEdge` reports a record removal exactly when it took a last
/// member; `RetargetGraphEdge` a detach when it moved a member.
#[test]
fn edge_operators_report_their_bundle_edits() {
    let context = test_sensor_context(1);
    let mut node = NodeGenome {
        node_id: crate::contracts::NodeId::new(0),
        input_refs: vec![InputReference::World(WorldInputKey::NeighborBarrierRing)],
        backend_def: BackendDef::Graph(CgpGraphBackendDef::new_with_fixed_outputs()),
        targets: vec![],
    };
    let wiring = Wiring::Ring {
        kind: DirectedVoteKind::Move,
        offset: 0,
    };
    let source = BundleSource::Sensor {
        ref_idx: 0,
        field_set: 0,
    };
    let birth = crate::genome::cgp::BundleBirth {
        field_set: 0,
        wiring,
        scale: 0.5,
    };
    node.add_bundle(source, birth, &context).unwrap();
    let refs = node.input_refs.clone();
    let BackendDef::Graph(def) = &mut node.backend_def else {
        unreachable!()
    };
    // Only members are on the graph: every retarget that changes a source
    // detaches one, and the eighth member's removal takes the record.
    let mut seed = 0;
    while def.groups().len() == 1 {
        let members = def.edges().count();
        let edits =
            retarget_edge(def, test_inputs(&refs), &mut SmallRng::seed_from_u64(seed)).unwrap();
        let detached = members - def.members(GroupId::new(0)).count();
        assert_eq!(usize::from(edits.detached), detached.min(1));
        seed += 1;
        if detached > 0 {
            break;
        }
    }
    let mut removals = 0;
    while !def.groups().is_empty() {
        let edits = remove_edge(def, &mut SmallRng::seed_from_u64(seed)).unwrap();
        removals += u32::from(edits.removed);
        seed += 1;
    }
    assert_eq!(removals, 1);
}
