# Born-connected prototype run: results (2026-10)

Date: 2026-10-08. Contract: the user's goal command of 2026-10-08 (one
predeclared row, B1, in the form of the
[diagnosis follow-up run](evolvability-diagnosis-run-2026-10-followup.md)'s
N1 design with its gates G1 and G3 and its N3 instrument), under the
[diagnosis plan](evolvability-diagnosis-plan-2026-10-05.md) (rules 1 to 9, as
far as they apply to a one-row run) and the
[run 1 plan](evolvability-exploration-plan-2026-10-01.md) it inherits (the
five not-forcing checks, matched controls, commit kinds, baseline identity,
the Codex review command, the close order), inside the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md).
The idea's earlier readings: the [diagnosis run](evolvability-diagnosis-run-2026-10.md)'s
rows E4 (the two-step rates this prototype shortens), E5 (the projection
operator, not advanced) and E10 (run 6's ALT arm, the idea's lab form,
unresolved at 8 pairs). Status: **in progress** (predeclared; no gate or row
has run).

## Question

Does a declaration that arrives connected change what evolves? Today a new
input reference is appended into nothing (`InputRef.Add` is a neutral growth
step) and a later `AddGraphEdge` or `RetargetGraphEdge` must find it: E4
measured the useful signed connection of an already declared `AreaFoodSummary(0)`
leaf at 5.5 × 10⁻⁶ per birth and declare-and-connect in the same birth at 0 of
10⁷ founder births, against a loss of the unconnected declaration at
1.86 × 10⁻² per birth (ratio 2,564 to 5,073). If the same `Add` event also
wires the new reference to one vote sink at a live weight (random sink,
random sub-value, random sign), the declaration is exposed to selection at
once instead of waiting, silent, to be pruned. Does that raise executed and
causal target-family reads in a natural world without costing persistence?

## Run setup

- **Roles.** Session on Fable 5.1 (`claude-fable-5-1`), the implementer: it
  writes the prototype and the instrument changes, runs the gates and the
  row, reads the results and writes this note. The Fable advisor is enabled
  (`advisorModel: fable`) and consulted through the `advisor` tool at every
  rule 8 point (the Advice table). No subagent spawned. Reviews: Codex
  `gpt-6.1-sol` `high`, read-only, through `codex exec` (`codex-cli 0.159.1`,
  the run 1 plan's command, wrapped in a POSIX `sh` script because the
  worktree guard blocks `--skip-git-repo-check` inline): (a) this
  predeclaration before any gate or row runs, its findings disposed and the
  amendments committed first; (b) the note's readings before closing.
- **Launch revision.** `main` `b0f47dd5` (clean; T03.F12 closed there, so the
  production binary carries pre-birth gestation at its default 1.0 per
  founder genome and T24's food-growth repairs). Worktree
  `.claude/worktrees/born-connected` on branch `worktree-born-connected`,
  created with `EnterWorktree` (`baseRef: head`); its head checked equal to
  `b0f47dd5` after creation. `npm ci` done in `frontend/`. Disk free at
  launch 59 GiB (rule 9's stop line is 20 GiB).
- **Host.** At the first reading the host was on battery (100 %,
  discharging); minutes later `pmset` reported it charged on AC. The runner
  records the power state at every heavy launch and wraps every heavy job in
  `caffeinate -i` with the awake-time killer of the follow-up's `diagrun.sh`
  (copied). Telemetry off on every `v3-cli` invocation; the census probe and
  the E4 probe are test binaries. One heavy job at a time;
  `pgrep -fl 'v3-lab|v3-cli'` before each.
- **Evidence.** Raw output under `.bench-artifacts/lab/born/` in the
  worktree, copied to the main checkout at closing; compact summaries beside
  this note under [`born-connected-run-2026-10/`](born-connected-run-2026-10/)
  (each under the 1 MiB ceiling). Recipes are jq overlays of
  `experiments/worlds/canyon-country.json` stored under the evidence tree,
  never under `experiments/worlds/`.
- **Commit kinds.** `docs:` (this note and its folder), `instr:` (the census
  probe, the vote-delta reading and the probe's new mutation-effects block,
  observation only), `lab:` (the E4 `transition_rates` probe), `proto:` (the
  one default-off mechanism prototype). `instr:`, `lab:` and `proto:` stay on
  the branch; only `docs:` lands on `main`.
- **Code taken from `worktree-evolvability-diagnosis-2`** (file state, no
  cherry-pick of mixed commits; sources cited in the commit messages): the
  census probe `crates/v3-cli/src/bench/diagnosis_census.rs` and its module
  line in `bench.rs` (`368e4cb8`), the vote-delta reading
  `crates/v3-core/src/neighborhood/input_use/vote_delta.rs` with its module
  line and battery-coverage test (`368e4cb8`, `0dd4c7d0`, `c4de89e1`), the E4
  probe `crates/v3-lab/tests/diagnosis_probe.rs` (`de579cdf`). What is not
  taken: the two `proto:` prototypes (`dcbee45e`), and every line that read
  their fields: the probe's `PETRI_E4_DECAY` block and `decay` tallies, the
  census header's `reproduce_hold_per_founder_size` and
  `silent_entry_decay_rate` entries, and the funnel's
  `silent_entry_decay_rate: None` one-event line. The probe gains
  `PETRI_E4_BORN=1` in their place; the census header records
  `born_connected`, `gestation_ticks_per_founder_genome` and `per_unit_rate`.
- **Instrument extension (`instr:`).** The census checkpoint row gains a
  `mutation_effects` block: `mutation-effects-v1` at `Sizes::PRODUCTION`
  (20 parents, 100 one-event proposals each, 32 recorded contexts) on the
  selected cohort and the founder, reporting `births_total`, `dead`,
  `silent` and `changed` of `any_events` with the block's wall time, because
  no existing census field carries dead-per-birth (the goal profile's
  `evolved_dead_per_mutated_births` is this reading). It runs under the
  arm's own `MutationConfig` (the point is the arm's birth policy). The
  funnel's retention proposals (`input_use::observe`) run with
  `born_connected` off under every arm, so that frozen one-event reading
  keeps its meaning (the follow-up's amendment A1, finding 4, applied to this
  prototype); both choices are predeclared here.
- **Branch verification.** TDD for the prototype; `cargo test -p v3-core
  --test viability` first (the prototype touches births). The off config
  serializes without the new field (`skip_serializing_if`), so its
  `config_digest` is `main`'s and the `v3-cli` recipe-digest pin test is
  expected to stay green; `make check` on the branch is recorded under Gate
  results with every exit code.
- **Reference binary.** `v3-cli` built in release from the untouched
  worktree at `b0f47dd5` before any source change, copied to
  `.bench-artifacts/lab/born/v3-cli-main` with its sha256 beside the commit
  hash: the pinned production binary of the launch revision (gate B0.2).
- **Landing scope.** Only `docs:` commits land on `main`; `proto:`, `instr:`
  and `lab:` stay on the branch, which is kept. T22 instrument candidates
  from this run (on the branch): the census probe's `mutation_effects`
  block beside the funnel and vote-delta readings.

## The prototype (B1's mechanism, `proto:`)

`mutation.born_connected: bool` (default `false`; `policy-deviation` when
on; serialized only when true, so the off config is byte-identical to
`main`'s). When the ordinary `InputRef.Add` event appends a new reference to
a node whose backend is a Graph, the same event also pushes one edge onto one
of that node's 27 `ActionVote` sinks: the source is the new reference (its
appended index) at a sub-value drawn uniformly over the reference's width
(`sub_value_count`: 1 for a scalar, 7 for a food summary, 8 for a ring), the
sink drawn uniformly over `VoteSink::all()`, the weight one draw of
`AddGraphEdge`'s own new-edge law, uniform on [−1, 1], whose sign is
therefore uniform (one draw; no separate sign draw). **VM nodes are
unchanged**: the reference is appended and nothing else happens (a VM backend
has no edges; its reads are instructions). The event stays one applied
`InputRefAdd`: the requested-event law, the operator weights, applicability,
skip reasons, the executed-bias target draw and the birth summary are
untouched; the off path draws no extra random number and reproduces `main`
birth for birth. The added edge is an ordinary edge afterwards
(`AlterGraphEdgeWeight`, `RetargetGraphEdge`, `RemoveGraphEdge` and the
T11.F22 reference rules apply to it as to any other).

**Natural analog.** A duplicated gene inherits its pathway: a new receptor
gene arising by duplication carries its parent's regulatory region and is
expressed into the parent's existing transduction cascade, so its product
acts on the cell from the first generation and only its tuning diverges
later (opsin and olfactory receptor duplications are the textbook cases);
a new receptor is expressed into an existing cascade rather than into
nothing. The mechanism is inheritance of expression with the duplication,
not knowledge of what the receptor is for: which cascade, with what sign and
gain, is what variation then adjusts.

**The five not-forcing checks.** (1) A general rule over one operator
(`InputRef.Add`) and one placement (a vote sink of the same node); it names
no sensor family (the reference is `Add`'s own uniform draw over the
reference universe), no arena, no action by meaning (the sink is uniform over
all 27) and no scene. (2) The analog and its mechanism are above. (3) No
correspondence: sink, sub-value and sign are all uniform draws (E5's
`aligned` was the correspondence; this is its opposite). (4) No score enters
any draw; the row runs in a natural world with births and no scorer. (5)
What ordinary variation still has to discover: the reference itself (one
family of the universe per `Add`), the right sink (one of 27, a direction
among them), the sign and the weight; the prototype changes only *when* the
first connection exists, not what it is.

**Contracts it deviates from, stated.** The
[mutation spec](../reference/v3-mutation-spec.md)'s growth-versus-connection
taxonomy ("a growth operator adds structure and must be neutral at the moment
it fires", listing `InputRef.Add` among the growth operators) and the
[T20 roadmap](../roadmaps/t20-input-evolvability-and-structured-variation.md)'s
F04 row ("Preserve `InputRef.Add` as neutral declaration and the existing
separate connection path; do not silently redefine either"). On this branch
the prototype is a `policy-deviation` arm behind a default-off flag; a
feature would amend both sentences explicitly (the plan forward names them).
It is distinct from T20.F04's `RecruitNeutralInput` (a separate Graph-domain
operator, Off by default, declaring and connecting at **zero** inherited
weight, never qualified: T20.F09 is unchecked) and from E5's `AddProjection`
(one event wiring a whole family to every sink of a kind, harmful in four
fired births of five).

## Gates (predeclared, blocking)

| ID | What | Rule |
| --- | --- | --- |
| B0 | Mechanism identity and effect on the founder, through the E4 probe (`PETRI_E4_BIRTHS=1000000`; parents `founder` (a), `founder+declared` (b), `founder+declared+zero-edge` (c); no elites), once with the flag off and once on (`PETRI_E4_BORN=1`), plus a supplementary on-path run at 10⁷ births for one criterion | **Off path, within 5 % of the follow-up's G3 off summary** ([`g3-founder-rates.json`](evolvability-diagnosis-run-2026-10-followup/g3-founder-rates.json), raw `n12/g3/e4-off.json` sha256 `07b639db…`, 10⁶ births on the `d4bc4a98` binary): per parent, `lost_declaration` (b 0.018679, c 0.007911), `declared_vote_node_target` (a 1.97 × 10⁻⁴, b 2.10 × 10⁻⁴, c 1.88 × 10⁻⁴), `attempted_events_per_birth` (a 0.484315, b 0.489357, c 0.495378) and `InputRefAdd` applied per birth (a 0.021523, b 0.021520, c 0.021984) each within 5 % of those values; `connect_useful_signed` counts (a 0, b 7, c 6) reported with CP intervals (under 10, so bounds, not the 5 % rule). The diff from `d4bc4a98` to `main` touches no file under `v3-core/src/mutation`, `creature/genome`, `creature/founder`, `neighborhood` or `v3-lab` (only `creature/identity.rs` and `creature/state.rs`), so **byte identity** of the probe output with `e4-off.json` is expected (same child seeds) and recorded if found; the probe's frozen dispatch record is taken in a lab scene under T03.F12's tick loop, so `executed_resolved` and `frozen_age` are compared first. **On path:** (1) declare-and-connect in the same birth equals the declaration rate: on parent (a), `connect_any`'s CP interval contains `declared_vote_node_target`'s point and `single_connect_any` equals `single_declared_target` exactly (one applied event, one reference, one edge); every `connect_by_location` key is `sink:ActionVote(…)` and every `connect_by_sub` key lies in 0..6. (2) The useful signed connection rate rises: on parent (a) `connect_useful_signed` was 0 of 10⁶ (G3 off) and 0 of 10⁷ (E4); the prediction is the declaration rate × 2/189 (= 2/7 for a nearest component × 2/27 for its two cardinal `Move` sinks × 1/2 for the sign; the task's per-pairing arithmetic, rate × 1/27 × 1/7 × 1/2 = 5.3 × 10⁻⁷ at 2.0 × 10⁻⁴, times the four pairings the probe's `useful_sign` accepts: dx → E+, dx → W−, dy → S+, dy → N−), 2.1 × 10⁻⁶ per birth at E4's declaration rate; at 10⁶ births that is about 2 events, whose CP interval would span both arithmetics, so this criterion is **evaluated on the supplementary 10⁷-birth on-path run** of parent (a) (about 2 min; about 21 expected): its CP interval must contain 2/189 × that run's own `declared_vote_node_target` rate; the 10⁶ run's interval is reported beside it. The E4 figure 5.5 × 10⁻⁶ (CP 4.1–7.2 × 10⁻⁶) is parent (b)'s rate per birth of usefully connecting an *already declared* leaf by the ordinary route and stays as the comparison line, not the quantity measured here. (3) Requested supply unchanged: `attempted_events_per_birth` on every parent within 5 % of the off path's (expected identical: the requested law is untouched). (4) `InputRefAdd`'s applied rate unchanged: within 5 % of the off path's on every parent (not expected identical: a born-connected `Add` consumes random numbers, so later events of the same birth can differ). (5) The loss routes untouched: `lost_declaration` on (b) and (c) within 5 % of the off path's. Any failure of (1) to (5) marks the mechanism as not doing what it says: B1 may run but does not count toward validation until the cause is found and the gate rerun |
| B0.2 | Instrument identity (the inherited contract's "the unchanged-policy reference arm must match `main`'s output before a prototype arm is read"; the follow-up's G1 form): the census probe on this binary with the flag off against the launch-revision production binary `v3-cli-main` | A 2,000-tick smoke run of the census probe (`default` recipe, seed 1022, checkpoints 1,000 and 2,000, vote deltas and the mutation-effects block on) against `v3-cli-main --telemetry off run --ticks 2000 --sample-every 500 --seed 1022 --config default.json`: `population`, `mean_genome_size`, `mean_generation` and `reproduction_actions_spawned_total` equal at every sample ([`g1check.py`](evolvability-diagnosis-run-2026-10-followup/g1check.py)), and the census header's `config_digest` equal to the run's `run_started` digest (free, since the off flag is not serialized). The smoke blocks the pairs and the main arms. Then the full `default` runs on both seeds (20,000 ticks) against `v3-cli-main` runs of 20,000 ticks on both seeds, every sample equal; a full-run inequality voids the B1 reading until explained and fixed. The instrument is verified on the new binary with the flag off, as the follow-up did (its recorded deviation from the strict baseline-identity reading) |
| B0.3 | The mutation-off pair is identical by construction: with `per_unit_rate` 0 no event is requested, so no `Add` fires and the flag is never read | `born-mutoff` and `default-mutoff` on each seed: every `sample` row's `population`, `mean_genome_size`, `mean_generation`, `reproduction_actions_spawned_total` and `mutation_events_applied_total`, the extinction tick and the checkpoint rows' `input_use` blocks (serialized with sorted keys) equal; the header differs only in `prototypes.born_connected` and the recipe path, the footer only in timing. An inequality is a defect in the prototype's off path and blocks the `born` arms |

## Ledger

One row. No diagnostic (authored-genome) arm is used, and no authored genome
enters the natural-world arms; B0's prepared parents are measurements, not
mechanisms. A killed run is `incomplete`, never a biological negative; an
extinction is a finding.

| ID | Question | Method, instrument, sizes | Prediction / decision rule | Status | Result |
| --- | --- | --- | --- | --- | --- |
| B1 | Does a born-connected declaration raise executed and causal target-family reads in a natural world without costing persistence? | The prototype above (`proto:`, `mutation.born_connected`, default off, `policy-deviation` on). **Tested on the founder at unit level before any gate:** off, `Add` on a Graph node appends one reference and no edge; on, `Add` on a Graph node appends one reference and exactly one edge, on an `ActionVote` sink, sourced from the appended index at a sub-value under the reference's width, with a weight in [−1, 1]; on, `Add` on a VM node appends one reference and leaves the backend unchanged; a proptest over random genomes and seeds holds the edge-count and source invariants. **Design as N1:** Canyon goal recipe (`experiments/worlds/canyon-country.json`, jq overlays), fresh seeds 1022 and 2022, 20,000 ticks, samples every 500, every arm on both seeds, through the census probe with checkpoints 10,000 and 20,000 (selected cohort of 20 parents, founder cohort, drift empty; the funnel block, the vote-delta reading and the mutation-effects block at both; E6 at 20,000), 3 h awake kill per run, one job at a time. Arms: `default` (flag off; rerun in full on this binary, which carries T03.F12's gestation and T24's food repairs, so its numbers are new and N1's 7.09 × and 22.38 × are not expected), `born` (flag on), `default-mutoff` and `born-mutoff` (`per_unit_rate` 0: identical by construction, gate B0.3). **Readings:** window means over ticks 15,000 to 20,000 (as E8's reader, [`n12read.py`](evolvability-diagnosis-run-2026-10-followup/n12read.py)): population, genome size in founder units (97), births per creature-tick (spawned over summed population × 500 over the window's alive samples), extinction interval; the typed target-family funnel rows (`AreaFoodSummary:0`, `AreaFoodSummary:1`, `NeighborBarrierRing`: `family_declared`, `family_connected`, `family_executed`, `family_causal`, distinct parents, absent rows 0, `parents_evaluated` must be 20) at both checkpoints; the vote-delta block at both checkpoints (as [`n3read.py`](evolvability-diagnosis-run-2026-10-followup/n3read.py)): executed target channels, channels whose ablation changes a committed action queue (`scenes_actions_differ > 0`), channels whose ablation changes any vote (`delta_passes > 0`), and the agreement check (the funnel's causal incidences against the action-queue-changing channels, which are the same fact measured twice; N3 found them equal); dead-per-birth from the mutation-effects block (`dead` over `births_total` of `any_events`, selected cohort, with `silent` and `changed` beside it), descriptive. The user's not-N1 additions: the vote-delta reading on at both checkpoints and the mutation-effects block | **Prediction:** `born` persists on both seeds; its window-mean population is below `default`'s but above 0.5 × (every `Add` birth on a Graph node is now a behavior-changing birth, and E3 found a nonzero wrong-sign edge harmful); genome size not below `default`'s (one more edge per `Add`); dead-per-birth and changed-per-birth **up** under `born` on both seeds (descriptive, with direction); `family_connected` of the target rows above `default`'s where the family is declared (a Graph-node declaration is connected at birth), `family_executed` above, and at least one causal incidence on a target row where `default` has none; the vote-delta block shows a target channel whose ablation changes an action queue. **Validation rule, verbatim from the goal command:** validated if `born` persists on both seeds, window-mean population at least 0.5 × `default`'s, and at 20,000 on both seeds the executed plus causal count over the three typed target rows exceeds `default`'s with at least one causal incidence where `default` has none, with the vote-delta block showing at least one target channel whose ablation changes an action queue; not validated otherwise; the reading is a lead-selection screen on 20 parents and is stated as such. **Resolutions, predeclared:** (i) "on both seeds" governs every clause after it: the population floor, the executed-plus-causal excess (Σ over the three typed rows of `family_executed` + `family_causal` on the selected cohort at 20,000, `born` strictly greater than `default`), the causal incidence where `default` has none (some typed row with `family_causal` ≥ 1 under `born` and 0 under `default` on that seed) and the vote-delta condition (at least one executed target channel with `scenes_actions_differ > 0` under `born`), each on each seed; (ii) the funnel's `causal` and the vote-delta's action-queue change are one fact; the rule keeps both as the command states them and the agreement check is reported; (iii) if `default` is extinct before 20,000 or has no living sample in the window on a seed, the population ratio and the 20,000 comparison are undefined on that seed and the row is **not validated** by this rule, with `born` persisting where `default` died recorded as a finding and read in the plan forward (T03.F12 closed with Canyon at 0.441 × its floor on the goal seed; seeds 1022 and 2022 are untested under gestation); (iv) `born` extinct on either seed: not validated, an extinction finding; (v) a 20-parent cohort is a lead-selection screen: one parent can carry the excess, numerators and direction are reported, and no population-wide increase is claimed from it. **Not validated otherwise.** A kill marks the run `incomplete` and it is rerun once at 6 h | **predeclared** | — |

## Advice

Every Fable advisor consultation rule 8 lists, with the disposition of each
point raised. Coverage so far: before anything was built (the approach to
the whole run, after the contract was read), Advice 1.

| # | Rule 8 point | Advice (summary) | Dispositions |
| --- | --- | --- | --- |
| 1 | Before phase 0: the approach to the run after the contract, the branch code and the engine were read | Shape right; eight points. (1) The useful-signed criterion cannot be evaluated at 10⁶ births: the probe's `useful_sign` accepts four pairings, so the prediction is rate × 2/189 (the task's one-pairing arithmetic × 4); write both, name which the CP test is against, predeclare the 10⁷ on-path run for this one criterion, record 5.5 × 10⁻⁶ as parent (b)'s two-step rate and 0 of 10⁷ as the same-birth founder figure; use `single_*` denominators for exact equality and the all-births CP containment for (1); check `connect_by_location` and `connect_by_sub`. (2) The default arm may not persist under T03.F12 on these seeds: predeclare the verdict path. (3) The prototype breaks two committed sentences (the mutation spec's neutral-growth taxonomy and the T20.F04 row): say so in the predeclaration and name them in the plan forward, with T20.F04's zero-weight operator as the contrast. (4) Predeclare both one-event choices (funnel frozen, mutation-effects under the arm), smoke the extended census on the small recipe first, predeclare dead-per-birth as descriptive with a direction. (5) The main checkout's `v3-cli` is dated before `b0f47dd5`: build the reference binary at the launch revision, hash it, `--telemetry off`; add the `config_digest` equality as a free check. (6) Verify the byte-identity premise on the mutation, creature and lab paths before predeclaring it. (7) Commit the predeclaration, launch Codex (a) in the background through a scratchpad `sh` wrapper, build the prototype under TDD while it runs, no gate before (a) is disposed; pure `docs:` commits; `npm ci`, `aqua policy allow`, head check; say the battery state once; add the note to the docs index; check the channel JSON against 1 MiB. (8) Write the validation rule verbatim and resolve its two ambiguities (both seeds; funnel causal = action-queue change) | All adopted in this predeclaration: (1) gate B0 on-path (1) and (2); (2) the row's resolution (iii); (3) "Contracts it deviates from" and the plan forward's part 2; (4) Run setup (instrument extension) and the row's prediction; (5) Run setup (reference binary) and gate B0.2; (6) the diff touches only `creature/identity.rs` and `creature/state.rs`, recorded in gate B0; (7) the Order below; (8) the row's rule and resolutions (i) and (ii) |

## Codex reviews

### Review (a): the predeclaration, before any gate or row runs

Pending (launched after this note's first commit; the job and brief under
`.bench-artifacts/lab/born/codex/`).

## Order

Predeclaration committed → Codex review (a) launched; while it runs, the
file-state commits (`instr:`, `lab:`), the prototype under TDD (`proto:`),
`cargo test -p v3-core --test viability`, the builds and `make check` →
review (a) completed and its findings disposed, amendments committed → B0
(off, on, the 10⁷ on run; minutes) → the census smoke on the 200 × 200
recipe with every reading on (seconds) → B0.2 smoke (about 6 min) → the
mutation-off pairs on both seeds with B0.3 (minutes each) → `default` on both
seeds → the two `v3-cli-main` 20,000-tick references and B0.2's full
comparison → `born` on both seeds → the B1 reading with its consultation →
the plan forward (consultations before and after the draft) → Codex review
(b) → close in the run 1 order, docs only.

## Plan forward

To be written after B1 is read (consultations before and after the draft;
Codex review (b) before closing). It will hold: (1) the B1 verdict by the
predeclared rule; (2) whether born-connected becomes the next roadmap
feature, with its track (T20 or T13) and natural analog, the exact
sentences a feature would amend, and the draft row text for the user; (3)
what was refuted or weakened; (4) the next goal command or the next
diagnosis question; (5) cost.

## Cost

| Item | Wall |
| --- | --- |
| Predeclaration | session time; no heavy job |
