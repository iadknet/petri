# T30.F08 — A Creature's Genome and Its Derived State Agree

**Status**: In Progress
**Last updated**: 2026-10-26
**Feature**: T30.F08
**Track**: [T30 — Core Boundaries](../../roadmaps/t30-core-boundaries.md)

## Goal

A creature's genome and everything the record computes from it at birth are
set together by one constructor and cannot drift apart: `CreatureState`'s
genome, its birth-derived state and its brain runtime are private fields
read through accessors, no constructor accepts derived state apart from the
genome it derives from, and every test that today swaps a genome or sets a
cache by hand builds the creature it needs through a construction operation.
G27 is paid and deleted. Every pin, deterministic block and existing test
value is identical to main's; any that moves is a defect.

## Non-Goals

- The birth transaction (G4, T29.F01): seeding and `spawn_child` keep
  their separate call sites, `cap_energy` after construction and the
  draft-then-mutate order.
- Renaming `GraphRuntimeState` or making its fields private (runtime's,
  T30.F02 invariant 2).
- Making the world-facing fields private (`position`, `energy`, `age`,
  `generation`, the memories, `body`, `heading`, the lifetime counters,
  `pending_birth`, `birth_*`): mechanics and tests write them by design.
- Changing which analyses the record derives, their values or when they
  are computed, or moving them into the genome type (rule 5; two are
  `sensors::reads::WorldKeySet`, above the genome).
- Any wire type or frontend change.

## Inputs and Invariants

Inputs: the T30 row and its Notes entry for F08 (G27; the user's quote), the
track [design review](../../progress/readings/t30-design-review.md) section
"T30.F08 row" (finding 1), `docs/architecture.md` rules 1 and 5, section 3
and debts G4 and G27, the T30.F02 spec (non-goal deferring the record's
field privacy to F08; invariant 2 on `GraphRuntimeState`), and the code on
`a03a3692`. Code sites, options and the preparatory audit:
[readings](../../progress/readings/t30-f08.md).

Invariants:

1. **Private fields.** `CreatureState` (`creature/state.rs`) keeps `genome`,
   `graph_runtime` and its birth-derived state private. Read accessors:
   `genome() -> &CreatureGenome`, `complexity() -> u32`,
   `genome_size() -> u32`, `reachable_nodes() -> &[usize]`,
   `has_barrier_reader() -> bool`,
   `live_vm_world_inputs() -> &[(WorldInputKey, u16)]`,
   `perception_requirements()` (as today),
   `declared_world_inputs() -> &WorldKeySet`,
   `wired_world_inputs() -> &WorldKeySet`,
   `graph_runtime() -> &GraphRuntimeState`. No `cached_*` name of the
   record is written or read outside `creature/state.rs`; in a non-test
   build, a write to a record's genome, derived state or runtime outside it
   does not compile (tests: invariants 4 and 5).
2. **One derived value, one function.** The eight analyses the record
   computes at birth (complexity, genome size, reachable nodes, barrier
   reader, live VM world inputs, perception requirements, declared and
   wired world inputs) are one value type with private fields in
   `creature/`, computed by one function from `&CreatureGenome` with the
   same eight calls in the same order as today's constructors (the
   reachable set feeds the sensor analyses), so every value is unchanged.
   The record stores one such value and delegates the accessors of
   invariant 1 to it.
3. **One constructor.** `CreatureState::new(id, genome, position, energy,
   generation, body, shared_memory)` is the only way to build a record. It
   computes the derived value from the genome it stores, takes the birth
   seeds out of that genome (`creature::newborn`) and builds the runtime
   from the genome and the seeds, as today. `new_with_cached_fields`, its
   unit test and `spawn_child`'s zero-event branch with its
   `parent_cached_*` copies are deleted; both reproduction outcomes call
   `new`. The parent's reachable set still reaches the mutation engine, now
   through `reachable_nodes()`. A proptest property test on the record
   checks that every accessor of a built record equals the analysis of its
   genome over generated genomes (a strategy such as
   `mutation/topology/f29_tests.rs:106`), replacing the deleted test.
4. **Brain operations.** No `&mut GraphRuntimeState` of a record leaves
   `creature/state.rs` in a non-test build. The record owns the operations
   the scheduler runs on its brain, thin calls on its own parts with the
   runtime's signatures unchanged (T30.F02):
   `pub(crate) fn begin_tick(&mut self)` (`tick.rs:270`);
   `pub fn think(&mut self, sensors, config) -> MeshOutput`,
   `pub(crate) fn think_traced(...)` and `pub(crate) fn think_budgeted(...,
   limit)`, each under its scheduler caller's gate (`not(telemetry-seams)`
   and `telemetry-seams`), each running the matching
   `execute_creature_mesh*` on `genome`, `energy`, `shared_memory`,
   `prev_shared_memory`, `graph_runtime` and `heading` and returning the
   runtime's output unchanged, the post-output writes staying in `tick.rs`
   (`:584-586`, `:626-628`); and `pub(crate) fn apply_reward(&mut self,
   signals)` returning the learned state's counts (`tick.rs:1358-1363`).
   `tick.rs`'s three cognition calls and `benches/tick.rs` call these (the
   bench's `Direction::N` becomes the creature's heading: a benchmark, not
   a pin). `observe_temporal_actions` and
   `observe_action_queue` read `genome()` and run the runtime on a clone of
   `graph_runtime()`. For tests only,
   `#[cfg(test)] pub(crate) fn runtime_mut(&mut self) -> &mut GraphRuntimeState`
   lets an in-crate test set per-tick or post-birth learning state on a
   constructed creature; no test assigns a whole state through it.
5. **Fixtures construct.** Every test in the readings' write table builds
   the creature it needs: a seeded simulation edits founder genomes through
   `seed_simulation_with_founder_genomes`; a fixture that must replace one
   record after seeding calls `CreatureState::new` under the same id,
   position and body (the `with_vm_founders` pattern). Learned state is
   arranged as the simulation arranges it: an inherited birth value (a
   Lamarckian learner's or a scale's seed) is a birth seed on the genome
   (`CgpGraphBackendDef::seed_birth`) before construction, which the
   constructor passes to the same `GraphRuntimeState::for_genome` the
   fixtures call today; post-birth learning state no seed carries (a
   non-Lamarckian learner's value, a trace, visitation;
   `runtime/plasticity/learned.rs:236-238,288-304`) and per-tick state
   (`node_state`, `node_outputs`, the scratch vectors, `dispatch_record`)
   are set on the constructed creature through `runtime_mut()`, and no
   learner's inheritance flag changes to fit a fixture
   (`temporal_probe_detects_each_substrate_and_preserves_live_state`,
   `tick.rs:1774-1795`, keeps `lamarckian: false`, value 1.0, trace 0.5).
   Fixtures outside `v3-core` (`server.rs`, telemetry `tests.rs`,
   `indicators/tests.rs`) need only seeds or construction from the genome
   they build. The tests that
   set `cached_genome_size` to 4, 7, 10 and 25 and `cached_complexity` to 3
   and 4 build a genome whose `genome_size()` or `complexity()` is that
   value and assert it. `final_action_observation_*` (`tick.rs:1644-1850`)
   build their creature through construction, so its perception
   requirements derive from the genome it runs. No test is deleted or
   ignored; a test whose arrangement changes keeps its assertions and
   expected values.
6. **No behavior change.** The gate's deterministic blocks are identical to
   the epoch; every existing test value a simulation produces is unchanged;
   the only new test is invariant 3's. Deleting the zero-event fast path
   changes no value: a zero-event child's genome is structurally its
   parent's, so recomputed analyses equal copied ones.
7. **Debt.** `docs/architecture.md` section 5 has no G27 row at close;
   `scripts/architecture-debts.txt` names no G27 line today and gains none.

## Architecture

Owners touched: the creature record (`creature::state`, owner of the
record's invariants and, in the desired state, of the one birth
constructor), which gains the privacy, the derived value and the brain
operations; the scheduler and mechanics (`simulation/{tick,seeding,actions}`),
which read through accessors and run the brain through those operations;
readers (`neighborhood/`), `v3-cli`, `v3-server`, `v3-telemetry` and
`v3-lab`, which read through accessors. Debt paid: G27, deleted at close.
Shared rule changed: "the genome is immutable after creation" and "derived
state agrees with the genome" have one owner, the record; today the
constructors apply them (`creature/state.rs:206-299`) and Phase 0's carrying
charge (`tick.rs:265`), move effort and predation's complexity bonus
(`creature/state.rs:178`, `simulation/actions/predation.rs`),
reproduction's parent reads (`reproduction.rs:385-407`) and the server's
creature detail (`v3-server/src/http/creature.rs`) interpret them; after the
feature visibility and one constructor enforce them and every interpreter
reads the owner's accessors. No boundary changes and no desired-state edit:
rule 1 already requires invariant-bearing state private behind owner
operations, and section 3 already places the birth constructor on the
record. G4 (T29.F01) stays in place and is used as it is: two birth call
sites and `cap_energy` after construction. Runtime's state type keeps its
public fields (T30.F02); production reaches a record's runtime only through
the brain operations, which hand runtime its own state for one call, and
tests through a `#[cfg(test)]` accessor on a constructed creature whose
genome cannot change under it.

## Telemetry

Not applicable: a visibility and construction refactor adding no signal,
config value or mechanism switch; nothing a run does changes.

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| none | — | — | — |

## Verification Plan

**Question.** F08 claims the record can hide its genome, derived state and
runtime behind one constructor and accessors, and compute every birth's
derived state from its genome, with no change to any trajectory, draw or
value and no compute cost; a moved deterministic block, a changed test
value, or a gate compute regression past its threshold would show it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate` | The track's verification note declares each row gate with pins unchanged; the gate's deterministic blocks against the epoch are the byte-identity reading, and its compute threshold is the reading that would show the deleted zero-event fast path costs something, in seconds; no population reading is in question. |
| Slow tests (`make rust-test-slow`) | `run` | The workflow's trigger names any change to `crates/v3-lab`, whose `eval.rs` and `campaign.rs` change their reads here, and the tier holds the lab's end-to-end runs, which `make check` does not run. |

## Implementation Tasks

- [x] 1. In `creature/state.rs`: the derived value type and its one function
      (invariant 2); private `genome`, `graph_runtime` and derived field;
      the accessors of invariant 1; delete `new_with_cached_fields` and its
      test; write invariant 3's record test first and make `new` the one
      constructor.
- [x] 2. The brain operations `begin_tick`, `think`, `think_traced`,
      `think_budgeted` and `apply_reward`, `graph_runtime()` and the
      `#[cfg(test)]` `runtime_mut()` (invariant 4); route
      `simulation/tick.rs`'s cognition calls, `begin_tick`, the reward
      update, the two observation paths and `benches/tick.rs` through them.
- [x] 3. `spawn_child`: delete the zero-event branch and the
      `parent_cached_*` copies; one `CreatureState::new` call; the mutation
      engine reads `reachable_nodes()`.
- [x] 4. Every production read in `v3-core`, `v3-cli`, `v3-server`,
      `v3-telemetry` and `v3-lab` goes through the accessors
      (compiler-driven; sites in the readings).
- [x] 5. The 14 test files in the readings' write table build their
      creatures as invariant 5 says; genomes of the required size and
      complexity for the Phase 0 and predation tests; birth seeds for the
      inherited values in `server.rs` and telemetry `tests.rs`; `runtime_mut()`
      for the post-birth and per-tick state in `tick.rs` and
      `reproduction.rs`.
- [ ] 6. `cargo test -p v3-core --test viability` first (tick-loop code
      changes), then `make check`.
- [ ] 7. At close: delete G27 from `docs/architecture.md`; run the declared
      gate profile and slow tier.

## Verification

- [x] Invariant 1: `grep -rnE "cached_(complexity|genome_size|reachable_nodes|has_barrier_reader|live_vm_world_inputs|perception_requirements|declared_world_inputs|wired_world_inputs)" crates`
      matches only in `crates/v3-core/src/creature/state.rs`.
- [x] Invariant 3: `cargo test -p v3-core creature::state` passes; `grep -rn new_with_cached_fields crates` is empty.
- [x] Invariant 4: `grep -rn "runtime_mut" crates` -> one `#[cfg(test)]`
      definition in `creature/state.rs`, uses in test code only; the
      whole-state assignment and `&mut ...graph_runtime` greps are empty
      outside `creature/state.rs` (commands in the readings).
- [ ] Invariants 5 and 6: `make check` exits 0, no test deleted or ignored.
- [x] Gate profile: summary at `docs/progress/features/t30-f08.json`, blocks
      equal to the epoch, raw hash and size in the readings, series entry
      added, no full report staged.
- [ ] Slow tier: `make rust-test-slow` -> exit 0 on the final code.
- [ ] Invariant 7: `make check-docs` -> exit 0 with G27 deleted.

## Performance and Goal Impact

**Predeclaration — written before the run.** One cost-bearing change: the
zero-event birth, about half of births at the default rate, recomputes the
eight analyses instead of copying three from the parent; every other path
is a field read behind an accessor. Expected compute cost: none measurable,
since births are rare beside creature-ticks, each of which runs the mesh.
The gate is compared against the current epoch
(`t27-f03-giving-energy.json`) with the standard thresholds and no re-pin.
Every deterministic block is expected identical to the epoch; a differing
block is a defect. A compute regression past the gate's threshold is not a
cost to accept: it goes to the spec owner, whose fallback (readings, options
table) keeps the reuse through an operation on the parent record. No
indicator can move.

**Measured verdict.** Gate, exit 0, `severe=false`: all 9 work counters
0.000000% and deterministic blocks equal to the epoch, as predeclared. Wall
clock flagged (+44.9%; severe is 100%) under host load 85–114, untouched
phases doubling alike; not rerun. The wall-clock reading is not attributed to
this feature; the next closure's gate on an idle host re-reads it.

- Summary of the declared profile: [gate](../../progress/features/t30-f08.json).
- Full readings: [`docs/progress/readings/t30-f08.md`](../../progress/readings/t30-f08.md).

## Success Criteria

- [ ] In a non-test build, a write to a record's genome, derived state or
      runtime outside `creature/state.rs` does not compile; tests reach the
      runtime only through `runtime_mut()` as invariants 4 and 5 bound it;
      the property test shows every built record's derived state equals the
      analysis of its genome.
- [ ] One constructor builds every creature; no constructor or operation
      accepts derived state apart from the genome it derives from.
- [ ] No test swaps a genome or sets a cache by hand; every former site
      builds its creature through a construction operation and keeps its
      assertions.
- [ ] The gate's blocks equal the epoch's, `make check` passes, G27 is deleted.

## Notes for AI Agents
