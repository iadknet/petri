# T13.F04 — Direct Graph Effect Activation readings

Tables and transcripts for the feature spec
[`docs/specs/roadmap/t13-f04-direct-graph-effect-activation.md`](../../specs/roadmap/t13-f04-direct-graph-effect-activation.md).

## Build-pass verification

| Command | Result |
| --- | --- |
| `cargo test -p v3-core --test viability` (run first) | ok, 24 passed, 0 failed |
| `cargo test -p v3-core` | ok, all suites pass (1428 lib, 19 baseline_worlds, 24 viability, remaining suites 0 failed) |
| `cargo test -p v3-core --test reproducibility` | ok, 3 passed |
| `cargo test -p v3-cli` | ok, 163 passed across suites, 0 failed, 1 ignored |
| `cargo clippy --workspace --all-targets` | clean, no warnings |
| `cargo fmt --all` | applied, no further diff |
| `cargo check --workspace --all-targets` | clean |
| `make roadmap-check` | pass |

## Re-pinned expectations

| Test | Change | Reason |
| --- | --- | --- |
| `crates/v3-core/tests/baseline_worlds.rs::legacy_default_short_run_identity` | `12330723111342885916` -> `11753828254793484309` | Wired zero-compute Graph modules now enter their visit, so the 30-tick run with births diverges in trajectory and charge. The new hash reproduced across three separate processes before re-pinning; the identity the test pins is unchanged. `AddRouteTarget` writes its weight-1 router-gate edge onto the selected source node; when that node is a zero-compute Graph (an earlier blank detour), that gate write was skipped before this repair. |

## Closure measurements

### Gate profile

| Item | Value |
| --- | --- |
| Command | `make bench PROFILE=gate FEATURE=t13-f04-direct-graph-effect-activation` |
| CLI exit (source: `measurement_evidence.cli_exit.code`, gate summary) | 0 |
| Outer-process exit (source: `echo "exit $?"` in `/private/tmp/t13-f04-bench-gate.log`) | 0 |
| `comparison.severe` (both references) | false |
| Raw report | `/Users/istefanek/projects/petri/.bench-artifacts/t13-f04-direct-graph-effect-activation/gate.json` |
| Raw SHA-256 | `98e9d835c0df608b111a42a74951265524a0b0d9fa30e3b5c18c994e3cc23feb` |
| Raw bytes | 94,086 |
| Summary path | `docs/progress/features/t13-f04-direct-graph-effect-activation.json` |
| Summary bytes | 96,923 |
| Verification time (`conversion.verified_at`) | 2026-09-14T03:50:49Z |
| Worktree dirty at gate measurement (`measurement_evidence.dirty`) | false |
| Gate wall time (`environment.wall_clock_ms_total`) | 627.908 ms |

Gate threshold verdicts (all `ok`), against epoch `remove-complementary-nutrition.json` and previous `t13-f03-mutation-target-applicability.json`:

| Counter | vs epoch delta% | epoch level | vs previous delta% | previous level |
| --- | --- | --- | --- | --- |
| mesh_hops | -0.213 | ok | 0.000 | ok |
| vm_steps | -1.317 | ok | 0.000 | ok |
| graph_relax_iters | -0.208 | ok | 0.003 | ok |
| plasticity_updates | -27.133 | ok | 0.000 | ok |
| actions_applied | -0.128 | ok | 0.000 | ok |
| births | 1.001 | ok | 0.000 | ok |

No epoch was re-pinned.

### Goal profile

| Item | Value |
| --- | --- |
| Command | `make bench PROFILE=goal FEATURE=t13-f04-direct-graph-effect-activation` |
| CLI exit (source: `measurement_evidence.cli_exit.code`, goal summary) | 0 |
| Outer-process (make) exit (source: `echo "exit $?"` appended to `/private/tmp/t13-f04-bench-goal.log`) | 0 |
| `comparison.severe` (top level and against the single reference) | false |
| Raw report | `/Users/istefanek/projects/petri/.bench-artifacts/t13-f04-direct-graph-effect-activation/goal.json` |
| Raw SHA-256 | `aa3e6fe73f8ab213ec7798ff52228c489662ca8253fd0863b4cd7c2aad32ad23` |
| Raw bytes | 350,744,293 |
| Summary path | `docs/progress/features/t13-f04-direct-graph-effect-activation-goal.json` |
| Summary bytes | 4,136,698 |
| Verification time (`conversion.verified_at`) | 2026-09-14T04:00:39Z |
| Worktree dirty at goal measurement (`measurement_evidence.dirty`) — untracked gate summary file present at run time (attributed from timestamps; the summary does not record which paths were dirty) | true |

Goal threshold verdicts, against the single predeclared reference (both epoch and previous point at `t13-f03-mutation-target-applicability-goal.json`, matching that closure's epoch re-pin):

| Counter | delta% | level |
| --- | --- | --- |
| mesh_hops | -2.502 | ok |
| vm_steps | 0.769 | ok |
| graph_relax_iters | -0.718 | ok |
| plasticity_updates | -15.586 | ok |
| actions_applied | 0.356 | ok |
| births | -1.711 | ok |
| wall_clock_ms_per_creature_tick | 6.084 | ok |

No epoch was re-pinned; the run used the predeclared epoch and previous file (the same file for both).

Cap timings (source: `environment.*` in the goal summary; caps from `measurement_evidence.wall_caps`):

| Cap | Measured | Budget | Verdict |
| --- | --- | --- | --- |
| Founder observation | 110.860 ms (`neighborhood_founder_wall_clock_ms`) | < 10 s | ok |
| Evolved observation, summed across seeds | 551.185 ms (`neighborhood_evolved_wall_clock_ms_total`; per-seed 164.253 / 212.435 / 174.496 ms) | < 180 s | ok |
| Drift depth, total across 3 worlds | 16,238.091 ms (`drift_depth_wall_clock_ms`) | < 30 s per world (≈90 s budget for 3 worlds) | ok |
| T13.F02 recruitment-paths experiment | 7,321.305 ms (`recruitment_paths_wall_clock_ms`) | < 120 s | ok |
| Goal end-to-end | 551,060.555 ms ≈ 9.18 min (`wall_clock_ms_total`) | < 15 min | ok |

`graph_relax_iters` and `graph_compute` energy per creature-tick: this summary schema reports `graph_relax_iters` only (no separately reported `graph_compute` energy counter). Gate `graph_relax_iters` per creature-tick is 0.993159 (vs epoch -0.208%, vs previous 0.003%); goal is 1.022229 (vs the single reference -0.718%). Both are inside the +10% flag with no floor; the predeclared direction was "up" and both moved slightly down.

Drift changed/all births per world, against floors 0.0015 (depth 1,000) and 0.005 (depth 2,000) (source: `deterministic.goal_indicators.cases[i].drift_depth.readings[]`, field `changed_per_all_births`):

| World | Depth 1,000 | vs floor 0.0015 | Depth 2,000 | vs floor 0.005 |
| --- | --- | --- | --- | --- |
| Orchards in grassland | 0.004500 | above | 0.003500 | **below floor** |
| Canyon country | 0.008500 | above | 0.004500 | **below floor** |
| Confluence | 0.004500 | above | 0.003500 | **below floor** |

T13.F01 Graph backend rungs at depth 1,000 / 2,000 per world (source: same `drift_depth.readings[]`, field `backends.graph`; dispatched-not-contributing computed as `executed - contributing`):

| World | Depth | contributing | executed (dispatched) | dispatched-not-contributing | total |
| --- | --- | --- | --- | --- | --- |
| Orchards in grassland | 1,000 | 0 | 72 | 72 | 1,746 |
| Orchards in grassland | 2,000 | 0 | 96 | 96 | 3,529 |
| Canyon country | 1,000 | 2 | 99 | 97 | 1,820 |
| Canyon country | 2,000 | 0 | 108 | 108 | 3,620 |
| Confluence | 1,000 | 0 | 72 | 72 | 1,746 |
| Confluence | 2,000 | 0 | 96 | 96 | 3,529 |

T13.F02 in-report experiment (`deterministic.goal_indicators.recruitment_paths.arms`, 18 arms, `summary.proposal_discovery`/`summary.retained_discovery`): `proposal_discovery.fraction` and `retained_discovery.fraction` are 0.0 for 14 of 18 arms (all `*_blank`, `*_copy`, `*_split`, `*_unprepared` starts) and non-zero only for the four `*_prepared` starts — `graph_prepared` 0.53125/0.40625 and 0.53125/0.53125, `vm_prepared` 0.5625/0.4375 and 0.5625/0.5625 — the same figures recorded at T13.F03 closure. Reported as consequences per the predeclaration; no floor.

### Facts reported, not interpreted

- Drift changed/all births at depth 2,000 is below the 0.005 floor in all three worlds this run: Orchards in grassland (0.0035), Canyon country (0.0045), Confluence (0.0035). No floor was met at depth 2,000.
- Graph backend `contributing` count stays at 0 in 5 of 6 world/depth cells (nonzero only at Canyon country depth 1,000, contributing=2), despite the repair entering more zero-compute wired visits (`executed` rose relative to `contributing`); the predeclared "contributing share rises" direction did not materialize in this run's Graph rungs.
- T13.F02 in-report discovery fractions are unchanged from the T13.F03 closure figures for every arm.
- These are reported to the orchestrator as facts; no threshold, baseline, or epoch was changed, and no remediation was attempted.

## Mutation gate

Fresh gate on the final feature code, `MUTANTS_ITERATE=0 make rust-mutants`.

| Field | Value |
| --- | --- |
| Summary line | `20 mutants tested in 5m: 1 missed, 16 caught, 3 unviable` |
| Timeouts | 0 (`mutants.out/timeout.txt` empty) |
| Run mode (`run-mode.txt`) | `fresh` |
| Diff base | `6ad57c30eab60bca1bdb0e0505b9e3df8d2e243f` |
| Output path | `~/.local/share/petri-tools/mutants/t13-f04/mutants.out` |
| Fresh runs used | 1 (no production content, test selection or tool configuration changed; no test deleted or weakened) |

### Survivor disposition

| # | Survivor (`mutants.out/missed.txt`) | Disposition | Resolution |
| --- | --- | --- | --- |
| 1 | `crates/v3-core/src/creature/genome/cgp.rs:297:58: replace \|\| with && in CgpGraphBackendDef::enters_visit` | killed | The inner `\|\|` in the action-bank predicate (`!slot.gate_inputs.is_empty() \|\| !slot.param_inputs.is_empty()`). Every prior action-slot fixture wired both lists, so `&&` still entered. `action_slot_enters_a_visit_on_a_gate_edge_or_a_param_edge_alone` in `crates/v3-core/src/runtime/cgp/f04_tests.rs` builds a zero-compute def with an `Emit(Move)` slot wired on one edge list only — once gate-only, once param-only — and asserts `enters_visit()`, `graph_relax_iters == 1`, the one node-equivalent energy charge and the matching `graph_compute` observation. Under `&&` both cases fail to enter: the counter stays 0 and energy is unchanged. |

Equivalent: none. Deferred: none.

No existing test was deleted or weakened; one test was added. No production code,
`.cargo/mutants.toml`, test selection, `#[mutants::skip]` or `exclude_re` entry
was touched.

### Post-remediation checks

| Command | Result |
| --- | --- |
| `MUTANTS_ITERATE=1 make rust-mutants` (feedback only, not closure evidence) | `1 mutant tested in 79s: 1 caught`; 19 prior caught/unviable excluded |
| `cargo test -p v3-core` | ok. 1429 lib passed (was 1428; +1 f04 test) plus all integration targets; 0 failed; 2 ignored |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p v3-core --all-targets` | clean (no warnings) |
| `make roadmap-check` | validation passed |

`cargo test -p v3-cli` was not rerun: no `v3-cli` file was touched by this gate.
