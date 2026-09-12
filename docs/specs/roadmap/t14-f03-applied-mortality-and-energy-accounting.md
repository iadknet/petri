# T14.F03 — Applied Mortality and Energy Accounting

**Status**: In Progress
**Last updated**: 2026-09-12
**Feature**: T14.F03
**Track**: [T14 — Runtime Telemetry and Report Integrity](../../roadmaps/t14-runtime-telemetry-and-report-integrity.md)

## Goal

Each standard goal world's terminal report counts creature deaths by the sink
that exhausted them and totals the energy flows actually applied during the
run. Later closures can read compute lethality and genome carrying exposure
from applied behavior without reconstructing them from configured rates.

## Non-Goals

- Change any charge, multiplier, rate, default, RNG draw, action order, energy
  arithmetic, recovery opportunity, or removal timing.
- Add environmental pressure, a diversity/cognition indicator, intervention,
  conservation model, per-creature event history, dashboard, or server payload.
- Implement T14.F06's reproductive classes or other telemetry rows. Initial
  founder energy is starting stock, not a new intake counter.
- Rewrite historical reports or treat an absent measurement as zero.

## Inputs and Invariants

Sources: the owning F03 row and note; the
[T14.F02 contract](t14-f02-existing-counter-transfer.md); `SimStats` in
`crates/v3-core/src/simulation/stats.rs`; `Simulation::remove_creature` in
`simulation/simulation.rs`; the phase loops and failed-action debit in
`simulation/tick.rs`; `simulation/actions/{mod,reproduction,predation}.rs`;
`runtime/{vm,mesh,types}.rs` and `runtime/cgp/execute.rs`; and `WorldTracking`
in `crates/v3-cli/src/bench.rs`. Core paths above are relative to
`crates/v3-core/src/`. These sites are verified directly during planning;
audit shorthand is not an additional requirement.

Research (2026-09-12): extend the existing fixed runtime counters, mesh side
outputs, sequential reduction and optional terminal report blocks. The credible
alternative is an event ledger with later aggregation; its storage and replay
machinery are unnecessary for fixed counts and sums. Integer counts plus
ordered `f64` accumulation retain the existing arithmetic; fixed-point
quantization would discard small applied flows. Rust's
[floating-point guidance](https://doc.rust-lang.org/std/primitive.f64.html)
and [Serde's field attributes](https://serde.rs/field-attrs.html) support the
existing explicit-order and missing-block conventions. No dependency is needed.

**Mortality definition and precedence.** `deaths_total` counts each successful
removal of a creature once, including externally removed creatures under an
explicit cause. The sole increment and cause tally belong to
`Simulation::remove_creature`; a repeated removal of an absent ID changes
neither. Cause records a sink's positive-to-nonpositive crossing, not the phase
that finally removed the creature. Maintain at most one pending cause per
creature: record the first crossing while energy remains nonpositive, retain it
through later costs and floors, and clear it if an applied credit restores
positive energy. Constructors/newborns start without a pending cause. This
state is observation only and never controls survival or execution. Copy a
dispatch's pending cause onto its own creature before Phase 2 starts, so an
earlier predator can observe it before the victim's reduction turn. This
private-state write is allowed in cognition; global totals remain sequential.
Within VM execution, crossings use the existing effective-energy checks, not
the temporary debt-backed energy store or the all-in rounding pin.

The complete stable cause-key set is:

| Keys | Exhausting event |
| --- | --- |
| `lifecycle_decay`, `genome_carrying` | Phase 0 combined debit, using the convention below |
| `vm_compute` | VM effective-energy exhaustion from opcode/ramp debt, including the instruction whose side effects never execute |
| `graph_compute` | Graph evaluation base debit |
| `hebbian_learning`, `reward_learning` | The corresponding applied learning debit |
| `priority_bid` | A VM bid payment exhausts effective energy after the current instruction's debt |
| `action_noop`, `action_eat`, `action_move`, `action_reproduce`, `action_steal_energy` | The corresponding base action debit after the existing complexity/age multipliers |
| `failed_action_penalty` | The separately applied, tick-ramped and multiplier-adjusted failure debit |
| `parental_transfer` | A successful birth transfers the parent's remaining energy |
| `predation` | An applied predation transfer takes its energy-losing participant from positive to nonpositive |
| `external_removal` | Paint/manual removal of a creature with positive energy; this is not evidence of an ecological energy death |
| `unattributed` | Removal at nonpositive energy with no observed crossing, such as an externally injected dead creature; never a substitute for an instrumented production sink |

Phase 0 currently subtracts `decay + carry_rate * cached_genome_size` **once**.
Keep that exact production expression and subtraction. For attribution only,
evaluate the existing `f32` expression `energy_before - decay`: when the actual
combined debit crosses zero, attribute to `lifecycle_decay` if this decay-only
result is nonpositive, otherwise `genome_carrying`. This is an explicit
decay-first allocation convention, not proof that carrying was the sole
counterfactual cause. An existing pending cause takes precedence when Phase 0
starts with nonpositive energy.

Two existing paths need particular care. Predation directly removes the victim
from the world, slotmap and action logs today; join it to the common removal
funnel without duplicating its kill count. This also includes those victims in
the funnel's existing mutation-outcome observation. Cognition precedes all
actions, so a selected victim may already have exhausted its energy: retain its
earlier compute/bid cause even though `predation_kills_total` counts this
predation removal. Thus that existing kill counter is not required to equal
`by_cause.predation`. Reward learning can exhaust energy in Phase 2.5 and leave
removal until the next Phase 0; retain `reward_learning` across that delay. The
post-exhaustion fallback NoOp and subsequent penalties cannot overwrite the
earlier cause. Do not make any of these removals earlier.

**Flow definition.** Each energy sum is a signed `f64` total of observed
changes at its existing arithmetic site: debit = `f64::from(before) -
f64::from(after)`, credit = the reverse. Observe the stored `f32` result, so an
adjusted charge rounded away contributes zero and an overshoot contributes its
actual debit. Report clipping separately; do not hide overshoot by changing
the charge. Capture intermediate values without rearranging the production
operations. This is a flow reading, not a claim of exact closed-system balance.

| Stored flow field(s) | Applied site and boundary |
| --- | --- |
| `food_intake_by_type` | `apply_typed_eat`: credit from the existing food × reward addition, before its cap; dense configured-food-type order; empty/invalid eats contribute zero |
| `action_charges.{noop,eat,move,reproduce,steal_energy}` | `apply_noop`, `apply_typed_eat`, `apply_move`, reproduction Step 5, predation Step 2: observe the actual adjusted debit, including charged rejected actions; no charge for an earlier rejection gate that never reaches the debit |
| `failed_action_penalty` | `debit_failed_action`: distinct from the base debit, on each branch that currently pays it |
| `vm_compute` | `runtime/vm.rs`: the one final debt settlement per VM node; opcode/ramp cost, including the exhausting instruction; excludes bid payments |
| `priority_bid` | Every `SetPriorityBid` payment through its possible all-in rounding pin; sum all payments, including overwritten bids and an exhausting bid that never reaches `side_outputs.priority_bid` |
| `graph_compute`, `hebbian_learning` | `runtime/cgp/execute.rs`: graph base debit and subsequent Hebbian debit separately |
| `reward_learning` | `run_reward_learning`: each existing node's update debit |
| `lifecycle_decay`, `genome_carrying` | `run_phase_0`: partition the actual combined debit using the decay-first formula below |
| `genome_size_creature_ticks` | `u64` sum of `cached_genome_size` at every Phase 0 charge, including creatures removed in that phase; independent of whether the carrying rate is zero; this is not the mesh-dispatch `creature_ticks_total` denominator |
| `parental_transfer_debit`, `offspring_energy_credit` | Reproduction Step 8's actual parent debit and the successfully inserted child's starting energy; rejected births transfer nothing |
| `predation_victim_debit`, `predation_attacker_credit` | Predation Step 6's two actual energy changes before the attacker's cap; preserve signed values if the existing path selects an already-negative victim; no repair to transfer mechanics |
| `predation_kill_bonus_credit` | The actual bonus addition before its cap, separately from stolen energy |
| `maximum_energy_clamp_loss` | Energy discarded by the existing food/predation maximum caps; also observe any lower-bound part of food's existing clamp in `zero_floor_credit` |
| `zero_floor_credit` | The existing food/action/reward-learning lower floors, including recovery of an overshoot; no new floor is introduced |
| `external_removal_loss` | Positive energy discarded by a live creature's external removal at the common funnel |

For Phase 0's one subtraction let `D` be its actual nonnegative debit and `B`
the nonnegative debit of the observational decay-only `f32` result, capped at
`D`. Add `B` to `lifecycle_decay` and `D - B` to `genome_carrying`; their sum is
the applied combined debit, and a zero carrying rate gives zero carrying
debit. The integer size integral separately preserves the rate-independent
exposure used by T03.F08.

The existing `ComputeCostReport.vm_cost` includes bids and its graph cost
includes Hebbian learning. Keep those fields' meaning unchanged; copying them
into the new disjoint buckets would double-count. Attach bounded dispatch-local
flow/cause observations to the shared runtime result and carry them through
all execution modes and exit paths. Observational re-executions may produce
local observations but never commit them to production `SimStats`.

**Deterministic order.** Keep all additions out of parallel shared state.
Dispatch-local sums follow mesh visit order and each backend's existing
instruction/debit order. Commit each completed dispatch in Phase 2's existing
stable priority-sorted queue order, before the missing-creature skip, since
even a creature removed by earlier predation already paid cognition. Record
action flows in that queue's action order and the arithmetic order within each
action. Phase 0 uses existing slotmap iteration order; Phase 2.5 uses existing
creature-ID collection order then genome node order. Commit the same values
for traced and untraced creatures. Use ordinary `f64` additions, no parallel
float reduction, unordered map reduction, telemetry RNG, or per-event log.

**Stored report shape.** Add two optional terminal-only fields to
`WorldTracking`: `mortality: Option<MortalityTracking>` and
`energy_flows: Option<EnergyFlowTracking>`, both
`#[serde(default, skip_serializing_if = "Option::is_none")]`. Leave them absent
in `WorldTracking::observe`; populate them at `with_transferred_counters`'s
existing terminal site. They appear once per goal case at
`deterministic.goal_indicators.cases[].tracking`, not in persistence samples.

- `MortalityTracking` has `definition: "applied-mortality-v1"`,
  `deaths_total: u64`, and `by_cause: BTreeMap<String, u64>` with every cause
  key above present, including measured zeros. The cause sum equals the total.
- `EnergyFlowTracking` has `definition: "applied-energy-flows-v1"`, exactly
  the flow fields in the table, and the integer size integral. Energy values
  use the existing `six(f64)` string representation, with no earlier rounding;
  `action_charges` is a fixed named struct, `food_intake_by_type` is a dense
  `Vec<String>`. Runtime accumulation uses bounded structs and a configured
  food vector. No unordered collection reaches the report.

Absence means unmeasured in a historical report; a present zero is an observed
zero. No existing indicator is redefined and no new composite score is added.

## Implementation Tasks

- [ ] Establish failing focused mortality/flow coverage and pure-invariant
  properties before adding production instrumentation; cover the existing delayed
  and predation-removal paths.
- [ ] Add bounded cumulative mortality/flow state and pending-cause handling;
  join predation to the removal funnel and instrument the verified charge,
  credit, cap and transfer sites without changing execution.
- [ ] Carry dispatch observations through shared runtime modes and the existing
  deterministic reduction; expose the two terminal report blocks.
- [ ] Complete verification, store the gate/goal reports and readings, update
  progress through the existing closure path, and close this spec and its row.

## Verification

- [ ] Focused core tests cover every cause and flow site, scaling and rejected
  gates, first-crossing/recovery precedence, delayed reward removal, duplicate
  removal, predation of a previously exhausted victim, successful/failed
  parental transfer, multiple/all-in bids, combined decay/carry allocation,
  carrying exposure, float rounding and caps. Results and exact test names:
  `docs/progress/readings/t14-f03.md`.
- [ ] Pure-invariant property tests cover count partition, pending-cause
  transitions and the combined-debit partition; traced/untraced runtime
  equivalence and all exit paths cover the new observations. Assertions are
  independent of which generated cases are drawn; commit regressions if any.
- [ ] CLI report tests check terminal source-to-report equality, all cause
  keys, configured food order, absent historical blocks, and omitted checkpoint
  blocks. A tiny real goal-world run demonstrates applied nonzero observations.
- [ ] Reproducibility coverage includes new integer totals and raw float bits
  across independent simulations/thread counts; existing gate two-run byte
  identity remains inside `make check`. Production trajectories, actions and
  RNG results match the pre-feature behavior under unchanged inputs.
- [ ] `cargo test -p v3-core --test viability` first for tick-loop work, then
  `make check`: results in the readings file.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: record summary, output path,
  and the full survivor list here, each killed, equivalent or user-deferred.
- [ ] `make bench PROFILE=gate FEATURE=t14-f03-applied-mortality-and-energy-accounting`
  and one `make bench PROFILE=goal FEATURE=t14-f03-applied-mortality-and-energy-accounting`:
  stored reports below; readings include each world's mortality and flow rows,
  effective config identity, existing persistence and pressure observations.
- [x] Second goal run: not applicable under the shared one-run closure rule;
  cross-process/thread reproducibility is checked by ordinary tests.

## Performance and Goal Impact

**Predeclaration — written before the run.** Observation-only feature, exempt
from the natural-analog rule; it introduces no environmental pressure. The
ordinary goal run retains Orchards, Canyon and Confluence with their existing
recipes and pressures. The gate profile remains unchanged.

References are selected by the existing series index: gate epoch
`docs/progress/features/remove-complementary-nutrition.json`, goal-worlds epoch
`docs/progress/features/t12-f04-baseline-world-set-goal.json`, plus the latest
closed report of each corresponding series, excluding the current output.
Normalized work thresholds remain +10% flag/+50% severe; matching-host wall
time uses +25%/+100% advisory flags. Founder-neighborhood observation stays
within 10 seconds, evolved-neighborhood observation within 180 seconds summed
across seeds, and total goal-profile investigation within 15 minutes. Existing
indicator floors, profile parameters and mutation gates are unchanged.

Expected compute cost is small constant bookkeeping per existing debit/credit,
one `u64` addition per Phase 0 creature, and bounded per-dispatch reduction;
no extra genome walk, opcode pass, snapshot, RNG draw or event allocation is
required. No severe regression or epoch re-pin is expected or authorized.

Expected direction for all existing goal indicators is none: population
persistence, births per 100 ticks, lineage diversity, memory sensitivity,
temporal memory sensitivity, mutational neighborhood, drift depth, reachable
structure and recruitment/structural observations must reflect unchanged
creatures. A change caused by telemetry affecting execution is a defect.
Existing mutation-outcome totals can increase because predation victims now
reach their existing observer; this is corrected observation coverage, not a
change in mutation, survival or selection. The new mortality and flow fields
are first readings, with no beneficial direction or historical-zero baseline.

Goal impact: later cost features can compare the death sink, applied compute
debit, actual action costs and carrying exposure in the same stored worlds.
These raw readings support causal investigation without making a cognition or
diversification claim.

**Measured verdict.** Pending gate and goal measurement; no epoch re-pin.

- Reports: `docs/progress/features/t14-f03-applied-mortality-and-energy-accounting.json`
  and `docs/progress/features/t14-f03-applied-mortality-and-energy-accounting-goal.json`.
- Full readings: `docs/progress/readings/t14-f03.md`.

## Success Criteria

- [ ] Every creature removal is counted once with the specified attribution;
  delayed removal and predation preserve the exhausting sink.
- [ ] Applied energy flows and genome carrying exposure reach each goal case's
  terminal report in deterministic, bounded fields.
- [ ] Historical absence stays unmeasured; execution, economics and standard
  environments remain unchanged; required verification and closure pass.

## Notes for AI Agents

- Decision: T14.F03 measures the existing sinks and transfers. Predation's
  removal counter and mortality's exhausting-sink counter answer distinct
  questions for a victim already exhausted by cognition.
