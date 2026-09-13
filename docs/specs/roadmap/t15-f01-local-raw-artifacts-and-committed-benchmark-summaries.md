# T15.F01 — Local Raw Artifacts and Committed Benchmark Summaries

**Status**: Complete
**Last updated**: 2026-09-13
**Feature**: T15.F01
**Track**: [T15 — Benchmark Evidence Storage](../../roadmaps/t15-benchmark-evidence-storage.md)

## Goal

The world's measured outcomes remain readable and comparable across closures
from concise committed summaries. Full gate, goal and sweep records stay in
local ignored storage, with verifiable provenance that states when their
availability was checked.

## Non-Goals

- Historical bulk migration, report deletion, Git history rewriting and remote
  updates belong to T15.F02.
- No simulation, default, RNG, telemetry definition, indicator, recipe, trial
  count or experiment change; no new campaign, hosting service, storage
  framework or progress-page redesign.

## Inputs and Invariants

Sources of truth are `Makefile::bench`, `crates/v3-cli/src/main.rs::run_bench`,
`crates/v3-cli/src/bench.rs` (`Report`, `case_readings`, comparison loading and
series selection), recruitment report types in
`crates/v3-core/src/neighborhood/recruitment_paths/records.rs`,
`docs/progress/benchmark-series.json`, and `docs/progress/index.html`.
[T14.F01](t14-f01-closure-comparison-and-indicator-coverage-integrity.md)
supplies self-reference exclusion, explicit reference-absence causes, temporal
memory comparison readings and indicator definition tokens. All remain intact.

**Storage and commands.** Both `make bench` and direct `v3-cli bench` use the
same output contract:

| Output | Default |
| --- | --- |
| Full gate report | `<main-checkout>/.bench-artifacts/<feature>/gate.json` |
| Full goal report | `<main-checkout>/.bench-artifacts/<feature>/goal.json` |
| Full sweep report | `<main-checkout>/.bench-artifacts/<feature>/sweep.json` |
| Gate summary | Calling checkout's `docs/progress/features/<feature>.json` |
| Goal summary | Calling checkout's `docs/progress/features/<feature>-goal.json` |
| Sweep summary | Calling checkout's `docs/progress/features/<feature>-sweep.json` |

`OUT` / `--out` overrides the raw destination only. Sweeps accept a feature
label for defaults; an unlabelled sweep with an explicit output remains usable
and places its generated summary beside that output under a distinct name.
Document a separate summary-path override for ad hoc runs. Default raw storage
resolves the main checkout through Git metadata, including from linked
worktrees and paths containing spaces; it never assumes the current checkout
is main. Missing Git context without explicit output paths is a clear error.
The main checkout's `/.bench-artifacts/` is ignored. No new full report is
committed, including small gate reports; existing historical reports remain
unchanged in this feature.

Raw and summary outputs must be distinct, including normalized paths and
existing symlink aliases. Comparison excludes references resolving to either
output, preserving T14.F01's absence semantics. Explicit raw output must not
overwrite a declared comparison reference. Report-write failure is an error;
do not announce a complete artifact pair or refresh verified availability
unless the raw bytes were successfully written and hashed. A severe completed
run still retains both artifacts and its nonzero benchmark status.

**Versioned summary.** Use a distinguishable summary kind and version, separate
from the existing full-report schema. Generate a deliberate projection of the
existing measurements, retaining their numeric representation and definition
tokens. A summary is self-sufficient for comparison and headline rendering;
neither operation opens its local raw path. Unknown summary versions and
malformed declared references fail clearly rather than loading as empty data.

| Evidence | Retained in the summary |
| --- | --- |
| Run identity | Feature, original measured revision and generation time, dirty-state evidence when captured, exact executable/arguments and working directory, command/exit evidence with its source, profile, seeds, effective config and recipe identities, indicator and counter-definition versions |
| Compute and verdict | Per-seed counters and totals, all six normalized counters, host/thread and relevant wall readings, complete comparison values and levels, reference paths and available measured identities, reference absence, severe and inherited-status evidence, threshold and wall-cap values with their provenance |
| Worlds and indicators | Each world's identity, persistence headline readings and bounded checkpoint series needed by the existing page, structure and lineage readings, memory/temporal-memory and exposure readings, applied-world headline counts/fractions, founder/evolved neighborhood aggregates and drift checkpoint aggregates |
| Recruitment experiment | Task/config/size/seed identities and limitations; actual proposal totals and outcome/opportunity counts; arm and batch estimates with numerators, denominators and existing intervals; compact lineage and paired-lineage outcomes needed to retain the experiment's uncertainty and pairing limits |
| Claim extracts | A documented deterministic, fixed-size selection of source-located scalar/count extracts sufficient to spot-check representative totals and outcomes; source locations resolve in the hashed raw report |
| Raw provenance | SHA-256 of exact stored bytes including formatting, byte count, resolved local path, availability state and verification time; omitted-detail categories are explicit |

Full genomes, genome deltas, per-proposal rows, scene/signature/route/state
traces and replay paths stay raw. Claim extracts do not reintroduce these
payloads. Summary detail is bounded by cases, fixed checkpoints and experiment
arms/batches/lineages, rather than proposal count or genome/trace size. Existing
aggregate report structures are reused where they satisfy this boundary.
Report summary/raw byte counts in this feature's readings so concision is
visible, not asserted from JSON whitespace changes.

An absent or `Undefined` reading remains unmeasured; measured zero retains its
denominator. Omitted detail is unavailable, never a fabricated empty or zero
observation. Inherited flags retain their source; unknown historical command,
exit, threshold, cap or inheritance evidence is explicitly unknown rather
than inferred from today's settings or the absence of a severe flag. For new
runs, record the CLI completion status and distinguish it from an observed
outer `make`/wrapper process status when that evidence is supplied.

**Conversion and consumers.** Provide a documented CLI conversion command
that reads an existing full artifact and emits the same summary format without
running simulations, assays or comparisons again. The same raw bytes and
explicit provenance inputs produce byte-identical summary bytes. Keep original
measurement identity separate from converter identity; verification timestamps
are captured provenance inputs, not an implicit relabeling of an old run.
Conversion never overwrites its input. Raw availability means verified at the
recorded time on this machine, not durable storage or a usable download URL;
later missing raw files do not invalidate committed measurements.

Historical full reports and summaries feed one comparison path with identical
normalized counters, per-case readings, deltas, severity and host-sensitive
wall comparisons for the same source. Retain original comparison inputs or
lossless comparison values; rounding a per-case ratio before calculating its
delta must not change the result. Preserve strict profile compatibility,
per-case recipe-change and absent-case labels, missing-reference errors and
value-only readings. Do not reconstruct a fictitious full `Report` with zeroed
omitted fields to obtain compatibility. Update current consumers, particularly
the existing static progress page, to display retained information identically
and label unavailable detail. Series entries keep their existing path format;
new closure entries point to summaries.

**Research decision (2026-09-13).** Extend the existing Rust CLI and comparison
types using Serde/serde_json and the workspace's `sha2` dependency. A separate
JSON-processing script is credible but would duplicate projection and
comparison semantics; compressing or pruning arbitrary JSON cannot establish
the required evidence contract. Serde documents both explicit tags and
first-matching untagged decoding, supporting a distinguishable summary version
with an explicit historical-report branch
([Serde](https://serde.rs/enum-representations.html)). Git's common-directory
and worktree porcelain interfaces supply repository discovery without a new
worktree convention ([rev-parse](https://git-scm.com/docs/git-rev-parse),
[worktree](https://git-scm.com/docs/git-worktree)). The already locked sha2
0.10.9 API hashes exact bytes incrementally
([sha2](https://docs.rs/sha2/0.10.9/sha2/)). The remaining tradeoff is maintaining
an explicit projection as report fields evolve; targeted consumer/parity
checks own that boundary, not a generic storage framework.

## Implementation Tasks

- [x] Add versioned summary projection and deterministic full-artifact
  conversion, including provenance and compact experiment evidence.
- [x] Make full reports and summaries valid comparison references through the
  existing comparison rules, retaining all absence and compatibility semantics.
- [x] Route raw output into the main checkout's ignored artifact root, retain
  explicit overrides, and emit a separate summary for gate, goal and sweep.
- [x] Adapt existing report consumers and update the shared workflow, Codex
  adapter, feature template and output documentation to this storage contract;
  closure review checks summary provenance and the absence of staged raw data.
- [x] Record this feature's summaries, concise readings and series entries;
  complete verification and truthful spec/track status updates.

## Verification

- [x] Focused CLI/library checks establish full/summary comparison parity,
  profile/case/reference errors, both-output self-reference filtering,
  measured-zero/unmeasured fidelity, experiment aggregation and bounded
  extracts, deterministic conversion, exact byte hashes and write failures:
  `cargo test -p v3-cli` -> concise results in this feature's readings.
- [x] Main-checkout and linked-worktree output checks cover gate/goal/sweep
  defaults, paths with spaces, explicit raw/summary overrides and distinct
  destinations; existing shell automation checks pass. Commands and temporary
  artifact paths are recorded in the readings.
- [x] Existing progress-page consumers load a mixture of full reports and
  summaries, retain headline values and expose unavailable detail. The check
  and its results are recorded in the readings.
- [x] A historical full report converts twice from fixed provenance without
  rerunning observations; its original identity and comparisons remain intact.
  Stored summary size, source hash, byte count and representative claim
  locations are spot-checked and recorded in the readings.
- [x] `make check` exited 0 on final-code commit `8159d211`; `make
  roadmap-check` passed, and closure inspection confirms no new full benchmark
  artifact is staged. Results live in the readings.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` ran once on final production
  content in `fresh` mode: `126 mutants tested in 29m: 21 missed, 94 caught, 11
  unviable`; no mutant timed out. Output:
  `/Users/istefanek/.local/share/petri-tools/mutants/t15-f01/mutants.out`.
  Test-only remediation used two permitted incremental feedback passes: `21
  mutants tested in 5m: 3 missed, 18 caught`, then `3 mutants tested in 3m: 2
  missed, 1 caught`. The final two survivors are equivalent; all other fresh-run
  survivors were killed. No second fresh run was required because production
  code, test selection/tool configuration, and existing tests were unchanged.

| Fresh-run survivor | Disposition |
| --- | --- |
| `crates/v3-cli/src/bench.rs:3891:9: replace ComparisonInputs::validate -> Result<(), String> with Ok(())` | Killed: an invalid numeric reading under an undeclared case key must still be rejected by summary validation. |
| `crates/v3-cli/src/bench/artifacts.rs:147:44: replace += with -= in mesh_summary` | Killed: exact two-genome mesh totals assert the genome count. |
| `crates/v3-cli/src/bench/artifacts.rs:136:5: replace mesh_summary -> Value with Default::default()` | Killed: exact mesh-summary projection asserts every retained aggregate. |
| `crates/v3-cli/src/bench/artifacts.rs:147:44: replace += with *= in mesh_summary` | Killed: exact two-genome mesh totals assert the genome count. |
| `crates/v3-cli/src/bench/artifacts.rs:157:49: replace += with -= in mesh_summary` | Killed: exact nonzero totals assert every measured mesh counter. |
| `crates/v3-cli/src/bench/artifacts.rs:157:49: replace += with *= in mesh_summary` | Killed: exact nonzero totals assert every measured mesh counter. |
| `crates/v3-cli/src/bench/artifacts.rs:160:48: replace += with -= in mesh_summary` | Killed: exact route-variation count is asserted. |
| `crates/v3-cli/src/bench/artifacts.rs:160:48: replace += with *= in mesh_summary` | Killed: exact route-variation count is asserted. |
| `crates/v3-cli/src/bench/artifacts.rs:160:92: replace == with != in mesh_summary` | Killed: mixed true/false route-variation rows assert the true-row count. |
| `crates/v3-cli/src/bench/artifacts.rs:170:5: replace neighborhood -> Value with Default::default()` | Killed: exact neighborhood projection asserts retained founder, battery, and evolved fields. |
| `crates/v3-cli/src/bench/artifacts.rs:170:8: delete ! in neighborhood` | Killed: the object fixture asserts raw fields are omitted and evolved rows are projected. |
| `crates/v3-cli/src/bench/artifacts.rs:261:5: replace constructed_stage -> Value with Default::default()` | Killed: recruitment projection asserts every retained field of a non-default constructed stage. |
| `crates/v3-cli/src/bench/artifacts.rs:404:5: replace claims -> Vec<Claim> with vec![]` | Killed: exact scalar claims are asserted. |
| `crates/v3-cli/src/bench/artifacts.rs:418:100: replace && with \|\| in claims` | Killed: exact claims exclude fixed-location arrays and objects. |
| `crates/v3-cli/src/bench/artifacts.rs:418:82: delete ! in claims` | Killed: exact claims exclude a fixed-location array. |
| `crates/v3-cli/src/bench/artifacts.rs:418:103: delete ! in claims` | Killed: exact claims exclude a fixed-location object. |
| `crates/v3-cli/src/bench/artifacts.rs:450:31: replace * with + in summarize` | Equivalent: for immutable stored artifact bytes, changing the internal read buffer from 65,536 to 65,540 bytes changes only chunk boundaries; the loop still compares and hashes every byte through EOF and returns the same result. |
| `crates/v3-cli/src/bench/artifacts.rs:535:13: replace \|\| with && in comparison_inputs_from_bytes` | Killed: a summary whose feature alone disagrees with its comparison-input identity is rejected. |
| `crates/v3-cli/src/bench/artifacts.rs:587:31: replace match guard e.kind() == std::io::ErrorKind::NotFound with true in resolved_path` | Killed: a permission-denied ancestor lookup is asserted to fail rather than be treated as an absent path. |
| `crates/v3-cli/src/bench/artifacts.rs:604:19: replace match guard e.kind() == std::io::ErrorKind::NotFound with true in same_path` | Equivalent: after `resolved_path` succeeds on a stable filesystem, each path is either canonicalized so identity metadata succeeds or absent so identity lookup returns `NotFound`; another error requires an external race outside this path-resolution contract. |
| `crates/v3-cli/src/bench/artifacts.rs:754:20: delete ! in measurement_evidence` | Killed: clean and untracked-dirty temporary Git worktrees assert opposite `dirty` values. |
- [x] Gate and goal summaries are stored at the paths below; raw provenance
  matches locally verified files and the series index points to the summaries.

Final review counts: P1 0, P2 1, P3 1. Post-review remediation passes: 1.

| Finding | Disposition |
| --- | --- |
| P2: artifact tests assumed progress inventory entries remain full reports | Resolved: one small, explicitly synthetic full-report fixture supplies literal decimals, measured zero and historical absences; conversion/hash/parity tests no longer read production progress artifacts. Focused suite: 18 passed. |
| P3: readings included consultation and implementation chronology | Resolved: chronology removed; concrete verification and numeric-fidelity results retained in tables. |

## Performance and Goal Impact

**Predeclaration — written before the run.** Storage/reporting only; no natural
analog or new environmental pressure applies. No simulation or observation
work counter, world reading or indicator is expected to move because of this
feature. Hashing, projection and writing add output work outside existing
simulation/assay timing. They do not justify a threshold change or epoch repin.
Comparison against a full artifact and its summary must produce identical
values and verdicts; historical nonzero deltas remain historical deltas.

| Profile | Command | References fixed before measurement |
| --- | --- | --- |
| Gate | `make bench PROFILE=gate FEATURE=t15-f01-local-raw-artifacts-and-committed-benchmark-summaries` | `docs/progress/features/remove-complementary-nutrition.json` epoch; `docs/progress/features/t13-f02-recruitment-paths-and-replicated-baseline.json` last closed |
| Goal | `make bench PROFILE=goal FEATURE=t15-f01-local-raw-artifacts-and-committed-benchmark-summaries` | `docs/progress/features/t12-f04-baseline-world-set-goal.json` epoch; `docs/progress/features/t14-f03-applied-mortality-and-energy-accounting-goal.json` last stored closure |

Keep current work-counter flag/severe thresholds at >10%/>50% and
host-matching wall flag/severe thresholds at >25%/>100%; wall levels remain
nonfatal. Existing zero-reference severity, goal-indicator floors and
no-regression rules apply. Founder 10 s, evolved 180 s, drift 30 s per world
and the 15-minute goal investigation threshold remain unchanged. The standard
goal still measures Orchards, Canyon and Confluence and the existing recruitment
experiment once. T13.F02's omitted goal artifact is not silently substituted
as a reference. No second goal determinism run or historical rerun is added.

**Measured verdict.** The prescribed gate and single goal measurements both
completed with outer `make` and CLI exit 0 and `comparison.severe=false`.
All fixed-reference gate counters are `ok`. Goal retains the historical
T12.F04 `plasticity_updates` +40.886836% flag, while the fixed T14.F03
comparison is exactly 0.000000%; it is therefore inherited, non-severe, and
does not justify a threshold or epoch change. All unchanged observation caps
passed, including founder (70.652 ms / 10 s), evolved (379.418 ms / 180 s),
drift (17.775 s / 30 s per world), recruitment (6.593 s / 120 s), and the
474.859 s goal total / 15-minute investigation threshold. The readings record
paths, byte counts, hashes, measurement identities and local availability.

- Summaries: [gate](../../progress/features/t15-f01-local-raw-artifacts-and-committed-benchmark-summaries.json),
  [goal](../../progress/features/t15-f01-local-raw-artifacts-and-committed-benchmark-summaries-goal.json).
- Full readings: [`docs/progress/readings/t15-f01-local-raw-artifacts-and-committed-benchmark-summaries.md`](../../progress/readings/t15-f01-local-raw-artifacts-and-committed-benchmark-summaries.md).

## Success Criteria

- [x] New gate, goal and sweep runs retain full local artifacts and produce
  concise versioned summaries with truthful provenance and availability.
- [x] Historical/full and summary references yield the same comparison values
  and verdicts, and current consumers preserve retained readings.
- [x] Existing artifacts convert deterministically without a new measurement;
  experiment totals, uncertainty, missing values and claim extracts survive.
- [x] The shared workflow and template use this one contract, this closure
  commits summaries only, and all required checks are satisfied.

## Notes for AI Agents

- Decision: T15.F02 reuses this converter and summary format for historical
  migration; T15.F01 grants no history-rewrite or remote-update authority.
- Cost: Codex roles/settings were Sol medium orchestrator (requested; root-session
  metadata unavailable), persistent Astra xhigh spec owner/advisor, persistent
  Astra xhigh implementer, Terra high benchmark specialist, fresh Astra xhigh
  reviewer, and Sol medium mutation specialist. Eight advisor consultations;
  one implementation self-review pass, one post-review remediation pass, and
  one mutation test-only remediation pass; no production-code remediation,
  requirement correction, or user intervention after launch. Review P1/P2/P3
  counts 0/1/1, both advisory findings resolved. Task-specific usage unavailable.
