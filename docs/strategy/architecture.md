# Petri — Architecture Design Document

## Purpose

This document defines the implemented repository architecture and current simulation semantics.

**Goal IDs:** `GP-01`, `GP-02`, `GP-03`, `GP-04`

## Goal Alignment

- `GP-01`: Keep decision mechanics deterministic and testable, including seeded tie-break behavior.
- `GP-02`: Keep crate boundaries and ownership explicit.
- `GP-03`: Keep semantics regression-resistant through contract-aligned tests across crates.
- `GP-04`: Surface per-creature diagnostics for cognition and action arbitration.

## Boundary Impact

- Crate direction: `petri-graph -> petri-core -> petri-server/petri-cli`.
- `petri-graph` owns controller representation, evaluation, and mutation mechanics.
- `petri-core` owns simulation lifecycle and action/arbitration policy.
- `petri-server` owns REST/WebSocket/runtime control contracts.
- `web` owns typed protocol consumption and visualization.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `petri-core` tick/arbitration ownership | `keep` | Cognition-first loop and final-action selection remain simulation policy concerns. |
| `petri-server` payload/config ownership | `keep` | Runtime/startup patch semantics and creature-detail wire contracts belong in transport boundary. |
| `petri-graph` controller I/O ownership | `keep` | Halt/no-op and introspection channels belong in graph representation/evaluation, not in server/web layers. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should cognition diagnostics include full per-step traces in stable payloads? | Keep stable payloads summary-only for now; defer trace expansion to future tooling slices if required. | `petri-server` maintainers | `resolved` |

## Repository Architecture

```
petri/
├── crates/
│   ├── petri-core/      # world state, cognition-first tick lifecycle
│   ├── petri-graph/     # controller node types, evaluation, mutation helpers
│   ├── petri-server/    # REST + WebSocket runtime wrapper around petri-core
│   └── petri-cli/       # headless runner, ablation, benchmark tools
├── web/                 # React + TypeScript client
├── docs/
└── README.md
```

## Core Runtime Model

### World and Creature

- World grid stores food density and barrier occupancy.
- Creature state includes position, energy, controller graph, memory register, inventory slots, lineage IDs, per-creature RNG, and inspector/debug caches.
- Creature state also includes cognition diagnostics:
  - `think_steps`
  - `halted`
  - `selected_action`
  - `selected_confidence`

### Tick Lifecycle (Implemented)

Per creature, per tick:
1. Age increment and passive energy charge.
2. Internal think loop:
   - stage A evaluate for memory address
   - memory read
   - stage B evaluate for outputs
   - optional memory write
   - introspection input refresh
   - `energy_per_think_step` charge
3. Stop on `halt`, energy exhaustion, or one-step fallback for graphs without `OutputHalt`.
4. Final-thought arbitration across six action candidates:
   - `move`, `eat`, `reproduce`, `inventory_pickup`, `inventory_put`, `no_op`
5. Execute at most one world interaction path.
6. Apply normal legality checks/penalties and death/reproduction handling.

Arbitration notes:
- Movement confidence uses vector magnitude `sqrt(move_x^2 + move_y^2)`.
- Exact confidence ties are broken with per-creature seeded RNG for deterministic replay under fixed seeds/snapshots.

## Data Contracts and Observability

Implemented contract surfaces:
- Controller I/O includes cognition channels:
  - inputs: previous-step and running-max action confidences, plus `energy_start_tick`, `energy_spent_tick`, `energy_remaining`
  - outputs: `halt`, `no_op`
- Inspector payload (`CreatureDetail`) includes cognition diagnostics and expanded I/O state.
- Snapshot payload includes cognition diagnostics for each creature.

Compatibility stance:
- Snapshot import intentionally rejects legacy payloads missing required cognition diagnostics.

## Performance and Verification Posture

- Benchmark tooling (`stage1_benchmark`) remains active.
- Throughput is tracked but treated as informational while cognition semantics stabilize.
- Quality gates prioritize correctness, deterministic behavior, and contract consistency across Rust and web layers.
