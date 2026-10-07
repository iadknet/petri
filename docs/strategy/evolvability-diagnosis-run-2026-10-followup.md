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
Status: **complete** (N1, N2 and N3 completed; Codex reviews (a) and (b)
disposed; landed on `main` as docs plus the dependency fix).

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
  before the first gate). `make check` at `3778b30a` (log
  `.bench-artifacts/lab/diagnosis2/make-check-branch.log`): policy, quality,
  format, viability and every Rust test binary up to `rust-test-cli` passed,
  where it exited 2 on exactly the failure the run 1 contract allows, the
  `v3-cli` recipe-digest pin test
  `checked_in_goal_recipe_identities_are_unchanged_by_json_precision` (35 of
  36 passing; two new `RuntimeConfig` fields). A first `make check` at
  `de579cdf` had stopped at `rust-format-check` on the new test code, fixed
  by the rustfmt-only commit `0dd4c7d0`. The targets after the pin failure
  were run one by one (`remaining-checks-branch.log`): `rust-test-server`,
  `rust-test-lab`, `rust-test-telemetry`, `rust-test-doc`, `rust-clippy`,
  `frontend-check`, `dependency-audit` and `skill-check` all exited 0. The
  prototypes' results therefore count.
- **Landing (recorded on `main`).** At closing `main` had advanced from
  `d4bc4a98` to `4fcf4cd3` by two commits of the user's: `2ceabea4`, their
  own upgrade of `source-map-js` to 1.2.2 (the same lock-file change as the
  branch's `056df5a8`), and the T24 Simulation Throughput track. The picks
  applied on top of it without conflict; the `chore(deps)` pick therefore
  added only the note's first predeclaration text on `main` (its lock-file
  change was already there), and the docs part of `c4de89e1` was extracted
  by `cherry-pick -n` with the branch-only crate file dropped from the
  index. The landing proceeded rather than stopping, since the advance was
  docs plus the same dependency fix and no pick touched what it changed.
- **Branch and commits.** Branch `worktree-evolvability-diagnosis-2` from
  `d4bc4a98`: `chore(deps)` `056df5a8` (which also carries the note's first
  predeclaration text, staged before the refused commit and committed with
  the lock file); `proto:` `dcbee45e` (both
  prototypes); `instr:` `368e4cb8` (census probe options, vote-delta
  reading, the one-event proposals' decay exclusion); `lab:` `de579cdf` (the
  E4 probe with the decay reading); `proto+instr+lab:` `0dd4c7d0` (rustfmt
  only); `docs:` `d62065fa`, `3778b30a`, `603cdeed`, `0c0bd03e`, `6e3237dc`,
  `ff340d25`; `docs+instr:` `c4de89e1` (the N3 completion with the ignored
  battery-coverage test in `vote_delta.rs`, landed on `main` as its docs part
  only); then the review (b) dispositions and the closing commit.
- **Landing scope.** Only `docs:` commits (and the docs part of `c4de89e1`)
  plus the `chore(deps)` commit land on `main`; `proto:`, `instr:` and `lab:`
  stay on the branch, which is kept. T22 instrument candidates from this run
  (on the branch): the census probe's recipe, seed and name options; the
  `vote_delta` reading (`vote-delta-v1`) with its battery-coverage test; the
  E4 probe's decay-firing reading. The two prototypes are mechanism
  prototypes, not instruments.
- **Vote-delta smoke (Advice 3, point 1; light, beside the running N3
  census, on the already-built test binary).** The census probe with
  `PETRI_E2_VOTE_DELTAS=1` on a 200 × 200 Canyon recipe with 400 founders
  (`smoke/small.json`), seed 33: 300 ticks (checkpoint 300) and 5,000 ticks
  (checkpoints 2,500 and 5,000), exit 0, 1.2 s and 8.1 s. The recording
  executor ran the baseline panels of all 60 sampled parents without a
  panic; none of them carried an executed target channel, so the
  per-channel and per-family ablation branches were reached by no parent in
  the smoke and rest on the module's unit tests of `compare` and
  `kind_ranks`. A unit test on a built genome could not be added while N3
  holds the cargo build lock; the first real exercise is N3's 20,000
  checkpoint.
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
| G3 | Mechanism identity for N2, on the founder, through the E4 probe (`PETRI_E4_BIRTHS=1000000`, parents founder, founder+declared, founder+declared+zero-edge, no elites) on the N2 binary, once with the rate off and once at 0.005 (`PETRI_E4_DECAY=0.005`) | **Off path, equivalence with E4:** founder+declared's integrated `lose_declaration` and the connected parent's must each have a 95 % CP interval containing E4's point estimate (0.018643 and 0.0079606) and a point estimate within 5 % of it (E4's own interval at 10⁷ births is negligible beside a 10⁶ estimate's). **On path:** the decay pass's firing rate (`decay.firing` = losses applied over silent entries visited, one Bernoulli trial each) has a CP interval containing 0.005; the connected parent's integrated `lose_declaration` (its entry is addressed, so N2 must not touch it) has a CP interval containing the off path's point estimate and is within 10 % of it; and the ordinary **requested** supply is unchanged: the on path's `attempted_events_per_birth` of the founder parent within 5 % of the off path's, its applied events per birth *excluding* `InputRefPrune` within 5 % of the off path's (amendment A2, pre-data: the founder's own silent entry makes every otherwise-applicable `Prune` draw a terminal skip, so raw applied events drop by about the Prune share and are not the invariant), `InputRefPrune` applied 0. The integrated final-child loss of founder+declared on the on path is reported beside the firing rate as a separate reading (it adds node deletion and connection-then-edit routes). Either failure marks N2 confounded: its world runs may run but do not count toward validation |

**Gate results.** **G3 passed** (12 of 12 checks,
[`g3-founder-rates.json`](evolvability-diagnosis-run-2026-10-followup/g3-founder-rates.json),
reader [`g3read.py`](evolvability-diagnosis-run-2026-10-followup/g3read.py);
raw probe outputs `n12/g3/e4-off.json` sha256 `07b639db…` and `e4-on.json`
`d601743b…`, 10⁶ births per parent, 111 s and 40 s). Off path: the silent
declaration lost at 0.018679 per birth (CP 0.01841 to 0.01895; E4 0.018643),
the connected entry at 0.007911 (CP 0.00774 to 0.00809; E4 0.0079606). On
path: the decay pass visited 2.11 silent entries per founder+declared birth
(the declared entry and the founder's own `NeighborOccupiedRing`) and fired
on 0.0049998 of them (CP 0.004905 to 0.005096, 10,570 of 2,114,094); the
connected entry's loss 0.007883 (CP 0.00771 to 0.00806), inside the off
path's; the founder's requested supply identical (0.484315 attempted events
per birth on both paths) and its applied events excluding `InputRefPrune`
0.334972 against 0.335356 (raw applied 0.334972 against 0.356773: the Prune
share, now terminal skips); `InputRefPrune` applied 0 on every parent. The
integrated final-child loss of the silent declaration on the on path is
0.004942 per birth (CP 0.00481 to 0.00508), 3.8 × below the default's. The
decay pass's whole-child route mix on that parent (10,570 losses over every
silent entry visited: the watched declaration, the founder's own ring entry,
which has no swap or retype route, and entries created within the birth) is
`Prune` 66 %, retype 17 % and `Swap` 17 %; the summaries do not attribute
routes to the watched entry (review (b) finding 4). The useful signed
connection counts and the lose-to-useful-connect ratios of both paths are
stored in the gate summary. **G1 smoke passed** (`g1-default-s1022`, 2,000 ticks,
8 threads, 371 s including the release test build:
[`g1-smoke.json`](evolvability-diagnosis-run-2026-10-followup/g1-smoke.json),
reader [`g1check.py`](evolvability-diagnosis-run-2026-10-followup/g1check.py)):
all four samples equal E8's `default-s1022` on every compared field, 0
mismatches. **G1 full comparison passed**: the `default` reruns equal E8's
`default-s1022` and `default-s2022` at all 40 samples (ticks 500 to 20,000)
and the `default-mutoff` reruns equal E8's at all 10 and 8 samples to their
extinctions, 0 mismatches on every compared field (`n12/*/g1.json`): the probe on this
binary with both flags off is **sample-equivalent to E8's production binary
on the four compared fields** at every sample; the inherited contract's
stronger requirement, the instrument built on the launch revision without
prototypes, stays the recorded deviation above (review (b) finding 14).

## Ledger

Every row is predeclared here before it runs; `incomplete` rows are kept. No
diagnostic (authored-genome) arm is used in this run, and no authored genome
enters a natural-world arm; G3's prepared parents and N3's named families are
measurements, not mechanisms. A killed run is `incomplete`, never a
biological negative; an extinction is a finding.

| ID | Question | Method, instrument, sizes | Prediction / decision rule | Status | Result |
| --- | --- | --- | --- | --- | --- |
| N1 | Does a correctly counted genome-length-dependent reproductive time hold genome size and keep the population? | `proto:` `energy.lifecycle.reproduce_hold_per_founder_size` (default off, policy-deviation when on), corrected: a committed `Reproduce` that spawns at tick *T* sets `held_until_tick = T + 1 + ceil(genome_size / 97)` and the parent commits no action while `sim.tick < held_until_tick`, that is on ticks *T*+1 … *T*+⌈size/97⌉ (the founder: exactly one tick; 98 to 194 units: two); cognition and its charges run as usual; death cancels the hold; supply, transfer and charges unchanged. Natural analog: copying a longer genome takes longer (Avida's replication time, Lenski et al. 2003, Methods); what variation still has to discover is unchanged (everything about the circuit). **Tested on the founder, at execution level:** with mutation off, a 97-unit spawner seen spawning at *T* is followed through *T*+1 (commits nothing, still marked) and *T*+2 (commits again); with mutation on, every held creature commits nothing while held, a 97-unit spawner is marked to *T*+2 and a 98-to-194-unit spawner to *T*+3; the off path leaves `held_until_tick` 0 everywhere. Design as E8: Canyon goal recipe, fresh seeds 1022 and 2022, 20,000 ticks, samples every 500, every arm on both seeds, through the census probe with checkpoints 10,000 and 20,000 (selected cohort of 20 parents, founder cohort, drift empty; the funnel block at both; E6 reading at 20,000), 3 h awake kill per run. Arms: `default` (rerun in full on this binary, flags off), `hold2` (the prototype on), `default-mutoff` and `hold2-mutoff` (`per_unit_rate` 0, the immediate-effect pair: fixed founder genomes, hold one tick per spawn). Readings over ticks 15,000 to 20,000 (window means, as E8's reader): mean genome size in founder units, mean population, births per creature-tick, extinction interval; the funnel's target-family rows at 20,000 descriptively; the mutation-off pair's extinction ticks and population trajectories (E8: both default pairs die in the trough before 5,500) | Prediction: `hold2` persists on both seeds with window-mean genome size below `default`'s (E8 `default`: 7.1 × and 22.4 ×), births per creature-tick lower, population lower than `default`'s but not collapsed. **Validated** (T03.F12 becomes the next feature) if on **both** seeds `hold2` reaches 20,000 ticks, its window-mean genome size is below `default`'s, and its window-mean population is at least 0.5 × `default`'s (E8's realized arm sat at 0.39 ×, the known failure); the pair reads the direct cost (an earlier extinction than `default-mutoff` on both seeds is a recorded cost, not a failure). **Not validated** otherwise. Validation means a reduced window-mean size over this horizon on two seeds of one world: a Canyon plausibility screen, not a size bound or a stabilization | **completed** | Twelve runs through the census probe (summary [`n12-summary.json`](evolvability-diagnosis-run-2026-10-followup/n12-summary.json), reader [`n12read.py`](evolvability-diagnosis-run-2026-10-followup/n12read.py); the applied recipes' and binary's sha256 in `n12/recipes-and-binary.sha256`; every run complete, exit 0, 191 to 1,474 s awake; power recorded at every launch, on AC from `hold2-mutoff-s2022` on). **Window means over ticks 15,000 to 20,000 (final tick-20,000 values in brackets).** `default` seed 1022: population 4,664 [4,889], genome size 688 = **7.09 ×** the founder [821], births 0.01834 per creature-tick, generation 392; seed 2022: 1,208 [1,061], 2,171 = **22.38 ×** [2,308], 0.01853, generation 442 (E8's values exactly, by G1). `hold2` seed 1022: population 2,565 [2,931], **4.57 ×** [433], births **0.01977**, generation 382, spawns 1,029,336 against the default's 1,371,904; its size trajectory peaks at 531 at tick 10,000 and sits between 345 (tick 11,500) and 519 (15,000) after (433 at 20,000: not growing over the second half); seed 2022: population **2,648** [1,848], **9.14 ×** [1,081], births 0.01817, generation 466, spawns 860,601 against 835,989; its size still grows (339 at 10,000, 1,081 at 20,000) while the default's runs 832 to 2,308. Population ratios `hold2`/`default`: **0.55** (seed 1022) and **2.19** (seed 2022). **Immediate-effect pair** (mutation off, fixed founder genomes): `default-mutoff` extinct at tick 5,340 (seed 1022) and 4,144 (seed 2022), E8's intervals exactly; `hold2-mutoff` extinct at **4,197** (1,143 ticks earlier) and **6,055** (1,911 ticks later); the one-tick hold changes every spawn from tick 500 on (populations differ from the first sample), so the direct cost is not a consistent earlier extinction in the founder trough: one seed each way. **Funnel at 20,000** (selected cohort, 20 parents, typed rows declared/connected/executed/causal): `default` seed 1022 `NeighborBarrierRing` 5/3/3/0, `AreaFoodSummary:0` 3/1/0/0; `hold2` 4/1/1/0 and 6/2/0/0; `default` seed 2022 7/3/1/0 and 1/0/0/0 (at 10,000 the same world sampled 14/7/0/0 for the food summary: a 20-parent sample swings widely between checkpoints); `hold2` seed 2022 **3/2/2/2 and 6/1/3/1** (three causal parent-family incidences, on two to three distinct parents of 20, since the typed rows do not record their overlap; `slow` seed 2022 has one causal food-summary incidence, the default none on either seed). E6 gradient-informative share at 20,000: `default` 3.0 % and 13.0 %, `hold2` 18.4 % and 8.7 %. **Size per generation** (window-mean size over window-mean generation, Advice 3): `hold2` 1.16 and 1.90 units against the default's 1.76 and 4.91, **34 % and 61 % lower on both seeds**, so the size effect survives normalization by generations. **Prediction check.** Both seeds persist and size is below the default's on both (predicted); births per creature-tick are **not lower** on seed 1022 (0.01977 against 0.01834; lower on 2022, 0.01817 against 0.01853), and the population is **higher** on seed 2022 (2.19 ×, against the predicted "lower but not collapsed"): two contradictions, taken to the advisor (Advice 3). Readings offered as hypotheses, not findings: more births per creature-tick at half the population on seed 1022 is consistent with density compensation (more food per creature) despite about five held ticks per spawn; on seed 2022 `hold2` is itself declining at the end (3,102 → 1,950 → 1,848 living at 15,000, 17,500, 20,000) with its size still growing, so its higher population is a reading within this horizon and may be a delay of the default's trajectory rather than its avoidance. **Decision rule: validated** on both seeds (reaches 20,000; size 4.57 × against 7.09 × and 9.14 × against 22.38 ×; population 0.55 × and 2.19 ×, both at or above 0.5 ×). The seed-1022 population ratio sits at the rule's floor (0.55), and the two seeds disagree on the direction of the population effect; what validates is a reduced window-mean size on both seeds with persistence, as the row says, with the size trajectory flat over the second half on one seed and still growing on the other |
| N2 | Does slower loss of silent structure raise the declared share of the target families in the live world without costing persistence? | `proto:` `mutation.silent_entry_decay_rate: Option<f64>` (default `None`, policy-deviation when set). A **silent entry** is an input reference no consumer on its node addresses (`prunable_indices`: no Graph input leaf and no VM `ReadInput` names it, live or not). On: in the ordinary engine an InputRef event whose drawn target entry is silent is **terminally skipped, never redrawn** (skip reason `SilentEntryDecaysOnly`, recorded as the selected attempt's skip): an otherwise applicable `Prune` attempt, whose every site is silent by construction, is skipped after its draw; an inapplicable `Prune` is still discarded and redrawn as today, with its `discarded` record; `Swap` and `RawFieldMutation` draw their entry exactly as today and skip when it is silent. The requested-event-count law and the ordinary operators' definitions are therefore the default's; a terminal skip changes the birth's applied and skip records and its RNG consumption, so later events of the same birth can differ, and the addressed-entry hazards are equivalent to the default's within gate G3's declared founder tolerances, not identical (Advice 1 point 1; review (a) findings 3 and 10; review (b) finding 3). A separate **decay pass**, run at every birth after the event loop (also on births that drew zero events), loses each silent entry independently with probability = the rate, by a route drawn uniformly among the routes that apply to that entry: prune (remove and renumber), swap (another member of its kind), retype (the food-type index, for typed food entries; the slot, for `UpstreamSlot`); entries are visited in descending index order per node so a removal never shifts a pending index; its visits and losses are recorded in the summary's separate `decay_eligible`, `decay_applied` and `decay_by_operator` fields, not as ordinary events, so E4's zero-event, single-event and operator classifications keep their meaning. The funnel's one-event retention proposals run with the rate unset (`input_use::observe`), so that frozen diagnostic path keeps its one-event meaning under every arm (finding 4). **Rate 0.005 per silent entry per birth**, numerically equal to the per-unit supply parameter `per_unit_rate` (which supplies a genome-wide event count, not a per-site edit rate): an experimental choice, not a uniquely justified biological rate; its analog is that an unused gene decays at the mutation rate per site, about 10⁻⁸ per generation, not at a janitor's 1 % (the natural analog of T11.F28; what variation still has to discover: the connection, its sign and its weight). Against E4's measured 1.86 × 10⁻² per birth this is 3.7 × slower, a decay *firing* rate; actual entry loss adds the other routes, and the lose-to-useful-connect ratio moving from about 3,400 toward about 900 is a prediction to measure, since it assumes the connection hazards unchanged. Gate G3 on the founder before any world run. Tests: the rate unset reproduces the default birth seed for seed; a fully addressed genome keeps the default redraws and nothing decays on non-growing births; the ordinary loop never loses an entry while it is silent and still swaps addressed entries; the decay pass at rate 1 loses every silent entry by two or more routes and touches nothing else; at rate 0.5 it fires at the rate and is deterministic. Design as N1: Canyon, seeds 1022 and 2022, 20,000 ticks, census probe with checkpoints 10,000 and 20,000; arms `slow` and `slow-mutoff` (`per_unit_rate` 0 with the rate on). **Correction to the predeclaration:** the founder carries one silent entry (node 0, entry 4, `NeighborOccupiedRing`), so `slow-mutoff` is **not** byte-identical to `default-mutoff`: the decay pass visits that entry on every founder birth and loses it in 0.5 % of them, a behaviorally neutral change (no consumer reads it) that shrinks the child by one unit or swaps or retypes the entry, so the pair reads the decay pass's direct cost on fixed founder lineages: expected to be the carry cost of one unit at most, and the pair's trajectories are compared descriptively. `default` and `default-mutoff` are N1's runs. Readings: N1's window readings plus, at 10,000 and 20,000 on the selected cohort, per **typed** target family row (`AreaFoodSummary:0`, `AreaFoodSummary:1`, `NeighborBarrierRing`, each a distinct-parent count; an absent row counts 0; `parents_evaluated` must be 20) `family_declared`, `family_connected`, `family_executed`, `family_causal` and the declared-but-unconnected count `family_declared − family_connected`, with the same typed row compared on both seeds (finding 5) | Prediction: at 20,000 the declared count of at least one typed target row is above `default`'s on both seeds; mean genome size is not below `default`'s (silent cargo accumulates); population not lower than 0.5 × `default`'s; `family_causal` unchanged at 0. **Validated** (T11.F28 becomes the next feature) if G3 passed, `slow` reaches 20,000 on both seeds, its window-mean population is at least 0.5 × `default`'s on both, and for at least one typed target row `family_declared` on the selected cohort at 20,000 is above `default`'s on both seeds. With 20 parents per cohort this is a lead-selection screen (a one-parent excess can pass it); numerators and the direction are reported, and no population-wide increase is claimed from it. **Not validated** otherwise. If N1 and N2 both validate, the diagnosis run's judgment order holds (size first) unless N1's population reading is the weaker | **completed** | Same runs and summary as N1; G3 passed before any world ran. **Window means** (final values in brackets): `slow` seed 1022: population 3,221 [2,866], genome size 578 = **5.96 ×** [681], births **0.01690**, generation 334, spawns 1,047,981; seed 2022: population 1,777 [1,613], 866 = **8.93 ×** [1,215], births 0.02215, generation 358, spawns 688,467, after a trough that reaches **52 living at tick 8,000** (125 at 5,000, 205 at 7,500; the default's lowest sample on that seed is 683, at 19,000). Population ratios `slow`/`default` **0.69** and **1.47**. **Pair:** `slow-mutoff` reproduces `default-mutoff`'s population and cumulative spawns at every sample on both seeds and goes extinct at the same ticks (5,340 and 4,144), with mean genome size 96.3 to 97.0 instead of 97 (the founder's silent ring entry decayed in a fraction of births): the decay pass's direct effect on fixed founder lineages is behaviorally nil, as predicted after the correction. **Funnel at 20,000** (selected cohort, 20 parents, typed rows declared/connected/executed/causal): `AreaFoodSummary:0` `slow` **9/3/3/0** against `default` 3/1/0/0 (seed 1022) and **8/4/3/1** against 1/0/0/0 (seed 2022); `NeighborBarrierRing` 7/2/1/0 against 5/3/3/0 and 4/0/0/0 against 7/3/1/0; `AreaFoodSummary:1` 0 everywhere. Declared-but-unconnected `AreaFoodSummary:0`: 6 against 2 and 4 against 1. At 10,000: `AreaFoodSummary:0` 7/0/0/0 against 1/0/0/0 (seed 1022) and 0/0/0/0 against 14/7/0/0 (seed 2022). E6 gradient-informative at 20,000: 22.8 % and 26.6 % against the default's 3.0 % and 13.0 %. **Prediction check.** Declared count above the default's on both seeds for one typed row (`AreaFoodSummary:0`): predicted and met; population not below 0.5 ×: met; `family_causal` unchanged at 0: **not met** on seed 2022 (one causal food-summary parent of 20); mean genome size **below** the default's on both seeds (5.96 × against 7.09 ×, 8.93 × against 22.38 ×), against the prediction that silent cargo accumulates: a contradiction, taken to the advisor (Advice 3) with the observation that generations are fewer (334 and 358 against 392 and 442) and that size per generation is near the default's on seed 1022 (1.73 against 1.76 units) and below it on 2022 (2.42 against 4.91): the smaller genomes are fewer generations on one seed and unexplained on the other. **The predeclared ratio prediction (Advice 4):** on G3's on path the silent declaration's integrated loss is 0.004942 per birth and the useful signed connection 7 × 10⁻⁶ (7 events of 10⁶, CP 2.8 × 10⁻⁶ to 1.4 × 10⁻⁵; the off path drew the same 7, as the connection draws are untouched), a lose-to-useful-connect ratio of **706 (bounds 333 to 1,806)** against the off path's 2,668 (1,277 to 6,732) at the same 10⁶ births; on E4's 10⁷-birth useful-connect denominator (5.5 × 10⁻⁶) the on-path loss gives 899, as predicted ("toward about 900"). **Decision rule: validated by the predeclared screen** (G3 passed; both seeds reach 20,000; population 0.69 × and 1.47 ×; `AreaFoodSummary:0` declared 9 > 3 and 8 > 1 at 20,000), **and not shown specific to slower loss on this evidence**; two things the rule did not read: the 10,000 checkpoint reverses the direction on seed 2022 (`slow` 0 against `default` 14 declared), so the rule is met at one checkpoint of two; and `slow` seed 2022 fell to 52 living at tick 8,000, a near-extinction the "reaches 20,000" criterion does not catch. **Caveat the rule did not anticipate:** `hold2`, which changes no loss rate, also sampled more `AreaFoodSummary:0` declarations than the default at 20,000 (6 > 3 and 6 > 1), and the default's own seed-2022 cohort fell from 14 declared at 10,000 to 1 at 20,000; the declared-count excess is therefore not shown to be specific to slower loss on this evidence, and with 20 parents per cohort the reading is the lead-selection screen the row calls it, not a supported population-wide increase. **The ordering tie-break** ("unless N1's population reading is the weaker") cannot be applied as written: each lead has one population ratio below 1 and one above (N1 0.55 and 2.19, N2 0.69 and 1.47), so "weaker" is indeterminate on this evidence and the order in the plan forward is the run's judgment, decided on mechanism specificity (N1's size effect is the mechanism's direct quantity and holds per generation on both seeds; N2's declared-count excess is matched by the `hold2` control and reverses at 10,000) |
| N3 | How large is an executed read's total effect on the vote against the margins the pass-end rule decides on? | `instr:` the E2 census (`diagnosis_census_run`, Confluence seed 33, horizon 50,000, checkpoints 20,000 and 50,000; the 10,000 checkpoint is dropped, which consumes no simulation RNG and changes no later state; eight threads as E2) plus a **vote-delta reading** (`input_use::vote_delta`, `vote-delta-v1`) on the selected cohort at each checkpoint: for each sampled parent and each channel of `AreaFoodSummary(k)` and `NeighborBarrierRing` at the executed stage, the parent and its one-channel ablation (`ablated`, the funnel's own rewrite) run on the same panels (`neighborhood-v1` battery plus the T11.F26 extension) under a vote-recording execution mode (the production executor with every pass's final vote vector, effective votes, end reason, hop count and committed action kept). Passes are aligned by (scene execution, pass index); unmatched passes and executions whose pass counts differ are counted apart, as are **divergent** aligned passes (end reason or hop count differs: the delta there is an intervention effect on dispatch), and executions whose **complete committed action queue** differs (the execution-level counterpart of the funnel's `causal`, which it must agree with: differing executions > 0 if and only if the channel is causal). Every delta is the total effect of the intervention on that pass, never a read's isolated contribution (finding 8). Per (parent, channel) over the aligned passes with any nonzero per-sink delta: their count, max |Δ| and its sink, the L1 sum, the signed sum per sink, the most-moved sink (largest summed |Δ|) with its kind and the positive share of its deltas; the pass's largest-|Δ| sink classed **kindless** (`Decide`, `Terminate`), **singleton kind** (`Eat`: no second sink, no within-kind margin), **tied** (within-kind margin 0) or **margined**, with, for margined passes, the baseline's within-kind margin (best minus second-best vote of that kind) and the ratio |Δ| / margin (median, max); the heuristic count of passes where |Δ| reaches the within-kind margin (not a decisiveness test); the relative shift of the moved sink against its kind's baseline leader (largest magnitude); the pass-end rule's **decision margin** from the baseline's effective votes (winning kind minus the largest other kind, when a kind is positive; otherwise the distance to the zero threshold, counted as a no-commit pass) and the **guarded** passes (a kind positive, nothing committed: the termination or energy guard); and the channel's consumer count and summed |weight| (Graph leaves) and VM reads. Per (parent, family): the same for the **whole-family ablation**, plus the **additive reconstruction**: the ablation subadditivity ratio (family L1 over the sum of the channels' L1), the additive residual ratio (L1 of family Δ minus the sum of channel Δs, over the family's L1, across every aligned pass either side touched), and on the family's most-moved sink the channels whose summed signed Δ is positive and negative and the family's |summed Δ| over the largest channel's (finding 7). Written as a `vote_deltas` block beside `input_use` in the checkpoint row, with its own wall time. **6 h awake kill** (rule 3's doubled limit, predeclared because E2's Confluence census took 15,108 s); G2 before the reading | Prediction: at both checkpoints the median over margined passes of executed target-family channels of |Δ| / within-kind margin is below 0.1, the family ablations change no action queue at 50,000 (E2: causal 0 there) and at most the two causal food-summary incidences at 20,000 (E2), and the family reading is near-additive (residual ratio under 0.2). Readings are descriptive and narrow the diagnosis run's three hypotheses, with mixed and inconclusive allowed: **dilution** when per-channel and per-family ratio medians are both under 0.1, the subadditivity ratio is between 0.8 and 1.2 and the additive residual is under 0.2; **cancellation among channels** only when the additive residual is under 0.2 (the channel effects add), the family's most-moved sink has channels of both signs, and the family's |Δ| there is below the largest channel's; **subadditive interaction** when the residual is 0.2 or more (no cancellation claim); **neither** when ratio medians are 0.5 or more; anything else **mixed**. Loss of a read before selection (E4's hazards) stays outside this census. A world extinct before 50,000 completes with the checkpoints reached (amendment A5 of the diagnosis run); a kill marks the row `incomplete`. **Two bad outcomes, predeclared before the first checkpoint landed (Advice 4):** if the vote-delta block panics at 20,000 (its ablation branch's first real exercise), the census dies there and the row is `incomplete`; the fix is made and one rerun is launched under rule 3 (6 h, the same seed and checkpoints), and only its readings count; if the 6 h kill fires before 50,000, the 20,000 reading stands as recorded, the row is `incomplete`, and no rerun is made (the limit was already the doubled one) | **completed** (50,000 reached; **G2 passed** at both checkpoints) | Census complete: 50,000 ticks, 15,547 s of simulation, 7.2 s of censuses, 8 threads, exit 0, 15,611 s awake inside the 6 h limit (elapsed 17,392 s: the host slept about 30 min during the run; the awake clock excluded it and the deterministic simulation was unaffected, G2 proving it). Summary [`n3-vote-margins.json`](evolvability-diagnosis-run-2026-10-followup/n3-vote-margins.json) with per-channel detail in [`n3-vote-margins-channels.json`](evolvability-diagnosis-run-2026-10-followup/n3-vote-margins-channels.json), reader [`n3read.py`](evolvability-diagnosis-run-2026-10-followup/n3read.py); gate record [`g2.json`](evolvability-diagnosis-run-2026-10-followup/g2.json): the `input_use` blocks at 20,000 and 50,000 and the `e6` block at 20,000 hash-equal E2's, so the trajectory is E2's (population 3,769 and 1,545, mean genome size 1,914 and 11,485 at the two checkpoints) and the funnel rows are E2's (at 50,000: `NeighborBarrierRing` 11/9/9/0, `AreaFoodSummary:0` 8/7/7/0, `:1` 4/4/4/0; at 20,000: 4/2/1/0, 7/6/3/1, 6/6/5/1). The vote-delta blocks took 0.11 s and 0.51 s. Every executed target channel is a food summary's channel 0 (`total_ratio`, the visible density), 1 (`gradient_x`), 4 (`nearest_dy`), 5 (`nearest_dist`) or 6 (`max_value`), or a barrier-ring direction 0, 1, 2 or 4: no `nearest_dx` read is executed in either sample. **At 20,000** (8 of 20 parents, 9 channels; 8 read by VM nodes only, 1 by one Graph edge): the funnel's two causal incidences (`AreaFoodSummary:0` and `:1` channel 0, one parent each, 6 and 3 VM reads) are exactly the two channels whose ablation changes complete action queues (8 and 4 of 264 scene executions; 40 and 20 passes) **with zero vote delta on every aligned pass and zero divergent passes** (equal hop counts and pass-end reasons; agreement with `causal`: 2 = 2): with the votes and those dispatch measures unchanged, the executor's only path to a different commit is `decode_commit` reading the action-parameter surface (`action_params`, written by VM nodes): a grounded inference, the census not having recorded which field differed. Five channels move votes with no action change: four (`AreaFoodSummary:0` and `:1` channel 0, 6 to 10 VM reads each, two pairs of identical readings) raise `StealEnergy(3)` under ablation (positive share 1.0: the ablation *increases* that sink, so the baseline read lowered it) by 0.351 to 0.377 in 980 to 1,530 of 2,640 aligned passes, ratio medians 0.48 to 0.82 against a within-kind margin of 0.074 on the margined passes (350 to 550 each; tied passes 630 to 980), reaching that margin in 780 to 1,230 passes; the baseline decision margin on their delta passes has median 0.5 and minimum 0.10, so the deltas exceed the decision margin on some passes without any commit changing, and the moved sink's kind was not the committing one: the stored statistics do not pair each delta with its pass's winning kind and gap, so **incumbent masking is a hypothesis here, not a finding**; the fifth (`AreaFoodSummary:1` channel 0, 41 VM reads) moves `Eat` and `Move` sinks by at most 0.00087 (ratio median 4.5 × 10⁻⁶; reaching a margin in 8 passes, singleton-kind passes 430). Divergent passes on the five: 1,160, 300, 730, 0 and 980 (3,170 of 5,975 delta passes in all; unmatched 0, guarded 0). Two channels neither move a vote nor change an action. Families: subadditivity 1.0 and additive residual 0.0 for the 5 of 9 families where they are defined, no opposing signs (every family has one channel). Classification by the predeclared labels on the reader's statistic: **mixed** (per-channel median of medians 0.48, under the 0.5 "neither" line, no action change, no cancellation). **At 50,000** (16 of 20 parents carry executed target channels; 35 channels on 20 parent-family pairs; 22 read by Graph edges only at weights of 0.09 to 1.09, 10 by VM only, 3 by both): **34 of 35 channels move no vote in any of about 1,000 aligned passes and change no action** (zero delta exactly, not small), among them Graph edges at 0.99 from `gradient_x`, 1.0 to 1.06 from `nearest_dy` and 0.09 from `nearest_dist` or `max_value` in recurring channel sets across five `AreaFoodSummary:0` and two `:1` parents (the same weights and sets: shared ancestry is a hypothesis, the census carrying no parent identity); the one exception, a `NeighborBarrierRing` direction 0 read with one Graph edge (0.93) and 11 VM reads, raises `Move(6)` by 0.372 on each of its 28 delta passes (2.7 % of 1,056), every one at or past the within-kind margin (median 0.147, minimum 0.014; ratio median 2.5, max 27), with the baseline decision margin median 0.435 and minimum 0.091 on the 21 committing passes and 7 passes with no committing kind, zero divergent passes and no committed action changed. The 18 divergent passes at 50,000 all belong to a zero-delta `AreaFoodSummary:1` channel (18 VM reads). Funnel causal 0 = action-queue changes 0. Subadditivity and residual are defined for 1 of 20 families (1.0 and 0.0). Classification on the reader's statistic: **neither** (ratio median 2.5 on the one moving channel). **Aggregation deviation (review (b) finding 8):** the row predicted "the median over margined passes of |Δ| / within-kind margin"; `n3read.py` reports the median of the per-channel medians, and the per-pass ratios were not stored, so the predicted statistic cannot be reconstructed and that prediction is **unevaluated**; the classifications above are reader-statistic classifications, kept separate per checkpoint (mixed at 20,000, neither at 50,000). **What a zero delta can mean (Advice 5).** The ablation forces every read of the channel to 0.0, so a channel whose value was already 0.0 in every panel scene where its consumer ran gives exactly the same zero delta as a consumer whose output never reaches a vote. The battery coverage reading ([`n3-battery-coverage.json`](evolvability-diagnosis-run-2026-10-followup/n3-battery-coverage.json), the ignored test `battery_nonzero_coverage_of_target_channels` in `input_use::vote_delta`) settles the first half: **the `neighborhood-v1` battery presents a zero value for every channel of `AreaFoodSummary:0`, `AreaFoodSummary:1` and `NeighborBarrierRing` in all 80 of its scenarios** (48 snapshots and 8 sequences of 4; its generator fills the founder's inputs and leaves the extended perception at zero), so the 80 original executions of each parent can never register a target-family ablation, and every delta and every action change above comes from the 184 extension executions: the 32 recorded contexts of the world at that checkpoint (each creature's complete production snapshot, extended perception included), their 4 recorded sequences of 32 ticks, and 24 authored contexts, each of which copies a battery snapshot and sets one of eight channel groups (three contexts per group; which groups cover which target family was not read here). Exposure **when a given consumer executes** was not recorded per parent, channel or checkpoint, and the recorded contexts' values are not stored, so **exposure against downstream silence stays unresolved for every channel**: N3's own E6 at 20,000 says food type 0 was visible to 94.2 % of living creatures (so most recorded contexts at that checkpoint presented a nonzero `AreaFoodSummary:0` `total_ratio` and `max_value`), but a nonzero nearest vector in 63 % of creatures establishes neither a nonzero `nearest_dy` nor `gradient_x` for a given context, E6 says nothing about 50,000, and `AreaFoodSummary:1` and `NeighborBarrierRing` exposure is unknown (E1 found the barrier edge exposed to 5.5 to 6.0 % of sampled creatures). The one channel that moved a vote at 50,000 is a barrier-ring read, which shows some contexts did present barriers. **Instrument finding** (outside the row's question, for T22 and T20): the funnel's `causal` and retention stages for these two families, in this census, in E2 and in T20.F01's goal reading, rest on the recorded and authored contexts alone, since the battery never exposes them. **Reading.** With both interpretations kept apart: **dilution is not what the data show** (the deltas are exactly zero, not small, on 34 of 35 channels at 50,000, and the four vote-moving food-summary channels at 20,000 raise a sink by 0.35 to 0.38, comparable to or above its within-kind margin, not by a small fraction of it); **cancellation among a family's channels is not seen** where the reading is defined (5 of 9 families at 20,000 and 1 of 20 at 50,000: no opposing signed contributions, additive residual 0; most families have one executed channel). What the census shows instead: (i) most executed target reads are ones **whose ablation changes nothing on these panels**, which is either a consumer whose output never reaches a vote or a channel the panels never presented nonzero where that consumer ran, undecided here per channel (the funnel does not distinguish either from `executed`; the panels cannot say whether other scenes would differ, and the census did not record the consumers' downstream paths); (ii) the reads that do move a vote raise a sink of a kind other than the committing one, by amounts up to and past that kind's within-kind margin and, on some passes, past the baseline decision margin, without any commit changing: consistent with run 1's H4 (incumbent masking) at the level of kinds, which paired effective votes, winning kinds and moved-sink gaps per pass would be needed to establish; (iii) the two reads that are causal at 20,000 do so with unchanged votes, hop counts and pass-end reasons, leaving the **action-parameter surface** as the path (a grounded inference). Ablation also changes dispatch on many passes (3,170 at 20,000; 18 at 50,000, on a zero-delta channel), so every delta is an intervention effect. Loss of a read before selection stays outside this census. The classification line's "mixed" and "neither" are reader-statistic labels; the observations above are the row's finding |

- **Kills and reruns.** None: no run hit its awake limit and no rerun was
  made in this follow-up.
- **G2 chronology (review (b) finding 15).** The 20,000 reading was
  interpreted and committed (`ff340d25`) when only the 20,000 checks of G2
  had passed (`n3/g2-partial.json` records `pass: false` with 50,000
  absent); after the census completed, G2 passed in full and the final
  reading above was written and revalidated against it.
- **Host sleep during N3 (deviation from the plan's "host stays awake"
  rule).** The host slept for about 30 min during the N3 census (elapsed
  17,392 s against 15,611 s awake); the run resumed and completed inside its
  awake limit, and G2's hash equality of the `input_use` and `e6` blocks with
  E2's shows the deterministic simulation was unaffected.
- **Amendment A3 (post-data reader change, stated as such).** After the
  20,000 checkpoint landed, `n3read.py` gained two tallies the predeclaration
  lacked (channels whose ablation changes an action with zero vote delta;
  channels with a vote delta and no action change, and the consumer-kind
  split) and a classification label for the first case when no channel moves
  a vote; the predeclared labels (dilution, cancellation, subadditive
  interaction, neither, mixed) and their thresholds are unchanged, and both
  checkpoints are classified by them in the N3 cell.
- **Amendment A2 (after Advice 2, before gate G3 ran; pre-data).** G3's
  supply invariant reads the requested supply (`attempted_events_per_birth`)
  and the applied events excluding `InputRefPrune`, not raw applied events
  (gate G3). The G3 reader is written against the probe's raw keys.

## Advice

Every Fable advisor consultation rule 8 lists, with the disposition of each
point raised. Coverage so far: before phase 0 (the whole run), Advice 1;
before the N1/N2 phase's approach (gates and the world chain), Advice 2;
after the N1 and N2 readings were drafted and before they were recorded,
with their three prediction contradictions, Advice 3; before the plan
forward was drafted, Advice 4; after the N3 reading was drafted and before
it was recorded, with its contradiction, and after the plan forward was
drafted, Advice 5; before closing, with N2's fourth contradicted
prediction put explicitly, Advice 6.

| # | Rule 8 point | Advice (summary) | Dispositions |
| --- | --- | --- | --- |
| 6 | Before closing (rule 8), after the review (b) dispositions were committed; also the consultation on N2's fourth contradicted prediction (`family_causal` staying 0: `slow` seed 2022 sampled one causal `AreaFoodSummary:0` incidence at 20,000), which Advice 3's brief had not named | (1) Record the fourth contradiction as a contradicted prediction, not a finding about slower loss: one incidence on one seed at one checkpoint, where the `hold2` control sampled three and the default none, and the validation screen never used `causal`; it changes nothing in parts 1 and 2. (2) Landing order: the `chore(deps)` commit first (the docs-part extraction of `c4de89e1` runs the hook, which needs the fixed lock file on `main`), then the pure `docs:` commits in order, the extracted docs part, the closing commit; verify purity by path first; the rustfmt commit stays. (3) Closing commit on the branch: Status, this row, the turn count, nothing else; `make check-docs`. (4) Copy the evidence tree and compare sizes before removal; kill a lingering codex broker; expect `.DS_Store`. (5) Confirm `main` at `d4bc4a98` and clean before the first pick; after: `make check-docs`, worktree list, branch list, status. (6) Memory and the final report with the three user items | All adopted: (1) this row; (2) to (6) the close |
| 5 | After the N3 reading and part 3 were drafted, before they were recorded; the contradiction consultation for N3 (ratio medians 0.48 and 2.5 against the predicted "under 0.1"); the post-draft consultation for the plan forward | One blocking point: "inert on the vote surface" is one of two readings the instrument cannot separate, since the ablation forces reads to 0.0 and a channel whose baseline value was already 0.0 wherever its consumer ran gives exactly the same zero delta; the cell and part 3 stated only the downstream reading. Fix: count the battery scenarios where each target channel is nonzero (a test inside the module), say what N3's `e6` implies for the recorded contexts, rewrite the finding with both interpretations, recast the T22 candidate as two readings. (2) State the parameter-surface mechanism (`decode_commit` reading `action_params`, the only path to a different commit with identical votes and dispatch) as a grounded inference; the census did not record which field differed. (3) Record the post-data reader change as amendment A3. (4) Label "shared ancestry" as inferred from identical weights. (5) Housekeeping: part 1's placeholder, the cost table, the coverage line, the host-sleep deviation with G2 as its evidence, the interpretation question added to the review (b) brief. (6) The close sequence | All adopted: (1) the coverage test and [`n3-battery-coverage.json`](evolvability-diagnosis-run-2026-10-followup/n3-battery-coverage.json), the N3 cell and part 3 rewritten; (2) to (4) in the N3 cell and Run setup; (5) done before review (b); (6) followed |
| 4 | Before the plan forward was drafted (rule 8), with its six-part structure presented, while N3 ran | Structure right; six things. (1) Phenomenon in the candidate, arithmetic in the evidence: name T03.F12 by its phenomenon and analog, keep the realized hold semantics in the N1 cell as the tested realization, and name the open spec questions as questions. (2) Draft the T03.F12 row text the user has to add, in the track's own form, labelled a draft. (3) Update the expected goal-profile reading from N1 (births not reliably lower; "size per generation and total spawns fell on both seeds", of which the spawns half was wrong: `hold2` seed 2022 spawned 860,601 against the default's 835,989, review (b) finding 12), state the horizon limits, and name the decisive check N1 could not run: Canyon at its goal seed past 36,652. (4) Report the predeclared lose-to-useful-connect ratio from G3's on path with bounds. (5) Make T11.F28's "stays a lead" actionable: a predeclared non-loss control, more than one checkpoint, more parents; say whether T03.F12's goal-profile funnel supplies it. (6) Predeclare N3's two bad outcomes before the checkpoint lands. Also: the two user rulings stand; the `chore(deps)` landing is outside "docs only" and said plainly; the T03.F12 goal command verbatim | All adopted: (1), (2), (3), (5) in the plan forward, with (3)'s spawns claim corrected after review (b); (4) in the N2 cell; (6) in the N3 row; the two one-liners in the plan forward |
| 3 | After the N1 and N2 readings were drafted, before they were recorded; also the contradiction consultation for N1 (births not lower on seed 1022, population higher on 2022) and N2 (genome size below the default's on both seeds) | Drafts close; eight points. (1) Smoke-test the vote-delta path on a shrunk world now, before N3 reaches 20,000: `observe` had never run end to end. (2) "Only arm with causal target reads" is false: `slow` seed 2022 has one. (3) Add size per generation: `hold2` 1.16 and 1.90 against 1.76 and 4.91 (34 % and 61 % lower, both seeds); `slow` 1.73 and 2.42 (equal on 1022, lower on 2022). (4) Flag in N2's decision line the 10,000 reversal on seed 2022 (0 against 14) and the 125-living trough at 5,000. (5) N1 contradictions: density compensation as a hypothesis for the higher births; `hold2` seed 2022 declining at the end with size still growing, so "higher population" may be delay, not avoidance. (6) The ordering tie-break is indeterminate (each lead has one ratio above and one below 1): record that and decide on mechanism specificity. (7) Keep N2's verdict as the predeclared screen gave it; put the specificity caveat in the decision line. (8) Record as Advice 3, commit the cells with the summary and reader | All adopted: (1) the smoke run under Run setup; (2) to (7) in the N1 and N2 cells; (8) this row and the commit. Coverage gap (review (b) finding 16): N2's fourth contradicted prediction, `family_causal` staying 0 (one causal food-summary incidence on seed 2022), was in the drafted cell the advisor read but not named in the brief; it is put to the advisor explicitly in Advice 6 |
| 2 | Before the N1/N2 phase (gates G3 and G1, the mutation-off pairs, the main arms), after `make check` had stopped at rustfmt and been relaunched | Approach right; (1) `g3read.py` reads the E4 summary's keys, not the probe's raw keys (`lost_declaration`, `lost_declaration_by_ops`, `attempted_events_per_birth`, `applied_events_per_birth`, `decay`); rewrite it and run it on the off output first. (2) G3's "applied within 5 %" is the wrong invariant: the founder's silent entry turns otherwise-applicable `Prune` draws into terminal skips, so applied drops by about the Prune share; the invariant is the requested supply and applied excluding `InputRefPrune`; amend as A2 before G3. (3) `make check` will stop at the recipe-digest pin test: run the remaining targets one by one and clippy explicitly, record each exit; prototype results count only after clippy passes. (4) Say the battery state plainly to the user, who is active in chat. (5) Record this as Advice 2 before launching the pairs. (6) The session's manual `pgrep` matched its own wrapper's command line; the script's check is fine. Confirm the G1 header's `threads` is 8 before N3, else pin `RAYON_NUM_THREADS=8` | All adopted: (1) reader rewritten to the raw keys; (2) amendment A2; (3) the Run setup's make check line records the step-by-step run; (4) said in the session's next message; (5) this row; (6) the G1 header is checked before N3 |
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

### Review (b): the whole note before landing

Job `.bench-artifacts/lab/diagnosis2/codex/review-b.out.md`, verdict
`not-ready`, 12 blocking and 5 advisory. Every finding is adopted below; no
finding changes a validation verdict (both predeclared screens reproduce),
the feature named, or the order, and no second round is run.

| # | Finding (short) | Severity | Disposition |
| --- | --- | --- | --- |
| 1 | Sampled extremes misstated: `hold2-s1022` second-half size minimum 345 at 11,500; `default-s2022` population minimum 683 at 19,000; `slow-s2022` minimum 52 at 8,000 | blocking | Adopted: all three corrected in the N1 and N2 cells and part 2; the window means and rule inputs were unchanged |
| 2 | "Causal target reads on 3 of 20 parents" summed typed incidences | blocking | Adopted: three parent-family incidences on two to three distinct parents, in the N1 cell and part 2 |
| 3 | "Ordinary counts, operator classes and events are the default's" exceeded what G3 shows | blocking | Adopted: the requested-event law and operator definitions unchanged; applied and skip records and RNG consumption change; connected-entry loss equivalent within G3's tolerances |
| 4 | The 66 / 17 / 17 % route mix was the whole-child decay mix, not the watched entry's | blocking | Adopted: relabelled under Gate results; the summary carries the note |
| 5 | The exposure-backed downstream reading for type-0 reads at 50,000 was unsupported (battery-only coverage; E6 at 20,000 only; a nonzero nearest vector is not a nonzero `nearest_dy`) | blocking | Adopted: exposure against downstream silence unresolved per channel; the authored contexts' channel groups described; the N3 cell and part 3 rewritten |
| 6 | "None past the decision margin" contradicted by the 0.372 deltas against a minimum decision margin of 0.091; a within-kind margin does not show the kind loses | blocking | Adopted: margin distributions reported, the universal claim removed, masking called a hypothesis, positive deltas called ablation increases |
| 7 | Transcription errors: 16 of 20 parents at 50,000; the moving barrier channel has 0 divergent passes (the 18 belong to a zero-delta channel); the fifth moving channel at 20,000 has ratio median 4.5 × 10⁻⁶; "seven parents in the same triplet" | blocking | Adopted: all corrected from the channel summary; ancestry kept as a hypothesis |
| 8 | The reader's median of channel medians is not the predeclared median over margined passes | blocking | Adopted: the aggregation deviation recorded, the prediction marked unevaluated, classifications labelled reader-statistic and kept per checkpoint |
| 9 | The draft T03.F12 row prescribed the hold and promised an unmeasured mechanism | blocking | Adopted: the row's goal is the phenomenon and its analog; hold semantics stay in the Notes; the intended outcome stated as such |
| 10 | More parents do not stop a one-parent excess passing a strict inequality; pooled checkpoints are not replicates | blocking | Adopted: a minimum effect with an uncertainty criterion, an aggregation rule and the dependence stated in part 2 |
| 11 | N1 does not refute E8's "did not hold size near the founder"; the equal-population birth prediction is untested | blocking | Adopted: part 4 rewritten (E8 stands for its realization; N1 passes the corrected screen; untested and contradicted predictions separated) |
| 12 | "Total spawns fell on both seeds" is false (860,601 against 835,989 on seed 2022) | blocking | Adopted: Advice 4's disposition and part 2 corrected; spawns are not a prediction |
| 13 | Zero divergence means equal hop counts and end reasons, not identical dispatch; a mandatory vote-path stage would miss the parameter route | advisory | Adopted: wording, and two separate path annotations in part 3 |
| 14 | Four matching sampled fields do not make the binary "E8's production binary" | advisory | Adopted: "sample-equivalent on the compared fields"; the no-prototype-build deviation kept |
| 15 | The 20,000 reading was interpreted before G2 passed in full | advisory | Adopted: the chronology recorded under Run setup |
| 16 | Advice 3's coverage omitted N2's fourth contradicted prediction | advisory | Adopted: recorded as a gap in Advice 3's row and put to the advisor in Advice 6 |
| 17 | Useful-connect counts and ratio bounds absent from the G3 summary; the residual is defined for 5 of 9 and 1 of 20 families; zero kills and reruns to record | advisory | Adopted: the G3 summary regenerated with them; the denominators stated; the Run setup line |

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

Drafted after N1 and N2 closed, with part 3 written when N3 closes
(consultations before the draft, Advice 4, and after it, Advice 5; Codex
review (b) before closing). The order inside part 2 is the run's judgment,
stated as such.

### 1. Verdicts

**N1, the reproductive-time lead: validated** by its predeclared rule on both
Canyon seeds (persists to 20,000; window-mean genome size 4.57 × against the
default's 7.09 × and 9.14 × against 22.38 ×; population 0.55 × and 2.19 ×),
with the size effect holding per generation on both seeds (34 % and 61 %
lower) and the horizon limits stated in the N1 cell. **N3** completed with
G2 passed at both checkpoints; its reading is part 3. **N2, the slower-loss
lead: validated by its predeclared screen and not shown specific to slower
loss** (the `AreaFoodSummary:0` declared count 9 against 3 and 8 against 1 at
20,000 on 20 parents, matched in direction by the `hold2` arm, which changes
no loss rate, and reversed at 10,000 on seed 2022; G3 confirms the mechanism
does what it says on the founder).

### 2. What to change, in order

1. **T03.F12 — Genome-Length-Dependent Reproductive Time** (track T03;
   distinct from the closed T03.F11 token surcharge). Phenomenon: a parent
   that has just reproduced is occupied for a span that grows with its
   genome's length. Natural analog: copying a longer genome takes longer
   (Avida bounds genome size through replication time, Lenski et al. 2003,
   Methods; a cell's replication time grows with its genome). Evidence: N1
   (above) for the phenomenon's effect, E2 and E8 for the size runaway it
   answers (goal seeds 16 × and 20 × the founder at 20,000 and 118 × at
   50,000; Canyon extinct at 36,652). The realization the prototype tested is
   recorded in the N1 cell (a hold of ⌈size / 97⌉ ticks starting the tick
   after the birth, cognition still run and charged while held, death
   cancelling); whether cognition should be charged during the span, whether
   the unit is the founder's size or a per-unit fraction of a tick, and
   whether the span is a hold or a reduced action rate are the spec's
   questions, not this note's prescriptions. What ordinary variation still
   has to discover: everything about the circuit; the feature changes only
   what a long genome costs in time. **Expected goal-profile reading,
   updated from N1:** mean genome size at the goal checkpoints a smaller
   multiple of the founder's than the previous closure's and size per
   generation lower; births per creature-tick *not* reliably lower (N1:
   0.01977 against 0.01834 on one seed) and total spawns not reliably lower
   either (N1: 1,029,336 against 1,371,904 on seed 1022 but 860,601 against
   835,989 on 2022), so neither is a prediction; population not under half
   the previous closure's; the decisive check N1 could not run is
   persistence of Canyon at its goal seed past tick 36,652 (E2's
   extinction); T14.F12's funnel at the goal checkpoints read descriptively
   (the hold arm sampled three causal target parent-family incidences, on
   two to three of 20 parents, on one seed where the default sampled none: a
   sign to watch, not a claim). Limits: two fresh seeds of one world over 20,000 ticks, size
   still growing on seed 2022 with population declining at the end; the
   natural-world check shows ecological plausibility, not retention (run 1
   plan). **Draft row for the T03 track, for the user to add** (rule 6
   forbids the run adding it), in the track's own form:

   > - [ ] **T03.F12 — Genome-Length-Dependent Reproductive Time** — Depends on: T03.F11, T14.F12
   >   - Goal: Copying a longer genome takes longer, as in Avida's replication time. A parent that has just reproduced is occupied for a span that grows with its genome's length, so a large genome pays in reproductive time at every birth, the quantity selection counts, beside the token surcharge it already pays; the intended outcome is a smaller mean genome size over a goal run than the surcharge alone leaves.

   with a Notes paragraph citing this note's N1 cell for the tested
   realization (the hold semantics belong there, not in the row) and the
   numbers above, the dependencies (T03.F11 closed, the surcharge it sits
   beside; T14.F12 for the goal-profile funnel reading), `cargo test -p
   v3-core --test viability` first, and the three spec questions above.
2. **T11.F28 — Slower Loss of Silent Structure** (track T11; option B of
   T11.F22's recorded follow-on) **stays a lead**, not the next feature.
   Natural analog unchanged (an unused gene decays at the mutation rate per
   site; nothing removes an unused receptor at 1 % per generation).
   Evidence: G3 (the decay pass fires at the rate, the connected entry's
   hazard and the requested supply unchanged, the silent declaration's
   integrated loss 0.004942 per birth, 3.8 × below the default's; the
   lose-to-useful-connect ratio 706, bounds 333 to 1,806, against 2,668) and
   N2 (persistence on both seeds, the declared excess as recorded). What
   would make the screen specific, which N2 did not predeclare: a non-loss
   control arm predeclared as the contrast (what `hold2` was by accident), the
   typed declared counts read at every checkpoint on both seeds under a
   predeclared aggregation rule (the checkpoints of one world are dependent
   samples of one trajectory, not replicates) and required to agree in
   direction, and a predeclared minimum effect with an uncertainty criterion
   (for example the declared count's CP interval on 100 parents excluding
   the default's, or a per-seed excess of at least 5 of 20), since more
   parents alone leave a strict inequality passable by one parent. T03.F12's own goal-profile
   funnel reading supplies the default and the time-cost arm at the goal
   checkpoints in three worlds, not a slower-loss arm, so a separate row of
   N2's design with those three additions is needed before T11.F28 becomes
   a feature. What variation still has to discover: the connection, its
   sign and its weight. Recorded against it: `slow` fell to 52 living on
   seed 2022 at tick 8,000, and its genome size was below the default's on
   both seeds, the opposite of the row's cargo prediction.

Order: size first, as the diagnosis run judged, now on N1's validation and
N2's non-specificity rather than on phenomenon alone; the predeclared
tie-break was indeterminate (N2 cell).

### 3. The follow-up diagnosis question (N3)

The question was how large an executed read's contribution to the vote is
against the margin that decides, with dilution, cancellation or neither as
the answers. N3's answer on Confluence, by the reader's statistic, is **mixed at 20,000
and neither at 50,000**, in a form the question did not anticipate and with
one instrument limit found on the way: 34 of the 35 executed target reads
sampled at 50,000 alter no vote at all on the observation panels (zero
delta, Graph edges at weight about 1 from `gradient_x` and `nearest_dy`
among them); the few that alter a vote raise a sink of a kind other than
the committing one (`StealEnergy(3)` at 20,000, `Move(6)` at 50,000) by
0.35 to 0.38, past that kind's within-kind margin and on some passes past
the baseline decision margin, without any commit changing; and the two
reads the funnel calls causal at 20,000 do so with unchanged votes, hop
counts and pass-end reasons, which leaves the action-parameter surface as
the path. A zero delta has two readings the instrument cannot separate (a
consumer whose output never reaches a vote, or a channel the panels never
presented nonzero where that consumer ran), the `neighborhood-v1` battery
never presents any target-family channel, exposure when a consumer executes
was not recorded, and so the two readings stay unresolved per channel;
masking is likewise a hypothesis the stored per-channel statistics cannot
establish. Within those limits: the executed → causal stop the diagnosis
run saw is not a dilution of one edge's weight, cancellation among a
family's channels is not seen where the reading is defined, and causality
reached the action through two routes, the vote path (none of the sampled
reads) and the action-parameter path (the two causal reads). What this
changes in the plan: the follow-up question is answered as far as this
instrument reaches, and the next questions are observation questions, not
a mechanism feature of this run: per channel, whether the panels presented
a nonzero value where its consumer ran, and whether its consumer's path
reaches a vote sink or an action parameter (custom-output sinks, saturated
thresholds and unvoted downstream nodes are the candidates the census
cannot separate); and whether the vote-moving reads are masked by the
committing kind, read with paired effective votes per pass. For T22 and
T20, three readings follow: per-channel panel exposure when the consumer
executes (which the funnel's `causal` and retention stages for these
families have so far taken from the recorded and authored contexts alone,
here, in E2 and in T20.F01's goal reading), and two annotations of a
consumer's static path, "reaches a vote sink" and "reaches an action
parameter", as separate routes to causality between `executed` and
`causal`. T18.F01's tie-state hypothesis is untouched: no `nearest_dx` read
was executed in either sample.

### 4. Refuted or weakened

- E8's reading "the replication-time arm, at its realized one-tick-short
  hold, did not hold genome size near the founder" **stands as evidence
  about that realization** (its longer genomes did experience a positive
  hold) and is **not refuted**: N1 does not hold size near the founder
  either (4.57 × and 9.14 ×, with no "near" threshold declared); what N1
  adds is that the corrected hold passes the predeclared screen on the same
  world and seeds (size below the default's on both, 7.09 × and 22.38 ×,
  with persistence), where E8's realized hold was zero ticks for a
  founder-size genome.
- N2's prediction "silent cargo accumulates" (genome size not below the
  default's under slower loss): **contradicted** on both seeds (5.96 × against
  7.09 ×, 8.93 × against 22.38 ×), with fewer generations on one seed and no
  explanation on the other.
- The predeclaration's claim that `slow-mutoff` is byte-identical to
  `default-mutoff` by construction: **wrong** (the founder's node 0 carries a
  silent `NeighborOccupiedRing` entry); the pair nonetheless reproduced the
  default's population and spawn counts at every sample.
- The diagnosis note's expected T03.F12 reading "births per creature-tick
  lower at equal population": **untested** (N1's populations were not
  equal); N1's own unconditional prediction "births per creature-tick lower"
  was **contradicted** on seed 1022 (0.01977 against 0.01834; the
  density-compensation hypothesis in the N1 cell).
- Carried, not refuted: `hold2`'s higher population on seed 2022 (within
  the horizon, declining at the end); the `hold2` and `slow` causal target
  reads at 20,000 on seed 2022 (3 and 1 of 20 parents; one checkpoint, one
  seed each).

### 5. Rulings, landing, the next goal command

The diagnosis run's two user rulings (index alignment against not-forcing
check 3; the lab's role and the T22 amendment sentences) are unchanged by
this run and still await the user. The landing on `main` is the `docs:`
commits plus the `chore(deps)` commit of the `source-map-js` upgrade, which
is outside "docs only" and lands by the user's direction of 2026-10-06.
The next goal command is the diagnosis note's T03.F12 command, verbatim,
to be pasted after the user adds the row above to the T03 track:

```
/goal Roadmap feature T03.F12 is complete on main. Read docs/workflow.md first and follow its per-feature contract exactly: confirm you are Opus 5.5 at effort medium in the main checkout on a clean main; create the feature worktree with EnterWorktree named t03-f12; delegate the flat spec and its Codex adversarial challenge rounds to roadmap-spec-owner, verify the final Codex verdict, and commit the spec there, and route requirement questions during implementation back to that same spec owner; delegate feature implementation and production-code remediation to roadmap-implementer, and, unless the workflow's Benchmark gate exempts this feature from them, the gate and goal baseline runs and their records to roadmap-benchmark-specialist and the mutation gate and test-only survivor remediation to roadmap-mutation-specialist; run the final diff review as a fresh read-only Codex Astra high job through the Codex channel; run the benchmark and mutation specialists sequentially and never alongside competing builds, tests, servers, or measurements; run make check in the worktree; ExitWorktree with keep, fast-forward main to the feature branch, then remove the worktree and its branch. Done means all of these are shown in this conversation: the T03.F12 row is checked in its track roadmap on main and its spec is Complete; make check exited 0 on the feature code now on main and make check-docs exited 0 at the commit now on main; git worktree list no longer lists the feature worktree; git status on main is clean. If a concrete blocker stops the feature, record it in the spec, report it, and stop. Stop after 80 turns.
```

## Cost

| Row | Wall |
| --- | --- |
| Predeclaration and amendments A1, A2 | session time; no heavy job |
| Builds, tests, `make check` and the step-by-step remainder | about 25 min in all |
| G3 | 111 s (off) and 40 s (on) |
| G1 smoke | 371 s (including the release test build) |
| N1 and N2 pairs (`default-mutoff`, `hold2-mutoff`, `slow-mutoff`, both seeds) | 230, 201, 456, 245, 235, 191 s awake |
| N1 and N2 main arms (`default`, `hold2`, `slow`, both seeds) | 1,474, 1,418, 1,164, 1,393, 1,323, 1,209 s awake (about 2 h in all) |
| Vote-delta smoke | 1.2 s and 8.1 s |
| N3 | 15,611 s awake (17,392 s elapsed, about 30 min of host sleep); 15,547 s of simulation, 7.2 s of censuses, 0.62 s of vote-delta reading |
| Battery coverage reading | seconds plus a debug test build |
| Codex reviews (a), (b) | remote; no host time |

Turns: about 165 of 200 at the closing commit; the final count is in the
final report.
