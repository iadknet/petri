//! Census pins (T30.F13): every population census the benchmark report and
//! the NDJSON stream emit, pinned on fixture populations against the code
//! before the censuses moved into core's readers. A pin that moves is a
//! defect; only the call expressions below may change, never the literals,
//! except under a predeclared re-pin. T30.F15 re-pinned one literal,
//! `NDJSON_PINS` `stream`: its `tick_sample` and `run_completed` mean
//! energies are now core's `f64` mean cast to `f32`. T25.F12 commit 2 (the
//! one learning type, its catalog and birth draws, `bundles-v2`, the
//! census's source keys and carriers) re-pinned every moved table.
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
    ("cases[0].input_use", "65599ece7dd68d6862d54d469b7c2792ff0941363124e1957a0ae2af55655dc3"),
    ("cases[0].mesh_dispatches_energy_exhausted_total", "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9"),
    ("cases[0].mortality", "67b6cb92d06d3a17e1f9f47ee7e8672f584f4a11e69c238342ce74338cb68cf7"),
    ("cases[0].move_attempts_with_barrier_neighbor_by_reader_state", "08ff183bb6c41c4cc56ea9399ae9bd537ffa5a39cdd71f6b4055977f1fb73820"),
    ("cases[0].moves_attempted_total", "c8c817e80216ce409c63a735acdd40fb7b023bcca6e87b6fc89973c447b6a858"),
    ("cases[0].moves_blocked_avoidable_by_reader_state", "c240f471433799244248df0f039ffc4d14ac60e3dad2bfaf261e33791a099ebf"),
    ("cases[0].moves_blocked_barrier_total", "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9"),
    ("cases[0].moves_blocked_barrier_with_barrier_neighbor_by_reader_state", "08ff183bb6c41c4cc56ea9399ae9bd537ffa5a39cdd71f6b4055977f1fb73820"),
    ("cases[0].moves_blocked_total_by_cause", "cda2abcd24922ef183e4cb40da4f3a7b9f8396f507a81ebc709a88fdfa5c7109"),
    ("cases[0].mutation_effects", "997e8064619cf5d6c1c621d30d93a8fdc08d534aea7bc4575c81e41b021b1c0e"),
    ("cases[0].mutation_outcome_summary", "b38742bcaec0a8011d1900f009f3e26a8797c07a7dead01cf375253f22fc20e1"),
    ("cases[0].mutation_supply", "45fe185e0d01e2c124cbf90b949f9275f1838c3254e92790e4ea5f47ee451fb5"),
    ("cases[0].mutation_value_totals_by_birth_edge_class", "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"),
    ("cases[0].mutation_value_totals_by_operator", "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"),
    ("cases[0].mutational_neighborhood", "4fc6cefe11e29e3a941a1471c5ed1092abff07dd70b3d7181491a657c06ec333"),
    ("cases[0].neighborhood_read", "0e012deb5055c4d8d599012e26deb6e694eead3d3a6f65a240400bcf383dd771"),
    ("cases[0].predation", "9ecdde608453a8db80d2a45463c02cf31564f7300761ad47972d8b9aa8f56236"),
    ("cases[0].reachable_structure_size_distribution", "efafbf45ee2dbaa31d503a46782f7aa40c962dd3293fcb345c1fbe6182f35a7b"),
    ("cases[0].reproductive_success_by_cognitive_class", "3d6049994a146f498f133dfbd33f93f135314ad4ee908449205a790e6ee12c4b"),
    ("cases[0].ring_rules", "52d27d282a8f08ff926b753e7cd7c3388346860148e3d9762933e63c064ccdee"),
    ("cases[0].surviving_clade_profiles", "b5fd706ac6ec3143d0e6608077d2ce33e454e4f7cd9b7318451aa3d4cfed9bc2"),
    ("cases[0].typed_eat_share", "02a93bda4e2e68b510052110c367661984400806d4bae4426470d0c0ca11ab52"),
    ("cases[0].typed_eats_failed_total", "3d5812abc84c11768aa73a732c85d75dbed439188f5bb3239e9b762ea31d9862"),
    ("cases[0].typed_eats_total", "8ada8db46d26aa873c1f1e74568b2b21a952c0cbefd7a2d7e66be05bcd0d9870"),
    ("cases[1].action_attempts", "fef38fc5432bed8f76bef52cbccd8259b05af26a976979a7d4ea4d775c9800b4"),
    ("cases[1].avoidable_blocked_share_of_all_moves_by_reader_state", "51c22afb435e3904976903e640871f69d7725b553a181912dfe1fb4b5c55a237"),
    ("cases[1].barrier_blocked_fraction_by_reader_state", "eaa47ace8c19833360629f4b8942133d2e440b57f873e65480b2bcd9a82a45f5"),
    ("cases[1].blocked_move_fraction", "4e67617e5393b492ae92e91e4a37f6d152274201af8c4601fbdd3c341e31404b"),
    ("cases[1].case", "5dc71ed354ece796260f31671ca2b15d74bbbb84458969c7b02efcd09956025c"),
    ("cases[1].cognition", "52749c10ba08f49d11c2a77eca462a8a3a8ed105626ed62967b1a43fa510c1ac"),
    ("cases[1].drift_depth", "52d8b993a957927c0787ec2dbe40282f149b20d8874963de0c6a9219a1d50464"),
    ("cases[1].drives", "ba5603e6f3d3e6c8cde6ba628dc870f093048d21ab089a80c231d4c5a26e9d5f"),
    ("cases[1].energy_flows", "29a14a7250d41fed02df790eb35ead9ed3c07091df08563dc21137c38e5bf26c"),
    ("cases[1].food_density_total", "7244246cb5f028789589b1f93bc779be34e1c2c7418a8e5abd1f9ba5af78d45f"),
    ("cases[1].gestation", "6416dc0b9922bd9bbf3fbebc2d13b37741f29e97a03c9b953d7c5254388e58cd"),
    ("cases[1].giving", "dc9dfbd44a34a86675c8c6ebb80b4ddd82783838104680df96142b26cce80cea"),
    ("cases[1].grazed_cell_share", "b0047c03711300d119c19d129cc2dfb98168b199f81d147fafcac5a24ba71501"),
    ("cases[1].grazing_modifier_mean", "f09bd0e1d5823dfe40742396b0459bfe76c62327b074a6e01f07aa28769a4637"),
    ("cases[1].input_use", "5e412958dee1899ce8cd7dc4ff37232037d8f4d1aa575a861765725df0029162"),
    ("cases[1].mesh_dispatches_energy_exhausted_total", "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9"),
    ("cases[1].mortality", "c2ee5cfb9cdd0fe304bf0073586dba1d8881fc0f18711473ebaad5fee28880fe"),
    ("cases[1].move_attempts_with_barrier_neighbor_by_reader_state", "55655d4121ffede6287b3740da52dfd1c6fce6bfdaa7f1a1dcd40ff0a508a847"),
    ("cases[1].moves_attempted_total", "34df2d15ee336296547593d47aa1f39f653dcb7b76215dacccd0488e14f579e6"),
    ("cases[1].moves_blocked_avoidable_by_reader_state", "37f4d1a7dd6ca7af352ced029172aa167180a15fd6682fa20dd07a077def17aa"),
    ("cases[1].moves_blocked_barrier_total", "434c9b5ae514646bbd91b50032ca579efec8f22bf0b4aac12e65997c418e0dd6"),
    ("cases[1].moves_blocked_barrier_with_barrier_neighbor_by_reader_state", "6922674ab8e9f97d22aa2799e7e5aef80994115b7aa48aa5e583ed14975c8465"),
    ("cases[1].moves_blocked_total_by_cause", "ac1bb17d714eeef1cb96446409aeeb3a71590ef58cf597b473203e2d2bd16245"),
    ("cases[1].mutation_effects", "a23fdc43a4029f402243af7ba69ac6db1033d2b80e32bd65ceb73fdf02a2674e"),
    ("cases[1].mutation_outcome_summary", "4e520ecaba173eac4f127d978fd088eb599aeaa13f1e5f949deefb8d7efa0c7e"),
    ("cases[1].mutation_supply", "f97bce1a527aaf412d25e98b61baf7038ec60ce5a24f2a37af1016209f9bbdd4"),
    ("cases[1].mutation_value_totals_by_birth_edge_class", "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"),
    ("cases[1].mutation_value_totals_by_operator", "382bc928d6efc0a618985c6925931f2da4de77e2082c368f060c930953c5e63c"),
    ("cases[1].mutational_neighborhood", "7099dade03bf0edcdb874fe00fd82aa73bc60b7c505606f90b3ea8829cb915b9"),
    ("cases[1].neighborhood_read", "7a253f50c61ab99c07918922758ae8ccfd8cb6b60c086c443bef2d6879b6ebb3"),
    ("cases[1].predation", "9ecdde608453a8db80d2a45463c02cf31564f7300761ad47972d8b9aa8f56236"),
    ("cases[1].reachable_structure_size_distribution", "1d75faa2ef3a1acd1a1ad17c4fe5ba4c00fed4460cd0e800f152754f8b865843"),
    ("cases[1].reproductive_success_by_cognitive_class", "a5fd425ea211673ddbf8a98cc8a47c4ad8d1b523c8919d4fa389b93339f88228"),
    ("cases[1].ring_rules", "5219a7473d7c94dbcb48247a23c7a655cda2ff027d2f105531729f87a93a718b"),
    ("cases[1].surviving_clade_profiles", "485167652a31e8b7fb437647ff6ee3670a3cf334f8a51b18c8e0e0e1c00f1a24"),
    ("cases[1].typed_eat_share", "33ac4af3c426fa84e02f7af93d5ba56c214176cb2bfe8c16b14eed4f2b1e5c40"),
    ("cases[1].typed_eats_failed_total", "d0bca111f8628137adc4c16f123496dcdd1d590d06cb5d9acd68b39fe656fb97"),
    ("cases[1].typed_eats_total", "f5beb03b21d9dca9cd7a3adcdf4199a54f95700d0b2d82409d191e2b561b63ef"),
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
    ("cases[2].input_use", "a45093584a34f8be581ea138863153993e2cd0dee9ceac0d96f8758aceb1ee27"),
    ("cases[2].mesh_dispatches_energy_exhausted_total", "5feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9"),
    ("cases[2].mortality", "61cfb12aae9529b788df5df4ce68874354e6c9b2b0d7ecb1592b7a735648fa70"),
    ("cases[2].move_attempts_with_barrier_neighbor_by_reader_state", "0ea321df39f3c33e97c385aeb7926b8009375c712eb4d14ff519139638fe74f7"),
    ("cases[2].moves_attempted_total", "09dc1bb2b25a72c6a5deecbd211750ba6f81b0bd809a2475eefcad2c11ab9091"),
    ("cases[2].moves_blocked_avoidable_by_reader_state", "9de773bd7f8d7a04761686d6094d566f06a837bc3722e0bb368e63b073a02b7e"),
    ("cases[2].moves_blocked_barrier_total", "4a44dc15364204a80fe80e9039455cc1608281820fe2b24f1e5233ade6af1dd5"),
    ("cases[2].moves_blocked_barrier_with_barrier_neighbor_by_reader_state", "cb221266e9cbb382506581815345a584a1a088549cefef97897afe9ffb545f43"),
    ("cases[2].moves_blocked_total_by_cause", "ca3ff3c142745d0756d1bf2ccd1c6b27d09d25a376fb6c19313f34da8302678b"),
    ("cases[2].mutation_effects", "79c2cc5efffcebba51e33771cb2d04ac173fbca02ae829fb69e8a840c99584a7"),
    ("cases[2].mutation_outcome_summary", "4244e569891f81a291ff571268934ccb75c1b9ac0ad22d8c5296c6f43e6e5cac"),
    ("cases[2].mutation_supply", "c7788947d0185adb7244f04c23148042c38f4785d0a1de279d3bb3edfcdc6658"),
    ("cases[2].mutation_value_totals_by_birth_edge_class", "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"),
    ("cases[2].mutation_value_totals_by_operator", "3cac20f189f1e4d6bb00749af3e80eb4383fae7a0c50117c1958df5fb9dd7acf"),
    ("cases[2].mutational_neighborhood", "aaf3ced582e04d5b8eb0fa8b8bd51537008923ec953d8c611bf7711300d07ca6"),
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
    ("deterministic.per_creature_tick", "b58c6c1d5cb73db1558d81f83db3b0d79e7fa3fa818262066fd4caaa37d92f2d"),
    ("deterministic.per_seed", "5663e1ab32927c49fdfc5b383234f1b6079a41b8b6e2b091d4d718276b866972"),
    ("deterministic.profile", "ba56df443f89948289c6042975a23c77a3f5330c2b20790b256ce0d3220be041"),
    ("deterministic.totals", "506c07a28d4f71bbafff4714e3e77252f6c2d1c22d2d5642d5938777c067359f"),
    ("goal_indicators.adaptive_novelty", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.births_per_100_ticks", "8d05f7f938ffd349510f291c16117c621b0f46a70777239f2cfeccc2528be51d"),
    ("goal_indicators.drift_depth", "45f46232a4e3bdaa55bc9e054a23c8cbb557da7fc7539203616c24aec11ed67f"),
    ("goal_indicators.evolutionary_activity", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.information_integration", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.learning_dependence", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.lineage_diversity", "9d10c34f84de4d1314265e4302458b337440919d7c350d5a3f3b3c47fe1b9c31"),
    ("goal_indicators.memory_dependence", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.memory_sensitivity", "ca92a6a85e6a404b90824d64ab9edc3fb522ccce6afd7961a39dbf799d27be01"),
    ("goal_indicators.mutational_neighborhood", "45f46232a4e3bdaa55bc9e054a23c8cbb557da7fc7539203616c24aec11ed67f"),
    ("goal_indicators.prediction_dependence", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.reachable_structure_size_distribution", "af0688a6521a7a2d1ea899e4cdecafdb6f4158ea6b20189d4f4cec18e4374a99"),
    ("goal_indicators.reciprocal_interaction", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.strategy_causal_distinctness", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.strategy_count", "efcedfd579cea7078f98d2a8c06610c41b6edbaeb255d299a7e7ccd2fb74f42a"),
    ("goal_indicators.structural_companions", "a8a5fa432a6827b725860475acc47812a1d330b8273ca170afd3f8435a2c1a59"),
    ("goal_indicators.temporal_memory_sensitivity", "3c97d681b859e69e2e26f447e692b3aa84304bb25ccff4b93ca6e3a0e05bc66a"),
    ("persistence[0]", "5cc4903d121b0676e5c8645e96d0bd6964e7f466c2b87a74bcb3e80638f80bb0"),
    ("persistence[0].last.births_total", "4523540f1504cd17100c4835e85b7eefd49911580f8efff0599a8f283be6b9e3"),
    ("persistence[0].last.bundles", "83708f0e2f51ab4453a01702d094ce33c34e7b8dc95841516001c6f5bae5f484"),
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
    ("persistence[0].last.plasticity_census", "a8504de6d21ab6b2cd09150ebfc3ce318df03bd18f69887498fe54e948d4454e"),
    ("persistence[0].last.population", "9f14025af0065b30e47e23ebb3b491d39ae8ed17d33739e5ff3827ffb3634953"),
    ("persistence[0].last.sensor_census", "e0feb67e62f8bb22ed7077f0a3fea91a99ebe07c51229189dc904f4f43bfa14c"),
    ("persistence[0].last.shannon_entropy_nats", "2632fc5321b23d864ae601b23e49f4ee48cf660aa36fe7469cd32efaa7dc695d"),
    ("persistence[0].last.shares", "791b5fe7847ce66254f448acbb8b35eebe0b8ad3b5516aa6a7d5f23cf820dc78"),
    ("persistence[0].last.surviving_founder_clade_count", "c2356069e9d1e79ca924378153cfbbfb4d4416b1f99d41a2940bfdb66c5319db"),
    ("persistence[0].last.tick", "811786ad1ae74adfdd20dd0372abaaebc6246e343aebd01da0bfc4c02bf0106c"),
    ("persistence[0].last.typed_eats_total", "8ada8db46d26aa873c1f1e74568b2b21a952c0cbefd7a2d7e66be05bcd0d9870"),
    ("persistence[0].samples", "5e5ac2e3602b806ef6262ad4029ea7c79e5a8a34c7dfe88e1f3f55e98d9e762d"),
    ("persistence[1]", "1bc9e35490adf7d490f163b82799191e54a69de28d7a702189aaab4d4f38c485"),
    ("persistence[1].last.births_total", "9f14025af0065b30e47e23ebb3b491d39ae8ed17d33739e5ff3827ffb3634953"),
    ("persistence[1].last.bundles", "cbebd73af7048efe6742065e0b5c2b29654b7b6702722ec674a6fe3204c3b407"),
    ("persistence[1].last.capacity", "2297f8e252fbf20fe344d4415b46b8a5d86dcb8469fde86a8c9ff455c603d527"),
    ("persistence[1].last.food_density_total", "7244246cb5f028789589b1f93bc779be34e1c2c7418a8e5abd1f9ba5af78d45f"),
    ("persistence[1].last.grazed_cell_share", "b0047c03711300d119c19d129cc2dfb98168b199f81d147fafcac5a24ba71501"),
    ("persistence[1].last.grazing_modifier_mean", "f09bd0e1d5823dfe40742396b0459bfe76c62327b074a6e01f07aa28769a4637"),
    ("persistence[1].last.mean_energy", "9447255a1a61d0bfccdf964d0d247b4d44f675a56ec3e27477095451584464c9"),
    ("persistence[1].last.mean_generation", "3edd52e8b46d6a2d604d99083412b8169f9e557f7292ca5890a20ed5fb9c7f52"),
    ("persistence[1].last.mean_genome_size", "f46915eee57c221dc491f835485bdfcd57003ca72cec2725da2f4781863142b7"),
    ("persistence[1].last.mean_mesh_nodes", "4e4faeec892654210404abfdf29df5eadc0722da64e6c399f790c4eb921cfc57"),
    ("persistence[1].last.mean_wired_sinks", "6b7d18583d9ce7a9a72d55a1fdccf977f257b78a467d7455aab3fbc71b0b52ef"),
    ("persistence[1].last.move_attempts_with_barrier_neighbor_by_reader_state", "55655d4121ffede6287b3740da52dfd1c6fce6bfdaa7f1a1dcd40ff0a508a847"),
    ("persistence[1].last.moves_attempted_total", "34df2d15ee336296547593d47aa1f39f653dcb7b76215dacccd0488e14f579e6"),
    ("persistence[1].last.moves_blocked_avoidable_by_reader_state", "37f4d1a7dd6ca7af352ced029172aa167180a15fd6682fa20dd07a077def17aa"),
    ("persistence[1].last.moves_blocked_barrier_total", "434c9b5ae514646bbd91b50032ca579efec8f22bf0b4aac12e65997c418e0dd6"),
    ("persistence[1].last.moves_blocked_barrier_with_barrier_neighbor_by_reader_state", "6922674ab8e9f97d22aa2799e7e5aef80994115b7aa48aa5e583ed14975c8465"),
    ("persistence[1].last.moves_blocked_total_by_cause", "ac1bb17d714eeef1cb96446409aeeb3a71590ef58cf597b473203e2d2bd16245"),
    ("persistence[1].last.occupancy_grid", "84a5ce8c254ad154337676602100c9562ff850a4baa0da67a12908a31001ecb4"),
    ("persistence[1].last.plasticity_census", "aae013576227007a8d9a009d751bc53d61a85b6bc5ce9962587e17ed4e98213d"),
    ("persistence[1].last.population", "eb1e33e8a81b697b75855af6bfcdbcbf7cbbde9f94962ceaec1ed8af21f5a50f"),
    ("persistence[1].last.sensor_census", "b912dda201c59924f02fd6d67c7a943601f3eb5c3e57173b92eaacb7facbb137"),
    ("persistence[1].last.shannon_entropy_nats", "39cb69d96e0c8525165ee73d9def4106811544b7ac18311d706a4d63b840e48e"),
    ("persistence[1].last.shares", "d0eeeb82091236747f84b26c116212d5ebd38977d24fd5698fe06576691568aa"),
    ("persistence[1].last.surviving_founder_clade_count", "785f3ec7eb32f30b90cd0fcf3657d388b5ff4297f2f9716ff66e9b69c05ddd09"),
    ("persistence[1].last.tick", "811786ad1ae74adfdd20dd0372abaaebc6246e343aebd01da0bfc4c02bf0106c"),
    ("persistence[1].last.typed_eats_total", "f5beb03b21d9dca9cd7a3adcdf4199a54f95700d0b2d82409d191e2b561b63ef"),
    ("persistence[1].samples", "0f80daec53382929681e8e4945c4e45fc90e37facfa2d2aa510b47e466a86c91"),
    ("persistence[2]", "c2d6eb137454fb6712620b68eed5ea3df1f439c215a89c121f33ae2e2b3dd7a5"),
    ("persistence[2].last.births_total", "44cb730c420480a0477b505ae68af508fb90f96cf0ec54c6ad16949dd427f13a"),
    ("persistence[2].last.bundles", "eb2bd85e8ab9e5173d1eeb36f0e47e10b9b960294025f6019a9896b9b5d4f336"),
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
    ("persistence[2].last.plasticity_census", "e86512bdbcc7f1f83328ca9e4828d3983cd2bbc09e24c052f24d9cf4701fedfd"),
    ("persistence[2].last.population", "73475cb40a568e8da8a045ced110137e159f890ac4da883b6b17dc651b3a8049"),
    ("persistence[2].last.sensor_census", "6ed45a4dd8ba75a59acaa79fae4eb85adba045ec7f8191af28e98ba1bfcc0118"),
    ("persistence[2].last.shannon_entropy_nats", "6c4b9ac785d64cfedceb22028e77935122fbb7ab1f6e3b725011b5d335e89722"),
    ("persistence[2].last.shares", "04b7631f60590a62732eb9d0bd31abc4db35366695e0d51ab236b077fc5f7032"),
    ("persistence[2].last.surviving_founder_clade_count", "785f3ec7eb32f30b90cd0fcf3657d388b5ff4297f2f9716ff66e9b69c05ddd09"),
    ("persistence[2].last.tick", "811786ad1ae74adfdd20dd0372abaaebc6246e343aebd01da0bfc4c02bf0106c"),
    ("persistence[2].last.typed_eats_total", "6e7d1d13383663a41fd3d780d610696b7cdd26dbfe8381f6452beea296651c7e"),
    ("persistence[2].samples", "3d101af30315be191969f8de2312bf11f64e0ea9f11823d40cef76a4e5a39641"),
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
    ("evolved_neighborhood", "a61a21050b015db3b0d1e60c224b808aada7788ee6305908b27a62dace125470"),
    ("generation_distribution", "78759ebefd64bbe86da1a6b58e7a8b33839485ef7abbce24ad6b921e13ecb53b"),
    ("gestation", "5b3476658aae17b52c06058aafecc99b1a800b609d634211ab067762fa6fe523"),
    ("lineage_diversity", "87f75640c8fa7b1e176769020468016568dd14c5aaddd1c13a57584e60d0bbae"),
    ("memory_sensitivity", "eca8c579acc0cae4a9830ad20d9bb232e13d0d2528b8994799ce51499563544d"),
    ("neighborhood_read", "8eeec1f6621498a607f2a138970879ad8cbba280fbc3089d383ca0ee13e19fb5"),
    ("read_exposure", "c3134fc57f207e4a382f6736fddef105a5b85690be28031af7b6e01b4b3c2e88"),
    ("sample", "a7ab30f3af403b261ffb4762836e4f0d8c9175e1ad3bfa651c884084c96fa28a"),
    ("selected_cohort", "ff927634d72309906d219ed39ac8eabbf7e13c7e67b3c9d043c06727847e3417"),
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
    ("sample", "01d55c586f41d9e00f071e070bc80985a9ca747b5da8c7eb23b65fd2a70ae25b"),
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
    "9b84964a64b10e9c6927adc083f014246f9df49652d7efc3e4da0e8c0d1a3446",
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
