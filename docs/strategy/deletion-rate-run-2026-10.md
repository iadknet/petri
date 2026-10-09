# Deletion-rate experiment run: results (2026-10)

Date: 2026-10-09. Contract: the user's goal command of 2026-10-09, which takes
the [deletion-rate experiment plan](deletion-rate-experiment-plan-2026-10-09.md)'s
prototype, gates S0 and S0.2, row S1 and draft validation rule as the
contract's form (the rule as written). It also takes the three conditions of the
[diagnosis follow-up run](evolvability-diagnosis-run-2026-10-followup.md)'s plan
forward item 2: a non-loss control arm declared in advance, every checkpoint
read on every seed under a declared aggregation rule, and a declared minimum
effect. Its form follows the
[output-wiring run](output-wiring-run-2026-10.md) (gates, frozen-instrument
pattern, Advice table, Codex reviews), under the
[diagnosis plan](evolvability-diagnosis-plan-2026-10-05.md)'s rules 1 to 9 as
far as they apply to a one-row run, and inside the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md).
Status: **predeclared** (nothing has run).

## Question

T11.F28's premise, restated for `main` after T20.F16: deletion operators remove
new structure before refining operators can tune it, so it never becomes
useful. The plan's three decision criteria are the run's questions. (1) Is
deletion still a material hazard on `main`, and for which structure (gate S0)?
(2) Does slowing deletion, and nothing else, raise executed and causal sensor
use by more than trajectory noise (row S1)? (3) What does that cost
persistence and genome size (row S1)?

## Run setup

- **Roles.** Session on Opus 5.5 (`claude-opus-5-5`, reported by the app's
  session record), the implementer: it writes the instrument, the probe and
  the prototype, runs the gates and the row, reads the results and writes this
  note. No subagent is spawned. The `advisor` tool is enabled and consulted at
  every rule 8 point (the Advice table). The goal names the advisor as Opus
  5.5, but both the project's `.claude/settings.json` and the user's
  `~/.claude/settings.json` set `advisorModel: "fable"`. The model serving the
  advisor cannot be inspected from inside the session, so confirming it is the
  user's. Reviews: Codex `gpt-6.1-sol` `high`, read-only, through `codex exec`
  (`codex-cli 0.159.1`, the run 1 plan's command in a POSIX `sh` wrapper).
  Review (a) covers this predeclaration before any gate or row runs, with its
  findings disposed and any amendment committed first. Review (b) covers the
  note's reading before closing.
- **Launch revision.** `main` `7582325d`, clean in tracked files. One
  untracked file, `docs/strategy/action-choice-pressure-research-2026-10-09.md`,
  belongs to another session; this run leaves it untouched. The worktree is
  `.claude/worktrees/deletion-rate` on branch `worktree-deletion-rate`, created
  with `EnterWorktree` (`baseRef: head`); its head was checked equal to
  `7582325d` after creation. `npm ci` ran in `frontend/`. Disk free at launch
  was 51 GiB (rule 9's stop line is 20 GiB).
- **Host.** On AC power at launch (100 %, charged). Every heavy job runs under
  `caffeinate -i` through `diagrun.sh` (copied from the output-wiring run),
  which kills a job after 3 h of awake time. The power state is recorded at
  every heavy launch. One heavy job runs at a time, with
  `pgrep -fl 'v3-lab|v3-cli|census-|probe-'` checked before each.
  `RAYON_NUM_THREADS=8`. The census and the probes are test binaries and
  start no telemetry.
- **Evidence.** Raw output goes under `.bench-artifacts/lab/delrate/` in the
  worktree and is copied to the main checkout at closing. Compact summaries
  and readers sit beside this note under
  [`deletion-rate-run-2026-10/`](deletion-rate-run-2026-10/), each under the
  1 MiB ceiling. Recipes are `jq` overlays of
  `experiments/worlds/canyon-country.json` stored under the evidence tree,
  never under `experiments/worlds/`.
- **Commit kinds.** `docs:` (this note and its folder), `instr:` (the census
  probe; observation only), `lab:` (the E4 and S0 probe), `proto:` (the
  default-off mechanism prototype). `instr:`, `lab:` and `proto:` stay on the
  branch; only `docs:` lands on `main`.
- **Code taken from `worktree-output-wiring`.** Taken as file state, not
  cherry-picked, with sources cited in the commit messages:
  - The census probe `crates/v3-cli/src/bench/diagnosis_census.rs`, the
    vote-delta reading, the sink-use funnel with its fixtures, and their
    module lines, all at `45923a4c` (the state of the `instr:` commits
    `520b4b4b` and `45923a4c`).
  - The E4 probe `crates/v3-lab/tests/diagnosis_probe.rs` at `3ead37f6`
    (`lab:`).

  The `proto:` commits `2065366e` and `7f1c8da8`
  (`born_connected_destinations`) are not taken. Every reference to that
  field is replaced by this run's field.
- **Draw-path drift since the output-wiring launch (`e847ec8b`).** One crate
  commit landed on `main`: `38ba9906`, a food recovery-spawn skip on
  zero-fertility cells in `kernel/ordinary_food/ecology.rs`, described as
  bit-identical. Nothing changed under `mutation`, `creature`, `neighborhood`
  or `v3-lab`. A descriptive check, not a gate: the frozen census's `default`
  samples on seeds 1022 and 2022 are compared tick for tick with the
  output-wiring run's `default` samples (population, genome size, generation,
  spawns). If they are equal, that run's numbers, and the born run's that it
  reproduced, describe today's production. If they differ, the difference is
  recorded as a finding.
- **Instrument changes (`instr:` `1e375bd7`).**
  1. The prototype field `mutation.deletion_keep_probability` is read through
     the serialized `MutationConfig` (absent reads as `null`). The funnel's
     one-event retention proposals drop the field (frozen meaning under every
     arm). The header's `prototypes` entry and the births block's
     `mutation_config` carry it.
  2. **Cohort dump.** With `PETRI_E2_DUMP_COHORT=1`, every checkpoint writes
     `cohort-<label>-t<tick>.json`, a file outside the census row. It holds the
     arm's serialized mutation config, the world's food type count, the
     executed window, and, per selected parent (the funnel's selection, same
     seed and ranks), the index, generation, age, genome size, genome, its
     `cached_reachable_nodes`, and its executed set. The executed set is
     resolved from the parent's dispatch record at its age over the executed
     window, exactly as `spawn_child` passes it to the engine.

  The instrument builds without the prototype.
- **Probe changes (`lab:` `339eb2ce`).**
  1. `PETRI_E4_KEEP=<s>` sets the prototype field through the serialized
     config. The frozen probe, built without the field, rejects it, which is
     correct.
  2. A new ignored test, `s0_deletion_hazards`. Parents are the founder, with
     E4's frozen lab-scene executed set, plus the parents of the cohort dumps
     named in `PETRI_S0_COHORTS`, each with the reachable and executed sets
     from its dump. Each parent gets `PETRI_S0_BIRTHS` one-birth children
     through `apply_mutations_with_food_type_count` at the probe's production
     `MutationConfig`. The probe asserts that config equals each dump's with
     the prototype key removed, so the keep setting is the only policy
     difference. Food types come from the dump (Canyon: 1).
  3. Child seeds are `0x5000000000 + 10⁸ × parent_index + i`, the same at
     every keep setting, so the requested event counts are equal by
     construction.
  4. Per parent the probe reports: requested and applied events, applied
     events by operator, skips by operator and reason, applied `RemoveNode`
     by the target's reachability, and the per-class sites counted on the
     parent (below). From those it derives h, L over 400 births, and the
     edit-before-deletion ratios R.
- **Reference binaries.** The **frozen instrument** is the census test binary
  and the probe, built from the `instr:` + `lab:` state (`339eb2ce`, no
  `proto:`) in a detached nested worktree under the evidence tree. They are
  copied to `census-frozen` and `probe-frozen` and hashed, and the nested
  worktree is removed before `ExitWorktree`. The **prototype binaries**
  `census-proto` and `probe-proto` are built from the branch head and hashed.
  Each binary is checked to list its test.
- **Branch verification.** The prototype was built test-first: the tests
  failed to compile before the field existed, then passed. On the prototype
  revision `cargo test -p v3-core --test viability` passed (28 tests) and
  `cargo test -p v3-core --lib mutation` passed (497 tests, the existing
  pinned engine tests run at the default `None`). The field is not serialized
  at `None`, so the default `config_digest` is `main`'s and the `v3-cli`
  recipe-digest pin test should stay green. `make check` on the branch is
  recorded under Gate results with its exit code, and it gates the driver.
- **Amendment A1 (pre-data, after Codex review (a)).** The 17 dispositions
  under Codex reviews. Every fix is in the readers, the runner, the gate
  wording or one prototype test; the instrument, the probe and the prototype's
  behavior are unchanged.

## The prototype (`proto:` `d6e9a82e`)

`MutationConfig.deletion_keep_probability: Option<f64>`.

- `None` is production. It is absent from the serialized config, so recipe
  digests are unchanged. Normalization clamps a set value to [0, 1] and drops
  a non-finite one.
- **Realization, and how it differs from the plan's wording.** The plan places
  the draw "after an operator passes the T13.F03 applicability check and
  before it is applied". Topology operators have no applicability predicate
  separate from their `apply` (the eligible set and the target draw happen
  inside it), so the prototype realizes the same policy at a uniform point:
  1. Each of the engine's four `apply_*_event` wrappers runs the operator as
     today.
  2. If the operator returns `Ok`, the operator is `Decreasing`, and the field
     is `Some(s)`, one explicit draw `u = rng.gen::<f64>()` follows the
     operator's own draws, before the parseability gate.
  3. On `u ≥ s` the genome is restored to its pre-event snapshot and the event
     returns `Err(DeletionNotKept)`. The engine loop records it as an
     operator-level terminal skip (counted applicable, then skipped) and never
     redraws it.
  4. An operator with no site (`NoApplicableTarget`) is discarded and redrawn
     as today, with its `discarded` record and no keep draw. An operator that
     fails for another reason draws nothing either.
- **Invariants.**
  - `None` executes no new path, so it is `main` draw for draw: the existing
    engine tests pass unchanged, and gates S0-F and S0.2 check it at the
    founder and the world.
  - `Some(1.0)` is policy-identical to `None` and stream-perturbed by one
    `f64` per applied `Decreasing` event. rand's `gen_bool(1.0)` would draw
    nothing, which is why the draw is explicit.
  - The requested count is drawn before the event loop, so it is identical at
    every s for the same seed.
  - The kept share is applied `Decreasing` ÷ (applied `Decreasing` +
    `DeletionNotKept`). Its expectation is s.
- **The `Decreasing` operators** (the plan's table): `InputRefPrune`,
  `GraphRemoveGraphEdge`, `GraphRemoveInternalGraphNode`,
  `GraphDisableHebbian`, `GraphDisableRewardModulation`, `TopologyRemoveNode`,
  `TopologyRemoveRouteTarget`, `VmDeleteInstruction`. Size pressure stays off,
  as in production.
- **Tests** (`mutation/engine/deletion_keep_tests.rs`, 8):
  - unset, or not `Decreasing`: no draw consumed;
  - set: exactly one `f64` draw, kept exactly when u < s;
  - `Some(1.0)`: never records the new skip (over 400 twelve-event founder
    births);
  - `Some(0.0)`: applies no `Decreasing` operator, keeps the requested count
    equal to `None`'s, and still discards and redraws the founder's
    inapplicable `DisableRewardModulation`;
  - a not-kept single-event birth leaves the founder unchanged;
  - `Some(0.5)`: kept share within a 99.9 % normal band of 0.5 over 2,000
    births, and deterministic;
  - serialization: absent at `None`, round-trips at `Some(0.03)`;
  - a proptest that applied plus skipped equals the requested count, and
    that the request equals the unset policy's, at every s in [0, 1].

**Natural analog** (the plan's): an unused or new gene decays at the mutation
rate per site. Deletion mutations are rarer than the substitutions that refine
a sequence. NEAT has no deletion mutation, and CGP keeps inactive genes as
neutral material. The prototype scales the rate of every deletion route by one
factor. It does not model selection against harmful structure (the Kuo and
Ochman counter-case): that removal is left to selection.

**The five not-forcing checks.**
1. A general rule over one existing operator class (`complexity_effect() ==
   Decreasing`) in all four domains. It names no sensor family, no arena, no
   sink and no scene.
2. The analog is above.
3. No correspondence: which deletion is kept is a uniform draw independent of
   the genome.
4. No score enters any draw. The row runs in a natural world with births and
   no scorer.
5. What variation still has to discover: everything about the circuit. The
   prototype only lets structure that already exists stay longer.

**What it changes beyond the deletion rate.** Kept structure enlarges the
genome, and `genome_size` sets the supply (`per_unit_rate` per unit), the
carrying cost (T03.F08) and gestation (T03.F12). So supply, cost and
reproductive time all move with the genome. This is a coupled consequence,
named here, that the `del-1.0` control does not remove.

**Contracts it deviates from, stated.** The
[mutation spec](../reference/v3-mutation-spec.md)'s operator semantics, under
which an applicable drawn operator is applied, do not hold when the field is
set. On this branch the prototype is a `policy-deviation` arm behind a
default-`None` field. A feature would either make it production or remove it.

## Gates (predeclared, blocking)

Every gate must pass, after any disposition and rerun, before the step it
blocks runs. S0-F passes before any world run. S0.2 passes before any S1
treatment arm is read. S0 must be a go, with its manipulation check passed,
before any treatment arm runs.

| ID | What | Rule |
| --- | --- | --- |
| S0-F | **Founder mechanism gate** (added by Advice 1 point 3; an ordering decision under rule 9: the plan's founder-level "check against E4" and the manipulation check are run on the founder before the world chain, so a broken prototype is found before four world runs; the go/no-go on the 40 evolved parents keeps the user's order). Through the probes at 10⁶ births per parent: (i) `e4_transition_rates` (parents (a) founder, (b) founder+declared, (c) founder+declared+zero-edge) on the **frozen probe** and on the **prototype probe** at `None`; (ii) `s0_deletion_hazards` on the founder alone (food types 1) on the frozen probe at `None` and on the prototype probe at `None` and at 0.03 | **(1) Identity at `None`:** the prototype probe's E4 output equals the frozen probe's in the deterministic projection (every field but `wall_secs`) on all three parents, and its S0 founder row equals the frozen probe's in the same projection; the `config` blocks are equal. **(2) Check against E4:** parent (b)'s `lost_declaration` CP interval at `None` contains E4's 0.018643. **(3) Manipulation at 0.03 on the founder:** requested events equal to `None`'s exactly; for every `Decreasing` operator with at least one keep trial, the kept share's CP interval contains 0.03; every other operator with at least 4,000 applied events at `None` within 5 % of its `None` count. (S0's check (c) is not part of S0-F.) **Dispositions (amendment A1, findings 3 and 5):** the exact checks ((1), the parent sets, (3)'s requested equality) block on any failure; a failure of (1) is a build or tree defect (rebuild, supersede every output, rerun). Each statistical check ((2) and the kept shares) is a 95 % CP containment; a miss is a **mechanism defect** only when its exact two-sided binomial p-value is below 0.001 (at most nine such checks here, a family-wise false-defect rate under 1 %), and then no world run starts until a fix passes; a miss with p ≥ 0.001 is a **statistical miss**, recorded with its p-value, and unchanged evidence is never rerun until it passes. A statistical miss or a (3) tolerance failure stops the runner until its disposition is committed |
| S0.2 | **Identity**: the census probe on the prototype binary at `None` against the frozen instrument built without the prototype, as W0.2 (ii) | The frozen census runs `default` on seeds 1022 and 2022 (20,000 ticks, checkpoints 10,000 and 20,000, every reading on, cohort dump on): the references. The prototype binary's `default` runs on the same seeds must equal them on every sample row (`population`, `mean_genome_size`, `mean_generation`, `mean_age`, `mean_energy`, `births_total`, `reproduction_actions_spawned_total`, `mutation_events_applied_total`, each required present), the extinction tick, the footer, every scheduled timing-free checkpoint block (`input_use`, `e6`, `vote_deltas`, `births`, `sink_use`, `memory_loop`, `memory_sensitivity`, `temporal_memory_sensitivity`, the cohort and decile fields; required present and not null) by hash of the sorted-key compact serialization, the whole header but the recipe path and run label (`config_digest` and `prototypes` included), and both cohort dump files by file hash. Expected ticks and checkpoints are derived from the header and the run's completion or extinction record (the output-wiring `identity.py`, field renamed). A failure blocks every treatment arm's reading until explained and fixed |
| S0 | **Does deletion still matter on `main`?** (go/no-go). Parents: the founder plus the 20-parent selected cohort of the prototype binary's `default` at 20,000 on seeds 1022 and 2022 (40 evolved parents, from the cohort dumps). The prototype probe's `s0_deletion_hazards` at `None` and at 0.03 (`PETRI_E4_KEEP=0.03`). **Birth count, predeclared with its fallback:** 10⁶ per parent per setting; before either setting runs, one smoke at 10⁵ births on the dump's largest genome at `None` is timed, and if 41 parents × 10⁶ births projected from it exceed 50 minutes per setting, both settings run at 10⁵ per parent with every CP interval reported; the choice is recorded before any hazard is read. **Classes and sites** (each the site its `Decreasing` operator draws over, counted on the parent): silent input entry (`Prune`; every node's `prunable_indices`), graph edge (`RemoveGraphEdge`; every edge of every Graph node, compute inputs and sink inputs), compute node (`RemoveInternalGraphNode`), plasticity config (`DisableHebbian`; compute nodes with plasticity), reward config (`DisableRewardModulation`; modulated compute nodes), mesh node (`RemoveNode`; non-entry nodes), route target (`RemoveRouteTarget`; the non-incumbent targets of nodes with two or more), VM instruction (`VmDeleteInstruction`; instructions of programs longer than one) | **Per parent and class at `None`:** h_c = applied deletions of the class ÷ (sites × births); L_c = 1 − (1 − h_c)^400. **Go** if some class has a median L_c ≥ 0.5 at `None`, the median taken over the evolved parents that carry the class (sites ≥ 1), a class counting only if at least 10 of the 40 carry it. **No-go** otherwise: the note records that deletion is not the block at a 20,000-tick horizon on `main`, lists the hazards, and S1's treatment arms do not run. **Manipulation check at 0.03**, pooled over the 41 parents per operator: (a) requested events equal to `None`'s for every parent exactly; (b) each `Decreasing` operator's kept share has a CP interval containing 0.03; (c) the plan's form: each `Decreasing` operator's applied count at 0.03 has an interval (over all births) containing 0.03 × its applied count at `None` per birth; (d) every other operator with at least 4,000 applied events at `None` (pooled) within 5 % of `None`'s count, smaller ones reported. **Amendment A1 (findings 2, 4 to 6 and 11):** (c) counts events, not births, so its interval treats the 0.03 count as Poisson (the binomial at n = births as a large-n approximation) and kept deletions change later applicability, so (c) is **descriptive** and never blocks; (b), conditional on an applied keep trial, is the mechanism invariant. (a) and the input checks are exact and block: unique parent names, exactly the founder plus 20 parents from each of the two named dumps at tick 20,000, one birth count for every parent at both settings, configs differing only in the keep field. (b) follows S0-F's disposition: a defect only at p < 0.001 (eight checks), otherwise a recorded statistical miss. Any statistical miss, (c) miss or (d) failure stops the runner before the treatment arms until its disposition is committed; a (b) defect or an exact failure blocks the arms until fixed. Per parent and class the reader reports an interval on h (applied events over site-births) and its transform to L; the medians of the lower and upper bounds are reported beside the median L. **Descriptive, not decision inputs:** the founder row; R for edges ((`AlterGraphEdgeWeight` + `RetargetGraphEdge`) ÷ `RemoveGraphEdge`, `GraphRawFieldMutation` beside) and for entries ((`Swap` + `RawFieldMutation`) per entry ÷ `Prune` per silent entry), medians over carriers; `RemoveNode` applied by target reachability beside the reachable and unreachable non-entry site counts (Advice 1 point 2: `RemoveNode` draws unreachable nodes first, so the mesh-node class's per-site average overstates what a new, spliced-in, reachable node faces; the class definition and the rule stay as the plan wrote them). **Interpretation limits, predeclared (A1, findings 12 to 14):** the mesh-node sites are an inventory count (non-entry nodes), not `RemoveNode`'s current draw set, which is the unreachable tier when one exists and otherwise the nodes with a bypass successor (that exclusion is not counted), so its h is an inventory-average hazard; h's numerator includes deletions of sites created earlier in the same birth, and L extrapolates one parent's average per-site event intensity over 400 births with sites, targeting and eligibility held fixed, so it is not a measured survival of an original site; a go establishes the triggering class's average hazard (named in the result) and licenses the treatment arms by the plan's rule, not that this class is the sensor-refinement bottleneck (a route-target or mesh-node go is read as such); a no-go is this fixed screen's negative result, not a general refutation of slow refinement |

## Ledger

One row. No authored genome enters a natural-world arm; S0's prepared parents
and dumped cohorts are measurements, not mechanisms. A killed run is
`incomplete` and counts as not persisting, never as a biological negative; an
extinction is a finding.

| ID | Question | Method, instrument, sizes | Prediction / decision rule | Status | Result |
| --- | --- | --- | --- | --- | --- |
| S1 | Does slowing deletion, and nothing else, raise executed and causal sensor use by more than trajectory noise, and what does it cost persistence and genome size? | The prototype above. **Design (the plan's):** Canyon goal recipe (`experiments/worlds/canyon-country.json`, `jq` overlays setting only `mutation.deletion_keep_probability`), seeds 1022, 2022, 3022 and 4022 (the seed is the replicate), 20,000 ticks, samples every 500, through the census probe with checkpoints 10,000 and 20,000 (selected cohort of 20 parents, founder cohort, drift empty; at both checkpoints the input funnel, the vote-delta reading, the births block, the sink-use funnel, the memory readings and E6; the cohort dump on), 3 h awake kill per run, one job at a time, `RAYON_NUM_THREADS=8`. **Arms:** `default` (`None`; also S0's parents and S0.2's candidate on seeds 1022 and 2022), `del-1.0` (the non-loss control: the same code runs and nothing is skipped), `del-0.3`, `del-0.03`. No mutation-off pairs: at `per_unit_rate` 0 no deletion is drawn, so they would be identical by construction. **Run order:** S0.2's frozen references, then `default` on 1022, 2022, 3022, 4022; after a go, the treatment arms seed by seed (`del-1.0`, `del-0.3`, `del-0.03` on 1022, then 2022, 3022, 4022). **Readings:** window means over ticks 15,000 to 20,000 (population, genome size in founder units (97), births per creature-tick, generation, size per generation) and the lowest population sample of the run with its tick; at both checkpoints the typed target-family funnel rows (`AreaFoodSummary:0`, `AreaFoodSummary:1`, `NeighborBarrierRing`: declared, connected, executed, causal; distinct parents of the selected cohort; an absent row counts 0, and in this one-food-type world `AreaFoodSummary:1` is always absent), **P = Σ over the three typed rows of `family_executed` + `family_causal`** (B1's measure; the output-wiring reader's `executed_plus_causal`), the vote-delta block's target channels whose ablation changes a committed action queue (`channels_with_action_queue_change`) and those that move any vote, the births block (dead, silent, changed), E6; the funnel's `family_causal` against the vote-delta's action-changing parents per family and channel (the W1 agreement check), reported descriptively | **Predictions (the plan's table):** P under `del-1.0` differs from `default` by trajectory noise only, `del-0.3` lies between, `del-0.03` above `default` if the hypothesis holds; vote-delta action-changing target channels: `del-1.0` noise, `del-0.3` not predicted, `del-0.03` above `default` if the hypothesis holds; mean genome size and size per generation: `del-1.0` as `default`, `del-0.3` higher, `del-0.03` highest (N2 saw the opposite, so a contradiction goes to the advisor); window-mean population: `del-1.0` as `default`, `del-0.03` at least 0.5 × `default`'s, its lowest sample recorded (N2 fell to 52); births block: `del-1.0` as `default`, the others not predicted. **Validation rule (the plan's draft rule, as written):** per seed and checkpoint t, noise N(t) = \|P(`del-1.0`) − P(`default`)\| and effect E(t) = P(`del-0.03`) − P(`default`); a seed **supports** the hypothesis if at both checkpoints E(t) ≥ 5 and E(t) > N(t) (the checkpoints of one world are dependent samples, so they must agree; seeds are what get counted). `del-0.03` is **validated as a lead** if all of: S0 was a go; `del-0.03` persists to 20,000 on all four seeds with window-mean population at least 0.5 × `default`'s on each; at least 3 of 4 seeds support; at 20,000, P(`del-0.03`) ≥ P(`del-0.3`) ≥ P(`default`) on at least 3 of 4 seeds (dose order); on each supporting seed the vote-delta block shows at least one target channel whose ablation changes an action under `del-0.03`. Otherwise **not validated**. With 20 parents per cohort this is a lead-selection screen. **Reading details, predeclared (no clause changed):** P(arm) at a checkpoint the arm did not reach is undefined, and a seed with an undefined P in any of the three arms at either checkpoint does not support; "persists to 20,000" means the run's footer reaches 20,000 not extinct, a killed run counting as not persisting; a seed whose `default` did not reach 20,000 has no defined window-mean ratio, so the persistence clause cannot hold on all four seeds and the row is not validated; the vote-delta clause is read at 20,000, the checkpoint the dose-order clause reads (W1's convention, review (a) finding 18 there); `parents_evaluated` 20 is required for a checkpoint's P to be defined; (A1, finding 8) a present typed row whose executed or causal count is unavailable makes P undefined (only an absent row counts 0), and the window means require every scheduled sample from 15,000 to 20,000 alive; (A1, finding 9) execution completeness is reported apart and is not a clause. **Interpretation limits, predeclared (A1, findings 15 and 16):** N(t) is one null-arm contrast, not an estimated noise distribution; beyond the deletion rate, a treatment arm differs from `default` by its RNG stream and, from the first founder birth on (the founder carries a silent entry, and its deletions are slowed too), by which lineages pass the founder trough (production sampled 10 living at tick 4,500 on seed 2022 in the output-wiring run), so an endpoint P gain is read beside each arm's lowest population sample and its tick, with stream perturbation and ancestry named as residual explanations; P counts parent-family incidences with executed and causal counted twice, not independent parents, and the replicate count is four seeds. **What each outcome means (the plan's):** a validated S1 makes a deletion-rate feature a candidate (T11.F28 retitled, its form, per-site decay or scale, left to the spec); an unvalidated S1, or a no-go at S0, retires T11.F28's premise on `main` | predeclared | |

## Advice

Every advisor consultation rule 8 lists, with the disposition of each point
raised.

| # | Rule 8 point | Advice (summary) | Dispositions |
| --- | --- | --- | --- |
| 1 | Before anything was built, after the contract, the branch code and the engine were read: the approach to the run | Approach sound; eight points. (1) State the keep realization as a deviation from "before it is applied", put it in the four `apply_*_event` wrappers (after `Ok`, before the parseability gate; restore the snapshot; `Err(DeletionNotKept)`; no engine-loop edit) and predeclare its invariants (`None` no new path; `Some(1.0)` stream-perturbed by one `f64` per applied `Decreasing` event; the kept-share denominator; the request drawn before the loop). (2) `RemoveNode` is two-tier (unreachable first), so the mesh-node class's averaged hazard overstates what spliced-in new nodes face: keep the class and rule, report the split by reachability. (3) Run the founder-level check against E4 and the manipulation check right after the builds, as a mechanism gate before the world chain; the go/no-go on evolved parents keeps the user's order; record it under rule 9. (4) Predeclare S0's birth count with its fallback. (5) Record the advisor-model facts (settings say `fable`, the goal says Opus 5.5, the confirmation is the user's). (6) The free bit-identity check of `default` against the output-wiring run's samples. (7) The cohort dump as a separate file with the world's reachable and executed sets, out of the census row, hashed under S0.2. (8) Turn economy: one runner script with phases, readers adapted from the output-wiring ones, the summaries folder before `make check-docs`, `npm ci` before `make check`, viability before the `proto:` commit, Codex (a) in the background after the predeclaration commit; plain git commands | All adopted: (1) the prototype section; (2) S0's descriptive readings; (3) gate S0-F and the Order; (4) S0's birth count; (5) Run setup; (6) Run setup's drift check; (7) the instrument changes and S0.2; (8) the Order and Run setup |

## Codex reviews

### Review (a): the predeclaration, before any gate or row runs

Job `.bench-artifacts/lab/delrate/codex/review-a.out.md` (brief `review-a.md`),
verdict `not-ready`, 11 blocking and 6 advisory. Every finding is adopted as
**amendment A1**, pre-data (nothing had run). The fixes sit in the sections
above, the readers, the runner and one test. No confirmation round is run:
no finding changes the row's question, its arms, the plan's validation rule
or the go rule, and every fix is a checker rule, a reader rule, a runner guard
or a predeclared wording.

| # | Finding (short) | Severity | Disposition |
| --- | --- | --- | --- |
| 1 | The chain checks neither `make check` nor the review's disposition; `treatments` can run after a no-go | blocking | Adopted: `chain` requires `make-check.done` with exit 0 and `review-a.disposed`; `treatments` requires an authorization file written only by a clean go (or by hand after a committed disposition) and `s0.json` a go |
| 2 | Reader exit 4 let any number of (c)/(d) failures launch the treatments | blocking | Adopted: exit 4 stops the runner; resumption only after a committed disposition |
| 3 | S0-F imported S0's (c) and made its tolerance failure fatal | blocking | Adopted: S0-F reads its own declared checks; tolerance failures stop for a disposition, not as defects |
| 4 | (c) counts events, not births; not an exact binomial | blocking | Adopted: (c) is descriptive with a Poisson-like interval, never blocking; (b) is the mechanism invariant |
| 5 | Containment checks can fail under a correct mechanism; fixing and rerunning until pass repeats W0's problem | blocking | Adopted: defect only at an exact two-sided p < 0.001; a statistical miss is recorded with its p-value and never rerun on unchanged evidence |
| 6 | The reader did not validate parent sets, provenance, birth counts, configs; duplicates overwrote | blocking | Adopted: exact input checks in `s0read.py` (unique names, founder + 20 + 20 from the two named dumps at 20,000, one birth count, configs differing only in the keep field; S0-F's exact parent sets) |
| 7 | `identity.py` let absent metadata compare equal and could skip the dump check | blocking | Adopted: `config_digest`, `prototypes.deletion_keep_probability`, the births `mutation_config` and `cohort_dump: true` required on both runs; the dump hash always checked |
| 8 | Unavailable causal counts read as 0; window means over missing samples | blocking | Adopted: P undefined on an unavailable count of a present row; every scheduled window sample required |
| 9 | `s1read.py` added an all-16-runs clause to the fixed rule | blocking | Adopted: completeness reported apart, from completion records; the rule reads only its clauses |
| 10 | Cached outputs reused without checking the binaries | blocking | Adopted: recipes and binaries hashed once, verified before every measurement phase; a rebuild supersedes every output first |
| 11 | The 10⁵ fallback promised intervals the outputs lack | blocking | Adopted: per-parent intervals on h and their transform to L in the S0 summary, with the medians of the bounds |
| 12 | Mesh-node sites are an inventory, not the two-tier draw set (the bypass restriction unnamed) | advisory | Adopted: labelled an inventory-average hazard, the tiers and the uncounted bypass exclusion named, unreachable counts reported |
| 13 | h includes deletions of same-birth new sites; L is an extrapolation | advisory | Adopted: stated in S0's interpretation limits |
| 14 | A route-target or mesh go licenses a sensor experiment; a no-go is not a general refutation | advisory | Adopted: the triggering class named and the readings restricted, in S0's interpretation limits and the plan forward |
| 15 | The founder trough and lineage filtering are unnamed alternatives | advisory | Adopted: named in S1's interpretation limits; P read beside the lowest sample and its tick |
| 16 | N is one contrast; stream perturbation and ancestry remain; P double-counts | advisory | Adopted: stated in S1's interpretation limits |
| 17 | The 0.5 test uses a normal band, not CP; rollback coverage per domain | advisory | Adopted in part: the normal 99.9 % band is a declared deviation from the plan's CP wording (n in the thousands); the rollback test now requires the domains of the not-kept single-event births to equal the domains that applied a `Decreasing` operator in single-event births at `None` on the same seeds. On the founder these are Graph and InputRef: it has no topology or VM deletion site, so those two wrappers, which run the same three lines, are not exercised on the founder. The kept path's parseability gate is the unchanged production call and gets no new fixture |

## Order

1. Predeclaration committed; Codex review (a) launched in the background.
2. The frozen instrument built and hashed (detached nested worktree), the
   prototype binaries built and hashed, `make check` on the branch (its exit
   gates the driver).
3. Review (a) disposed and any amendment committed.
4. S0-F (minutes; must pass).
5. S0.2: the frozen references `default` on seeds 1022 and 2022 (about 25
   minutes), then the prototype binary's `default` on 1022 and 2022 with the
   identity check, then its `default` on 3022 and 4022. The drift check
   against the output-wiring samples runs beside.
6. S0: the timing smoke, then `None` and 0.03, with the go/no-go and the
   manipulation check read by the reader.
7. On a go with the manipulation check passed, the twelve treatment runs seed
   by seed. On a no-go, nothing more runs.
8. The S1 reading with its consultation.
9. The plan forward, with consultations before and after the draft.
10. Codex review (b).
11. Close in the run 1 order: docs only land on `main`, the branch is kept.

The driver stops itself on an S0-F, S0.2 or manipulation failure, or on a
no-go, before the next step.

## Plan forward

Written after the row is read.

## Cost

Recorded at closing.
