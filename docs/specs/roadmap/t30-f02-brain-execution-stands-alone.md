# T30.F02 — Brain Execution Stands Alone

**Status**: In Progress
**Last updated**: 2026-10-26
**Feature**: T30.F02
**Track**: [T30 — Core Boundaries](../../roadmaps/t30-core-boundaries.md)

## Goal

A creature thinks, spends energy and dies exactly as before, with `runtime`
owning its brain's execution state (`DispatchRecord`, `GraphRuntimeState`)
and its brain-execution trace records, one energy ledger (`ledger/`) below
runtime owning the energy primitives and accounting records every layer
reads, and the simulation composing its tick records (applied actions,
after-state, removal cause, phase timings, the creature-window recording)
around runtime's. Mutation reads the parent's dispatch history through a
query it owns. Debts G7, G8 and G16 of
[`docs/architecture.md`](../../architecture.md) are paid; every trajectory,
population and draw stream is unchanged.

## Non-Goals

- Any change to what a dispatch costs, when a creature dies, which cause is
  attributed, or what the window and sampler records carry.
- Renaming `GraphRuntimeState`, `DispatchRecord` or the record's
  `graph_runtime` field, or making the record's fields private (G27, T30.F08).
- The action-result and flooring work of T30.F06 (G18, G19, G22), the brain
  I/O vocabulary of T25.F11 (G2), the genome move of T30.F07 (G24–G26).
- Compatibility re-exports from `simulation::energy_accounting`,
  `creature::state` or `runtime::trace::recording`: one home per type.
- The frontend, the server's wire types, telemetry names or attributes.

## Inputs and Invariants

Inputs: the row and the track's F02 notes; `docs/architecture.md` section 3
and debts G7, G8, G16; the code named below; the
[readings file](../../progress/readings/t30-f02.md) (options, audit R1–R6).

The invariants state the closing state.

1. **The ledger.** `crates/v3-core/src/ledger/mod.rs` is today's
   `simulation/energy_accounting.rs` moved whole (`git mv`): `DeathCause`
   (variants, `ALL`, `as_key` and the key strings unchanged),
   `MortalityTotals`, `observe_energy_change`, `applied_debit`,
   `decay_partition`, `overdrawn_share`, `CognitionEnergyObservation`,
   `ActionCharges`, `FailedActionEffort`, `EnergyFlows` with
   `net_action_overdraft` and `record_cognition`, each with its visibility
   and its tests. It imports `contracts` only. `lib.rs` declares
   `pub mod ledger;`; `simulation::energy_accounting` no longer exists, and
   every importer in the five crates names `ledger`.
2. **Runtime state.** `runtime/state.rs` holds `DispatchRecord` and
   `GraphRuntimeState` with their fields, field visibilities, methods
   (`begin_tick`, `record_dispatch`, `executed_indices`, `is_empty`,
   `learned`, `for_genome`), doc comments and tests as they are in
   `creature/state.rs` today; it imports `contracts`, the genome and
   `runtime::plasticity`. `CreatureState.graph_runtime` keeps its name and
   is typed from it; `new_with_cached_fields` still takes the birth seeds
   out of the genome (`creature::newborn::take_birth_seeds`) and passes them
   to `for_genome`. No production file under `runtime/` imports
   `creature::state`.
3. **Shared-memory width.** `contracts::SHARED_MEMORY_SLOTS: usize = 16` is
   the one definition: `creature::state::SHARED_MEMORY_SLOTS` is deleted,
   `creature::genome::cgp::SHARED_MEMORY_SLOTS` is
   `contracts::SHARED_MEMORY_SLOTS as u8`, and every `[f32; 16]` type and
   `[0.0; 16]` literal that is a creature's shared memory in production
   code under `runtime/`, `simulation/`, `neighborhood/` and in the lab
   names the constant. The record's 64-byte size assertion stays.
4. **Mutation's query.** `mutation::reachability::ParentExecuted<'a>` has
   `Indices(&'a [usize])`, `Query(&'a dyn Fn(u64) -> Vec<usize>)` and
   `NONE`; `resolve(window)` borrows `Indices` and calls a `Query` with
   `window`, which answers with the sorted ascending indices dispatched less
   than `window` ticks before the parent's age. `Record` is gone;
   `mutation/reachability.rs` imports nothing from `runtime` or
   `creature::state`, and no file under `mutation/` imports
   `creature::state` (the G2 runtime imports elsewhere in `mutation/` are
   T25.F11's and T30.F05's and stay as they are).
   `spawn_child` (`simulation/actions/reproduction.rs:392`) passes
   `&|window| parent.graph_runtime.dispatch_record.executed_indices(parent.age, window)`;
   the engine resolves it where it resolves `Record` today
   (`mutation/engine/mod.rs:147`), after the zero-event return, so a
   zero-event birth never calls the query. `Debug` is written by hand. The
   lab sites pass the same closure; the one lab site that resolves at once
   (`campaign.rs:757`) calls `executed_indices` directly;
   `engine/tests.rs:1492` passes a closure that counts its calls and
   asserts zero on a zero-event birth.
5. **Runtime's trace.** `runtime/trace/domain.rs` keeps the brain-execution
   records: `ExecutionSample`, `TickTrace` without its `outcome` field,
   the hop, pass, backend, route, snapshot and decision-input records,
   `TerminationReason`, `PassEndReason`, `serialize_array` and `kind_label`,
   every field and serialization unchanged. `runtime/trace/size.rs` sizes
   those records (`TickTrace::heap_bytes` loses the outcome term).
   `TruncationReason` lives in `runtime/traced_mesh.rs` beside
   `RecordLimit`; `runtime/trace/recording.rs` is deleted. No production
   file under `runtime/` imports `creature::action_log`, `simulation` or
   `creature::state`.
6. **The simulation's recording.** `simulation/recording.rs` holds
   `RecordedTick { trace: TickTrace, outcome: Option<TickOutcome> }` (the
   `outcome` field and the three outcome types under `telemetry-seams`, as
   today), `TickOutcome`, `AppliedAction`, `StateAfter`, `ActiveTrace`
   (fields as today, `ticks: Vec<RecordedTick>`, `into_sample` mapping each
   tick to its `trace`), `TraceBudget`, `Truncation`, `TickReserve` and
   the outcome's size accounting (`TickOutcome::heap_bytes`,
   `RecordedTick::{retained_bytes, heap_bytes}` = the trace's plus the
   outcome's). `run_tick` pushes a `RecordedTick` whose `outcome` the
   cognition phase builds and `window_seam::finish_tick` fills, as today;
   `SimStats::observed_actions` is `Vec<AppliedAction>` from this module.
   The tick buffer's slot is `size_of::<RecordedTick>()`, charged where
   `size_of::<TickTrace>()` is charged today (the two may differ by
   padding; a recording's event count and heap bytes are unchanged);
   `TickReserve::for_config` keeps its formula.
7. **Outer crates.** `v3-cli`, `v3-server`, `v3-telemetry` and `v3-lab`
   change import paths and read `tick.trace.<field>` where they read a
   recorded tick's hops, passes or final actions today; no wire type,
   payload, metric, span attribute, record body or lab summary field
   changes name, and every value is unchanged except where a byte-capped
   window's truncation tick moves with the slot padding invariant 6
   allows (`window_seam.rs:109` charges the actual slot size; the window's
   `petri.events`, truncation attributes and body follow). The server's
   sampler fixtures
   (`v3-server/src/transport/sample_assembler.rs:206-219`) name every
   `TickTrace` field themselves: the `recorded_tick()` helper that ran a
   simulation to supply the seam-only `outcome`, its `..recorded_tick()`
   struct updates, the `needless_update` allowances and their comments
   are deleted (audit R6).
8. **Checker and debts.** `scripts/architecture-debts.txt` loses every G7,
   G8 and G16 line (fourteen lines: seven, five and two) and gains none;
   `scripts/architecture-check.mjs` passes unchanged (it already places
   `ledger` at level 2).
9. **No behavior change.** No `SimulationConfig` field, RNG draw, catalog
   key, death-cause key, wire type or frontend file changes. Every
   `deterministic` value and work counter of the gate summary equals the
   reference's, and the accounting, dispatch-record and window-seam tests
   of the Verification section pass unchanged apart from paths.
10. **Overlap.** T30.F06 also changes the ledger file and the action
    flooring in `simulation/tick.rs`; T30.F08 and T29.F01 also change
    `creature/state.rs`; concurrent sessions reconcile those files at
    integration (track note).

## Architecture

Owners touched: the energy ledger (new, the module G7 names moved below
runtime), `contracts` (one brain I/O width), genome (its slot constant
derives from contracts), `mutation` (the query it owns), `runtime` (its
state and its trace records), `creature` (the record types its brain state
from runtime), the scheduler (the recording it composes), readers and outer
crates (import paths and one field hop). Debts paid: G7, G8 and G16 (section
5 rows at closure, the debts-file lines in the build). Debts used and left:
G2, in place and not widened (`runtime::types::OUTPUT_SLOT_COUNT` and the
genome vocabulary stay where they are); G27, since `GraphRuntimeState`'s
fields keep their visibility and the tests that set them by hand
(`simulation/tick.rs:1646`, `reproduction.rs:1125`) keep doing so until
T30.F08; G24, so runtime imports the genome under `creature/genome/`. Shared
rules read, not changed: the pending-cause rule (`observe_energy_change`)
and the debit arithmetic keep their one owner, now the ledger, and
`CreatureState::observe_energy` (`creature/state.rs:317`) and the tick's
`Phase0Charge::settle` (`simulation/tick.rs:189`) keep calling it; the
executed-window rule stays `DispatchRecord::executed_indices`, which the
query wraps. `docs/architecture.md` already describes this shape (section 3's
order with `ledger/` at level 2, the runtime and simulation cells of the
owner table), so the plan commit does not edit it.

## Telemetry

Not applicable: the feature changes nothing a run does; the window spans,
sample payloads, death-cause keys and energy-flow metrics keep their names,
sources and values, apart from the byte-capped window truncation tick
invariant 7 qualifies.

## Verification Plan

**Question.** This feature claims every creature thinks, spends and dies
exactly as before while runtime, the ledger and the simulation each own
their records; a gate `deterministic` value or work counter differing from
the reference, or a moved trace or accounting test value, would show it
wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate` | tick-loop code moves (dispatch, energy observation, recording) with no population reading in question: the gate pins the deterministic blocks and the compute counters. |
| Slow tests (`make rust-test-slow`) | `run` | `crates/v3-lab` changes (import paths, the query at its mutation sites), and the workflow runs the tier for any change to the lab; no operator, draw, catalog or recruitment change. |

## Implementation Tasks

- [ ] `git mv crates/v3-core/src/simulation/energy_accounting.rs
      crates/v3-core/src/ledger/mod.rs`; `pub mod ledger;` in `lib.rs`;
      the module declaration removed from `simulation/mod.rs`; every
      importer in the five crates on the new path (R1; invariant 1).
- [ ] `contracts::SHARED_MEMORY_SLOTS`; the genome's constant derived from
      it; the creature-side constant deleted; the array types on the
      constant (R5; invariant 3).
- [ ] `runtime/state.rs` with the two types and their tests moved from
      `creature/state.rs`; `creature/state.rs` and the readers, the lab and
      runtime's own files import it (R2; invariant 2).
- [ ] `ParentExecuted::Query`; `Record` deleted; `spawn_child`, the engine
      test and the lab sites on the closure (R3; invariant 4).
- [ ] `simulation/recording.rs` with the recording, the outcome types, the
      reserve and the outcome sizes; `TickTrace` without `outcome`;
      `TruncationReason` in `traced_mesh.rs`; `runtime/trace/recording.rs`
      deleted; `tick.rs`, `window_seam.rs`, `stats.rs`, the readers and the
      outer crates on the new types (R4; invariants 5–7).
- [ ] `v3-server/src/transport/sample_assembler.rs` tests: `recorded_tick()`,
      the struct updates, the allowances and their comments deleted; each
      fixture names its `heading` (R6; invariant 7).
- [ ] `scripts/architecture-debts.txt`: the fourteen G7, G8 and G16 lines
      removed; `scripts/architecture-check.mjs` green (invariant 8).
- [ ] At closure: G7, G8 and G16 rows deleted from `docs/architecture.md`;
      the gate run and its summary recorded.

## Verification

- [ ] `cargo test -p v3-core --test viability` -> result (run first).
- [ ] Focused tests: the ledger's property tests, the moved
      `runtime::state` tests, `mutation::engine::tests::a_zero_event_birth_never_resolves_the_parent_dispatch_record`,
      `simulation/tick/tests/{dispatch_record,window_seam,energy_accounting}.rs`,
      the v3-telemetry `windows` tests and the v3-server sampler tests ->
      result.
- [ ] Workspace tests, clippy, fmt, `cargo check --all-targets`, the
      architecture and roadmap checkers -> result.
- [ ] `make check` -> exit code and commit; `make rust-test-slow` -> exit
      code.
- [ ] Gate: summary at `docs/progress/features/t30-f02.json`, local raw
      hash, byte count and verification time checked, series entry points
      to it, no full report staged; every `deterministic` value and work
      counter equal to the reference's. Goal: `Not run: Verification Plan`.

## Performance and Goal Impact

**Predeclaration — written before the run.** A refactor with no mechanism
and no natural analog to name. Expected cost: none. The moves change
paths, not code: the query is one indirect call per birth that draws an
event, replacing a match on the same data; `RecordedTick` is today's
`TickTrace` with its outcome beside it rather than inside it, built only
for the one traced creature. Reference: the gate's selected reference, the
last measured closure's summary; thresholds: the gate's own. Expected
direction: the nine work counters and every `deterministic` value
identical; wall clock neutral within the gate's noise, a regression there
attributed before closure. No indicator can move; nothing is re-pinned.

**Measured verdict.** Not yet run.

- Summary: [gate](../../progress/features/t30-f02.json).
- Readings: [`t30-f02.md`](../../progress/readings/t30-f02.md).

## Success Criteria

- [ ] `crates/v3-core/src/ledger/mod.rs`, `runtime/state.rs` and
      `simulation/recording.rs` exist with the contents of invariants 1, 2
      and 6; `simulation/energy_accounting.rs` and
      `runtime/trace/recording.rs` do not.
- [ ] No production file under `runtime/` or `mutation/` imports
      `creature::state`, `creature::action_log` or `simulation`, and
      `scripts/architecture-debts.txt` holds no G7, G8 or G16 line.
- [ ] The tests of the Verification section pass, and `make check` and
      `make rust-test-slow` exit 0.
- [ ] The gate's `deterministic` values and work counters equal the
      reference's.
- [ ] `docs/architecture.md` section 5 no longer lists G7, G8 or G16.

## Notes for AI Agents

- Decision: in-feature refactors R1–R6 of the preparatory audit in
  `docs/progress/readings/t30-f02.md` are this feature's scope.
- Decision: the record's `graph_runtime` field and `GraphRuntimeState`'s
  field visibility are unchanged; T30.F08 (G27) owns them.
