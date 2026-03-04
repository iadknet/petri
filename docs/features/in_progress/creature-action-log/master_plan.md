**Goal:** Add per-creature action log ring buffer with rich metadata, exposed via server API for creature inspection.
**Goal IDs:** GP-04, GP-02
**Scope:** v3-core (action log struct, ring buffer, tick integration) and v3-server (API endpoint). No frontend changes.
**Docs Impact:** `docs/reference/v3-evolution-observability-spec.md` (add action log section)
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- **GP-04** (Keep Behavior Observable): The action log is a direct observability feature — it records per-creature action history with rich metadata (result, energy delta, direction, amounts) for debugging and behavioral analysis.
- **GP-02** (Maintain Clean Architecture Boundaries): The log types live in `v3-core::creature` (data definitions), action logs are stored in a parallel `SlotMap` on `Simulation` (keeping `CreatureState` pure simulation state), recording happens in `v3-core::simulation::tick` (the action execution owner), and the API surface lives in `v3-server::handlers`. No boundary violations.

## Boundary Impact

- **v3-core::creature**: New `ActionLog` and `ActionLogEntry` types added to creature module. These are data definitions only — no new field on `CreatureState`.
- **v3-core::simulation**: `Simulation` struct gains a parallel `SecondaryMap<CreatureId, ActionLog>` field (`action_logs`). This keeps observational infrastructure separate from simulation state. Lifecycle managed at spawn (insert empty log) and death (remove log).
- **v3-core::simulation::tick**: Phase 2 action execution records entries after each action — tick owns action application, so it owns log recording. Writes to `sim.action_logs` instead of creature state.
- **v3-core::config**: New `ActionLogConfig` with `capacity` field added to `SimulationConfig`.
- **v3-server::handlers**: Extended creature endpoint returns action log data for selected creature.
- **No changes** to: kernel, sensors, runtime, contracts, mutation, `CreatureState`.

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `v3-core::creature` | keep | CreatureState remains pure simulation state. Action log types (ActionLogEntry, ActionLog) are defined in the creature module as data types but not added as fields on CreatureState. |
| `v3-core::simulation` | keep | Simulation gains a parallel `action_logs: SecondaryMap<CreatureId, ActionLog>` for observational state. Keeps observation separate from simulation state, following the same pattern as `stats: SimStats`. |
| `v3-core::simulation::tick` | keep | Tick Phase 2 owns action execution and outcome recording. Log recording is a natural extension of the existing `outcome_acc.record_action_result()` pattern. |
| `v3-core::contracts` | keep | WorldAction and result enums already exist. ActionLogEntry references them by value, not by adding to contracts. |
| `v3-server::handlers::creature` | keep | Existing `get_creature` endpoint returns per-creature detail. Action log is added to this response within existing boundary. |

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Default ring buffer capacity? | 500 entries (not ticks — a creature with 3 actions/tick fills faster). Configurable via `ActionLogConfig.capacity`. | agent | resolved |
| Fixed struct or generic/extensible? | Fixed struct `ActionLogEntry`. The action set is known and stable. If actions are added later, the struct is extended then. | agent | resolved |
| Log NoOp actions? | Yes. NoOps show "idle" patterns and are cheap to store. | agent | resolved |
| Interaction with outcome/reward system? | Independent. OutcomeRecord is computed separately for plasticity. The log is purely observational. | agent | resolved |
| Per-action or per-tick granularity? | Per-action. Multiple actions per tick are stored as separate entries, each with the tick number. | agent | resolved |
| Reproduction inheritance semantics? | Not inherited. A new empty ActionLog is inserted into `sim.action_logs` at spawn. Removed at death. A creature's log records only its own lifetime. | agent | resolved |
| Where do action logs live? | Parallel `SecondaryMap<CreatureId, ActionLog>` on `Simulation`, not on `CreatureState`. Keeps CreatureState as pure simulation state and avoids touching every test fixture that constructs creatures. | agent | resolved |
| Ring buffer implementation? | `VecDeque<ActionLogEntry>` with manual cap enforcement (pop_front when at capacity). Simple, no external deps, cache-friendly. | agent | resolved |
| ActionLogEntry size? | Target exactly 32 bytes. Use `#[repr(u8)]` enums for action type and result (type-safe, 1 byte each), `u8` for direction, `f32` for energy/amounts. Assert size with compile-time `const _: () = assert!(size_of::<ActionLogEntry>() == 32);`. | agent | resolved |

## Required Skills

- Rust/backend changes: invoke `rust-skills` BEFORE writing any Rust code and before each review

## TDD Policy

For all behavior changes and bug fixes: write a failing test FIRST, then implement.
A step is not complete until:
1. The failing test exists and is committed
2. The implementation makes it pass
3. No existing tests regress

## Code Review Policy

After completing each implementation step:
1. Run a thorough code review (backend: `rust-skills`)
2. Fix ALL findings
3. Run review AGAIN — repeat until no new findings (clean recursive pass)
4. Only after clean pass: commit the step

## Commit Policy

- Commits happen AFTER a clean code review pass, never before
- One commit per implementation step (focused, atomic)
- Do NOT advance to the next step until current step is committed and reviewed clean

## Design

### ActionLogEntry struct

```rust
/// Action type discriminant for log entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[repr(u8)]
pub enum ActionType {
    NoOp = 0,
    Eat = 1,
    Move = 2,
    Reproduce = 3,
    StealEnergy = 4,
}

/// Outcome of an action for log entries. Covers all action-specific result variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[repr(u8)]
pub enum ActionResult {
    Success = 0,
    NoFood = 1,
    Blocked = 2,
    InvalidTarget = 3,
    EnergyConstraints = 4,
    PopulationCap = 5,
    TransferredAndKilled = 6,
    NoVictim = 7,
}

/// Single action record in a creature's action log.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct ActionLogEntry {
    /// Simulation tick when this action was executed.
    pub tick: u64,
    /// Action type.
    pub action_type: ActionType,
    /// Action result.
    pub result: ActionResult,
    /// Direction parameter (0-7 for cardinal+diagonal, 255=N/A).
    pub direction: u8,
    /// Creature energy BEFORE this action was applied.
    pub energy_before: f32,
    /// Creature energy AFTER this action was applied (includes costs and penalties).
    pub energy_after: f32,
    /// Action-specific amount:
    /// - Eat: food consumed
    /// - Reproduce: energy transferred to offspring
    /// - StealEnergy: energy actually stolen
    /// - Move/NoOp: 0.0
    pub amount: f32,
    /// Priority bid value for this tick (same for all actions in a tick).
    pub priority_bid: f32,
}
```

Size: 4×f32 (16) + u64 (8) + 3×u8 (3) + padding = 32 bytes. Assert exactly 32 at compile time with `const _: () = assert!(size_of::<ActionLogEntry>() == 32);`.

### ActionLog struct

```rust
/// Ring-buffer action log for a single creature.
pub struct ActionLog {
    entries: VecDeque<ActionLogEntry>,
    capacity: usize,
}
```

Methods: `new(capacity)` (uses `VecDeque::with_capacity(capacity)` to pre-allocate), `push(entry)` (auto-evicts oldest), `entries() -> &VecDeque<ActionLogEntry>`, `len()`, `is_empty()`.

### Config

```rust
/// Action log configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActionLogConfig {
    /// Maximum entries per creature. Default: 500.
    pub capacity: usize,
}
```

Added to `SimulationConfig` as `pub action_log: ActionLogConfig` with `#[serde(default)]`.

### Storage on Simulation

Action logs live in a parallel `SecondaryMap<CreatureId, ActionLog>` on `Simulation`:

```rust
pub struct Simulation {
    pub world: WorldState,
    pub creatures: SlotMap<CreatureId, CreatureState>,
    pub action_logs: SecondaryMap<CreatureId, ActionLog>,  // NEW — slotmap::SecondaryMap
    pub tick: u64,
    pub config: SimulationConfig,
    pub stats: SimStats,
    pub(crate) rng: SmallRng,
}
```

This keeps `CreatureState` as pure simulation state. Lifecycle:
- **Spawn** (seeding + reproduction): insert `ActionLog` keyed to match the creature's `CreatureId`.
- **Death** (Phase 0 removal + predation kill): remove corresponding `action_logs` entry.

`SecondaryMap` is the idiomatic slotmap companion type — it accepts keys from the primary `SlotMap<CreatureId, CreatureState>` and validates them against it. Insert with `action_logs.insert(creature_id, log)` at spawn sites; remove with `action_logs.remove(creature_id)` at death sites.

### Recording in tick.rs

In Phase 2, after each action is applied and outcome recorded, build an `ActionLogEntry` and push it to `sim.action_logs[id]`. The `energy_before` is captured before calling `apply_*`, and `energy_after` is read after (including penalty). The `priority_bid` is available from the `MeshOutput`.

### Server API

Extend `get_creature` response with an `"action_log"` field containing the serialized entries array. The entry struct gets `serde::Serialize` for JSON transport. The handler reads from `sim.action_logs` using the same creature ID.

## Implementation Steps

- [x] Step 1: Define `ActionType` enum, `ActionResult` enum, `ActionLogEntry` struct, and `ActionLog` ring buffer in new file `v3/crates/v3-core/src/creature/action_log.rs`. Include compile-time size assertion (`size_of::<ActionLogEntry>() == 32`). Write unit tests for ring buffer behavior (push, eviction at capacity, pre-allocation with `VecDeque::with_capacity`).
- [x] Step 2: Add `ActionLogConfig` to `v3/crates/v3-core/src/config/simulation.rs` with default. Add `action_logs: SecondaryMap<CreatureId, ActionLog>` field to `Simulation`. Update `Simulation::new()` to initialize empty `SecondaryMap`. Update spawn sites (seeding, reproduction) to insert empty `ActionLog` keyed to the creature's ID. Update death sites (Phase 0 removal, predation kill) to remove corresponding log entry.
- [x] Step 3: Record action log entries in `tick.rs` Phase 2. For each action branch, capture `energy_before`, execute action, capture `energy_after` (including penalty), build `ActionLogEntry`, push to `sim.action_logs[id]`. Write integration test: run a tick, verify creature has log entries with correct action types and energy deltas.
- [x] Step 4: Extend `get_creature` handler in `v3-server` to include `action_log` field in JSON response (read from `sim.action_logs`). `serde::Serialize` is already derived on `ActionLogEntry`/`ActionType`/`ActionResult` from Step 1. Write server test verifying the endpoint returns action log data.
- [x] Review Gate: Interim code review — review Steps 1-4 changes. Invoke `rust-skills`. Fix findings, re-review until clean.
- [x] Step 5: Run viability tests (`cargo test -p v3-core --test viability`) and full workspace tests. Fix any regressions.
- [ ] Review Gate: Code review — dispatch `superpowers:code-reviewer` subagent on full branch diff. Invoke `rust-skills`. Fix all findings. Re-review until clean pass.
- [ ] Review Gate: Architecture & decomposition review — review all changes for boundary violations, decomposition opportunities, separation of concerns. Re-read `docs/strategy/` and relevant `AGENTS.md` files. Fix easy issues, capture larger items in `docs/features/brainstorms/ideas.md`. Repeat until clean pass.
- [ ] Completion gate — run all checks from AGENTS.md Completion Gate section

**Review cycles:** 3 (cycle 1: initial plan with action log on CreatureState. Cycle 2: moved to parallel SlotMap on Simulation for cleaner separation. Cycle 3: rust-skills review — replaced raw u8 fields with `#[repr(u8)]` enums per `type-no-stringly`, fixed SlotMap→SecondaryMap per slotmap API semantics, pinned size assertion to exactly 32, noted VecDeque pre-allocation — clean pass)
