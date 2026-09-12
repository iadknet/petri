# T14.F05 — cognition telemetry resolution

| Build record | Value |
| --- | --- |
| Date | 2026-09-12 |
| Worktree / branch | `.worktrees/t14-f05` / `codex/t14-f05` |
| Planning commit | `07dd1948` |
| State | Build and self-review complete; final review, mutation gate, benchmark reports, and final `make check` remain. |

## Build verification

| Command | Result |
| --- | --- |
| `cargo check --workspace --all-targets` | Passed after coherent implementation and test corrections; final Rust edit passed with no warnings. |
| `cargo test -p v3-core runtime::plasticity` | 17 passed. |
| `cargo test -p v3-core runtime::cognition_tests` | 5 passed. |
| `cargo test -p v3-core runtime:: --quiet` | 219 passed. |
| `cargo test -p v3-core simulation::tick::tests --quiet` | 88 passed. |
| `cargo test -p v3-core --test creature_workflow_e2e --test vm_all_opcodes_e2e --test temporal_fixtures --test applied_trajectory --test reproducibility --quiet` | 25 passed across five integration suites: 7 creature workflow, 1 VM opcode, 13 temporal fixtures, 1 sampled trajectory, 3 reproducibility. Original trajectory digest unchanged. |
| `cargo test -p v3-cli bench::tests --quiet` | 81 passed; preceding CLI pass had 79 before two additional property/empty-report tests. |
| `cargo fmt --all` | Completed. |
| `git diff --check` | Clean after build and self-review/spec updates. |
| `make roadmap-check` | Passed after build and self-review/spec updates; Aqua's package timestamp permission warning was nonfatal. |
| Viability-first applicability | Not applicable: production defaults, founders, dispatch/order, formulas, costs, and tick-loop mechanics are unchanged. `tick.rs` additions only accumulate observations at existing sites. |
| Proptest regressions | None generated during build verification. |

## TDD evidence

| Area | Observed red and green |
| --- | --- |
| Learning | New tests initially failed to compile because changed counts and the return component were absent. After instrumentation, zero activity, clamped-away and rounded-away assignments retained their cost and reported zero changes; changed assignments and generated per-edge invariants passed. |
| Memory | Tests first reported the missing counter; the new fixture's constant index was also corrected to the existing `u8` operand. After instrumentation, epsilon boundaries, sanitation, repeated/reversed writes, both VM store forms, clear, invalid/unwired Graph sinks, and traced/plain parity passed. |
| Reporting | Tests first reported missing cognition/census fields and census projection. After implementation, two fixture compile errors were corrected as recorded below. Terminal transfer, checkpoint omission, historical absence, measured empty populations, overlapping structural counts, unchanged comparison inputs, and unchanged sensitivity values passed. |

## Focused coverage

| Requirement | Evidence |
| --- | --- |
| Learning partition | Mixed production graph exercises both pathways across four ticks, including final clamped-away updates; generated edge/tick counts check assignment and changed partitions exactly. |
| Memory event semantics | Runtime property checks sanitized event sequences against each immediate predecessor; VM exhaustion fixture counts the executed write while preserving discarded-memory behavior and excluding the unaffordable write. |
| Production ownership | VM and Graph events accumulate across ticks in plain and traced runs; memory decay/snapshots do not count. Final and temporal observations leave all seven cumulative counters and creature memory/weights unchanged. |
| Structural census | Authored reader/writer, stateful/plastic reader, and unreachable-only fixtures verify overlapping flags. Generated populations preserve counts under reordering; real goal reports use sensitivity's final-population denominator. |
| Repeatability | Existing cross-thread report test includes the new census; core seeded/cross-thread reproducibility now includes all new integer counters. Existing sampled-trajectory digest remains `5898914f4106b258d26d724379ac3cc50cf2f811f9b505294098e9cc137f41f7`. |

## Advisor consultations

| Checkpoint | Decision |
| --- | --- |
| 1 — approach | Accepted flat report fields, existing integer `+=` reductions, stored-weight inequality, shared epsilon predicate, optional terminal/census blocks, and observation-only viability interpretation. Counting executed VM writes before a later discarded working copy matches the spec's event definition. No extra abstraction or dependency. |
| 2 — repeated compile diagnostics | A workspace check and queued CLI test both reported the same two fixture errors before remediation: `creature_count` instead of existing `final_creature_count`, and an explicit checkpoint initializer missing `cognition`. Accepted the two test-only corrections: correct field and `cognition: None`. No requirement revision. |
| 3 — final build checkpoint | No correctness, requirement, or scope blocker. Accepted confirmation of unchanged learning/costs, exact partitions, pre-exhaustion event retention, memory coverage, optional report semantics, census/sensitivity meaning, and viability inapplicability. No corrective change recommended. |
| 4 — final self-review/spec checkpoint | Pass sufficient; checked items are supported, closure-dependent items correctly remain open, absent benchmark files match the unmeasured verdict, and the performance predeclaration is intact. Accepted no-correction/no-runtime-rerun guidance; documentation checks passed. |

## Self-review

| Area | Finding and action |
| --- | --- |
| Scope reviewed | Entire feature diff against merge base `6bd9d1bf7359e8773739834f919c0e069fc7d1af`, including the added runtime test file and readings. |
| Runtime reuse and efficiency | Existing assignment/write sites, integer side outputs, queue-order reduction, and Phase 2.5 accumulation are retained. The epsilon predicate is shared. No per-event allocation, dependency, configuration, or extra execution/tracing layer was introduced. No Rust correction needed. |
| Report representation | Existing Serde `Option` default/skip-none fields express absence. Flat integer fields and the existing structural scan avoid a second reachability analysis or validation layer. Census storage follows the existing per-seed report vectors. No string-state framework or custom utility was introduced. |
| Test reuse | Existing plasticity, tick, and report fixtures are extended; common plain/traced execution and paired VM/Graph fixtures keep event assertions together. Tests retain exact partition and absence checks. No test correction needed. |
| Documentation | Recorded completed consultation 3 and focused evidence. Checked only implemented/tested tasks and criteria; closure-dependent items remain unchecked. Performance predeclaration and unmeasured verdict are unchanged. |
| Verification scope | This pass changes documentation only; the focused build results above remain current. No additional runtime tests, benchmarks, mutation gate, or full check were run. |

## Closure measurements

No mutation, gate, goal, or final full-check result is claimed in this build pass.
