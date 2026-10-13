# T14.F06 — Reproductive Success by Cognitive Class

**Status**: Complete
**Last updated**: 2026-09-12
**Feature**: T14.F06
**Track**: [T14 — Runtime Telemetry and Report Integrity](../../roadmaps/t14-runtime-telemetry-and-report-integrity.md)

## Goal

Each standard goal world's terminal report stores the number of removed
creatures, their successful offspring, and their observed lifetimes by cognitive
structure class. The outcome of carrying memory, state or plasticity is readable
beside creatures carrying none of them, without inferring reproductive payoff
from structural presence or perturbation sensitivity.

## Non-Goals

- Change simulation behavior, charges, rates, defaults, action order, death
  timing, RNG draws, or goal-world recipes.
- Measure executed use, beneficial cognition, causal selection, trait-specific
  effects, or T01.F13's shadow model. Add no assay, composite fitness score,
  controller ranking, per-creature history, clade table, dashboard or server API.
- Add a census of survivors to this death-time reading, rewrite historical
  reports, or create a new indicator floor.

## Inputs and Invariants

Sources: the owning F06 row and F03/F06/north-star Notes;
[T14.F03's mortality contract](t14-f03-applied-mortality-and-energy-accounting.md);
`Simulation::remove_creature` in
`crates/v3-core/src/simulation/simulation.rs`; `SimStats` in
`simulation/stats.rs`; `CreatureState::{age,offspring_spawned_count}` in
`creature/state.rs`; `simulation/actions/reproduction.rs` and the existing
Phase 0/predation removal paths; `neighborhood/companions.rs` and
`neighborhood/mod.rs`; and `WorldTracking` in `crates/v3-cli/src/bench.rs`.
Unqualified core paths are under `crates/v3-core/src/`.

Research (2026-09-12): extend the existing removal funnel, fixed runtime totals
and optional terminal report blocks. An event ledger with later aggregation
adds storage and replay machinery without improving these twelve integers.
Computing structural facts at birth and maintaining a cache adds constructor
and invalidation work; one analysis at removal is sufficient. The existing
analyzer is RNG-free but **is not cached on `CreatureState`**. Its
`neighborhood` module forbids a reverse dependency from `simulation`. Move the
pure analyzer and its type into `creature::genome` and re-export them from the
existing neighborhood paths, retaining one implementation and its semantics.
The [Rust Reference's re-export rules](https://doc.rust-lang.org/reference/items/use-declarations.html#use-visibility)
support that small relocation; [Serde's field attributes](https://serde.rs/field-attrs.html)
support the established absent-versus-measured block convention. No dependency
or general telemetry framework is needed.

**Classification.** The roadmap names four potentially overlapping booleans
and requires twelve integers while its goal requires a non-cognitive comparison.
Twelve integers is controlling: use four exhaustive, mutually exclusive classes,
each with three `u64` totals. Classify the removed creature's genome with the
existing `structural_companions` definition, in the precedence below. This
precedence resolves overlap for accounting; it is not a cognitive ability rank.

| Stable class key | Membership, first matching row wins |
| --- | --- |
| `plasticity` | `has_plasticity` |
| `stateful` | No plasticity, and `has_stateful_compute_node` |
| `shared_memory` | Neither above, and `reads_shared_memory || writes_shared_memory` |
| `none` | All four flags are false |

Preserve the analyzer's mesh-level reachability and current VM/graph definitions:
current/previous-slot reads, writes/clears, graph stateful kinds and carried
plasticity rules. Do not substitute VM instruction liveness, actual dispatches,
changed weights, or memory sensitivity. A write-only genome is in
`shared_memory`; unreachable mesh nodes do not change class. A plastic genome
may also read memory and hold state, so its totals cannot identify the separate
effect of any one trait.

**Applied lifetime totals.** Add to exactly one class at each successful
`Simulation::remove_creature` removal, alongside F03's mortality observation:

| Field in each class | Contribution from the removed creature |
| --- | --- |
| `creatures_observed_total` | `1` |
| `offspring_spawned_sum` | Existing `offspring_spawned_count` |
| `survival_ticks_sum` | Existing `age` |

Include founders and offspring regardless of generation or birth-mutation
operators. Include every F03 removal cause, including predation, external
removal and unattributed removal. Repeated removal of an absent ID changes
nothing. The four counts sum to `mortality.deaths_total`; summed offspring and
ages equal those of the removed creatures. Successful births contribute through
their parent's existing counter, not action attempts or child counts. The
offspring sum need not equal all run births because surviving parents have not
entered the death cohort. Age is the value at the existing removal instant:
newborns may contribute zero; external removal is an observed exit age, not a
natural lifespan. Do not flush surviving creatures into these totals at closure.

Totals accumulate sequentially in the common funnel, never in parallel runtime
execution or observational probes. Runtime storage is exactly four fixed rows
of three `u64`s per simulation, with no growth by creatures, deaths or clades.
The analyzer may use its existing temporary genome traversal; do not add a
per-tick scan or alter it to optimize unrelated callers.

**Report shape and interpretation.** Add
`reproductive_success_by_cognitive_class: Option<ReproductiveSuccessByCognitiveClassTracking>`
to `WorldTracking` with
`#[serde(default, skip_serializing_if = "Option::is_none")]`. Populate it only
through `with_transferred_counters`, leaving it absent from checkpoint samples
and historical reports. The flattened terminal case path is
`deterministic.goal_indicators.cases[].reproductive_success_by_cognitive_class`.
The block contains `definition: "reproductive-success-by-cognitive-class-v1"`
and `by_class`, with exactly the four stable keys above and all three integer
fields per key, including observed zeros. Use deterministic key order. No
stored means, ratios or additional cohort totals are needed.

A missing historical block is unmeasured; a present zero-count row is an empty
observed cohort whose mean is undefined. The readings file shows these raw
totals per Orchards, Canyon and Confluence. This is a correlation among removed
creatures, confounded by carrying cost, genotype, environment and the exclusion
of survivors. A class with more offspring is not evidence of useful cognition
or causal selection; class precedence also prevents independent-trait claims.

## Implementation Tasks

- [x] Share the existing structural analyzer from the genome layer while
  retaining its neighborhood callers and semantics.
- [x] Add the fixed class totals and the sole removal-time observation.
- [x] Transfer all twelve integers into each standard goal case's optional
  terminal block, with its definition token and historical absence semantics.
- [x] Complete verification, store the gate/goal reports and concise readings,
  and close the feature under the shared workflow.

## Verification

- [x] Focused core tests: `cargo test -p v3-core --lib` checks all class
  combinations, unchanged structural-companion callers, unreachable structure,
  all removals counted once, birth outcomes and age sums; property tests cover
  classification partition and additive/permutation invariants. Record executed
  test names and results in `docs/progress/readings/t14-f06.md`.
- [x] Focused report tests: `cargo test -p v3-cli` checks twelve-field transfer,
  definition/key shape, empty cohorts, historical absence, checkpoint omission
  and terminal per-world replay agreement. Results: same readings file.
- [x] `make check` passes at tested commit
  `4f116b4cad727f9d29e6b7cefe520e2faee05f84`, including viability and
  cross-process/thread-count
  reproducibility; `make roadmap-check` passes. Results: same readings file.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: fresh mode, output
  `/Users/istefanek/.local/share/petri-tools/mutants/t14-f06/mutants.out`;
  summary `21 mutants tested in 4m: 2 missed, 16 caught, 3 unviable`.
  Full survivor resolution:
  - **Killed:** `crates/v3-core/src/simulation/reproductive_success.rs:37:9:
    replace CognitiveClass::as_key -> &'static str with ""` — the new
    `cognitive_class_keys_match_the_report_contract` core test pins all four
    public report keys; the incremental remediation pass caught the mutant.
  - **Killed:** `crates/v3-core/src/simulation/reproductive_success.rs:37:9:
    replace CognitiveClass::as_key -> &'static str with "xyzzy"` — the same
    exact-key contract test caught this arbitrary replacement.
  - Incremental feedback: `MUTANTS_ITERATE=1 make rust-mutants`, incremental
    mode, `2 mutants tested in 2m: 2 caught`, no survivors. No second fresh run
    was required: only a test was added; production, test selection and tool
    configuration were unchanged, and no test was deleted or weakened.
- [x] Gate and one goal benchmark run stored the reports below; both exit 0 and
  have `severe=false`. Gate is all `ok`; goal has one non-severe advisory flag
  against its older epoch. Exact threshold verdicts, class totals, effective
  configuration identities and observation budgets are in
  `docs/progress/readings/t14-f06.md`.
- [x] Second goal run: Not applicable under the shared one-run closure contract;
  cross-process reproducibility remains in `make check`, and the gate's two-run
  deterministic check remains required there.

## Performance and Goal Impact

**Predeclaration — written before the run.** This observation-only feature is
exempt from the natural-analog rule and adds no environmental pressure. The
ordinary goal run retains Orchards, Canyon and Confluence with all their
existing recipes and pressures. The short compute gate is unchanged.

Expected cost is one existing structural analysis per removed creature and
three integer additions into one fixed row, plus a bounded terminal report.
The analysis traverses the removed genome and may allocate its existing
temporary reachability data; it adds no work to surviving creatures per tick.
No severe compute regression or epoch re-pin is expected or authorized.

References remain the existing series index: gate epoch
`docs/progress/features/remove-complementary-nutrition.json`, goal-worlds epoch
`docs/progress/features/t12-f04-baseline-world-set-goal.json`, and the last
closed report of each corresponding series, excluding the current output. At
planning both last-closed references are T14.F03's matching reports. Normalized
work thresholds remain +10% flag/+50% severe; matching-host wall time has the
existing +25%/+100% advisory flags. Founder-neighborhood observation stays
within 10 seconds, evolved-neighborhood observation within 180 seconds summed
across seeds, and total goal-profile investigation within 15 minutes. Existing
floors, profile parameters, sample/trial counts and mutation gates are unchanged.

Expected direction is none for population persistence, births per 100 ticks,
lineage diversity, memory and temporal-memory sensitivity, mutational
neighborhood, drift depth, reachable/structural/recruitment readings, and the
existing mortality, energy, sensor and cognition observations. These must
reflect unchanged simulation behavior. The new class totals are first
readings with no beneficial direction and no historical-zero reference. They
are raw outcome telemetry in the goal profile, not a new scalar cognition
indicator or a selection verdict.

Required measured commands, run sequentially through the existing preflight:

```sh
make bench PROFILE=gate FEATURE=t14-f06-reproductive-success-by-cognitive-class
make bench PROFILE=goal FEATURE=t14-f06-reproductive-success-by-cognitive-class
```

**Measured verdict.** Gate and goal completed sequentially on 2026-09-12 at
revision `698510b4a233643f9934fc232375c0f78d939483`; both exit 0 and report
`severe=false`. Gate is all `ok`. Goal is non-severe: the only normalized
advisory flag is `plasticity_updates` (+40.886836% against the older T12.F04
epoch), while its wall-clock comparison is `ok`; the matching T14.F03 goal
comparison is all `ok`. Founder and evolved-neighborhood observations are
within their 10 s and 180 s caps. The goal report includes the first terminal
class totals for all three unchanged standard-world configuration identities.
They are confounded removed-creature correlations, not a cognition-payoff or
selection conclusion. Full values and comparison boundary:
`docs/progress/readings/t14-f06.md`.

- Reports: `docs/progress/features/t14-f06-reproductive-success-by-cognitive-class.json`
  and `docs/progress/features/t14-f06-reproductive-success-by-cognitive-class-goal.json`.
- Full readings: `docs/progress/readings/t14-f06.md`.

## Success Criteria

- [x] Every removed creature contributes its successful offspring and age once
  to exactly one specified cognitive class, including the `none` comparison.
- [x] Every standard goal case stores all twelve applied integers; checkpoints
  and historical reports preserve unmeasured absence.
- [x] Interpretation states the class precedence, survivor exclusion and
  confounding; no cognition payoff claim is inferred from these correlations.
- [x] Required tests, review, mutation gate, stored benchmark verdicts and
  workflow closure are complete without behavior changes or waived checks.

## Notes for AI Agents

- Decision: Cognitive class is a structural, mutually exclusive death cohort
  defined by the precedence above, not a claim of executed or useful cognition.
- Decision: Final review found P1=0, P2=0, P3=1: the stale readings status
  was corrected in documentation.
- Decision: Workflow roles were Sol `medium` orchestrator; persistent Astra
  `xhigh` spec owner/advisor; persistent Astra `xhigh` implementer; Terra
  `high` benchmark specialist; fresh Astra `xhigh` reviewer; and Sol `medium`
  mutation specialist. Explicit subagent selections were enforced; active
  orchestrator runtime metadata was unavailable beyond the goal configuration.
- Decision: Three advisor consultations covered the pre-approach boundary,
  pre-handoff verification, and the sandboxed full-check permission exception.
  All advice was accepted; no implementation requirement correction or check
  waiver occurred. One planning clarification made the twelve integers four
  exclusive precedence-ordered classes.
- Decision: One documentation-only post-review remediation pass corrected the
  P3 status row. Production remediation passes were zero.
- Decision: One user intervention requested a durable Codex benchmark-agent
  instruction to request process-inspection permission on its first measured
  attempt; `docs/workflow-codex.md` now records it. No remote was mutated.
- Cost: Native goal telemetry immediately before completion reported 642,353
  tokens and 3,535 seconds elapsed. Advisor consultations: 3; implementation
  passes: 1 build plus 1 documentation-only remediation; final review findings:
  P1=0, P2=0, P3=1.
