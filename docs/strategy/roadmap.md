# Petri — Implementation Roadmap

## Guiding Principles

- Each stage must produce a runnable and inspectable simulation state.
- Architecture is allowed to evolve when it is necessary to support higher-level goals.
- Documentation must separate current implementation behavior from planned behavior.
- Deterministic reproducibility and observability remain core constraints.

## Current Baseline (Implemented)

- Workspace crates: `petri-core`, `petri-graph`, `petri-server`, `petri-cli`, and `web`.
- Simulation currently executes graph evaluation during each tick and may execute multiple world interactions in one tick (`eat`, `move`, `reproduce`, inventory actions) when outputs exceed thresholds.
- Creature memory is currently addressable byte memory with two-stage per-tick memory addressing/write semantics.
- Stage 2 environment features up through phenotype color pipeline are present.

## Rebaseline (2026-02-13)

A new blocking Stage 2 slice is introduced before Slice 6 and Slice 7.

Reason for rebaseline:
- The project goal is to encourage evolution of richer decision-making behavior.
- Current tick semantics do not enforce single-action arbitration and do not provide explicit internal deliberation loops with a halt primitive.

This roadmap now distinguishes:
- **Current behavior**: implemented in code today.
- **Planned cognition-first behavior**: target model for the next major refactor.

---

## Stage 1: Minimal Viable Life

**Status:** Complete.

Delivered baseline capabilities include:
- world grid simulation, food economy, movement, reproduction, death
- graph-based creature controllers and mutation
- save/load snapshot pipeline
- runtime/server/web integration and inspector
- benchmark harness and ablation tooling

---

## Stage 2: Rich Environment and Cognition Foundation

**Goal:** Preserve environment-editing and observability progress while establishing a cognition-first control model for future slices.

### Slices 1-5 (Delivered)

**Slice 1: Barrier substrate + frame/render**
- Barrier cell world state and rendering.

**Slice 2: Paint API + paint UI**
- Food/barrier paint workflows and controls.

**Slice 3: Barrier-aware food rules + configurable sensing radius**
- Barrier-aware growth/spawn behavior and sensor radius controls.

**Slice 4: Barrier sensors + inspector exposure**
- Barrier direction/distance sensing and inspector visibility.

**Slice 4.1: Slot-addressed inventory + illegal-action penalties**
- Addressed slot operations, touch/slot sensors, illegal-action accounting.

**Slice 5: Phenotype color pipeline**
- Heritable phenotype color propagation and display.

### Slice 5.5 (New Blocking Slice): Cognition-First Tick Refactor

**Status:** Planned (not implemented).

**Why this is blocking:**
- Slice 6 lineage-tree UX and Slice 7 graph-inspector metrics should be built on final action semantics, not on transitional multi-action tick behavior.

**Planned behavior:**
- Energy-bounded internal think loop per creature within a tick.
- Explicit `halt` output to end think loop.
- Explicit `no_op` output for intentional no world interaction.
- One world interaction maximum per tick.
- Final-thought action selection.
- Direct introspection inputs for previous-step and running-max action confidences.
- Three energy-awareness inputs: `energy_start_tick`, `energy_spent_tick`, and `energy_remaining`.
- Movement confidence derived from `sqrt(move_x^2 + move_y^2)`.
- Random tie-break for equal final confidences using per-creature seeded RNG.

**Planned config/economics direction:**
- Introduce `energy_per_think_step` as cognition-cost knob.
- Normalize planned energy-awareness inputs against `energy_max`, with spent/remaining refreshed each think step.
- Keep throughput benchmark informational while redesign stabilizes.

### Slice 6 (Downstream of Slice 5.5): Evolutionary Tree API + Canvas Visualization

- Add lineage tree query/filter API.
- Add scalable tree visualization (`d3-hierarchy` layout + canvas render).

### Slice 7 (Downstream of Slice 5.5): Graph Inspector + Expanded Metrics

- Add interactive computation-graph inspector view (ReactFlow-based).
- Add expanded metrics: species diversity, food availability, average genome complexity.

### Stage 2 Done Criteria (Updated)

- Slices 1-5 are retained.
- Slice 5.5 cognition-first semantics are implemented and verified.
- Slice 6 and 7 are implemented on top of the cognition-first model.

---

## Stage 3: Predation

**Status:** Planned.

Predation design and balancing proceed after Slice 5.5, so predation action economics align with one-action arbitration semantics.

---

## Stage 4: Social Layer

**Status:** Planned.

Communication, kin, sharing, and extended reproduction mechanics should be introduced only after cognition-first control semantics are stable.

---

## Stage 5: Group Mechanics and Analysis

**Status:** Planned.

Large-scale analysis tooling and stress-test environments remain downstream and should assume cognition-first action semantics.

---

## Cross-Cutting Concerns

### Testing

- Unit tests: node/eval behavior, mutation behavior, energy accounting.
- Integration tests: world tick lifecycle and action resolution.
- Snapshot tests: deterministic save/load continuation.
- Benchmarking: throughput tracked continuously; strict thresholds may be relaxed during cognition refactor.

### Documentation Policy

- Canonical docs must explicitly label **Current** vs **Planned** behavior.
- Plan files in `docs/plans/` are execution artifacts and may be removed when superseded by rebaselines.

### Compatibility Policy (for upcoming Slice 5.5)

- The cognition-first refactor is expected to be a breaking semantic change.
- Old snapshots/config expectations may not remain compatible.

---

## Immediate Next Step

- Execute the documentation-only rebaseline plan.
- Then execute the cognition-first tick refactor implementation plan.
