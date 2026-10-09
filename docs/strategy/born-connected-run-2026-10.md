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
unresolved at 8 pairs). Status: **in progress** (predeclared and amended
after Codex review (a); no gate or row has run).

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
  `lab:` state without the `proto:` commit, copied to
  `.bench-artifacts/lab/born/census-frozen` with its sha256: it runs the
  control recipes (`default` and `default-mutoff`, both seeds, 20,000 ticks)
  and the prototype binary's off arms must reproduce its samples and its
  timing-free checkpoint blocks (gate B0.2's full comparison; the inherited
  contract's "the unchanged-policy reference arm must match `main`'s output
  before a prototype arm is read", finding 4).
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

## Ledger

One row. No diagnostic (authored-genome) arm is used, and no authored genome
enters the natural-world arms; B0's prepared parents are measurements, not
mechanisms. A killed run is `incomplete`, never a biological negative; an
extinction is a finding.

| ID | Question | Method, instrument, sizes | Prediction / decision rule | Status | Result |
| --- | --- | --- | --- | --- | --- |
| B1 | Does a born-connected declaration raise executed and causal target-family reads in a natural world without costing persistence? | The prototype above (`proto:`, `mutation.born_connected`, default off, `policy-deviation` on). **Tested on the founder at unit level before any gate:** off, `Add` on a Graph node appends one reference and no edge and is `apply_to_node` draw for draw; on, `Add` on a Graph node appends one reference and exactly one edge, on an `ActionVote` sink, sourced from the appended index at a sub-value under the reference's width, with a weight in [−1, 1], compute nodes and the other node untouched; on, `Add` on a VM node appends one reference and leaves the backend unchanged; the flag reaches `Add` through the config and off is the default; the flag is absent from the serialized config when off and round-trips when on; over 4,096 seeds the born edge reaches all 27 vote sinks and all 8 sub-values of a ring with signs near even; a proptest over generated genomes holds the one-reference, at-most-one-edge, source and untouched-node invariants (exactly one edge on a full sink catalog). **Design as N1:** Canyon goal recipe (`experiments/worlds/canyon-country.json`, jq overlays), fresh seeds 1022 and 2022, 20,000 ticks, samples every 500, every arm on both seeds, through the census probe with checkpoints 10,000 and 20,000 (selected cohort of 20 parents, founder cohort, drift empty; the funnel block, the vote-delta reading and the births block at both; E6 at 20,000), 3 h awake kill per run, one job at a time. Arms: `default` (flag off; rerun in full on this binary, which carries T03.F12's gestation and T24's food repairs, so its numbers are new and N1's 7.09 × and 22.38 × are not expected), `born` (flag on), `default-mutoff` and `born-mutoff` (`per_unit_rate` 0: identical by construction, gate B0.3). **Readings:** window means over ticks 15,000 to 20,000 (as E8's reader, [`n12read.py`](evolvability-diagnosis-run-2026-10-followup/n12read.py)): population, genome size in founder units (97), births per creature-tick (spawned over summed population × 500 over the window's alive samples), extinction interval; the typed target-family funnel rows (`AreaFoodSummary:0`, `AreaFoodSummary:1`, `NeighborBarrierRing`: `family_declared`, `family_connected`, `family_executed`, `family_causal`, distinct parents, absent rows 0, `parents_evaluated` must be 20) at both checkpoints; the vote-delta block at both checkpoints (as [`n3read.py`](evolvability-diagnosis-run-2026-10-followup/n3read.py)): executed target channels, channels whose ablation changes a committed action queue (`scenes_actions_differ > 0`), channels whose ablation changes any vote (`delta_passes > 0`); the **agreement check** (finding 3): per typed family, the funnel's `family_causal` (distinct parents) against the number of distinct parents with any vote-delta channel of that family having `scenes_actions_differ > 0`, and per channel row, the funnel's channel-row `causal` (parents) against the parents whose vote-delta channel of that (family, channel) changes an action queue; both are the same ablation on the same panels, so a disagreement is an instrument failure that blocks the interpretation of the causal and vote-delta readings until explained (N3's 2 = 2 was one checkpoint's coincidence of one-channel families); dead-per-birth from the births block (`dead` over `births_total` and over `any_events.trials`, selected cohort, with `silent` and `changed` beside it), descriptive. The user's not-N1 additions: the vote-delta reading on at both checkpoints and the births block | **Prediction, with a direction or its absence for every reading (finding 14):** `born` persists on both seeds; its window-mean population **below** `default`'s but above 0.5 × (every `Add` birth on a Graph node *can now be* a behavior-changing birth, since its edge is live at birth, and E3 found a nonzero wrong-sign edge harmful; structural connection does not guarantee a vote or action change, finding 13); genome size **not below** `default`'s (one more unit per `Add`); births per creature-tick: **no directional prediction** (N1 found it uninformative); extinction: none predicted; per birth on the selected cohort at both checkpoints, `dead` and `changed` **up** and `silent` **down** under `born` (hypotheses, descriptive); funnel typed rows at both checkpoints, descriptively: `family_declared` **unchanged** (the `Add` draw is unchanged), `family_connected` **up** where the family is declared (a Graph-node declaration is connected at birth), `family_executed` **up**, `family_causal` at least one incidence at 20,000 where `default` has none; vote-delta: channels moving any vote **up**, and at least one target channel whose ablation changes an action queue; E6: **no prediction**. The validation reads 20,000; the 10,000 checkpoint is descriptive. **Validation rule, verbatim from the goal command:** validated if `born` persists on both seeds, window-mean population at least 0.5 × `default`'s, and at 20,000 on both seeds the executed plus causal count over the three typed target rows exceeds `default`'s with at least one causal incidence where `default` has none, with the vote-delta block showing at least one target channel whose ablation changes an action queue; not validated otherwise; the reading is a lead-selection screen on 20 parents and is stated as such. **Resolutions, predeclared:** (i) "on both seeds" governs every clause after it: the population floor, the executed-plus-causal excess (Σ over the three typed rows of `family_executed` + `family_causal` on the selected cohort at 20,000, `born` strictly greater than `default`), the causal incidence where `default` has none (some typed row with `family_causal` ≥ 1 under `born` and 0 under `default` on that seed) and the vote-delta condition (at least one executed target channel with `scenes_actions_differ > 0` under `born`), each on each seed; (ii) the funnel's `causal` and the vote-delta's action-queue change are one fact measured twice; the rule keeps both as the command states them, and the agreement check above must hold for the clause to be read; (iii) if `default` is extinct before 20,000 or has no living sample in the window on a seed, the population ratio and the 20,000 comparison are undefined on that seed and the row is **not validated** by this rule, with `born` persisting where `default` died recorded as a finding and read in the plan forward (T03.F12 closed with Canyon at 0.441 × its floor on the goal seed; seeds 1022 and 2022 are untested under gestation); (iv) `born` extinct on either seed: not validated, an extinction finding; (v) a 20-parent cohort is a lead-selection screen: one parent can carry the excess, numerators and direction are reported, and no population-wide increase is claimed from it. **Alternative explanations named now (finding 15):** the fixed vote-sink destination class, the added units and their supply and gestation costs, the pruning protection of connected references, backend composition of the cohort, generation count, panel exposure of the target channels (the follow-up's instrument finding: the battery never presents them), drift and shared ancestry within a 20-parent sample, and checkpoint sampling swings (N1's 14 → 1); a positive verdict is a screen of the combined mechanism. **Not validated otherwise.** A kill marks the run `incomplete` and it is rerun once at 6 h | **predeclared** | — |

## Advice

Every Fable advisor consultation rule 8 lists, with the disposition of each
point raised. Coverage so far: before anything was built (the approach to
the whole run, after the contract was read), Advice 1.

| # | Rule 8 point | Advice (summary) | Dispositions |
| --- | --- | --- | --- |
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

To be written after B1 is read (consultations before and after the draft;
Codex review (b) before closing). It will hold: (1) the B1 verdict by the
predeclared rule; (2) whether born-connected becomes the next roadmap
feature, with its track (T20 or T13) and natural analog, the three
sentences a feature would amend, and the draft row text for the user; (3)
what was refuted or weakened; (4) the next goal command or the next
diagnosis question; (5) cost.

## Cost

| Item | Wall |
| --- | --- |
| Predeclaration, Codex review (a), amendment A1 | session time; the review remote |
