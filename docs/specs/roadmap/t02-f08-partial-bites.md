# T02.F08 — Partial Bites

**Status**: In Progress
**Last updated**: 2026-10-12
**Feature**: T02.F08
**Track**: [T02 — Environmental Dynamics](../../roadmaps/t02-environmental-dynamics.md)

## Goal

Partial grazing, the natural analog: a herbivore crops a plant and leaves
enough to regrow. `Eat` gains one evolvable parameter field, the share of the
cell's food to leave behind, decoded to `[0, 1]`. A creature eats
`amount × (1 − leave)`; the rest stays on the cell, where local growth acts on
it in place. The grazing bite scales with the share eaten, so eating
everything is exactly today's bite. An unwired field reads 0.0, so founders and
every genome that never writes it eat everything and behave as before. A
creature can therefore stop short of energy it cannot store above
`max_energy`, and a lightly grazed cell keeps growing.

## Non-Goals

- No new config field, runtime-panel control or mechanism switch: the share is
  a genome parameter, not a world setting.
- No sensor for the cell's remaining food beyond what exists; no change to
  `consume_any`, which still takes everything with a full bite.
- No change to the grazing layer's recovery, floor, factor or read sites, to
  the occupancy depletion layer, or to the growth passes in `ecology.rs`.
- No benchmark gate, goal profile or mutation gate: user exemption
  (2026-10-12), recorded in the T02 track note and below.
- No edit to the closed T02.F04 spec; this spec states the bite rule that
  supersedes its "one applied `Eat` is one bite".

## Inputs and Invariants

**Contract.** The T02.F08 track row and its 2026-10-12 Notes entry are the
feature definition; this spec adds nothing to them. Dependency T02.F04 gives
the per-type grazing modifier, bitten by `consume_type` and read by growth.
T11.F27's catalog rule binds the storage: the catalog holds exactly the
decoded fields in kind-major order, and a feature that decodes another field
extends the catalog and the decoder together.

**Current code (verified 2026-10-12 at `94d80322`).**

- `creature/genome/vote.rs`: `ActionParamField` is `EatFoodType`,
  `ReproduceTransferFraction`, `StealEnergyAmount`; `ACTION_PARAM_FIELD_COUNT`
  is 3. `cgp.rs` appends the sinks at 94..97 and const-asserts
  `FIXED_SINK_COUNT == 97`; `genome_size` counts only wired sinks, so the
  founder stays 97 units when a sink is added.
- `runtime/action_decode.rs`: `decode_commit` builds `WorldAction::Eat {
  type_idx }` from `EatFoodType`; `clamp_unit_interval` (NaN, ±∞ and negatives
  to 0.0, above 1.0 to 1.0) already decodes `ReproduceTransferFraction`.
- `runtime/types.rs`: `action_params` is zero when a creature's mesh run
  starts and is tick-scoped (`begin_pass` clears votes, not the surface);
  last write wins, so a value written in an earlier pass of the tick is read
  by a later pass's commit. The T02.F08 Notes entry's "starts each pass at
  0.0" is corrected by this feature to "starts each tick's mesh run at 0.0";
  its consequence, that an unwired field reads 0.0, is unchanged. The VM write (`runtime/vm.rs`) ignores an invalid
  `field_idx`; opcode 25 draws `field_idx` over `0..ACTION_PARAM_FIELD_COUNT`
  (`mutation/vm/operators.rs`); `pick_random_surface` draws over compute nodes
  plus `FIXED_SINK_COUNT` sinks.
- `kernel/ordinary_food/state.rs`: `consume_type` reads the cell, sets it to
  0.0 when positive and calls `GrazingLayer::bite`, which applies
  `bite_modifier(m, factor, floor) = max(floor, m × factor)`.
- `simulation/actions/mod.rs`: `apply_typed_eat` credits `food × reward`,
  clamps to `max_energy` and books the excess in
  `maximum_energy_clamp_loss`; a zero consume is a failed eat (effort booked
  under `failed_action_effort.eat`). `simulation/tick.rs` `execute_eat` logs
  `food_before` as the amount and counts `eat_actions_applied_total_by_type`.
- `kernel/ordinary_food/ecology.rs` `claim_cell` returns `EMPTY` when the
  source density is `<= 0.0`: a cell with food left grows in place with no
  change to the growth passes.
- `contracts/actions.rs` `WorldAction::param`: `Eat` carries slot 0 only.
- Founder: `cgp_founder.rs` wires `ActionParam(ReproduceTransferFraction)`;
  `founder.rs` writes it through `WriteActionParam { field_idx: 1 }`.
- Frontend: `types/genome.ts` `ACTION_PARAM_FIELDS`, `graphNodeFormatters.ts`
  labels, `actionParamLabels.test.ts` fixtures assume three fields.
- Reference docs naming the three-field surface or 97 sinks:
  `v3-genome-spec.md`, `v3-graph-backend-spec.md`, `v3-vm-isa-spec.md`,
  `v3-mesh-execution-spec.md`, `v3-mutation-spec.md`.
- CI runs on `ubuntu-24.04` while the host is macOS; `exp()` already sits on
  the production path (`cgp/eval.rs` sigmoid), so libm use has precedent, and
  the full-bite path below avoids `powf` entirely.

**Research, 2026-10-12.** The row fixes the mechanism; the research settles
how to express it.

| Option | Evidence and fit | Disposition |
| --- | --- | --- |
| A leave share in `[0, 1]` decoded by `clamp_unit_interval` | Same shape and sanitizing as `ReproduceTransferFraction` (T17.F01); unit-free, so one value means the same on a dense and a sparse cell; 0.0 is the current behavior, which the zero surface gives for free | Adopted (the row's choice) |
| An absolute amount to eat (as `StealEnergyAmount`) | Depends on cell density and `energy_per_unit`; 0.0 would mean "eat nothing", so an unwired field would starve founders | Rejected |
| Stop automatically at `max_energy` | Not evolvable, no grazing choice; the row asks for an evolvable share | Rejected |
| Bite strength `factor^share` | Multiplicative in the share: `n` bites of share `1/n` equal one full bite, so splitting a bite neither escapes nor compounds grazing; share 1 is `factor` exactly | Adopted (the row's rule) |
| Bite strength linear, `1 − share × (1 − factor)` | A bite split in two grazes less than one full bite, rewarding bite-splitting for no ecological reason | Rejected |

Analog evidence: grazing lawns (McNaughton 1984, *Am. Nat.* 124:863–886)
show cropped-not-cleared vegetation regrowing faster than cleared ground,
which is the in-place growth this feature exposes; Charnov 1976 (read for
T12.F04) already frames the rest of the patch decision.

**Fixed design.**

| Decision | Value |
| --- | --- |
| Catalog | `ActionParamField::EatLeaveShare` after `EatFoodType`: `ALL` is `EatFoodType`, `EatLeaveShare`, `ReproduceTransferFraction`, `StealEnergyAmount`; `index()` 0..4; `kind()` of the new field is `VoteKind::Eat`. `ACTION_PARAM_FIELD_COUNT` 4, `FIXED_SINK_COUNT` 98, parameter sinks at 94..98. Every field index outside `vote.rs` comes from `index()`, the founder's `WriteActionParam` included. |
| Action | `WorldAction::Eat { type_idx, leave: f32 }`, `leave` in `[0, 1]`; `WorldAction::eat(type_idx)` builds `leave` 0.0. `param(1)` returns `leave` (slot 0 stays the type index). |
| Decode | `decode_commit` reads `leave = clamp_unit_interval(params[EatLeaveShare.index()])`; a zero surface decodes `leave` 0.0. Multi-pass: a `leave` written in an earlier pass of the tick is read by a later pass's `Eat` commit until overwritten and is zero again at the next tick's mesh run (T11.F27's tick-scoped surface, unchanged). |
| Consume split | `consume_type(pos, type_idx, leave, grazing)` returns the amount eaten. `amount` is the cell's density; `eaten = amount × (1 − leave)`; `left = amount − eaten`. When `eaten > 0`: the cell becomes `left` and the type's modifier is bitten with share `1 − leave`. When `amount <= 0` or `eaten <= 0` (`leave` 1.0, or an underflow): nothing changes, no bite, 0.0 returned. `leave` 0.0 is today's path bit for bit: `eaten == amount`, the cell set to 0.0, a full bite. `consume_any` bites with share 1.0. |
| Bite | `bite_modifier(m, factor, floor, share)`, `share` in `(0, 1]`: `max(floor, m × factor)` when `share >= 1.0`, else `max(floor, m × factor.powf(share))`. `GrazingLayer::bite` takes the share; disabled grazing records nothing. |
| Eat action | `apply_typed_eat` passes `leave` through, credits `eaten × reward` with the existing clamp and flow booking, and returns the eaten amount (0.0 is a failed eat, effort booked as today). `execute_eat` logs the eaten amount, not `food_before`, and increments `SimStats.eat_actions_partial_total_by_type` (a `HashMap<OrdinaryFoodTypeId, u64>` beside the applied and failed maps) when the applied eat left food (`eaten < amount`); the bench tracking's `typed_eats_total` and the lab's `eaten` keep reading the applied map. |
| Growth | No change: the remaining density is a positive source for `claim_cell`, so a partially eaten cell grows in place on the next pass while a cleared cell with no food neighbor and recovery spawns off stays empty (a recovery spawn can seed any cell). A fixture test shows both. |
| Mutation reach | No new operator. Opcode 25 and `pick_random_surface` cover the new field through the catalog constants; `mutate_one_instruction_field`'s ±1 nudge reaches it. |
| Frontend | `ACTION_PARAM_FIELDS` gains `EatLeaveShare` in catalog order with label `Eat.leave`; the fixtures assert four sinks and `field_idx` 0..3 distinct, 4 invalid. `types/trace.ts`'s `{ Eat: { type_idx } }` action shape gains `leave`. |
| Reference docs | The five docs above state four fields, 98 sinks, sinks 94..98, `field_idx` in `0..4`, `Eat` slot 1 = leave share, and the consume split. |
| Determinism | No RNG; `powf` runs only for a partial bite; seeded runs stay reproducible. |

**Trajectory pins that legitimately move.** Three causes: the representation
(every graph genome carries a fourth parameter sink, so serialized genome
text and anything hashing it, such as `legacy_default_short_run_identity`,
which hashes `{:?}` of each creature's genome from tick 0, change with no
behavior change); the draws (opcode 25 over `0..4`, `pick_random_surface`
over `c + 98`); and the nudge (`mutate_one_instruction_field`'s ±1 on a
`WriteActionParam` field index now lands on a field with a different
meaning). Pins hashing a default run or a mutating trajectory (for example
`legacy_default_short_run_identity`,
`accounting_preserves_pre_feature_sampled_trajectories_and_actions`) and
tests asserting 97 sinks (`cgp.rs`, `mutation/graph/operators.rs`) are
re-pinned once, old and new values in the readings file, each attributed to
one of the three causes or to the trajectory downstream of one. Unwired
behavior is verified by tests on `decode_commit` and `consume_type`, not by
a genome hash. No config changes, so no config digest moves, and the
founder's `genome_size` stays 97. Any other moved pin is a defect.

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.eat_actions_partial_by_type` | counter (`Sum`), as `petri.run.eat_actions_applied_by_type` is | `SimStats.eat_actions_partial_total_by_type`: applied eats that left food on the cell | attribute `petri.food_type`; a row in the expected families of `crates/v3-telemetry/src/metrics/tests.rs` and its Prometheus name in `scripts/telemetry-dashboards-check` `keyed_names` |
| `Param Eat.leave` | creature inspector label for the Graph sink and the VM `WriteActionParam` target, not a metric | the genome | none |
| `maximum_energy_clamp_loss`, `failed_action_effort.eat` (existing `petri.run.energy_flow` flows), `petri.tick.food_grazed_cell_share` | existing signals the mechanism moves: clamp loss falls when creatures leave food, failed eat effort rises when they leave all of it | unchanged | unchanged |
| configuration | none: no config field or mechanism switch is added, so nothing new reaches `run.started` or a phase group | — | — |

## Implementation Tasks

- [ ] Catalog and decoder: `EatLeaveShare`, counts 4 and 98, `WorldAction::Eat
      { leave }`, `param(1)`, `decode_commit`, founder index by `index()`;
      decode unit tests and a proptest that `leave` lands in `[0, 1]` for any
      `f32` and equals `clamp(0, 1)` of a finite input; the existing
      sensitivity and independence tests cover the new field through `ALL`.
- [ ] Consume split and bite: `consume_type` with `leave`, `bite_modifier`
      with `share`, `GrazingLayer::bite` with `share`; unit tests for `leave`
      0.0 (bit-identical full bite), 1.0 (failed eat, cell and modifier
      untouched) and a mid value; edge cases: a positive subnormal density
      whose `eaten` rounds to zero (failed eat, cell untouched), a tiny
      positive `leave` whose complement rounds to 1.0 (a full bite), disabled
      grazing (the split applies, no bite), an unknown food type (the
      `mod.rs` catalog check still returns 0.0), and `m < floor` after a live
      floor edit (the bite clamps up to `floor`, today's behavior). Proptests
      over finite `amount > 0`, `leave` in `[0, 1]`, `factor` and `floor` in
      `[0, 1]`, `m` in `[floor, 1]`: `0 <= eaten <= amount`, `left >= 0`,
      `eaten + left` within `f32::EPSILON × amount` of `amount`, the bitten
      modifier in `[floor, m]` and never below the full bite's, and, for
      `m` and `factor` in `[1e-3, 1]` (every product normal, so the
      relative bound holds; equivalence is not claimed in the subnormal
      range) where the floor is not reached, two half bites equal to one
      full bite within a relative `1e-5`.
- [ ] Eat action and stats: `apply_typed_eat` returns the eaten amount,
      `execute_eat` logs it and counts `eat_actions_partial_total_by_type`;
      the counter exported, listed in the telemetry tests and the dashboards
      check. An applied partial-eat fixture through `execute_eat` checks the
      retained density, the credited energy, the logged amount and the
      partial, applied and failed counters; the pass-loop commit fixture
      (`runtime/pass_loop_tests.rs`) gains `EatLeaveShare`, including a value
      written in an earlier pass and committed in a later one.
- [ ] Growth fixture: a partially eaten cell grows in place on the next
      growth pass; a cleared cell with no food neighbor and recovery spawns
      off stays empty.
- [ ] Frontend types, labels and fixtures; reference docs; re-pins recorded in
      the readings file.

## Verification

- [ ] `cargo test -p v3-core --test viability` first (tick-loop and founder
      path change), then `make check` -> exit 0; results in
      [`docs/progress/readings/t02-f08.md`](../../progress/readings/t02-f08.md).
- [ ] Focused tests, one or more per Fixed-design row: names and results in
      the readings' "Focused tests" table.
- [ ] Mutation gate: `Not applicable: user exemption (2026-10-12)`.
- [ ] Benchmark summary: `Not applicable: user exemption (2026-10-12)`.
- [ ] Codex final review output path and findings in the readings file.

## Performance and Goal Impact

**Predeclaration — written before the run.**

`Not applicable: user exemption (2026-10-12)`. No gate or goal profile runs
for this feature (T02 track note, 2026-10-12; the exemption is the workflow's
Benchmark gate user exemption). Expected cost, unmeasured: two subtractions
and a multiply per applied eat, a `powf` per partial bite, one more surface
cell per mesh run and one more sink per graph scan. Adding a parameter field shifts mutation draws over field indices,
so trajectories move; the next closure that runs the profiles re-pins and
names T02.F08 among the changed inputs.

**Measured verdict.** Not measured: user exemption (2026-10-12).

## Success Criteria

- [ ] `decode_commit` yields `leave` in `[0, 1]` for any surface value and
      0.0 for an unwired field; the catalog, VM address space and runtime
      surface hold exactly the four decoded fields.
- [ ] A creature eats `amount × (1 − leave)`, the rest stays on the cell and
      grows in place, the grazing bite is `factor^(1 − leave)`, `leave` 0.0
      is bit-identical to the previous full bite, and an eat that takes
      nothing is a failed eat.
- [ ] Founders behave as before, every re-pinned value is attributed to the
      representation, the draws or the nudge semantics (or their downstream
      trajectory), and `make check` passes with the Codex review's P1
      findings resolved.

## Notes for AI Agents

- Exception: user exemption (2026-10-12) from the benchmark gate, goal
  profile and mutation gate; verification is TDD with unit and proptest
  coverage, the spec challenge loop, the Codex review and `make check`.
