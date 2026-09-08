# T03.F10 — Activity-Ramped Compute Cost

**Status**: In Progress
**Last updated**: 2026-09-07
**Feature**: T03.F10
**Track**: [T03 — Functional Traits and Metabolism](../../roadmaps/t03-functional-traits-and-metabolism.md)

## Goal

A brain pays energy for every VM instruction it runs. Ordinary programs and
short bounded loops stay nearly free; past a free allowance the cost of each
further step within one node dispatch rises linearly with its step index, so
a run to the `max_vm_steps` cap costs a lethal fraction of a creature's
energy, and the existing exhausted-energy path (queue discarded, `NoOp`,
death at zero energy) removes the looping creature. Natural analog: neural
activity is metabolically expensive in proportion to what fires, and
sustained activity beyond a cell's local supply draws on the body at rising
cost. The ramp reaches creatures through the same energy accounting that
governs eating, decay, and reproduction; there is no sensor, no score, and no
bonus.

## Non-Goals

- No cost on program length, node count, reachable structure, or junk
  (T03.F08 and T11.F13); no size cap, pruning, or bonus for short programs.
- No change to `max_vm_steps` (it stays the per-tick compute bound), to the
  opcode base cost table, to `opcode_cost_multiplier`, or to graph-node,
  plasticity, or reward-learning costs.
- No cost across node dispatches: the hop cap and the single-visit rule
  already bound the chain, so the ramp resets at every dispatch.
- No new work counter, telemetry field, inspector annotation, or sensor. A
  cap-hit counter was considered and is not added: the goal profile's
  `vm_steps` reading is the instrument, and the 2026-09-07 probe method in
  the T11.F17 spec remains available for a one-off diagnosis.
- No change to the neighborhood battery, the drift walk, or their versions;
  they execute production code and read what it does.

## Inputs and Invariants

- Source intent: the owning track's T03.F10 row and detailed note
  (evidence: the T11.F17 goal run's seed 33, 12.05% of 11,379 creatures at
  the 10,000-step cap holding 97.76% of the tick's VM steps; `vm_steps`
  146.184594 per creature-tick, +132.76% severe, accepted post-observation
  and re-pinned as the goal epoch); the
  [T11.F17 spec](t11-f17-executed-biased-mutation-targeting.md) Performance
  and Goal Impact section for the probe readings (control seed 11: p50 25,
  p99 28, max 30 steps per creature; seed 33: p50 39, p90 10,000).
- Existing seams: `execute_vm_node_impl` in `crates/v3-core/src/runtime/vm.rs`
  (the single opcode loop, shared by the traced executor through
  `VmTraceSink`; `steps`, `max_steps`, `cost_mult`, the pre-execution
  exhaustion check, `SetPriorityBid`'s bid deduction, and the five exit
  paths: step cap, pc past program, `Halt`, `ExecuteActionQueue`,
  exhaustion); `VmRuntimeConfig` and `SimulationConfig::normalize` in
  `config/simulation.rs`; `execute_creature_mesh_impl` in `runtime/mesh.rs`
  (attributes each dispatch's energy delta to a backend and turns
  `energy_exhausted` into a `NoOp` output); death removal at `energy <= 0`
  in `simulation/tick.rs`; the runtime config patch path and its frontend
  panel (`RuntimeSection.tsx`, `types/config.ts`, `test/fixtures.ts`,
  `ControlBar.test.tsx`); `docs/reference/v3-runtime-config-spec.md`
  Section 2 and `docs/reference/v3-vm-isa-spec.md` Sections 3 and 6.
- Why cognition is free today: at `opcode_cost_multiplier` 1e-6 a step costs
  about 1e-7 energy and is subtracted from the creature's `f32` energy one
  step at a time; the ulp of an `f32` near 20 is about 1.9e-6, so every
  per-step subtraction rounds to nothing for any creature above about 16
  energy, and a full 10,000-step dispatch nominally costs 0.01.

Research, 2026-09-07, on the activity-cost literature:

- Linear part. Attwell and Laughlin (2001, *J Cereb Blood Flow Metab* 21:
  1133–1145) budget grey-matter signaling per action potential and per
  vesicle: cost is proportional to activity, with about 47% of signaling
  energy on action potentials and 34% on postsynaptic effects at a mean
  rate of 4 Hz. Lennie (2003, *Curr Biol* 13: 493–497) derives from that
  budget that the available ATP supports an average of about 0.16 spikes per
  second per neuron, so fewer than 1% of cortical neurons can be
  substantially active at once. Harris, Jolivet, and Attwell (2012, *Neuron*
  75: 762–777) review supply and use at synapses. These support a cost per
  executed step that is linear in steps: the base opcode table already
  encodes it, and this feature keeps it.
- Superlinear part. Lewis, Gilmour, Moorhead, Perry, and Markham (2014,
  *J Neurosci* 34: 197–201) measured whole-organism oxygen consumption in
  *Eigenmannia* and found that ATP cost per action potential rises
  nonlinearly with firing rate over 200–600 Hz, attributed to the rising
  cost of holding spike amplitude at high rates; they caution that
  per-spike estimates from sodium influx hold at low baseline rates and
  undercount at high rates. Joos, Markham, Lewis, and Morris (2018,
  *PLoS ONE* 13: e0196508) model the same cells: cost per spike from
  voltage-gated sodium current rises nonlinearly with frequency while the
  synaptic-current cost per spike stays constant, and depolarization block
  follows when ATP depletion outruns supply. Measured superlinearity exists
  for sustained high-rate activity; the specific linear ramp below is
  engineering on that analog, chosen for a closed-form total, not a curve
  taken from nature.

Fixed design, decided before implementation:

| Decision | Value |
| --- | --- |
| Charge | The k-th instruction executed within one VM node dispatch (k from 1) costs `opcode_base_cost(instr) * runtime.vm.opcode_cost_multiplier + runtime.vm.step_ramp_cost * max(0, k - runtime.vm.step_ramp_allowance)`. The base term is unchanged; the ramp term is new. |
| Allowance | `runtime.vm.step_ramp_allowance`, `u32`, default 100. Covers the control seed's p99 of 28 steps, seed 33's median of 39, and a pass over the 16 shared-memory slots with room for a bounded loop. Any `u32` is valid; 0 ramps from the first step. |
| Ramp rate | `runtime.vm.step_ramp_cost`, `f32`, default 1e-6 energy per step per excess step. Finite and non-negative; invalid values fall back to 1e-6; 0.0 disables the ramp. Closed form for n executed steps with m = n − allowance > 0: ramp total = 1e-6 · m(m+1)/2. Defaults give 200 steps 0.00505, 1,000 steps 0.405 (about 0.8 ticks of the 0.5 per-tick decay), 10,000 steps 49.0 energy (initial energy 20, reproduction at 30, maximum 200). |
| Accumulation | The dispatch keeps its opcode and ramp charges in a local accumulator and subtracts the sum from the creature's energy exactly once, at whichever exit path ends the dispatch. The accumulator preserves sub-ulp early steps, and settlement rounds once instead of once per step. The trace sink's `energy_after` reports the effective remaining energy (`energy - accumulator`) and `energy_cost` the step's own charge, so the inspector stays truthful. |
| Exhaustion | Before executing the k-th instruction, add its charge to the accumulator; if `energy - accumulator <= 0.0`, subtract the accumulator (energy goes to zero or below), do not execute the instruction, do not commit shared memory, and return `NodeResult::exhausted()`. This is the existing rule with the accumulator in place of the running subtraction. |
| `SetPriorityBid` | The bid is capped at the effective energy, `raw.min(energy - accumulator)`, and is still subtracted from energy immediately (the mesh reads it this tick); the post-bid exhaustion check uses the same effective-energy rule and the same debt settlement. A bid can never spend energy the dispatch already owes. |
| Energy input | `ReadInput` of `EnergyCurrent` resolves to the effective energy (`energy - accumulator`), and `EnergyConsumedThisTick` to the tick's consumption so far plus the accumulator, so a brain reads its own starvation mid-dispatch through the existing introspection inputs; no new sensor. |
| Empty dispatches | `register_count == 0` and empty programs execute no step and charge nothing, unchanged. |
| Scope | VM backend only, per dispatch; `steps` is the ramp index. `max_vm_steps` remains the hard bound; at defaults a capped dispatch costs about 49 energy, so energy binds first for any creature below that and a creature at the 200 maximum survives at most four capped ticks. |
| Determinism | Production behavior changes for every creature (accumulated subtraction now lands where per-step subtraction rounded away), so no gate or goal `deterministic` block is predeclared identical to any prior report, including the founder neighborhood block. Cross-process reproducibility is unchanged in kind and remains covered by `crates/v3-core/tests/reproducibility.rs`. |
| Frontend | `VmConfig` gains the two fields; the runtime panel gains two rows (allowance 0–10,000 step 1; ramp cost 0–1 step 0.000001); fixtures and the control-bar fixture carry them. |
| Reference docs | `v3-runtime-config-spec.md` Section 2 gains the two rows; `v3-vm-isa-spec.md` Section 3 names the per-dispatch ramp and Section 6 states the per-step formula. |

Predeclared readings, taken from the stored closure reports and read against
the T11.F17 gate and goal reports (previous closure and, for the goal
profile, the pinned epoch) and the gate epoch `remove-complementary-nutrition`:

| Reading | Reference | Predeclaration |
| --- | --- | --- |
| Goal `vm_steps` per creature-tick | 146.184594 (T11.F17 goal, the pinned epoch) | Down. The cap-running tail is the mechanism this feature removes; the control seed read about 24 per creature-tick. |
| Goal `mesh_hops`, `graph_relax_iters`, `plasticity_updates`, `actions_applied`, `births` | T11.F17 goal | Reported under the usual thresholds (work flag +10%, severe +50%); no severe budgeted. Ecology moves, so ok/flag readings are expected and are read as ecology, not compute. |
| Gate counters and wall time | T11.F17 gate and the gate epoch | No severe; wall flags/severe at +25%/+100% on a matching host. No epoch re-pin budgeted. |
| Persistence (goal profile, 1600², 10,000 founders, seeds 11/22/33, 2,000 ticks) | T11.F17 goal: final 714 / 669 / 11,379, minimum 602 / 605 / 1,371; T11.F04 `w1600` sweep: final 11,610 / 10,398 / 11,093 | No seed goes extinct; final, minimum, and plateau populations reported beside both references. Seed 33's bloom was carried by cap-running creatures, so its population is expected to fall toward the other seeds; that is the predicted direction, not a regression. |
| Drift walk changed/all births at 1,000 / 2,000 (track floors) | 0.001500 / 0.008000 (T11.F17) | Not below (strict rule). The drift walk has no energy budget but its battery runs production code at energies 5–80, so a genome that loops to the cap now exhausts at five of the six battery energies (5, 15, 25, 31, 45; not 80) and a mutation that creates or breaks such a loop reads as changed or dead rather than silent; direction of the changed fraction is otherwise unpredicted. |
| Drift walk dead/all births pooled at 1,000 and 2,000 | 8 / 4,000 (T11.F17) | Not above 20 / 4,000, a count allowance predeclared before measurement because loop-creating mutations now classify dead (all-`NoOp`); any reading above 8 is reported as a count with that attribution stated. |
| Drift walk hop-cap hits, executed nodes, total nodes | T11.F17 | Hop-cap hits zero; executed and total nodes reported, no direction. |
| Goal evolved half changed / dead per mutated birth | 0.288182 / 0.013030 (T11.F17) | Reported; no direction. The evolved population itself changes under the new cost, so the reading is confounded (T11.F17's caveat applies). |
| Observation budgets | Workflow caps | Founder neighborhood below 10 s; summed evolved neighborhood below 180 s; drift walk below 30 s; whole goal run below 15 minutes. |

## Implementation Tasks

- [x] Add `step_ramp_allowance` and `step_ramp_cost` to `VmRuntimeConfig`
      with serde defaults, normalization, config-spec rows, and the frontend
      types, fixtures, panel rows, and tests, failing tests first.
      (`f205092e`)
- [x] Rewrite the charge in `execute_vm_node_impl` as the per-dispatch
      accumulator with the ramp term, settled once on every exit path, with
      the exhaustion and `SetPriorityBid` rules above; tests first, including
      the trace sink's effective-energy reporting. (`2323831a`, simplified in
      `7debaaa7`, trace and consumption coverage in `e8ca7523`)
- [x] Property tests for the pure charge: the ramp total equals the closed
      form within tolerance for any allowance and step count; total charge is
      monotone non-decreasing in executed steps and in ramp rate; with ramp
      0.0 the total equals the sum of base costs; the accumulator matches the closed form plus the summed base costs
      directly, and the mesh-attributed node cost equals the settled total
      within one ulp of the starting energy unless exhausted.
      (`crates/v3-core/src/runtime/tests/vm_step_ramp.rs`; no
      `proptest-regressions` file survived the final suite)
- [x] Update `v3-vm-isa-spec.md` Sections 3 and 6, `docs/progress.md`, and
      `docs/progress/benchmark-series.json`; store the gate and goal reports.

## Verification

- [x] `cargo test -p v3-core --test viability` first after the charge lands;
      `cargo check --workspace --all-targets` after coherent Rust edits;
      focused suites `cargo test -p v3-core --lib runtime`,
      `cargo test -p v3-core --lib config`, `cargo test -p v3-cli --lib
      bench::tests`, and `npm --prefix frontend test -- --run` pass.
- [x] Unit tests at `opcode_cost_multiplier` 1.0 and `step_ramp_cost` 1.0:
      a dispatch below the allowance charges base costs only; the first step
      past the allowance charges base plus 1.0 and the m-th excess step base
      plus m; a capped dispatch settles the closed-form total once; a
      dispatch that exhausts mid-loop leaves energy at or below zero, does
      not commit shared memory, and returns exhausted; a `SetPriorityBid`
      bid is capped at effective energy; the traced executor reports the
      same charges and effective energy as the untraced one.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after the simplify pass;
      record the summary line, output path, and every survivor's resolution.
- [x] `make bench PROFILE=gate FEATURE=t03-f10-activity-ramped-compute-cost`
      stores `docs/progress/features/t03-f10-activity-ramped-compute-cost.json`;
      one `make bench PROFILE=goal FEATURE=t03-f10-activity-ramped-compute-cost`
      stores the `-goal.json` report. Record every predeclared reading above,
      the observation budgets, and the compute comparisons.
- [x] Second goal run: Not applicable by the 2026-09-05 workflow decision;
      `crates/v3-core/tests/reproducibility.rs` covers cross-process
      reproducibility inside `make check`.
- [x] `make roadmap-check` on document edits; final `make check` exits 0 on
      the closure content, with the tested commit reported in the parent task.

### Results, 2026-09-07

Commands run in the worktree and their results:

- `cargo test -p v3-core --test viability` — ok, 24 passed, 0 failed, run
  first after the charge landed and again after the simplify pass.
- `cargo check --workspace --all-targets` — clean, run after every coherent
  Rust edit through the compile hook.
- `cargo test -p v3-core --lib runtime` — ok, 209 passed, 0 failed.
- `cargo test -p v3-core --lib config` — ok, 129 passed, 0 failed.
- `cargo test -p v3-core --lib` — ok, 1,228 passed, 0 failed, 1 ignored
  (pre-existing).
- `cargo test -p v3-cli --lib bench::tests` — ok, 37 passed, 0 failed.
- `npm --prefix frontend test -- --run` — 58 files, 308 tests passed.
- `cargo clippy -p v3-core --all-targets` — clean, no warnings.
- `make roadmap-check` — `roadmap-check: validation passed`, exit 0.
- `make check` — exit 0 at `9f1be0d2` on the closure content (Rust format,
  viability, the whole workspace test suite including
  `crates/v3-core/tests/reproducibility.rs`, Clippy, frontend lint, tests, and
  build). The frontend lint step prints three pre-existing
  `lint/suspicious/noArrayIndexKey` warnings in
  `config-panel/startup/FoodTypesSection.tsx` and `FertilitySection.tsx`, files
  this feature does not touch (last changed 2026-08-29); they are warnings and
  do not fail the step. An earlier `make check` at `fca399d2` exited 2 on a
  formatter error in this feature's own `RuntimeSection.tsx` rows, fixed in
  `9f1be0d2`.
- `make bench PROFILE=gate FEATURE=t03-f10-activity-ramped-compute-cost` —
  exit 0, report stored, `severe=false` against both references.
- `make bench PROFILE=goal FEATURE=t03-f10-activity-ramped-compute-cost` —
  exit 0, report stored, `severe=false`. Run once, per the 2026-09-05
  decision.

Unit tests, in `crates/v3-core/src/runtime/tests/vm_step_ramp.rs`: the m-th
step past the allowance charges base plus m at allowance 3 and at allowance 0;
a dispatch the ramp does not reach (inside the allowance, or ramp 0.0) charges
base costs only; a capped dispatch settles the closed-form total once; at
production defaults a 10,000-step dispatch charges 48.9–49.2, a 1,000-step
dispatch 0.40–0.41, 200 Noops from energy 20 charge the closed form to within
1e-6 (per-step subtraction would charge nothing), and 20 steps charge under
1e-5; a mid-loop exhaustion leaves energy at or below zero, arrives before the
step cap, does not commit shared memory, and returns exhausted; an oversized
`SetPriorityBid` is capped at the effective energy and exhausts, while a bid
inside it is paid in full; the traced executor reports each step's effective
energy and the bid inside its own step cost, and settles the same energy,
memory, and step count as the untraced one; `ReadInput` of `EnergyCurrent` and
`EnergyConsumedThisTick` read the effective energy and the accumulator
mid-dispatch.

Property tests for the pure charge (`step_charge`): summing per-step charges
reproduces the closed form for any step count, allowance, ramp rate, and cost
multiplier; the total is monotone non-decreasing in steps and in ramp rate and
never negative; a step inside the allowance carries the base charge exactly;
and a whole settled dispatch charges the closed form within one ulp of the
starting energy plus one of the settled debt. No assertion depends on which
cases were drawn. One `proptest-regressions` entry exists and is committed at
`crates/v3-core/proptest-regressions/runtime/tests/vm_step_ramp.txt`; it
records a first-draft generator that drew ramps a dispatch could not afford
(`steps = 188, allowance = 0, ramp = 0.0094774915, energy = 20.0`), which
tripped the harness assertion that the dispatch does not exhaust rather than
the invariant. The generator is now bounded to ramps the drawn energy can
absorb and the seed replays green.

Two pre-existing tests were loosened, both in
`crates/v3-core/src/runtime/tests/vm_execution.rs`, with the reason recorded
in place: `priority_bid_capped_at_available_energy` and
`oversized_bid_produces_exact_all_in_exhaustion` asserted that a capped bid
lands energy on exactly `0.0`. Under the single settlement the bid cap and the
debt subtraction round separately, so an all-in lands within one ulp of zero
(measured -1.2218952e-6 from a starting energy of 100, against an ulp of
7.6e-6). Both now assert `e.abs() <= f32::EPSILON * starting_energy`;
the exhaustion assertion is unchanged and still holds, because the code treats
a bid the cap bit into as an all-in regardless of which way the `f32` store
rounded. Reference doc `v3-vm-isa-spec.md` Section 9 was corrected by one
clause to match (bids cap at the effective energy).

Mutation testing, fresh (`MUTANTS_ITERATE=0 make rust-mutants`, run after the
simplify pass, diff against merge base `8b27c813`):

- First run summary line: `35 mutants tested in 4m: 4 missed, 30 caught,
  1 unviable`.
- Second run, after the two tests below were added: `35 mutants tested in 3m:
  1 missed, 33 caught, 1 unviable`.
- Output path (both runs):
  `/Users/istefanek/.local/share/petri-tools/mutants/t03-f10/mutants.out`.
- No mutant timed out, and no `#[mutants::skip]` or `exclude_re` entry was
  added anywhere in this feature.

Survivors and their resolutions:

- `crates/v3-core/src/runtime/vm.rs:382:58: replace + with * in
  execute_vm_node_impl` (`energy_consumed + debt as f32`) — **killed** by
  `a_mid_dispatch_consumption_read_includes_what_the_dispatch_owes`.
- `crates/v3-core/src/runtime/vm.rs:445:34: replace += with -= in
  execute_vm_node_impl` and the `*=` variant at the same site
  (`step_energy_cost += bid as f32`, the trace sink's per-step cost) —
  **killed** by `the_traced_bid_step_reports_the_bid_inside_its_energy_cost`.
- `crates/v3-core/src/runtime/vm.rs:439:34: replace > with >= in
  execute_vm_node_impl` (`if raw > 0.0` in the bid guard) — **equivalent**.
  The only register value the two guards treat differently is a zero: at
  `raw == 0.0` the `>=` branch computes `0.0f64.min(effective)`, which is
  `0.0` because `effective > 0.0` is already checked, so the bid, the energy
  subtraction, the all-in test, and the recorded `priority_bid` are all
  identical; a negative zero differs only in the sign of a zero, which no
  comparison, sum, or ordering downstream distinguishes.

## Performance and Goal Impact

Natural analog: activity-proportional metabolic cost with a rising cost for
sustained activity beyond local supply (the literature above). It reaches
creatures through the body: the charge is energy, settled through the same
accounting as decay, eating, and reproduction, and selection sees it only as
starvation and death. No feature-specific sensor, reward, or authored script.

Predeclared cost: one multiply-add and one comparison per executed VM step
and one subtraction per dispatch, replacing one subtraction per step. Wall
time per creature-tick is expected flat or lower, since the cap-running tail
that held 97.76% of seed 33's VM steps is removed by selection. No severe
compute allowance and no epoch re-pin are budgeted for either profile. The
goal profile's `vm_steps` is predeclared down; a reading that is not below
the epoch's 146.184594 is investigated before closure, since it would mean
the ramp did not bind where the probe said it would. The gate and goal
`deterministic` blocks differ from every prior report for every seed,
including the founder neighborhood block, because the settled charge now
lands on energies where the per-step charge rounded away.

Every mutational-neighborhood and drift-depth reading is taken with the
production charge in force, because the battery executes production code.
That is deliberate: a loop that now starves its creature is a behavior
change, and the indicator should say so. The drift floors are read under the
strict not-below rule; a miss is investigated and resolved before closure,
never waived.

### Measured, 2026-09-07

Reports, both stored on this branch and produced with production code
unchanged since `e8ca7523`: gate
`docs/progress/features/t03-f10-activity-ramped-compute-cost.json`
(`make bench PROFILE=gate` exit 0, `severe=false`) and goal
`docs/progress/features/t03-f10-activity-ramped-compute-cost-goal.json`
(`make bench PROFILE=goal` exit 0, `severe=false`). Both carry
`git_revision` `e8ca7523ce39b46852f93925b8707d53aede0ef0`. T11.F17's goal
report is both the previous closure and the pinned epoch, so every goal delta
below is against one reference.

Predeclared readings:

| Reading | Reference | Predeclaration | Measured | Result |
| --- | --- | --- | --- | --- |
| Goal `vm_steps` per creature-tick | 146.184594 (pinned epoch) | Down | 22.566373 (-84.563098%) | Met |
| Goal `mesh_hops` / `graph_relax_iters` / `plasticity_updates` / `actions_applied` / `births` | T11.F17 goal | No severe; ok or flag read as ecology | 2.072588 (+0.066096%, ok); 0.999001 (-0.361749%, ok); 0.033524 (+14.623722%, flag); 1.234813 (-11.857434%, ok); 0.015597 (-5.225740%, ok) | Met; `severe=false` |
| Gate counters and wall time | T11.F17 gate and the gate epoch | No severe; wall flags/severe at +25%/+100% | All six ok against both references (`vm_steps` 22.494483, -7.173819% vs T11.F17 and -1.128871% vs the epoch; `plasticity_updates` 0.007356, -34.150927% vs the epoch and -1.710315% vs T11.F17); wall 0.0013164589 ms/creature-tick, -10.374609% vs T11.F17 and -15.602470% vs the epoch, ok | Met; no epoch re-pin |
| Persistence, goal profile, seeds 11/22/33 | T11.F17 final 714/669/11,379, minimum 602/605/1,371; T11.F04 `w1600` final 11,610/10,398/11,093 | No seed extinct; final, minimum, plateau reported | No extinction on any seed. Final 1,786 / 3,712 / 1,306; minimum 959 / 1,366 / 536; plateau 1,449.400000 / 2,830.522000 / 900.308000 (T11.F17 plateau 765.398000 / 704.260000 / 7,286.532000) | Met; seed 33 fell from 11,379 toward the other seeds as predicted, and seeds 11 and 22 rose |
| Drift changed/all births at 1,000 / 2,000 (floors) | 0.001500 / 0.008000 | Not below (strict) | 0.001500 / 0.008000 (3 / 2,000 and 16 / 2,000) | Met, equal to the floors |
| Drift dead/all births pooled at 1,000 and 2,000 | 8 / 4,000 | Not above 20 / 4,000 | 8 / 4,000 (4 and 4) | Met |
| Drift hop-cap hits, executed nodes, total nodes | T11.F17 | Hop-cap hits zero; nodes reported, no direction | Hop-cap hits 0 at depths 0, 22, 250, 1,000, 2,000. Mean executed nodes 2.000000 / 2.260000 / 2.880000 / 4.280000 / 4.860000 (T11.F17 …/4.260000/4.760000); mean total nodes at 2,000 143.760000 (T11.F17 143.320000) | Met |
| Goal evolved half changed / dead per mutated birth | 0.288182 / 0.013030 | Reported; no direction | 967 / 3,300 = 0.293030 and 57 / 3,300 = 0.017273 | Reported |
| Observation budgets | Workflow caps | Founder below 10 s; summed evolved below 180 s; drift below 30 s; goal run below 15 min | Founder 0.061 s (gate 0.052 s); summed evolved 0.408 s; drift walk 3.382 s; whole goal run 296.2 s | Met |

The drift walk moved in only one row against T11.F17: depth 250 changed
17 / 2,000 (0.008500) against 13 / 2,000 (0.006500), with silent 877 against
881. Depths 0, 22, 1,000, and 2,000 have identical birth classifications.
The spec predicted the battery would reclassify loop-creating mutations,
and one checkpoint did; the walk's genomes mostly execute programs far shorter
than the 100-step allowance, so the ramp reaches almost none of them. The two
floors are met exactly rather than moved.

The goal evolved half is read as the confound the spec predeclared, not as a
regression: the populations behind the three samples are different populations
(seed 33's cap-running bloom is gone, and the median sampled generation moved
from 38/41/58 to 54/17/48). The dead fraction rose from 0.013030 to 0.017273,
a difference of 14 births in 3,300; the drift walk, which holds founder,
seeds, battery, and birth subset fixed, shows the pooled dead count unchanged
at 8 / 4,000, which is the controlled reading of the same question.

Other goal indicators, reported without predeclaration: surviving founder
clades 152 / 126 / 136 with Shannon entropy 1.692196 / 1.628665 / 1.649503
(T11.F17 161 / 146 / 54 with 2.585000 / 2.798111 / 0.158688 — seed 33's
near-monoculture is gone); reachable structure mean 88.249706, median 74,
max 367 (T11.F17 118.863266 / 114 / 428); mean energy 37.870462 / 60.418919 /
50.473631 (T11.F17 62.444766 / 63.259692 / 25.355656). No cognition claim.

The `deterministic` blocks differ from every prior report for every seed, as
predeclared, with one observed exception worth recording: the gate and goal
founder neighborhood blocks are identical to T11.F17's (208 applied, 95
changed, 113 silent, 0 dead), because the founder's programs run far inside
the 100-step allowance and its battery never reaches the ramp. Nothing was
predeclared identical, so this is a reported observation, not a criterion.

Wall time per creature-tick fell on both profiles (gate -10.37% against
T11.F17, -15.60% against the gate epoch; goal -11.51% against the pinned
epoch), consistent with the predeclaration that removing the cap-running tail
would leave wall time flat or lower. Neither profile shows a severe compute
regression and no epoch re-pin is requested.

## Success Criteria

- [x] At production defaults a VM dispatch that runs to the step cap charges
      about 49 energy, a 1,000-step dispatch about 0.4, and a dispatch within
      the allowance only its base opcode costs, settled once per dispatch,
      with exhaustion mid-dispatch discarding the queue and shared-memory
      commit as before.
- [x] The stored goal report reads `vm_steps` below the pinned epoch, every
      seed persists, the drift floors hold, and neither profile shows a
      severe unbudgeted compute regression.
- [ ] Required checks, fresh mutation evidence, independent review, reference
      spec updates, and closure records are complete; the feature row is
      checked and this spec is Complete on main.

## Notes for AI Agents

- Planning base: `8b27c813` (T11.F17 closed, T03.F08 re-scoped); worktree
  `.claude/worktrees/t03-f10`, branch `worktree-t03-f10`. Track promoted
  Planned to In Progress; master already Active.
- Roles: Fable 5.1 `high` orchestrator; Opus 5 implementer with the Fable
  advisor; Fable 5.1 reviewer.
- Readiness self-review, 2026-09-07, with one advisor consult during
  planning (count 1; the T11.F17 precedent): 0 P1, 4 P2, 2 P3; Ready after
  one revision. The P2s were an unverified author list on the Joos 2018
  citation (fetched and corrected), no rule for what `ReadInput` returns for
  the creature's own energy mid-dispatch (now the effective energy), a
  property test that asserted an exact `f32` difference the single settlement
  cannot honor (reworded to the accumulator and a one-ulp bound), and two
  wording errors in the predeclarations (the battery exhausts at five of six
  energies, and energy binds before the cap only below about 49). Runtime
  behavior is not verified by this author review; the fresh final review is
  separate.
- Sequencing note from the track: this feature is not a dependency of
  T11.F17 and did not gate its closure; T03.F08 later reads this feature's
  realized execution cost beside the 2026-09-07 figure (0.000152 energy per
  creature-tick on the user's long run).
- Deviation from the letter of the fixed-design table, for the orchestrator to
  accept or amend (the table is left as decided): the post-bid exhaustion
  check is `bid >= effective || effective_energy!() <= 0.0`, not the
  effective-energy clause alone. Measured reason: when the cap bites, the bid
  is `effective` and the separate `f32` stores of `energy - bid` and the debt
  settlement leave a residual of about one ulp of the starting energy in
  either direction (-1.2218952e-6 measured from 100 energy). Without the first
  clause, whether an all-in bid exhausts would depend on which way that store
  rounded. The clause makes an all-in bid exhaust deterministically, which is
  the pre-feature behavior; `v3-vm-isa-spec.md` Section 9 documents the
  implemented rule.
- Also outside the task's named scope, and reported rather than assumed:
  `v3-vm-isa-spec.md` Section 9 gained a one-clause truthfulness fix (bids cap
  at the effective energy), since the task named Sections 3 and 6 only. Section
  3's stale `max_vm_steps` default of `1024` (the config default is 10,000) is
  a pre-existing error left untouched.
- Deferred finding: none from the mutation run. The one survivor is argued
  equivalent above, not deferred.
