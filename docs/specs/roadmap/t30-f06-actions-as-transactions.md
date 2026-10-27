# T30.F06 — Actions as Transactions

**Status**: In Progress
**Last updated**: 2026-10-26
**Feature**: T30.F06
**Track**: [T30 — Core Boundaries](../../roadmaps/t30-core-boundaries.md)

## Goal

Creatures act exactly as before, each action resolved by mechanics as one
transaction over narrow inputs that returns one applied result, a
`contracts` value carrying the outcome kind, the attempt effort and the amount
moved. The scheduler derives the creature's action log, its reward success
count, the run totals and the barrier telemetry from that one value and
settles every removal an action causes through `Simulation::remove_creature`.
The move cost is composed in mechanics with the body supplying its factor; the
server's action-log wire entry derives from the stored outcome. Debts G18,
G19 and G22 of [`docs/architecture.md`](../../architecture.md) are paid; every
trajectory, population and draw stream is unchanged.

## Non-Goals

- Any change to what an action costs, when it succeeds, what it moves, when a
  creature dies or which cause is attributed.
- The birth path (`spawn_child`, `deliver_pending_birth`, seeding; G4,
  T29.F01). Delivered litters and Phase 0 removals stay lifecycle records.
- New metrics, frontend behaviour beyond the generated types, any rename of
  a `SimStats` field or an exported key (bench tracking, telemetry).
- Making the creature record's fields private (G27, T30.F08).

## Inputs and Invariants

Inputs: the row and the track's F06 notes; `docs/architecture.md` sections 3
and 4 and debts G18, G19, G22; the code named below; the
[readings file](../../progress/readings/t30-f06.md) (options, audit R1–R8).

The invariants state the closing state.

1. **Vocabulary.** `contracts::actions::ActionOutcome`, `repr(u8)` in this
   order: `Success`, `NoFood`, `Blocked`, `InvalidTarget`,
   `EnergyConstraints`, `PopulationCap`, `TransferredAndKilled`, `NoVictim`,
   `AgeConstraints`, `NoRecipient`, `NothingToTake`, `NothingToGive`,
   `LitterTooSmall`; `ALL`, `as_key` (the variant name), `serde::Serialize`
   and `succeeded()`, true for `Success` and `TransferredAndKilled` only.
   `contracts::actions::ActionResult { outcome: ActionOutcome, attempt_effort: f64, amount: f32 }`:
   the effort is the attempt's applied debit (the noop, eat or move effort,
   the exchange effort of a steal or give, step 0 of a reproduce); the
   amount is what the log records today (food eaten, energy moved, the
   requested transfer fraction of a reproduce, `0.0` otherwise).
   `creature::action_log::ActionResult` and `actions::TransferOutcome` are
   deleted. The outcome each action produces, from today's mechanics result:

   | Action | Today | Outcome |
   | --- | --- | --- |
   | NoOp | always | `Success` |
   | Eat | `eaten > 0.0` / else | `Success` / `NoFood` |
   | Move | moved / blocked | `Success` / `Blocked` |
   | Reproduce | `Conceived`, `RejectedInvalidTarget`, `RejectedPopulationCap`, `RejectedAgeConstraints`, `RejectedEnergyConstraints` at step 6, the litter refusal at step 7 | `Success`, `InvalidTarget`, `PopulationCap`, `AgeConstraints`, `EnergyConstraints`, `LitterTooSmall` |
   | StealEnergy | `Transferred`, `TransferredAndKilled`, `RejectedNoVictim`, `RejectedNothingToTake` | `Success`, `TransferredAndKilled`, `NoVictim`, `NothingToTake` |
   | GiveEnergy | `Transferred`, `RejectedNoRecipient`, `RejectedNothingToGive` | `Success`, `NoRecipient`, `NothingToGive` |

2. **Transaction.** `simulation/actions/transaction.rs` holds
   `ActionInputs<'a>` (`creatures: &'a mut SlotMap<CreatureId, CreatureState>`,
   `world: &'a mut WorldState`, `config: &'a SimulationConfig`,
   `flows: &'a mut EnergyFlows`, `tick: u64`) and
   `resolve(inputs, id, &WorldAction) -> Applied` (no RNG; only the
   commit step's birth path draws), with
   `Applied { result: ActionResult, energy_after: f32, actor_removed: bool, counterpart: Option<CreatureId>, conceived: Option<Conception> }`
   and `Conception { target: Position, litter: f32 }`. `apply_noop`,
   `apply_typed_eat` and `apply_move` keep today's parameters (one creature,
   the world, the config, the flows); `apply_steal_energy`,
   `apply_give_energy`, `apply_reproduce` and `neighbour_creature` take
   `&mut ActionInputs`; every `apply_*` returns `ActionResult`, and their
   energy arithmetic, flow bookings (`action_charges`, the exchange and food
   flows, the clamp loss) and pending-cause writes run in today's order.
   `apply_reproduce` returns `(ActionResult, Option<Conception>)`: at
   gestation span 0 the conception computed where the litter is debited
   today (`reproduction.rs:254-280`), which `resolve` forwards as
   `Applied.conceived` instead of spawning; a span above 0 sets
   `pending_birth` with `deliver_tick = tick.saturating_add(span)` as today
   (`reproduction.rs:299-302`). A counterpart is removed only by a
   `TransferredAndKilled` outcome; an exhausted victim yields
   `NothingToTake` and stays, as today (`predation.rs:103-110`). The
   transaction's settle step then,
   in this order, books `failed_action_effort[kind] += attempt_effort` when
   `!outcome.succeeded()` (the one site), floors the actor at `0.0` and nets
   the overdraft with `EnergyFlows::net_action_overdraft` against the
   charged-before snapshot (today `tick.rs:1308-1317`), and reports
   `energy_after` as the actor's energy before the floor and `actor_removed`
   as its energy at or below `0.0` after it. The transaction reads and writes no `SimStats` field,
   pushes no log entry and removes no creature; `simulation/actions/`
   imports `Simulation` only on the birth path (invariant 7).
3. **Move cost.** `simulation::actions::move_effort(body: &HeritableBody, energy: &EnergyConfig, complexity: u32, age: u64) -> f32`
   is `energy.action_effort(energy.costs.move_cost * body.move_factor(&energy.capacity), complexity, age)`;
   `CreatureState::move_effort` is deleted; `apply_move` and the lab
   (`v3-lab/src/eval.rs:822,881`) call it, and the lab's `apply_typed_eat`
   and `apply_move` calls (`eval.rs:646,654`) read the returned outcome.
   `energy_cap` stays a body rule on the record.
4. **Commit.** `simulation/tick/actions.rs` (`pub(super)`) holds today's
   `run_phase_2` loop: the gestation hold and delivery unchanged, then per
   action: the context (tick, priority bid, energy before, the actor's
   position, the cell's food before an eat, the barrier context before a
   move or reproduce), `resolve` over inputs borrowed from `sim`'s fields,
   `spawn_child(id, sim, target, direction, litter, rng)` for a returned
   conception, the records of invariant 5, settlement (invariant 6), then
   the queue rules (actor removed, or `pending_birth` set, ends the queue).
   `ActionContext`, `push_action_log`, `BarrierContext` and the six
   `execute_*` functions leave `tick.rs`.
5. **Derived records.** Every record below is written by the commit step from
   the action, the result and the context, with today's values and keys:

   | Record | Derivation |
   | --- | --- |
   | Creature history | `record_action_attempt(kind)` once per action; `record_applied_eat` on `Success` eat; `lifetime_blocked_move_count` on `Blocked`; `lifetime_invalid_reproduce_count` on `InvalidTarget`; `lifetime_predation_kills_count` on `TransferredAndKilled`; the counterpart's `lifetime_predation_hits_taken_count` on a steal `Success` |
   | Reward | `OutcomeAccumulator::record_action_result(id, outcome.succeeded())`; `record_damage(counterpart, amount)` on a steal `Success`; `record_offspring(id)` and the spawn-target set on a conception born this tick |
   | Log | one `ActionLogEntry { tick, action_type, outcome, direction, energy_before, energy_after, amount, food_type, priority_bid }` (32 bytes, `NO_DIRECTION` as today) from the context and the result, pushed to the actor's log and to the window seam's capture |
   | Per-type totals | `last_tick_{noop,eat,move,reproduce,steal,give}`, the `*_attempted_total`, `*_transferred_total`, `*_rejected_total` counters, `eat_actions_{applied,partial,failed}_total_by_type` (partial: `amount < food before`), `move_actions_blocked_total_by_cause` (`classify_move_blocked_cause`), `gestations_started_total`, `reproduction_actions_rejected_litter_total` on `LitterTooSmall`, `predation_kills_total`, `last_tick_predation_kills` |
   | Keyed totals | `predation_actions_by_result`, `give_actions_by_result`, `reproduction_actions_rejected_by_reason` keyed by today's `PredationActionResult`, `GiveActionResult`, `ReproductionActionResult`, which stay in `simulation::actions` with their keys, `ALL` and `REJECTIONS`, each built from the outcome by a constructor over the rows of invariant 1 (`LitterTooSmall` keys `RejectedEnergyConstraints`) |
   | Events and seams | `last_tick_predation_events` from the actor's and the counterpart's positions read before settlement; `observed_damage`; `FlowMarks::applied` for the observed creature |
   | Barrier telemetry | the `*_with_barrier_neighbor_*`, `*_avoidable_*` and `*_invalid_target_*` maps from the barrier context and the causes, as in today's `execute_move` and `execute_reproduce` |

6. **Removal.** The commit step settles the counterpart of a
   `TransferredAndKilled` outcome and then a removed actor through
   `Simulation::remove_creature`; `apply_steal_energy` no longer removes;
   `tick/helpers.rs::remove_creature_if_dead` keeps serving the
   gestation-hold check; Phase 0 and paint removal are unchanged.
7. **Birth path.** `spawn_child` and `deliver_pending_birth` keep their
   signatures and `&mut Simulation`, called from the commit step (and
   `spawn_child` from `deliver_pending_birth`, `reproduction.rs:356`, as
   today), each under a `WORKAROUND(G4): the birth transaction of T29.F01 takes world context, not the simulation` comment.
8. **Wire.** `v3-server/src/transport/action_log.rs`: the wire enum is
   `ActionOutcome` with invariant 1's variants, pinned to
   `contracts::ActionOutcome::ALL` by the existing test; the wire entry's
   field is `outcome`; `From<&CoreEntry>` reads the stored outcome and
   amount. The generator test rewrites `frontend/src/types/generated/ActionOutcome.ts`
   and `ActionLogEntry.ts` and deletes `ActionResult.ts`; the frontend reads
   `outcome`; `http/creature.rs` diagnostics key `"{type}:{outcome key}"`;
   `PROTOCOL_VERSION` becomes `v3alpha15` in `v3-server/src/types.rs` and
   `frontend/src/types/protocol.ts`.
9. **Preservation.** After each pass the gate's `deterministic` values and
   nine work counters equal the reference's; no `scripts/architecture-debts.txt`
   line is added; at closure G18, G19 and G22 leave `docs/architecture.md`.

## Architecture

Owners touched: `contracts` (the applied action result and its outcome
kind), the energy ledger (the failed-effort subset booked once), `creature`
(the record loses the move-cost composition; its log stores the contracts
outcome), `simulation` (mechanics resolve a transaction over narrow inputs;
the commit step derives the records and settles removals through the one
path), `v3-server` (the wire entry from the stored outcome), the frontend
(generated types) and the lab (four call sites in `eval.rs`). Debts paid: G18, G19 and G22 (section 5 rows at
closure; no debts-file line exists for them). Debts used and left: G4, since
`spawn_child` and `deliver_pending_birth` keep `&mut Simulation` as scheduler
calls under a `WORKAROUND(G4)` comment until T29.F01's birth transaction;
G27, since the commit step writes the record's lifetime counters through
today's `pub(crate)` methods and public fields; G5, since the move factor is
read from `creature::body::HeritableBody`. Shared rules read, not changed:
`EnergyConfig::action_effort`, the ledger's debit and pending-cause rules,
`EnergyFlows::net_action_overdraft` (now called from the settle step) and
`Simulation::remove_creature` (`simulation/simulation.rs:102`).
`docs/architecture.md` already places the result in `contracts` and the
transaction, removal path and derived records in `simulation`; the plan
commit widens the `contracts` cell to name the amount.

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `creature_window` action event attribute `petri.result` (`v3-telemetry/src/windows.rs:426`) and the record body's `res` array (`windows/record.rs:117`) | span event attribute and record body field, existing | `ActionOutcome::as_key` of the logged outcome | unchanged; the three finer keys `NothingToTake`, `NothingToGive`, `LitterTooSmall` replace `NoVictim`, `EnergyConstraints`, `EnergyConstraints` for exactly those cases |

No counter, gauge, span or config value is added; the metric families keyed
by the per-action result enums keep their keys; energy-flow and mortality
values are unchanged.

## Verification Plan

**Question.** Each action resolves as one transaction with one result from
which every record derives; a moved gate pin, a changed `deterministic` value
or work counter, or a renamed exported key would show it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate` | Tick-loop code moves with no population reading in question; the row runs the gate after each of the two passes with pins unchanged. |
| Slow tests (`make rust-test-slow`) | `run` | `crates/v3-lab` changes (the `move_effort` and `apply_*` call sites in `eval.rs`), and the workflow runs the tier for any lab change; no operator, draw, catalog or recruitment change. |

## Implementation Tasks

Pass 1 (executors with narrow inputs and the cost composition):

- [x] `simulation::actions::move_effort`; `CreatureState::move_effort`
      deleted; `apply_move` and the lab on it (R3; invariant 3).
- [x] `simulation/actions/transaction.rs` with `ActionInputs`, `resolve`
      and `Applied` with the settle step (the floor, the netting, the
      removal report); the six executors and the three exchange and reproduce
      functions on the inputs, the victim removal and the span-0 spawn
      returned to the caller (R1, R2, R6; invariants 2, 6, 7 with the
      `WORKAROUND(G4)` comments).
- [x] `simulation/tick/actions.rs` with the loop, the log push and the
      settlement; `tick.rs` without the executors (R1; invariant 4).
- [ ] Gate run; `deterministic` values and work counters equal.

Pass 2 (the one applied result and the records derived from it):

- [x] `contracts::actions::{ActionOutcome, ActionResult}`; the log enum and
      `TransferOutcome` deleted; every `apply_*` returns `ActionResult`; the
      key enums gain their outcome constructors (R4; invariants 1, 5).
- [x] `ActionInputs` loses `stats` for `flows`; the failed-effort booking
      at its one site; the commit step derives every record of invariant 5
      from the result (R4, R7; invariants 2, 5).
- [x] `ActionLogEntry.outcome`; the wire enum, entry and `From`; generated
      files rewritten with `PETRI_WRITE_TS_TYPES=1`; frontend readers on
      `outcome`; diagnostics keys (R5; invariant 8).
- [ ] At closure: the G18, G19 and G22 rows deleted; the gate summary
      recorded.

## Verification

- [x] `cargo test -p v3-core --test viability` first -> 30 passed.
- [x] `simulation/actions/transaction` tests (settle order, failed effort
      booked once, span-0 handoff, exhausted victim kept, no removal in
      `resolve`) -> 6 passed.
- [x] Focused tests: v3-core `--all-features` lib 2348 and every
      integration test passed; v3-server 174, v3-telemetry 80, v3-lab 212;
      `npm test` 673, `tsc -b`, lint, `make rust-clippy`, `cargo fmt --all
      --check` and `cargo check --workspace --all-targets` clean.
- [ ] `applied_trajectory`: state-only sampled digests (log bytes excluded)
      pinned, equal at `18342fef^` and after -> values in the readings file.
- [ ] `make check` -> exit 0 at the tested commit; `make rust-test-slow` ->
      exit 0.
- [ ] Gate after pass 1 (readings only) and after pass 2 (summary at
      `docs/progress/features/t30-f06.json`, raw hash and byte counts, series
      entry, no raw staged); every `deterministic` value and work counter
      equal in both. Goal: `Not run: Verification Plan`.

## Performance and Goal Impact

**Predeclaration — written before the run.** A refactor with no mechanism
and no natural analog to name. Expected cost: none: the arithmetic, its
order and every draw are unchanged; each action returns one struct and the
records derive from a match on it. Reference: the last measured closure's
summary (t30-f07); thresholds: the gate's own.
Expected direction: the nine work counters and every `deterministic` value
identical after each pass; wall clock neutral within the gate's noise, a
regression attributed before closure. No indicator can move; nothing is
re-pinned.

**Measured verdict.** Pending.

- Summary: [gate](../../progress/features/t30-f06.json).
- Readings: [`t30-f06.md`](../../progress/readings/t30-f06.md).

## Success Criteria

- [ ] `contracts/actions.rs` holds `ActionOutcome` and `ActionResult`;
      `simulation/actions/transaction.rs` and `simulation/tick/actions.rs`
      exist with the contents of invariants 2 and 4; `creature::action_log::ActionResult`,
      `TransferOutcome` and `CreatureState::move_effort` do not.
- [ ] No file under `simulation/actions/` imports `Simulation` except the
      birth path of invariant 7, and no action-resolution code there writes
      a `SimStats` field; the birth helpers keep their lifecycle bookings.
- [ ] The wire enum pins to `contracts::ActionOutcome::ALL`; the generated
      files match the generator test.
- [ ] The tests of the Verification section pass; `make check` and
      `make rust-test-slow` exit 0.
- [ ] Both gates' `deterministic` values and work counters equal the
      reference's.
- [ ] `docs/architecture.md` section 5 no longer lists G18, G19 or G22.

## Notes for AI Agents

- Decision: in-feature refactors R1–R7 of the preparatory audit in
  `docs/progress/readings/t30-f06.md` are this feature's scope.
- Decision: two implementation passes under this spec, the gate after each
  with pins unchanged, one closure (user, 2026-10-25).
- Decision: the per-tick reproduction RNG derivation in `run_tick` is the
  tick's design, not a workaround (user, 2026-10-26).
- Decision: `ActionOutcome` carries every distinction today's records make,
  so the log and the wire gain `NothingToTake`, `NothingToGive` and
  `LitterTooSmall` for the cases the log classed `NoVictim` and
  `EnergyConstraints`; every exported run-total key is unchanged.
- Decision: the sampled `applied_trajectory` digests hash the log text and
  follow its vocabulary; state-only digests pinned beside them enforce the
  unchanged histories; no test maps the new vocabulary to the old.
- Deferred: audit R8, the birth path on `&mut Simulation`, is paid by
  T29.F01 (G4); the `WORKAROUND(G4)` comments mark both sites.
