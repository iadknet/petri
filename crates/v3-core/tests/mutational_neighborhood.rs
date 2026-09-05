//! Maintained mutational-neighborhood integration test (T11.F01).
//!
//! Lifts the brain evolvability audit's Appendix A probe
//! (`docs/strategy/brain-evolvability-audit-2026-09-04.md`) into the
//! permanent suite: it runs the founder half of the `neighborhood-v1`
//! battery at reduced sizes (fast enough for `make check`), asserts
//! determinism across thread counts, and asserts the audit's structural
//! facts about specific operators. It asserts no floor — the T11 track
//! floors are met by the close of T11.F10, not by this observation-only
//! feature.

use v3_core::config::SimulationConfig;
use v3_core::creature::founder::founder_genome;
use v3_core::neighborhood::{evaluate_genome, Battery, EvalContext, GenomeEvaluation};

/// Reduced from the predeclared founder-half sizes so this test runs in a
/// fraction of a second in a debug build.
const REDUCED_OPERATOR_TRIALS: u32 = 10;
const REDUCED_BIRTHS: u32 = 100;

fn founder_evaluation(threads: usize) -> GenomeEvaluation {
    let config = SimulationConfig::default();
    let subject = founder_genome(config.population.founder_profile);
    let battery = Battery::generate(config.world.food.types.len());
    let context = EvalContext::from_config(&config);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("a rayon pool of at least one thread can always be built");
    pool.install(|| {
        evaluate_genome(
            &subject,
            &battery,
            &config.mutation,
            &context,
            REDUCED_OPERATOR_TRIALS,
            REDUCED_BIRTHS,
            0,
        )
    })
}

/// The `deterministic` block's byte-identity claim rests on the founder
/// half's tallies being independent of thread count, since trials are
/// evaluated in parallel with rayon. Two runs at the same thread count would
/// not catch a chunking-dependent fold; this compares two different thread
/// counts against each other.
#[test]
fn founder_neighborhood_is_identical_across_thread_counts() {
    let single_threaded = founder_evaluation(1);
    let multi_threaded = founder_evaluation(4);

    assert_eq!(
        single_threaded, multi_threaded,
        "founder-half tallies must be byte-identical regardless of thread count"
    );
}

/// The audit's headline structural facts: two operators are reliably silent
/// on the founder because of how they are constructed, not because of any
/// repair. `VmCopyConstantBlock` only appends to the (unread by control
/// flow) constant pool; `CopySubgraph` lands its copy disconnected. Neither
/// fact depends on any T11 repair, so this assertion is not a floor.
#[test]
fn specific_operators_are_fully_silent_on_the_founder_as_the_audit_recorded() {
    let evaluation = founder_evaluation(1);
    let row = |operator: &str| {
        evaluation
            .operator_rows
            .iter()
            .find(|row| row.operator == operator)
            .unwrap_or_else(|| panic!("operator {operator} missing from the founder's rows"))
    };

    let copy_constant_block = row("VmCopyConstantBlock");
    assert_eq!(
        copy_constant_block.tally.applied(),
        REDUCED_OPERATOR_TRIALS,
        "VmCopyConstantBlock always applies on the founder"
    );
    assert_eq!(
        copy_constant_block.tally.silent, REDUCED_OPERATOR_TRIALS,
        "VmCopyConstantBlock is append-only, so every trial is silent (audit: 100%)"
    );

    let copy_subgraph = row("CopySubgraph");
    assert_eq!(copy_subgraph.tally.applied(), REDUCED_OPERATOR_TRIALS);
    assert_eq!(
        copy_subgraph.tally.silent, REDUCED_OPERATOR_TRIALS,
        "CopySubgraph lands disconnected, so every trial is silent (audit: 100%)"
    );
}

/// Every operator in the four `ALL` lists produces a row, whether or not it
/// could apply to the founder (an inapplicable operator is a skip, not an
/// absence).
#[test]
fn every_operator_in_the_four_domains_produces_a_row() {
    let evaluation = founder_evaluation(1);
    assert_eq!(
        evaluation.operator_rows.len(),
        v3_core::neighborhood::operator_catalog().len()
    );
    for row in &evaluation.operator_rows {
        assert_eq!(row.tally.trials, REDUCED_OPERATOR_TRIALS);
    }
}

/// Per-birth buckets are counted, not evaluated, for zero-event births, and
/// every attempted birth is accounted for exactly once.
#[test]
fn per_birth_buckets_account_for_every_attempted_birth() {
    let evaluation = founder_evaluation(1);
    let births = &evaluation.births;
    assert_eq!(births.births_total, REDUCED_BIRTHS);
    let bucketed_trials: u32 = births.by_events.values().map(|tally| tally.trials).sum();
    assert_eq!(
        births.zero_event_births + bucketed_trials,
        births.births_total
    );
    assert_eq!(births.any_events.trials, bucketed_trials);
}
