# Vote-margin read run: results (2026-10)

Date: 2026-10-09. Contract: the user's goal command of 2026-10-09, in the
form of the [vote-margin read plan](vote-margin-read-experiment-plan-2026-10-09.md)
(its gaps table, sign convention, class table, predictions, gates V0.2 and
V0, row V1, stopping rule and review dispositions, taken as written), with
the [output-wiring run](output-wiring-run-2026-10.md)'s gates,
frozen-instrument pattern and Advice table as the precedent, extending the
[diagnosis follow-up run](evolvability-diagnosis-run-2026-10-followup.md)'s
row N3 instrument, inside the
[T22 exploration contract](../archive/program-1/roadmaps/t22-capability-assays-and-evolvability-lab.md).
Status: **complete** (gates V0.2 and V0 passed; V1 completed to 50,000; D
run beside on both Canyon seeds; Codex reviews (a) and (b) disposed; landed
on `main` as docs only, the `instr:` commits kept on the branch).

## Question

N3 found, on Confluence at 50,000 ticks on the engine of `d4bc4a98`, that
34 of 35 executed target-family reads alter no vote on the observation
panels, that the few vote-moving reads raise a sink of a kind other than the
committing one without any commit changing, and that the two causal reads
change actions with unchanged votes. It left three observation questions
open: (1) exposure against downstream silence, (2) incumbent masking at the
kind level, (3) the action-parameter route; plus the pooled per-pass
statistic its row predicted and never computed. This run answers them **on
current production** (`main` after T03.F12, T20.F16 and T11.F29) with
instrument work only. N3's readings are the prior, not an identity target.

## Run setup

- **Roles (the plan's Roles section; no Fable).** Session on Opus 5.5
  (`claude-opus-5-5`) at effort `xhigh`, confirmed from the session's own
  metadata; it is the implementer. Advisor consulted through the `advisor`
  tool at every point the Roles section lists (the Advice table). **Setup
  deviation, recorded:** `/advisor opus` was not run before the goal was
  pasted (no such command in the session transcript), both the user and the
  project settings still read `advisorModel: fable`, and the advisor cannot
  state its own model from inside the tool; the tool answered without a
  credit error, so the Roles fallback (Codex as advice job) does not
  trigger. The user was told once in the session. No subagent is spawned.
  Reviews: Codex `gpt-6.1-sol` `high`, read-only, through `codex exec`:
  (a) this predeclaration, disposed before the hashed v2 binary is built and
  before V0 runs; (b) the note's reading before closing.
- **Launch revision.** `main` `785dc18e`: `3a8b0572` plus one `docs:` commit
  made by this session as its clean-main step (the plan's Roles sections and
  updated goal commands, uncommitted in the main checkout when the goal was
  pasted; the pasted goal is that commit's text). Worktree
  `.claude/worktrees/vote-margin` on branch `worktree-vote-margin`, created
  with `EnterWorktree` (`baseRef: head`), head checked equal to `785dc18e`.
  `npm ci` in `frontend/`. Disk free at launch 51 GiB (stop line 20 GiB).
- **Host.** On AC (100 %) at launch. No other Petri heavy job
  (`pgrep -fl 'v3-lab|v3-cli|census-|cargo|rustc'` empty); a long-running
  virtual machine (the telemetry stack) and the desktop app hold a load
  average near 4, which is not a Petri job and is not waited on. Every
  heavy job runs under `caffeinate -i` with the output-wiring run's
  awake-time killer (`diagrun.sh`, copied), and the runner records
  `pmset -g batt` and the `pgrep` line at every heavy launch. One heavy job
  at a time. **"Host free" for D** is operational: no other Petri heavy job
  by that `pgrep` at D's launch.
- **Evidence.** Raw output under `.bench-artifacts/lab/votemargin/` in the
  worktree, copied to the main checkout at closing; compact summaries and
  readers beside this note under
  [`vote-margin-read-run-2026-10/`](vote-margin-read-run-2026-10/), each under
  the 1 MiB ceiling.
- **Branch and commits.** `worktree-vote-margin` from `785dc18e`:
  `instr:` `09f91f13` (v1 file state), `813c1ea0` (v2), `ca4796c8` (the
  header line); `docs:` `4d5975b7` (predeclaration), `35808cb2` (A1),
  `d92c75ab` (gates), `a21d6547` (V1 and the plan forward), `c338614d`
  (review (b) dispositions) and the closing commit, each touching only
  paths under `docs/`, cherry-picked to `main` in that order.
- **Commit kinds.** `docs:` (this note and its folder) and `instr:`
  (observation only). No `proto:` and no `lab:` code beyond the taken probe.
  Only `docs:` lands on `main`; the branch is kept.
- **Code taken from `worktree-output-wiring`** (file state, no
  cherry-pick): the census probe `crates/v3-cli/src/bench/diagnosis_census.rs`
  and the vote-delta reading `input_use/vote_delta.rs` as of `45923a4c`
  (instr `520b4b4b` plus amendment A1), the sink-use funnel
  `input_use/sink_use.rs` with `sink_use/fixtures.rs` (same state), the
  diagnosis probe `crates/v3-lab/tests/diagnosis_probe.rs` from `3ead37f6`,
  and the two module lines, committed as `instr:` `09f91f13`. No `proto:`
  commit is taken (`2065366e`, `7f1c8da8` stay on that branch); the census
  reads `born_connected_destinations` through the serialized config and
  builds without it. `cargo check --tests --release` on v3-core, v3-cli and
  v3-lab exits 0 on `main` `785dc18e` plus `09f91f13`.
- **The v1 census (gate V0's reference).** Built from `09f91f13` in a
  detached nested worktree (`.bench-artifacts/lab/votemargin/frozen-tree`,
  removed before `ExitWorktree`), copied to `census-v1` and hashed
  (`149ad949…`, `census-v1.sha256`); it lists `diagnosis_census_run`.
- **The v2 census.** `instr:` `813c1ea0` (the v2 additions with the A1
  instrument fixes, six class fixtures, the tie-rule and exposure-context
  tests; the module's 10 tests pass), built from the branch head after the
  Codex (a) dispositions are committed, copied to `census-v2` and hashed.
  The fixtures were written after the instrument code in the same commit,
  not before it (a TDD-order deviation, recorded); each asserts the class
  and the readings that define it. Every gate
  and row runs a copied, hashed binary (`./census-vN diagnosis_census_run
  --ignored --nocapture` with the `PETRI_E2_*` variables), never `cargo
  test` in the live worktree, so no source edit can rebuild a binary
  mid-chain.

### The instrument: `vote-delta-v2` (`instr:`)

The v1 reading (`DeltaStats` per channel and per family, the additive
reconstruction, candidate discovery from `scenes.observe`, the ablation as
the funnel's `ablated`) stays exactly as it is; every v1 field keeps its
value and its key. `VERSION` becomes `vote-delta-v2`. Additions, one per gap
of the plan's table, with the resolutions the plan left open stated here
(each is a predeclared reading for review (a) to check):

1. **Per-consumer exposure (gap 1).** For each candidate channel, every
   **consumer** of it on a reachable node: a Graph compute node with at least
   one input-leaf edge addressing the channel (`(node, Compute(i))`), a Graph
   output sink with such an edge (`(node, Sink(kind))`), or a VM `ReadInput`
   addressing it (`(node, Vm(pc))`); kept per identity, not deduplicated
   across nodes. A consumer **executed** in a scene execution when the
   funnel's read recorder records its read there (a VM step executed past
   its charge, a compute node evaluated, a Graph node's sink edges when its
   effects pass ran: the funnel's own `executed` events). It was **exposed**
   in that execution when it executed and the channel's value in that
   execution's scenario is nonzero (the target families are world compounds,
   read from the scenario's sensors and fixed within one execution). Counts
   per consumer and panel group: `battery` (the 80 `neighborhood-v1`
   executions), `recorded`, `authored`, `sequences` (the T11.F26
   extension's executions: 184 when 32 contexts are recorded; the counts and
   group boundaries derive from the actual panel lengths, including the
   sequence-source fallback, A1 finding 7). A `consumer_exposure` list per
   channel; the channel's exposed consumers and exposure counts derive from
   it. "The original panels" in the class table are these executions.
2. **Local endpoint annotation (gap 1, `consumers.rs`).** Per Graph
   consumer, the set of sink kinds reachable from it **within its node**: a
   sink consumer's own kind; a compute-node consumer's sinks reached by a
   forward walk: from compute node c, every compute node and sink whose
   inputs carry a `ComputeNode(c)` edge, transitively (a visited set, so
   cycles and higher-index references are followed as structural paths,
   some of which span visits; A1 finding 6). Labels: `vote` (`ActionVote`), `param` (`ActionParam`), `bus`
   (`CustomOutput`), `memory` (`WriteSlot`, `ClearSlot`), `gate`
   (`RouterGate`). Weights are ignored (the annotation is structural).
   **Decision** when the set contains `vote` or `param`; **intermediate**
   when it is nonempty without them; **`none`** when it is empty (a dead-end
   leaf), which counts as intermediate for classes 6 to 8 (class 7's "only
   bus, memory or gate sinks" holds of an empty set) and is reported apart;
   a `none` endpoint is not evidence of a cross-node path. VM consumers carry
   `dynamic`. Cross-node, memory and control routes are not followed.
3. **Per-pass effective-kind record (gap 2).** For every **delta pass** (an
   aligned pass with any nonzero raw-sink difference; v1's set), in both
   arms: each kind's within-kind maximum, its effective vote (the trace's
   `effective_votes`), the selected kind (**the production rule**,
   `vote_select::select`, reconstructed with the arm's bars and previous
   committed kind in that execution, so a tie goes to the incumbent; the tie
   flagged, and whether the reconstruction's effective votes equal the
   trace's; A1 finding 1), the committed kind, and guard status (a kind
   positive, nothing committed);
   beside them the scene's panel group, divergence (v1's rule), the moved
   sink (largest raw |Δ|) and its kind, the **read effect** per kind on the
   effective vote (baseline minus ablated, the plan's sign convention) and
   on the moved sink, and the pass's |Δ| / within-kind margin when margined
   (v1's rule). Records for delta passes only.
4. **Masked and suppressing, per pass (classes 9 and 10).** Over **S**, the
   delta passes that commit in both arms: a pass is **raising** when the
   read raises (read effect > 0) the effective vote of some kind other than
   the baseline's committing kind, and **lowering** when it lowers (< 0) the
   effective vote of some kind that commits in neither arm. Class 9
   `masked` when raising passes are a strict majority of S; otherwise class
   10 `suppressing` when lowering passes are; otherwise class 11
   `moving-residual` (also when S is empty, or when no kind's effective vote
   moves). The **shortfall** of a raising pass is the baseline's effective
   vote of the committing kind minus that of the most-raised kind. The moved
   sink is recorded beside, descriptively.
5. **Action-parameter fields (gap 3).** The recording executor keeps the
   action-parameter surface at each pass end: zero at the start of each
   execution, updated after every dispatch, carried across passes without
   reset, so a pass that ends before any dispatch carries the surface
   `decode_commit` would read (no engine change; A1 finding 5). In each
   execution whose committed queue differs, on **every aligned pass** (A1
   finding 2): the fields whose value differs between arms, each with
   `consumed` when a kind committed on that pass (either arm) decodes it.
   `param_fields_differ` is the union of differing fields over those passes
   (prediction 3 counts these), `param_fields_consumed` the union of
   consumed ones (class 2 reads these); the fields differing on the passes
   whose committed action differs, the passes whose commit differs and the
   unmatched passes are reported apart.
6. **Pooled per-pass median (gap 4).** Over every margined delta pass of
   every executed target channel at a checkpoint, the median of |Δ| /
   within-kind margin, computed by the reader from the per-pass records,
   with N3's labels (dilution, cancellation, subadditive, neither, mixed) on
   it and the family readings.
7. **Classes.** Per channel, in Rust, in the plan's order, on the original
   panels: "the committed action queue differs" is v1's
   `scenes_actions_differ` > 0; "nonzero read effect" is v1's `delta_passes`
   > 0, and "zero on every aligned pass" is `delta_passes` = 0. A
   consistency count, expected 0 by construction (the ablation forces the
   value a zero-valued read already had): channels with no exposed consumer
   and a nonzero read effect.
8. **Option C, exposure contexts (`exposure_contexts`, own block per
   parent).** 48 contexts, each a copy of one `neighborhood-v1` snapshot
   with every channel of every target family set nonzero: every food type's
   area-food summary drawn from the authored group's production ranges with
   a zero redrawn, and `NeighborBarrierRing` all eight 1.0; seed
   `27_000_000 + i`. Singles only. C runs its own candidate discovery (the
   executed target channels on these contexts), its own v1 statistics,
   v2 records and classes; and, for every original candidate and each of
   its consumers, `exposed`, `executed-zero` or `coverage-unresolved` (did
   not execute on C). Never merged into the original candidate set, the
   family readings or the predictions.

**Fixtures (gate V0.2),** in `make check` on the branch, each an authored
genome on authored panels: a `param-causal` read (a VM `ReadInput` written
to `EatFoodType` under an `Eat` commit); a silent Graph read with a local
vote endpoint at weight zero (`silent-local-decision`); a `masked` read
(raises a losing kind); a `suppressing` read (lowers a losing kind); a
two-consumer channel with one exposed dead-end consumer and one unexposed
voting consumer (must not class as `silent-local-decision`); an
exposure-context consumer that does not execute (`coverage-unresolved`).

## Gates (predeclared, blocking)

| ID | What | Rule |
| --- | --- | --- |
| V0.2 | Fixtures and smoke (first) | (i) The six fixtures and the module's unit tests pass, and `make check` exits 0 on the branch before any binary is hashed. (ii) **Smoke:** `census-v2` with `PETRI_E2_VOTE_DELTAS=1` on the follow-up's 200 × 200 Canyon recipe (`smoke/small.json`, sha256 `485152f6…`, 400 founders), seed 33, checkpoints 300, 2,500 and 5,000: exit 0; every reached checkpoint carries a `vote_deltas` block whose parents each carry `exposure_contexts`, and every channel carries its v2 record (class, `consumer_exposure`, per-pass records when `delta_passes` > 0, parameter fields when `scenes_actions_differ` > 0); the block's serialized size and wall time recorded. A smoke with no executed target channel passes on the structural checks and says so (the fixtures carry the classification) |
| V0 | Identity between builds (second) | `census-v1` and `census-v2`, one after the other, eight threads (`RAYON_NUM_THREADS=8`), the Confluence goal world (`experiments/worlds/confluence.json`, seed 33), `PETRI_E2_VOTE_DELTAS=1`, horizon 10,000, checkpoint 10,000. **Extension rule:** if the v1 run's 10,000 block has no parent with `executed_target_channels` > 0, both builds rerun to 20,000 with checkpoints 10,000 and 20,000, and the rerun is the gate. **Pass** when: every sample row equal; the extinction record and footer (`ticks`, `extinct`) equal; at every scheduled checkpoint the `input_use` and `e6` blocks, each serialized once with sorted keys and compact separators, hash-equal; and the **whitelist projection** of v2's `vote_deltas` (per parent `parent_index`, `generation`, `genome_size`, `nodes`, `reachable_nodes`, `executed_target_channels`; per channel `family`, `channel`, `consumers_graph`, `consumers_graph_abs_weight`, `consumers_vm`, `stats`; per family every v1 key) hash-equals v1's block under the same serialization. The header must be equal on every nested value except `vote_delta_version` (v1 against v2), any key v2 adds, and the recipe path and label; timing fields excluded and listed. The other checkpoint blocks (`births`, `sink_use`, `memory_loop`, `memory_sensitivity`, `temporal_memory_sensitivity`, the cohort and decile fields) are hashed and reported, not blocking. **A1 (finding 3):** every legacy key must be present at every level of both blocks, v1's raw block must equal its own projection, v2's projection is compared against v1's raw block, and `vote_delta_version` must be exactly `vote-delta-v1` against `vote-delta-v2`. A blocking mismatch stops the plan. **V1's budget (A1 finding 4: the plan's "wall time of the v2 run"):** W = v2's wall time to tick 10,000 including that checkpoint's census (the footer's `elapsed_secs` at horizon 10,000; under the extension rule, the run log's `elapsed` at the tick-10,000 sample plus that checkpoint's `census_secs`); projection = W × 15.26 (N3's elapsed 15,560.9 s to 50,000 over its 1,019.7 s to 10,000); V1's awake kill = min(2 × projection, 28,800 s), computed by `v0check.py` |

**Gate results.** **Binaries.** `census-v1` `149ad949…` built at
`09f91f13` in the nested tree; `census-v2` `61587da3…` built at `ca4796c8`
(the A1 instrument commit `813c1ea0` plus a one-line census header fix for
clippy's `too_many_lines`, 169 against 167, which stopped the first
`make check`); each lists `diagnosis_census_run`. **V0.2 (i) passed:** the
module's 10 tests (the six class fixtures, the incumbent-tie test, the
exposure-context construction, the two v1 unit tests) pass, and **`make
check` exited 0** on the branch at `ca4796c8` (log
`make-check-branch.log`; `npm ci` in `frontend/` first). **V0.2 (ii)
passed** ([`smoke.json`](vote-margin-read-run-2026-10/smoke.json), checker
[`smokecheck.py`](vote-margin-read-run-2026-10/smokecheck.py)): `census-v2`
on the 200 × 200 recipe completed 5,000 ticks in 9.2 s (exit 0, 8 threads,
on AC), every checkpoint (300, 2,500, 5,000) carries a `vote_deltas` block
of 20 parents each with `exposure_contexts`; **no sampled parent carries an
executed target channel at any smoke checkpoint** (the small world's cohort
reads no area summary or barrier ring), so the smoke passes on the
structural checks alone, as predeclared, and the fixtures carry the
classification; the block is 4.8 KB and 0.01 to 0.03 s per checkpoint.
**V0 passed** (17 of 17 checks, [`v0.json`](vote-margin-read-run-2026-10/v0.json),
checker [`v0check.py`](vote-margin-read-run-2026-10/v0check.py)): `census-v1`
then `census-v2` on Confluence seed 33 to 10,000, footers' `elapsed_secs`
701.8 s and 709.0 s, 8 threads, on AC, no other Petri job; the v1 block at 10,000 has 3 executed target
channels on 2 of 20 parents, so **the extension rule did not trigger**.
Every sample row (20 ticks) equal, both footers at 10,000 and not extinct,
the `input_use` and `e6` blocks hash-equal, every legacy key present, v1's
raw `vote_deltas` equal to its own projection and to v2's projection, the
version pair exactly `vote-delta-v1` / `vote-delta-v2`, the header equal
but for that pair and v2's one added key
(`vote_delta_v2_exposure_seed_base`). Reported, not blocking: `births`,
`sink_use`, `memory_loop`, both memory sensitivities, the cohort and decile
fields all hash-equal. The v2 block is 2.4 MB at 10,000 (per-pass records
for 2,240 delta passes, about 1 KB each) against v1's 9.6 KB, and costs
0.07 s. **V1's budget:** W = 709.0 s (the footer's `elapsed_secs`),
projection 709.0 × 15.26 = 10,819 s, **awake kill 21,638 s** (6.0 h, under
the 8 h cap). Today's Confluence carries about twice N3's population (7,197
living at 10,000 against N3's 3,582) at a lower simulation cost per tick
(699 s to 10,000 against N3's 1,019 s): the comparison is across engines
(T24's food repairs, T03.F12's gestation and T11.F29's feedback routes
since N3), and the 15.26 ratio is N3's growth profile, which today's denser
world need not follow; a budget kill is covered by the stopping rule
(`incomplete`, the checkpoints reached kept) and is not a host failure.
**Consistency check on real data:** `smokecheck.py` on V0's v2 output
passes (exit 0; per-pass record counts equal `delta_passes` and parameter
executions equal `scenes_actions_differ` on all six channel readings,
original and C), which the smoke could not exercise.

**V0's v2 run at 10,000 (a by-product, not a row checkpoint; no prediction
is read here).** Three executed target channels on two parents
([`v0-v2read.json`](vote-margin-read-run-2026-10/v0-v2read.json)): one
`vote-causal` (`AreaFoodSummary:1` channel 2, one Graph sink consumer with
a vote endpoint; 1,180 delta passes, one action-changing execution with 10
committed-differing passes and no differing parameter field) and two
`moving-residual` (`AreaFoodSummary:0` channels 1 and 6 on two parents, one
Graph consumer each, every one of 530 delta passes moving only the kindless
`Terminate` sink: a target read reaching a pass-control sink, no kind's
effective vote moving). Two "moved" statistics differ by design: v1's
`most_moved_sink` (largest |Δ| summed over passes) is `Move(6)` on the
`vote-causal` channel, carried by the 9 passes where `Eat` moves most, while
the per-pass moved sink of v2's records is `StealEnergy(3)` on 1,171 of
1,180 passes. The pooled per-pass median is 0.037 over the 951 margined
passes of that one channel (220 of its delta passes have a zero within-kind
margin), so N3's "dilution" label there rests on a single channel that is
nevertheless `vote-causal`; the label is reported with its contributing
channels and passes, never alone. No battery execution exposes any of the
three consumers (N3's coverage finding reproduced on this engine); option C
discovers the same three channels with the same classes (the causal one
changes 7 of 48 executions there) and all three consumers exposed. The
consistency counts are 0 (unexposed with effect) and 0 (effective-vote
reconstruction mismatches on recorded delta passes).

## Ledger

| ID | Question | Method, instrument, sizes | Prediction / decision rule | Status | Result |
| --- | --- | --- | --- | --- | --- |
| V1 | On current production, for each executed target channel: was each consumer exposed when it executed and where does its output land locally; is a vote-moving read masked, suppressing or neither; which parameter field carries a zero-vote action change? | `census-v2`, `PETRI_E2_VOTE_DELTAS=1`, Confluence goal world seed 33, 50,000 ticks, checkpoints 20,000 and 50,000 (`PETRI_E2_E6_TICK=20000`), eight threads, the original panels plus C's exposure contexts in their own block; awake kill per V0's budget. **Readings:** the plan's class table per channel and checkpoint with counts; per consumer exposure and local endpoint; the per-pass effective-kind record summarized per channel (S, raising, lowering, shortfalls); `param_fields_differ` and `param_fields_consumed` per action-changing channel; the pooled per-pass median with N3's labels; the family readings; C's block beside, never merged. **Optional D:** Canyon seeds 1022 and 2022 through the goal recipe file (`PETRI_E2_RECIPE=experiments/worlds/canyon-country.json`, `PETRI_E2_SEED`, as the output-wiring run's `default` arm; the goal world's own seed is 22; corrected after review (b) finding 5, the driver having run it this way), 20,000 ticks, checkpoints 10,000 and 20,000, 3 h awake kill each, run after V1 only if the host is free | **The plan's predictions, verbatim:** "1. At the 50,000 checkpoint, at least half of the channels in classes 4 to 8 are `unexposed` or `silent-local-intermediate`. Fails if fewer than half. 2. Among the vote-moving channels (classes 9 to 11) at each checkpoint, `masked` is a minority. Fails if `masked` is half or more. (N3's positive sink shares show sink suppression; they do not prove a kind was suppressed, so this is a prior, not a reading.) 3. Every action-changing channel is `param-causal` with exactly one differing field. Fails on any `vote-causal` or `action-unresolved` channel, or any `param-causal` channel with two or more fields. If instead most of classes 4 to 8 are `silent-local-decision`, the stall sits downstream of a decision endpoint and the routing changes accordingly." Prediction 1 reads 50,000 only; 2 each checkpoint; 3 every checkpoint reached; a prediction whose channel set is empty at a checkpoint is recorded **vacuous** there, not met. **Disposition:** descriptive; the row succeeds when V0 passed and every executed target channel carries a class at every checkpoint reached. **Stopping rule:** on a kill or an extinction the checkpoints reached are kept, the row is `incomplete` and the note closes on what was reached; no rerun unless the kill was a host failure, then one rerun under the same limit. D's classes are reported beside V1's, never pooled with them | **completed** (50,000 reached; V0 passed; every executed target channel classed at both checkpoints) | Census complete: 50,000 ticks, 8,048 s of simulation, 11.0 s of censuses, 8 threads, exit 0, 8,087 s elapsed (8,093 s awake) inside the 21,638 s kill, on AC; summary [`v1-summary.json`](vote-margin-read-run-2026-10/v1-summary.json), per-channel rows [`v1-channels.json`](vote-margin-read-run-2026-10/v1-channels.json), reader [`v1read.py`](vote-margin-read-run-2026-10/v1read.py). Population 6,606 at 20,000 and 12,013 at 50,000. **At 20,000** (7 of 20 parents, 13 channels): `vote-causal` 10 (`AreaFoodSummary:1` 8, `NeighborBarrierRing` 2), `vm-unresolved` 1 (a barrier-ring direction read by 5 VM instructions), `silent-local-intermediate` 1, `moving-residual` 1 (`AreaFoodSummary:0` channel 5 moving a `Reproduce` sink and no kind's effective vote); 19 consumers (14 Graph, 5 VM), 15 executed, 15 exposed (endpoints: decision 10, intermediate 4, dynamic 1); pooled per-pass median 0.663 over 3,696 margined passes of 11 channels, N3's label **neither**. **At 50,000** (20 of 20 parents, 89 channels): `vote-causal` **38** (`AreaFoodSummary:0` 12, `:1` 12, `NeighborBarrierRing` 14), `moving-residual` **33** (barrier ring 23), `silent-local-intermediate` **9** (two with `none` endpoints only), `masked` **7** (all barrier-ring reads: six of direction 5 raising `Reproduce`, one of direction 2 raising `StealEnergy`; shortfall medians 0.69 to 18.0), `suppressing` 1, `unexposed` 1 (a barrier-ring direction-4 read whose one Graph consumer never executed with the channel nonzero), and **0** `param-causal`, `action-unresolved`, `vm-unresolved`, `silent-local-decision` and `silent-mixed`; 108 consumers (105 Graph, 3 VM), 106 executed, 105 exposed (decision 86, intermediate 12, none 4, dynamic 3); pooled per-pass median 0.944 over 11,491 margined passes of 77 channels, label **neither** (one of 47 families cancelling). Of the 33 `moving-residual` channels, 19 move a raw sink on every delta pass without any kind's effective vote moving. `EatFoodType` also differs on 7 of the 38 `vote-causal` channels at 50,000 (consumed on 6) and on 4 of 10 at 20,000 (`ReproduceTransferFraction` on one more): a parameter difference alongside a vote difference, whose share in the action change the census does not separate. **Exposure:** no consumer is exposed on the `neighborhood-v1` battery at either checkpoint (N3's coverage finding holds on this engine); exposure comes from the recorded contexts, their sequences and the authored contexts. **Option C** (own block): its own discovery finds the same 13 and 89 channels; classes on C at 50,000 `vote-causal` 43, `silent-local-intermediate` 9, `masked` 4, `moving-residual` 33; original consumers `coverage-unresolved` on C 4 of 19 at 20,000 and 2 of 108 at 50,000, `executed-zero` 0. **Consistency:** 0 channels unexposed with an effect and 0 effective-vote reconstruction mismatches on recorded delta passes, at both checkpoints (the trace records no selected kind; the reconstruction's effective votes are what is compared). **Lineage proxy** (the counts are channels, not lineages): at 50,000 the 38 `vote-causal` channels carry 38 distinct (family, channel, delta passes, action-changing executions, consumers) signatures, the 7 `masked` 3, the 33 `moving-residual` 32, the 9 `silent-local-intermediate` 8; at 20,000 the 10 `vote-causal` carry 7 (parents 37, 44 and 47 share `AreaFoodSummary:1` channel 3 and 5 readings). **Funnel rows and agreement:** the census's own `input_use` typed rows on the selected cohort (declared / connected / executed / causal distinct parents) are `NeighborBarrierRing` 3/3/3/2, `AreaFoodSummary:0` 1/1/1/0, `:1` 5/5/5/5 at 20,000 and 20/20/20/**11**, 15/15/15/**11**, 12/12/12/**9** at 50,000 (N3 at 50,000: 11/9/9/0, 8/7/7/0, 4/4/4/0), `causal_original` 0 throughout; the funnel's `causal` equals the distinct parents with an action-changing vote-delta channel of that family in 6 of 6 comparisons. **Consumer sites:** every exposed consumer of a `vote-causal` channel is a Graph sink edge, none a compute node (50,000: 44 on `ActionVote` sinks and 1 on a `RouterGate`; 20,000: 9 `ActionVote`, 3 `CustomOutput`). The vote-sink leaves (44 and 9) have the shape of T20.F16's born vote edge, which N3's engine did not have, and every typed funnel row now reads declared = connected = executed (N3: 11/9/9, 8/7/7), the "connected tracks declared" signature the output-wiring run recorded for the born edge; that the born edge carries these reads is **a hypothesis**, the census recording no edge provenance. **Predictions:** (1) **met** at 50,000 on 10 channels: the 10 channels in classes 4 to 8 are all `unexposed` (1) or `silent-local-intermediate` (9); `silent-local-decision` 0; the prediction's premise that most executed channels are silent is gone (10 of 89). (2) **met** at both checkpoints: `masked` 0 of 1 at 20,000 and 7 of 41 at 50,000. (3) **failed** at both checkpoints: every action-changing channel is `vote-causal` (10 of 10 and 38 of 38), none `param-causal`. **Against N3's prior** (engine of `d4bc4a98`, 34 of 35 channels with zero delta at 50,000): on today's production 79 of 89 executed target channels move a vote and 38 change a committed action at 50,000; the cohort, engine and population differ, so this is a new baseline, not a refutation of N3 on its own engine. **D (Canyon, beside V1, never pooled):** seed 1022 complete (608 s), 4 channels at 10,000 (3 `vote-causal`, 1 `moving-residual`) and 24 at 20,000 (13 `vote-causal`, 11 `moving-residual`); seed 2022 complete (851 s), 4 at 10,000 (1 `vote-causal`, 3 `moving-residual`) and 3 at 20,000 (2 `vote-causal`, 1 `masked`; 9 VM consumers `coverage-unresolved` on C); consistency 0 and 0 on both; read in the predictions' form (beside V1, never pooled): prediction 1 not reached (no 50,000 checkpoint), prediction 2 **failed** on seed 2022 at 20,000 (`masked` 1 of 1) and met at the other three checkpoints, prediction 3 **failed** at all four (every action-changing channel `vote-causal`, 19 of 19) ([`d-canyon-s1022-summary.json`](vote-margin-read-run-2026-10/d-canyon-s1022-summary.json), [`d-canyon-s2022-summary.json`](vote-margin-read-run-2026-10/d-canyon-s2022-summary.json), with their channel files) |

## Advice

Every advisor consultation the Roles section lists, with the disposition of
each point raised.

| # | Point | Advice (summary) | Dispositions |
| --- | --- | --- | --- |
| 5 | Before closing, after the review (b) dispositions were committed | Nothing blocks closing. One closing commit (this row, Status complete, the branch commits, the turns row); the close order with the nested tree removed first, the evidence copied, `ExitWorktree` keep, `main` checked clean (advanced but clean: pick anyway), purity checked per commit, the picks, `make check-docs` on `main`, the worktree removed and pruned; no second Codex round; the plan's status line untouched (outside this run's docs scope); the report answer first, with the advisor-model deviation | Followed |
| 4 | After the plan forward was drafted and committed, while Codex review (b) ran (the Roles section's after-draft point) | Right in substance; four text fixes. (1) Section 1 contradicts the routing: the plan's `masked` bullet has no "mostly", so T19 receives the hypothesis at minority scale; the "mostly" branches and `param-causal` do not fire. (2) Close the plan's fourth criterion: what stays unresolved for T22. (3) Prediction 2's prior: N3's sink-share suppression did not become kind suppression. (4) A second fingerprint: every funnel row declared = connected = executed, the born edge's recorded signature, still a hypothesis. Process: fold into the review (b) dispositions commit; remove the nested tree before copying the evidence and before `ExitWorktree`; verify each picked commit touches only `docs/`; leave the plan's status line; write the memory before the report | All adopted: (1) to (4) in the plan forward and the V1 cell, in the dispositions commit; the close order followed; the memory written |
| 3 | After the V1 reading and the plan forward were drafted, before they were recorded; also the contradiction consultation (prediction 3 failed) and the before-plan-forward point, batched into one call | Verdict and "no feature" right; the plan's outcome branches all fail to fire, say so. (1) Name the N3 → now shift's candidate cause as a hypothesis: check whether causal reads' consumers are sink leaves (T20.F16's born edge shape). (2) Lineage: report distinct signatures beside counts; read the census's funnel typed rows and N3's agreement rule. (3) Tighten: "exposed on the panels", world exposure stays T20's; "failed: every action-changing channel is vote-causal"; "no longer blocks on this cohort; the Variation verdict untouched"; prediction 1 "on 10 channels". Process: one Python pass, commit, Codex (b) in the same turn, Cost while it runs; if `main` advanced but is clean, cherry-pick anyway and record the base | All adopted: (1) every exposed consumer of a causal read is a sink leaf (44 `ActionVote`, 1 `RouterGate`), recorded as a hypothesis; (2) signatures and funnel rows in the V1 cell (agreement 6 of 6); (3) the plan forward; the close contingency below |
| 2 | After the V0.2 and V0 readings were drafted, before they were recorded (one batched consultation, as Advice 1 suggested); whether to report V0's v2 reading at 10,000; what to watch in V1 | Gate paragraph right. (1) Quote both V0 run times from the footers; say the population and cost comparison is across engines and that 15.26 is N3's profile, so a budget kill reads as predeclared; run `smokecheck.py` on V0's v2 output, the only check of record counts against `delta_passes`. (2) Report the 10,000 reading as a V0 by-product with no prediction read; the "dilution" label rests on one channel that is `vote-causal`, so carry contributing channels beside every label (`channels_with_margined_passes` in the reader); define v1's `most_moved_sink` against v2's per-pass moved sink after a spot check; include C's agreement, the battery's zero exposure and the consistency counts. (3) V1: do not poll; pre-write the Codex (b) brief, the row cell, the cost table and the plan-forward skeleton; at 20,000 compare elapsed with N3's 2,320 s; a `vote-causal` channel failing prediction 3 is a result; name `Terminate`-only movers when they recur; split the reader's rows into a channels file before 1 MiB; update the Status line | All adopted: (1) and (2) in Gate results above and `v1read.py`; (3) the Status line, the waits and the summaries split |
| 1 | Before the first gate: the approach to the whole run, after the contract, the taken code and the executor were read | Approach sound; ten points. (0) The advisor cannot state its model; record the settings and the missing `/advisor opus` as a setup deviation, tell the user once, continue (no credit error, so no Codex fallback). (1) Dispose Codex (a) before the hashed v2 binary and V0, not after V0. (2) Masked and suppressing on per-kind effective-vote read effects (any kind), not through the moved sink's kind; moved sink beside. (3) Walk the genome for consumer identity, sinks by kind; predeclare `none` endpoints, `ClearSlot` as intermediate, an order-safe compute walk. (4) V0's header may differ in `vote_delta_version` and v2's new keys only; hash the other blocks and report. (5) Run copied, hashed binaries, never `cargo test` mid-chain; fresh scripts; `pgrep` and power at every launch; define "host free" for D now. (6) Load `rust-skills`; fixtures first; create the summaries folder before the first `make check-docs`. (7) Predeclare V1's kill formula and automate the extension rule. (8) Per-pass records for delta passes only; read the block size at the smoke and at V0. (9) Turn economy: one note write, one driver, batched consultations. (10) Predeclare the close checklist | All adopted in this predeclaration: (0) Run setup; (1) the Order; (2) instrument item 4; (3) items 1 and 2; (4) gate V0; (5) Run setup and the driver; (6) done before code; (7) gate V0; (8) item 3 and gate V0.2; (9) the Order; (10) the Order |

## Codex reviews

### Review (b): the whole note before landing

Job `.bench-artifacts/lab/votemargin/codex/review-b.out.md` (brief
`review-b.md`), verdict `not-ready`, 3 blocking and 3 advisory. Every
finding is adopted below; no finding changes a gate result, a class count,
a prediction's outcome or the row's status, and no second round is run.

| # | Finding (short) | Severity | Disposition |
| --- | --- | --- | --- |
| 1 | "No branch fires" adds a prevalence threshold; the 7 `masked` channels still route to T19 | blocking | Adopted: the `masked` branch fires at minority scale; T19 receives the hypothesis with shortfalls; no feature (with Advice 4 (1)) |
| 2 | `vote-causal` does not establish mediation through the vote path; the hypothesis must be restricted to vote-sink leaves | blocking | Adopted: the operational class reading in the cell and plan forward; the born-edge hypothesis on the 44 and 9 `ActionVote` leaves only, the `RouterGate` and `CustomOutput` consumers named |
| 3 | The V0 by-product's pooled median rests on 951 margined passes, not 1,171 | blocking | Adopted: 951 (220 zero-margin passes); the unsubstantiated per-pass values removed; 0.037 and the label unchanged |
| 4 | "Reconstructed selection disagrees" overstates: the effective-vote vectors are compared | advisory | Adopted: renamed in both places |
| 5 | D's launch method misstated; D's prediction-form outcomes unlabelled | advisory | Adopted: the recipe file with `PETRI_E2_SEED` stated; prediction 2 failed on seed 2022 at 20,000, prediction 3 failed at all four, prediction 1 not reached |
| 6 | The process account contradicts itself (Advice 3 after the draft; "fixtures first") | advisory | Adopted: the plan-forward header and the Order state the chronology and the deviations |

### Review (a): the predeclaration, before the v2 binary is hashed

Job `.bench-artifacts/lab/votemargin/codex/review-a.out.md` (brief
`review-a.md`), verdict `not-ready`, 4 blocking and 3 advisory. Every
finding is adopted as **amendment A1**; no finding changes a class rule, a
prediction or a gate's question, every fix is an instrument rule tested on
the branch or a checker rule, and no confirmation round is run. Codex
confirmed the exposure reading (wrapped channel, the typed-food resolver's
zero fallback), the read events as an execution record, item 4 against
classes 9 to 11 and prediction 2, and option C's simultaneous settings
under its own block.

| # | Finding (short) | Severity | Disposition |
| --- | --- | --- | --- |
| 1 | The selected kind ignored production's incumbent tie rule | blocking | Adopted: `vote_select::select` reconstructed per arm with bars and the previous kind; agreement flagged; an incumbent-tie test |
| 2 | Restricting parameter fields to changed-commit passes could hide a second field and narrow prediction 3 | blocking | Adopted: every aligned pass of an action-changing execution, per-pass consumption, changed-commit fields and unmatched passes apart |
| 3 | The projection's `.get()` lets a deleted nullable key pass; the version pair was unchecked | blocking | Adopted: legacy keys required, v1 raw equal to its projection, v2's projection against v1 raw, the exact version pair |
| 4 | Simulation time changes the plan's "wall time" budget | blocking | Adopted: v2's wall time to 10,000 with that checkpoint's census, N3's elapsed ratio 15.26 |
| 5 | Dispatch-free pass ends: the surface's initialization and carry | advisory | Adopted: zero per execution, updated after each dispatch, carried across passes (as built) |
| 6 | State the forward walk and the empty-endpoint reading | advisory | Adopted: item 2 |
| 7 | 264 executions only when 32 contexts are recorded | advisory | Adopted: counts from the actual panel lengths (as built) |

## Order

This predeclaration committed → Codex review (a) launched in the background
→ v2 written on the branch (the fixtures in the same commit, after the code:
the TDD-order deviation recorded under Run setup) → the unit tests and
`make check` (V0.2 (i)) → review (a) disposed, amendments committed → the v2
census built, copied and hashed → one driver: V0.2 (ii) smoke → V0 (v1 then
v2 to 10,000, the extension rule automated, the identity checker) → V1 under
V0's budget → D if the host is free; stop on the first failing gate → the
V0.2 and V0 readings with one advisor consultation → V1's reading with its
consultation → the plan forward (consultations before and after the draft)
→ Codex review (b) → close.

**Close checklist.** `git status` empty in the worktree; the nested
`frozen-tree` removed (`git worktree remove --force`, `git worktree prune`)
before `ExitWorktree` with `keep`; `.bench-artifacts/lab/votemargin/`
copied to the main checkout; `main` still at `785dc18e` and clean (stop
and report otherwise, since other sessions run on this host); the `docs:`
commits cherry-picked in order, each verified to touch only paths under
`docs/` (if `main` has advanced but is clean, the commits are picked onto
it anyway, since every one touches only this note's paths, and the new
base is recorded; a dirty `main` or a conflict stops the close); `make
check-docs` on `main` with its exit recorded; the worktree
removed and pruned, the branch listed; `git status` clean.

## Plan forward

Drafted after V1 was read. The Roles section's before-draft consultation
was not held separately: it was batched into Advice 3, held after the draft
and before it was recorded (a deviation, recorded); Advice 4 is the
after-draft consultation; Codex review (b) before closing.

### 1. Verdict

The lead the plan was written for, "reads execute but never decide", does
not describe current production on these panels. On Confluence at 50,000,
38 of 89 executed target channels on the selected cohort (38 distinct
signatures, 11, 11 and 9 distinct parents per family in the funnel's own
`causal` row against N3's 0, 0 and 0) are `vote-causal`: their ablation
changes a committed action queue and moves a raw vote sink on some aligned
pass (parameter and dispatch effects are not separated from the vote
change); 41 more move a vote without changing an action, and 10 are
silent. The
three predictions: (1) met, on 10 channels; (2) met; (3) failed: every
action-changing channel is `vote-causal`, none `param-causal`. Every
exposed consumer of a causal read is a leaf on a sink, the shape of
T20.F16's born vote edge (production since after N3) for the vote-sink
leaves among them: a hypothesis for the shift from N3, not a measured
attribution. Of the plan's "what each outcome means" branches, the three
"mostly" branches do not fire (the silent channels are 10 of 89, none
`silent-local-decision`) and the `param-causal` branch does not (none);
the `masked` branch fires at minority scale (7 of 41 vote-moving channels,
in 3 signatures, 6 on one barrier-ring direction raising `Reproduce`), so
T19 receives the incumbent-masking hypothesis with its shortfalls and no
feature; the 9 `silent-local-intermediate` channels route per channel to
T22, then T20. **Unresolved for T22 before any mechanism is proposed** (the
plan's fourth criterion): 9 `silent-local-intermediate` and 1 `unexposed`
at 50,000, 1 `vm-unresolved` at 20,000, 0 `action-unresolved`; every other
class is decided or reported. **N3's question 2 answered:** N3's positive
sink shares, read as suppression, do not translate into kind-level
suppression on this engine (`suppressing` 1 against `masked` 7 and
`moving-residual` 33 at 50,000). The Canyon readings beside point the same way (all 19
action-changing channels over the four Canyon checkpoints `vote-causal`).

### 2. Routing by class, with counts (V1 at 50,000; at 20,000 in brackets)

| Class | Channels | Routed to (the plan's table) | What it asks |
| --- | --- | --- | --- |
| `vote-causal` | 38 [10] | the funnel's existing `causal` | nothing new: these reads decide |
| `param-causal` | 0 [0] | T11.F27 parameter surface | no second causality route seen alone; `EatFoodType` co-differs on 7 [4] vote-causal channels, unseparated |
| `action-unresolved` | 0 [0] | T22 instrument | none |
| `unexposed` | 1 [0] | T22 instrument; world exposure to T20 | one channel: no row |
| `vm-unresolved` | 0 [1] | T22 instrument (VM attribution) | one channel at 20,000: no row |
| `silent-local-decision` | 0 [0] | T22, then T19/T11 | none: no arithmetic question for T19/T11 |
| `silent-local-intermediate` | 9 [1] | T22 (cross-node tracing), then T20 | 9 reads landing on bus, memory or gate sinks (2 dead-end leaves) |
| `silent-mixed` | 0 [0] | reported | none |
| `masked` | 7 [0] | T19 with shortfalls | 6 of 7 are one barrier-ring direction raising `Reproduce` on different parents (shared ancestry is a hypothesis) |
| `suppressing` | 1 [0] | reported | none |
| `moving-residual` | 33 [1] | reported | 19 move a non-maximal raw sink only |

### 3. What follows

- The N3 follow-up questions are answered for current production: 105 of
  108 consumers are exposed on the panels (the 32 recorded production
  snapshots, their sequences and the authored contexts; world exposure is
  unmeasured and stays T20's question); masking is a minority reading (7 of
  41 vote-moving channels); the parameter surface is not a separate route in
  this cohort. The executed → causal stage no longer blocks these families
  on this cohort; the diagnosis run's Variation verdict is untouched by
  this run.
- **T22 instrument candidates** (lab, not production): the per-consumer
  exposure record and local endpoints (the battery still presents zero for
  these families, so the funnel's `causal` stage rests on recorded and
  authored contexts), and a cross-node trace for the 9
  `silent-local-intermediate` reads before T20 is handed any wiring
  question.
- **T19:** receives the 7 `masked` channels as the incumbent-masking
  hypothesis with their per-pass shortfalls (medians 0.69 to 18.0); a
  small, likely related set (3 signatures) that supports no pass-end
  feature on its own.
- **No roadmap feature and no mechanism prototype** follows from this run.

## Cost

| Step | Time |
| --- | --- |
| Builds | `census-v1` release test build in the nested tree; `census-v2` 300 s |
| `make check` | twice on the branch (the first stopped at clippy `too_many_lines`), about 10 min each |
| V0.2 smoke | 9.2 s |
| V0 | `census-v1` 701.8 s, `census-v2` 709.0 s (footers) |
| V1 | 8,086.7 s elapsed, 8,093 s awake, 8,048 s of simulation, 11.0 s of censuses |
| D | Canyon seed 1022 608 s, seed 2022 851 s |
| Codex | review (a) one job; review (b) one job |
| Session | Opus 5.5 `xhigh`, about 140 of the 150 turns; the cost line awaits `/usage` |
