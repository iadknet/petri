# T14.F05 — cognition telemetry resolution

| Build record | Value |
| --- | --- |
| Date | 2026-09-12 |
| Worktree / branch | `.worktrees/t14-f05` / `codex/t14-f05` |
| Planning commit | `07dd1948` |
| State | Feature complete in the worktree: focused checks, final review, fresh mutation gate, both benchmark reports, and final full check passed; integration remains the orchestrator's responsibility. |

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
| 5 — final mutation checkpoint | Fresh mode, successful baseline, 92 = 86 caught + 6 unviable, zero missed/timeouts, empty survivor files, and no added exclusions/skips satisfy the gate. Accepted no-remediation/no-additional-run guidance; spec record is truthful. |
| 6 — final benchmark checkpoint | Confirmed deterministic equality, measured deltas and caps, cognition partitions, census denominators, and checkpoint omission. Inherited plasticity flag and founder gap are unchanged. Accepted no-rerun/no-waiver/no-re-pin/no-remediation guidance and the required spec clarification explicitly ruling out an epoch re-pin for both profiles. |
| 7 — full-check environment checkpoint | Accepted a full host rerun after seven websocket tests could not bind localhost in the sandbox. No test skip, requirement waiver, or final-check pass was authorized or claimed. |
| 8 — final test-only remediation checkpoint | Confirmed exactly five clone removals on `Copy` `GraphEdge`, unchanged fixtures/assertions, and passing focused/Clippy checks. Accepted no further correction, self-review, or viability run; existing benchmark/mutation evidence remains applicable. |
| 9 — final closure-document checkpoint | Confirmed Complete status, supported feature checkboxes, unchanged broader rollups, and truthful verification records. Accepted the final telemetry correction from eight to nine completed consultations; no blocker or runtime rerun. |

## Self-review

| Area | Finding and action |
| --- | --- |
| Scope reviewed | Entire feature diff against merge base `6bd9d1bf7359e8773739834f919c0e069fc7d1af`, including the added runtime test file and readings. |
| Runtime reuse and efficiency | Existing assignment/write sites, integer side outputs, queue-order reduction, and Phase 2.5 accumulation are retained. The epsilon predicate is shared. No per-event allocation, dependency, configuration, or extra execution/tracing layer was introduced. No Rust correction needed. |
| Report representation | Existing Serde `Option` default/skip-none fields express absence. Flat integer fields and the existing structural scan avoid a second reachability analysis or validation layer. Census storage follows the existing per-seed report vectors. No string-state framework or custom utility was introduced. |
| Test reuse | Existing plasticity, tick, and report fixtures are extended; common plain/traced execution and paired VM/Graph fixtures keep event assertions together. Tests retain exact partition and absence checks. No test correction needed. |
| Documentation | Recorded completed consultation 3 and focused evidence. Checked only implemented/tested tasks and criteria; closure-dependent items remain unchecked. Performance predeclaration and unmeasured verdict are unchanged. |
| Verification scope | This pass changes documentation only; the focused build results above remain current. No additional runtime tests, benchmarks, mutation gate, or full check were run. |

## Mutation gate

| Measurement | Result |
| --- | --- |
| Command | `MUTANTS_ITERATE=0 make rust-mutants` |
| Tested commit | `123e90e2ccb3e0dadd767afc2188ac4a917d2205` |
| Diff base | `6bd9d1bf7359e8773739834f919c0e069fc7d1af` |
| Run mode | `fresh`, confirmed by `/Users/istefanek/.local/share/petri-tools/mutants/t14-f05/run-mode.txt` |
| Output directory | `/Users/istefanek/.local/share/petri-tools/mutants/t14-f05/mutants.out` |
| Printed summary | `92 mutants tested in 10m: 86 caught, 6 unviable` |
| Exit result | 0; `rust-mutants: no survivors` |
| Baseline | Unmutated build and full selected-package tests passed: printed 46s build + 13s test. |
| Full missed survivor list | None; `missed.txt` is 0 bytes. |
| Full timed-out survivor list | None; `timeout.txt` is 0 bytes. |
| Machine evidence | `outcomes.json`: total 92, caught 86, unviable 6, missed 0, timeout 0. |
| Triage | No survivor requires a killed/equivalent/deferred resolution. No production or test edits, incremental pass, or second fresh run. |
| Documentation verification | `make roadmap-check` passed after the mutation record, with the nonfatal Aqua timestamp warning; `git diff --check` was clean. |
| Environment | Initial sandbox invocation stopped in `bench-wait` before mutation testing because host process inspection was denied. The authorized host-access retry performed the single fresh run. |

Raw mutation command output:

```text
rust-mutants: fresh run; no prior mutant results reused
rust-mutants: diff against 6bd9d1bf7359e8773739834f919c0e069fc7d1af, output in /Users/istefanek/.local/share/petri-tools/mutants/t14-f05/mutants.out
Found 92 mutants to test
ok       Unmutated baseline in 46s build + 13s test
 INFO Auto-set build timeout to 374s
 INFO Auto-set test timeout to 120s
92 mutants tested in 10m: 86 caught, 6 unviable
rust-mutants: no survivors
```

## Closure measurements

| Measurement record | Value |
| --- | --- |
| Measured revision | `22cd1afd5e14747d9bf7c849242677ac174fa736`; production unchanged from reviewed/mutation-tested `123e90e2ccb3e0dadd767afc2188ac4a917d2205` |
| Host / build / threads | Apple M1 Pro, macOS aarch64, `Isaacs-MacBook-Pro-2.local`; release; 8 threads |
| Gate command | `make bench PROFILE=gate FEATURE=t14-f05-cognition-telemetry-resolution` — exit 0; one invocation |
| Goal command | `make bench PROFILE=goal FEATURE=t14-f05-cognition-telemetry-resolution` — exit 0; exactly one invocation, after gate success |
| Isolation | Both commands used the unchanged `bench-wait` host preflight; authorized host access allowed process inspection. No competing benchmark/server/build/test/mutant work was started. |
| Gate report | [t14-f05-cognition-telemetry-resolution.json](../features/t14-f05-cognition-telemetry-resolution.json), generated `2026-09-12T19:34:49Z` |
| Goal report | [t14-f05-cognition-telemetry-resolution-goal.json](../features/t14-f05-cognition-telemetry-resolution-goal.json), generated `2026-09-12T19:43:10Z` |
| Gate SHA-256 | `63f6045b7934a9f9ce954b14d355bb4a301e23a25d9765fb35547ccdbb43f051` |
| Goal SHA-256 | `4d853ec67836a95b9a707a43ccad8389aa12e362d87b2faef3bde56fc6832b23` |
| Gate profile | `gate`, 128×128, 256 founders, seeds 11/22/33, 75 ticks, food coverage `1.000000` |
| Goal profile | `goal-worlds-v1`, 1600×1600, 10,000 founders per world, seeds 11/22/33, 2,000 ticks, recipe-specific food coverage |
| Reference selection | Unchanged `docs/progress/benchmark-series.json`: `gate-v1` and `goal-worlds-v1` epochs plus latest indexed T14.F03 closures; no baseline edits or epoch re-pin |
| Final full check | Not run in this benchmark pass; subsequent successful closure check is recorded below |
| Documentation verification | `make roadmap-check` exited 0 after the measured record (`roadmap-check: validation passed`; nonfatal Aqua timestamp warning); `git diff --check` exited 0. |

### Reference comparisons and caps

Reference paths below are relative to `docs/progress/features/`.

| Profile / reference | Baseline | Severe | Wall delta / level |
| --- | --- | --- | --- |
| Gate / epoch | `remove-complementary-nutrition.json` | false | -11.553548% / ok |
| Gate / latest | `t14-f03-applied-mortality-and-energy-accounting.json` | false | +3.408074% / ok |
| Goal / epoch | `t12-f04-baseline-world-set-goal.json` | false | +2.369318% / ok |
| Goal / latest | `t14-f03-applied-mortality-and-energy-accounting-goal.json` | false | -1.293679% / ok |

| Normalized work / creature-tick | Gate current | Gate vs epoch | Goal current | Goal vs epoch | Both vs latest |
| --- | --- | --- | --- | --- | --- |
| `mesh_hops` | 2.028954 | +0.083512% | 2.258369 | +0.464115% | 0.000000% |
| `vm_steps` | 22.425973 | -1.429996% | 23.478709 | +0.595336% | 0.000000% |
| `graph_relax_iters` | 0.994920 | -0.031550% | 1.024449 | -0.397748% | 0.000000% |
| `plasticity_updates` | 0.009880 | -11.556709% | 0.066183 | +40.886836% (inherited flag) | 0.000000% |
| `actions_applied` | 1.272860 | -0.208934% | 1.360568 | -0.773642% | 0.000000% |
| `births` | 0.026902 | +0.455564% | 0.019659 | +5.642431% | 0.000000% |

| Cost boundary | Applied reading / verdict |
| --- | --- |
| Work >+10% flagged / >+50% severe | Only aggregate flag is goal plasticity vs T12.F04; identical to T14.F03, no new or severe regression |
| Wall >+25% flagged / >+100% severe | All four comparisons `ok`; gate `0.001379615195` ms/creature-tick, goal `0.007582147304` |
| Founder observation ≤10,000 ms/profile | Gate 53.309042 ms; goal 103.718667 ms — met |
| Evolved observation ≤180,000 ms summed across seeds | Goal 499.322792 ms — met |
| Goal timed simulation | 462273.505085 ms |
| Final-state observation, outside timed simulation | 817.288167 ms |
| Drift-depth observation, outside timed simulation | 18252.618751 ms |
| Goal invocation 900-second investigation threshold | Approximately 483 seconds (8.05 min), observed CLI start `19:35:07 UTC` to report timestamp `19:43:10 UTC`; even the conservative interval from the preceding gate report is only 501 seconds — met |

### Deterministic and report audit

Read-only `jq` comparisons and invariant checks passed:

- The gate's entire `deterministic` block equals T14.F03. The goal's entire
  `deterministic` block equals T14.F03 after deleting exactly
  `goal_indicators.structural_companions` and `goal_indicators.cases[].cognition`.
  No existing trajectory, indicator, persistence sample, work counter, mortality,
  energy, or mutation reading changed.
- Goal profile identity equals T14.F03. All three effective config digests equal
  both selected references; no absent case or `inputs_changed` flag. The 104
  case comparisons (35/34/35) equal T14.F03's comparisons for each reference.
- Exactly three terminal cognition blocks exist; zero of 60 persistence
  checkpoints has one. Assignment and changed totals each equal their two
  pathways; changes do not exceed assignments; combined assignment counts equal
  the existing per-seed work counters.
- All census counts are bounded by their final population, and each denominator
  equals the same seed's memory-sensitivity denominator. Counts overlap; they
  are not disjoint categories or a capability score.
- The new cognition and four census counts are positive in all three measured
  worlds; none was omitted or replaced with a historical zero. Existing zero
  sensitivity readings are explicitly preserved below. Historical new-field
  absence remains unmeasured, as covered by focused tests.

### World identity and cognition observations

| Field | Orchards in grassland | Canyon country | Confluence |
| --- | --- | --- | --- |
| Seed | 11 | 22 | 33 |
| Recipe | `experiments/worlds/orchards-in-grassland.json` | `experiments/worlds/canyon-country.json` | `experiments/worlds/confluence.json` |
| Effective config digest | `sha256:8141056a33bc30445fd29340fa40498372835e08459b7c04c347f9724b445423` | `sha256:ee72c5532cb947fad7349a3a4d3c5a5b5bef2c501ebe5b299b5de12ad26bd99d` | `sha256:25fb4d0baf34719c0f1e657c98b4e8a7932216510d69651c6fd58d4f6d318676` |
| Configured food slots | 2 | 1 | 2 |
| `plasticity_updates_total` | 1948986 | 1451896 | 634227 |
| `plasticity_changes_total` | 149840 | 159555 | 103850 |
| Unchanged assignments (updates minus changes) | 1799146 | 1292341 | 530377 |
| `hebbian_updates_total` | 1909900 | 1437191 | 618955 |
| `hebbian_changes_total` | 147898 | 158553 | 103788 |
| `reward_modulated_updates_total` | 39086 | 14705 | 15272 |
| `reward_modulated_changes_total` | 1942 | 1002 | 62 |
| `shared_memory_writes_changed_total` | 177139 | 473073 | 516538 |
| Final living population / census denominator | 11313 | 7090 | 9995 |
| `reads_shared_memory` | 6341 | 3134 | 5250 |
| `writes_shared_memory` | 7894 | 4210 | 5716 |
| `has_stateful_compute_node` | 252 | 487 | 458 |
| `has_plasticity` | 1551 | 626 | 383 |
| Memory-sensitive to zeroing, count / fraction | 0 / 0.000000 | 5 / 0.000705 | 0 / 0.000000 |
| Memory-sensitive to scrambling, count / fraction | 1 / 0.000088 | 5 / 0.000705 | 0 / 0.000000 |
| Memory-sensitive to either, count / fraction | 1 / 0.000088 | 5 / 0.000705 | 0 / 0.000000 |
| Temporal previous-slot sensitivity, either | 0 / 0.000000 | 0 / 0.000000 | 0 / 0.000000 |
| Temporal persisted-output sensitivity, either | 19 / 0.001679 | 47 / 0.006629 | 150 / 0.015008 |
| Temporal operator-state sensitivity, either | 2 / 0.000177 | 127 / 0.017913 | 49 / 0.004902 |

These first readings distinguish executed assignments from stored-weight
changes and eventful memory use from final structural exposure. They do not
claim evolved cognition: Confluence has 5250 memory readers and 516538 changed
write events yet zero final memory-sensitive creatures under the unchanged
probe. No mechanism or environmental pressure was added; environmental-pressure
integration is not applicable.

### Existing goal floors and no-regression verdict

| Floor / observation | Orchards in grassland | Canyon country | Confluence | Verdict |
| --- | --- | --- | --- | --- |
| Memory motifs silent ≥0.80 (read/store, read/bid, load/compare) | 0.900000 / 1.000000 / 0.880000 | 0.900000 / 1.000000 / 0.860000 | 0.900000 / 1.000000 / 0.880000 | Met |
| Graph add-node / copy-node / input-reference-add silent ≥0.95 | 1.000000 / 1.000000 / 1.000000 | 1.000000 / 1.000000 / 1.000000 | 1.000000 / 1.000000 / 1.000000 | Met |
| Founder mutated-birth dead ≤0.05 | 0.000000 | 0.000000 | 0.000000 | Met |
| Founder single-event silent ≥0.60 | 0.567073 | 0.597561 | 0.567073 | Inherited gap; unchanged, not claimed met |
| Evolved mutated-birth dead ≤0.05 | 0.000909 | 0.000909 | 0.009091 | Met |
| Evolved single-event silent ≥0.60 | 0.695175 | 0.739035 | 0.725877 | Met |

Gate founder single-event silence is also unchanged at 0.597561 below 0.60.
The inherited founder floor belongs to T11.F01's fixed targets due by T11.F10;
this feature neither resolves nor waives it. No existing goal or comparison
coverage regressed. No rerun, baseline change, threshold weakening, verification
exception, or production remediation was used in the benchmark pass. Its
pending final full check was subsequently completed as recorded below.

## Post-review test-only remediation

The orchestrator reported these first two full-check attempts; neither is
passing closure evidence, and no tested commit is claimed for either attempt.

| Full-check attempt | Result |
| --- | --- |
| 1 — `make check` in sandbox | Exit 2: seven websocket tests could not bind localhost. Consultation 7 authorized a full host rerun, not a waiver. |
| 2 — `make check` with host access | All tests and doctests passed, then `rust-clippy` failed on exactly five `clone_on_copy` diagnostics in the three test files below. This attempt is non-passing. |

Removed exactly five unnecessary `GraphEdge::clone()` calls using the type's
existing `Copy` semantics: two in `runtime/plasticity/tests.rs`, one in
`runtime/cognition_tests.rs`, and two in
`simulation/tick/tests/work_counters.rs`. Fixture values, test selection,
assertions, and production code are unchanged. The reported Clippy failure is
the red evidence; the same lint gate passes after this correction. Viability
first is not applicable to this test-only edit. No new self-review pass is
required for this scoped post-review test remediation.

| Remediation command | Result |
| --- | --- |
| `cargo fmt --all` | Exit 0 |
| `cargo check --workspace --all-targets` | Exit 0 |
| `make rust-clippy` | Exit 0; exact Makefile gate: `cargo clippy --workspace --all-targets -- -D warnings` |
| `cargo test -p v3-core runtime::plasticity --quiet` | Exit 0; 17 passed |
| `cargo test -p v3-core runtime::cognition_tests --quiet` | Exit 0; 5 passed |
| `cargo test -p v3-core simulation::tick::tests::work_counters --quiet` | Exit 0; 8 passed |

No full check, benchmark, or mutation run was repeated in this remediation
pass. Its final full-check item and dependent closure items remained unchecked
until the successful run below.

## Final closure verification and workflow telemetry

| Closure evidence | Result |
| --- | --- |
| Final command | Full host-permitted `make check` — exit 0, run by the orchestrator after the five test-only clone removals |
| Tested commit | `a0ea25c264e9fb5eef7b5f42d78a4451163242dd` |
| Completed gate coverage | Rust tests/doctests and `rust-clippy`; frontend lint with four pre-existing warnings only; 61 frontend test files / 322 tests; frontend build; dependency scan; skill check; documentation validation |
| Spec state | Complete; all Implementation Tasks, Verification items, and Success Criteria checked with stored evidence |
| Roadmap state | T14.F05 checked; T14 remains In Progress, its remaining feature rows and master rollup are unchanged |
| Heavyweight evidence | Existing fresh mutation and single gate/goal reports retained; no additional benchmark or mutation run |
| Documentation checks | `make check-docs` exited 0 after correcting the initially rejected `Workflow:` Notes prefix to permitted `Cost:`; separate `make roadmap-check` and `git diff --check` exited 0. Aqua timestamp warnings were nonfatal. |

| Required / used role | Model / effort |
| --- | --- |
| Orchestrator | Sol `gpt-5.6-sol` / `medium` |
| Separate persistent spec owner and advisor | Astra `gpt-6-astra` / `xhigh` |
| Single persistent implementer and remediator | Astra `gpt-6-astra` / `xhigh` |
| Fresh final reviewer | Astra `gpt-6-astra` / `xhigh` |

| Workflow telemetry | Reading |
| --- | --- |
| Completed advisor consultations | 9; decisive guidance and acceptance recorded above |
| Final review | P1=0 / P2=0 / P3=0 |
| Post-review remediation | One test-only pass; no production remediation |
| Requirement corrections | 0; explicit no-epoch-re-pin wording was a documentation clarification, not a requirement change |
| Completed user interventions | 1: direct authorization for local planning and implementation commits |
| Verification exception | 1 sandbox exception: localhost websocket binding was unavailable; resolved by full host rerun, with no waiver or skip |
| Task-total usage | `usage unavailable`: native goal usage is stale at an earlier blocked state, and no task-total including subagents is exposed; no stale partial total or account rate-limit figure is used |
