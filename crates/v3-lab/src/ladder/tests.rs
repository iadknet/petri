use super::*;
use proptest::prelude::*;

#[test]
fn the_adequacy_minimum_is_the_rule_of_three_count_at_the_defaults() {
    let rates = StallRates::default();
    let minima: Vec<u64> = Rung::COUNTED
        .iter()
        .map(|&rung| adequacy(rates.rate(rung).unwrap()))
        .collect();
    assert_eq!(minima, [300, 60, 60, 15]);
    assert_eq!(rates.rate(Rung::Exposure), None);
}

#[test]
fn a_status_is_inconclusive_below_the_minimum_then_a_rate_against_rho() {
    assert_eq!(status(0, 14, 0.2), Status::Inconclusive);
    assert_eq!(status(14, 14, 0.2), Status::Inconclusive);
    assert_eq!(status(0, 15, 0.2), Status::Fail);
    assert_eq!(status(2, 15, 0.2), Status::Fail);
    assert_eq!(status(3, 15, 0.2), Status::Pass, "s / n = ρ passes");
    assert_eq!(status(3, 300, 0.01), Status::Pass);
    assert_eq!(status(2, 300, 0.01), Status::Fail);
}

proptest! {
    #[test]
    fn status_follows_the_adequacy_minimum_and_the_share(
        rate in 0.01f64..0.99,
        trials in 0u64..2_000,
        fraction in 0.0f64..=1.0,
    ) {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
        let successes = (fraction * trials as f64).floor() as u64;
        let got = status(successes, trials, rate);
        let minimum = (3.0 / rate).ceil();
        #[allow(clippy::cast_precision_loss)]
        let expected = if (trials as f64) < minimum {
            Status::Inconclusive
        } else if successes as f64 / trials as f64 >= rate {
            Status::Pass
        } else {
            Status::Fail
        };
        prop_assert_eq!(got, expected);
        // Zero successes at the adequacy minimum always fail.
        prop_assert_eq!(status(0, adequacy(rate), rate), Status::Fail);
    }
}

#[test]
fn stall_rates_parse_four_values_each_in_the_open_unit_interval() {
    assert_eq!(
        StallRates::from_slice(&[0.01, 0.05, 0.05, 0.2]),
        Some(StallRates::default())
    );
    assert_eq!(StallRates::from_slice(&[0.1, 0.1, 0.1]), None);
    for bad in [0.0, 1.0, -0.5, f64::NAN] {
        assert_eq!(StallRates::from_slice(&[0.1, bad, 0.1, 0.1]), None, "{bad}");
    }
}

/// `(s, n)` per counted rung.
type Counts = [(u64, u64); 4];

/// A replicate whose pooled counts give `(s, n)` per counted rung.
fn counted(counts: [(u64, u64); 4], depth: usize) -> Pooled {
    let mut pooled = Pooled::new(depth);
    pooled.supply.touching_births = counts[0].0;
    pooled.supply.births = counts[0].1;
    pooled.children.touching.viable = counts[1].0;
    pooled.children.touching.children = counts[1].1;
    pooled.children.touching.outcomes.improved = counts[2].0;
    assert_eq!(counts[2].1, counts[1].0, "benefit's n is viability's s");
    let (retained, trials) = counts[3];
    pooled.retention.depths[depth - 1] = [retained, trials - retained, 0];
    pooled
}

const PASS: [(u64, u64); 4] = [(100, 400), (60, 100), (10, 60), (5, 15)];

fn replicate(counts: [(u64, u64); 4], index: u32, incomplete: bool) -> ReplicateLadder {
    ReplicateLadder::new(
        index,
        incomplete,
        counted(counts, 2),
        &StallRates::default(),
    )
}

#[test]
fn synthetic_counts_place_the_stall_at_each_rung_or_at_inconclusive() {
    assert_eq!(replicate(PASS, 0, false).first_not_pass, None);
    let cases: [(Counts, Rung, Status); 8] = [
        (
            [(2, 400), (60, 100), (10, 60), (5, 15)],
            Rung::Supply,
            Status::Fail,
        ),
        (
            [(2, 299), (60, 100), (10, 60), (5, 15)],
            Rung::Supply,
            Status::Inconclusive,
        ),
        (
            [(100, 400), (4, 100), (1, 4), (5, 15)],
            Rung::Viability,
            Status::Fail,
        ),
        (
            [(100, 400), (59, 59), (10, 59), (5, 15)],
            Rung::Viability,
            Status::Inconclusive,
        ),
        (
            [(100, 400), (60, 100), (2, 60), (5, 15)],
            Rung::Benefit,
            Status::Fail,
        ),
        (
            [(100, 400), (59, 100), (10, 59), (5, 15)],
            Rung::Benefit,
            Status::Inconclusive,
        ),
        (
            [(100, 400), (60, 100), (10, 60), (2, 15)],
            Rung::Retention,
            Status::Fail,
        ),
        (
            [(100, 400), (60, 100), (10, 60), (5, 14)],
            Rung::Retention,
            Status::Inconclusive,
        ),
    ];
    for (counts, rung, expected) in cases {
        let ladder = replicate(counts, 0, false);
        assert_eq!(ladder.first_not_pass, Some(rung), "{counts:?}");
        assert_eq!(ladder.status(rung), Some(expected), "{counts:?}");
        // Every rung is computed, the later ones included.
        assert_eq!(ladder.statuses.len(), 4);
    }
}

#[test]
fn retention_reads_depth_d() {
    let mut pooled = Pooled::new(3);
    pooled.retention.depths = vec![[9, 0, 0], [9, 0, 0], [1, 2, 3]];
    assert_eq!(pooled.counts(Rung::Retention), Some((1, 6)));
}

const BENEFIT_FAIL: [(u64, u64); 4] = [(100, 400), (60, 100), (2, 60), (5, 15)];
const BENEFIT_INCONCLUSIVE: [(u64, u64); 4] = [(100, 400), (59, 100), (10, 59), (5, 15)];
const RETENTION_INCONCLUSIVE: [(u64, u64); 4] = [(100, 400), (60, 100), (10, 60), (5, 14)];
const SUPPLY_FAIL: [(u64, u64); 4] = [(2, 400), (60, 100), (10, 60), (5, 15)];

fn verdict(replicates: &[ReplicateLadder]) -> ArmVerdict {
    ArmVerdict::of(replicates, &vec![Some(false); replicates.len()])
}

#[test]
fn an_all_inconclusive_tally_is_never_a_stall() {
    let all: Vec<_> = (0..4)
        .map(|i| replicate(RETENTION_INCONCLUSIVE, i, false))
        .collect();
    let v = verdict(&all);
    assert_eq!(
        v.text(),
        "inconclusive at retention: fail 0 / inconclusive 4 of 4"
    );
    assert_eq!(v.route(), Some("T13 / T14"));
}

#[test]
fn a_mixed_tally_stalls_only_on_a_fail_majority_of_the_replicates_naming_the_rung() {
    let three_one = [
        replicate(BENEFIT_FAIL, 0, false),
        replicate(BENEFIT_FAIL, 1, false),
        replicate(BENEFIT_FAIL, 2, false),
        replicate(BENEFIT_INCONCLUSIVE, 3, false),
    ];
    let v = verdict(&three_one);
    assert_eq!(v.text(), "stalls at benefit: fail 3 / inconclusive 1 of 4");
    assert_eq!(
        v.route(),
        Some("the scorer (T22) or, in the world, the pressure tracks")
    );
    let two_two = [
        replicate(BENEFIT_FAIL, 0, false),
        replicate(BENEFIT_FAIL, 1, false),
        replicate(BENEFIT_INCONCLUSIVE, 2, false),
        replicate(BENEFIT_INCONCLUSIVE, 3, false),
    ];
    assert_eq!(
        verdict(&two_two).text(),
        "inconclusive at benefit: fail 2 / inconclusive 2 of 4"
    );
}

#[test]
fn the_modal_rung_ties_to_the_earliest_and_passing_replicates_do_not_vote() {
    let tie = [
        replicate(BENEFIT_FAIL, 0, false),
        replicate(SUPPLY_FAIL, 1, false),
        replicate(PASS, 2, false),
    ];
    let v = verdict(&tie);
    assert_eq!(v.rung, Some(Rung::Supply));
    assert_eq!(v.text(), "stalls at supply: fail 1 / inconclusive 0 of 1");
    let modal = [
        replicate(BENEFIT_FAIL, 0, false),
        replicate(SUPPLY_FAIL, 1, false),
        replicate(BENEFIT_FAIL, 2, false),
    ];
    assert_eq!(verdict(&modal).rung, Some(Rung::Benefit));
}

#[test]
fn no_stall_needs_every_completed_replicate_to_pass_and_prints_reach() {
    let all = [replicate(PASS, 0, false), replicate(PASS, 1, false)];
    let v = ArmVerdict::of(&all, &[Some(true), Some(false)]);
    assert_eq!(v.kind, VerdictKind::NoStall);
    assert_eq!(v.text(), "no stall: reached 1/2");
    assert_eq!(v.route(), None);
    assert_eq!(
        verdict(&all).text(),
        "no stall: reached 0/2, censored at the horizon"
    );
}

#[test]
fn incomplete_replicates_are_excluded_and_label_the_verdict_partial() {
    let none = [replicate(PASS, 0, true), replicate(SUPPLY_FAIL, 1, true)];
    let v = verdict(&none);
    assert_eq!(v.text(), "inconclusive, partial (2 incomplete)");
    assert_eq!((v.rung, v.route()), (None, None));
    let some = [
        replicate(SUPPLY_FAIL, 0, true),
        replicate(BENEFIT_FAIL, 1, false),
    ];
    assert_eq!(
        verdict(&some).text(),
        "stalls at benefit: fail 1 / inconclusive 0 of 1, partial (1 incomplete)"
    );
}

#[test]
fn the_arm_tallies_replicate_statuses_with_a_wilson_interval_on_the_fail_share() {
    let replicates = vec![
        replicate(BENEFIT_FAIL, 0, false),
        replicate(BENEFIT_FAIL, 1, false),
        replicate(BENEFIT_INCONCLUSIVE, 2, false),
        replicate(PASS, 3, false),
        replicate(SUPPLY_FAIL, 4, true),
    ];
    let arm = ArmLadder::new(
        "native".into(),
        Role::Reference,
        Policy::Native,
        replicates,
        &[Some(false); 5],
        2,
    );
    let benefit = &arm.tallies[2];
    assert_eq!(benefit.rung, Rung::Benefit);
    assert_eq!(
        (benefit.pass, benefit.fail, benefit.inconclusive),
        (1, 2, 1)
    );
    assert_eq!(benefit.fail_wilson_95, crate::stats::wilson_95(2, 4));
    assert_eq!(arm.incomplete_replicates, 1);
    assert!(!arm.diagnostic);
    // Pooled over the completed replicates only.
    assert_eq!(arm.pooled.supply.births, 1_600);
    let deviation = ArmLadder::new(
        "mutation-off".into(),
        Role::Control,
        Policy::PolicyDeviation,
        Vec::new(),
        &[],
        2,
    );
    assert!(deviation.diagnostic);
    assert_eq!(
        deviation.verdict.text(),
        "inconclusive, partial (0 incomplete)"
    );
}

fn outcomes_of(children: &[(f64, f64, bool)]) -> Outcomes {
    let mut outcomes = Outcomes::default();
    for &(child, parent, progress) in children {
        outcomes.add(child, parent, progress);
    }
    outcomes
}

#[test]
fn outcomes_count_improved_worse_and_the_largest_improvement() {
    let o = outcomes_of(&[
        (2.0, 1.0, true),
        (1.0, 1.0, false),
        (0.5, 1.0, true),
        (4.0, 1.5, false),
        (0.25, 1.0, false),
    ]);
    assert_eq!((o.improved, o.worse, o.progress_improved), (2, 2, 2));
    assert_eq!(o.delta_sum, 1.0 + 0.0 - 0.5 + 2.5 - 0.75);
    assert_eq!(o.delta_max, Some(2.5));
    assert_eq!(outcomes_of(&[(0.0, 1.0, false)]).delta_max, None);
}

fn child() -> impl Strategy<Value = (f64, f64, bool)> {
    // Quarter steps keep the sums exact whatever the fold order.
    (0i32..40, 0i32..40, any::<bool>())
        .prop_map(|(c, p, g)| (f64::from(c) / 4.0, f64::from(p) / 4.0, g))
}

proptest! {
    #[test]
    fn pooling_split_outcomes_equals_folding_them_all(
        children in prop::collection::vec(child(), 0..30),
        split in 0usize..30,
    ) {
        let split = split.min(children.len());
        let mut pooled = outcomes_of(&children[..split]);
        pooled.merge(&outcomes_of(&children[split..]));
        prop_assert_eq!(pooled, outcomes_of(&children));
    }

    #[test]
    fn the_verdict_stalls_iff_most_replicates_naming_its_rung_fail(
        picks in prop::collection::vec(0usize..6, 1..10),
    ) {
        let kinds = [PASS, SUPPLY_FAIL, BENEFIT_FAIL, BENEFIT_INCONCLUSIVE, RETENTION_INCONCLUSIVE, [(2, 299), (60, 100), (10, 60), (5, 15)]];
        let replicates: Vec<ReplicateLadder> = picks
            .iter()
            .enumerate()
            .map(|(i, &k)| replicate(kinds[k], u32::try_from(i).unwrap(), false))
            .collect();
        let v = verdict(&replicates);
        match v.rung {
            None => {
                prop_assert_eq!(v.kind, VerdictKind::NoStall);
                prop_assert!(replicates.iter().all(|r| r.first_not_pass.is_none()));
            }
            Some(rung) => {
                let naming: Vec<&ReplicateLadder> =
                    replicates.iter().filter(|r| r.first_not_pass == Some(rung)).collect();
                let fail = naming.iter().filter(|r| r.status(rung) == Some(Status::Fail)).count();
                prop_assert_eq!(v.of as usize, naming.len());
                prop_assert_eq!(v.fail as usize, fail);
                prop_assert_eq!(v.fail + v.inconclusive, v.of);
                prop_assert_eq!(v.kind == VerdictKind::Stalls, 2 * fail > naming.len());
                // No rung is named by more replicates; ties go earlier.
                for other in Rung::COUNTED {
                    let count = replicates.iter().filter(|r| r.first_not_pass == Some(other)).count();
                    prop_assert!(count < naming.len() || (count == naming.len() && other >= rung));
                }
            }
        }
    }

    #[test]
    fn a_ladder_row_round_trips_through_json(
        births in 0u64..1_000,
        touching in 0u64..1_000,
        reference in prop::option::of(0i32..10_000),
        children in prop::collection::vec(child(), 0..10),
        keys in prop::collection::btree_map("[A-Za-z]{1,12}", 0u64..50, 0..6),
        depths in prop::collection::vec(prop::array::uniform3(0u64..9), 1..9),
    ) {
        let row = LadderRow {
            supply: Supply {
                births,
                touching_births: touching,
                sites: births * 2,
                reachable: births * 3,
                uniform_reference: reference.map(|r| f64::from(r) / 7.0),
                targeted: Targeted {
                    requested: keys.clone(),
                    applied: keys.clone(),
                    skipped: keys.clone(),
                    skipped_by_reason: keys,
                },
                discarded: 1,
                created: 2,
                removed: 3,
            },
            children: Children {
                touching: TouchingChildren {
                    children: 5,
                    scene_changed: 4,
                    silent: 1,
                    changed: 3,
                    dead: 1,
                    viable: 3,
                    outcomes: outcomes_of(&children),
                },
                other: OtherChildren {
                    children: 2,
                    scene_changed: 1,
                    outcomes: outcomes_of(&children),
                },
            },
            retention: RetentionRow {
                selected: 2,
                selected_touching: 1,
                depths_touching: depths.clone(),
                depths,
            },
        };
        let text = serde_json::to_string(&row).unwrap();
        prop_assert_eq!(serde_json::from_str::<LadderRow>(&text).unwrap(), row);
    }
}

#[test]
fn supply_pooling_keeps_the_uniform_reference_null_once_a_parent_had_no_reachable_node() {
    let block = |births, reference| Supply {
        births,
        uniform_reference: reference,
        ..Supply::default()
    };
    let mut pooled = Supply::default();
    pooled.add(&block(0, None));
    pooled.add(&block(2, Some(1.5)));
    assert_eq!(pooled.uniform_reference, Some(1.5));
    pooled.add(&block(0, None));
    pooled.add(&block(1, Some(0.5)));
    assert_eq!(pooled.uniform_reference, Some(2.0));
    pooled.add(&block(1, None));
    pooled.add(&block(1, Some(0.5)));
    assert_eq!((pooled.births, pooled.uniform_reference), (5, None));
}

fn calibration(verdict: Verdict) -> Calibration {
    use crate::summary::{CalibrationPoint, PointMeans, Selected};
    let means = PointMeans {
        founder: 0.5,
        floor: 0.2,
        half: 0.25,
        oracle: 0.9,
        comparator: 0.7,
        floor_progress: 0.1,
        comparator_progress: 0.3,
        floor_blocked_fraction: 0.0,
        comparator_blocked_fraction: 0.0,
        floor_efficiency: 0.0,
        half_efficiency: 0.0,
        oracle_efficiency: 0.0,
        comparator_efficiency: 0.0,
        comparator_wins: 5,
    };
    let point = |exposure: bool, means: Option<PointMeans>| CalibrationPoint {
        food_fraction: Some(0.04),
        scale: None,
        lifetime: 200,
        scenes: 16,
        redraws: if exposure {
            vec![0; 16]
        } else {
            vec![0, 1, 100]
        },
        exposure,
        means,
        competence: false,
        sensitivity: false,
        validation_competence: None,
        validation: None,
    };
    Calibration {
        margin: 1.0,
        points: vec![point(false, None), point(true, Some(means.clone()))],
        selected: (verdict == Verdict::Calibrated).then_some(Selected {
            food_fraction: Some(0.04),
            scale: None,
            lifetime: 200,
            means,
        }),
        verdict,
        reach_threshold: None,
        reach_threshold_source: None,
    }
}

#[test]
fn an_uncalibrated_exposure_names_every_failing_check_with_its_counts() {
    let exposure = Exposure::of(&calibration(Verdict::Uncalibrated), false);
    assert_eq!(exposure.status, Status::Fail);
    let [unexposed, exposed] = &exposure.failing[..] else {
        panic!("two points");
    };
    assert_eq!((unexposed.exposure, unexposed.exposure_scenes), (false, 2));
    assert_eq!(unexposed.comparator_wins, None);
    assert_eq!(exposed.exposure_scenes, 16);
    assert_eq!(
        (exposed.comparator_wins, exposed.wins_needed),
        (Some(5), 12)
    );
    let gaps = exposed.sensitivity_gaps.unwrap();
    assert!((gaps[0] - 0.05).abs() < 1e-12 && (gaps[1] - 0.65).abs() < 1e-12);
    let mut out = String::new();
    exposure_lines(&mut out, &exposure);
    assert!(out.contains("exposure (2/16 scenes drawn)"), "{out}");
    assert!(
        out.contains("competence (comparator wins 5/16 needed 12, gap 0.500 vs margin 1)"),
        "{out}"
    );
    assert!(out.contains("sensitivity (half−floor 0.050"), "{out}");

    let calibrated = Exposure::of(&calibration(Verdict::Calibrated), true);
    assert_eq!(calibrated.status, Status::Pass);
    assert!(calibrated.failing.is_empty());
    assert_eq!(calibrated.selected.unwrap().lifetime, 200);
}

#[test]
fn routes_name_the_owning_track_of_each_rung() {
    assert_eq!(Rung::Exposure.route(), "the assay (T22): instrument");
    assert_eq!(Rung::Supply.route(), "T11 / T13");
    assert_eq!(Rung::Viability.route(), "T11 / T17");
    assert_eq!(Rung::Retention.route(), "T13 / T14");
}

#[test]
fn a_status_name_is_its_serialized_label() {
    for status in [Status::Pass, Status::Fail, Status::Inconclusive] {
        let serialized = serde_json::to_value(status).unwrap();
        assert_eq!(serialized.as_str(), Some(status.name()));
    }
    assert_eq!(Status::Pass.name(), "pass");
}

fn keyed(pairs: &[(&str, u64)]) -> Keyed {
    pairs.iter().map(|&(k, v)| (k.to_owned(), v)).collect()
}

fn supply_block(base: u64) -> Supply {
    Supply {
        births: base,
        touching_births: base + 1,
        sites: base + 2,
        reachable: base + 3,
        uniform_reference: Some(1.0),
        targeted: Targeted {
            requested: keyed(&[("a", base)]),
            applied: keyed(&[("a", base + 1)]),
            skipped: keyed(&[("a", base + 2)]),
            skipped_by_reason: keyed(&[("r", base + 3)]),
        },
        discarded: base + 4,
        created: base + 5,
        removed: base + 6,
    }
}

#[test]
fn supply_pooling_sums_every_count_and_adds_keyed_counts_by_key() {
    // Arrange: distinct counts, and a key only the second block fired.
    let mut pooled = supply_block(2);
    let mut other = supply_block(3);
    other.targeted.requested.insert("new".into(), 4);

    // Act
    pooled.add(&other);

    // Assert
    let expected = Supply {
        births: 5,
        touching_births: 7,
        sites: 9,
        reachable: 11,
        uniform_reference: Some(2.0),
        targeted: Targeted {
            requested: keyed(&[("a", 5), ("new", 4)]),
            applied: keyed(&[("a", 7)]),
            skipped: keyed(&[("a", 9)]),
            skipped_by_reason: keyed(&[("r", 11)]),
        },
        discarded: 13,
        created: 15,
        removed: 17,
    };
    assert_eq!(pooled, expected);
}

fn children_block(base: u64) -> Children {
    Children {
        touching: TouchingChildren {
            children: base,
            scene_changed: base + 1,
            silent: base + 2,
            changed: base + 3,
            dead: base + 4,
            viable: base + 5,
            outcomes: outcomes_of(&[(2.0, 1.0, true)]),
        },
        other: OtherChildren {
            children: base + 6,
            scene_changed: base + 7,
            outcomes: outcomes_of(&[(0.5, 1.0, false)]),
        },
    }
}

#[test]
fn children_pooling_sums_every_class_count_and_merges_the_outcomes() {
    let mut pooled = children_block(2);

    pooled.add(&children_block(3));

    let t = &pooled.touching;
    assert_eq!(
        [
            t.children,
            t.scene_changed,
            t.silent,
            t.changed,
            t.dead,
            t.viable
        ],
        [5, 7, 9, 11, 13, 15]
    );
    assert_eq!((t.outcomes.improved, t.outcomes.delta_sum), (2, 2.0));
    let o = &pooled.other;
    assert_eq!([o.children, o.scene_changed], [17, 19]);
    assert_eq!((o.outcomes.worse, o.outcomes.delta_sum), (2, -1.0));
}

fn ladder_row(base: u64) -> LadderRow {
    LadderRow {
        supply: supply_block(base),
        children: children_block(base),
        retention: RetentionRow {
            selected: base,
            selected_touching: base + 1,
            depths: vec![[base, base + 1, base + 2], [base + 3, base + 4, base + 5]],
            depths_touching: vec![[base + 6, base + 7, base + 8], [base + 9, 0, 1]],
        },
    }
}

#[test]
fn pooling_rows_then_replicates_sums_supply_children_and_retention() {
    // Arrange: two replicates of one written row each.
    let mut first = Pooled::new(2);
    first.add(&ladder_row(2));
    first.retention.censored = vec![2, 3];
    first.retention.censored_touching = vec![4, 5];
    let mut second = Pooled::new(2);
    second.add(&ladder_row(3));
    second.retention.censored = vec![3, 4];
    second.retention.censored_touching = vec![5, 6];

    // Act
    let mut pooled = Pooled::new(2);
    pooled.merge(&first);
    pooled.merge(&second);
    pooled.add(&ladder_row(4));

    // Assert
    assert_eq!((pooled.supply.births, pooled.supply.removed), (9, 27));
    assert_eq!(pooled.children.touching.viable, 7 + 8 + 9);
    let r = &pooled.retention;
    assert_eq!((r.selected, r.selected_touching), (9, 12));
    assert_eq!(r.depths, vec![[9, 12, 15], [18, 21, 24]]);
    assert_eq!(r.depths_touching, vec![[27, 30, 33], [36, 0, 3]]);
    assert_eq!(
        (r.censored.as_slice(), r.censored_touching.as_slice()),
        (&[5, 7][..], &[9, 11][..])
    );
}

#[test]
fn rung_context_prints_each_rungs_pooled_counts_means_and_shares() {
    let mut pooled = Pooled::new(2);
    pooled.add(&ladder_row(2));
    pooled.children.other.outcomes.improved = 2;
    pooled.retention.censored = vec![1, 0];

    assert_eq!(rung_context(&pooled, Rung::Exposure), "");
    assert_eq!(
        rung_context(&pooled, Rung::Supply),
        "targeted requested {a 2}, applied {a 3}, skipped {a 4}, skipped by reason {r 5}; \
         discarded 6, created 7, removed 8; mean sites 2.000, mean reachable 2.500; \
         uniform reference 1.000"
    );
    assert_eq!(
        rung_context(&pooled, Rung::Viability),
        "silent 4, changed 5, dead 6, scene_changed 3"
    );
    assert_eq!(
        rung_context(&pooled, Rung::Benefit),
        "progress_improved 1, worse 0, delta_mean 0.143, delta_max 1.000; \
         non-touching improved 2/8 = 0.250"
    );
    assert_eq!(
        rung_context(&pooled, Rung::Retention),
        "selected 2 (3 touching); \
         all: d1 retained 2 deleted 3 lineage_loss 4 censored 1; \
         d2 retained 5 deleted 6 lineage_loss 7 censored 0; \
         touching: d1 retained 8 deleted 9 lineage_loss 10 censored 0; \
         d2 retained 11 deleted 0 lineage_loss 1 censored 0"
    );
}

#[test]
fn rung_context_prints_null_means_and_shares_with_no_births_or_children() {
    let empty = Pooled::new(1);

    let supply = rung_context(&empty, Rung::Supply);
    let benefit = rung_context(&empty, Rung::Benefit);

    assert!(
        supply.starts_with("targeted requested {}, applied {}"),
        "{supply}"
    );
    assert!(
        supply.ends_with("mean sites null, mean reachable null; uniform reference null"),
        "{supply}"
    );
    assert!(
        benefit.contains("delta_mean null, delta_max null"),
        "{benefit}"
    );
    assert!(
        benefit.ends_with("non-touching improved 0/0 = null"),
        "{benefit}"
    );
}

#[test]
fn an_axis_names_the_food_fraction_then_the_scale_then_a_layout() {
    assert_eq!(axis(Some(0.04), Some(2)), "fraction 0.04");
    assert_eq!(axis(None, Some(2)), "scale 2");
    assert_eq!(axis(None, None), "layout");
}

#[test]
fn an_uncalibrated_point_lists_only_the_checks_it_failed() {
    let mut calibration = calibration(Verdict::Uncalibrated);
    calibration.points[1].validation_competence = Some(false);
    let exposure = Exposure::of(&calibration, false);
    let gaps = exposure.failing[1].sensitivity_gaps.unwrap();
    assert!((gaps[2] - 0.2).abs() < 1e-12, "{gaps:?}");

    let mut out = String::new();
    exposure_lines(&mut out, &exposure);

    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines[..2],
        [
            "- exposure: fail (uncalibrated)",
            "  - fraction 0.04 lifetime 200: exposure (2/16 scenes drawn)",
        ],
        "an unexposed point fails exposure alone"
    );
    assert!(
        lines[2].ends_with("progress 0.200); validation competence"),
        "{out}"
    );
}
