use super::*;
use crate::bench::run::{build_report, report_json_pretty};
use crate::bench::schema::{undefined_temporal_memory_sensitivity, Indicator};
use crate::bench::tests::{small_profile, small_world_set_report};
use crate::UNDEFINED;

#[test]
fn lockfile_identity_selects_core_dependency_among_reordered_versions() {
    let core = "[[package]]\nname = \"v3-core\"\nversion = \"0.1.0\"\ndependencies = [\n \"rand 0.8.6\",\n]\n";
    let old = "[[package]]\nname = \"rand\"\nversion = \"0.8.6\"\n";
    let new = "[[package]]\nname = \"rand\"\nversion = \"0.9.5\"\n";
    for lock in [format!("{new}{old}{core}"), format!("{core}{old}{new}")] {
        assert_eq!(rand_version_from_lock(&lock), Some("0.8.6"));
    }
    let upgraded = format!("{old}{new}{}", core.replace("rand 0.8.6", "rand 0.9.5"));
    assert_eq!(rand_version_from_lock(&upgraded), Some("0.9.5"));
    let no_rand = core.replace("rand 0.8.6", "serde");
    assert_eq!(rand_version_from_lock(&format!("{old}{no_rand}")), None);
    let unversioned = core.replace("rand 0.8.6", "rand");
    assert_eq!(
        rand_version_from_lock(&format!("{old}{unversioned}")),
        Some("0.8.6")
    );
    assert_eq!(
        rand_version_from_lock(&format!("{old}{new}{unversioned}")),
        None
    );
}

/// The work-counter thresholds are strict: a delta exactly at 10 percent
/// is `ok` and one exactly at 50 percent is `flag`, not the next level up.
#[test]
fn counter_levels_are_strict_at_their_thresholds() {
    assert_eq!(counter_level(None), ComparisonLevel::Ok);
    assert_eq!(counter_level(Some(0.0)), ComparisonLevel::Ok);
    assert_eq!(counter_level(Some(-80.0)), ComparisonLevel::Ok);
    assert_eq!(
        counter_level(percent_delta(11.0, 10.0)),
        ComparisonLevel::Ok,
        "exactly +10 percent is inside the flag threshold"
    );
    assert_eq!(counter_level(Some(10.5)), ComparisonLevel::Flag);
    assert_eq!(
        counter_level(percent_delta(1.5, 1.0)),
        ComparisonLevel::Flag,
        "exactly +50 percent is inside the severe threshold"
    );
    assert_eq!(counter_level(Some(60.0)), ComparisonLevel::Severe);
}

fn case_reading<'a>(comparison: &'a CaseComparison, name: &str) -> &'a CaseReadingComparison {
    comparison
        .readings
        .iter()
        .find(|reading| reading.name == name)
        .unwrap_or_else(|| panic!("reading {name} must be compared"))
}

/// A world-set reference whose recipes were edited still compares: the
/// changed case is labeled `inputs_changed`, its deltas are computed, and a
/// case the reference never ran is recorded absent rather than dropped.
#[test]
fn world_set_comparison_labels_changed_inputs_and_records_absent_cases() {
    let mut current = small_world_set_report();
    let mut reference = current.clone();

    let edited_seed = current.deterministic.profile.cases[1].seed;
    reference.deterministic.profile.cases[1].config_digest = "sha256:edited".to_string();
    let population = |report: &mut Report, seed: u64, value: u64, extinction: Option<u64>| {
        for row in &mut report
            .deterministic
            .goal_indicators
            .population_persistence
            .per_seed
        {
            if row.seed == seed {
                row.final_population = value;
                row.extinction_tick = extinction;
            }
        }
    };
    population(&mut current, edited_seed, 200, Some(7));
    population(&mut reference, edited_seed, 100, Some(3));
    let dropped = current.deterministic.profile.cases[2].name.clone();
    reference.deterministic.profile.cases.remove(2);
    reference.deterministic.goal_indicators.cases.remove(2);

    let path = std::env::temp_dir().join(format!(
        "t12-f04-world-set-reference-{}.json",
        std::process::id()
    ));
    std::fs::write(&path, report_json_pretty(&reference)).expect("write the reference");
    let comparison = compare_against_path(&current, &path)
        .expect("a world-set reference whose case digests differ must still be comparable");
    let _ = std::fs::remove_file(&path);

    assert_eq!(comparison.cases.len(), 3);
    assert!(
        !comparison.cases[0].inputs_changed,
        "an untouched recipe is not an input change"
    );
    assert_eq!(
        comparison.cases[0].reference_digest.as_deref(),
        Some(comparison.cases[0].current_digest.as_str())
    );
    assert!(!comparison.cases[0].absent_in_reference);

    let edited = &comparison.cases[1];
    assert!(edited.inputs_changed);
    assert!(!edited.absent_in_reference);
    assert_eq!(edited.reference_digest.as_deref(), Some("sha256:edited"));
    let final_population = case_reading(edited, "final_population");
    assert_eq!(final_population.current.as_deref(), Some("200.000000"));
    assert_eq!(final_population.reference.as_deref(), Some("100.000000"));
    assert_eq!(
        final_population.percent_delta.as_deref(),
        Some("100.000000")
    );
    let extinction = case_reading(edited, "extinction_tick");
    assert_eq!(extinction.current.as_deref(), Some("7.000000"));
    assert_eq!(extinction.reference.as_deref(), Some("3.000000"));
    assert_eq!(
        extinction.percent_delta, None,
        "an extinction tick is a value, not a rate"
    );

    let absent = &comparison.cases[2];
    assert_eq!(absent.case, dropped);
    assert!(absent.absent_in_reference);
    assert!(!absent.inputs_changed);
    assert_eq!(absent.reference_digest, None);
    let reading = case_reading(absent, "final_population");
    assert!(reading.current.is_some());
    assert_eq!(reading.reference, None);
    assert_eq!(reading.percent_delta, None);
    assert!(
        !comparison.severe,
        "a per-case difference never makes a comparison severe by itself"
    );
}

/// A profile difference that is not the per-case block is still a hard
/// error, and a single-config profile records no cases at all.
#[test]
fn comparison_still_rejects_a_different_world_and_records_no_cases_off_the_world_set() {
    let current = small_world_set_report();
    let mut reference = current.clone();
    reference.deterministic.profile.ticks += 1;
    let path = std::env::temp_dir().join(format!(
        "t12-f04-world-set-mismatch-{}.json",
        std::process::id()
    ));
    std::fs::write(&path, report_json_pretty(&reference)).expect("write the reference");
    let error = compare_against_path(&current, &path).unwrap_err();
    let _ = std::fs::remove_file(&path);
    assert!(error.contains("different profile"), "{error}");

    let single = build_report(&small_profile("sweep"), "t12-f04-single-config-check")
        .expect("a valid profile");
    assert!(
        compare_against(&single, std::path::Path::new("reference.json"), &single)
            .cases
            .is_empty(),
        "only the world set has cases to compare"
    );
}

/// Every per-case reading comes from that case's own seed and its own
/// observation, is parsed from the report's own strings, and is
/// unmeasured — never zero — where its denominator is.
#[test]
fn case_readings_follow_the_case_seed_and_observation() {
    let mut report = small_world_set_report();
    let cases: Vec<(String, u64)> = report
        .deterministic
        .profile
        .cases
        .iter()
        .map(|case| (case.name.clone(), case.seed))
        .collect();
    let first_seed = cases[0].1;

    // Stamp every row with a value derived from its own seed, so a lookup
    // that takes some other case's row reads a different number. The first
    // case runs no creature-ticks, so its normalized counters are
    // unmeasured rather than a division by zero.
    for row in &mut report.deterministic.per_seed {
        row.creature_ticks = if row.seed == first_seed { 0 } else { 10 };
        row.mesh_hops = row.seed;
        row.births = row.seed * 2;
    }
    for row in &mut report
        .deterministic
        .goal_indicators
        .population_persistence
        .per_seed
    {
        row.final_population = row.seed + 1;
        row.minimum_population = row.seed;
        row.peak_population = row.seed + 2;
        row.plateau_population = Some(six(f64::from(row.seed as u32) / 2.0));
        row.mean_energy = Some(six(f64::from(row.seed as u32) / 4.0));
        row.extinction_tick = Some(row.seed + 3);
    }
    if let Indicator::Defined(lineage) = &mut report.deterministic.goal_indicators.lineage_diversity
    {
        for row in &mut lineage.per_seed {
            row.surviving_founder_clade_count = row.seed;
            row.shannon_entropy_nats = six(f64::from(row.seed as u32));
        }
    }
    if let Indicator::Defined(memory) = &mut report.deterministic.goal_indicators.memory_sensitivity
    {
        for row in &mut memory.per_seed {
            row.different_from_either_fraction = six(f64::from(row.seed as u32) / 100.0);
        }
    }
    for observation in &mut report.deterministic.goal_indicators.cases {
        let seed = observation.case.seed;
        // Three different multiples of the seed, so reading one cause's
        // total under another cause's name is visible. The first case
        // carries none at all, as a report stored before these totals did.
        observation.tracking.moves_blocked_total_by_cause =
            (seed != first_seed).then(|| MovesBlockedByCause {
                barrier: seed,
                occupied: seed * 3,
                out_of_bounds: seed * 5,
            });
        observation.fractions.typed_eat_share = vec![six(f64::from(seed as u32) / 50.0)];
        observation.fractions.blocked_move_fraction = six(f64::from(seed as u32) / 200.0);
        observation
            .fractions
            .barrier_blocked_fraction_by_reader_state = ByReaderState {
            has_barrier_reader: six(f64::from(seed as u32) / 400.0),
            no_barrier_reader: six(f64::from(seed as u32) / 500.0),
        };
        observation
            .fractions
            .avoidable_blocked_share_of_all_moves_by_reader_state = ByReaderState {
            has_barrier_reader: six(f64::from(seed as u32) / 800.0),
            no_barrier_reader: six(f64::from(seed as u32) / 1_100.0),
        };
        if let Indicator::Defined(drift) = &mut observation.drift_depth {
            // Two checkpoints with different readings, so selecting the
            // wrong depth is visible.
            let mut shallow = drift.readings[0].clone();
            shallow.depth = 1_000;
            shallow.changed_per_all_births = six(0.125);
            let mut deep = drift.readings[0].clone();
            deep.depth = 2_000;
            deep.changed_per_all_births = six(f64::from(seed as u32) / 400.0);
            drift.readings = vec![shallow, deep];
        }
    }

    let value = |case: &str, name: &str| {
        case_readings(&report, case)
            .into_iter()
            .find(|(reading, _)| reading == name)
            .unwrap_or_else(|| panic!("{name} must be a compared reading"))
            .1
    };
    let (second, seed) = (&cases[1].0, cases[1].1);
    let seeded = f64::from(seed as u32);
    assert_eq!(value(second, "final_population"), Some(seeded + 1.0));
    assert_eq!(value(second, "minimum_population"), Some(seeded));
    assert_eq!(value(second, "peak_population"), Some(seeded + 2.0));
    assert_eq!(value(second, "plateau_population"), Some(seeded / 2.0));
    assert_eq!(value(second, "mean_energy"), Some(seeded / 4.0));
    assert_eq!(value(second, "extinction_tick"), Some(seeded + 3.0));
    assert_eq!(value(second, "births"), Some(seeded * 2.0));
    assert_eq!(
        value(second, "mesh_hops_per_creature_tick"),
        Some(seeded / 10.0),
        "a counter is normalized by that case's own creature-ticks"
    );
    assert_eq!(
        value(&cases[0].0, "mesh_hops_per_creature_tick"),
        None,
        "no creature-ticks makes a normalized counter unmeasured, not zero"
    );
    assert_eq!(value(second, "surviving_founder_clade_count"), Some(seeded));
    assert_eq!(value(second, "lineage_shannon_entropy_nats"), Some(seeded));
    assert_eq!(
        value(second, "memory_different_from_either_fraction"),
        Some(seeded / 100.0)
    );
    assert_eq!(value(second, "typed_eat_share_type_0"), Some(seeded / 50.0));
    assert_eq!(value(second, "blocked_move_fraction"), Some(seeded / 200.0));
    assert_eq!(
        value(second, "barrier_blocked_fraction_has_barrier_reader"),
        Some(seeded / 400.0)
    );
    assert_eq!(
        value(second, "barrier_blocked_fraction_no_barrier_reader"),
        Some(seeded / 500.0)
    );
    assert_eq!(
        value(
            second,
            "avoidable_blocked_share_of_all_moves_has_barrier_reader"
        ),
        Some(seeded / 800.0)
    );
    assert_eq!(
        value(
            second,
            "avoidable_blocked_share_of_all_moves_no_barrier_reader"
        ),
        Some(seeded / 1_100.0)
    );
    assert_eq!(value(second, "moves_blocked_barrier_total"), Some(seeded));
    assert_eq!(
        value(second, "moves_blocked_occupied_total"),
        Some(seeded * 3.0)
    );
    assert_eq!(
        value(second, "moves_blocked_out_of_bounds_total"),
        Some(seeded * 5.0)
    );
    assert_eq!(
        value(&cases[0].0, "moves_blocked_occupied_total"),
        None,
        "a report that never carried the by-cause totals is unmeasured, not zero"
    );
    assert_eq!(
        value(second, "drift_changed_per_all_births_at_2000"),
        Some(seeded / 400.0),
        "the depth-2,000 checkpoint, not the depth-1,000 one"
    );
    assert!(
        case_readings(&report, "a world no report ran").is_empty(),
        "an unknown case has no readings at all"
    );
}

/// The three temporal memory fractions are compared per case by their fixed
/// names, read from the outer row whose seed is the case seed, and sit
/// immediately after `memory_different_from_either_fraction`.
#[test]
fn temporal_readings_follow_the_outer_row_seed_and_sit_after_memory() {
    let mut report = small_world_set_report();
    let Indicator::Defined(temporal) = &mut report
        .deterministic
        .goal_indicators
        .temporal_memory_sensitivity
    else {
        panic!("world set defines temporal memory sensitivity");
    };
    // Three different divisors per component, and the inner component
    // seeds deliberately mismatched, so the reading is proven to follow
    // the outer row's seed and the right component.
    for row in &mut temporal.per_seed {
        let seeded = f64::from(row.seed as u32);
        row.previous_slots.seed = row.seed + 100;
        row.previous_slots.different_from_either_fraction = six(seeded / 8.0);
        row.persisted_outputs.seed = row.seed + 200;
        row.persisted_outputs.different_from_either_fraction = six(seeded / 16.0);
        row.operator_state.seed = row.seed + 300;
        row.operator_state.different_from_either_fraction = six(seeded / 32.0);
    }

    let case = &report.deterministic.profile.cases[1];
    let (name, seeded) = (case.name.clone(), f64::from(case.seed as u32));
    let readings = case_readings(&report, &name);
    let value = |reading: &str| {
        readings
            .iter()
            .find(|(candidate, _)| candidate == reading)
            .unwrap_or_else(|| panic!("{reading} must be a compared reading"))
            .1
    };
    assert_eq!(
        value("temporal_memory_previous_slots_different_from_either_fraction"),
        Some(seeded / 8.0)
    );
    assert_eq!(
        value("temporal_memory_persisted_outputs_different_from_either_fraction"),
        Some(seeded / 16.0)
    );
    assert_eq!(
        value("temporal_memory_operator_state_different_from_either_fraction"),
        Some(seeded / 32.0)
    );
    let names: Vec<&str> = readings.iter().map(|(name, _)| name.as_str()).collect();
    let memory_index = names
        .iter()
        .position(|name| *name == "memory_different_from_either_fraction")
        .expect("memory reading is compared");
    assert_eq!(
        &names[memory_index + 1..memory_index + 4],
        &[
            "temporal_memory_previous_slots_different_from_either_fraction",
            "temporal_memory_persisted_outputs_different_from_either_fraction",
            "temporal_memory_operator_state_different_from_either_fraction",
        ],
        "the temporal readings sit immediately after the memory reading"
    );
}

/// A report whose temporal memory indicator is `Undefined` still lists the
/// three temporal readings, each unmeasured, so a comparison against a
/// report that did measure them labels the gap instead of dropping it.
#[test]
fn temporal_readings_are_unmeasured_when_the_indicator_is_undefined() {
    let mut report = small_world_set_report();
    report
        .deterministic
        .goal_indicators
        .temporal_memory_sensitivity = undefined_temporal_memory_sensitivity();
    let case = report.deterministic.profile.cases[0].name.clone();
    let readings = case_readings(&report, &case);
    for name in [
        "temporal_memory_previous_slots_different_from_either_fraction",
        "temporal_memory_persisted_outputs_different_from_either_fraction",
        "temporal_memory_operator_state_different_from_either_fraction",
    ] {
        let (_, value) = readings
            .iter()
            .find(|(reading, _)| reading == name)
            .unwrap_or_else(|| panic!("{name} must be a compared reading"));
        assert_eq!(*value, None, "{name} is unmeasured, not dropped or zero");
    }
}

/// Comparing a report against its own output path — whether the path was
/// given explicitly or spelled differently — records the self-reference
/// cause and no comparison entry. A different explicit reference alongside
/// the output path is still compared, and an empty explicit selection names
/// its own cause.
#[test]
fn self_reference_is_skipped_and_its_absence_recorded() {
    let scratch_dir =
        std::env::temp_dir().join(format!("t14-f01-self-reference-{}", std::process::id()));
    std::fs::create_dir_all(&scratch_dir).expect("create scratch directory");
    let out_path = scratch_dir.join("this-report.json");
    let mut report =
        build_report(&small_profile("gate"), "t14-f01-self-reference").expect("a valid profile");
    let self_cause = format!(
        "the only candidate reference is this report's own output path {}",
        out_path.display()
    );

    // The output file does not exist yet: a `--baseline` naming it through
    // a `..` component matches by lexical normalization.
    let spelled_differently = scratch_dir
        .join("elsewhere")
        .join("..")
        .join("this-report.json");
    assert!(!out_path.exists());
    let explicit = ReferenceSelection {
        paths: vec![spelled_differently],
        absence: None,
    };
    let severe = apply_comparisons(&mut report, &explicit, &out_path)
        .expect("a skipped self-reference is never an error");
    assert!(!severe);
    assert!(report.comparison.references.is_empty());
    assert_eq!(
        report.comparison.reference_absence.as_deref(),
        Some(self_cause.as_str())
    );
    assert!(!report.comparison.severe);

    // The output file exists from an earlier run: canonical paths match,
    // and a genuine reference next to it is still compared.
    std::fs::write(&out_path, report_json_pretty(&report)).expect("write earlier report");
    let other = scratch_dir.join("other.json");
    std::fs::write(&other, report_json_pretty(&report)).expect("write other reference");
    let mixed = ReferenceSelection {
        paths: vec![other.clone(), out_path.clone()],
        absence: None,
    };
    apply_comparisons(&mut report, &mixed, &out_path).expect("the other reference parses");
    assert_eq!(report.comparison.references.len(), 1);
    assert_eq!(
        report.comparison.references[0].path,
        other.display().to_string()
    );
    assert_eq!(report.comparison.reference_absence, None);

    let only_self = ReferenceSelection {
        paths: vec![out_path.clone()],
        absence: None,
    };
    apply_comparisons(&mut report, &only_self, &out_path).expect("nothing to compare");
    assert!(report.comparison.references.is_empty());
    assert_eq!(
        report.comparison.reference_absence.as_deref(),
        Some(self_cause.as_str())
    );

    apply_comparisons(&mut report, &ReferenceSelection::default(), &out_path)
        .expect("nothing to compare");
    assert!(report.comparison.references.is_empty());
    assert_eq!(
        report.comparison.reference_absence.as_deref(),
        Some("no reference paths were given")
    );

    // The absence field is serialized only when set, so a report with
    // references never carries it and a historical block loads as `None`.
    let json = serde_json::to_value(&report.comparison).unwrap();
    assert_eq!(
        json["reference_absence"],
        serde_json::Value::String("no reference paths were given".to_string())
    );
    let historical: Comparison =
        serde_json::from_str(r#"{"references":[],"severe":false}"#).unwrap();
    assert_eq!(historical.reference_absence, None);
    assert!(!serde_json::to_string(&historical)
        .unwrap()
        .contains("reference_absence"));
    std::fs::remove_dir_all(&scratch_dir).expect("remove scratch directory");
}

/// `Undefined` is the report's no-denominator value and must not parse as
/// a number a delta would then compare against.
#[test]
fn an_undefined_reading_parses_as_unmeasured() {
    assert_eq!(parse_reading(&six(0.25)), Some(0.25));
    assert_eq!(parse_reading(UNDEFINED), None);
    assert_eq!(parse_reading(""), None);
}
