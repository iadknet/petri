use super::*;
use crate::creature::founder::v3alpha1_founder_genome;
use rand::{rngs::SmallRng, SeedableRng};

#[test]
fn recruitment_off_preserves_pre_feature_genomes_events_and_rng() {
    // Whole-family recruitment is on by default since 2026-10-12; this pin
    // keeps reading the path with it off, which must add no draw.
    let config = MutationConfig {
        neutral_input_recruitment: crate::config::NeutralInputRecruitment::Off,
        ..MutationConfig::default()
    };
    let mut hash = 0xcbf29ce484222325u64;
    for seed in 0..256 {
        let mut genome = v3alpha1_founder_genome();
        let mut rng = SmallRng::seed_from_u64(seed);
        let summary = MutationEngine::apply_mutations_with_food_type_count(
            &mut genome,
            &config,
            &[0, 1, 2],
            ParentExecuted::NONE,
            &mut rng,
            2,
        );
        let bytes = format!(
            "{}{:?}{}",
            serde_json::to_string(&genome).unwrap(),
            summary.events,
            rng.gen::<u64>()
        );
        for byte in bytes.bytes() {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
    }
    // Recruitment `Off` adds no draw relative to the current production path.
    // Re-pinned by T20.F16: Graph-node `Add` draws three more numbers and
    // appends a vote edge (3980442848510747838 -> 13774101911075560278).
    // Re-pinned by T11.F29: the `AddFeedbackRoute` entry remaps every
    // topology draw (13774101911075560278 -> 3281291995176387451).
    // Re-pinned by T11.F31: five ring projection operators join the Graph
    // default arm (total weight 44 -> 56), remapping every Graph draw
    // (3281291995176387451 -> 16135082583858842888).
    // Re-pinned by T11.F30: three drive sinks join every Graph node, remapping
    // every sink and surface draw (16135082583858842888 -> 11747062888261140569).
    // Re-pinned by T02.F08: every graph genome carries the `EatLeaveShare`
    // sink and the surface draw covers `c + 98`
    // (11747062888261140569 -> 13078250158860285847).
    // Re-pinned by T25.F02 commit 1 (unit rule): the production supply draws
    // one `gen_bool` per `genome_size()` unit, and the founder falls 97 -> 80
    // (13078250158860285847 -> 4353978680418239853); the old size rule and
    // anchor 97 together restore. Re-pinned by T25.F02 commit 2 (default
    // rate 0.005 -> 0.0060625; 4353978680418239853 -> 7164882519974651247);
    // rate 0.005 restores.
    // Re-pinned by T25.F03 commit 1 (text only): the genome JSON names
    // `own_weight` (7164882519974651247 -> 12816372541544823979); the hash
    // without the genome JSON is 8964731210341994819 at the merge base and
    // after the commit, and reverting the key rename restores.
    // Re-pinned by T25.F03 commit 2: the five projection operators become
    // the edge-group operators at the same catalog positions and weights, so
    // the event keys and the genome text of every group-creating draw change
    // (12816372541544823979 -> 7833229115364439623); reverting the commit
    // restores.
    // Re-pinned by T25.F04 commit 1: `AddDirectionPairGroup` joins the Graph
    // default arm (weight 56 -> 58) and remaps every Graph draw
    // (7833229115364439623 -> 4902026005096458209); removing the catalog
    // entry alone restores.
    // Re-pinned by T25.F07 commit 1 (text only): every member's JSON names
    // its `scope` and every group's its `inherit_scale`
    // (4902026005096458209 -> 11853091526805612651); the hash without the
    // genome JSON is 14048641739953688336 at the merge base and after the
    // commit.
    // Re-pinned by T25.F07 commit 2: `SetScope` joins the Graph default arm
    // (weight 58 -> 60), the plasticity operators admit members and draw the
    // scope coin, and group creation draws its learning
    // (11853091526805612651 -> 6729442929459510286).
    // Held by T25.F08 commit 1: this path never enabled refinement, so its
    // enabled set did not change. Re-pinned by T25.F08 commit 2: the two
    // randomized creation operators and `DetachGraphEdge` join the Graph
    // catalog (enabled weight 60 -> 66) and remap every Graph draw
    // (6729442929459510286 -> 1771622774480443924).
    // Re-pinned by T25.F09 commit 1 (text only): every plastic edge's JSON
    // names its `retention` and every group's its `retention`
    // (1771622774480443924 -> 2113204031778778398); the hash without the
    // genome JSON is 8882269626750794464 at the merge base and after the
    // commit.
    // Re-pinned by T25.F09 commit 3: the six retention operators join the
    // Graph catalog (enabled weight 66 -> 78 on this path) and remap every
    // Graph draw (2113204031778778398 -> 5304583554271412245).
    // Held by T27.F01 commit 1 (the 40-wide creature family). Re-pinned by
    // T27.F01 commit 2 after two agreeing runs: the key draw is
    // `gen_range(0..30)` over the bucket table and the three own-colour keys
    // join the width-1 introspection `Swap` alternatives
    // (5304583554271412245 -> 2059421590127301874).
    // Re-pinned by T27.F03 commit 1 (text only, sink serde and `Debug`
    // shape): a directed vote sink reads `{"Directed":["Move",3]}` and
    // `Directed(Move, 3)` where `{"Move":3}` and `Move(3)` stood
    // (2059421590127301874 -> 12995421853181291158); mapping that text back
    // to the old shape restores the old value.
    // Re-pinned by T27.F03 commit 3 (text only): parameter field 3 is
    // `StealEffort` where `StealEnergyAmount` stood
    // (12995421853181291158 -> 1565583786713441820); mapping the name back
    // restores the old value.
    // Re-pinned by T27.F03 commit 4 after two agreeing runs: the
    // `GiveEnergy` kind and `GiveEffort` widen the kind, sink and field
    // draws (1565583786713441820 -> 6751617150376296849).
    assert_eq!(hash, 6751617150376296849, "pin of the recruitment-Off path");
}

#[test]
fn recruitment_configuration_accepts_every_arm_and_defaults_to_whole_family() {
    let original = serde_json::to_value(MutationConfig::default()).unwrap();
    for arm in ["Off", "SingleChannel", "WholeFamily"] {
        let mut json = original.clone();
        json["neutral_input_recruitment"] = serde_json::json!(arm);
        let decoded: MutationConfig = serde_json::from_value(json).unwrap();
        assert_eq!(
            serde_json::to_value(decoded).unwrap()["neutral_input_recruitment"],
            arm
        );
    }
    let mut omitted = original;
    omitted
        .as_object_mut()
        .unwrap()
        .remove("neutral_input_recruitment");
    let decoded: MutationConfig = serde_json::from_value(omitted).unwrap();
    assert_eq!(
        serde_json::to_value(decoded).unwrap()["neutral_input_recruitment"],
        "WholeFamily"
    );
    assert_eq!(
        MutationConfig::default().neutral_input_recruitment,
        crate::config::NeutralInputRecruitment::WholeFamily
    );
}

proptest::proptest! {
    #[test]
    fn recruitment_is_admitted_only_by_opt_in_and_never_under_restricted_pressure(seed in proptest::prelude::any::<u64>(), whole in proptest::prelude::any::<bool>()) {
        use crate::config::NeutralInputRecruitment as Arm;
        let mut config = MutationConfig { neutral_input_recruitment: if whole { Arm::WholeFamily } else { Arm::SingleChannel }, per_unit_rate: 1.0, mesh_layer_probability: 0.0, ..MutationConfig::default() };
        let mut genome = v3alpha1_founder_genome();
        let summary = MutationEngine::apply_mutations_on_units(&mut genome, 64, &config, &[0, 1, 2], ParentExecuted::NONE, &mut SmallRng::seed_from_u64(seed), 2);
        proptest::prop_assert_eq!(summary.attempted_events, 64);
        proptest::prop_assert_eq!(summary.applied_events + summary.skipped_events, 64);
        let key = MutationOperator::GraphRecruitNeutralInput;
        proptest::prop_assert_eq!(summary.applied_by_operator.get(&key).copied().unwrap_or(0) as usize, summary.recruitment_deltas.len());
        for event in &summary.events {
            if event.operator == Some(key) { proptest::prop_assert!(event.target.is_some()); }
        }
        config.genome_size_pressure_enabled = true;
        config.genome_size_cap = 1;
        let mut genome = v3alpha1_founder_genome();
        let summary = MutationEngine::apply_mutations_on_units(&mut genome, 1, &config, &[], ParentExecuted::NONE, &mut SmallRng::seed_from_u64(seed), 2);
        proptest::prop_assert!(!summary.operator_funnel_by_operator.contains_key(&key));
    }
}
