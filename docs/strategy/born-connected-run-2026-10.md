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
unresolved at 8 pairs). Status: **complete** (every gate passed; B1 completed and validated by
its predeclared rule; Codex reviews (a) and (b) disposed; landed on `main`
as docs only, the prototype, instrument and probe kept on the branch).

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
  probe with its births block and the vote-delta reading, observation only),
  `lab:` (the E4 `transition_rates` probe), `proto:` (the one default-off
  mechanism prototype). `instr:`, `lab:` and `proto:` stay on the branch;
  only `docs:` lands on `main`.
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
  **The instrument reads the prototype flag through the serialized
  `MutationConfig`** (absent means off; the probe sets it the same way), so
  the `instr:` and `lab:` commits build without the prototype and the
  frozen instrument of gate B0.2 is that build (review (a) finding 4).
- **Instrument extension (`instr:`).** The census checkpoint row gains a
  `births` block: the goal profile's neighborhood-read birth classes
  (`neighborhood::births::per_birth_result`: production births from each
  selected parent at the arm's own mutation config, supply drawn on the
  parent's own genome size, 100 births per parent, the read's seed formula
  8,000,000 + 1,000 × (parent index + 1) + 9,000 + birth index), pooled over
  the 20 parents and reported per parent: `dead`, `silent` and `changed`
  over `births_total` (per all births) and over `any_events.trials` (the
  event-bearing births; the goal's `evolved_dead_per_mutated_births` is this
  denominator, review (a) finding 2), with the block's wall time. Advice 1
  point 4 and the first predeclaration named `mutation_effects::observe` as
  the source; its result carries no births tally (finding 1), so the
  neighborhood read's own classes are used. The funnel's retention
  proposals (`input_use::observe`) run on the arm's config with the flag
  dropped (frozen one-event meaning under every arm, as the follow-up's
  amendment A1 finding 4 did for its prototype); the births block runs
  under the arm's actual policy (the prototype flag as the arm sets it, and
  `per_unit_rate` as set, so a mutation-off arm's births are all zero-event
  births); the vote-delta reading draws no mutation. All three stated here.
- **Branch verification.** TDD for the prototype; `cargo test -p v3-core
  --test viability` first (the prototype touches births): passed, 28 tests,
  before any gate. The off config serializes without the new field
  (`skip_serializing_if`), so its `config_digest` is `main`'s and the `v3-cli`
  recipe-digest pin test is expected to stay green; `make check` on the
  branch is recorded under Gate results with every exit code.
- **Reference binary.** `v3-cli` built in release from the untouched
  worktree at `b0f47dd5` before any source change, copied to
  `.bench-artifacts/lab/born/v3-cli-main` (sha256 `ff904e98…`, beside the
  commit hash): the pinned production binary of the launch revision (gate
  B0.2's smoke).
- **Frozen instrument.** The census test binary built from the `instr:` +
  `lab:` state without the `proto:` commit (both built there, exit 0),
  copied to `.bench-artifacts/lab/born/census-frozen` (sha256 `41b5413e…`;
  the probe built there too, `probe-frozen` `eecb3a5d…`): it runs the
  control recipes (`default` and `default-mutoff`, both seeds, 20,000 ticks)
  and the prototype binary's off arms must reproduce its samples and its
  timing-free checkpoint blocks (gate B0.2's full comparison; the inherited
  contract's "the unchanged-policy reference arm must match `main`'s output
  before a prototype arm is read", finding 4).
- **Branch and commits.** Branch `worktree-born-connected` from `b0f47dd5`:
  `docs:` `338b0f23` (the predeclaration), `71216a57` (amendment A1);
  `instr:` `24d0f553`; `lab:` `3abc1b70`; `proto:` `765e429c` (the prototype
  with its unit tests, draw-coverage test, serialization test and proptest
  invariant, plus the proptest regression file the first run of that
  invariant wrote on a generated genome with a truncated sink catalog); then
  this run's `docs:` commits. The prototype's release test binaries built
  at `765e429c`: `census-proto` sha256 `5cb7857f…`, `probe-proto`
  `fbc71263…` (rebuilt at `a172c734` after the clippy fix: `55ad8560…`,
  `892dcd44…`). `cargo test -p v3-core --test viability` 28 passed before
  the prototype was committed; the seven born-connected unit tests pass.
  The `docs:` commits that land on `main`, in order: `338b0f23`,
  `71216a57`, `ee131937`, `849d8808`, `098e3533`, `009e8b1e`, `f8dfa62d`,
  `a931077e` and this closing commit; the `proto:` fix `a172c734` stays on
  the branch with the rest.
- **Amendment A2 (pre-data, Advice 2, before any gate ran).** The census's
  `births` block embeds the arm's `mutation_config` (the flag itself), so
  the identity checkers hash the block's counts without that entry and
  compare the entry apart: equal under B0.2 (ii), `false` against `true` at
  every checkpoint under B0.3, beside the predeclared header differences.
  The B0 reader's two identity readings of the off path (the deterministic
  projection against G3's raw output, with each probe version's own block
  dropped, and the `genome_size`, `executed_resolved` and `frozen_age`
  comparison) are recorded, not gating, as the gate text says; the 5 %
  rules and the on-path checks are the gate. The driver stops before the
  gates unless `make check` exited 0 on the branch.
- **Amendment A3 (post-data reader corrections at gate B0.3: after the
  first identity comparison had run and its sample, extinction, footer and
  block results had been seen, before any B1 interpretation).** `identity.py` failed the first
  B0.3 comparison on two checks that were the reader's, not the runs': the
  header's `world` field is the run's label (`PETRI_E2_NAME`, which names
  the arm), so it is an allowed difference beside `config_digest`,
  `prototypes` and `recipe`; and the births-flag comparison was vacuous
  for a pair extinct before its first checkpoint (no `births` block
  exists) and now says so instead of failing. Every compared quantity
  (samples, extinction, footer, blocks) had passed; the driver stopped as
  designed and was relaunched after the correction, re-running the checks.
- **Amendment A4 (post-data reader corrections after Codex review (b),
  stated as such; no recorded number changed).** `b1read.py` now stores the
  summed absolute consumer weight in the per-channel lists and the grouped
  tuples and distinct-parent counts (`distinct_action_change_readings`,
  `distinct_action_change_parents`), so the readings the note counts can
  be reconstructed from the committed summary (finding 13); it makes the
  agreement check part of `seed_validates` and exits non-zero on a
  disagreement (finding 14; agreement held, so the verdict is unchanged);
  `identity.py` compares the whole removed `mutation_config` entry apart
  (every key but the flag equal in both modes; finding 15). The summaries
  were regenerated from the stored raw output with the corrected readers;
  every gate and the verdict reproduce.
- **Landing scope.** Only `docs:` commits land on `main`; `proto:`, `instr:`
  and `lab:` stay on the branch, which is kept. T22 instrument candidates
  from this run (on the branch): the census probe's `births` block beside
  the funnel and vote-delta readings, and the probe's single-event-`Add`
  tallies.

## The prototype (B1's mechanism, `proto:`)

`mutation.born_connected: bool` (default `false`; `policy-deviation` when
on; serialized only when true, so the off config is byte-identical to
`main`'s). When the ordinary `InputRef.Add` event appends a new reference to
a node whose backend is a Graph, the same event also pushes one edge onto one
of that node's 27 `ActionVote` sinks: the source is the new reference (its
appended index) at a sub-value drawn uniformly over the reference's width
(`sub_value_count`: 1 for a scalar, 7 for a food summary, 8 for a ring), the
sink drawn uniformly over `VoteSink::all()` (by index, never by meaning), the
weight one draw of `AddGraphEdge`'s own new-edge law, uniform on [−1, 1],
whose sign is therefore uniform (one draw; no separate sign draw). **VM
nodes are unchanged**: the reference is appended and nothing else happens (a
VM backend has no edges; its reads are instructions). The event stays one
applied `InputRefAdd`: the operator definitions and weights, the
requested-supply law (`Binomial(genome_size, per_unit_rate)`) and the birth
summary's accounting are unchanged; within a birth that fired, the later
events can differ from the off path's (the connected reference leaves
`Prune`'s eligible set, the new edge joins the edge pools, and three extra
random numbers shift the later draws), so such births' applied and skip
records are not the off path's (review (a) finding 7); the off path draws no
extra random number and reproduces `main` birth for birth. The added edge is
an ordinary edge afterwards (`AlterGraphEdgeWeight`, `RetargetGraphEdge`,
`RemoveGraphEdge` and the T11.F22 reference rules apply to it as to any
other). A production Graph node always carries the fixed 94-sink catalog, so
the drawn sink exists; a test genome with a truncated catalog may miss it,
and the event then stays a plain declaration.

**Natural analog.** A new receptor gene arising by duplication carries its
parent's regulatory region and is expressed into the parent's existing
transduction cascade, so its product acts on the cell from the first
generation and only its tuning diverges later (opsin and olfactory receptor
duplications are the textbook cases): a new receptor is expressed into an
existing cascade rather than into nothing. That inherited expression is what
motivates immediate pathway integration here. The prototype approximates the
integration by a fresh random connection: it models neither duplication nor
pathway inheritance (the reference is a fresh draw, the destination and gain
are drawn independently of any existing pathway), and what variation then
adjusts, which cascade with what sign and gain, is the same in both (review
(a) finding 16).

**The five not-forcing checks.** (1) A general rule over one operator
(`InputRef.Add`) and one placement (a vote sink of the same node); it names
no sensor family (the reference is `Add`'s unchanged catalog sampling law,
with its typed-food and upstream-slot draws; finding 17), no arena, no
action by meaning (the sink is uniform over all 27) and no scene. (2) The
analog and its mechanism are above. (3) No correspondence: sink, sub-value
and sign are all uniform draws (E5's `aligned` was the correspondence; this
is its opposite). (4) No score enters any draw; the row runs in a natural
world with births and no scorer. (5) What ordinary variation still has to
discover: the reference itself (one draw of the catalog per `Add`), the
right sink (one of 27, a direction among them), the sign and the weight.
What the prototype changes beyond *when* the first connection exists
(finding 15): the first destination class is fixed to vote sinks (never a
compute node or another surface), each `Add` on a Graph node adds a unit
(the edge) with its size-derived supply and gestation cost, and the
connected reference leaves `Prune`'s eligible set at birth; the mutation-off
pairs cannot separate these, since the intervention never fires there, so a
positive verdict is a screen of the combined mechanism and timing-specific
attribution waits on matched controls.

**Contracts it deviates from, stated.** The
[mutation spec](../reference/v3-mutation-spec.md)'s growth-versus-connection
taxonomy ("a growth operator adds structure and must be neutral at the moment
it fires", listing `InputRef.Add` among the growth operators) and its
InputRef-domain `Add` paragraph (the new reference "is wired into nothing on
either backend" and becomes addressable only through a later connection
operator, which the `apply_add` doc comment repeats; finding 18), and the
[T20 roadmap](../roadmaps/t20-input-evolvability-and-structured-variation.md)'s
F04 row ("Preserve `InputRef.Add` as neutral declaration and the existing
separate connection path; do not silently redefine either"). On this branch
the prototype is a `policy-deviation` arm behind a default-off flag; a
feature would amend all three explicitly, preserving the stated VM behavior
(the plan forward names them). It is distinct from T20.F04's
`RecruitNeutralInput` (a separate Graph-domain operator, Off by default,
declaring and connecting at **zero** inherited weight, never qualified:
T20.F09 is unchecked) and from E5's `AddProjection` (one event wiring a
whole family to every sink of a kind, harmful in four fired births of five).

## Gates (predeclared, blocking)

Every gate must pass, after any disposition and rerun, before the step it
blocks runs: B0 before any world arm, B0.2's smoke before the pairs, B0.3 and
B0.2's full comparison before the `born` arms are read; the row's
agreement check (below) blocks the interpretation of its causal and
vote-delta readings (review (a) finding 10).

| ID | What | Rule |
| --- | --- | --- |
| B0 | Mechanism identity and effect on the founder, through the E4 probe (`PETRI_E4_BIRTHS=1000000`; parents `founder` (a), `founder+declared` (b), `founder+declared+zero-edge` (c); no elites), once with the flag off and once on (`PETRI_E4_BORN=1`), plus a supplementary on-path run at 10⁷ births for the useful-signed criterion | **Off path, within 5 % of the follow-up's G3 off summary** ([`g3-founder-rates.json`](evolvability-diagnosis-run-2026-10-followup/g3-founder-rates.json), raw `n12/g3/e4-off.json` sha256 `07b639db…`, 10⁶ births on the `d4bc4a98` binary): per parent, `lost_declaration` (b 0.018679, c 0.007911), `declared_vote_node_target` (a 1.97 × 10⁻⁴, b 2.10 × 10⁻⁴, c 1.88 × 10⁻⁴), `attempted_events_per_birth` (a 0.484315, b 0.489357, c 0.495378) and `InputRefAdd` applied per birth (a 0.021523, b 0.021520, c 0.021984) each within 5 % of those values; `connect_useful_signed` counts (a 0, b 7, c 6) reported with CP intervals (under 10, so bounds, not the 5 % rule). The diff from `d4bc4a98` to `main` touches no file under `v3-core/src/mutation`, `creature/genome`, `creature/founder`, `neighborhood` or `v3-lab` (only `creature/identity.rs` and `creature/state.rs`), so a **deterministic projection** of the probe output (every field but `wall_secs` and the probe's config metadata) is expected equal to `e4-off.json`'s (same child seeds) and recorded if found; the frozen dispatch record is taken in a lab scene under T03.F12's tick loop, so each parent's `genome_size`, `executed_resolved` and `frozen_age` are compared first and any difference is explained before rates are read (finding 11). **On path:** (1) declare-and-connect in the same birth equals the declaration rate: on parent (a), among the single-event `InputRefAdd` births that declared the target on the vote node (`single_add_births.declared_vote_node_target`), `connect_any_given_declared` is exactly 1 and every `single_add_births.connect_by_location` key is `sink:ActionVote(…)` with every `connect_by_sub` key in 0..6 (finding 6: the pooled final-child destinations include later ordinary edges and stay descriptive); over all births, `connect_any`'s CP interval contains `declared_vote_node_target`'s point (multi-event births can erase either). (2) The useful signed connection rate rises: on parent (a) `connect_useful_signed` was 0 of 10⁶ (G3 off) and 0 of 10⁷ (E4); the prediction is the declaration rate × 2/189 (= 2/7 for a nearest component × 2/27 for its two cardinal `Move` sinks × 1/2 for the sign; the task's per-pairing arithmetic, rate × 1/27 × 1/7 × 1/2 = 5.3 × 10⁻⁷ at 2.0 × 10⁻⁴, times the four pairings the probe's `useful_sign` accepts: dx → E+, dx → W−, dy → S+, dy → N−), 2.1 × 10⁻⁶ per birth at E4's declaration rate. The **direct mechanism check** is `single_add_births.connect_useful_signed_given_declared` on parent (a) against 2/189 = 0.01058 on the 10⁷-birth run (about 2,000 such declarations, an interval a few per cent wide): its 95 % CP interval must contain 2/189. The all-birth `connect_useful_signed` rate and its CP interval are reported beside it against 2/189 × that run's `declared_vote_node_target`, a compatibility check at 95 %, not a proof of uniform sampling (finding 12): the final-child count also carries later edits and the ordinary routes; at 10⁶ births (about 2 events) the interval spans both arithmetics and is reported only. The E4 figure 5.5 × 10⁻⁶ (CP 4.1–7.2 × 10⁻⁶) is parent (b)'s rate per birth of usefully connecting an *already declared* leaf by the ordinary route and stays as the comparison line, not the quantity measured here. (3) Requested supply unchanged: `attempted_events_per_birth` on every parent within 5 % of the off path's (expected identical: the requested law is untouched). (4) `InputRefAdd`'s applied rate: within 5 % of the off path's on every parent (not expected identical: later events of a fired birth can differ, finding 7). (5) Integrated-loss equivalence: `lost_declaration` on (b) and (c) within 5 % of the off path's, with the route classifications (`lost_declaration_by_ops`) reported beside and this limit stated: `lost_declaration` reads the reference's *value* on the final child, so a born-connected second `AreaFoodSummary(0)` declaration in the same birth as a loss masks the occurrence's loss (both in one birth: rare), and a total within 5 % establishes neither unchanged routes nor occurrence survival (finding 8). A failure of (1) or (2) means the mechanism does not do what it says; a failure of (3), (4) or (5) beyond tolerance is investigated and explained; in every case B0 must pass after the disposition and a rerun before any world arm runs |
| B0.2 | Instrument identity (the inherited contract's "the unchanged-policy reference arm must match `main`'s output before a prototype arm is read"; the follow-up's G1 form, strengthened after finding 4): the census probe on the prototype binary with the flag off against (i) the launch-revision production binary `v3-cli-main` and (ii) the frozen instrument built without the prototype | (i) **Smoke:** a 2,000-tick run of the census probe (`default` recipe, seed 1022, checkpoints 1,000 and 2,000, vote deltas and the births block on) against `v3-cli-main --telemetry off run --ticks 2000 --sample-every 500 --seed 1022 --config default.json`: the sample tick sets equal (500, 1,000, 1,500, 2,000, both sides, no missing tick), every compared field present on both sides, `population`, `mean_genome_size`, `mean_generation` and `reproduction_actions_spawned_total` equal at every tick, both runs completed to 2,000 (the probe's footer, the run's completion record), and the census header's `config_digest` equal to the run's `run_started` digest (free, since the off flag is not serialized); the checker `b02check.py` fails on any missing tick, field or completion record (finding 5: the follow-up's `g1check.py` compared common ticks only). The smoke blocks the pairs and the main arms. (ii) **Full comparison:** the frozen instrument runs `default` and `default-mutoff` on both seeds (20,000 ticks; the reference runs); the prototype binary's `default` and `default-mutoff` runs must equal them on every sample row (the fields above plus `mutation_events_applied_total` and `mean_energy`), the extinction tick, the footer's `ticks` and `extinct`, and every checkpoint block (`input_use`, `e6`, `vote_deltas`, `births`, `living`, `selected_parents`, `selected_generations`, `age_deciles`, `energy_deciles`), each serialized once with sorted keys and compact separators and hashed; the header's `prototypes` entry, `config_digest` and the timing fields are excluded and listed. A full-run inequality blocks the `born` arms from being read until explained and fixed |
| B0.3 | The mutation-off pair is identical by construction: with `per_unit_rate` 0 no event is requested, so no `Add` fires and the flag is never read | `born-mutoff` and `default-mutoff` on each seed: the same sample fields, extinction tick, footer and timing-free checkpoint blocks as B0.2 (ii) equal. The header's `config_digest` **differs** (the flag serializes when true) and `prototypes.born_connected` differs; both differences are verified, and nothing else in the header but the recipe path may differ (finding 9). The diagnostic blocks' policies under this pair: the funnel's retention proposals use the forced one-event supply with the flag dropped (so they are the default's); the births block uses the arm's actual zero-rate policy (every birth zero-event, so the flag never fires there either); the vote-delta reading draws no mutation. An inequality is a defect in the prototype's off path and blocks the `born` arms |

**Gate results.** **B0 passed** (every gating check, 25 of 25, with 6
recorded identity checks beside them, on the prototype binary at `a172c734`; summary
[`b0.json`](born-connected-run-2026-10/b0.json), reader
[`b0read.py`](born-connected-run-2026-10/b0read.py); raw `b0/e4-off.json`
sha256 `74d8861e…`, `e4-on.json` `7539a870…`, `e4-on-10m.json` `1dfaa6bb…`;
10⁶ births per parent in 40 s and 35 s, 10⁷ in 347 s). **Off path:
identical to G3's raw off output** (`07b639db…`) in the deterministic
projection on all three parents, `genome_size`, `executed_resolved` and
`frozen_age` included, so the frozen dispatch record is the same under
T03.F12's tick loop and every compared rate agrees to the last digit
(attempted events 0.484315, 0.489357, 0.495378 per birth; `InputRefAdd`
0.021523, 0.021520, 0.021984; `lost_declaration` 0.018679 and 0.007911;
useful signed 0, 7 and 6 of 10⁶). **On path.** (1) On the founder, 15,121 of
10⁶ births were single-event `InputRefAdd` births and 138 of them declared
`AreaFoodSummary(0)` on the vote node; every one of the 138 carried the
born edge (`connect_any_given_declared` 1.0, CP 0.974 to 1), and at 10⁷
1,393 of 1,393 (CP 0.9974 to 1); every destination an `ActionVote` sink
(all 27 kinds hit at 10⁷, 37 to 63 each) and every sub-value in 0..6 (175
to 240 each); over all births `connect_any` equals
`declared_vote_node_target` exactly (194 = 194 at 10⁶, 2,019 = 2,019 at
10⁷), and the pooled final-child destinations of the founder's new leaf
are `ActionVote` sinks only at both sizes (no later ordinary edge from it
in these samples). (2) Useful signed: the direct check at 10⁷, 19 of 1,393
single-`Add` declarations = 0.01364 (CP 0.00823 to 0.02122), contains
2/189 = 0.01058; the all-birth rate 26 of 10⁷ = 2.6 × 10⁻⁶ (CP 1.70 to
3.81 × 10⁻⁶) contains the predicted 2/189 × 2.019 × 10⁻⁴ = 2.14 × 10⁻⁶, and
the chain's other links sit at their arithmetic too: a cardinal `Move` 295
(2.95 × 10⁻⁵ against 4/27 × declared = 2.99 × 10⁻⁵), a nearest component
onto its cardinal `Move` 44 (4.4 × 10⁻⁶ against 4.27 × 10⁻⁶), wrong sign
18 (1.8 × 10⁻⁶); at 10⁶ the all-birth count was 6 (CP 2.2 × 10⁻⁶ to
1.3 × 10⁻⁵) and the single-`Add` share 2 of 138, reported only. The
founder's same-birth useful signed rate therefore moves from 0 of 10⁷ (E4)
to 2.6 × 10⁻⁶ per birth, about half of parent (b)'s two-step rate per birth
(5.5 × 10⁻⁶), which presupposes a surviving declaration. (3) The requested
supply is identical on and off on every parent. (4) `InputRefAdd` applied
per birth 0.021533 against 0.021523, 0.021513 against 0.021520, 0.021970
against 0.021984 (within 0.1 %). (5) Integrated loss 0.018691 against
0.018679 on (b) and 0.007909 against 0.007911 on (c) (within 0.1 %): the
integrated-loss equivalence the gate asks for, no more; the operator-class
tallies behind it differ (on (b), `InputRefAdd+InputRefPrune` 197 against
170, and a four-operator class `InputRefAdd+InputRefPrune+InputRefRawFieldMutation+InputRefSwap`
appears once), as a fired birth's later draws differ.
**Census smoke passed** (`smoke/small.json`, the follow-up's 200 × 200
Canyon recipe with 400 founders, seed 33, 300 ticks, checkpoint 300, vote
deltas and the births block on, the prototype binary with the flag off):
exit 0 in 5 s, 213 living at the checkpoint, census 0.15 s, vote-delta
reading 0.010 s, births block 0.066 s (20 parents × 100 births: 653
event-bearing births; dead 4, silent 358, changed 291, that is 0.6 %,
54.8 % and 44.6 % of the mutated births), no executed target channel among
the 20 sampled parents (so the ablation branches rest on their unit tests
until the main runs, as in the follow-up's smoke), 8 threads,
`config_digest` `cf5a12f8…` under the off flag; the block timings project
to seconds per checkpoint at 20,000-tick genome sizes (Advice 2 point 6).
On parents (b) and (c) the pooled connections of the declared reference
rise under the on path (744 and 852 against 532 and 658 at 10⁶, increases
of 212 and 194 beside on-path declaration counts of 208 and 185) because a
duplicate `AreaFoodSummary(0)` declaration now arrives with its edge and the
probe matches edges by reference value; the final-child value match gives
no partition of those totals into born and ordinary edges, so the
single-`Add` tallies are the only clean reading. **B0.2 (i) smoke passed** (7 of
7 checks, [`b02-smoke.json`](born-connected-run-2026-10/b02-smoke.json),
checker [`b02check.py`](born-connected-run-2026-10/b02check.py)): the
prototype binary's census run (`g1-default-s1022`, 2,000 ticks, 8 threads,
60 s) and `v3-cli-main`'s run (60 s) carry the same four sample ticks, every
compared field present and equal at every tick, both completed, and the
census header's `config_digest` equals the run's `run_started` digest;
the checkpoints at 1,000 and 2,000 took 0.17 s and 0.20 s with the births
block at 0.07 s and 0.10 s (774 and 900 event-bearing births of 2,000; dead
1 and 0) and no executed target channel among the sampled parents.
**Frozen references** (the frozen instrument, `census-frozen`, 8 threads):
`default-mutoff` extinct at 5,557 (seed 1022) and 7,509 (seed 2022) in
105 s and 121 s; `default` complete at 20,000 in 697 s and 637 s (T24's
regrowth repairs make a 20,000-tick Canyon run about twice as fast as the
follow-up's), window-mean population 2,486 and 3,007, genome size 7.58 ×
and 3.89 × the founder, lowest samples 795 (tick 2,500) and 815 (3,000):
the default persists on both seeds under T03.F12's gestation, so the row's
resolution (iii) is not triggered. **B0.3 passed on both seeds** (after
amendment A3; [`b03-s1022.json`](born-connected-run-2026-10/b03-s1022.json),
[`b03-s2022.json`](born-connected-run-2026-10/b03-s2022.json), checker
[`identity.py`](born-connected-run-2026-10/identity.py)): `born-mutoff`
equals `default-mutoff` on every sample field at every tick (11 and 15
samples), the extinction tick (5,557 and 7,509), the footer and the
checkpoint sets (none reached: both pairs die in the founder trough before
10,000, as E8's and N1's pairs did), the header differing only in
`config_digest` (the flag serializes when true), `prototypes.born_connected`
(`false` against `true`), `recipe` and `world`. **B0.2 (ii) on the
mutation-off runs passed on both seeds** (`b02-mutoff-s*.json`): the
prototype binary's `default-mutoff` runs equal the frozen instrument's on
every compared field, the extinction tick, the footer and the header
entirely. **B0.2 (ii) on the `default` runs:** **passed on both seeds** ([`b02-default-s1022.json`](born-connected-run-2026-10/b02-default-s1022.json), [`b02-default-s2022.json`](born-connected-run-2026-10/b02-default-s2022.json)): the prototype binary's `default` runs (692 s and 627 s) equal the frozen instrument's on every sample field at all 40 ticks, the footer, and every timing-free checkpoint block at 10,000 and 20,000 (`input_use`, `e6`, `vote_deltas`, `births`, the cohort and decile fields) by hash, the header entirely: the prototype binary with the flag off is the instrument built without the prototype, reading for reading, and the `born` arms may be read.
**`make check` on the branch**: a first run at `765e429c` exited 2,
stopped at `rust-clippy` (dead code: `apply_to_node` had become test-only),
fixed by `a172c734` (`#[cfg(test)]`); the rerun at `a172c734` exited 0 with every
step passing, the `v3-cli` recipe-digest pin test included (the off flag
is not serialized), log `make-check-branch.log`; the prototype binaries
were rebuilt and re-hashed after the fix (`census-proto` `55ad8560…`,
`probe-proto` `892dcd44…`, superseding the hashes under Branch and
commits), and the driver started only on that exit line.

## Ledger

One row. No diagnostic (authored-genome) arm is used, and no authored genome
enters the natural-world arms; B0's prepared parents are measurements, not
mechanisms. A killed run is `incomplete`, never a biological negative; an
extinction is a finding.

| ID | Question | Method, instrument, sizes | Prediction / decision rule | Status | Result |
| --- | --- | --- | --- | --- | --- |
| B1 | Does a born-connected declaration raise executed and causal target-family reads in a natural world without costing persistence? | The prototype above (`proto:`, `mutation.born_connected`, default off, `policy-deviation` on). **Tested on the founder at unit level before any gate:** off, `Add` on a Graph node appends one reference and no edge and is `apply_to_node` draw for draw; on, `Add` on a Graph node appends one reference and exactly one edge, on an `ActionVote` sink, sourced from the appended index at a sub-value under the reference's width, with a weight in [−1, 1], compute nodes and the other node untouched; on, `Add` on a VM node appends one reference and leaves the backend unchanged; the flag reaches `Add` through the config and off is the default; the flag is absent from the serialized config when off and round-trips when on; over 4,096 seeds the born edge reaches all 27 vote sinks and all 8 sub-values of a ring with signs near even; a proptest over generated genomes holds the one-reference, at-most-one-edge, source and untouched-node invariants (exactly one edge on a full sink catalog). **Design as N1:** Canyon goal recipe (`experiments/worlds/canyon-country.json`, jq overlays), fresh seeds 1022 and 2022, 20,000 ticks, samples every 500, every arm on both seeds, through the census probe with checkpoints 10,000 and 20,000 (selected cohort of 20 parents, founder cohort, drift empty; the funnel block, the vote-delta reading and the births block at both; E6 at 20,000), 3 h awake kill per run, one job at a time. Arms: `default` (flag off; rerun in full on this binary, which carries T03.F12's gestation and T24's food repairs, so its numbers are new and N1's 7.09 × and 22.38 × are not expected), `born` (flag on), `default-mutoff` and `born-mutoff` (`per_unit_rate` 0: identical by construction, gate B0.3). **Readings:** window means over ticks 15,000 to 20,000 (as E8's reader, [`n12read.py`](evolvability-diagnosis-run-2026-10-followup/n12read.py)): population, genome size in founder units (97), births per creature-tick (spawned over summed population × 500 over the window's alive samples), extinction interval; the typed target-family funnel rows (`AreaFoodSummary:0`, `AreaFoodSummary:1`, `NeighborBarrierRing`: `family_declared`, `family_connected`, `family_executed`, `family_causal`, distinct parents, absent rows 0, `parents_evaluated` must be 20) at both checkpoints; the vote-delta block at both checkpoints (as [`n3read.py`](evolvability-diagnosis-run-2026-10-followup/n3read.py)): executed target channels, channels whose ablation changes a committed action queue (`scenes_actions_differ > 0`), channels whose ablation changes any vote (`delta_passes > 0`); the **agreement check** (finding 3): per typed family, the funnel's `family_causal` (distinct parents) against the number of distinct parents with any vote-delta channel of that family having `scenes_actions_differ > 0`, and per channel row, the funnel's channel-row `causal` (parents) against the parents whose vote-delta channel of that (family, channel) changes an action queue; both are the same ablation on the same panels, so a disagreement is an instrument failure that blocks the interpretation of the causal and vote-delta readings until explained (N3's 2 = 2 was one checkpoint's coincidence of one-channel families); dead-per-birth from the births block (`dead` over `births_total` and over `any_events.trials`, selected cohort, with `silent` and `changed` beside it), descriptive. The user's not-N1 additions: the vote-delta reading on at both checkpoints and the births block | **Prediction, with a direction or its absence for every reading (finding 14):** `born` persists on both seeds; its window-mean population **below** `default`'s but above 0.5 × (every `Add` birth on a Graph node *can now be* a behavior-changing birth, since its edge is live at birth, and E3 found a nonzero wrong-sign edge harmful; structural connection does not guarantee a vote or action change, finding 13); genome size **not below** `default`'s (one more unit per `Add`); births per creature-tick: **no directional prediction** (N1 found it uninformative); extinction: none predicted; per birth on the selected cohort at both checkpoints, `dead` and `changed` **up** and `silent` **down** under `born` (hypotheses, descriptive); funnel typed rows at both checkpoints, descriptively: `family_declared` **unchanged** (the `Add` draw is unchanged), `family_connected` **up** where the family is declared (a Graph-node declaration is connected at birth), `family_executed` **up**, `family_causal` at least one incidence at 20,000 where `default` has none; vote-delta: channels moving any vote **up**, and at least one target channel whose ablation changes an action queue; E6: **no prediction**. The validation reads 20,000; the 10,000 checkpoint is descriptive. **Validation rule, verbatim from the goal command:** validated if `born` persists on both seeds, window-mean population at least 0.5 × `default`'s, and at 20,000 on both seeds the executed plus causal count over the three typed target rows exceeds `default`'s with at least one causal incidence where `default` has none, with the vote-delta block showing at least one target channel whose ablation changes an action queue; not validated otherwise; the reading is a lead-selection screen on 20 parents and is stated as such. **Resolutions, predeclared:** (i) "on both seeds" governs every clause after it: the population floor, the executed-plus-causal excess (Σ over the three typed rows of `family_executed` + `family_causal` on the selected cohort at 20,000, `born` strictly greater than `default`), the causal incidence where `default` has none (some typed row with `family_causal` ≥ 1 under `born` and 0 under `default` on that seed) and the vote-delta condition (at least one executed target channel with `scenes_actions_differ > 0` under `born`), each on each seed; (ii) the funnel's `causal` and the vote-delta's action-queue change are one fact measured twice; the rule keeps both as the command states them, and the agreement check above must hold for the clause to be read; (iii) if `default` is extinct before 20,000 or has no living sample in the window on a seed, the population ratio and the 20,000 comparison are undefined on that seed and the row is **not validated** by this rule, with `born` persisting where `default` died recorded as a finding and read in the plan forward (T03.F12 closed with Canyon at 0.441 × its floor on the goal seed; seeds 1022 and 2022 are untested under gestation); (iv) `born` extinct on either seed: not validated, an extinction finding; (v) a 20-parent cohort is a lead-selection screen: one parent can carry the excess, numerators and direction are reported, and no population-wide increase is claimed from it. **Alternative explanations named now (finding 15):** the fixed vote-sink destination class, the added units and their supply and gestation costs, the pruning protection of connected references, backend composition of the cohort, generation count, panel exposure of the target channels (the follow-up's instrument finding: the battery never presents them), drift and shared ancestry within a 20-parent sample, and checkpoint sampling swings (N1's 14 → 1); a positive verdict is a screen of the combined mechanism. **Not validated otherwise.** A kill marks the run `incomplete` and it is rerun once at 6 h | **completed** (validated by the predeclared rule on both seeds; a lead-selection screen) | Eight runs through the census probe on the prototype binary `census-proto` (summary [`b1-summary.json`](born-connected-run-2026-10/b1-summary.json), reader [`b1read.py`](born-connected-run-2026-10/b1read.py); the recipes' and binaries' sha256 in `b1/recipes-and-binaries.sha256`; every run complete, exit 0: `default` 692 and 627 s, `born` 693 and 642 s, the pairs 95 to 121 s awake; 8 threads; on AC at every launch). **Window means over ticks 15,000 to 20,000 (final tick-20,000 values in brackets).** `default` seed 1022: population 2,486 [2,577], genome size 735 = **7.58 ×** the founder [817], births 0.02064 per creature-tick, generation 346, size per generation 2.13 units; seed 2022: 3,007 [3,642], 378 = **3.89 ×** [405], 0.02045, generation 356, 1.06. `born` seed 1022: population **2,876** [3,065], **5.52 ×** [631], births 0.01994, generation 373, 1.44; seed 2022: **2,681** [2,878], **6.08 ×** [751], 0.02069, generation 457, 1.29. Population ratios `born`/`default` **1.16** (seed 1022) and **0.89** (seed 2022). Lowest samples: `default` 795 (tick 2,500) and 815 (3,000); `born` 1,509 (3,000) on seed 1022, a shallower trough than the default's, and **10 living at tick 4,500 on seed 2022** (the samples from 3,000 on: 217, 186, 366, 10, 952, then 1,390 and 2,685 at 10,000; the 500-tick sampling cannot bound the true minimum), a near-extinction the "reaches 20,000" criterion does not catch, as the follow-up's `slow` arm's 52 was not, beside the default's own minimum of 815 at 3,000 on that seed (1,760 at 4,500): one seed each way, a recorded risk, not a consistent cost. **Pairs:** identical by construction and in fact (B0.3), extinct at 5,557 and 7,509. **Funnel at 20,000** (selected cohort, 20 parents, typed rows declared/connected/executed/causal): seed 1022 `default` `NeighborBarrierRing` **5/4/4/2**, `AreaFoodSummary:0` **0/0/0/0**; `born` **10/9/9/3** and **8/6/5/5**; seed 2022 `default` 4/0/0/0 and 6/3/3/3; `born` **6/6/6/2** and **7/7/7/4**; `AreaFoodSummary:1` 0 everywhere. Executed plus causal over the three rows: **22 against 6** (seed 1022) and **19 against 6** (seed 2022). Causal where the default has none: `AreaFoodSummary:0` 5 against 0 on seed 1022, `NeighborBarrierRing` 2 against 0 on seed 2022. At 10,000: seed 1022 `default` 3/0/2/0 and 1/1/1/0, `born` **10/10/10/8** and 3/2/2/0; seed 2022 `default` 3/0/0/0 and 9/5/5/0, `born` 4/4/1/0 and 5/5/3/3. The seed-1022 barrier-ring row swings from 10/10/10/8 at 10,000 to 10/9/9/3 at 20,000 (the checkpoint-sampling alternative named in advance), and the default's base has moved: N1's default sampled no causal target read, this binary's default samples two and three causal parents; on the row where the default is causal `born` is higher (3 against 2 on seed 1022, 4 against 3 on 2022), but those rows do not satisfy the default-has-none clause, which passed on the other row of each seed. **Vote-delta block at 20,000:** executed target channels 18 against 5 (seed 1022) and 16 against 3 (seed 2022); channels whose ablation changes a committed action queue **8 against 2** and **6 against 3**; channels whose ablation moves any vote 12 against 2 and 14 against 3; at 10,000 the same three counts 14/8/12 against 3/0/0 and 6/5/6 against 5/0/0. **Agreement:** the funnel's `family_causal` equals the distinct parents with an action-changing vote-delta channel of that family at every checkpoint on every arm (24 of 24 family comparisons, 12 per seed: four runs × two checkpoints × three typed rows, and per channel row likewise), so the two readings are one fact and the clause is read. **What the causal reads are under `born`** (every one read by 1 or 2 Graph edges of |weight| 0.3 to 1.5; the census records no edge provenance, and the default's own causal read on seed 2022 is also one Graph edge, so these weights are consistent with either origin; the mechanism's live fingerprint is instead `family_connected` tracking `family_declared` under `born`, 9 of 10, 6 of 8, 6 of 6 and 7 of 7 at 20,000, where the default leaves most declarations unconnected, 4 of 5, 0 of 0, 0 of 4 and 3 of 6): on seed 1022 at 20,000, `AreaFoodSummary:0` `gradient_x` (channel 1) on two parents moving a `Move` sink (12 and 16 of 264 scene executions change the committed queue), `nearest_dx` (3) on two parents of identical reading (weight 0.79, moving a `Reproduce` sink, 37 executions), `total_ratio` (0) on one parent (weight 0.29 moving `Move` by up to 8.3, 5 executions), and `NeighborBarrierRing` directions 2 and 6 (`Reproduce`, `Move`; 8, 8 and 17 executions); on seed 2022, `gradient_x` on three parents with matching consumer count and summed |weight| 0.89 but two distinct grouping tuples (parent 30: 1,177 delta passes, 79 executions; parents 46 and 49: 1,231 delta passes, 68 and 73), moving `Move`, `nearest_dy` (4) on one parent (two edges, |weights| 1.51, 11) and `NeighborBarrierRing` direction 4 on two parents (weight 0.95, `StealEnergy`, 28). Grouping the action-changing channels by (family, channel, consumer counts, summed |weight|, delta passes) gives 7 distinct readings among the 8 channels at 20,000 on seed 1022 (4 among 8 at 10,000) and 4 among 6 on seed 2022 (5 among 5 at 10,000, three of them on one parent), on 8 and 4 distinct parents at 20,000 (8 and 3 at 10,000); the grouped tuples are stored in the summary (`distinct_action_change_readings`). These are phenotype-summary groups: matching tuples (parents 30 and 31 on seed 1022; 46 and 49 on 2022 on the ring read) make shared ancestry a hypothesis, not a finding, different tuples can belong to one lineage, and independent origins were not measured (the census carries no parent identity). Under `default` the causal reads are `NeighborBarrierRing` direction 0 by VM reads (seed 1022, two parents, 4 and 22 executions) and `AreaFoodSummary:0` `gradient_y` (2) by one Graph edge of weight 1.0 on three parents of identical reading (seed 2022, 20 executions): the default's own causal target reads under T03.F12, which N1's default had none of. **Births block** (selected cohort, 100 production births per parent; dead / silent / changed per event-bearing birth): seed 1022 `default` 0.0005 / 0.736 / 0.264 at 10,000 and 0.0010 / 0.708 / 0.291 at 20,000, `born` 0.0021 / 0.777 / 0.221 and 0.0005 / 0.747 / 0.253; seed 2022 `default` 0.0108 / 0.744 / 0.245 and 0.0006 / 0.779 / 0.220, `born` 0.0042 / 0.822 / 0.173 and 0.0011 / 0.702 / 0.297: no consistent direction (dead up in two of four comparisons, changed up in one). E6 gradient-informative at 20,000: `default` 18.0 % and 19.9 %, `born` 20.2 % and 17.7 %. Every reading took under 2 s per checkpoint. **Prediction check.** Met: `born` persists on both seeds; at 20,000 `family_connected`, `family_executed` and `family_causal` of the declared target rows above the default's on both seeds; vote-moving and action-changing channels up on both seeds and checkpoints; a causal incidence where the default has none on both seeds at 20,000. **At 10,000 the funnel predictions fail in places** (review (b) finding 3): seed 2022 `AreaFoodSummary:0` connected 5 = 5 (not above), executed 3 < 5, declared 5 < 9 (causal 3 > 0); seed 2022 `NeighborBarrierRing` executed 1 against 0 with causal 0 = 0; seed 1022 `AreaFoodSummary:0` causal 0 = 0; `AreaFoodSummary:1` 0 everywhere. The 20,000 validation verdict stands apart from these. **Contradicted:** connected and executed above the default's at both checkpoints (the 10,000 exceptions above: a fifth contradiction, not named in Advice 3's brief and put to the advisor in Advice 5); population below the default's (1.16 × on seed 1022, 0.89 × on 2022); genome size not below the default's (5.52 × against 7.58 × on seed 1022, 6.08 × against 3.89 × on 2022: one seed each way, within the seed-to-seed spread N1 showed); `family_declared` unchanged (**up**: 10 against 5 and 8 against 0 on seed 1022, 6 against 4 and 7 against 6 on 2022; a connected declaration is not prunable and can be kept by selection, the alternative named in advance); dead and changed per birth up and silent down (contradicted and underpowered: dead counts of 1 to 15 per 1,400 to 1,900 event-bearing births cannot show a direction either way; on the founder 2.2 % of births apply an `InputRefAdd` (B0), each on a Graph node, and the evolved cohorts' share was not measured; and the block evaluates the original `neighborhood-v1` battery alone, which presents zero for every target-family channel and lacks the funnel's recorded and authored contexts, with `dead` meaning every battery execution is `NoOp`, not an ecological death, review (b) finding 12). Births per creature-tick (no prediction): 0.01994 against 0.02064 and 0.02069 against 0.02045. The contradictions were taken to the advisor (Advice 3). **Decision rule: validated** on both seeds by every clause (reaches 20,000; population 1.16 × and 0.89 ×; executed plus causal 22 > 6 and 19 > 6; a causal row where the default has none on each seed; 8 and 6 action-changing target channels; `parents_evaluated` 20; agreement holds): a lead-selection screen on 20 parents and two seeds of one world, in which one lineage can carry a count, with the `born` near-extinction on seed 2022 (10 living) and the default's own causal reads under T03.F12 recorded beside it |

## Advice

Every Fable advisor consultation rule 8 lists, with the disposition of each
point raised. Coverage so far: before anything was built (the approach to
the whole run, after the contract was read), Advice 1; before the gate and
world chain was launched (after review (a) was disposed and the prototype
committed), Advice 2; after the B1 reading was drafted and before it was
recorded, with its four contradicted predictions, and before the plan
forward was drafted, Advice 3; after the plan forward was drafted, Advice 4;
before closing, with the fifth contradicted prediction put explicitly,
Advice 5.

| # | Rule 8 point | Advice (summary) | Dispositions |
| --- | --- | --- | --- |
| 5 | Before closing (rule 8), after the review (b) dispositions were committed; also the consultation on the fifth contradicted prediction (the 10,000 funnel exceptions on seed 2022: food-summary connected 5 = 5 and executed 3 < 5 with causal 3 > 0, the barrier ring executed 1 > 0), which Advice 3's brief had not named | (1) Record the fifth contradiction as a contradicted prediction at one checkpoint on one seed, beside the checkpoint-sampling alternative named in advance (N1's default 14 → 1, this run's 8 → 3 on seed 1022), one more reason the row is a lead-selection screen; it touches nothing in parts 1 and 2. (2) One closing commit, note only: this row, Status complete, the docs commit list for the pick, the cost's closing row. (3) The close order with its checks: no run process left (the `cxc-` broker is another session's, not to be killed; the reviews ran through `codex exec`); `ExitWorktree` keep; `main` at `b0f47dd5` and clean before the first pick, stop and report if it advanced; purity of the nine docs commits verified by path, then one cherry-pick in order; `make check-docs`; worktree removed (`--force` acceptable on ignored files since every commit is on the branch and the evidence copy diffed clean), branch kept. (4) The report's items: the verdict, what is recorded against it, the recommendation with its condition, the three user items with the draft row quoted | (1) Recorded: it changes nothing in the verdict or the recommendation; (2) this commit; (3) and (4) followed |
| 4 | After the plan forward was drafted (rule 8) | Structure and recommendation right (T20, the named tension, route (a)); five fixes. (1) The agreement count is 24 of 24 (four runs × two checkpoints × three typed rows), not 12 of 12, in the B1 cell and part 1. (2) The draft row prescribes the realization (the follow-up's review (b) finding 9 struck the same from the T03.F12 row): keep the Goal to the phenomenon and the analog, move the sink, sub-value and weight law to the Notes as the tested realization. (3) Part 2 lacks two parts of the format: the evidence rows and what ordinary variation still has to discover. (4) The F04 sentence is a roadmap edit the user makes at row-addition time; (1) and (2) are the spec's. (5) T11.F21 came from a research note; T03.F12 is the exact precedent, cite it alone. Then the batched process to the finish: launch Codex (b) now, write the memory while it runs (`MEMORY.md` at its limit: a short line, shorten an existing one), one dispositions commit, Advice 5, the closing commit, the close order with the pick list verified by path | All adopted in this commit: (1) both places; (2) the row and its Notes; (3) the two parts added; (4) stated under the amendment sentences; (5) reworded; the process followed |
| 3 | After the B1 reading was drafted, before it was recorded; the contradiction consultation (population not lower on seed 1022, size one seed each way, `family_declared` up, the births block without direction); before the plan forward was drafted, with its sketch presented | Reading right on the rule and the arithmetic; five fixes and one tension. (1) Count the lineages from the stored tuples instead of estimating: group action-changing channels by (family, channel, consumers, weight, delta passes), report the number per seed as an upper bound on lineages. (2) Do not attribute the causal edges to the mechanism: the census records no provenance and the default's causal read is also one Graph edge; the live fingerprint is `family_connected` tracking `family_declared` under `born`. (3) The trough needs its sample sequence, the sampling limit, the default's own trough and `born`'s shallower one on the other seed: one seed each way, a recorded risk; frame size the same way. (4) Flag the 10,000 → 20,000 swing on seed 1022 and the moved default base (two and three causal parents under T03.F12 where N1's default had none); on the row where the default is causal `born` is higher but that row cannot satisfy the default-has-none clause. (5) The births block is contradicted and underpowered; take the 2 % from the summary or drop it. Plan forward: name the tension with T20's qualification chain (F12 enables only qualified opportunities; F04's non-goal is no availability switch), state the two routes (an ordinary mechanism row read by the goal profile, or an opt-in arm for F09's lab question), recommend the row with the reason, carry the goal-profile risks, draft the row with the amended analog, T20 over T13 by the ownership line; then the process to the finish in order with batched commits | All adopted: (1) to (5) in the B1 cell before it was recorded (the lineage counts later recast as phenotype-summary groups, review (b) finding 2); the plan forward's part 2 names the tension, the routes and the reason; the close follows the listed order. Coverage gap (review (b) finding 3): the brief named four contradicted predictions and not the 10,000-checkpoint funnel exceptions on seed 2022, which are put to the advisor in Advice 5 |
| 2 | Before the gates and the world chain (the phase's approach), after amendment A1, the `instr:`, `lab:` and `proto:` commits, the frozen build and the readers | Chain correctly sequenced, predeclaration sound; six points. (1) `b0read.py` would fail B0 by construction: the projection compares whole parent objects, but the reference carries a `decay` block and this probe `single_add_births`; and the identity readings sat in the gate's `all(...)` although the note calls them recorded; drop each version's block before comparing and make both identity checks non-gating. (2) `identity.py` would fail B0.3 on the `births` block, which embeds `mutation_config.born_connected`; strip it before hashing and compare it apart with the expected value per mode; record as a pre-data amendment A2. (3) Gate the driver on `make check` exiting 0 (prototype results count only after clippy passes). (4) Record Advice 2 before launching, with the commit hashes, the four binary hashes, the viability result and A2. (5) Turn economy: one background waiter per milestone (B0, smoke, B0.3, B0.2 (ii), DONE/STOP), batched note commits after B0, after the identity gates and with the B1 reading. (6) Watch, not change: the smoke's `births_secs` and `vote_delta_secs` projected to 20,000-tick genomes; a lone `config_digest` failure in B0.2 (i) with equal samples is a question about what `run_started` hashes, to amend, not a stop; B0 (2) at 10⁷ is a 95 % containment with a one-in-twenty false failure, so a lone failure near 0.0106 is investigated before anything is concluded | All adopted: (1) and (2) in the readers before launch, (2) also as amendment A2 (Run setup); (3) in the driver; (4) this commit (the `make check` line follows in the next); (5) the waiters; (6) read at the smoke and gate wakeups |
| 1 | Before phase 0: the approach to the run after the contract, the branch code and the engine were read | Shape right; eight points. (1) The useful-signed criterion cannot be evaluated at 10⁶ births: the probe's `useful_sign` accepts four pairings, so the prediction is rate × 2/189 (the task's one-pairing arithmetic × 4); write both, name which the CP test is against, predeclare the 10⁷ on-path run for this one criterion, record 5.5 × 10⁻⁶ as parent (b)'s two-step rate and 0 of 10⁷ as the same-birth founder figure; use `single_*` denominators for exact equality and the all-births CP containment for (1); check `connect_by_location` and `connect_by_sub`. (2) The default arm may not persist under T03.F12 on these seeds: predeclare the verdict path. (3) The prototype breaks two committed sentences (the mutation spec's neutral-growth taxonomy and the T20.F04 row): say so in the predeclaration and name them in the plan forward, with T20.F04's zero-weight operator as the contrast. (4) Predeclare both one-event choices (funnel frozen, the births reading under the arm), smoke the extended census on the small recipe first, predeclare dead-per-birth as descriptive with a direction. (5) The main checkout's `v3-cli` is dated before `b0f47dd5`: build the reference binary at the launch revision, hash it, `--telemetry off`; add the `config_digest` equality as a free check. (6) Verify the byte-identity premise on the mutation, creature and lab paths before predeclaring it. (7) Commit the predeclaration, launch Codex (a) in the background through a scratchpad `sh` wrapper, build the prototype under TDD while it runs, no gate before (a) is disposed; pure `docs:` commits; `npm ci`, `aqua policy allow`, head check; say the battery state once; add the note to the docs index; check the channel JSON against 1 MiB. (8) Write the validation rule verbatim and resolve its two ambiguities (both seeds; funnel causal = action-queue change) | All adopted in the predeclaration: (1) gate B0 on-path (1) and (2), with the single-event-`Add` tallies after review (a); (2) the row's resolution (iii); (3) "Contracts it deviates from" and the plan forward's part 2, with a third sentence after review (a) finding 18; (4) Run setup (instrument extension) and the row's prediction, the source corrected after review (a) finding 1; (5) Run setup (reference binary) and gate B0.2 (i); (6) the diff touches only `creature/identity.rs` and `creature/state.rs`, recorded in gate B0; (7) the Order below; the docs index lists companion notes only (the follow-up note is not in it), so no entry is added; (8) the row's rule and resolutions (i) and (ii), the agreement check strengthened after review (a) finding 3 |

## Codex reviews

### Review (a): the predeclaration, before any gate or row ran

Job `.bench-artifacts/lab/born/codex/review-a.out.md` (brief `review-a.md`),
verdict `not-ready`, 10 blocking and 8 advisory. Every finding is adopted as
**amendment A1** (the sections above carry the fixes, each marked with its
finding number); no confirmation round is run, since no finding changes the
row's question, its arms or the user's validation rule, and every fix is a
predeclared wording, a gate rule, a reader rule or an instrument change
tested on the branch before any gate runs.

| # | Finding (short) | Severity | Disposition |
| --- | --- | --- | --- |
| 1 | `mutation_effects::observe` has no `births_total` or `any_events`; those are `births::BirthResult`'s | blocking | Adopted: the births block uses `neighborhood::births::per_birth_result` on the selected cohort under the arm's production mutation config, 100 births per parent, the read's seed formula (Run setup) |
| 2 | Per-all-births and per-mutated-births are different denominators; the goal indicator divides by event-bearing births | blocking | Adopted: both reported and named; `any_events.trials` is the goal indicator's denominator |
| 3 | `family_causal` counts parents per typed family; the vote reader counts channels, so 1 against 2 is consistent; N3's equality was contingent and unenforced | blocking | Adopted: the agreement check compares per typed family after deduplicating vote channels by (parent, family), and per channel row; disagreement blocks interpretation (B1 Readings) |
| 4 | Four sample fields do not verify the funnel, vote-delta or births readings; the contract requires the frozen instrument built without `proto:` | blocking | Adopted: the instrument reads the flag through the serialized config so the `instr:`/`lab:` state builds without the prototype; that frozen binary runs the control recipes and the prototype's off arms must match every sample and timing-free checkpoint block (B0.2 (ii), Run setup) |
| 5 | `g1check.py` compares common ticks only and passes on truncated references or missing fields | blocking | Adopted: `b02check.py` requires the expected tick sets, present fields, completion or extinction records and matching terminal ticks (B0.2 (i)) |
| 6 | `connect_by_location` pools every birth's final-child edges; a born-connected `Add` followed by an ordinary edge can legitimately land on a compute node | blocking | Adopted: the destination and sub-value checks read the probe's new single-event-`InputRefAdd` tallies; pooled destinations stay descriptive (B0 (1)) |
| 7 | "Untouched" overstates: applicability, later targets and outcome records can change within a fired birth | blocking | Adopted: the prototype paragraph and B0 (4) reworded; the applied-rate tolerance kept as the empirical check |
| 8 | `lost_declaration` is an integrated final-child indicator, not a route measurement; a duplicate preserves the reference's value | blocking | Adopted: B0 (5) is integrated-loss equivalence with routes reported beside and the value-versus-occurrence limit stated; not every tolerance failure proves a wrong mechanism |
| 9 | B0.3's header differs in `config_digest` too; the forced one-event diagnostic could fire the prototype in a mutation-off arm | blocking | Adopted: the digest difference is expected and verified; identity is over live samples, extinction, footer and timing-free blocks; the three diagnostic policies stated (funnel flag dropped, births block under the actual zero-rate policy, vote delta draws nothing) |
| 10 | "B1 may run but does not count" contradicts blocking gates and lets rows run with a mechanism finding unresolved | blocking | Adopted: every gate must pass after disposition and rerun before the step it blocks (Gates preamble); identity and agreement failures named as stops |
| 11 | Whole-file byte identity is impossible (`wall_secs`, metadata); unchanged source paths do not fix the frozen inputs | advisory | Adopted: a deterministic projection; `genome_size`, `executed_resolved` and `frozen_age` compared first (B0 off path) |
| 12 | CP containment is a compatibility check, not proof of uniform sampling; the prediction is about created edges, the probe counts surviving ones | advisory | Adopted: the direct check is the useful share among single-event `Add` declarations against 2/189; the all-birth interval is the compatibility check with its limits stated (B0 (2)) |
| 13 | Structural connection does not guarantee behavioral change | advisory | Adopted: "can now be"; dead and changed fractions are hypotheses (B1 prediction) |
| 14 | No direction for births per creature-tick, `family_declared`, silent fraction, vote-moving channels, E6; checkpoints unclear | advisory | Adopted: a direction or its absence for every reading; validation at 20,000, 10,000 descriptive (B1 prediction) |
| 15 | The intervention also fixes the destination class, adds units, protects the connected reference from pruning; the pairs cannot separate these | advisory | Adopted: named under the not-forcing checks and the row's alternative explanations; a positive verdict is a screen of the combined mechanism |
| 16 | The analog claims duplication and pathway inheritance the prototype does not implement | advisory | Adopted: the analog motivates immediate integration; the prototype approximates it by a fresh random connection and models neither duplication nor inheritance |
| 17 | `Add`'s sampler is not uniform over the enumerated universe | advisory | Adopted: "`Add`'s unchanged catalog sampling law, with its typed-food and upstream-slot draws" |
| 18 | The mutation spec's InputRef-domain `Add` paragraph ("wires nothing on either backend") also needs amending | advisory | Adopted: named as the third sentence a feature would amend, preserving the VM behavior |

### Review (b): the whole note before landing

Job `.bench-artifacts/lab/born/codex/review-b.out.md` (brief `review-b.md`),
verdict `not-ready`, 7 blocking and 10 advisory. Every finding is adopted
below; no finding changes the gate results or the B1 verdict (every gate and
the rule reproduce with the corrected readers), and no second round is run.

| # | Finding (short) | Severity | Disposition |
| --- | --- | --- | --- |
| 1 | Part 2 repeated the rejected "changes only when the first connection exists, never what it is" | blocking | Adopted: the screen evaluates immediate connection with its coupled changes; timing-specific attribution unresolved |
| 2 | "Upper bounds on lineages" from grouped tuples: phenotype-summary groups, not genealogical bounds; distinct parents 8/4 at 20,000, 8/3 at 10,000 | blocking | Adopted: tuple counts kept with the parent counts, ancestry a hypothesis, independent origins unmeasured, in the cell and part 1; the Advice 3 disposition reworded |
| 3 | The blanket "Met" for connected/executed/causal omitted the 10,000 exceptions on seed 2022 (`AreaFoodSummary:0` connected 5 = 5, executed 3 < 5, declared 5 < 9) and the zero rows | blocking | Adopted: the exceptions listed, a fifth contradiction recorded in the cell and part 3, the 20,000 verdict kept apart; the Advice 3 coverage gap recorded and put to Advice 5 |
| 4 | The three amendments leave T20's "Candidate Graph creation stays opt-in until F12" boundary and the F04 spec's separate-paths row unresolved | blocking | Adopted: route (a) made conditional on a user-approved T20 Notes exception naming the preserved F09 to F12 chain and the superseded F04 contract (part 2) |
| 5 | The draft Goal still prescribed the action-vote destination and the random gain | blocking | Adopted: the Goal keeps immediate live integration, Graph scope, the analog and the outcome; placement and gain stay in the Notes as the tested realization |
| 6 | "Route classes as the off path's" and an inferred unchanged ordinary-route count underneath the pooled connections | blocking | Adopted: integrated-loss equivalence only, the differing operator classes described (197 against 170; a four-operator class), the partition inference deleted |
| 7 | Part 1 set the trough beside "the default's own 815 there"; the default had 1,760 at 4,500 and its minimum 815 at 3,000 | blocking | Adopted: both places corrected |
| 8 | "Three parents of identical reading" on seed 2022's `gradient_x`: parent 30 has 1,177 delta passes against 1,231 | advisory | Adopted: matching consumer count and summed weight, two distinct grouping tuples |
| 9 | "At or above, not in excess": 3 > 2 and 4 > 3 are excesses that fail the default-has-none clause for another reason | advisory | Adopted: reworded in the cell and the Advice 3 row |
| 10 | "24 of 24" gating checks: `b0.json` has 25 gating and 6 recorded | advisory | Adopted: 25 of 25, 6 recorded |
| 11 | "Every single-event declaration arrives connected" measures the sampled target declaration on the founder's vote node | advisory | Adopted: qualified in parts 1 and 2; the general guarantee attributed to the code and its tests |
| 12 | The births block's limits are exposure and classification, not only rarity | advisory | Adopted: the battery-only panel, the absent extension contexts and the `NoOp` meaning of `dead` stated |
| 13 | The committed per-channel lists omit the summed weight the grouping used | advisory | Adopted: amendment A4 (weights and grouped tuples stored; summary regenerated) |
| 14 | `b1read.py` reports agreement apart from `validated`; a disagreement could coexist with a validated verdict | advisory | Adopted: amendment A4 (agreement required, non-zero exit on failure; the verdict unchanged) |
| 15 | `identity.py` compares only the flag of the removed `mutation_config` entry | advisory | Adopted: amendment A4 (the whole entry compared apart; the six identity summaries regenerated, all pass) |
| 16 | A3 is a post-data reader amendment | advisory | Adopted: reworded |
| 17 | The first `make check`'s exit code was omitted | advisory | Adopted: 2 at `765e429c`, 0 at `a172c734` |

## Order

Predeclaration committed → Codex review (a) completed and its findings
disposed, amendment A1 committed → the `instr:` and `lab:` commits (built
without the prototype; the frozen census binary copied and hashed) → the
`proto:` commit under TDD, `cargo test -p v3-core --test viability`, the
unit tests, the builds and `make check` → B0 (off, on, the 10⁷ on run;
minutes; must pass) → the census smoke on the 200 × 200 recipe with every
reading on (seconds) → B0.2 (i) smoke (about 6 min; must pass) → the frozen
instrument's reference runs (`default-mutoff`, `default` on both seeds;
about 1 h) → the prototype binary's `default-mutoff` and `born-mutoff` on
both seeds with B0.3 → its `default` on both seeds → B0.2 (ii) full
comparison (must pass before `born` is read) → `born` on both seeds → the
B1 reading with its consultation → the plan forward (consultations before
and after the draft) → Codex review (b) → close in the run 1 order, docs
only.

## Plan forward

Drafted after B1 was read (consultations before the draft, Advice 3, and
after it, Advice 4; Codex review (b) before closing).

### 1. Verdict

**B1 validated** by its predeclared rule on both Canyon seeds: `born`
persists; window-mean population 1.16 × and 0.89 × the default's; executed
plus causal target reads at 20,000 22 against 6 and 19 against 6; a causal
row where the default has none on each seed (`AreaFoodSummary:0` 5 against
0, `NeighborBarrierRing` 2 against 0); 8 and 6 target channels whose
ablation changes a committed action; the funnel and the vote-delta reading
agreeing 24 of 24 (12 per seed). Gate B0 established that the mechanism does on the
founder what it says for the sampled case (every sampled `AreaFoodSummary(0)`
declaration on the founder's vote node in a birth whose sole applied event
is `InputRefAdd` arrives connected, 138 of 138 and 1,393 of 1,393; the
useful signed share 19 of 1,393 contains 2/189; the off path identical to
G3's output), with the general Graph-node guarantee resting on the code and
its unit and property tests, and gates B0.2 and B0.3 that the instrument on
the prototype binary is the instrument built without it. What validates is
a **lead-selection screen** on 20 parents and two seeds of one world, where
one lineage can carry a count (the action-changing channels fall into 7 and 4
distinct readings at 20,000, phenotype-summary groups with shared ancestry a
hypothesis; independent origins unmeasured). Recorded against the arm: a near-extinction
trough on seed 2022 (10 living at tick 4,500, beside the default's own
minimum of 815 at 3,000 on that seed, 1,760 at 4,500, and `born`'s
shallower 1,509 on seed 1022), genome size one seed each
way (5.52 × against 7.58 ×, 6.08 × against 3.89 ×), and four contradicted
predictions (the B1 cell).

### 2. Born-connected as a roadmap feature: yes, on T20

- **Track: T20.** The T20 track's ownership line gives it the scoped input
  mechanisms ("T20 owns the scoped input mechanisms; T11 retains general
  representation, supply and inheritance, T13 whole-module recruitment,
  and T09 general learning/cognition qualification", the master roadmap's
  T20 note): born-connected changes how a declared input reference is
  first connected, not how a module is recruited (T13).
- **The tension, named.** T20's main path is F04 → F05 → F09 → F10 → F11 →
  F12, with F12 enabling "only positively qualified inherited Graph
  creation/refinement opportunities through ordinary mutation" and F04's
  non-goal "no production availability switch". Born-connected as a
  feature is a production mutation-policy change that sits outside that
  qualification chain. Two routes: **(a)** an ordinary mechanism row on
  T20 read by the goal profile in three worlds, the path T03.F12 took (N1
  validated, the row added by the user, then the feature; this run is the
  natural-world check in that path), amending the three sentences below
  and depending on T20.F04 (the neutral-recruitment contract it sits
  beside) and T14.F12 (the funnel reading), not on the unexecuted F09; **(b)** an opt-in arm for F09's lab discovery question.
  **Recommended: (a)**, because the mechanism changes the declaration
  operator for every family rather than rolling out one qualified
  candidate, the goal-profile floors in three worlds are the gate such a
  change needs, and the lab form of the question (run 6's ALT arm, E10)
  was unresolved at 8 pairs while the natural world has now answered the
  funnel question on two seeds. **Route (a) is conditional on a
  user-approved exception in the T20 track's Notes** (review (b) finding
  4): the track's own boundary says "Candidate Graph creation stays opt-in
  until F12, VM until F03's own positive qualification", and the closed
  T20.F04 spec's separate-paths row says "`InputRef.Add` stays unwired
  declaration on both backends"; a new row alone does not supersede them.
  The exception would name T20.F16 as a general mutation-policy feature
  outside the F09 to F12 candidate chain (which it leaves intact for
  qualified candidates) and record that the F04 separate-path contract is
  superseded for Graph nodes by the feature when it closes.
- **Natural analog** (as amended at review (a)): expression inherited with
  a duplication motivates immediate integration of a new receptor into an
  existing cascade; the prototype approximates that integration by a fresh
  random connection (random sink, sign and sub-value) and models neither
  duplication nor pathway inheritance; what variation then adjusts, which
  cascade with what sign and gain, is the same in both.
- **Evidence rows:** B0 (the mechanism's identity on the founder: every
  sampled single-event target declaration arrives connected, the useful signed share
  contains 2/189, the off path identical to G3's output) and B1 (the
  natural-world screen on two seeds); E4 for the two-step rates it
  shortens (declare 2.0 × 10⁻⁴, useful connect of an existing declaration
  5.5 × 10⁻⁶, loss of the unconnected declaration 1.86 × 10⁻² per birth)
  and E10 for the lab form it answers in the natural world.
- **What ordinary variation still has to discover after it:** the
  reference itself (`Add`'s unchanged catalog draw), the right sink among
  27, the sign and the weight. The screen evaluated immediate connection
  together with the changes coupled to it (the fixed vote-sink destination
  class, the added unit with its supply and gestation costs, the pruning
  protection of a connected reference, the shifted later draws of a fired
  birth); timing-specific attribution stays unresolved (review (a) finding
  15, review (b) finding 1).
- **The three sentences a feature amends** (explicitly, preserving the VM
  behavior; amendments (1) and (2) are the spec's to make, amendment (3)
  is a roadmap edit and therefore the user's, made when the row below is
  added, since rule 6 forbids this run making either): (1) the [mutation spec](../reference/v3-mutation-spec.md)'s
  growth-versus-connection taxonomy, "a growth operator adds structure and
  must be neutral at the moment it fires", which lists `InputRef.Add`; (2)
  its InputRef-domain `Add` paragraph, where the new reference "is wired
  into nothing on either backend" and becomes addressable only through a
  later connection operator (the `apply_add` doc comment repeats it); (3)
  the T20 roadmap's F04 row, "Preserve `InputRef.Add` as neutral declaration
  and the existing separate connection path; do not silently redefine
  either", which the feature redefines explicitly for Graph nodes.
- **Spec questions, not prescriptions:** the destination class (vote sinks
  only, as tested, or every sink surface); the weight law (the new-edge law
  as tested, or a smaller gain); whether T20.F04's `RecruitNeutralInput`
  (Off, never qualified) is retired or kept beside it; VM nodes unchanged
  until T20.F03; and the consequence that a connected declaration is
  unprunable under T11.F22, which this run's `family_declared` rise
  predicts for the goal reading.
- **Expected goal-profile reading:** the target rows' connected, executed
  and causal counts up at the goal checkpoints in all three worlds, with
  connected tracking declared; population not under half the previous
  closure's, and the decisive risk Canyon's goal-seed persistence (T03.F12
  closed at 0.441 × its floor there; the seed-2022 trough shows this
  mechanism can move a trough either way); genome size not reliably
  different (one seed each way here); dead-per-birth not reliably up
  (underpowered here); `family_declared` up as a consequence of pruning
  protection. Limits of this evidence: two fresh seeds of one world over
  20,000 ticks, a 20-parent cohort, no edge provenance in the census, and
  the follow-up's instrument finding that the battery never presents the
  target channels (the funnel's causal stage rests on the recorded and
  authored contexts).
- **Draft row for the T20 track, for the user to add** (rule 6 forbids the
  run adding it), in the track's own form:

  > - [ ] **T20.F16 — Born-Connected Input Declaration** — Depends on: T20.F04, T14.F12
  >   - Goal: A new sensory input is expressed into an existing pathway rather than into nothing, as a receptor gene duplicated with its parent's expression enters the parent's cascade. When ordinary variation declares a new input reference on a Graph node, the declaration arrives already wired into that node's existing pathway and live, so it is exposed to selection at once instead of waiting, silent, for a separate connection event; VM nodes are unchanged. The intended outcome is more executed and causal sensor reads of the target families at the goal checkpoints without a persistence or population cost.

  with a Notes paragraph citing this note's B1 cell and gate B0 for the
  tested realization, which belongs there and not in the row (one edge per
  `Add` on a Graph node: the appended reference at a uniform sub-value
  into one uniformly drawn `ActionVote` sink at the new-edge law's weight,
  uniform on [−1, 1]), the three amendment sentences, the spec questions
  (the destination class and the weight law among them),
  the expected goal-profile reading and its risks, `cargo test -p v3-core
  --test viability` first, and the T22 instrument candidates on the branch
  (the census's births block, the probe's single-event-`Add` tallies).
- **Priority:** appended to the master roadmap's order of new starts after
  T03.F12, the user's decision.

### 3. Refuted or weakened

- **Contradicted predictions** (the B1 cell): connected and executed
  target rows above the default's at both checkpoints (at 10,000 on seed
  2022 the food summary's connected count is equal and its executed count
  lower); population below the default's (1.16 × on seed 1022); genome
  size not below the default's (one seed each way); `family_declared`
  unchanged (up on both seeds, the pruning-protection alternative); dead
  and changed per birth up and silent down (no direction, underpowered and
  battery-limited).
- **Run 6's ALT reading** (E10, the idea's lab form, "unresolved at this
  size"): the natural-world funnel now reads positive on two seeds; the lab
  question itself stays unresolved and was not retried here.
- **E5's "projection not advanced" stands**: a projection wires a whole
  family to every sink of a kind in one event; born-connected adds one edge
  per declaration, and E5's harmful-share finding does not transfer to it.
- **Not weakened:** the diagnosis run's variation verdict; the cleanup
  diagnosis of E4 (the mechanism bypasses the silent-declaration hazard
  rather than lowering it: `lost_declaration` unchanged under B0 (5)).

### 4. The next goal command

After the user adds the row above to the T20 track and the ID to the order
of new starts, in the workflow's form (the T03.F12 command with the ID
swapped):

```
/goal Roadmap feature T20.F16 is complete on main. Read docs/workflow.md first and follow its per-feature contract exactly: confirm you are Opus 5.5 at effort medium in the main checkout on a clean main; create the feature worktree with EnterWorktree named t20-f16; delegate the flat spec and its Codex adversarial challenge rounds to roadmap-spec-owner, verify the final Codex verdict, and commit the spec there, and route requirement questions during implementation back to that same spec owner; delegate feature implementation and production-code remediation to roadmap-implementer, and, unless the workflow's Benchmark gate exempts this feature from them, the gate and goal baseline runs and their records to roadmap-benchmark-specialist and the mutation gate and test-only survivor remediation to roadmap-mutation-specialist; run the final diff review as a fresh read-only Codex Astra high job through the Codex channel; run the benchmark and mutation specialists sequentially and never alongside competing builds, tests, servers, or measurements; run make check in the worktree; ExitWorktree with keep, fast-forward main to the feature branch, then remove the worktree and its branch. Done means all of these are shown in this conversation: the T20.F16 row is checked in its track roadmap on main and its spec is Complete; make check exited 0 on the feature code now on main and make check-docs exited 0 at the commit now on main; git worktree list no longer lists the feature worktree; git status on main is clean. If a concrete blocker stops the feature, record it in the spec, report it, and stop. Stop after 80 turns.
```

### 5. Cost

The Cost table below, with the turn count in its closing row; the session's
cost line awaits `/usage`.

## Cost

| Item | Wall |
| --- | --- |
| Predeclaration, Codex review (a), amendment A1 | session time; the review remote |
| Builds (reference binary, frozen instrument, prototype twice) and `make check` twice | about 45 min in all |
| B0 | 40 s (off), 35 s (on), 347 s (on, 10⁷) |
| Census smoke | 5 s |
| B0.2 (i) smoke (census run and `v3-cli-main` run) | 60 s and 60 s |
| Frozen references (`default-mutoff`, `default`, both seeds) | 105, 697, 121, 637 s awake |
| Pairs (`default-mutoff`, `born-mutoff`, both seeds) | 100, 95, 120, 121 s awake |
| Main arms (`default`, `born`, both seeds) | 692, 627, 693, 642 s awake (about 45 min in all) |
| Census readings (funnel, vote delta, births block) | under 2 s per checkpoint |
| Codex reviews (a), (b) | remote; no host time |
| Turns | about 126 of 150 at the closing commit; the final count is in the final report |
