//! Census pins (T30.F13): every population census the benchmark report and
//! the NDJSON stream emit, pinned on fixture populations against the code
//! before the censuses moved into core's readers. A pin that moves is a
//! defect; only the call expressions below may change, never the literals,
//! except under a predeclared re-pin. T30.F15 re-pinned one literal,
//! `NDJSON_PINS` `stream`: its `tick_sample` and `run_completed` mean
//! energies are now core's `f64` mean cast to `f32`. T25.F12 commit 2 (the
//! one learning type, its catalog and birth draws, `bundles-v2`, the
//! census's source keys and carriers) re-pinned every moved table. T29.F02
//! (body genes: the 85-unit founder, the body's own supply and the sample
//! `body` block) re-pinned every moved table and added the `body` pins.
//!
//! Each pin hashes a sorted-key JSON rendering (`serde_json::Value`), so a
//! mismatch names the block and prints its JSON for a diff against the
//! pre-move commit.

use std::collections::BTreeMap;

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::profiles::{goal_profile_params, NeighborhoodSizes, ProfileParams};
use super::run::build_report;

fn digest(value: &Value) -> String {
    let text = serde_json::to_string(value).expect("a JSON value serializes");
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

/// Compare each named block's digest to its pin, printing every moved
/// block's JSON before failing.
fn assert_pins(blocks: &BTreeMap<String, Value>, pins: &[(&str, &str)]) {
    let measured: BTreeMap<&str, String> = blocks
        .iter()
        .map(|(key, value)| (key.as_str(), digest(value)))
        .collect();
    let expected: BTreeMap<&str, String> = pins
        .iter()
        .map(|&(key, hash)| (key, hash.to_string()))
        .collect();
    let mut moved = Vec::new();
    for (key, hash) in &measured {
        if expected.get(key) != Some(hash) {
            eprintln!("moved pin {key}: {hash}\n{}", blocks[*key]);
            moved.push(*key);
        }
    }
    assert_eq!(
        measured.keys().collect::<Vec<_>>(),
        expected.keys().collect::<Vec<_>>(),
        "the pinned block set"
    );
    assert!(moved.is_empty(), "moved pins: {moved:?}");
}

/// The goal world set shrunk to a population that lives long enough to
/// carry several clades, generations, pending births, mutated structure and
/// plastic edges, with every goal-only indicator measured at small sizes.
fn pin_params() -> ProfileParams {
    let mut params = goal_profile_params();
    params.width = 40;
    params.height = 40;
    params.founders = 48;
    params.ticks = 45;
    params.neighborhood = NeighborhoodSizes {
        founder_operator_trials: 1,
        founder_births: 2,
        evolved_operator_trials: 1,
        evolved_births: 2,
        read_sample: 3,
        read_births: 2,
    };
    params.drift = Default::default();
    params.mutation_effects = Default::default();
    params
}

/// Every key of the report's `deterministic` block, the goal indicators and
/// each case's tracking, and the first case's population samples one by one.
fn report_blocks() -> BTreeMap<String, Value> {
    let report = build_report(&pin_params(), "t30-f13-census-pins").expect("a valid profile");
    let deterministic =
        serde_json::to_value(&report.deterministic).expect("the deterministic block serializes");
    let mut blocks = BTreeMap::new();
    for (key, value) in deterministic.as_object().expect("an object") {
        if key != "goal_indicators" {
            blocks.insert(format!("deterministic.{key}"), value.clone());
        }
    }
    let indicators = &deterministic["goal_indicators"];
    for (key, value) in indicators.as_object().expect("an object") {
        if key != "cases" && key != "population_persistence" {
            blocks.insert(format!("goal_indicators.{key}"), value.clone());
        }
    }
    for (index, case) in indicators["cases"]
        .as_array()
        .expect("case rows")
        .iter()
        .enumerate()
    {
        for (key, value) in case.as_object().expect("an object") {
            blocks.insert(format!("cases[{index}].{key}"), value.clone());
        }
    }
    for (index, row) in indicators["population_persistence"]["per_seed"]
        .as_array()
        .expect("persistence rows")
        .iter()
        .enumerate()
    {
        let samples = row["samples"].as_array().expect("samples");
        let mut head = row.clone();
        head.as_object_mut().expect("an object").remove("samples");
        blocks.insert(format!("persistence[{index}]"), head);
        let last = samples.last().expect("a final sample");
        for (key, value) in last.as_object().expect("an object") {
            blocks.insert(format!("persistence[{index}].last.{key}"), value.clone());
        }
        blocks.insert(
            format!("persistence[{index}].samples"),
            Value::Array(samples.clone()),
        );
    }
    blocks
}

#[test]
fn every_report_census_keeps_its_pre_move_value() {
    let blocks = report_blocks();
    print_pins("REPORT", &blocks);
    assert_pins(&blocks, REPORT_PINS);
}

#[rustfmt::skip]
const REPORT_PINS: &[(&str, &str)] = &[
    ("cases[0].action_attempts", "2a68964ab59c6409dd053150898d765f8546d1c497db49941e8b39abd279672f"),
    ("cases[0].avoidable_blocked_share_of_all_moves_by_reader_state", "ba2ee7397c30bf39e5f429ea5392dcdf8ebfa2080f01afbd8e44182ed0f681d2"),
    ("cases[0].barrier_blocked_fraction_by_reader_state", "36eaae82614253375a80d7b59562d8df5b9817d8b810c8d7dc734123d944f3c6"),
    ("cases[0].blocked_move_fraction", "134ec893e059d2380c05e49ed77ffad50690063fedb063b023c4f0026f89af68"),
    ("cases[0].case", "19d4221af607c2228f56bbbdba2083177e32d4e1ee8d38fd835bc315c2310130"),
    ("cases[0].cognition", "81dd181792f1209310f9223771ba20e2afb7893330e9d8e57d781ad317f0627a"),
    ("cases[0].drift_depth", "07c4bf1006c227ab9a95df5cfee9cecbebb6dd6f7afc4b6db38116b6197ef695"),
    ("cases[0].drives", "33be9cc77adbcf250e0bbff027dfb761ff4b5f5ee15273c026e6ff98c528c592"),
    ("cases[0].energy_flows", "aff04dca31c2043e95789de7103debac0d804cff3cd0f0d037a211543dc865d9"),
    ("cases[0].food_density_total", "99796afa1c5db9317f4b692571d8fe286c444b887d851d2c907cb1aabb45e428"),
    ("cases[0].gestation", "b4de7573af7cb140f4a6a745e89e25c2c59a9ad6eea5a061cdae6b1ba6129f9b"),
    ("cases[0].giving", "dc9dfbd44a34a86675c8c6ebb80b4ddd82783838104680df96142b26cce80cea"),
    ("cases[0].grazed_cell_share", "bc6a862a0d0bccaf0ece69fe9f13b6a348323bc0de0eaf3664bd52e61d1176d7"),
    ("cases[0].grazing_modifier_mean", "145a8c40ed8d59b7635e6dff9121854ab1c318a9c0f5845fa438073ce86b07e5"),
    ("cases[0].input_use", "a45093584a34f8be581ea138863153993e2cd0dee9ceac0d96f8758aceb1ee27"),
    ("cases[0].mesh_dispatches_energy_exhausted_total", "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9"),
    ("cases[0].mortality", "67b6cb92d06d3a17e1f9f47ee7e8672f584f4a11e69c238342ce74338cb68cf7"),
    ("cases[0].move_attempts_with_barrier_neighbor_by_reader_state", "08ff183bb6c41c4cc56ea9399ae9bd537ffa5a39cdd71f6b4055977f1fb73820"),
    ("cases[0].moves_attempted_total", "01a0123885ebec5b37b52ddc058c20d052525c654b69e7b7bfd5feb291428bba"),
    ("cases[0].moves_blocked_avoidable_by_reader_state", "300c640e837a0ba452bd3420749f41ed838fb18d32887ab942e4c75e1963568a"),
    ("cases[0].moves_blocked_barrier_total", "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9"),
    ("cases[0].moves_blocked_barrier_with_barrier_neighbor_by_reader_state", "08ff183bb6c41c4cc56ea9399ae9bd537ffa5a39cdd71f6b4055977f1fb73820"),
    ("cases[0].moves_blocked_total_by_cause", "712e92e2114e9d4668c00006ee0ef5cef2743689a369afae70c923b01ffae622"),
    ("cases[0].mutation_effects", "1a877f0c757d731da0562c28180d6c0b65ccd92b20bdd901232434138661d1cb"),
    ("cases[0].mutation_outcome_summary", "b38742bcaec0a8011d1900f009f3e26a8797c07a7dead01cf375253f22fc20e1"),
    ("cases[0].mutation_supply", "e0b20015cd7454134c345c388e35a86002c22c77720d1bf3aec311513002d10f"),
    ("cases[0].mutation_value_totals_by_birth_edge_class", "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"),
    ("cases[0].mutation_value_totals_by_operator", "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"),
    ("cases[0].mutational_neighborhood", "7ce3c0cf424e7d5d41dba2e6f9ee8934d714da9dc407ccc99c11859defbc26b3"),
    ("cases[0].neighborhood_read", "d10582db439835c21fc2ba637e7146d8022147bbd66669bec9a9b6cfdfee841e"),
    ("cases[0].predation", "9ecdde608453a8db80d2a45463c02cf31564f7300761ad47972d8b9aa8f56236"),
    ("cases[0].reachable_structure_size_distribution", "3e65509f7b1d0d54632ea6d875e74ef7b1ee0e6488fb0e026e5e498f9670db7f"),
    ("cases[0].reproductive_success_by_cognitive_class", "3d6049994a146f498f133dfbd33f93f135314ad4ee908449205a790e6ee12c4b"),
    ("cases[0].ring_rules", "f9e46b72fbeab337dabe368f5e4f9078d06bd63e5d57748e7b9bcd46f735c729"),
    ("cases[0].surviving_clade_profiles", "abf6b147d0bb5046d321a297555922b9adee7ebf215515e2c65f61c242ec20ca"),
    ("cases[0].typed_eat_share", "02a93bda4e2e68b510052110c367661984400806d4bae4426470d0c0ca11ab52"),
    ("cases[0].typed_eats_failed_total", "3d5812abc84c11768aa73a732c85d75dbed439188f5bb3239e9b762ea31d9862"),
    ("cases[0].typed_eats_total", "8ada8db46d26aa873c1f1e74568b2b21a952c0cbefd7a2d7e66be05bcd0d9870"),
    ("cases[1].action_attempts", "9c146223e6c9513da6810a37a4f819ce220ebb67875af49972098c1998fe21c0"),
    ("cases[1].avoidable_blocked_share_of_all_moves_by_reader_state", "d6f36457ca26cfc0e1e44508b5f639e449e90889964ad04ac533b413258d2712"),
    ("cases[1].barrier_blocked_fraction_by_reader_state", "00241fd1645da5ba1008af94b74136e10629dc16ca50007d0747d58bec2f79e5"),
    ("cases[1].blocked_move_fraction", "11d891d200bd3e03d289517098fa55cb705aead47d9278ac210cba26ac682abf"),
    ("cases[1].case", "a4512a782fb565bee661e6e3d1a7e2580297c0b9ceafcf9bb492d1cfe6b85566"),
    ("cases[1].cognition", "703a39c59f7c0323c437d492db0a090ae60d3c3ca6527ae7b0a222c764580b4c"),
    ("cases[1].drift_depth", "71dbf25076397aa9eacf83d249e4e70286ba03445d57415ce5d9d71516d7707d"),
    ("cases[1].drives", "678f9aa6634eb7fabfb2cc39a2209c21bd479baf9358ef2720dfa612dab631bd"),
    ("cases[1].energy_flows", "43b0a7fbcc8f989e70ca7123538f97cb8c01084da79ab1e9fb17f3c619097682"),
    ("cases[1].food_density_total", "5c837b66c450ac926072f5da399e5da36bf45eaedd873abac66e36fecaa47d06"),
    ("cases[1].gestation", "67c5ceba1e7a22c92bd9e07576a131edadd01fc65ffc905eb5cc00812247c8d8"),
    ("cases[1].giving", "dc9dfbd44a34a86675c8c6ebb80b4ddd82783838104680df96142b26cce80cea"),
    ("cases[1].grazed_cell_share", "897dd3ece58ca347fc551be0041fa74e860fdfd6101ec4ea0816c8ddb14c1730"),
    ("cases[1].grazing_modifier_mean", "2c28acc625ae31319dcb4b982d134abeb107a11e1e919ee5fe7b845abc9e3f26"),
    ("cases[1].input_use", "0abfc00b2cd62702491d3204842eb536812760f5f85b868a570e49ee0f94d4ee"),
    ("cases[1].mesh_dispatches_energy_exhausted_total", "6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b"),
    ("cases[1].mortality", "e43130875e035ca2930e1d7edb534c3089c26328aac1da879d1d668a1fdb47bc"),
    ("cases[1].move_attempts_with_barrier_neighbor_by_reader_state", "28ba9fda17d5410f8cc65053861a28daf9d82e236f0b61808d396e38a7c28a4f"),
    ("cases[1].moves_attempted_total", "bc83cbb2d6dcba934deedb695609dd3ae689a72b210a8f1b86f6b1bc4c68d348"),
    ("cases[1].moves_blocked_avoidable_by_reader_state", "e80f34313a2cff6cdd8f123b3c519a45c0b3848a4227ea304aef497a1a032c4d"),
    ("cases[1].moves_blocked_barrier_total", "ad48ff99415b2f007dc35b7eb553fd1eb35ebfa2f2f308acd9488eeb86f71fa8"),
    ("cases[1].moves_blocked_barrier_with_barrier_neighbor_by_reader_state", "3538e8a83831713e0d46b0059d45398d5801f40be0be5e7c87a1e07517809367"),
    ("cases[1].moves_blocked_total_by_cause", "de63eb16107911b99047dd6f9f9d0e1afabe9ef806519a1e92d88dc14f21f2d3"),
    ("cases[1].mutation_effects", "84a1b7404b05fd67965d13c2fb5d5aa7fb355640fe3b9b23b47ef8bcb199566d"),
    ("cases[1].mutation_outcome_summary", "4781d15335152c539c85b71d7224436f81c6fb31e0a0ab83cab8a82d72989a04"),
    ("cases[1].mutation_supply", "c2975de48e89f8075274c2923835fda2bcea2b905b1ff394042f042fd0859643"),
    ("cases[1].mutation_value_totals_by_birth_edge_class", "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"),
    ("cases[1].mutation_value_totals_by_operator", "c5b50eca021d03fde687265b46b094a440a04f2eba28d0dbf0810fbb8e2d1f2c"),
    ("cases[1].mutational_neighborhood", "e18c75f737d2df08d477181a920033cc32805181e4af2438e9934ff385406300"),
    ("cases[1].neighborhood_read", "d9a259a30cc06c9f9999d8902bb3a6af23c6909c85ef29c13f1a908bbb4242f3"),
    ("cases[1].predation", "9ecdde608453a8db80d2a45463c02cf31564f7300761ad47972d8b9aa8f56236"),
    ("cases[1].reachable_structure_size_distribution", "70607023976efd385877a09ece24f8eb3a4260f0893d2c3d64a6ad2a44ec68c7"),
    ("cases[1].reproductive_success_by_cognitive_class", "94bcbe40c4b8a94a4a6e21e9f25dc76a7aa16e9e0ecf5054a3b26732d2dcb3d7"),
    ("cases[1].ring_rules", "98afe5d02c95718be027fe1f3ca1edf55b997cf1ffaf6043544f86ceb021f406"),
    ("cases[1].surviving_clade_profiles", "56e7144b88fea53604c3d6c902749c626de8c0cd90f885c88f7454eb2fdc9310"),
    ("cases[1].typed_eat_share", "33ac4af3c426fa84e02f7af93d5ba56c214176cb2bfe8c16b14eed4f2b1e5c40"),
    ("cases[1].typed_eats_failed_total", "d0bca111f8628137adc4c16f123496dcdd1d590d06cb5d9acd68b39fe656fb97"),
    ("cases[1].typed_eats_total", "671177a5690bac5a73d2f797c309f62ad1902acedcfc20b9596908a6be425448"),
    ("cases[2].action_attempts", "c859a036af3d8d2647df69d28e8f76f44bca9cdfbab68842f0941bbbbde991c8"),
    ("cases[2].avoidable_blocked_share_of_all_moves_by_reader_state", "139ea8b8ec7e09e15ef00ba4fa903954e1890f11f6fbd85641781ff22c5beb33"),
    ("cases[2].barrier_blocked_fraction_by_reader_state", "a4aadde8545cbc42efbd6783135f5e2f83b682b08173c1ec78356160ca102900"),
    ("cases[2].blocked_move_fraction", "5afdaa5c77632545c73d321c18e991089363d3ff6bd18155161183fd5026fee6"),
    ("cases[2].case", "16d502892d986604c5db579b98fd080b16847f7be6206a91800fd7e62822f655"),
    ("cases[2].cognition", "26ef007375df007b9355b3c5532e5eb8c06c75ce529d9d76b4b2a1b13f7272fe"),
    ("cases[2].drift_depth", "07c4bf1006c227ab9a95df5cfee9cecbebb6dd6f7afc4b6db38116b6197ef695"),
    ("cases[2].drives", "b96aa85ebdb08d1ff232f2b04b05d82e34d27a9dae2e95f7555c8fdd48c81d20"),
    ("cases[2].energy_flows", "ccf09fcb64248fe6cf36867846695f531182fbedbfd64868c3168acb577f821b"),
    ("cases[2].food_density_total", "d7229b70def6a230115c06d371d995141da0cb425fe58c3a00c8fb872c409d78"),
    ("cases[2].gestation", "ee64248c93187379f23ec9eafb6c0a730826139d72ac01e2369c75ee461998ab"),
    ("cases[2].giving", "dc9dfbd44a34a86675c8c6ebb80b4ddd82783838104680df96142b26cce80cea"),
    ("cases[2].grazed_cell_share", "212dd102e34247ddd4e7ac2b5d299d3ea206149942508adcc9a3d55a1a18f6fb"),
    ("cases[2].grazing_modifier_mean", "ccbd07f270b0314f618489f7ab82441975abaa07f71413f194a83f38afdd7a18"),
    ("cases[2].input_use", "a45093584a34f8be581ea138863153993e2cd0dee9ceac0d96f8758aceb1ee27"),
    ("cases[2].mesh_dispatches_energy_exhausted_total", "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9"),
    ("cases[2].mortality", "d98e2f85586acfa188e5d6a3d927b4321f82a5efd8e0660584651894710c1958"),
    ("cases[2].move_attempts_with_barrier_neighbor_by_reader_state", "b80db82691ebc1abfd2bd64e6ccc79baa4b49c53b8d46b93c3957af62903910e"),
    ("cases[2].moves_attempted_total", "bcac371b54f59945a14aa49e2e408e5d6e4dbc59387f5d8cfc6b015d40d5bb02"),
    ("cases[2].moves_blocked_avoidable_by_reader_state", "bacef8a673cdcce235ce39a4c8c18eac90a1795f146b3b39b120a65477677b7d"),
    ("cases[2].moves_blocked_barrier_total", "4a44dc15364204a80fe80e9039455cc1608281820fe2b24f1e5233ade6af1dd5"),
    ("cases[2].moves_blocked_barrier_with_barrier_neighbor_by_reader_state", "cb221266e9cbb382506581815345a584a1a088549cefef97897afe9ffb545f43"),
    ("cases[2].moves_blocked_total_by_cause", "fb1879d75a8fb64420c1f41660fc220b831851573d2a0e19b138cede060b8aa3"),
    ("cases[2].mutation_effects", "05e8570f6b09e5622f8520dae3a3d496b455f410c73c50c4542d1cadaff352de"),
    ("cases[2].mutation_outcome_summary", "77004255bac3f105fc8947cb2081d6642481a0a856327869e2bac88a1edcce33"),
    ("cases[2].mutation_supply", "4ca72eccd3c63dc01b7908312cf473acb3b6f0dc15aaace7351d2596570155fa"),
    ("cases[2].mutation_value_totals_by_birth_edge_class", "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"),
    ("cases[2].mutation_value_totals_by_operator", "2bb0b15bce99341e37e61e83512495eb7f5cd7d3fc143cfd24657a3d52906713"),
    ("cases[2].mutational_neighborhood", "a62748ff0d3fe31dfcff47018c5052b6be2ef9fc74db882aba30d0fdeb11fd58"),
    ("cases[2].neighborhood_read", "817c8374c3b72b36c39c3affc8a89e2a054fb74dc1c3a9991ae10d4c5f06d636"),
    ("cases[2].predation", "9ecdde608453a8db80d2a45463c02cf31564f7300761ad47972d8b9aa8f56236"),
    ("cases[2].reachable_structure_size_distribution", "c394dfde3a3ce06e1278648add7e1ce16786cb245ae612d7f9fbdc085dc34281"),
    ("cases[2].reproductive_success_by_cognitive_class", "1081bf8519ab0b56eb7795d313b7034ab6359c2938066c3bf2a7c7ce4885b80e"),
    ("cases[2].ring_rules", "c052a0b43f8273ec2609d85383d6c5f118636d2bb9b8e52a80a5b209d89fa210"),
    ("cases[2].surviving_clade_profiles", "5c3b57a90a3770cb0c75e6bf72e3913585d58e8a630b4b3262354dc004a0d93e"),
    ("cases[2].typed_eat_share", "02a93bda4e2e68b510052110c367661984400806d4bae4426470d0c0ca11ab52"),
    ("cases[2].typed_eats_failed_total", "3d5812abc84c11768aa73a732c85d75dbed439188f5bb3239e9b762ea31d9862"),
    ("cases[2].typed_eats_total", "a7f8f44fde291701ba103729bffa5552edd29f6c6ea19f03777ee7f1c4792494"),
    ("deterministic.graph_work_definition", "1d3f007229b10125cce1e6f6089410696bc4815f557e5dbcbf87fe6a8728f5f1"),
    ("deterministic.per_creature_tick", "e7315c39249bca619ca7680f5538ab4da8f4736348c1d20ee4db52950917846f"),
    ("deterministic.per_seed", "13f7186d2a0161d297a2f7d4ed7e1394c4e7f80e3783389b549b27499f0dc4c9"),
    ("deterministic.profile", "414b6bb569c444e7d0927b1ee1792046515884b3ddccd3cc4d9937f9e04ff849"),
    ("deterministic.totals", "95e47609feb521a221d8d0e8daae14da1782ee5e8411c021d51367aca1d2db46"),
    ("goal_indicators.adaptive_novelty", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.births_per_100_ticks", "4fd32b888e0df64db3eba2a018feaa53b47d219df2183d8b4a5d883a34eeff38"),
    ("goal_indicators.drift_depth", "45f46232a4e3bdaa55bc9e054a23c8cbb557da7fc7539203616c24aec11ed67f"),
    ("goal_indicators.evolutionary_activity", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.information_integration", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.learning_dependence", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.lineage_diversity", "49d4a8856b6b0830c9f7adefdc1e4c447840837f192b2cbc83ba6a94c29c9739"),
    ("goal_indicators.memory_dependence", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.memory_sensitivity", "5fac68778fbb405f92c316bcf03e1179b6f65a77d3f6b3d8ad5c57409f4f61dd"),
    ("goal_indicators.mutational_neighborhood", "45f46232a4e3bdaa55bc9e054a23c8cbb557da7fc7539203616c24aec11ed67f"),
    ("goal_indicators.prediction_dependence", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.reachable_structure_size_distribution", "7f7f9060089620f546ec93b30f428c26dfe9d709831ed11501fc2affdd97fca9"),
    ("goal_indicators.reciprocal_interaction", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.strategy_causal_distinctness", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.strategy_count", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.structural_companions", "257f3f3cef206310892c61c7b1b4a63dfcb88fe7a72d92e8d2202b05345ba34e"),
    ("goal_indicators.temporal_memory_sensitivity", "b94415aa7921ed1ca8ac6abefee5c268865c7f098cc79357ba6eee486408557a"),
    ("persistence[0]", "3cb10b512ee5909d37b53eaa565efcdabb949948c4886cba0998da05ed3e4a0e"),
    ("persistence[0].last.births_total", "b17ef6d19c7a5b1ee83b907c595526dcb1eb06db8227d650d5dda0a9f4ce8cd9"),
    ("persistence[0].last.body", "d17a92fd5767050afced388fe0d7aaf140171300c48affc1d4c1d5798f691a70"),
    ("persistence[0].last.bundles", "e584d1258140722104bec8c7336080239358b010a5e98c48fccd586dc86903bd"),
    ("persistence[0].last.capacity", "b04ae863253ff4855acea323da3e5b32bca9377f11e0f0f038ddfadc34264b01"),
    ("persistence[0].last.food_density_total", "99796afa1c5db9317f4b692571d8fe286c444b887d851d2c907cb1aabb45e428"),
    ("persistence[0].last.grazed_cell_share", "bc6a862a0d0bccaf0ece69fe9f13b6a348323bc0de0eaf3664bd52e61d1176d7"),
    ("persistence[0].last.grazing_modifier_mean", "145a8c40ed8d59b7635e6dff9121854ab1c318a9c0f5845fa438073ce86b07e5"),
    ("persistence[0].last.mean_energy", "7ee1ba6278d4a4efb66576ce36a814bd5b0a00559e1860c73e778daaed73aa15"),
    ("persistence[0].last.mean_generation", "0e34727ba3b8630f7300bdf0781aa1564de290978672a85f910633273835e0cf"),
    ("persistence[0].last.mean_genome_size", "153bfb57263b3955ec425bfbbc4a5789e1dd405637ef048f0f2dc9c8e54dba5f"),
    ("persistence[0].last.mean_mesh_nodes", "bd4e5c1c5783b10f259719fd0a2ecfbd524ff5adb476edfaa0462a2e6b89b03c"),
    ("persistence[0].last.mean_wired_sinks", "a8bb9d6287749f5e6e24dc43fc4fa92bc4ec1ee7efa2ad4dab7cf62b760442e2"),
    ("persistence[0].last.move_attempts_with_barrier_neighbor_by_reader_state", "08ff183bb6c41c4cc56ea9399ae9bd537ffa5a39cdd71f6b4055977f1fb73820"),
    ("persistence[0].last.moves_attempted_total", "01a0123885ebec5b37b52ddc058c20d052525c654b69e7b7bfd5feb291428bba"),
    ("persistence[0].last.moves_blocked_avoidable_by_reader_state", "300c640e837a0ba452bd3420749f41ed838fb18d32887ab942e4c75e1963568a"),
    ("persistence[0].last.moves_blocked_barrier_total", "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9"),
    ("persistence[0].last.moves_blocked_barrier_with_barrier_neighbor_by_reader_state", "08ff183bb6c41c4cc56ea9399ae9bd537ffa5a39cdd71f6b4055977f1fb73820"),
    ("persistence[0].last.moves_blocked_total_by_cause", "712e92e2114e9d4668c00006ee0ef5cef2743689a369afae70c923b01ffae622"),
    ("persistence[0].last.occupancy_grid", "8769b44174ff143f300b5f832c98ad4b5b002fc336972aaff45e9b68646fdf0a"),
    ("persistence[0].last.plasticity_census", "bb708e25d3ae58675e8da1e71e7ba09b0251bfeb25859434a42d4e20aa8b9cb6"),
    ("persistence[0].last.population", "86e50149658661312a9e0b35558d84f6c6d3da797f552a9657fe0558ca40cdef"),
    ("persistence[0].last.sensor_census", "de9b14f8e582841c3710c37ba7a138f584d07d9f8fe248b545b0782c2ca9a338"),
    ("persistence[0].last.shannon_entropy_nats", "e641ce415e2cf7fd37821b027573fdf9c2966ed1b00cdfdce35355faf480e64e"),
    ("persistence[0].last.shares", "564749f34eaaa595a9525e14d21c5fc99796a27311e884ab9a7bfe1dc70ea8a1"),
    ("persistence[0].last.surviving_founder_clade_count", "c2356069e9d1e79ca924378153cfbbfb4d4416b1f99d41a2940bfdb66c5319db"),
    ("persistence[0].last.tick", "811786ad1ae74adfdd20dd0372abaaebc6246e343aebd01da0bfc4c02bf0106c"),
    ("persistence[0].last.typed_eats_total", "8ada8db46d26aa873c1f1e74568b2b21a952c0cbefd7a2d7e66be05bcd0d9870"),
    ("persistence[0].samples", "301154d008d3a12b9766c58fc9ae4a574d1c4f373712eedf19d597c726f6b454"),
    ("persistence[1]", "9431c6a8e8aa252e64789008b94cbcff773df4d56ded397c52a47a9ebd321aa5"),
    ("persistence[1].last.births_total", "c6f3ac57944a531490cd39902d0f777715fd005efac9a30622d5f5205e7f6894"),
    ("persistence[1].last.body", "ff8ad8ea0a956917025294db4f0a18177e1d6aa14eda61401ca07406f6c06706"),
    ("persistence[1].last.bundles", "0df6de03fa7cd0b1913b9918b060b70b04909110376f68ca50a57ddbf61a43b9"),
    ("persistence[1].last.capacity", "15c19ddd6cf1c6c11343f6db4ffb095057e9cd5771d2bae8e77da8af0d48348f"),
    ("persistence[1].last.food_density_total", "5c837b66c450ac926072f5da399e5da36bf45eaedd873abac66e36fecaa47d06"),
    ("persistence[1].last.grazed_cell_share", "897dd3ece58ca347fc551be0041fa74e860fdfd6101ec4ea0816c8ddb14c1730"),
    ("persistence[1].last.grazing_modifier_mean", "2c28acc625ae31319dcb4b982d134abeb107a11e1e919ee5fe7b845abc9e3f26"),
    ("persistence[1].last.mean_energy", "21d0cb5b45d4cba7f98f5dc6f1b7fd913da72bbdda840766e34e54789a54394e"),
    ("persistence[1].last.mean_generation", "cea02d34139112f6d3151231520c035f6b510baf315b905f08ffb6e4452f1316"),
    ("persistence[1].last.mean_genome_size", "00cc86f76eec642ae31cf4839dbcf7b53612a7b8ff5ac4700555b6508ef78795"),
    ("persistence[1].last.mean_mesh_nodes", "e0b67e603c53cc5ac979079d856781a08cf3007290e4954fc5fc30ff3e1a7139"),
    ("persistence[1].last.mean_wired_sinks", "92ebe2dc13166cbd8857ba04c75e6851dd845f1d7ca7456751d87963c4bc9f1e"),
    ("persistence[1].last.move_attempts_with_barrier_neighbor_by_reader_state", "28ba9fda17d5410f8cc65053861a28daf9d82e236f0b61808d396e38a7c28a4f"),
    ("persistence[1].last.moves_attempted_total", "bc83cbb2d6dcba934deedb695609dd3ae689a72b210a8f1b86f6b1bc4c68d348"),
    ("persistence[1].last.moves_blocked_avoidable_by_reader_state", "e80f34313a2cff6cdd8f123b3c519a45c0b3848a4227ea304aef497a1a032c4d"),
    ("persistence[1].last.moves_blocked_barrier_total", "ad48ff99415b2f007dc35b7eb553fd1eb35ebfa2f2f308acd9488eeb86f71fa8"),
    ("persistence[1].last.moves_blocked_barrier_with_barrier_neighbor_by_reader_state", "3538e8a83831713e0d46b0059d45398d5801f40be0be5e7c87a1e07517809367"),
    ("persistence[1].last.moves_blocked_total_by_cause", "de63eb16107911b99047dd6f9f9d0e1afabe9ef806519a1e92d88dc14f21f2d3"),
    ("persistence[1].last.occupancy_grid", "5cae231eaa5bb907ca378dfa03a119a417b99cc1e0c27b9d42570789b58ef523"),
    ("persistence[1].last.plasticity_census", "4995ef9f67e6511d5dc1997a6146173ec59dd4b158ed859a11a068322bda3f7c"),
    ("persistence[1].last.population", "e29c9c180c6279b0b02abd6a1801c7c04082cf486ec027aa13515e4f3884bb6b"),
    ("persistence[1].last.sensor_census", "26d94602adf761bc83e2a26156a06fd7efe70950b4c4d17d6587d4997ef6b30c"),
    ("persistence[1].last.shannon_entropy_nats", "6735774da625c6a6639f5d1ae9423d8409b91de517cd4b50c779b929ed24d3c5"),
    ("persistence[1].last.shares", "23339b5f5fa4d49ad84230a1743849d8734f9d7c1d914e4dc9a2896cf8ed4a73"),
    ("persistence[1].last.surviving_founder_clade_count", "785f3ec7eb32f30b90cd0fcf3657d388b5ff4297f2f9716ff66e9b69c05ddd09"),
    ("persistence[1].last.tick", "811786ad1ae74adfdd20dd0372abaaebc6246e343aebd01da0bfc4c02bf0106c"),
    ("persistence[1].last.typed_eats_total", "671177a5690bac5a73d2f797c309f62ad1902acedcfc20b9596908a6be425448"),
    ("persistence[1].samples", "d472f1547a32a735213985d42a5c7e799a84812ef27a2a61686883201c9a9c67"),
    ("persistence[2]", "98c8fdb3bd22dd03ff9a05d5bbfe065a8642b28148ba3607d6d3b8c23f705352"),
    ("persistence[2].last.births_total", "44cb730c420480a0477b505ae68af508fb90f96cf0ec54c6ad16949dd427f13a"),
    ("persistence[2].last.body", "8e425a82accc9ea8f9b91e8b64cd09365bdef7d073041d311c5371e4b060f705"),
    ("persistence[2].last.bundles", "ce96d7029f0c5596d4f072f2e135b0b449cffb8071a04967829538b49b439b72"),
    ("persistence[2].last.capacity", "fb8fd80cfed8659b53a67cff6d04b219ab55657c83a493c98d604db869bf106f"),
    ("persistence[2].last.food_density_total", "d7229b70def6a230115c06d371d995141da0cb425fe58c3a00c8fb872c409d78"),
    ("persistence[2].last.grazed_cell_share", "212dd102e34247ddd4e7ac2b5d299d3ea206149942508adcc9a3d55a1a18f6fb"),
    ("persistence[2].last.grazing_modifier_mean", "ccbd07f270b0314f618489f7ab82441975abaa07f71413f194a83f38afdd7a18"),
    ("persistence[2].last.mean_energy", "18c00b8f515407aab63a1602735bc177d82302fe532c60072bdf36b73d2d969d"),
    ("persistence[2].last.mean_generation", "2f444b64abd4ee243acafd863480a20c555e98253925fcd49d0e8a3c1415e1ef"),
    ("persistence[2].last.mean_genome_size", "c55110c1e68c2aee669378218421d772494c4bc2ed3b06b6a077286afb6882b3"),
    ("persistence[2].last.mean_mesh_nodes", "45b767271c7afe402b235e5cfd8f7dc90c72d63de25ff4d34d59aa1597bf7843"),
    ("persistence[2].last.mean_wired_sinks", "ad29f56554112799e934a1d730551fb2d02446fe30b78906c19f72606e6ff31b"),
    ("persistence[2].last.move_attempts_with_barrier_neighbor_by_reader_state", "b80db82691ebc1abfd2bd64e6ccc79baa4b49c53b8d46b93c3957af62903910e"),
    ("persistence[2].last.moves_attempted_total", "bcac371b54f59945a14aa49e2e408e5d6e4dbc59387f5d8cfc6b015d40d5bb02"),
    ("persistence[2].last.moves_blocked_avoidable_by_reader_state", "bacef8a673cdcce235ce39a4c8c18eac90a1795f146b3b39b120a65477677b7d"),
    ("persistence[2].last.moves_blocked_barrier_total", "4a44dc15364204a80fe80e9039455cc1608281820fe2b24f1e5233ade6af1dd5"),
    ("persistence[2].last.moves_blocked_barrier_with_barrier_neighbor_by_reader_state", "cb221266e9cbb382506581815345a584a1a088549cefef97897afe9ffb545f43"),
    ("persistence[2].last.moves_blocked_total_by_cause", "fb1879d75a8fb64420c1f41660fc220b831851573d2a0e19b138cede060b8aa3"),
    ("persistence[2].last.occupancy_grid", "96009016c0f28f4098500eb92b8df636075e690c15698c74c58aefcc2581381d"),
    ("persistence[2].last.plasticity_census", "bc8c1df4f83ad3339dce6de820b5aed03447d832fb973c0e7a71b28611fa19d9"),
    ("persistence[2].last.population", "25fc0e7096fc653718202dc30b0c580b8ab87eac11a700cba03a7c021bc35b0c"),
    ("persistence[2].last.sensor_census", "d3e965d38cb0c919318e1a61b5de7df6741f90fbaff9c11f641955e0534f4e23"),
    ("persistence[2].last.shannon_entropy_nats", "eaf2b4b4f3e748007b6cd59228e5e7250d5c650c9e5e8e53ea3b5e294527d71f"),
    ("persistence[2].last.shares", "564749f34eaaa595a9525e14d21c5fc99796a27311e884ab9a7bfe1dc70ea8a1"),
    ("persistence[2].last.surviving_founder_clade_count", "785f3ec7eb32f30b90cd0fcf3657d388b5ff4297f2f9716ff66e9b69c05ddd09"),
    ("persistence[2].last.tick", "811786ad1ae74adfdd20dd0372abaaebc6246e343aebd01da0bfc4c02bf0106c"),
    ("persistence[2].last.typed_eats_total", "a7f8f44fde291701ba103729bffa5552edd29f6c6ea19f03777ee7f1c4792494"),
    ("persistence[2].samples", "a33ba16f7d1f1d430bf38bb13b09c1d6e308aea216256866c6bb1c6950050c8e"),
];

// ── Crafted population: bundles, plastic edges, pending births ──────────────

use v3_core::config::SimulationConfig;
use v3_core::contracts::votes::{DirectedVoteKind, VoteSink};
use v3_core::contracts::{Direction, InputReference, WorldInputKey};
use v3_core::creature::state::PendingBirth;
use v3_core::genome::cgp::{BundleBirth, BundleSource, EdgeOccurrence, EdgeSurface, Wiring};
use v3_core::genome::{BackendDef, CreatureGenome, HebbianRule, Learning};
use v3_core::simulation::Simulation;

fn crafted_config() -> SimulationConfig {
    let mut config = SimulationConfig::default();
    config.world.width = 32;
    config.world.height = 32;
    config.population.initial_creatures = 12;
    config
}

/// Founder `index` edited: a `Ring` and a `Plain` bundle (0, 3, 6, 9), a
/// `DirectionPair` and a `Plain` bundle (1, 4, 7, 10), plastic sink edges
/// (2, 5, 8, 11), each on the founder's Graph node.
fn craft(index: usize, genome: &mut CreatureGenome, config: &SimulationConfig) {
    let context = config.sensor_context(config.world.food.types.len());
    let node = genome
        .nodes
        .iter_mut()
        .find(|node| matches!(node.backend_def(), BackendDef::Graph(_)))
        .expect("the founder carries a Graph node");
    let scale = 0.25 * (index as f32 + 1.0) * if index.is_multiple_of(2) { 1.0 } else { -1.0 };
    match index % 3 {
        0 => {
            node.add_sensor(
                InputReference::World(WorldInputKey::NeighborBarrierRing),
                &[BundleBirth {
                    field_set: 0,
                    wiring: Wiring::Ring {
                        kind: DirectedVoteKind::Move,
                        offset: (index % 8) as u8,
                    },
                    scale,
                }],
                &context,
            )
            .expect("a ring sensor");
            node.add_bundle(
                BundleSource::Memory { previous: false },
                BundleBirth {
                    field_set: 0,
                    wiring: Wiring::Plain {
                        sink: VoteSink::Eat,
                    },
                    scale: if index == 0 { 0.0 } else { scale },
                },
                &context,
            )
            .expect("a memory bundle");
        }
        1 => {
            let area = InputReference::World(WorldInputKey::AreaBarrierSummary);
            let BackendDef::Graph(_def) = node.backend_def() else {
                unreachable!("a Graph node");
            };
            let births: Vec<BundleBirth> =
                v3_core::contracts::SensorDescriptor::of(&area, &context)
                    .field_sets()
                    .enumerate()
                    .map(|(field_set, set)| {
                        let plans = v3_core::genome::cgp::plans(&set);
                        BundleBirth {
                            field_set: field_set as u8,
                            wiring: plans[index % plans.len()],
                            scale,
                        }
                    })
                    .collect();
            node.add_sensor(area, &births, &context)
                .expect("an area summary sensor");
        }
        _ => {
            let BackendDef::Graph(def) = node.backend_edit().1 else {
                unreachable!("a Graph node");
            };
            for sink in 0..def.output_sinks().len() {
                let occurrence = EdgeOccurrence {
                    surface: EdgeSurface::SinkInput(sink),
                    index: 0,
                };
                if !def.output_sinks()[sink].inputs.is_empty() && def.can_learn(occurrence) {
                    def.edge_mut(occurrence).set_learning(Some(Learning {
                        rule: HebbianRule::Classic,
                        learning_rate: 0.1,
                        weight_clamp: 5.0,
                        lamarckian: sink % 2 == 0,
                        modulation: None,
                        retention: 1.0,
                    }));
                }
            }
        }
    }
}

/// Twelve crafted founders run eight ticks, then four creatures given a
/// pending litter of distinct energy.
fn crafted_simulation() -> Simulation {
    let config = crafted_config();
    let edit_config = config.clone();
    let mut sim = v3_core::simulation::seeding::seed_simulation_with_founder_genomes(
        config,
        13,
        |index, genome| craft(index, genome, &edit_config),
    );
    for _ in 0..8 {
        v3_core::simulation::run_tick(&mut sim, &mut None);
    }
    for (creature, litter_energy) in sim.creatures.values_mut().zip([1.5_f32, 2.25, 0.1, 7.0]) {
        creature.pending_birth = Some(PendingBirth {
            direction: Direction::N,
            litter_energy,
            deliver_tick: 99,
        });
    }
    sim
}

/// The sample a run records at `sim`'s tick, built the way the benchmark
/// run builds it.
fn sample(sim: &Simulation) -> Value {
    use super::tracking::{PersistenceAccumulator, SampleBlocks, WorldTracking};
    let population = sim.creatures.len() as u64;
    let mut accumulator = PersistenceAccumulator::new(sim.tick, population);
    accumulator.observe(
        sim.tick,
        population,
        sim.stats.reproduction_actions_spawned_total,
        || SampleBlocks::observe(sim),
        || WorldTracking::observe(sim),
    );
    serde_json::to_value(accumulator.finish(0)).expect("a persistence row serializes")
}

fn json(value: &impl serde::Serialize) -> Value {
    serde_json::to_value(value).expect("a pinned block serializes")
}

fn crafted_blocks(sim: &Simulation) -> BTreeMap<String, Value> {
    use super::indicators::{
        evolved_neighborhood_for_seed, generation_distribution, lineage_diversity,
        memory_sensitivity, neighborhood_read_for_seed, structural_companions_census,
        structure_size_distribution, temporal_memory_sensitivity,
    };
    use super::tracking::WorldTracking;
    use v3_core::neighborhood::{Battery, EvalContext};
    let mut blocks = BTreeMap::new();
    blocks.insert("sample".to_string(), sample(sim));
    let terminal = json(&WorldTracking::observe(sim).with_transferred_counters(sim));
    for key in ["gestation", "surviving_clade_profiles"] {
        blocks.insert(key.to_string(), terminal[key].clone());
    }
    blocks.insert(
        "lineage_diversity".into(),
        json(&lineage_diversity(
            5,
            sim.creatures.values().map(|c| c.lineage_id),
        )),
    );
    blocks.insert(
        "structural_companions".into(),
        json(&structural_companions_census(
            5,
            sim.creatures.values().map(|c| c.genome()),
        )),
    );
    blocks.insert(
        "memory_sensitivity".into(),
        json(&memory_sensitivity(
            5,
            &v3_core::simulation::observe_final_actions(sim),
        )),
    );
    blocks.insert(
        "temporal_memory_sensitivity".into(),
        json(&temporal_memory_sensitivity(5, sim)),
    );
    blocks.insert(
        "generation_distribution".into(),
        json(&generation_distribution(
            sim.creatures.values().map(|c| c.generation).collect(),
        )),
    );
    blocks.insert(
        "structure_size_distribution".into(),
        json(&structure_size_distribution(
            sim.creatures
                .values()
                .map(|c| v3_core::genome::analysis::functional_complexity(c.genome()))
                .collect(),
        )),
    );
    let battery = Battery::generate(sim.config.world.food.types.len());
    let context = EvalContext::from_config(&sim.config);
    let sizes = pin_params().neighborhood;
    blocks.insert(
        "evolved_neighborhood".into(),
        json(&evolved_neighborhood_for_seed(
            5,
            sim,
            &battery,
            &sim.config.mutation,
            &context,
            sizes,
        )),
    );
    let (read, exposure) =
        neighborhood_read_for_seed(5, sim, &battery, &sim.config.mutation, &context, sizes);
    blocks.insert("neighborhood_read".into(), json(&read));
    blocks.insert(
        "read_exposure".into(),
        Value::String(format!("{exposure:?}")),
    );
    let cohort = super::mutation_effects::selected_cohort(
        5,
        sim,
        3,
        v3_core::neighborhood::mutation_effects::Sizes {
            parents: 2,
            ..Default::default()
        },
    );
    blocks.insert(
        "selected_cohort".into(),
        Value::String(format!("{cohort:?}")),
    );
    blocks.insert(
        "tick_sample".into(),
        json(&crate::build_tick_sample(sim, sim.tick)),
    );
    blocks
}

fn print_pins(tag: &str, blocks: &BTreeMap<String, Value>) {
    for (key, value) in blocks {
        println!("PIN {tag} (\"{key}\", \"{}\"),", digest(value));
    }
}

#[test]
fn every_crafted_population_census_keeps_its_pre_move_value() {
    let sim = crafted_simulation();
    let blocks = crafted_blocks(&sim);
    print_pins("CRAFTED", &blocks);
    assert_pins(&blocks, CRAFTED_PINS);
}

#[rustfmt::skip]
const CRAFTED_PINS: &[(&str, &str)] = &[
    ("evolved_neighborhood", "a61a21050b015db3b0d1e60c224b808aada7788ee6305908b27a62dace125470"),
    ("generation_distribution", "78759ebefd64bbe86da1a6b58e7a8b33839485ef7abbce24ad6b921e13ecb53b"),
    ("gestation", "5b3476658aae17b52c06058aafecc99b1a800b609d634211ab067762fa6fe523"),
    ("lineage_diversity", "87f75640c8fa7b1e176769020468016568dd14c5aaddd1c13a57584e60d0bbae"),
    ("memory_sensitivity", "eca8c579acc0cae4a9830ad20d9bb232e13d0d2528b8994799ce51499563544d"),
    ("neighborhood_read", "1ec097925a737285c73ccb870f56678536acf86689304c0135edd9894490313d"),
    ("read_exposure", "c3134fc57f207e4a382f6736fddef105a5b85690be28031af7b6e01b4b3c2e88"),
    ("sample", "eeba77bae2342e3921d279b3a9d7d8a8bf92e1e232736c3dea0b10ba62a4cb9d"),
    ("selected_cohort", "5fdc217408d0781af4780284e3774863db984309fcfcbc0ad7373e9ef952f04c"),
    ("structural_companions", "0d14ef19eab906849017dd376c209ed6449e708aadf1f2a009d28e7e225239e1"),
    ("structure_size_distribution", "955569fe046d3bd5d9dea73b80e1a52040a8732fc21c6d10a606aab35eed773b"),
    ("surviving_clade_profiles", "af9a43bfd059a8ff1e70026c2429c349626a3e2baf41c287ad293943527374ec"),
    ("temporal_memory_sensitivity", "3002c136b4696ac26c28b51528345f3dc580a3f477d83e4be3d64c72179e7bd4"),
    ("tick_sample", "736dce680f328a76784e1ccaedd95a870852abea52655ab2e8102f0010f0b4c0"),
];

/// The extinct population: an empty world's sample and tick sample.
#[test]
fn an_extinct_population_keeps_its_pre_move_values() {
    let mut config = crafted_config();
    config.population.initial_creatures = 0;
    let sim = v3_core::simulation::seed_simulation(config, 13);
    let mut blocks = BTreeMap::new();
    blocks.insert("sample".to_string(), sample(&sim));
    blocks.insert(
        "tick_sample".to_string(),
        json(&crate::build_tick_sample(&sim, sim.tick)),
    );
    print_pins("EXTINCT", &blocks);
    assert_pins(&blocks, EXTINCT_PINS);
}

#[rustfmt::skip]
const EXTINCT_PINS: &[(&str, &str)] = &[
    ("sample", "6412263d1b377733d324a13206ea2e64183fa4405aaba33e29e915d2bd6d3a0d"),
    ("tick_sample", "38da697af978ecb492e4d884143e9484552d7e65e4be3cd62851c2ea777c10a6"),
];

/// The NDJSON stream of a short run: `run_started`, every `tick_sample`
/// and `run_completed`, each line in sorted-key form.
#[test]
fn the_ndjson_stream_keeps_its_pre_move_values() {
    let mut config = crafted_config();
    config.population.initial_creatures = 24;
    let mut out = Vec::new();
    crate::run_simulation(config, 17, 30, 10, &mut out).expect("a run");
    let lines: Vec<Value> = String::from_utf8(out)
        .expect("UTF-8")
        .lines()
        .map(|line| serde_json::from_str(line).expect("a JSON line"))
        .collect();
    let mut blocks = BTreeMap::new();
    blocks.insert("stream".to_string(), Value::Array(lines));
    print_pins("NDJSON", &blocks);
    assert_pins(&blocks, NDJSON_PINS);
}

#[rustfmt::skip]
const NDJSON_PINS: &[(&str, &str)] = &[(
    "stream",
    "90667e9129fa63582e756be50f3b788defcc40232d8ec620daf7917b84396a20",
)];

/// The bundle census over chosen reachable sets: every node reachable, none
/// reachable, and every node but the founder Graph node (index 0).
#[test]
fn the_bundle_census_keeps_its_pre_move_value_on_any_reachable_set() {
    let sim = crafted_simulation();
    let mut blocks = BTreeMap::new();
    let all: Vec<Vec<usize>> = sim
        .creatures
        .values()
        .map(|c| (0..c.genome().nodes.len()).collect())
        .collect();
    for (name, reachable) in [
        ("all", all.clone()),
        ("none", all.iter().map(|_| Vec::new()).collect()),
        (
            "graph_unreachable",
            all.iter().map(|nodes| nodes[1..].to_vec()).collect(),
        ),
    ] {
        let census = super::bundles::Bundles::census(
            sim.creatures
                .values()
                .zip(&reachable)
                .map(|(c, reachable)| (c.genome(), reachable.as_slice(), None)),
            &sim.stats.bundles_created_total,
        );
        blocks.insert(name.to_string(), json(&census));
    }
    print_pins("BUNDLE", &blocks);
    assert_pins(&blocks, BUNDLE_PINS);
}

#[rustfmt::skip]
const BUNDLE_PINS: &[(&str, &str)] = &[
    ("all", "8b81054411d64c5564a17b7db560cf0a261c0f0b4f946b63ea181fb91098775d"),
    ("graph_unreachable", "cf8c81210e4b9e3e4716b5bea7e2c15074d763d54609946b8d60db7a74367b19"),
    ("none", "cf8c81210e4b9e3e4716b5bea7e2c15074d763d54609946b8d60db7a74367b19"),
];
