# T13.F02 — Recruitment paths and replicated baseline readings

## Implementation verification

The observation uses eight fresh production ticks per subject, default runtime,
mutation and lifecycle charges, and only the specified fixture-world changes.
The complete historical drift walk and the three ecological profiles are
unchanged. The report-level experiment has its own configuration digest and
wall-clock field; gate and older reports remain explicitly unmeasured.

| Verification coverage | Result |
| --- | --- |
| Applied scenes and constructed paths | Both backends, matched starts/history, exact-copy activation and stationary wrong actions covered. |
| Pure invariants and replay | Selection, Wilson intervals, retention, complete genome deltas, reduced supply and independent production-engine sibling replay covered. |
| Missing observations | Removed-subject energy and memory are absent, not observed zero; paired memory effect is unmeasured if any required observation is missing. Living/dead controls and property tests preserve the distinction, including a measured difference followed by an unavailable scene. |
| Report wiring | Goal-only, once-per-world-set observation; deterministic reduced results and explicit historical absence covered. |
| TDD and regression files | Behavior changes used red/green tests; no new proptest regression file was generated. |

Final implementation checks, 2026-09-12:

| Command | Result |
| --- | --- |
| `cargo check --workspace --all-targets` | Exit 0, all workspace targets checked. |
| `cargo test -p v3-core recruitment_paths` | Exit 0, 12 unit and 2 integration tests passed; unrelated tests filtered. |
| `cargo test -p v3-core --test recruitment_paths` | Exit 0, 2 tests passed. |
| `cargo test -p v3-cli recruitment_paths` | Exit 0, 2 tests passed, including once-per-world-set wiring and identical reduced results. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Exit 0, no warnings. |
| `make roadmap-check` | Exit 0, validation passed; repeated after the verification record update. |

Documentation-only checks, 2026-09-12:

| Command | Result |
| --- | --- |
| `make roadmap-check` | Exit 0; validation passed, including the spec prose budget. |
| `make check-docs` | Exit 0; roadmap, policy, quality and documentation gate regression checks passed. |
| `git diff --check` | Exit 0; no whitespace errors. |

Completion checks, 2026-09-13; current tested commit
`0448db5c3fab6dd4e331dc37de9e84fa93412ead`:

| Command / environment | Result and evidence |
| --- | --- |
| Clean rebase | Main advanced from `5262acf427f0e0e7df13881c65e6832a0b1ef9c9` to `e635c3df93e1a70253b2daa5fdb0ed391de5b405`; `codex/t13-f02` rebased with no conflicts. |
| `make check`, authorized outside-sandbox, rebased `0448db5c` | Captured exit 0 on exactly `0448db5c3fab6dd4e331dc37de9e84fa93412ead`, with no content changes during the run. Log: `/private/tmp/t13-f02-make-check-rebased.log`, 2,162 lines, through final dependency and skill scans. |
| `make check`, pre-rebase sandbox, `575b8ab8` | Exit 2; seven v3-server WebSocket tests fail at `tests/server.rs:36` while binding a local listener: `PermissionDenied (Operation not permitted)`. No feature assertion fails. Log: `/private/tmp/t13-f02-make-check.log`. |
| `make check`, pre-rebase authorized outside-sandbox retry | Exit 0 on `575b8ab898a791194b6c79006150ca65d5bb91fa`, unchanged from the sandbox attempt; all Rust/server suites, frontend 61 files / 322 tests, build and scans pass. Log: `/private/tmp/t13-f02-make-check-escalated.log`, 2,159 lines, through final dependency and skill scans. Advisor consultation 8 approves this environment retry, not a waiver or altered command. |
| Closure `make roadmap-check` / `make check-docs` / `git diff --check` | Exit 0 on the documentation-only closure edits; no production content change. |
| Log availability | Local files, inspected 2026-09-13; persistence is not promised. |

## Mutation gate

The required fresh command was `MUTANTS_ITERATE=0 make rust-mutants`. Its
wrapper preamble was `rust-mutants: fresh run; no prior mutant results reused`.
Fresh terminal counts: 322 mutants; 163 caught; 106 missed; 53 unviable; 0
timed out. Output was written to
`/Users/istefanek/.local/share/petri-tools/mutants/t13-f02/mutants.out`.
The launching handle detached before returning an exit status; the terminal
counts are the captured command result.

Two permitted `MUTANTS_ITERATE=1 make rust-mutants` feedback passes reused that
directory after test-only strengthening. The first killed 86 of the 106 fresh
survivors. The second reused 302 caught/unviable results and retested 20,
killing 8 and leaving the 12 equivalent mutants below. Consequently the
fresh `missed.txt`, `timeout.txt`, and `outcomes.json` were superseded; the
captured fresh counts and survivor set, incremental `previously_caught.txt`,
and terminal 12-entry `missed.txt` support this adjudication. No second fresh
run is required: production code, test selection, tool configuration,
thresholds, exclusions, and existing tests were unchanged; only tests were
added or strengthened.

The following 94 fresh survivors were killed by the strengthened tests:

```text
crates/v3-core/src/neighborhood/recruitment_paths/mod.rs:49:31: replace != with == in evaluate_with_config
crates/v3-core/src/neighborhood/recruitment_paths/mod.rs:49:47: replace != with == in evaluate_with_config
crates/v3-core/src/neighborhood/recruitment_paths/mod.rs:49:43: replace & with | in evaluate_with_config
crates/v3-core/src/neighborhood/recruitment_paths/mod.rs:49:63: replace != with == in evaluate_with_config
crates/v3-core/src/neighborhood/recruitment_paths/mod.rs:49:59: replace & with | in evaluate_with_config
crates/v3-core/src/neighborhood/recruitment_paths/mod.rs:49:59: replace & with ^ in evaluate_with_config
crates/v3-core/src/neighborhood/recruitment_paths/mod.rs:97:17: replace + with - in evaluate_with_config
crates/v3-core/src/neighborhood/recruitment_paths/mod.rs:97:17: replace + with * in evaluate_with_config
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:23:5: replace fingerprint -> String with String::new()
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:23:5: replace fingerprint -> String with "xyzzy".into()
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:67:46: replace && with || in uses
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:64:5: replace uses -> Vec<ModuleUse> with vec![]
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:72:43: replace == with != in uses
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:91:54: replace - with + in uses
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:91:54: replace - with / in uses
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:94:41: replace |= with &= in uses
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:94:59: replace != with == in uses
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:95:42: replace |= with &= in uses
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:98:47: replace != with == in uses
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:100:43: replace |= with &= in uses
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:100:61: replace != with == in uses
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:176:61: replace - with + in propose_siblings
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:176:61: replace - with / in propose_siblings
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:200:69: replace && with || in propose_siblings
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:200:48: replace && with || in propose_siblings
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:200:90: replace >= with < in propose_siblings
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:204:13: replace && with || in propose_siblings
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:203:47: replace > with == in propose_siblings
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:203:47: replace > with < in propose_siblings
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:204:16: delete ! in propose_siblings
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:260:17: replace && with || in propose_siblings
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:259:17: replace && with || in propose_siblings
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:258:17: replace && with || in propose_siblings
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:260:55: replace >= with < in propose_siblings
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:260:51: replace + with - in propose_siblings
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:340:23: replace <= with > in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:365:27: replace <= with > in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:403:27: replace == with != in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:403:51: replace + with - in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:403:51: replace + with * in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:406:49: replace == with != in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:414:57: replace - with + in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:414:57: replace - with / in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:426:23: replace == with != in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:442:53: replace + with - in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:442:53: replace + with * in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:442:36: replace * with + in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:442:36: replace * with / in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:445:50: replace + with - in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:445:50: replace + with * in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:445:33: replace * with + in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:445:33: replace * with / in lineage
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:463:60: replace == with != in summary
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:507:69: replace != with == in pair
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:512:64: replace == with != in pair
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:518:21: replace - with + in pair
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:520:21: replace - with + in pair
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:521:70: replace - with + in pair
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:524:67: replace != with == in pair
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:525:54: replace - with + in pair
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:525:54: replace - with / in pair
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:530:58: replace != with == in pair
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:573:61: replace == with != in observe
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:600:27: replace + with * in observe
crates/v3-core/src/neighborhood/recruitment_paths/experiment.rs:601:32: replace == with != in observe
crates/v3-core/src/neighborhood/recruitment_paths/fixtures.rs:81:26: replace += with *= in sites
crates/v3-core/src/neighborhood/recruitment_paths/fixtures.rs:82:29: replace += with *= in sites
crates/v3-core/src/neighborhood/recruitment_paths/fixtures.rs:85:39: replace += with *= in sites
crates/v3-core/src/neighborhood/recruitment_paths/fixtures.rs:86:36: replace += with *= in sites
crates/v3-core/src/neighborhood/recruitment_paths/fixtures.rs:89:43: replace += with *= in sites
crates/v3-core/src/neighborhood/recruitment_paths/fixtures.rs:90:42: replace += with *= in sites
crates/v3-core/src/neighborhood/recruitment_paths/fixtures.rs:91:35: replace += with *= in sites
crates/v3-core/src/neighborhood/recruitment_paths/fixtures.rs:106:21: replace + with - in sites
crates/v3-core/src/neighborhood/recruitment_paths/fixtures.rs:106:21: replace + with * in sites
crates/v3-core/src/neighborhood/recruitment_paths/fixtures.rs:101:21: replace + with * in sites
crates/v3-core/src/neighborhood/recruitment_paths/fixtures.rs:96:21: replace + with * in sites
crates/v3-core/src/neighborhood/recruitment_paths/fixtures.rs:104:60: replace + with * in sites
crates/v3-core/src/neighborhood/recruitment_paths/fixtures.rs:293:29: replace && with || in stage
crates/v3-core/src/neighborhood/recruitment_paths/fixtures.rs:293:54: replace > with >= in stage
crates/v3-core/src/neighborhood/recruitment_paths/fixtures.rs:483:23: replace - with / in starts_from_paths
crates/v3-core/src/neighborhood/recruitment_paths/fixtures.rs:490:5: replace direction with ()
crates/v3-core/src/neighborhood/recruitment_paths/records.rs:111:9: replace TaskReading::dispatched -> BTreeSet<NodeId> with BTreeSet::new()
crates/v3-core/src/neighborhood/recruitment_paths/records.rs:119:28: replace += with *= in TaskReading::summary
crates/v3-core/src/neighborhood/recruitment_paths/records.rs:120:27: replace += with *= in TaskReading::summary
crates/v3-core/src/neighborhood/recruitment_paths/records.rs:121:31: replace += with *= in TaskReading::summary
crates/v3-core/src/neighborhood/recruitment_paths/records.rs:122:29: replace += with -= in TaskReading::summary
crates/v3-core/src/neighborhood/recruitment_paths/records.rs:122:29: replace += with *= in TaskReading::summary
crates/v3-core/src/neighborhood/recruitment_paths/records.rs:208:13: replace || with && in GenomeDelta::apply
crates/v3-core/src/neighborhood/recruitment_paths/records.rs:202:13: replace || with && in GenomeDelta::apply
crates/v3-core/src/neighborhood/recruitment_paths/records.rs:381:38: replace / with * in estimate
crates/v3-core/src/neighborhood/recruitment_paths/records.rs:384:41: replace / with % in estimate
crates/v3-core/src/neighborhood/recruitment_paths/records.rs:384:41: replace / with * in estimate
crates/v3-core/src/neighborhood/recruitment_paths/records.rs:384:29: replace * with / in estimate
crates/v3-core/src/neighborhood/recruitment_paths/records.rs:384:36: replace - with + in estimate
```

The terminal 12 survivors are equivalent; each disposition names the reason
the mutation cannot change this experiment's observable report:

| Survivor | Disposition |
| --- | --- |
| `mod.rs:94:71: replace > with >= in evaluate_with_config` | Equivalent: `run_tick` removes every subject whose energy is nonpositive, so a retained creature can never expose the zero-energy boundary to this predicate. |
| `experiment.rs:203:47: replace > with >= in propose_siblings` | Equivalent: the exhaustive maximum valid batch/lineage/generation seed space contains no live useful-module proposal whose child score equals its starting score. |
| `experiment.rs:218:21: delete match arm MutationDomain::Graph in propose_siblings` | Equivalent: for every fixed proposal discard with a target, fallback target lookup resolves the same Graph backend; targetless discards are skipped before the match. |
| `experiment.rs:219:21: delete match arm MutationDomain::Vm in propose_siblings` | Equivalent: for every fixed proposal discard with a target, fallback target lookup resolves the same VM backend; targetless discards are skipped before the match. |
| `experiment.rs:224:51: replace == with != in propose_siblings` | Equivalent: across the exhaustive fixed proposal universe, the alternate located node has the same backend as the matched target whenever fallback attribution contributes. |
| `experiment.rs:234:32: replace += with -= in propose_siblings` | Equivalent: exhaustive production observation records zero unresolved targeted discards, so the unresolved increment is never executed. |
| `experiment.rs:234:32: replace += with *= in propose_siblings` | Equivalent: exhaustive production observation records zero unresolved targeted discards, so the unresolved increment is never executed. |
| `experiment.rs:260:51: replace + with * in propose_siblings` | Equivalent: the exhaustive fixed proposal universe contains no otherwise-viable child exactly one score point below its parent, the only boundary where these expressions differ. |
| `experiment.rs:407:25: replace && with \|\| in lineage` | Equivalent: no retained discovery in the exhaustive fixed run has another tracked module whose partial identity/presence can change the exact discovered module's presence result. |
| `experiment.rs:406:25: replace && with \|\| in lineage` | Equivalent: no retained discovery in the exhaustive fixed run has another tracked module whose partial identity/presence can change the exact discovered module's presence result. |
| `experiment.rs:405:33: replace == with != in lineage` | Equivalent: no retained discovery in the exhaustive fixed run has another tracked module whose partial identity/presence can change the exact discovered module's presence result. |
| `fixtures.rs:96:21: replace + with - in sites` | Equivalent: every authored Graph fixture has zero output-sink input edges, so this changes addition of zero to subtraction of zero. |

No survivor was deferred.

| Mutation-specialist verification | Result |
| --- | --- |
| `cargo test -p v3-core recruitment_paths` | Exit 0; 20 unit and 2 integration tests passed. |
| `cargo clippy -p v3-core --all-targets -- -D warnings` | Exit 0; no warnings. |
| `make roadmap-check` | Exit 0; validation and prose budget passed. |
| `git diff --check` | Exit 0; no whitespace errors. |

No production defaults, founder behavior or tick-loop mechanics changed;
viability-first was not triggered. The completed gates are recorded above.

| Closure record | Value |
| --- | --- |
| Orchestrator | `gpt-5.6-sol`, `medium` |
| Persistent spec owner/advisor | `gpt-6-astra`, `xhigh` |
| Persistent implementer | `gpt-6-astra`, `xhigh` |
| Fresh reviewer | `gpt-6-astra`, `xhigh` |
| Mutation specialist | `gpt-5.6-sol`, `medium` |
| Benchmark specialist | `gpt-5.6-terra`, `high` |
| Advisor consultations | 8; planning/readiness excluded. |
| Initial review findings | P1=1, P2=2, P3=1 |
| Final review disposition | No P1 or new findings after one documentation-only pass; one typed-outcome P2 deferred to T15.F01. |
| Remediation passes | Production: 0; post-review documentation: 1; mutation test-only incremental feedback: 2. |
| Mutation disposition | 94 fresh survivors killed / 12 equivalent / 0 deferred / 0 timeout. |
| Requirement corrections | 1: local-only raw goal evidence and Planned, unscheduled T15. |
| User interventions | Explicit local-commit authorization confirmation; storage-policy correction/clarification; repeated mutation-approval messages needed to unblock tool prompts. |
| Task-specific usage | Unavailable |

## Observation interpretation

The five Task A starting forms have identical always-NoOp battery actions and
4/8 task correctness. Their sizes, carrying charges and mutable sites differ.
Graph and VM constructed blank/copy paths reach 8/8 Task A correctness. The
changed-task arms share an explicitly recorded Task A-correct history; the
prepared module changes only its dormant cue and eastward action material.
Prepared-module activation reaches 8/8 Task B and 4/8 Task A in the fixture.
These are authored possibility controls, not production discovery counts.

Every complete production mutation call supplies one sibling record, including
zero-event calls and selected-but-inapplicable operator discards. Construction
registration is kept separate from those proposal totals. The tracker retains
the incumbent separately from the added scaffold and later modules. Full
battery and module facts are recorded at depths 0/32/48; task survival and the
battery's dead/silent/changed classification have separate meanings.

Mutation replay stores exact whole-birth before/after node values, insertion,
deletion, order and entry-node changes alongside the ordered event records and
seed. The first observed successful path includes its unselected terminal
sibling when applicable. This resolution does not attribute a multiply changed
field to an individual event inside the same birth.

The conditional VM sham is not applicable: this feature introduces no
mutation-RNG draw or constructor intervention. Pairing records the first
genotype/transition/RNG divergence; common seeds alone do not establish common
mutations after eligible sites diverge. Any unresolved transient target backend
is reported rather than inferred without evidence.

## Closure measurements

The original benchmark profiles were each launched exactly once, sequentially, through
the unchanged `scripts/bench-wait` host preflight with the repository tool
paths prefixed to `PATH`. The measurement commit was
`7f4c54fb78cb16c7bf462e46d85a017f4addf8c5`; no competing benchmark, server,
build, test, or mutation process was started by this specialist. This original
measurement identity is preserved after the clean rebase and full recheck.

| Profile and command | Exit / report | Comparison and thresholds |
| --- | --- | --- |
| `make bench PROFILE=gate FEATURE=t13-f02-recruitment-paths-and-replicated-baseline` | Exit 0; [gate report](../features/t13-f02-recruitment-paths-and-replicated-baseline.json) | `comparison.severe=false`. All six counters are `ok` against both `remove-complementary-nutrition.json` and T14.F03. Wall/creature-tick is `ok` against the epoch (+11.755%) and `flag` against T14.F03 (+30.659%); this is a new prior-closure flag, not a threshold change. |
| `make bench PROFILE=goal FEATURE=t13-f02-recruitment-paths-and-replicated-baseline` | Exit 0; local raw artifact identified below, not committed by user exception | `comparison.severe=false`. All six counters are unchanged (`ok`) against T14.F03; wall/creature-tick is `ok` against the goal epoch (+14.188%) and T14.F03 (+10.102%). Against the T12.F04 epoch, `plasticity_updates` is `flag` (+40.887%) while all other counters are `ok`; it is inherited because the T14.F03 comparison is exactly 0.000000%. |

Both reports were generated at the expected Makefile-derived paths. The benchmark executor
detached its output handle after launching each child process, but the gate
artifact and its `comparison.severe=false` record establish its completed
non-severe CLI outcome; the CLI writes the report before its only post-write
nonzero (severe) exit. The goal artifact likewise records a non-severe outcome.
No epoch was re-pinned, and no baseline or threshold was changed.

User storage exception, 2026-09-12: one intervention and requirement correction
excludes the 356,934,683-byte goal report from Git. These single-run measurements
remain the evidence; no compaction or benchmark rerun is required. The small
gate report remains a closure artifact under the current contract. Raw source
availability and checksum were checked locally on 2026-09-12; this temporary
path is not a persistence promise or a portable link. A checksum identifies
the source, not independent proof of its contents. Claims needing omitted
detail must be checked against the raw source while available, otherwise marked
unverified. T15.F01 owns the broader policy and summary implementation later.

| Local raw goal provenance | Value |
| --- | --- |
| Path | `/Users/istefanek/projects/petri/tmp/bench-artifacts/t13-f02/t13-f02-recruitment-paths-and-replicated-baseline-goal.json` |
| Bytes | `356934683` |
| SHA-256 | `bc7f039febdae9c6e32e9f5f83485d3705f0a1ff30181dbba32fa509fa6bb849` |
| Series disposition | Gate is indexed; goal is omitted by exception. `goal_worlds.closed` retains T14.F03 as the latest available committed goal reference, not a T13.F02 measurement. No epoch changes. |

| Observation timing | Measurement / individual cap verdict |
| --- | --- |
| Gate founder neighborhood | 51.388 ms; passes <10 s. |
| Gate evolved neighborhood | Unmeasured/0; no drift or recruitment measurement is present. |
| Goal founder neighborhood | 122.067 ms; passes <10 s. |
| Goal evolved neighborhood | 550.998 ms summed across seeds; passes <180 s. |
| Goal drift | 21,265.834 ms total; therefore passes <30 s per world. |
| Goal recruitment paths | 7,750.448 ms; passes <120 s. |
| Goal final-state reading | 858.761 ms; no separate cap. |

| Overall timing / investigation | Evidence and limitation |
| --- | --- |
| Goal simulation timing | 515,641.938 ms; this is simulation timing, not full goal elapsed time. |
| Goal through completed observations and report writing | `<9m40s`, conservatively bounded by preceding gate completion `2026-09-13T04:06:19Z` and complete goal artifact mtime `2026-09-13T04:15:58Z`. Exact process-exit duration was not captured. The 15-minute investigation threshold is reviewed against this bound, separately from individual observation caps. |
| New gate wall-time flag versus T14.F03 | Simulation wall 570.409→745.293 ms, +30.659%; deterministic results and work counters match, and recruitment observation is absent in gate. Cause remains unattributed; no host-noise attribution is established. The flag is non-severe. |

The `recruitment-paths-v1` experiment identifies configuration
`sha256:c9ebc462c9c6e74aa34e6935055e70d65708c83d3b3d236b6e60e4c4764f68e9` and
ran the fixed 18 arms, four batches, eight lineages, 32 discovery generations,
16 follow-up generations, and two siblings: 55,296 complete proposals. It
recorded 30,726 attempted and 27,768 applied events, 2,958 skips (all Graph
no-eligible-node), and 30,888 zero-event births. All blank/copy Task A and
unprepared Task B arms were null: 0/32 proposal discoveries (Wilson 95%
0.000-0.107) under both policies. The prepared Task B arms each discovered
19/32 (0.594, 0.423-0.745): Graph drift retained 14/32 and useful-retained
1/32; Graph selection 19/32 and 16/32; VM drift 15/32 and 2/32; VM selection
19/32 and 15/32. Batch proposal-discovery numerators range from 4–7 per eight
lineages: Graph under both policies `[5, 4, 6, 4]`; VM under both policies
`[4, 4, 7, 4]`. Complete per-batch intervals and paths remain only in the local
raw report identified above, not in the committed readings.

Paired policy comparisons are not causal treatment estimates: 20/32 Graph and
25/32 VM prepared lineages first diverged in parent history, with mutation/RNG
divergence following where recorded. Each prepared-policy pair has zero summed
proposal-discovery difference, while post-divergence retention/useful
differences remain reported as uncertainty rather than superiority. The sham
RNG control remains not applicable: observations consume no mutation RNG.
Construction-only registration remains excluded from all proposal totals.

`Event.outcome` is a debug-derived string. Current accounting and replay use
typed engine records; no parser exists. T15.F01 owns the future persisted
representation/summary; T13.F03/F06 consumers must not parse the debug string.
