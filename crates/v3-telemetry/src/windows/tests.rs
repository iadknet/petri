use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use proptest::prelude::*;
use v3_core::config::SimulationConfig;
use v3_core::simulation::{run_tick, seed_simulation};

use super::*;
use crate::testing::{decode_traces, parse_run_line, ReceivedSpan, ReceivedTrace, Receiver};
use crate::{EndStatus, Flush, Limits, Options, ReportSink, RunStart, Service, Telemetry};

const RUN_KEY: u128 = 0x0123_4567_89ab_cdef_fedc_ba98_7654_3210;
const STARTED_NS: u64 = 1_700_000_000_000_000_000;

fn run(started: Instant) -> RunHandle {
    RunHandle {
        key: RUN_KEY,
        id: format!("{RUN_KEY:032x}"),
        seed: 7,
        world: "24x24".to_owned(),
        recipe: None,
        config_digest: "digest-0".to_owned(),
        started,
        started_ns: STARTED_NS,
        last_snapshot: None,
        last_stamp_ns: None,
        traces_taken: 0,
        samples: Samples::default(),
    }
}

/// Two food types, so the typed banks hold one entry per type.
fn config() -> SimulationConfig {
    let mut config = SimulationConfig::default();
    config.world.width = 24;
    config.world.height = 24;
    config.population.initial_creatures = 12;
    let mut second = config.world.food.types[0].clone();
    second.name = "Fruit".to_owned();
    config.world.food.types.push(second);
    config
}

/// A window of `ticks` on the first creature of a fresh seeded run.
fn recorded(ticks: u32) -> (ActiveTrace, u64) {
    let mut sim = seed_simulation(config(), 3);
    let id = sim.creatures.keys().next().unwrap();
    let mut slot = Some(window_trace(id, ticks));
    for _ in 0..ticks {
        run_tick(&mut sim, &mut slot);
    }
    (slot.unwrap(), id.data().as_ffi())
}

fn meta(index: u64, policy: Policy, creature_id: u64, started_at: Instant) -> SampleMeta {
    SampleMeta {
        index,
        policy,
        creature_id,
        started_at,
        digest: "digest-0".to_owned(),
        root: Vec::new(),
        start: Start::default(),
        traced: true,
        ticks_requested: 3,
        tick_digests: Vec::new(),
        config_changed: false,
    }
}

fn decoded(request: &ExportTraceServiceRequest) -> ReceivedTrace {
    decode_traces(&request.encode_to_vec()).remove(0)
}

fn ns(run: &RunHandle, instant: Instant) -> u64 {
    STARTED_NS + u64::try_from(instant.duration_since(run.started).as_nanos()).unwrap()
}

#[test]
fn window_settings_default_to_on_8_ticks_10_s_and_refuse_anything_else() {
    assert_eq!(
        WindowSettings::resolve(WindowSettings::default(), None, Some(""), Some(" ")),
        Ok(WindowSettings::default())
    );
    assert_eq!(WindowSettings::default().ticks, 8);
    assert_eq!(WindowSettings::default().interval, Duration::from_secs(10));
    let set = WindowSettings::resolve(
        WindowSettings::default(),
        Some("off"),
        Some("64"),
        Some("10"),
    )
    .unwrap();
    assert_eq!(
        (set.switch, set.ticks, set.interval),
        (Switch::Off, 64, Duration::from_millis(10))
    );
    for (switch, ticks, interval) in [
        (Some("yes"), None, None),
        (None, Some("0"), None),
        (None, Some("65"), None),
        (None, None, Some("9")),
        (None, None, Some("3600001")),
        (None, None, Some("1.5")),
    ] {
        assert!(
            WindowSettings::resolve(WindowSettings::default(), switch, ticks, interval).is_err()
        );
    }
}

proptest! {
    #[test]
    fn selection_is_a_position_in_the_population(seed: u64, k: u64, population in 1usize..10_000) {
        let n = select(seed, k, population);
        prop_assert!(n < population);
        prop_assert_eq!(n, select(seed, k, population));
    }

    #[test]
    fn a_sample_trace_id_never_meets_a_tick_trace_id(
        key: u128,
        k in 0u64..(1 << 62),
        tick in 0u64..FIRST_UNTRACED_TICK,
    ) {
        let sample = sample_trace_id(key, k);
        prop_assert_eq!(&sample[..8], &trace::trace_id(key, tick)[..8]);
        prop_assert_eq!(&sample[8..], &((1u64 << 63) + k).to_be_bytes()[..]);
        prop_assert_ne!(sample, trace::trace_id(key, tick));
    }
}

use crate::FIRST_UNTRACED_TICK;

#[test]
fn a_window_encodes_predicted_ids_parents_times_names_and_attributes() {
    let started = Instant::now();
    let (trace, creature_id) = recorded(3);
    let run = run(started);
    let mut meta = meta(
        5,
        Policy::Window,
        creature_id,
        started + Duration::from_millis(1),
    );
    meta.tick_digests = vec!["d1".into(), "d2".into(), "d3".into()];
    let mut config_attrs = Attributes::default();
    trace::cognition(&mut config_attrs, &config());
    meta.root = config_attrs.0;
    let ending = Ending {
        reason: EndReason::Complete,
        tick: 3,
        tick_end: None,
    };
    let (request, encoded) = encode(&[], &run, &meta, &trace, &ending);
    let received = decoded(&request);
    assert_eq!(received.spans.len(), 4);
    let trace_id = format!("{:016x}{:016x}", RUN_KEY >> 64, (1u64 << 63) + 5);
    let root = &received.spans[0];
    assert!(received.spans.iter().all(|span| span.trace_id == trace_id));
    assert_eq!(root.name, "creature_window");
    assert_eq!(root.span_id, "0000000000000001");
    assert_eq!(root.parent_span_id, "");
    assert_eq!(root.start_time_unix_nano, ns(&run, meta.started_at));
    assert_eq!(root.attribute("petri.config_digest"), Some("digest-0"));
    assert_eq!(root.attribute("petri.sample_policy"), Some("window"));
    assert_eq!(root.attribute("petri.window"), Some("5"));
    assert_eq!(
        root.attribute("petri.creature_id"),
        Some(creature_id.to_string().as_str())
    );
    assert_eq!(root.attribute("petri.tick"), Some("1"));
    assert_eq!(root.attribute("petri.tick_end"), Some("3"));
    assert_eq!(root.attribute("petri.ticks_recorded"), Some("3"));
    assert_eq!(root.attribute("petri.end_reason"), Some("complete"));
    assert_eq!(root.attribute("petri.truncated"), Some("false"));
    assert_eq!(
        root.attribute("petri.config.runtime.max_mesh_hops"),
        Some("64")
    );
    let mut events = 0;
    for (i, (span, record)) in received.spans[1..].iter().zip(&trace.ticks).enumerate() {
        let seam = record.outcome.as_ref().unwrap().phases.unwrap();
        assert_eq!(span.name, "creature_tick");
        assert_eq!(span.span_id, format!("{:016x}", (i as u64 + 1) << 8));
        assert_eq!(span.parent_span_id, root.span_id);
        assert_eq!(span.start_time_unix_nano, ns(&run, seam.started));
        assert_eq!(
            span.end_time_unix_nano,
            ns(&run, seam.started + seam.elapsed)
        );
        assert_eq!(
            span.attribute("petri.tick"),
            Some((record.tick_number + 1).to_string().as_str())
        );
        assert_eq!(
            span.attribute("petri.config_digest"),
            Some(meta.tick_digests[i].as_str())
        );
        assert_eq!(span.attribute("petri.phase"), Some("cognition"));
        for key in [
            "petri.neighbor_food",
            "petri.neighbor_food.0",
            "petri.neighbor_food.1",
            "petri.food_here_by_type",
            "petri.position",
            "petri.actions_selected",
            "petri.commit_counts",
            "petri.energy_end",
            "petri.shared_memory",
            "petri.previous_outcome_end",
        ] {
            assert!(span.attribute(key).is_some(), "{key}");
        }
        // The founder never reads extended perception: its banks are absent.
        for key in PERCEPTION_KEYS {
            assert!(span.attribute(key).is_none(), "{key}");
        }
        assert!(span
            .attribute("petri.neighbor_food")
            .unwrap()
            .starts_with('['));
        let cognition = ns(&run, seam.started + seam.phases[2].offset);
        let actions = ns(&run, seam.started + seam.phases[3].offset);
        assert_eq!(span.events_named("hop").count(), record.hops.len());
        assert_eq!(span.events_named("pass").count(), record.passes.len());
        let applied = record.outcome.as_ref().unwrap().applied.len();
        assert_eq!(span.events_named("action").count(), applied);
        assert!(span
            .events_named("hop")
            .all(|e| e.time_unix_nano == cognition));
        assert!(span
            .events_named("action")
            .all(|e| e.time_unix_nano == actions && e.attribute("petri.phase") == Some("actions")));
        events += record.hops.len() + record.passes.len() + applied;
    }
    let last = &received.spans[3];
    assert_eq!(root.end_time_unix_nano, last.end_time_unix_nano);
    assert_eq!(
        root.attribute("petri.events"),
        Some(events.to_string().as_str())
    );
    assert_eq!(
        encoded,
        Encoded {
            ticks: 3,
            events: events as u32
        }
    );
}

/// The extended-perception attributes of a two-food-type tick span.
const PERCEPTION_KEYS: [&str; 8] = [
    "petri.area_food",
    "petri.area_barrier",
    "petri.area_occupancy",
    "petri.nearby_core",
    "petri.nearby_vitals",
    "petri.nearby_identity",
    "petri.area_food.0",
    "petri.area_food.1",
];

const TYPED_LOCAL_KEYS: [&str; 3] = [
    "petri.food_here_by_type",
    "petri.neighbor_food.0",
    "petri.neighbor_food.1",
];

#[test]
fn a_tick_span_carries_exactly_the_banks_sensor_assembly_built() {
    let started = Instant::now();
    let (mut trace, creature_id) = recorded(1);
    let run = run(started);
    let meta = meta(0, Policy::Window, creature_id, started);
    let ending = Ending {
        reason: EndReason::Complete,
        tick: 1,
        tick_end: None,
    };
    let outcome = trace.ticks[0].outcome.as_mut().unwrap();
    outcome.uses_extended_perception = true;
    outcome.uses_typed_local_food = false;
    outcome.typed_area_food = vec![[0.25; 7]; 2];
    let received = decoded(&encode(&[], &run, &meta, &trace, &ending).0);
    let span = &received.spans[1];
    for key in PERCEPTION_KEYS {
        assert!(span.attribute(key).is_some(), "{key}");
    }
    for key in TYPED_LOCAL_KEYS {
        assert!(span.attribute(key).is_none(), "{key}");
    }
}

#[test]
fn a_zero_tick_window_is_its_root_alone_spanning_its_start_instant() {
    let started = Instant::now();
    let (mut trace, creature_id) = recorded(1);
    trace.ticks.clear();
    let run = run(started);
    let meta = meta(
        0,
        Policy::Manual,
        creature_id,
        started + Duration::from_millis(2),
    );
    let ending = Ending {
        reason: EndReason::Replaced,
        tick: 4,
        tick_end: None,
    };
    let received = decoded(&encode(&[], &run, &meta, &trace, &ending).0);
    assert_eq!(received.spans.len(), 1);
    let root = &received.spans[0];
    assert_eq!(root.start_time_unix_nano, root.end_time_unix_nano);
    assert_eq!(root.start_time_unix_nano, ns(&run, meta.started_at));
    assert_eq!(root.attribute("petri.tick"), Some("4"));
    assert_eq!(root.attribute("petri.ticks_recorded"), Some("0"));
    assert_eq!(root.attribute("petri.end_reason"), Some("replaced"));
    assert_eq!(root.attribute("petri.sample_policy"), Some("manual"));
}

#[test]
fn a_manual_sample_past_2048_events_is_cut_at_encoding_and_marked_truncated() {
    let started = Instant::now();
    let (mut trace, creature_id) = recorded(2);
    trace.budget = None;
    let hop = trace.ticks[1].hops[0].clone();
    trace.ticks[1].hops = vec![hop; 2_100];
    let run = run(started);
    let meta = meta(0, Policy::Manual, creature_id, started);
    let ending = Ending {
        reason: EndReason::Complete,
        tick: 2,
        tick_end: None,
    };
    let (request, encoded) = encode(&[], &run, &meta, &trace, &ending);
    let received = decoded(&request);
    let events: usize = received.spans.iter().map(|span| span.events.len()).sum();
    assert_eq!(events, 2_048);
    assert_eq!(encoded.events, 2_048);
    let root = &received.spans[0];
    assert_eq!(root.attribute("petri.truncated"), Some("true"));
    assert_eq!(root.attribute("petri.truncated_reason"), Some("events"));
    assert_eq!(root.attribute("petri.truncated_tick"), Some("2"));
}

#[test]
fn a_tick_allocates_no_more_events_than_the_allowance_left() {
    let (mut trace, _) = recorded(1);
    let hop = trace.ticks[0].hops[0].clone();
    trace.ticks[0].hops = vec![hop; 100];
    let record = &trace.ticks[0];
    for allowance in [0, 1, 5, 99] {
        let mut left = allowance;
        let (events, cut) = tick_events(record, 0, 0, &mut left);
        assert_eq!(events.len(), allowance as usize);
        assert!(events.capacity() <= allowance as usize, "{allowance}");
        assert!(cut);
        assert_eq!(left, 0);
    }
    let total = record.hops.len() + record.passes.len();
    let mut left = 2_048;
    let (events, cut) = tick_events(record, 0, 0, &mut left);
    assert!(!cut);
    assert!(events.len() >= total);
    assert_eq!(left as usize, 2_048 - events.len());
}

// ── The `creature.window` record (T23.F10) ──────────────────────────────

#[test]
fn keep_levels_of_indexes_0_1_2_10_and_15() {
    let levels: Vec<u8> = [0, 1, 2, 10, 15]
        .into_iter()
        .map(record::keep_level)
        .collect();
    assert_eq!(levels, [2, 0, 1, 2, 0]);
}

proptest! {
    /// The levels nest: level 2 is every tenth index, level at least 1 every
    /// second, so each thinning band keeps a subset of the band before it.
    #[test]
    fn keep_levels_nest_tenths_inside_evens(index: u64) {
        let level = record::keep_level(index);
        prop_assert!(level <= 2);
        prop_assert_eq!(level == 2, index % 10 == 0);
        prop_assert_eq!(level >= 1, index % 2 == 0);
    }
}

/// `value` as the record serializes it: an `f32` in shortest round-trip form.
fn f(value: f32) -> serde_json::Value {
    serde_json::from_str(&serde_json::to_string(&value).unwrap()).unwrap()
}

fn ring(values: &[f32; 8]) -> serde_json::Value {
    values.iter().copied().map(f).collect()
}

fn record_of(meta: &SampleMeta, trace: &ActiveTrace, ending: &Ending) -> serde_json::Value {
    serde_json::from_str(&record::body(meta, trace, ending)).unwrap()
}

#[test]
fn a_window_record_holds_every_body_field_with_exact_values() {
    let started = Instant::now();
    let (mut trace, creature_id) = recorded(2);
    let graph = trace.ticks[0].hops[0].clone();
    assert!(matches!(graph.backend_trace, BackendTrace::Graph(_)));
    let mut first = graph.clone();
    first.node_id = v3_core::contracts::NodeId(7);
    first.vote_contribution.fill(0.0);
    first.vote_contribution[0] = 0.5;
    first.vote_contribution[1] = -0.25;
    let mut vm = graph;
    vm.node_id = v3_core::contracts::NodeId(3);
    vm.vote_contribution.fill(0.0);
    vm.backend_trace = BackendTrace::Vm(v3_core::runtime::trace::domain::VmTrace {
        register_count: 0,
        constants: Vec::new(),
        steps: Vec::new(),
        final_registers: Vec::new(),
        final_payload: vm.output_slots,
        slot_writes: Vec::new(),
    });
    trace.ticks[0].hops = vec![first, vm];
    trace.ticks[0].energy_before = 0.1;
    trace.ticks[0].static_inputs.food_here = 0.3;
    trace.ticks[0]
        .outcome
        .as_mut()
        .unwrap()
        .after
        .as_mut()
        .unwrap()
        .energy = 2.5;
    // The second tick removed its creature and read no typed food bank.
    let second = trace.ticks[1].outcome.as_mut().unwrap();
    second.after = None;
    second.died = Some(DeathCause::LifecycleDecay);
    second.uses_typed_local_food = false;
    let mut meta = meta(15, Policy::Window, creature_id, started);
    meta.start = Start {
        lineage: 4,
        generation: 2,
        genome_size: 97,
        genome_hash: "sha256:ab".to_owned(),
        age: 11,
    };
    meta.config_changed = true;
    let ending = Ending {
        reason: EndReason::Died,
        tick: 2,
        tick_end: None,
    };
    let body = record::body(&meta, &trace, &ending);
    assert!(body.contains("\"e0\":0.1,"), "{body}");
    assert!(body.contains("\"e1\":2.5,"), "{body}");
    assert!(body.contains("[7,\"g\",0.75],[3,\"v\",0.0]"), "{body}");
    let ticks: Vec<serde_json::Value> = trace
        .ticks
        .iter()
        .map(|record| {
            let outcome = record.outcome.as_ref().unwrap();
            let sensed = &record.static_inputs;
            let mut tick = serde_json::json!({
                "t": record.tick_number + 1,
                "e0": f(record.energy_before),
                "here": f(sensed.food_here),
                "food": ring(&sensed.neighbor_food),
                "barrier": ring(&sensed.neighbor_barrier),
                "occupied": ring(&sensed.neighbor_occupied),
                "sel": record.final_actions.iter().map(action_label).collect::<Vec<_>>(),
                "res": outcome.applied.iter().map(|a| a.entry.result.as_key()).collect::<Vec<_>>(),
                "hops": record.hops.iter().map(|hop| serde_json::json!([
                    hop.node_id.0,
                    if matches!(hop.backend_trace, BackendTrace::Vm(_)) { "v" } else { "g" },
                    f(hop.vote_contribution.iter().map(|v| v.abs()).sum()),
                ])).collect::<Vec<_>>(),
            });
            if let Some(after) = &outcome.after {
                tick["e1"] = f(after.energy);
            }
            if outcome.uses_typed_local_food {
                tick["food_by_type"] = outcome
                    .typed_local_food
                    .neighbor_food_by_type
                    .iter()
                    .map(ring)
                    .collect();
            }
            tick
        })
        .collect();
    assert_eq!(ticks[0]["food_by_type"].as_array().map(Vec::len), Some(2));
    assert!(ticks[1].get("e1").is_none());
    assert!(ticks[1].get("food_by_type").is_none());
    let expected = serde_json::json!({
        "v": 1,
        "window": 15,
        "policy": "window",
        "creature": creature_id.to_string(),
        "ticks_requested": 3,
        "lineage": 4,
        "generation": 2,
        "genome_size": 97,
        "genome_hash": "sha256:ab",
        "age_start": 11,
        "end": "died",
        "died": "lifecycle_decay",
        "config_changed": true,
        "ticks": ticks,
    });
    assert_eq!(record_of(&meta, &trace, &ending), expected);
}

#[test]
fn a_zero_tick_manual_sample_records_its_start_and_no_tick() {
    let started = Instant::now();
    let (mut trace, creature_id) = recorded(1);
    trace.ticks.clear();
    let meta = meta(4, Policy::Manual, creature_id, started);
    let ending = Ending {
        reason: EndReason::Replaced,
        tick: 4,
        tick_end: None,
    };
    let record = record_of(&meta, &trace, &ending);
    assert_eq!(record["policy"], "manual");
    assert_eq!(record["end"], "replaced");
    assert_eq!(record["ticks"], serde_json::json!([]));
    for absent in ["died", "truncated", "config_changed"] {
        assert!(record.get(absent).is_none(), "{absent}");
    }
}

#[test]
fn a_manual_record_past_2048_hops_cuts_only_hops_and_is_truncated() {
    let started = Instant::now();
    let (mut trace, creature_id) = recorded(3);
    trace.budget = None;
    let hop = trace.ticks[1].hops[0].clone();
    trace.ticks[1].hops = vec![hop; 2_100];
    let meta = meta(0, Policy::Manual, creature_id, started);
    let ending = Ending {
        reason: EndReason::Complete,
        tick: 3,
        tick_end: None,
    };
    let record = record_of(&meta, &trace, &ending);
    let ticks = record["ticks"].as_array().unwrap();
    let hops: usize = ticks
        .iter()
        .map(|t| t["hops"].as_array().unwrap().len())
        .sum();
    assert_eq!(hops, record::MAX_RECORD_HOPS);
    assert_eq!(
        ticks[0]["hops"].as_array().unwrap().len(),
        trace.ticks[0].hops.len()
    );
    assert_eq!(ticks[2]["hops"], serde_json::json!([]));
    for (tick, recorded) in ticks.iter().zip(&trace.ticks) {
        assert_eq!(
            tick["sel"].as_array().unwrap().len(),
            recorded.final_actions.len()
        );
        assert_eq!(
            tick["res"].as_array().unwrap().len(),
            recorded.outcome.as_ref().unwrap().applied.len()
        );
    }
    assert_eq!(
        record["truncated"],
        serde_json::json!({"reason": "hops", "tick": 2})
    );
}

// ── Through the exporter ────────────────────────────────────────────────

fn telemetry(receiver: &Receiver, limits: Limits, interval_ms: u64) -> (Telemetry, ReportSink) {
    let reports = ReportSink::capture();
    let telemetry = Telemetry::start_with(Options {
        service: Service::Cli,
        endpoint: receiver.endpoint().to_owned(),
        limits,
        reports: reports.clone(),
        metrics_interval: Duration::from_secs(3_600),
        tick_traces: Switch::On,
        windows: WindowSettings {
            switch: Switch::On,
            ticks: 2,
            interval: Duration::from_millis(interval_ms),
        },
        preset: crate::Preset::Standard,
    });
    (telemetry, reports)
}

fn start(config: &SimulationConfig) -> RunStart<'_> {
    RunStart {
        seed: 3,
        config,
        recipe: None,
        tick: 0,
        ticks_requested: None,
        sample_every: None,
    }
}

fn roots(receiver: &Receiver) -> Vec<ReceivedSpan> {
    receiver
        .traces()
        .into_iter()
        .filter_map(|trace| {
            trace
                .spans
                .into_iter()
                .find(|s| s.name == "creature_window")
        })
        .collect()
}

fn line(reports: &ReportSink, run: &str) -> std::collections::BTreeMap<String, String> {
    reports
        .lines()
        .iter()
        .filter_map(|line| parse_run_line(line))
        .find(|fields| fields.get("run").map(String::as_str) == Some(run))
        .expect("the run line")
}

/// Runs ticks with windows until `ticks` have run.
fn tick_with_windows(
    telemetry: &Telemetry,
    run: &mut RunHandle,
    sim: &mut Simulation,
    slot: &mut Option<ActiveTrace>,
    ticks: u64,
) {
    for _ in 0..ticks {
        telemetry.before_tick(run, sim, slot, false);
        run_tick(sim, slot);
        telemetry.after_tick(run, sim, slot, None);
    }
}

#[test]
fn windows_and_manual_samples_alternate_with_distinct_ids() {
    let receiver = Receiver::start();
    let (telemetry, reports) = telemetry(&receiver, Limits::default(), 10);
    let config = config();
    let mut sim = seed_simulation(config.clone(), 3);
    let mut run = telemetry.begin_run(start(&config)).unwrap();
    let mut slot = None;
    tick_with_windows(&telemetry, &mut run, &mut sim, &mut slot, 1);
    std::thread::sleep(Duration::from_millis(15));
    let id = sim.creatures.keys().nth(1).unwrap();
    let manual = ActiveTrace::new(id, 1);
    telemetry.start_manual(&mut run, &sim, &mut slot, None, &manual);
    assert!(slot.is_none(), "the window ended for the manual sample");
    let mut manual = Some(manual);
    run_tick(&mut sim, &mut manual);
    telemetry.after_tick(&mut run, &sim, &mut slot, manual.as_ref());
    telemetry.hand_over(&mut run, &sim, manual.as_ref().unwrap());
    std::thread::sleep(Duration::from_millis(15));
    tick_with_windows(&telemetry, &mut run, &mut sim, &mut slot, 2);
    telemetry.end_samples(&mut run, &sim, &mut slot, None);
    let id = run.id().to_owned();
    telemetry.end_run(run, EndStatus::Completed, sim.tick, Flush::Wait);

    let roots = roots(&receiver);
    let policies: Vec<_> = roots
        .iter()
        .map(|r| {
            (
                r.attribute("petri.window").unwrap().to_owned(),
                r.attribute("petri.sample_policy").unwrap().to_owned(),
                r.attribute("petri.end_reason").unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        policies,
        [
            ("0".into(), "window".into(), "manual".into()),
            ("1".into(), "manual".into(), "complete".into()),
            ("2".into(), "window".into(), "complete".into()),
        ]
    );
    let ids: BTreeSet<_> = roots.iter().map(|r| r.trace_id.clone()).collect();
    assert_eq!(ids.len(), 3);
    let fields = line(&reports, &id);
    assert_eq!(fields["windows"], "3");
    assert_eq!(fields["window_records"], "3");
    assert_eq!(fields["recorded"], "4");
    let records: Vec<_> = receiver
        .records()
        .into_iter()
        .filter(|r| r.event_name == "creature.window")
        .map(|r| {
            let body: serde_json::Value = serde_json::from_str(r.body.as_deref().unwrap()).unwrap();
            (
                r.attribute("petri.window").unwrap().to_owned(),
                r.attribute("petri.keep").unwrap().to_owned(),
                r.attribute("petri.sample_policy").unwrap().to_owned(),
                r.attribute("petri.tick").unwrap().to_owned(),
                body["end"].as_str().unwrap().to_owned(),
                body["creature"] == r.attribute("petri.creature_id").unwrap(),
            )
        })
        .collect();
    let row = |w: &str, keep: &str, policy: &str, tick: &str, end: &str| {
        (
            w.to_owned(),
            keep.to_owned(),
            policy.to_owned(),
            tick.to_owned(),
            end.to_owned(),
            true,
        )
    };
    assert_eq!(
        records,
        [
            row("0", "2", "window", "1", "manual"),
            row("1", "0", "manual", "2", "complete"),
            row("2", "1", "window", "4", "complete"),
        ]
    );
    let genomes = receiver
        .records()
        .into_iter()
        .filter(|r| r.event_name == "creature.genome")
        .count();
    assert_eq!(genomes, 3);
}

/// What one sample started with the run's samples prepared by `prepare`
/// exported.
#[derive(Debug, PartialEq, Eq)]
struct Capped {
    admitted: bool,
    cap: Option<String>,
    /// Bytes counted toward the per-run cap.
    counted: u64,
    genomes: usize,
    traces: usize,
    /// The `creature.window` records' `petri.window` values.
    records: Vec<String>,
}

fn capped_run(receiver: &Receiver, manual: bool, prepare: impl Fn(&mut Samples)) -> Capped {
    let (telemetry, reports) = telemetry(receiver, Limits::default(), 10);
    let config = config();
    let mut sim = seed_simulation(config.clone(), 3);
    let mut run = telemetry.begin_run(start(&config)).unwrap();
    prepare(&mut run.samples);
    let before = run.samples.bytes;
    let mut slot = None;
    let sample = ActiveTrace::new(sim.creatures.keys().next().unwrap(), 1);
    let admitted = if manual {
        telemetry.start_manual(&mut run, &sim, &mut slot, None, &sample);
        run.samples.manual.is_some()
    } else {
        telemetry.before_tick(&mut run, &sim, &mut slot, false);
        slot.is_some()
    };
    let mut manual_slot = Some(sample);
    if manual {
        run_tick(&mut sim, &mut manual_slot);
        telemetry.end_samples(&mut run, &sim, &mut slot, manual_slot.as_ref());
    } else {
        run_tick(&mut sim, &mut slot);
        telemetry.end_samples(&mut run, &sim, &mut slot, None);
    }
    let counted = run.samples.bytes - before;
    let id = run.id().to_owned();
    telemetry.end_run(run, EndStatus::Completed, sim.tick, Flush::Wait);
    let mine: Vec<_> = receiver
        .records()
        .into_iter()
        .filter(|r| r.attribute("petri.run_id") == Some(id.as_str()))
        .collect();
    let cap = mine
        .iter()
        .find(|r| r.event_name == "run.ended")
        .and_then(|r| r.attribute("petri.windows_capped").map(str::to_owned));
    let named = |name: &'static str| mine.iter().filter(move |r| r.event_name == name);
    let fields = line(&reports, &id);
    let records: Vec<String> = named("creature.window")
        .map(|r| r.attribute("petri.window").unwrap().to_owned())
        .collect();
    assert_eq!(fields["window_records"], records.len().to_string());
    Capped {
        admitted,
        cap,
        counted,
        genomes: named("creature.genome").count(),
        traces: roots(receiver)
            .iter()
            .filter(|r| r.attribute("petri.run_id") == Some(id.as_str()))
            .count(),
        records,
    }
}

#[test]
fn samples_past_either_cap_are_admitted_and_recorded_without_trace_or_genome() {
    let receiver = Receiver::start();
    for manual in [false, true] {
        let below = capped_run(&receiver, manual, |s| s.next_index = 4_095);
        assert!(below.admitted, "{manual}");
        assert_eq!(below.cap, None);
        assert!(below.counted > 0, "the genome body and trace count");
        assert_eq!((below.genomes, below.traces), (1, 1));
        assert_eq!(below.records, ["4095"]);
        let count = capped_run(&receiver, manual, |s| s.next_index = 4_096);
        assert_eq!(
            count,
            Capped {
                admitted: true,
                cap: Some("count".to_owned()),
                counted: 0,
                genomes: 0,
                traces: 0,
                records: vec!["4096".to_owned()],
            },
            "{manual}"
        );
        let below = capped_run(&receiver, manual, |s| s.bytes = 248 * MIB);
        assert!(below.admitted);
        assert_eq!(below.cap, None);
        assert_eq!((below.genomes, below.traces), (1, 1));
        let bytes = capped_run(&receiver, manual, |s| s.bytes = 248 * MIB + 1);
        assert_eq!(
            bytes,
            Capped {
                admitted: true,
                cap: Some("bytes".to_owned()),
                counted: 0,
                genomes: 0,
                traces: 0,
                records: vec!["0".to_owned()],
            },
            "{manual}"
        );
    }
}

#[test]
fn a_manual_sample_inside_the_interval_is_skipped_with_no_record() {
    let receiver = Receiver::start();
    let (telemetry, reports) = telemetry(&receiver, Limits::default(), 60_000);
    let config = config();
    let sim = seed_simulation(config.clone(), 3);
    let mut run = telemetry.begin_run(start(&config)).unwrap();
    let mut slot = None;
    telemetry.before_tick(&mut run, &sim, &mut slot, false);
    let sample = ActiveTrace::new(sim.creatures.keys().nth(1).unwrap(), 1);
    telemetry.start_manual(&mut run, &sim, &mut slot, None, &sample);
    assert!(run.samples.manual.is_none());
    assert_eq!(run.samples.skipped, 1);
    let id = run.id().to_owned();
    telemetry.end_run(run, EndStatus::Completed, sim.tick, Flush::Wait);
    // Only the window the manual sample ended records.
    assert_eq!(line(&reports, &id)["window_records"], "1");
}

#[test]
fn an_oversized_genome_body_and_window_item_are_dropped_counted_and_marked() {
    let receiver = Receiver::start();
    let limits = Limits {
        max_body_bytes: 256,
        ..Limits::default()
    };
    let (telemetry, reports) = telemetry(&receiver, limits, 10);
    let config = config();
    let mut sim = seed_simulation(config.clone(), 3);
    let mut run = telemetry.begin_run(start(&config)).unwrap();
    let mut slot = None;
    telemetry.before_tick(&mut run, &sim, &mut slot, false);
    let meta = run.samples.window.clone().unwrap();
    assert!(meta.root.iter().any(|kv| kv.key == "petri.genome_dropped"));
    run_tick(&mut sim, &mut slot);
    telemetry.end_samples(&mut run, &sim, &mut slot, None);
    let id = run.id().to_owned();
    telemetry.end_run(run, EndStatus::Completed, sim.tick, Flush::Wait);
    let fields = line(&reports, &id);
    assert_eq!(fields["windows"], "1");
    // `run.started` (its config body), the genome record, the window record
    // and the window.
    assert_eq!(fields["dropped"], "4", "{fields:?}");
    assert!(reports
        .lines()
        .iter()
        .any(|l| l.contains("record=creature.genome")));
    assert!(reports.lines().iter().any(|l| l.contains("record=window")));
    assert!(receiver.traces().is_empty());
}

#[test]
fn a_rejected_window_is_failed() {
    let receiver = Receiver::rejecting(1);
    let (telemetry, reports) = telemetry(&receiver, Limits::default(), 10);
    let config = config();
    let mut sim = seed_simulation(config.clone(), 3);
    let mut run = telemetry.begin_run(start(&config)).unwrap();
    let mut slot = None;
    tick_with_windows(&telemetry, &mut run, &mut sim, &mut slot, 2);
    let id = run.id().to_owned();
    telemetry.end_run(run, EndStatus::Completed, sim.tick, Flush::Wait);
    let fields = line(&reports, &id);
    assert_eq!(fields["windows"], "1");
    // Every logs request loses one record too; the window is one of the failed.
    let failed: u64 = fields["failed"].parse().unwrap();
    assert!(failed >= 2, "{fields:?}");
    assert!(roots(&receiver).is_empty());
}

#[test]
fn no_tick_trace_is_taken_at_tick_2_pow_63() {
    let receiver = Receiver::start();
    let (telemetry, reports) = telemetry(&receiver, Limits::default(), 10);
    let config = config();
    let mut sim = seed_simulation(config.clone(), 3);
    run_tick(&mut sim, &mut None);
    sim.tick = FIRST_UNTRACED_TICK;
    let mut seam = sim.stats.last_tick_phases.unwrap();
    seam.tick = FIRST_UNTRACED_TICK;
    sim.stats.last_tick_phases = Some(seam);
    let mut run = telemetry.begin_run(start(&config)).unwrap();
    assert!(telemetry.tick_snapshot(&mut run, &sim, crate::TickSample::RunEnd));
    let id = run.id().to_owned();
    telemetry.end_run(run, EndStatus::Completed, sim.tick, Flush::Wait);
    assert_eq!(line(&reports, &id)["traces"], "0");
}

#[test]
fn run_started_records_the_three_settings() {
    let receiver = Receiver::start();
    let (telemetry, _) = telemetry(&receiver, Limits::default(), 25);
    let config = config();
    let run = telemetry.begin_run(start(&config)).unwrap();
    telemetry.end_run(run, EndStatus::Completed, 0, Flush::Wait);
    let started = receiver
        .records()
        .into_iter()
        .find(|r| r.event_name == "run.started")
        .unwrap();
    assert_eq!(started.attribute("petri.creature_windows"), Some("on"));
    assert_eq!(started.attribute("petri.window_ticks"), Some("2"));
    assert_eq!(started.attribute("petri.window_interval_ms"), Some("25"));
    let ended = receiver
        .records()
        .into_iter()
        .find(|r| r.event_name == "run.ended")
        .unwrap();
    assert_eq!(ended.attribute("petri.samples_skipped"), Some("0"));
}
