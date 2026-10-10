# Vote-margin read run: results (2026-10)

Date: 2026-10-09. Contract: the user's goal command of 2026-10-09, in the
form of the [vote-margin read plan](vote-margin-read-experiment-plan-2026-10-09.md)
(its gaps table, sign convention, class table, predictions, gates V0.2 and
V0, row V1, stopping rule and review dispositions, taken as written), with
the [output-wiring run](output-wiring-run-2026-10.md)'s gates,
frozen-instrument pattern and Advice table as the precedent, extending the
[diagnosis follow-up run](evolvability-diagnosis-run-2026-10-followup.md)'s
row N3 instrument, inside the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md).
Status: **predeclared** (no gate or row has run).

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
- **The v2 census.** Built from the branch head after the Codex (a)
  dispositions are committed, copied to `census-v2` and hashed. Every gate
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
   executions), `recorded`, `authored`, `sequences` (the T11.F26 extension's
   184). A `consumer_exposure` list per channel; the channel's exposed
   consumers and exposure counts derive from it. "The original panels" in
   the class table are these 264 executions.
2. **Local endpoint annotation (gap 1, `consumers.rs`).** Per Graph
   consumer, the set of sink kinds reachable from it **within its node**: a
   sink consumer's own kind; a compute-node consumer's sinks reached by a
   forward walk over `ComputeNode` edges (a visited set, so any index order
   is safe). Labels: `vote` (`ActionVote`), `param` (`ActionParam`), `bus`
   (`CustomOutput`), `memory` (`WriteSlot`, `ClearSlot`), `gate`
   (`RouterGate`). Weights are ignored (the annotation is structural).
   **Decision** when the set contains `vote` or `param`; **intermediate**
   when it is nonempty without them; **`none`** when it is empty (a dead-end
   leaf), which counts as intermediate for classes 6 to 8 and is reported
   apart. VM consumers carry `dynamic`. Cross-node, memory and control routes
   are not followed.
3. **Per-pass effective-kind record (gap 2).** For every **delta pass** (an
   aligned pass with any nonzero raw-sink difference; v1's set), in both
   arms: each kind's within-kind maximum, its effective vote (the trace's
   `effective_votes`), the selected kind (the largest positive effective
   vote, the first in catalog order on a tie, the tie flagged), the
   committed kind, and guard status (a kind positive, nothing committed);
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
   action-parameter surface at each pass end (the surface after the pass's
   last dispatch, which is what `decode_commit` reads; no engine change). In
   each execution whose committed queue differs, on each aligned pass whose
   committed action differs: the fields whose value differs between arms,
   and `consumed` when the field's kind is the committed sink's kind in
   either arm. `param_fields_differ` is the union of differing fields over
   those passes, `param_fields_consumed` the union of consumed ones.
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
| V0 | Identity between builds (second) | `census-v1` and `census-v2`, one after the other, eight threads (`RAYON_NUM_THREADS=8`), the Confluence goal world (`experiments/worlds/confluence.json`, seed 33), `PETRI_E2_VOTE_DELTAS=1`, horizon 10,000, checkpoint 10,000. **Extension rule:** if the v1 run's 10,000 block has no parent with `executed_target_channels` > 0, both builds rerun to 20,000 with checkpoints 10,000 and 20,000, and the rerun is the gate. **Pass** when: every sample row equal; the extinction record and footer (`ticks`, `extinct`) equal; at every scheduled checkpoint the `input_use` and `e6` blocks, each serialized once with sorted keys and compact separators, hash-equal; and the **whitelist projection** of v2's `vote_deltas` (per parent `parent_index`, `generation`, `genome_size`, `nodes`, `reachable_nodes`, `executed_target_channels`; per channel `family`, `channel`, `consumers_graph`, `consumers_graph_abs_weight`, `consumers_vm`, `stats`; per family every v1 key) hash-equals v1's block under the same serialization. The header must be equal on every nested value except `vote_delta_version` (v1 against v2), any key v2 adds, and the recipe path and label; timing fields excluded and listed. The other checkpoint blocks (`births`, `sink_use`, `memory_loop`, `memory_sensitivity`, `temporal_memory_sensitivity`, the cohort and decile fields) are hashed and reported, not blocking. A blocking mismatch stops the plan. **V1's budget:** projection = v2's `sim_secs` at tick 10,000 × 15.25 (N3's 15,547.6 s to 50,000 over its 1,019.4 s to 10,000; simulation time only); V1's awake kill = min(2 × projection, 28,800 s) |

## Ledger

| ID | Question | Method, instrument, sizes | Prediction / decision rule | Status | Result |
| --- | --- | --- | --- | --- | --- |
| V1 | On current production, for each executed target channel: was each consumer exposed when it executed and where does its output land locally; is a vote-moving read masked, suppressing or neither; which parameter field carries a zero-vote action change? | `census-v2`, `PETRI_E2_VOTE_DELTAS=1`, Confluence goal world seed 33, 50,000 ticks, checkpoints 20,000 and 50,000 (`PETRI_E2_E6_TICK=20000`), eight threads, the original panels plus C's exposure contexts in their own block; awake kill per V0's budget. **Readings:** the plan's class table per channel and checkpoint with counts; per consumer exposure and local endpoint; the per-pass effective-kind record summarized per channel (S, raising, lowering, shortfalls); `param_fields_differ` and `param_fields_consumed` per action-changing channel; the pooled per-pass median with N3's labels; the family readings; C's block beside, never merged. **Optional D:** Canyon goal world seeds 1022 and 2022 (`PETRI_E2_WORLD=canyon`, `PETRI_E2_RECIPE` unset; the goal recipe as the census resolves it), 20,000 ticks, checkpoints 10,000 and 20,000, 3 h awake kill each, run after V1 only if the host is free | **The plan's predictions, verbatim:** "1. At the 50,000 checkpoint, at least half of the channels in classes 4 to 8 are `unexposed` or `silent-local-intermediate`. Fails if fewer than half. 2. Among the vote-moving channels (classes 9 to 11) at each checkpoint, `masked` is a minority. Fails if `masked` is half or more. (N3's positive sink shares show sink suppression; they do not prove a kind was suppressed, so this is a prior, not a reading.) 3. Every action-changing channel is `param-causal` with exactly one differing field. Fails on any `vote-causal` or `action-unresolved` channel, or any `param-causal` channel with two or more fields. If instead most of classes 4 to 8 are `silent-local-decision`, the stall sits downstream of a decision endpoint and the routing changes accordingly." Prediction 1 reads 50,000 only; 2 each checkpoint; 3 every checkpoint reached; a prediction whose channel set is empty at a checkpoint is recorded **vacuous** there, not met. **Disposition:** descriptive; the row succeeds when V0 passed and every executed target channel carries a class at every checkpoint reached. **Stopping rule:** on a kill or an extinction the checkpoints reached are kept, the row is `incomplete` and the note closes on what was reached; no rerun unless the kill was a host failure, then one rerun under the same limit. D's classes are reported beside V1's, never pooled with them | pending | |

## Advice

Every advisor consultation the Roles section lists, with the disposition of
each point raised.

| # | Point | Advice (summary) | Dispositions |
| --- | --- | --- | --- |
| 1 | Before the first gate: the approach to the whole run, after the contract, the taken code and the executor were read | Approach sound; ten points. (0) The advisor cannot state its model; record the settings and the missing `/advisor opus` as a setup deviation, tell the user once, continue (no credit error, so no Codex fallback). (1) Dispose Codex (a) before the hashed v2 binary and V0, not after V0. (2) Masked and suppressing on per-kind effective-vote read effects (any kind), not through the moved sink's kind; moved sink beside. (3) Walk the genome for consumer identity, sinks by kind; predeclare `none` endpoints, `ClearSlot` as intermediate, an order-safe compute walk. (4) V0's header may differ in `vote_delta_version` and v2's new keys only; hash the other blocks and report. (5) Run copied, hashed binaries, never `cargo test` mid-chain; fresh scripts; `pgrep` and power at every launch; define "host free" for D now. (6) Load `rust-skills`; fixtures first; create the summaries folder before the first `make check-docs`. (7) Predeclare V1's kill formula and automate the extension rule. (8) Per-pass records for delta passes only; read the block size at the smoke and at V0. (9) Turn economy: one note write, one driver, batched consultations. (10) Predeclare the close checklist | All adopted in this predeclaration: (0) Run setup; (1) the Order; (2) instrument item 4; (3) items 1 and 2; (4) gate V0; (5) Run setup and the driver; (6) done before code; (7) gate V0; (8) item 3 and gate V0.2; (9) the Order; (10) the Order |

## Codex reviews

### Review (a): the predeclaration, before the v2 binary is hashed

Pending.

## Order

This predeclaration committed → Codex review (a) launched in the background
→ v2 written under TDD (fixtures first) on the branch → the unit tests and
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
`docs/`; `make check-docs` on `main` with its exit recorded; the worktree
removed and pruned, the branch listed; `git status` clean.

## Plan forward

Pending: drafted after V1 is read, with consultations before and after.

## Cost

Pending.
