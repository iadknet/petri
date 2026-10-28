//! Census pins (T30.F13): every population census the benchmark report and
//! the NDJSON stream emit, pinned on fixture populations against the code
//! before the censuses moved into core's readers. A pin that moves is a
//! defect; only the call expressions below may change, never the literals.
//! T30.F15 re-pinned one literal, `NDJSON_PINS` `stream`: its `tick_sample`
//! and `run_completed` mean energies are now core's `f64` mean cast to `f32`.
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
    ("cases[0].action_attempts", "635e1752c286a0bed9d00a8f3e5a818811a23563ef4d02acad63419e3bee8b7b"),
    ("cases[0].avoidable_blocked_share_of_all_moves_by_reader_state", "2b1245d3dd21a54e5f1e18d53005b9964eb490bf16c6230d3abf68f21208fd89"),
    ("cases[0].barrier_blocked_fraction_by_reader_state", "36eaae82614253375a80d7b59562d8df5b9817d8b810c8d7dc734123d944f3c6"),
    ("cases[0].blocked_move_fraction", "134ec893e059d2380c05e49ed77ffad50690063fedb063b023c4f0026f89af68"),
    ("cases[0].case", "551d8a9dc1ef2f07c49a7dde09f50347cb00c391e779c1a60c5339665d6d2f08"),
    ("cases[0].cognition", "81dd181792f1209310f9223771ba20e2afb7893330e9d8e57d781ad317f0627a"),
    ("cases[0].drift_depth", "3d53ee47f884157dc25a7666109a754539707f96cb1b5ee373366315bba1bcb1"),
    ("cases[0].drives", "d9618c8233c15ef902a1a30051a4d5db87d9db1496a31ebc609cf00852ba24a3"),
    ("cases[0].energy_flows", "9ed7263c0b025291cba492598339b56183ba3ddcd8fcb2613e5f65dbffa0ddaa"),
    ("cases[0].food_density_total", "10d3cd81033d3373fdb0371d2443f2ecc4d772a5f1ca8336677662c1f39bfe9e"),
    ("cases[0].gestation", "0d8a23d70e7bacc081137908cfe80b461919aad2368c7b284a3cb087f0f0d745"),
    ("cases[0].giving", "dc9dfbd44a34a86675c8c6ebb80b4ddd82783838104680df96142b26cce80cea"),
    ("cases[0].grazed_cell_share", "f1b7f3965f68b8c11455252c2757c5976ec9bb972a8e85ba07f485d024a42ac4"),
    ("cases[0].grazing_modifier_mean", "27661b7407fa954c1425c1c4f8de4c4ff9fd357dafc9389eb3eb298c664c84ea"),
    ("cases[0].input_use", "3181c75e7c4edf370547b0742ff7f9fa2364cf2387c56f9574c303706d6e9d6d"),
    ("cases[0].mesh_dispatches_energy_exhausted_total", "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9"),
    ("cases[0].mortality", "67b6cb92d06d3a17e1f9f47ee7e8672f584f4a11e69c238342ce74338cb68cf7"),
    ("cases[0].move_attempts_with_barrier_neighbor_by_reader_state", "08ff183bb6c41c4cc56ea9399ae9bd537ffa5a39cdd71f6b4055977f1fb73820"),
    ("cases[0].moves_attempted_total", "c8c817e80216ce409c63a735acdd40fb7b023bcca6e87b6fc89973c447b6a858"),
    ("cases[0].moves_blocked_avoidable_by_reader_state", "c240f471433799244248df0f039ffc4d14ac60e3dad2bfaf261e33791a099ebf"),
    ("cases[0].moves_blocked_barrier_total", "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9"),
    ("cases[0].moves_blocked_barrier_with_barrier_neighbor_by_reader_state", "08ff183bb6c41c4cc56ea9399ae9bd537ffa5a39cdd71f6b4055977f1fb73820"),
    ("cases[0].moves_blocked_total_by_cause", "cda2abcd24922ef183e4cb40da4f3a7b9f8396f507a81ebc709a88fdfa5c7109"),
    ("cases[0].mutation_effects", "c984341912f52a1b48fa246ca5feccf91d1e5dbdb45df26777ea7b474912666d"),
    ("cases[0].mutation_outcome_summary", "b38742bcaec0a8011d1900f009f3e26a8797c07a7dead01cf375253f22fc20e1"),
    ("cases[0].mutation_supply", "a47723375870a253f3db4584694ca0aca16a4da55c1965d6ff1533f54797617a"),
    ("cases[0].mutation_value_totals_by_birth_edge_class", "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"),
    ("cases[0].mutation_value_totals_by_operator", "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"),
    ("cases[0].mutational_neighborhood", "71ec4f3298756b4d75c34ece31b158d3bf7aacc04f5280748306f661df02b938"),
    ("cases[0].neighborhood_read", "0e012deb5055c4d8d599012e26deb6e694eead3d3a6f65a240400bcf383dd771"),
    ("cases[0].predation", "9ecdde608453a8db80d2a45463c02cf31564f7300761ad47972d8b9aa8f56236"),
    ("cases[0].reachable_structure_size_distribution", "efafbf45ee2dbaa31d503a46782f7aa40c962dd3293fcb345c1fbe6182f35a7b"),
    ("cases[0].reproductive_success_by_cognitive_class", "3d6049994a146f498f133dfbd33f93f135314ad4ee908449205a790e6ee12c4b"),
    ("cases[0].ring_rules", "52d27d282a8f08ff926b753e7cd7c3388346860148e3d9762933e63c064ccdee"),
    ("cases[0].surviving_clade_profiles", "b5fd706ac6ec3143d0e6608077d2ce33e454e4f7cd9b7318451aa3d4cfed9bc2"),
    ("cases[0].typed_eat_share", "02a93bda4e2e68b510052110c367661984400806d4bae4426470d0c0ca11ab52"),
    ("cases[0].typed_eats_failed_total", "3d5812abc84c11768aa73a732c85d75dbed439188f5bb3239e9b762ea31d9862"),
    ("cases[0].typed_eats_total", "8ada8db46d26aa873c1f1e74568b2b21a952c0cbefd7a2d7e66be05bcd0d9870"),
    ("cases[1].action_attempts", "3673f762d61f41262f5a9a3364de7261bd46325089bd7a33a18f52fc95636b8a"),
    ("cases[1].avoidable_blocked_share_of_all_moves_by_reader_state", "08aaa155b445146d83a4a83fb4bfd7a471ed2fdc5455eb420377c0168d2db329"),
    ("cases[1].barrier_blocked_fraction_by_reader_state", "f3f38da137f9c6f464b977d6f21cf752ad7da89c28ddacc6164576dd950f129d"),
    ("cases[1].blocked_move_fraction", "d99d6e2f017c34a4427a6f90f7226aa17f21415ea17518d4f83d52f8bc77b1b7"),
    ("cases[1].case", "5dc71ed354ece796260f31671ca2b15d74bbbb84458969c7b02efcd09956025c"),
    ("cases[1].cognition", "81dd181792f1209310f9223771ba20e2afb7893330e9d8e57d781ad317f0627a"),
    ("cases[1].drift_depth", "52d8b993a957927c0787ec2dbe40282f149b20d8874963de0c6a9219a1d50464"),
    ("cases[1].drives", "a3242a3433097eb9f93d47fa3b82df78760286b2183169f447feb47db49f6c49"),
    ("cases[1].energy_flows", "75ca7fe4bd18e9a30d4b679bc295465400b34e58e436a5dfa79d362ea6a2de7b"),
    ("cases[1].food_density_total", "6fa2c842df8f3f41e56609e9dae9496a992017acad1f452709a6ae790767510e"),
    ("cases[1].gestation", "67c5ceba1e7a22c92bd9e07576a131edadd01fc65ffc905eb5cc00812247c8d8"),
    ("cases[1].giving", "dc9dfbd44a34a86675c8c6ebb80b4ddd82783838104680df96142b26cce80cea"),
    ("cases[1].grazed_cell_share", "e1db8af6bc2d8ec9131cc6b38977e1a1a88531f68c651533f0d39fa5ae81a890"),
    ("cases[1].grazing_modifier_mean", "277174bd1c1b2895a537b7a161209e18bb32e5489fe27d34d53f906120e262c3"),
    ("cases[1].input_use", "0729ab2bce167d5d1fd9c7d62bcadf7002f347b13c98456ee9ec8969f8bfe147"),
    ("cases[1].mesh_dispatches_energy_exhausted_total", "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9"),
    ("cases[1].mortality", "e27c333088f9a7ee88b724aad8529590dd54d466ef3867eab115aa2acd562910"),
    ("cases[1].move_attempts_with_barrier_neighbor_by_reader_state", "d39250c61d18af2b520d864fe5bb3b2d989719a16029a0cbd5e21dcc0b5a33cd"),
    ("cases[1].moves_attempted_total", "e2135c76ca7863e51154829de3d9cb0a8f6398d31518c7581d370ed9cc521811"),
    ("cases[1].moves_blocked_avoidable_by_reader_state", "8c336f68072ad7ead978ce20300784309fd3063b6e3ddc6a70e778bddd5fb914"),
    ("cases[1].moves_blocked_barrier_total", "6e4001871c0cf27c7634ef1dc478408f642410fd3a444e2a88e301f5c4a35a4d"),
    ("cases[1].moves_blocked_barrier_with_barrier_neighbor_by_reader_state", "ad6d9ac4322ee4d99da769dcaba21f2bfc274bfbc2d5a5e454ce0686c34f0568"),
    ("cases[1].moves_blocked_total_by_cause", "d25f40ad9b021cbf9b6d10730cc2fec38ed73b997105dfc1e49fb87275cdab12"),
    ("cases[1].mutation_effects", "041e53d5aee00246800127484bd3f76219545bebf46300a65fcb50b2725eba14"),
    ("cases[1].mutation_outcome_summary", "8b10e6bc43674b5105be960ee9f5a8570013e8422e4a56b1863435f9b3120f30"),
    ("cases[1].mutation_supply", "2db935083179588a43f437a06f0efaebf314fc54a1adf7515b521271747ce2ae"),
    ("cases[1].mutation_value_totals_by_birth_edge_class", "6b5b2b9bd61963b13fa81f696e3249d4702743c00669bd7faee1b1757fe17143"),
    ("cases[1].mutation_value_totals_by_operator", "dc2a4be23855cc1ba047be22747de25d24fbc21ac8268cc0f2f17a4488bf6ac2"),
    ("cases[1].mutational_neighborhood", "79e4f972ad4f0935e994e86a2d511aaeba707cf9cab0dd8c05bd8cd4fc09512d"),
    ("cases[1].neighborhood_read", "e7e6fa991e23636f10e0c0337dffe2a7f4a2fab2c11489b8253083d01c90ebf3"),
    ("cases[1].predation", "9ecdde608453a8db80d2a45463c02cf31564f7300761ad47972d8b9aa8f56236"),
    ("cases[1].reachable_structure_size_distribution", "5437585be912ba85ad5fb1e6b04de2c66c852ada738f22b5d86de4557c87dc55"),
    ("cases[1].reproductive_success_by_cognitive_class", "6e19795002193762b9cc2f04b95cc58639f59e2495078cd820b7487ecae448e6"),
    ("cases[1].ring_rules", "491c4c78ed594754f4521a1839ab7c56f07b050a1ce5e57e8cd16b12e9b2a0d8"),
    ("cases[1].surviving_clade_profiles", "5fea5dfc052f409e51d4411dd406807bd4c8ede281370bdcc3570961921b4222"),
    ("cases[1].typed_eat_share", "33ac4af3c426fa84e02f7af93d5ba56c214176cb2bfe8c16b14eed4f2b1e5c40"),
    ("cases[1].typed_eats_failed_total", "589ffd3acf522ae81192579f297589e93b0c41f748b224d1b4bb5c857a2f70cb"),
    ("cases[1].typed_eats_total", "ea989b31db7115d5ced6bcae78670d7824c1653b3f1a796f3497f4d100d8c54d"),
    ("cases[2].action_attempts", "4acab21ac3bc341110787e6f2116e5ab9618fc870c8553d596314fa9442d317a"),
    ("cases[2].avoidable_blocked_share_of_all_moves_by_reader_state", "93979a6ec6a238292172b7ab8cfa0753d79fccce880210ea75701b729283dbc2"),
    ("cases[2].barrier_blocked_fraction_by_reader_state", "2d1fade2e5b0f3093127bd147aa3ca8fe93f654395335dc0dffdd2ca26c47a70"),
    ("cases[2].blocked_move_fraction", "d71fd1b54f9e3f0c341d2dc2b68392d871278f29f07e9ace25909ba27a55348d"),
    ("cases[2].case", "01c2390c799558429629ff18d7d95422eadf24b5cae3bea19f80d1cfe042d4d4"),
    ("cases[2].cognition", "81dd181792f1209310f9223771ba20e2afb7893330e9d8e57d781ad317f0627a"),
    ("cases[2].drift_depth", "3d53ee47f884157dc25a7666109a754539707f96cb1b5ee373366315bba1bcb1"),
    ("cases[2].drives", "bb43b5d2da7872f7185337d5c5bb4d20a22a5450fb97e106804e2ed832e45717"),
    ("cases[2].energy_flows", "eb75c45b97116f9af156c282875f2a418578cec4f86139c0fcd519f478692e98"),
    ("cases[2].food_density_total", "b5473d40662be45d787684e9d4dafa1e8fa02cc17065b82142572f3f13e057c9"),
    ("cases[2].gestation", "6cb484b1698125399d3865cacdd057efa7fe626a4010c4cf48635479484bdf53"),
    ("cases[2].giving", "dc9dfbd44a34a86675c8c6ebb80b4ddd82783838104680df96142b26cce80cea"),
    ("cases[2].grazed_cell_share", "71cb7c662a413ace93277849a96593f537870c2dfb1c2c2f548b58c0a464fbe6"),
    ("cases[2].grazing_modifier_mean", "10f6a66263b9e7258be9e322dbba121b95d024ed8acf511b611fd906ead6b4c3"),
    ("cases[2].input_use", "011941e055e6ddbfa36fefeff376d657c0a18a04ded916b9d94bd7c0a5350367"),
    ("cases[2].mesh_dispatches_energy_exhausted_total", "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9"),
    ("cases[2].mortality", "61cfb12aae9529b788df5df4ce68874354e6c9b2b0d7ecb1592b7a735648fa70"),
    ("cases[2].move_attempts_with_barrier_neighbor_by_reader_state", "0ea321df39f3c33e97c385aeb7926b8009375c712eb4d14ff519139638fe74f7"),
    ("cases[2].moves_attempted_total", "09dc1bb2b25a72c6a5deecbd211750ba6f81b0bd809a2475eefcad2c11ab9091"),
    ("cases[2].moves_blocked_avoidable_by_reader_state", "9de773bd7f8d7a04761686d6094d566f06a837bc3722e0bb368e63b073a02b7e"),
    ("cases[2].moves_blocked_barrier_total", "4a44dc15364204a80fe80e9039455cc1608281820fe2b24f1e5233ade6af1dd5"),
    ("cases[2].moves_blocked_barrier_with_barrier_neighbor_by_reader_state", "cb221266e9cbb382506581815345a584a1a088549cefef97897afe9ffb545f43"),
    ("cases[2].moves_blocked_total_by_cause", "ca3ff3c142745d0756d1bf2ccd1c6b27d09d25a376fb6c19313f34da8302678b"),
    ("cases[2].mutation_effects", "4bb6f3546e2c777d2f7169d75526ff246e2a1fdbfda3cff8ebe10ab6d9f19a4a"),
    ("cases[2].mutation_outcome_summary", "4244e569891f81a291ff571268934ccb75c1b9ac0ad22d8c5296c6f43e6e5cac"),
    ("cases[2].mutation_supply", "a5ce9267e7f00c366032493a094f362fc8d91c0b2378c05a27d095d19ff3ab7b"),
    ("cases[2].mutation_value_totals_by_birth_edge_class", "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"),
    ("cases[2].mutation_value_totals_by_operator", "3cac20f189f1e4d6bb00749af3e80eb4383fae7a0c50117c1958df5fb9dd7acf"),
    ("cases[2].mutational_neighborhood", "a7a8de4f9d81fa956bcfab1400e56c654a55a43adea672a61cb37747e828d708"),
    ("cases[2].neighborhood_read", "eefdf968866e0099a710c64039833b3086bd4f320e652b976c08ace409ddb24f"),
    ("cases[2].predation", "9ecdde608453a8db80d2a45463c02cf31564f7300761ad47972d8b9aa8f56236"),
    ("cases[2].reachable_structure_size_distribution", "6def9f15fa5a21ad8cd3a1322a7e6fdbada9ca85dd4018629d2ad32f03a25d59"),
    ("cases[2].reproductive_success_by_cognitive_class", "57bf70b19a53e592a99130589e931ba8ff2249cfd0aa0da50514d500329d78a0"),
    ("cases[2].ring_rules", "e4af052855faf4a4be907a8c4783be8da5a8480f7fd4db84d6205b68eeede951"),
    ("cases[2].surviving_clade_profiles", "be325b33828750c90c3665a0ec3c2eda0858d4d7b72d8fcf6dbbfc8f0054cc37"),
    ("cases[2].typed_eat_share", "02a93bda4e2e68b510052110c367661984400806d4bae4426470d0c0ca11ab52"),
    ("cases[2].typed_eats_failed_total", "4c43cc6ce951844692ed18556701055b4756e0a753072c18e98ea10d92b526e4"),
    ("cases[2].typed_eats_total", "6e7d1d13383663a41fd3d780d610696b7cdd26dbfe8381f6452beea296651c7e"),
    ("deterministic.graph_work_definition", "1d3f007229b10125cce1e6f6089410696bc4815f557e5dbcbf87fe6a8728f5f1"),
    ("deterministic.per_creature_tick", "030c5aeef21cc7ab03ed144c7582ee53d6bda8645a6bc1f74d0dc6d5ed0601f0"),
    ("deterministic.per_seed", "9f2fdd05445714ce80f8cb5a0f0499eab36850d704d92898eb03209f54b4edb7"),
    ("deterministic.profile", "ba56df443f89948289c6042975a23c77a3f5330c2b20790b256ce0d3220be041"),
    ("deterministic.totals", "3d598ae38e61097cf2f6444fb334455250186f8f0cdc8f569c1524f6c5e73be8"),
    ("goal_indicators.adaptive_novelty", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.births_per_100_ticks", "67fada8b3bd5dcc6899ef8795c5d1a83f864e5e79b2d6577c61c693afca5ed4e"),
    ("goal_indicators.drift_depth", "45f46232a4e3bdaa55bc9e054a23c8cbb557da7fc7539203616c24aec11ed67f"),
    ("goal_indicators.evolutionary_activity", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.information_integration", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.learning_dependence", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.lineage_diversity", "8cfd8b39ca6a7a5640669a36f7adf07cd114a9840c6eb8ac6c9cadd1ad323b10"),
    ("goal_indicators.memory_dependence", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.memory_sensitivity", "140a477c1e24e31ca649d354dc5424bf08686d52b8dd79531fd83a0bb332721c"),
    ("goal_indicators.mutational_neighborhood", "45f46232a4e3bdaa55bc9e054a23c8cbb557da7fc7539203616c24aec11ed67f"),
    ("goal_indicators.prediction_dependence", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.reachable_structure_size_distribution", "39bf5dfbdd4fcf80d90d3b487b6a1a2a1574b9b930d09f9b733e97a4d62b49cd"),
    ("goal_indicators.reciprocal_interaction", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.strategy_causal_distinctness", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.strategy_count", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.structural_companions", "b22ec3dd616a156ffc0d0651fac371a995a628cc88c3806bb5a77ce3c47d41f3"),
    ("goal_indicators.temporal_memory_sensitivity", "08119cc39cf4ca4d9aed514ed65979e1b4bb9cf617ca59eac00d305f4351ef32"),
    ("persistence[0]", "5cc4903d121b0676e5c8645e96d0bd6964e7f466c2b87a74bcb3e80638f80bb0"),
    ("persistence[0].last.births_total", "4523540f1504cd17100c4835e85b7eefd49911580f8efff0599a8f283be6b9e3"),
    ("persistence[0].last.bundles", "41c65dd45e13492c21eabb8a357ff4afe21efeee2a6d0012a0cc4214bcd44e52"),
    ("persistence[0].last.capacity", "47d41050ca33122ec420879a4d9a5d868c198f096197871a7b093c9dea9cc3ff"),
    ("persistence[0].last.food_density_total", "10d3cd81033d3373fdb0371d2443f2ecc4d772a5f1ca8336677662c1f39bfe9e"),
    ("persistence[0].last.grazed_cell_share", "f1b7f3965f68b8c11455252c2757c5976ec9bb972a8e85ba07f485d024a42ac4"),
    ("persistence[0].last.grazing_modifier_mean", "27661b7407fa954c1425c1c4f8de4c4ff9fd357dafc9389eb3eb298c664c84ea"),
    ("persistence[0].last.mean_energy", "07d971668588bc1f3c35c27a1a1360ebc4cbff8704c0f1049f4474e66d433669"),
    ("persistence[0].last.mean_generation", "7fdce9be89e92b69f7aaa15446afc951b3330ae9e1af233010178cf8aeb55843"),
    ("persistence[0].last.mean_genome_size", "b9a57dc95b565a6cb50d9cd23c75e1ea1f6b4f012478c3dc1a40d2a3f3b6ec8a"),
    ("persistence[0].last.mean_mesh_nodes", "559e9bf2014edb1cc0fb3286be93001fc53f5ce06d1deb2f6e79153215b139d8"),
    ("persistence[0].last.mean_wired_sinks", "33e8d664b4fbc8daebed2a05aaaec19ebe2d576b4182ed2a54de309135f04b7c"),
    ("persistence[0].last.move_attempts_with_barrier_neighbor_by_reader_state", "08ff183bb6c41c4cc56ea9399ae9bd537ffa5a39cdd71f6b4055977f1fb73820"),
    ("persistence[0].last.moves_attempted_total", "c8c817e80216ce409c63a735acdd40fb7b023bcca6e87b6fc89973c447b6a858"),
    ("persistence[0].last.moves_blocked_avoidable_by_reader_state", "c240f471433799244248df0f039ffc4d14ac60e3dad2bfaf261e33791a099ebf"),
    ("persistence[0].last.moves_blocked_barrier_total", "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9"),
    ("persistence[0].last.moves_blocked_barrier_with_barrier_neighbor_by_reader_state", "08ff183bb6c41c4cc56ea9399ae9bd537ffa5a39cdd71f6b4055977f1fb73820"),
    ("persistence[0].last.moves_blocked_total_by_cause", "cda2abcd24922ef183e4cb40da4f3a7b9f8396f507a81ebc709a88fdfa5c7109"),
    ("persistence[0].last.occupancy_grid", "c7aa9f9daad3bd3d9176a22898de05ee56fe5424eb0826bda74dc3d66de9200b"),
    ("persistence[0].last.plasticity_census", "5b2b65eeaa0a9564fcdf51398596740245ed9e17ee3ddb4436458dd644271799"),
    ("persistence[0].last.population", "9f14025af0065b30e47e23ebb3b491d39ae8ed17d33739e5ff3827ffb3634953"),
    ("persistence[0].last.sensor_census", "e0feb67e62f8bb22ed7077f0a3fea91a99ebe07c51229189dc904f4f43bfa14c"),
    ("persistence[0].last.shannon_entropy_nats", "2632fc5321b23d864ae601b23e49f4ee48cf660aa36fe7469cd32efaa7dc695d"),
    ("persistence[0].last.shares", "791b5fe7847ce66254f448acbb8b35eebe0b8ad3b5516aa6a7d5f23cf820dc78"),
    ("persistence[0].last.surviving_founder_clade_count", "c2356069e9d1e79ca924378153cfbbfb4d4416b1f99d41a2940bfdb66c5319db"),
    ("persistence[0].last.tick", "811786ad1ae74adfdd20dd0372abaaebc6246e343aebd01da0bfc4c02bf0106c"),
    ("persistence[0].last.typed_eats_total", "8ada8db46d26aa873c1f1e74568b2b21a952c0cbefd7a2d7e66be05bcd0d9870"),
    ("persistence[0].samples", "7b118506611eb2489180075ed8cd4cdc70a2137a0aa7334b4b84e39d25963dfc"),
    ("persistence[1]", "f6a28b5038a37f4bd9c5e8056d6fe23e5c2a030a31a53105ebbc8d46e14d6f64"),
    ("persistence[1].last.births_total", "c6f3ac57944a531490cd39902d0f777715fd005efac9a30622d5f5205e7f6894"),
    ("persistence[1].last.bundles", "dd0a42bb6191e0c023cc7480ae4439a564c51441634d76f6f75df8ca0af8edee"),
    ("persistence[1].last.capacity", "ecc8e4ee31e446992598eeb47394e0d30f6b40dbeb019fa60e2518dcc213ad47"),
    ("persistence[1].last.food_density_total", "6fa2c842df8f3f41e56609e9dae9496a992017acad1f452709a6ae790767510e"),
    ("persistence[1].last.grazed_cell_share", "e1db8af6bc2d8ec9131cc6b38977e1a1a88531f68c651533f0d39fa5ae81a890"),
    ("persistence[1].last.grazing_modifier_mean", "277174bd1c1b2895a537b7a161209e18bb32e5489fe27d34d53f906120e262c3"),
    ("persistence[1].last.mean_energy", "a059cdbbe5378d28f6fa287ddfac05857a818ea0d410e568f55177354259d7b3"),
    ("persistence[1].last.mean_generation", "cb8d3a9446bd1d5ec286eda431ac15d81b338baf951f75357cce0c52761b3075"),
    ("persistence[1].last.mean_genome_size", "99c0b847ff16744cb05a70deb5746ca0a8b7cca0e934e9e0dc9fc3863c54db09"),
    ("persistence[1].last.mean_mesh_nodes", "c4ae07708d60cf0c0729f9cdb55069cb6e0bc5fe64cf50cdac03929a81b08c58"),
    ("persistence[1].last.mean_wired_sinks", "ca0e188e5017e5e0eaf242dcb625643cb0190301163d921bb8fc160c566d013d"),
    ("persistence[1].last.move_attempts_with_barrier_neighbor_by_reader_state", "d39250c61d18af2b520d864fe5bb3b2d989719a16029a0cbd5e21dcc0b5a33cd"),
    ("persistence[1].last.moves_attempted_total", "e2135c76ca7863e51154829de3d9cb0a8f6398d31518c7581d370ed9cc521811"),
    ("persistence[1].last.moves_blocked_avoidable_by_reader_state", "8c336f68072ad7ead978ce20300784309fd3063b6e3ddc6a70e778bddd5fb914"),
    ("persistence[1].last.moves_blocked_barrier_total", "6e4001871c0cf27c7634ef1dc478408f642410fd3a444e2a88e301f5c4a35a4d"),
    ("persistence[1].last.moves_blocked_barrier_with_barrier_neighbor_by_reader_state", "ad6d9ac4322ee4d99da769dcaba21f2bfc274bfbc2d5a5e454ce0686c34f0568"),
    ("persistence[1].last.moves_blocked_total_by_cause", "d25f40ad9b021cbf9b6d10730cc2fec38ed73b997105dfc1e49fb87275cdab12"),
    ("persistence[1].last.occupancy_grid", "4109679b04a33ebade3dd75ef0d5d78ef81eeb5ae26f339c4ed45d98a58e5d0a"),
    ("persistence[1].last.plasticity_census", "561534a6236c78808843cfa9a582c2ff5d4e42baafa5cb7ff1c3ff866489e9bf"),
    ("persistence[1].last.population", "35135aaa6cc23891b40cb3f378c53a17a1127210ce60e125ccf03efcfdaec458"),
    ("persistence[1].last.sensor_census", "366ab7b6a9bd117fa742c107f79925f0ce7c20371c1ad8627be361cf53973f03"),
    ("persistence[1].last.shannon_entropy_nats", "1311ba80cc10eb99b22043432cd684221a7af161029fde42b56cf30a27d136e3"),
    ("persistence[1].last.shares", "c99014e038372c78811bde4325558de1bfd75ded7cd7909c356460893517bf8f"),
    ("persistence[1].last.surviving_founder_clade_count", "6f4b6612125fb3a0daecd2799dfd6c9c299424fd920f9b308110a2c1fbd8f443"),
    ("persistence[1].last.tick", "811786ad1ae74adfdd20dd0372abaaebc6246e343aebd01da0bfc4c02bf0106c"),
    ("persistence[1].last.typed_eats_total", "ea989b31db7115d5ced6bcae78670d7824c1653b3f1a796f3497f4d100d8c54d"),
    ("persistence[1].samples", "6b377ccdfc35dd1430e59f5a760a33c0daa13adb9e9095b4437bbcc892193e33"),
    ("persistence[2]", "c2d6eb137454fb6712620b68eed5ea3df1f439c215a89c121f33ae2e2b3dd7a5"),
    ("persistence[2].last.births_total", "44cb730c420480a0477b505ae68af508fb90f96cf0ec54c6ad16949dd427f13a"),
    ("persistence[2].last.bundles", "69921216700143e0b92bbfab53d2cc5db5bc506958cae30dade4aac66e0dc750"),
    ("persistence[2].last.capacity", "5caf1ab8a935826ed2df7fc0f0c63ec0dbdab7335c5f305d52d079a4337f100c"),
    ("persistence[2].last.food_density_total", "b5473d40662be45d787684e9d4dafa1e8fa02cc17065b82142572f3f13e057c9"),
    ("persistence[2].last.grazed_cell_share", "71cb7c662a413ace93277849a96593f537870c2dfb1c2c2f548b58c0a464fbe6"),
    ("persistence[2].last.grazing_modifier_mean", "10f6a66263b9e7258be9e322dbba121b95d024ed8acf511b611fd906ead6b4c3"),
    ("persistence[2].last.mean_energy", "ecf4fbc5bdbc9ee2851a4dbaf0268bf3db94aecb64794147af4ee8afb750d3c5"),
    ("persistence[2].last.mean_generation", "b058f9476c6e428a6a1e0973ac65b54aae976e7c53093b315c46dea2d21a542f"),
    ("persistence[2].last.mean_genome_size", "372e28fc581ef807977d7004f5766121f68cd411469eb75f3201b8ef59c00533"),
    ("persistence[2].last.mean_mesh_nodes", "6b4027120a909c313e296a62e6066ff45fa019e58d315688e3718f3cafe6a762"),
    ("persistence[2].last.mean_wired_sinks", "5aba0398e83c880436407677135152815e502a4a44b96ffcbe7e259894c7a1f8"),
    ("persistence[2].last.move_attempts_with_barrier_neighbor_by_reader_state", "0ea321df39f3c33e97c385aeb7926b8009375c712eb4d14ff519139638fe74f7"),
    ("persistence[2].last.moves_attempted_total", "09dc1bb2b25a72c6a5deecbd211750ba6f81b0bd809a2475eefcad2c11ab9091"),
    ("persistence[2].last.moves_blocked_avoidable_by_reader_state", "9de773bd7f8d7a04761686d6094d566f06a837bc3722e0bb368e63b073a02b7e"),
    ("persistence[2].last.moves_blocked_barrier_total", "4a44dc15364204a80fe80e9039455cc1608281820fe2b24f1e5233ade6af1dd5"),
    ("persistence[2].last.moves_blocked_barrier_with_barrier_neighbor_by_reader_state", "cb221266e9cbb382506581815345a584a1a088549cefef97897afe9ffb545f43"),
    ("persistence[2].last.moves_blocked_total_by_cause", "ca3ff3c142745d0756d1bf2ccd1c6b27d09d25a376fb6c19313f34da8302678b"),
    ("persistence[2].last.occupancy_grid", "dcc7a1dc17512bb301fd4b32c33a73d6afecc27c524193d3fe7780d7e1ab0e63"),
    ("persistence[2].last.plasticity_census", "245c0cfe07db585845f08637cd423368daaab9f3709bd0b142e0d25082ee4f2c"),
    ("persistence[2].last.population", "73475cb40a568e8da8a045ced110137e159f890ac4da883b6b17dc651b3a8049"),
    ("persistence[2].last.sensor_census", "6ed45a4dd8ba75a59acaa79fae4eb85adba045ec7f8191af28e98ba1bfcc0118"),
    ("persistence[2].last.shannon_entropy_nats", "6c4b9ac785d64cfedceb22028e77935122fbb7ab1f6e3b725011b5d335e89722"),
    ("persistence[2].last.shares", "04b7631f60590a62732eb9d0bd31abc4db35366695e0d51ab236b077fc5f7032"),
    ("persistence[2].last.surviving_founder_clade_count", "785f3ec7eb32f30b90cd0fcf3657d388b5ff4297f2f9716ff66e9b69c05ddd09"),
    ("persistence[2].last.tick", "811786ad1ae74adfdd20dd0372abaaebc6246e343aebd01da0bfc4c02bf0106c"),
    ("persistence[2].last.typed_eats_total", "6e7d1d13383663a41fd3d780d610696b7cdd26dbfe8381f6452beea296651c7e"),
    ("persistence[2].samples", "eb435f1f3a2c60983596fe9ed6ea672a1fd6f5be7308d9e745f2da40bcbaae72"),
];

// ── Crafted population: bundles, plastic edges, pending births ──────────────

use v3_core::config::SimulationConfig;
use v3_core::contracts::votes::{DirectedVoteKind, VoteSink};
use v3_core::contracts::{Direction, InputReference, WorldInputKey};
use v3_core::creature::state::PendingBirth;
use v3_core::genome::cgp::{BundleBirth, BundleSource, EdgeOccurrence, EdgeSurface, Wiring};
use v3_core::genome::{BackendDef, CreatureGenome, HebbianRule, PlasticityConfig};
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
        .find(|node| matches!(node.backend_def, BackendDef::Graph(_)))
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
            let BackendDef::Graph(def) = &node.backend_def else {
                unreachable!("a Graph node");
            };
            let births: Vec<BundleBirth> =
                v3_core::contracts::SensorDescriptor::of(&area, &context)
                    .field_sets()
                    .enumerate()
                    .map(|(field_set, set)| {
                        let plans = v3_core::genome::cgp::plans(def, &set);
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
            let BackendDef::Graph(def) = &mut node.backend_def else {
                unreachable!("a Graph node");
            };
            for sink in 0..def.output_sinks().len() {
                let occurrence = EdgeOccurrence {
                    surface: EdgeSurface::SinkInput(sink),
                    index: 0,
                };
                if !def.output_sinks()[sink].inputs.is_empty() && def.can_learn(occurrence) {
                    def.edge_mut(occurrence)
                        .set_plasticity(Some(PlasticityConfig {
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
            sim.creatures.values().map(|c| c.body.identity.lineage_id),
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
    ("evolved_neighborhood", "38e3829b7d4d5530768ae64edb7cf67351cca69a84aa0980d8ee458ded1c744b"),
    ("generation_distribution", "78759ebefd64bbe86da1a6b58e7a8b33839485ef7abbce24ad6b921e13ecb53b"),
    ("gestation", "5b3476658aae17b52c06058aafecc99b1a800b609d634211ab067762fa6fe523"),
    ("lineage_diversity", "87f75640c8fa7b1e176769020468016568dd14c5aaddd1c13a57584e60d0bbae"),
    ("memory_sensitivity", "eca8c579acc0cae4a9830ad20d9bb232e13d0d2528b8994799ce51499563544d"),
    ("neighborhood_read", "8eeec1f6621498a607f2a138970879ad8cbba280fbc3089d383ca0ee13e19fb5"),
    ("read_exposure", "c3134fc57f207e4a382f6736fddef105a5b85690be28031af7b6e01b4b3c2e88"),
    ("sample", "9422588a4c61a1afcbd6f2a0754f26733ad148a6f38c25bcc6812bbd6fab74df"),
    ("selected_cohort", "3d282cebaca4c22d0e3c7af79171ceaf32a5c2efd6cccfb8f0642b12d0cc7f61"),
    ("structural_companions", "0d14ef19eab906849017dd376c209ed6449e708aadf1f2a009d28e7e225239e1"),
    ("structure_size_distribution", "955569fe046d3bd5d9dea73b80e1a52040a8732fc21c6d10a606aab35eed773b"),
    ("surviving_clade_profiles", "d1de6e01417b9611696ebebc37d1f562cd725d4420705a950d9e012ba9978dc5"),
    ("temporal_memory_sensitivity", "3002c136b4696ac26c28b51528345f3dc580a3f477d83e4be3d64c72179e7bd4"),
    ("tick_sample", "f1e1bfe92573e00caeef81386faf8a7c5a9c2ad2a74a25c0f6b335101316e607"),
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
    ("sample", "4a2170279e45be8cad3e75970b32c25b6d4f72767d2cf86be1cb9c4b2bd3081a"),
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
    "4c7ca23267416a82fbba43446c3ed4683d34c6af35e03170ce388bee307b86d3",
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
    ("all", "d752b8bc4c2d0290da1d86a5bbfe54efc8b671a91bde45afb76b6d6a7e6758e2"),
    ("graph_unreachable", "3268c796a8d9da0af126575f58d67949b4a20bfb97300f18bc4cac651f502f08"),
    ("none", "3268c796a8d9da0af126575f58d67949b4a20bfb97300f18bc4cac651f502f08"),
];
