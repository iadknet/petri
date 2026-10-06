# Evolvability diagnosis follow-up run: results (2026-10)

Date: 2026-10-06. Contract: the Plan forward of the
[diagnosis run note](evolvability-diagnosis-run-2026-10.md) (part 2's two
unvalidated leads, T03.F12 and T11.F28, and its follow-up diagnosis question),
under the [diagnosis plan](evolvability-diagnosis-plan-2026-10-05.md) (rules 1
to 9, as far as they apply to a follow-up) and the
[run 1 plan](evolvability-exploration-plan-2026-10-01.md) it inherits (the
natural-world check, matched controls, commit kinds, baseline identity, the
Codex review command, the close order), inside the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md).
Status: **in progress** (predeclaration, amended once after Codex review (a)).

## Question

The diagnosis run named two unvalidated leads and one open question. (1) Does
a genome-length-dependent reproductive time, counted correctly from the tick
after the birth, hold genome size and keep the population, in a natural world
beside the default? (2) Does a slower loss of silent structure, with the
janitor routes (`Prune`, type retargeting, `Swap`) acting on silent entries at
a per-unit decay rate instead of their operator rates, raise the declared share
of the target sensor families in the live world without costing persistence?
(3) In the live Confluence world at 20,000 and 50,000 ticks, how large is an
executed sensor read's total effect on the vote surface against the margins
the pass-end rule decides on: small against them (dilution), subadditive with
opposing signs across a family's channels (cancellation), or neither?

## Run setup

- **Roles.** Session on Fable 5.1 (`claude-fable-5-1`), the implementer; the
  Fable advisor is enabled and consulted through the `advisor` tool at every
  rule 8 point (the Advice table). No subagent spawned. Reviews: Codex
  `gpt-6.1-sol` `high`, read-only, through `codex exec` (`codex-cli 0.159.1`):
  (a) the predeclaration before any gate or row runs, with its findings
  disposed and the amendments committed first; (b) the note's readings
  before closing.
- **Launch revision.** `main` `d4bc4a98` (clean). Worktree
  `.claude/worktrees/evolvability-diagnosis-2` on branch
  `worktree-evolvability-diagnosis-2`, created with `EnterWorktree`; its head
  checked equal to `d4bc4a98` after creation. `npm ci` done in `frontend/`.
  Disk free at launch 98 GiB.
- **Host.** No `caffeinate` was running when the goal was pasted; the session
  started a session-wide `caffeinate -i -s` and wraps every heavy job in
  `caffeinate -i` through `diagrun.sh`. **The host was on battery (99 %,
  discharging) at launch**; `-s` is inert on battery, so the run orders its
  short jobs first (Advice 1), records the power state at every heavy launch,
  and treats a still-on-battery N3 start as a recorded risk, not a stop. The
  user was asked once to plug in. Telemetry off on every invocation. One
  heavy job at a time; `pgrep -fl 'v3-lab|v3-cli'` before each.
- **Pre-commit gate and the `source-map-js` advisory (user decision,
  2026-10-06).** The project pre-commit hook's OSV scan refused the
  predeclaration commit on GHSA-68fv-2mgg-jv7q (frontend dev dependency
  `source-map-js` 1.2.1; the 1.2.2 fix, released 2026-09-30 14:08 UTC, was
  still behind the frontend's seven-day release-age gate until 2026-10-07
  14:08 UTC), and the session's auto mode denied committing with the hook
  skipped, as the diagnosis run had done. The user then directed the run to
  fix the advisory. Under `SECURITY.md` ("an urgent security fix may bypass
  the age gate only for a specific reviewed package and version"), the run
  upgraded that one package with `npm update source-map-js
  --min-release-age=0` in `frontend/` (a 3-line `package-lock.json` change:
  1.2.1 to 1.2.2, both consumers deduplicated), left `.npmrc`'s
  `min-release-age=7` untouched (nothing to restore), and reran
  `scripts/dependency-audit`: "No issues found". The lock-file commit is its
  own commit on the branch, cited here as the reviewed exception's record,
  and lands on `main` with the docs at closing. Every later commit runs the
  full hook.
- **Evidence.** Raw output under `.bench-artifacts/lab/diagnosis2/` in the
  worktree, copied to the main checkout at closing; compact summaries beside
  this note under
  [`evolvability-diagnosis-run-2026-10-followup/`](evolvability-diagnosis-run-2026-10-followup/).
  Recipes are jq overlays of `experiments/worlds/canyon-country.json` stored
  under the evidence tree, never under `experiments/worlds/`.
- **Commit kinds.** `docs:` (this note and its folder), `lab:` (the E4
  `transition_rates` probe, taken as file state from
  `worktree-evolvability-diagnosis`), `instr:` (the census probe and the
  vote-delta reading, observation only), `proto:` (the two default-off
  mechanism prototypes), and the one dependency commit above. `instr:`,
  `lab:` and `proto:` stay on the branch; `docs:` and the dependency fix
  land on `main`.
- **Code taken from the previous branch** (file state, sources cited in the
  commit messages, no cherry-pick of mixed commits): the census probe
  `crates/v3-cli/src/bench/diagnosis_census.rs` (`ccb7d1fe`, `8cc1730a`), the
  reproduce-hold prototype (`7ea581e8`: `config/simulation.rs`,
  `creature/state.rs`, `simulation/tick.rs`), the E4 probe
  `crates/v3-lab/tests/diagnosis_probe.rs` (`09f78786`, `b2ed9dd9`). The
  `AddProjection` prototype and the one-edge arm sets are not taken.
- **Branch verification.** TDD for the prototypes; `cargo test -p v3-core
  --test viability` first (both prototypes touch births; it passed, 20 tests,
  before the first gate); `make check` on the branch before a prototype's
  results count, with the `v3-cli` recipe-digest pin test the one allowed
  failure (two new `RuntimeConfig` fields), as the run 1 plan allows;
  recorded under Run setup when run.
- **Founder fact found while testing N2.** The founder is not fully
  addressed: node 0's entry 4, `NeighborOccupiedRing`, has no consumer, so
  the N2 decay pass visits one silent entry on every founder birth. The
  predeclaration's claim that `slow-mutoff` is byte-identical to
  `default-mutoff` was wrong and is corrected in the N2 row.
- **Amendment A1 (after Codex review (a), before any gate or row ran).**
  Every finding is adopted (the dispositions table under Codex reviews): the
  N2 engine records the decay pass apart from the ordinary supply and the
  funnel's one-event proposals exclude it; gate G3 reads decay firing
  directly and tests equivalence with tolerances; N2's family contrasts are
  typed rows; N3's margins have explicit singleton, kindless, tied and
  guarded cases, control-flow divergence and action-queue agreement, and its
  family reading is an additive reconstruction with an opposing-signs test;
  the order waits for the review's dispositions; `default` and
  `default-mutoff` rerun in full on the new binary; the rate and the
  validation rules are described as what they are.

## Gates (predeclared, blocking)

| ID | What | Rule |
| --- | --- | --- |
| G1 | Instrument identity: the census probe (which builds its config through `resolve_config` on the recipe, as `v3-cli run --config` does) against E8's `v3-cli run` samples. A 2,000-tick smoke run on the Canyon goal recipe, seed 1022, default config, before any prototype arm; then the full `default` and `default-mutoff` reruns of N1 (both seeds, 20,000 ticks) compared at every sample | Every inline sample's `population`, `mean_genome_size`, `mean_generation` and `reproduction_actions_spawned_total` must equal E8's `default-s1022` (and, for the full runs, `default-s2022`, `default-mutoff-s1022`, `default-mutoff-s2022`) `tick_sample` fields at the same ticks (E8 ran the pinned launch-revision production binary). The smoke run blocks N1 and N2 until equal; a full-run inequality voids the N1 and N2 readings until explained and fixed. The instrument is verified on the new binary with both prototype flags off, not on a separate no-prototype build, as E2's census was (a recorded deviation from the strict baseline-identity reading) |
| G2 | Observation identity: N3's census is the E2 census plus a reading | N3's `input_use` blocks at 20,000 and 50,000 and its `e6` block at 20,000, each serialized once with sorted keys and compact separators, must hash-equal the same blocks of E2's Confluence census (`.bench-artifacts/lab/diagnosis/e2/census-confluence.ndjson`, same procedure). The run keeps E2's eight-thread rayon pool. N3's vote-delta reading is interpreted only after G2 passes, or after an observation-preserving correction is made and G2 then passes; a mismatch is never waived |
| G3 | Mechanism identity for N2, on the founder, through the E4 probe (`PETRI_E4_BIRTHS=1000000`, parents founder, founder+declared, founder+declared+zero-edge, no elites) on the N2 binary, once with the rate off and once at 0.005 (`PETRI_E4_DECAY=0.005`) | **Off path, equivalence with E4:** founder+declared's integrated `lose_declaration` and the connected parent's must each have a 95 % CP interval containing E4's point estimate (0.018643 and 0.0079606) and a point estimate within 5 % of it (E4's own interval at 10⁷ births is negligible beside a 10⁶ estimate's). **On path:** the decay pass's firing rate (`decay.firing` = losses applied over silent entries visited, one Bernoulli trial each) has a CP interval containing 0.005; the connected parent's integrated `lose_declaration` (its entry is addressed, so N2 must not touch it) has a CP interval containing the off path's point estimate and is within 10 % of it; and the ordinary supply is unchanged: the on path's `applied_events_per_birth` of the founder parent within 5 % of the off path's, `InputRefPrune` applied 0. The integrated final-child loss of founder+declared on the on path is reported beside the firing rate as a separate reading (it adds node deletion and connection-then-edit routes). Either failure marks N2 confounded: its world runs may run but do not count toward validation |

## Ledger

Every row is predeclared here before it runs; `incomplete` rows are kept. No
diagnostic (authored-genome) arm is used in this run, and no authored genome
enters a natural-world arm; G3's prepared parents and N3's named families are
measurements, not mechanisms. A killed run is `incomplete`, never a
biological negative; an extinction is a finding.

| ID | Question | Method, instrument, sizes | Prediction / decision rule | Status | Result |
| --- | --- | --- | --- | --- | --- |
| N1 | Does a correctly counted genome-length-dependent reproductive time hold genome size and keep the population? | `proto:` `energy.lifecycle.reproduce_hold_per_founder_size` (default off, policy-deviation when on), corrected: a committed `Reproduce` that spawns at tick *T* sets `held_until_tick = T + 1 + ceil(genome_size / 97)` and the parent commits no action while `sim.tick < held_until_tick`, that is on ticks *T*+1 … *T*+⌈size/97⌉ (the founder: exactly one tick; 98 to 194 units: two); cognition and its charges run as usual; death cancels the hold; supply, transfer and charges unchanged. Natural analog: copying a longer genome takes longer (Avida's replication time, Lenski et al. 2003, Methods); what variation still has to discover is unchanged (everything about the circuit). **Tested on the founder, at execution level:** with mutation off, a 97-unit spawner seen spawning at *T* is followed through *T*+1 (commits nothing, still marked) and *T*+2 (commits again); with mutation on, every held creature commits nothing while held, a 97-unit spawner is marked to *T*+2 and a 98-to-194-unit spawner to *T*+3; the off path leaves `held_until_tick` 0 everywhere. Design as E8: Canyon goal recipe, fresh seeds 1022 and 2022, 20,000 ticks, samples every 500, every arm on both seeds, through the census probe with checkpoints 10,000 and 20,000 (selected cohort of 20 parents, founder cohort, drift empty; the funnel block at both; E6 reading at 20,000), 3 h awake kill per run. Arms: `default` (rerun in full on this binary, flags off), `hold2` (the prototype on), `default-mutoff` and `hold2-mutoff` (`per_unit_rate` 0, the immediate-effect pair: fixed founder genomes, hold one tick per spawn). Readings over ticks 15,000 to 20,000 (window means, as E8's reader): mean genome size in founder units, mean population, births per creature-tick, extinction interval; the funnel's target-family rows at 20,000 descriptively; the mutation-off pair's extinction ticks and population trajectories (E8: both default pairs die in the trough before 5,500) | Prediction: `hold2` persists on both seeds with window-mean genome size below `default`'s (E8 `default`: 7.1 × and 22.4 ×), births per creature-tick lower, population lower than `default`'s but not collapsed. **Validated** (T03.F12 becomes the next feature) if on **both** seeds `hold2` reaches 20,000 ticks, its window-mean genome size is below `default`'s, and its window-mean population is at least 0.5 × `default`'s (E8's realized arm sat at 0.39 ×, the known failure); the pair reads the direct cost (an earlier extinction than `default-mutoff` on both seeds is a recorded cost, not a failure). **Not validated** otherwise. Validation means a reduced window-mean size over this horizon on two seeds of one world: a Canyon plausibility screen, not a size bound or a stabilization | predeclared | — |
| N2 | Does slower loss of silent structure raise the declared share of the target families in the live world without costing persistence? | `proto:` `mutation.silent_entry_decay_rate: Option<f64>` (default `None`, policy-deviation when set). A **silent entry** is an input reference no consumer on its node addresses (`prunable_indices`: no Graph input leaf and no VM `ReadInput` names it, live or not). On: in the ordinary engine an InputRef event whose drawn target entry is silent is **terminally skipped, never redrawn** (skip reason `SilentEntryDecaysOnly`, recorded as the selected attempt's skip): an otherwise applicable `Prune` attempt, whose every site is silent by construction, is skipped after its draw; an inapplicable `Prune` is still discarded and redrawn as today, with its `discarded` record; `Swap` and `RawFieldMutation` draw their entry exactly as today and skip when it is silent. The ordinary requested-event distribution, the addressed-entry hazards and the summary's ordinary counts, operator classes and `events` are therefore the default's (Advice 1 point 1; review (a) findings 3 and 10). A separate **decay pass**, run at every birth after the event loop (also on births that drew zero events), loses each silent entry independently with probability = the rate, by a route drawn uniformly among the routes that apply to that entry: prune (remove and renumber), swap (another member of its kind), retype (the food-type index, for typed food entries; the slot, for `UpstreamSlot`); entries are visited in descending index order per node so a removal never shifts a pending index; its visits and losses are recorded in the summary's separate `decay_eligible`, `decay_applied` and `decay_by_operator` fields, not as ordinary events, so E4's zero-event, single-event and operator classifications keep their meaning. The funnel's one-event retention proposals run with the rate unset (`input_use::observe`), so that frozen diagnostic path keeps its one-event meaning under every arm (finding 4). **Rate 0.005 per silent entry per birth**, numerically equal to the per-unit supply parameter `per_unit_rate` (which supplies a genome-wide event count, not a per-site edit rate): an experimental choice, not a uniquely justified biological rate; its analog is that an unused gene decays at the mutation rate per site, about 10⁻⁸ per generation, not at a janitor's 1 % (the natural analog of T11.F28; what variation still has to discover: the connection, its sign and its weight). Against E4's measured 1.86 × 10⁻² per birth this is 3.7 × slower, a decay *firing* rate; actual entry loss adds the other routes, and the lose-to-useful-connect ratio moving from about 3,400 toward about 900 is a prediction to measure, since it assumes the connection hazards unchanged. Gate G3 on the founder before any world run. Tests: the rate unset reproduces the default birth seed for seed; a fully addressed genome keeps the default redraws and nothing decays on non-growing births; the ordinary loop never loses an entry while it is silent and still swaps addressed entries; the decay pass at rate 1 loses every silent entry by two or more routes and touches nothing else; at rate 0.5 it fires at the rate and is deterministic. Design as N1: Canyon, seeds 1022 and 2022, 20,000 ticks, census probe with checkpoints 10,000 and 20,000; arms `slow` and `slow-mutoff` (`per_unit_rate` 0 with the rate on). **Correction to the predeclaration:** the founder carries one silent entry (node 0, entry 4, `NeighborOccupiedRing`), so `slow-mutoff` is **not** byte-identical to `default-mutoff`: the decay pass visits that entry on every founder birth and loses it in 0.5 % of them, a behaviorally neutral change (no consumer reads it) that shrinks the child by one unit or swaps or retypes the entry, so the pair reads the decay pass's direct cost on fixed founder lineages: expected to be the carry cost of one unit at most, and the pair's trajectories are compared descriptively. `default` and `default-mutoff` are N1's runs. Readings: N1's window readings plus, at 10,000 and 20,000 on the selected cohort, per **typed** target family row (`AreaFoodSummary:0`, `AreaFoodSummary:1`, `NeighborBarrierRing`, each a distinct-parent count; an absent row counts 0; `parents_evaluated` must be 20) `family_declared`, `family_connected`, `family_executed`, `family_causal` and the declared-but-unconnected count `family_declared − family_connected`, with the same typed row compared on both seeds (finding 5) | Prediction: at 20,000 the declared count of at least one typed target row is above `default`'s on both seeds; mean genome size is not below `default`'s (silent cargo accumulates); population not lower than 0.5 × `default`'s; `family_causal` unchanged at 0. **Validated** (T11.F28 becomes the next feature) if G3 passed, `slow` reaches 20,000 on both seeds, its window-mean population is at least 0.5 × `default`'s on both, and for at least one typed target row `family_declared` on the selected cohort at 20,000 is above `default`'s on both seeds. With 20 parents per cohort this is a lead-selection screen (a one-parent excess can pass it); numerators and the direction are reported, and no population-wide increase is claimed from it. **Not validated** otherwise. If N1 and N2 both validate, the diagnosis run's judgment order holds (size first) unless N1's population reading is the weaker | predeclared | — |
| N3 | How large is an executed read's total effect on the vote against the margins the pass-end rule decides on? | `instr:` the E2 census (`diagnosis_census_run`, Confluence seed 33, horizon 50,000, checkpoints 20,000 and 50,000; the 10,000 checkpoint is dropped, which consumes no simulation RNG and changes no later state; eight threads as E2) plus a **vote-delta reading** (`input_use::vote_delta`, `vote-delta-v1`) on the selected cohort at each checkpoint: for each sampled parent and each channel of `AreaFoodSummary(k)` and `NeighborBarrierRing` at the executed stage, the parent and its one-channel ablation (`ablated`, the funnel's own rewrite) run on the same panels (`neighborhood-v1` battery plus the T11.F26 extension) under a vote-recording execution mode (the production executor with every pass's final vote vector, effective votes, end reason, hop count and committed action kept). Passes are aligned by (scene execution, pass index); unmatched passes and executions whose pass counts differ are counted apart, as are **divergent** aligned passes (end reason or hop count differs: the delta there is an intervention effect on dispatch), and executions whose **complete committed action queue** differs (the execution-level counterpart of the funnel's `causal`, which it must agree with: differing executions > 0 if and only if the channel is causal). Every delta is the total effect of the intervention on that pass, never a read's isolated contribution (finding 8). Per (parent, channel) over the aligned passes with any nonzero per-sink delta: their count, max |Δ| and its sink, the L1 sum, the signed sum per sink, the most-moved sink (largest summed |Δ|) with its kind and the positive share of its deltas; the pass's largest-|Δ| sink classed **kindless** (`Decide`, `Terminate`), **singleton kind** (`Eat`: no second sink, no within-kind margin), **tied** (within-kind margin 0) or **margined**, with, for margined passes, the baseline's within-kind margin (best minus second-best vote of that kind) and the ratio |Δ| / margin (median, max); the heuristic count of passes where |Δ| reaches the within-kind margin (not a decisiveness test); the relative shift of the moved sink against its kind's baseline leader (largest magnitude); the pass-end rule's **decision margin** from the baseline's effective votes (winning kind minus the largest other kind, when a kind is positive; otherwise the distance to the zero threshold, counted as a no-commit pass) and the **guarded** passes (a kind positive, nothing committed: the termination or energy guard); and the channel's consumer count and summed |weight| (Graph leaves) and VM reads. Per (parent, family): the same for the **whole-family ablation**, plus the **additive reconstruction**: the ablation subadditivity ratio (family L1 over the sum of the channels' L1), the additive residual ratio (L1 of family Δ minus the sum of channel Δs, over the family's L1, across every aligned pass either side touched), and on the family's most-moved sink the channels whose summed signed Δ is positive and negative and the family's |summed Δ| over the largest channel's (finding 7). Written as a `vote_deltas` block beside `input_use` in the checkpoint row, with its own wall time. **6 h awake kill** (rule 3's doubled limit, predeclared because E2's Confluence census took 15,108 s); G2 before the reading | Prediction: at both checkpoints the median over margined passes of executed target-family channels of |Δ| / within-kind margin is below 0.1, the family ablations change no action queue at 50,000 (E2: causal 0 there) and at most the two causal food-summary incidences at 20,000 (E2), and the family reading is near-additive (residual ratio under 0.2). Readings are descriptive and narrow the diagnosis run's three hypotheses, with mixed and inconclusive allowed: **dilution** when per-channel and per-family ratio medians are both under 0.1, the subadditivity ratio is between 0.8 and 1.2 and the additive residual is under 0.2; **cancellation among channels** only when the additive residual is under 0.2 (the channel effects add), the family's most-moved sink has channels of both signs, and the family's |Δ| there is below the largest channel's; **subadditive interaction** when the residual is 0.2 or more (no cancellation claim); **neither** when ratio medians are 0.5 or more; anything else **mixed**. Loss of a read before selection (E4's hazards) stays outside this census. A world extinct before 50,000 completes with the checkpoints reached (amendment A5 of the diagnosis run); a kill marks the row `incomplete` | predeclared | — |

## Advice

Every Fable advisor consultation rule 8 lists, with the disposition of each
point raised. Coverage so far: before phase 0 (the whole run), Advice 1.

| # | Rule 8 point | Advice (summary) | Dispositions |
| --- | --- | --- | --- |
| 1 | Before anything was built: the approach to the three rows after the contract was read | Approach sound in shape; six fixes. (1) N2 as "Prune inapplicable, Swap/RawField draw only addressed entries" confounds slower silent loss with faster connected loss and more growth (the discard-and-redraw loop hands the draws to other operators); make it skip-not-redraw and verify on the founder with the E4 probe: silent loss within the rate's CP interval, connected loss within E4's. (2) N3: predeclare a 6 h limit (E2 took 15,108 s); define margin on the effective votes per pass, align passes by index; add a whole-family ablation, since one-channel ablation cannot see cancellation. (3) Two blocking identity gates: the probe's default arm against E8's samples before any proto arm; N3's `input_use` blocks hash-equal to E2's. (4) Battery: `caffeinate -s` is inert on battery; order short jobs first, notify the user once, record the state at every heavy launch. (5) EnterWorktree may base on `origin/main`: confirm the head is `d4bc4a98`; take probe and hold code as file state, not mixed cherry-picks. (6) Predeclare the validation thresholds (N1: size below default on both seeds, both persist, population at least a stated fraction; N2: declared share above default on both seeds, persistence not worse, the G3 rates), the rate choice and why, and that `slow-mutoff` reads nothing by construction but runs | All six adopted in the predeclaration: (1) N2 row and gate G3 (both reworked again at A1); (2) N3 row (6 h, effective-vote margins, whole-family ablation); (3) gates G1 and G2; (4) Run setup (host) and the order; (5) head confirmed `d4bc4a98`; file-state commits listed under Run setup; (6) the N1 and N2 decision rules, the 0.005 rate and its reason; the `slow-mutoff` sentence was later found wrong (the founder carries a silent entry) and corrected in the N2 row |

## Codex reviews

### Review (a): the predeclaration, before any gate or row ran

Job `.bench-artifacts/lab/diagnosis2/codex/review-a.out.md`, verdict
`not-ready`, 9 blocking and 6 advisory. Every finding is adopted as amendment
A1 (the rows and gates above carry the fixes); no confirmation round is run,
since no finding changes a row's question or the three rows' scope and every
fix is a predeclared wording, a gate rule or a code change tested on the
branch.

| # | Finding (short) | Severity | Disposition |
| --- | --- | --- | --- |
| 1 | G3 used E4's 10⁷-birth CP intervals as identity tolerances for 10⁶-birth estimates | blocking | Adopted: equivalence rules with the new estimate's CP interval containing E4's point estimate and a 5 % (off) or 10 % (connected, on) relative tolerance; the on-path firing rate read directly |
| 2 | `lost_declaration` is an integrated final-child reading, not the decay clock | blocking | Adopted: the engine records `decay_eligible` and `decay_applied`; the probe reports `decay.firing` (losses over Bernoulli trials) with its CP interval, and the integrated loss apart |
| 3 | The decay pass changed the event budget and E4's summary-based classifications | blocking | Adopted: decay recorded in separate summary fields, never as ordinary attempted or applied events; "event budget identical" replaced by "ordinary requested-event distribution unchanged", with G3 checking the founder's `applied_events_per_birth` |
| 4 | The funnel's one-event proposals would also run the decay pass | blocking | Adopted: `input_use::observe` builds its one-event config with the rate unset; stated in the N2 row |
| 5 | Food-family aggregation undefined (typed rows are not distinct parents) | blocking | Adopted: typed rows compared separately, absent rows 0, `parents_evaluated` 20, same row on both seeds |
| 6 | N3 margins undefined for singleton, kindless and tied sinks; the zero threshold and guards missing | blocking | Adopted: the four classes counted apart, ratios on margined passes only, decision margin with the zero threshold and no-commit distance, guarded passes counted, "reaches margin" labelled a heuristic, the relative shift against the kind leader added |
| 7 | The L1 ratio does not identify cancellation | blocking | Adopted: called subadditivity; cancellation requires an additive residual under 0.2 with opposing channel signs on the most-moved sink; mixed and inconclusive readings allowed; loss before selection stated as outside the census |
| 8 | Pass-index alignment reads intervention effects, not a read's contribution; funnel causality compares action queues | blocking | Adopted: unmatched and divergent passes counted; deltas described as total intervention effects; execution-level action-queue differences recorded and required to agree with `causal` |
| 9 | The order let gates and rows run before the review's findings were resolved | blocking | Adopted: the order now reads review completed → dispositions and amendments committed → builds and tests → gates → world runs; G2 is never waived |
| 10 | "Prune skipped whenever drawn" needs the applicability qualification | advisory | Adopted: wording and a test on a fully addressed genome (inapplicable `Prune` still discarded and redrawn) |
| 11 | N1's arithmetic is right; the test must follow a real spawn through the held and release ticks and check the 98-unit boundary at execution level | advisory | Adopted: both tests as described in the N1 row; validation described as a reduced window-mean size over this horizon |
| 12 | G1's four samples cannot authorize reusing E8's controls; N2 needs the funnel on the default | advisory | Adopted: `default` and `default-mutoff` rerun in full on the new binary; G1 is a smoke gate plus the full-run comparison; the no-prototype-build deviation recorded |
| 13 | The 0.005 rate is an experimental choice; firing is not loss; ≈900 assumes unchanged connection hazards; a one-parent excess can pass the rule | advisory | Adopted: the N2 row's wording |
| 14 | G2 is achievable; hash the extracted blocks with one frozen procedure; E2 had two causal incidences at 20,000 | advisory | Adopted: G2's procedure; N3's prediction distinguishes the checkpoints |
| 15 | Not-forcing checks pass; state no authored genome enters a natural-world arm, carry the analogs and remainders, keep kills apart from extinctions | advisory | Adopted: the Ledger preamble and the N1 and N2 rows |

## Order

Predeclaration committed → Codex review (a) completed and its findings
disposed, amendments and code committed → builds and tests (viability first)
→ G3 (seconds) → G1 smoke (about 3 min) → the four mutation-off pairs
(minutes each) → `default`, `hold2`, `slow` on both seeds (about 25 min each;
G1's full comparison on the `default` runs) → N1 and N2 readings with their
consultations → N3 (about 4.5 h) and G2 → N3's reading → plan forward
(consultations before and after) → Codex review (b) → close in the run 1
order, docs and the dependency fix only.

## Plan forward

Written after every row closes. Parts: verdict on each lead (validated or
not, with the rows), the next roadmap feature if either lead validates (its
track and natural analog, in the diagnosis run's part 2 form) or the
statement that neither does and what the follow-up question's reading says
should happen instead, the refuted or weakened items, the next goal command,
and the cost table.

## Cost

| Row | Wall |
| --- | --- |
| Predeclaration and amendment A1 | session time; no heavy job |
