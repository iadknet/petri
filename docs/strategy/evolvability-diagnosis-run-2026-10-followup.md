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
Status: **in progress** (predeclaration).

## Question

The diagnosis run named two unvalidated leads and one open question. (1) Does
a genome-length-dependent reproductive time, counted correctly from the tick
after the birth, hold genome size and keep the population, in a natural world
beside the default? (2) Does a slower loss of silent structure, with the
janitor routes (`Prune`, type retargeting, `Swap`) acting on silent entries at
a per-unit decay rate instead of their operator rates, raise the declared share
of the target sensor families in the live world without costing persistence?
(3) In the live Confluence world at 20,000 and 50,000 ticks, how large is an
executed sensor read's contribution to the vote surface against the margin
that decides the action: dilution (small against the margin), cancellation
(channels of one family cancel each other), or neither?

## Run setup

- **Roles.** Session on Fable 5.1 (`claude-fable-5-1`), the implementer; the
  Fable advisor is enabled and consulted through the `advisor` tool at every
  rule 8 point (the Advice table). No subagent spawned at predeclaration.
  Reviews: Codex `gpt-6.1-sol` `high`, read-only, through `codex exec`
  (`codex-cli 0.159.1`): (a) this predeclaration before any row runs; (b) the
  note's readings before closing.
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
  mechanism prototypes). `instr:`, `lab:` and `proto:` stay on the branch;
  only `docs:` lands on `main`.
- **Code taken from the previous branch** (file state, sources cited in the
  commit messages, no cherry-pick of mixed commits): the census probe
  `crates/v3-cli/src/bench/diagnosis_census.rs` (`ccb7d1fe`, `8cc1730a`), the
  reproduce-hold prototype (`7ea581e8`: `config/simulation.rs`,
  `creature/state.rs`, `simulation/tick.rs`), the E4 probe
  `crates/v3-lab/tests/diagnosis_probe.rs` (`09f78786`, `b2ed9dd9`). The
  `AddProjection` prototype and the one-edge arm sets are not taken.
- **Branch verification.** TDD for the prototypes; `cargo test -p v3-core
  --test viability` first (both prototypes touch births); `make check` on the
  branch before a prototype's results count, with the `v3-cli` recipe-digest
  pin test the one allowed failure (two new `RuntimeConfig` fields), as the
  run 1 plan allows; recorded under Run setup when run.

## Gates (predeclared, blocking)

| ID | What | Rule |
| --- | --- | --- |
| G1 | Instrument identity: the census probe (which builds its config through `resolve_config` on the recipe, as `v3-cli run --config` does) run on the Canyon goal recipe, seed 1022, default config, for 2,000 ticks with samples every 500 | Every inline sample's `population`, `mean_genome_size`, `mean_generation` and cumulative spawn count must equal E8's `default-s1022` `tick_sample` fields at the same ticks (E8 ran `v3-cli run` on the pinned launch-revision binary). Unequal blocks N1 and N2 until fixed |
| G2 | Observation identity: N3's census is the E2 census plus a reading | N3's `input_use` blocks at 20,000 and 50,000 must be byte-identical (sha256 of the serialized block) to E2's Confluence blocks in `.bench-artifacts/lab/diagnosis/e2/census-confluence.ndjson`; its `e6` block at 20,000 likewise. Unequal voids N3's vote-delta reading until explained |
| G3 | Mechanism identity for N2, on the founder, through the E4 probe (`PETRI_E4_BIRTHS=1000000`, parents founder, founder+declared, founder+declared+zero-edge, no elites) on the N2 binary, once with the rate off and once at 0.005 | Off: `lose_declaration` of founder+declared inside E4's 95 % CP interval [0.01856, 0.01873] and of the connected parent inside [0.00791, 0.00802] (the flag-off path is E4's). On: the silent declaration's per-birth loss has a CP interval containing 0.005 (the decay clock alone), **and** the connected parent's loss stays inside E4's [0.00791, 0.00802] (the addressed-entry hazards unchanged). Either failure marks N2 confounded: its world runs may run but do not count toward validation |

## Ledger

Every row is predeclared here before it runs; `incomplete` rows are kept. No
diagnostic (authored-genome) arm is used in this run.

| ID | Question | Method, instrument, sizes | Prediction / decision rule | Status | Result |
| --- | --- | --- | --- | --- | --- |
| N1 | Does a correctly counted genome-length-dependent reproductive time hold genome size and keep the population? | `proto:` `energy.lifecycle.reproduce_hold_per_founder_size` (default off, policy-deviation when on), corrected: a committed `Reproduce` that spawns at tick *T* sets `held_until_tick = T + 1 + ceil(genome_size / 97)` and the parent commits no action while `sim.tick < held_until_tick`, that is on ticks *T*+1 … *T*+⌈size/97⌉ (the founder: exactly one tick; 98 to 194 units: two); cognition and its charges run as usual; death cancels the hold; supply, transfer and charges unchanged. **Tested on the founder:** a unit test pins a 97-unit spawner to commit nothing on *T*+1 and to be unheld on *T*+2, a 98-unit genome to two ticks, and the off path to leave `held_until_tick` 0 everywhere. Design as E8: Canyon goal recipe, fresh seeds 1022 and 2022, 20,000 ticks, samples every 500, every arm on both seeds, through the census probe with checkpoints 10,000 and 20,000 (selected cohort of 20 parents, founder cohort, drift empty; the funnel block at both; E6 reading at 20,000), 3 h awake kill per run. Arms: `default` (rerun on this binary, flags off), `hold2` (the prototype on), `default-mutoff` and `hold2-mutoff` (`per_unit_rate` 0, the immediate-effect pair: fixed founder genomes, hold one tick per spawn). Readings over ticks 15,000 to 20,000 (window means, as E8's reader): mean genome size in founder units, mean population, births per creature-tick, extinction interval; the funnel's target-family rows at 20,000 descriptively; the mutation-off pair's extinction ticks and population trajectories (E8: both default pairs die in the trough before 5,500) | Prediction: `hold2` persists on both seeds with window-mean genome size below `default`'s (E8 `default`: 7.1 × and 22.4 ×), births per creature-tick lower, population lower than `default`'s but not collapsed. **Validated** (T03.F12 becomes the next feature) if on **both** seeds `hold2` reaches 20,000 ticks, its window-mean genome size is below `default`'s, and its window-mean population is at least 0.5 × `default`'s (E8's realized arm sat at 0.39 ×, the known failure); the pair reads the direct cost (an earlier extinction than `default-mutoff` on both seeds is a recorded cost, not a failure). **Not validated** otherwise. Two seeds, one world: a Canyon observation, never a general bound | predeclared | — |
| N2 | Does slower loss of silent structure raise the declared share of the target families in the live world without costing persistence? | `proto:` `mutation.silent_entry_decay_rate: Option<f64>` (default `None`, policy-deviation when set). A **silent entry** is an input reference no consumer on its node addresses (`prunable_indices`: no Graph input leaf and no VM `ReadInput` names it, live or not). On: in the ordinary engine an InputRef event whose drawn target entry is silent is **skipped, not redrawn** (a new skip reason `SilentEntryDecaysOnly`): `Prune`, whose every site is silent by construction, is skipped whenever drawn; `Swap` and `RawFieldMutation` draw their entry exactly as today and skip when it is silent, so the event budget and the addressed-entry hazards are those of the default (Advice 1, point 1). A separate **decay pass**, run at every birth after the event loop (also on births that drew zero events), loses each silent entry independently with probability = the rate, by a route drawn uniformly among the routes that apply to that entry: prune (remove and renumber), swap (another member of its kind), retype (the food-type index, for typed food entries; the slot, for `UpstreamSlot`); entries are visited in descending index order per node so a removal never shifts a pending index; each loss is recorded as an applied `InputRefPrune` / `InputRefSwap` / `InputRefRawFieldMutation` event. **Rate 0.005 per silent entry per birth**, equal to `per_unit_rate`: a silent entry is one genome unit and decays at the rate that supplies every other event (the "lost at the per-site mutation rate" analog, scaled to Petri's clock); against E4's measured 1.86 × 10⁻² this is 3.7 × slower and moves the lose-to-useful-connect ratio from about 3,400 toward about 900. Gate G3 on the founder before any world run. Tests: off path byte-identical (an engine test with the rate `None` reproduces the existing fixtures; the viability suite); on: `Prune` never applies, the decay pass loses a silent entry at the rate within a seeded tolerance and never touches an addressed entry; determinism on a fixed seed. Design as N1: Canyon, seeds 1022 and 2022, 20,000 ticks, census probe with checkpoints 10,000 and 20,000; arms `slow` and `slow-mutoff` (`per_unit_rate` 0 with the rate on: the founder's two nodes carry 5 and 7 references, all addressed, so no decay fires and the arm is byte-identical to `default-mutoff` by construction; it runs because the design requires the pair and the run verifies the identity). `default` and `default-mutoff` are N1's runs. Readings: N1's window readings plus, at 10,000 and 20,000 on the selected cohort, per target family (`AreaFoodSummary`, `NeighborBarrierRing`) `family_declared`, `family_connected`, `family_executed`, `family_causal` over `parents_evaluated`, and the declared-but-unconnected count `family_declared − family_connected` | Prediction: at 20,000 the declared share of at least one target family is above `default`'s on both seeds; mean genome size is not below `default`'s (silent cargo accumulates); population not lower than 0.5 × `default`'s; `family_causal` unchanged at 0. **Validated** (T11.F28 becomes the next feature) if G3 passed, `slow` reaches 20,000 on both seeds, its window-mean population is at least 0.5 × `default`'s on both, and for at least one target family `family_declared` on the selected cohort at 20,000 is above `default`'s on both seeds (20 parents per cohort: a direction, stated as such). **Not validated** otherwise. If N1 and N2 both validate, the diagnosis run's judgment order holds (size first) unless N1's population reading is the weaker | predeclared | — |
| N3 | How large is an executed read's contribution to the vote against the margin that decides? | `instr:` the E2 census (`diagnosis_census_run`, Confluence seed 33, horizon 50,000, checkpoints 20,000 and 50,000; the 10,000 checkpoint is dropped, which changes no later state) plus a **vote-delta reading** on the selected cohort at each checkpoint: for each sampled parent and each channel of `AreaFoodSummary(k)` and `NeighborBarrierRing` at the executed stage, the parent and its one-channel ablation (`ablated`, the funnel's own rewrite) run on the same panels (`neighborhood-v1` battery plus the T11.F26 extension) under a vote-recording execution mode that keeps every pass's final vote vector, effective votes and committed action; passes are aligned by index, and a pass-count mismatch is counted apart. Per (parent, channel): scenes, passes, passes with any nonzero Δ (Δ = ablated − baseline votes, per sink), max |Δ| and its sink, the L1 sum of |Δ| over sinks, the sign share of Δ on its most-moved sink, the baseline **decision margin** on each Δ-pass (effective vote of the winning kind minus the largest other kind's effective vote; when no kind is positive, minus the largest effective vote), the within-kind margin of the moved sink's kind (best minus second-best vote), the ratio max |Δ| / within-kind margin and the count of passes where max |Δ| ≥ that margin ("near-decisive"), passes where the committed action differs (must agree with the funnel's `causal`), and the channel's consumer count and summed |weight|. Per (parent, family): the same for the **whole-family ablation** (every channel of the family at once, Advice 1, point 2c), plus the ratio of the family's L1 delta to the sum of its channels' L1 deltas (below 1: cancellation among channels). Written as a `vote_deltas` block beside `input_use` in the checkpoint row. **6 h awake kill** (rule 3's doubled limit, predeclared because E2's Confluence census took 15,108 s); G2 before the reading | Prediction: at both checkpoints the median over executed target-family channels of max |Δ| / within-kind margin is below 0.1, and the whole-family ablations change no committed action (consistent with E2's causal 0). Readings are descriptive and narrow the diagnosis run's three hypotheses: **dilution** when per-channel and per-family ratios are both small (median below 0.1) and the family/channel L1 ratio is near 1; **cancellation** when the family's L1 delta is well below the channels' sum (ratio below 0.5) or the per-channel ratios are large (median above 0.5) while no action changes; **neither** when ratios are large and actions do change under the family ablation (then the executed → causal stop is a panel question). A world extinct before 50,000 completes with the checkpoints reached (amendment A5 of the diagnosis run) | predeclared | — |

## Advice

Every Fable advisor consultation rule 8 lists, with the disposition of each
point raised. Coverage so far: before phase 0 (the whole run), Advice 1.

| # | Rule 8 point | Advice (summary) | Dispositions |
| --- | --- | --- | --- |
| 1 | Before anything was built: the approach to the three rows after the contract was read | Approach sound in shape; six fixes. (1) N2 as "Prune inapplicable, Swap/RawField draw only addressed entries" confounds slower silent loss with faster connected loss and more growth (the discard-and-redraw loop hands the draws to other operators); make it skip-not-redraw and verify on the founder with the E4 probe: silent loss within the rate's CP interval, connected loss within E4's. (2) N3: predeclare a 6 h limit (E2 took 15,108 s); define margin on the effective votes per pass, align passes by index; add a whole-family ablation, since one-channel ablation cannot see cancellation. (3) Two blocking identity gates: the probe's default arm against E8's samples before any proto arm; N3's `input_use` blocks hash-equal to E2's. (4) Battery: `caffeinate -s` is inert on battery; order short jobs first, notify the user once, record the state at every heavy launch. (5) EnterWorktree may base on `origin/main`: confirm the head is `d4bc4a98`; take probe and hold code as file state, not mixed cherry-picks. (6) Predeclare the validation thresholds (N1: size below default on both seeds, both persist, population at least a stated fraction; N2: declared share above default on both seeds, persistence not worse, the G3 rates), the rate choice and why, and that `slow-mutoff` reads nothing by construction but runs | All six adopted in this predeclaration: (1) N2 row and gate G3; (2) N3 row (6 h, effective-vote margins, whole-family ablation); (3) gates G1 and G2; (4) Run setup (host) and the order below; (5) head confirmed `d4bc4a98`; file-state commits listed under Run setup; (6) the N1 and N2 decision rules, the 0.005 rate and its reason, the `slow-mutoff` sentence |

## Codex reviews

Review (a), the predeclaration, is sent before any row runs; its findings and
dispositions follow here as amendment A1.

## Order

Predeclaration committed → Codex review (a) launched → builds and tests
(viability first) → G3 (seconds) → G1 (about 3 min) → the four mutation-off
pairs (minutes each) → `default`, `hold2`, `slow` on both seeds (about 25 min
each) → N1 and N2 readings with their consultations → N3 (about 4.5 h) and
G2 → N3's reading → plan forward (consultations before and after) → Codex
review (b) → close in the run 1 order, docs only.

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
| Predeclaration | — |
