# Phase 3: Advanced Tracing (Tick Phases & Creature Cognition)

**Parent plan:** `2026-03-20-otel-observability.md`

**BLOCKED:** This phase requires the tick-phase-system refactor (`docs/features/needs_refinement/refactors/tick-phase-system.md`) to be completed first. All phases must be exposed as separate callable functions from `v3-core` before the server can wrap them with tracing spans.

---

### Task 11: Tick Phase Spans

**Prerequisites:** Tick-phase-system refactor complete — `run_phase_0()`, `run_phase_1()`, `run_phase_2()`, `run_phase_2_5()` exposed by v3-core.

**Files:**
- Modify: `v3/crates/v3-server/src/observability/tracing_spans.rs` (add phase span helpers)
- Modify: `v3/crates/v3-server/src/handlers/lifecycle.rs` (wrap phase calls with spans)

- [ ] **Step 1: Add phase span helpers**

```rust
// In tracing_spans.rs — add helpers for each phase span

pub fn phase_0_span() -> tracing::Span {
    tracing::info_span!("phase_0_world_updates")
}

pub fn food_growth_span() -> tracing::Span {
    tracing::info_span!(
        "food_growth",
        "food.spawned" = tracing::field::Empty,
        "food.total_after" = tracing::field::Empty,
    )
}

pub fn death_removal_span() -> tracing::Span {
    tracing::info_span!(
        "death_removal",
        "deaths.total" = tracing::field::Empty,
    )
}

pub fn queue_build_span() -> tracing::Span {
    tracing::info_span!(
        "queue_build",
        "queue.size" = tracing::field::Empty,
    )
}

pub fn phase_1_span() -> tracing::Span {
    tracing::info_span!("phase_1_cognition")
}

pub fn sensor_assembly_span() -> tracing::Span {
    tracing::info_span!("sensor_assembly")
}

pub fn mesh_execution_span() -> tracing::Span {
    tracing::info_span!(
        "mesh_execution",
        "creatures.executed" = tracing::field::Empty,
        "creatures.traced" = tracing::field::Empty,
    )
}

pub fn decision_sorting_span() -> tracing::Span {
    tracing::info_span!(
        "decision_sorting",
        "decisions.count" = tracing::field::Empty,
        "priority_bid.mean" = tracing::field::Empty,
    )
}

pub fn phase_2_span() -> tracing::Span {
    tracing::info_span!("phase_2_actions")
}

pub fn phase_2_5_span() -> tracing::Span {
    tracing::info_span!("phase_2_5_learning")
}
```

- [ ] **Step 2: Wrap phase calls in lifecycle.rs**

Replace the monolithic `run_tick()` call with individual phase calls wrapped in spans. The exact API depends on how the tick-phase-system refactor exposes phases:

```rust
// Pseudocode — actual API will be determined by tick-phase refactor
{
    let _span = phase_0_span().entered();
    {
        let _sub = food_growth_span().entered();
        sim.run_food_growth();
        // record food.spawned, food.total_after on _sub
    }
    // ... creature_aging, energy_decay, shared_memory_decay ...
    {
        let _sub = death_removal_span().entered();
        let deaths = sim.run_death_removal();
        _sub.record("deaths.total", deaths);
    }
}

{
    let _span = queue_build_span().entered();
    let queue = sim.build_turn_queue();
    _span.record("queue.size", queue.len() as u32);
}

{
    let _span = phase_1_span().entered();
    // ... sensor_assembly, mesh_execution, decision_sorting sub-spans ...
}

{
    let _span = phase_2_span().entered();
    sim.run_action_execution(&queue);
}

{
    let _span = phase_2_5_span().entered();
    sim.run_reward_learning();
}
```

- [ ] **Step 3: Verify phase spans appear in SigNoz**

Run simulation, check SigNoz for the tick → phase_0 → food_growth hierarchy.

- [ ] **Step 4: Commit**

```bash
git add v3/crates/v3-server/src/observability/ v3/crates/v3-server/src/handlers/lifecycle.rs
git commit -m "feat: add tick phase tracing spans with sub-phase decomposition"
```

---

### Task 12: Phase Timing Metrics

**Files:**
- Modify: `v3/crates/v3-server/src/observability/metrics.rs` (add phase timing histograms)
- Modify: `v3/crates/v3-server/src/handlers/lifecycle.rs` (measure phase wall-clock times)

- [ ] **Step 1: Measure wall-clock per phase**

In `lifecycle.rs`, wrap each phase call with `Instant::now()` / `elapsed()`:

```rust
let phase_0_start = Instant::now();
// ... run phase 0 ...
let phase_0_ms = phase_0_start.elapsed().as_secs_f64() * 1000.0;
```

- [ ] **Step 2: Record phase timing histograms**

```rust
meter.f64_histogram("petri.tick.duration_ms")
    .build()
    .record(total_tick_ms, &attrs);
meter.f64_histogram("petri.tick.phase_0_duration_ms")
    .build()
    .record(phase_0_ms, &attrs);
// ... phases 1, 2, 2.5 ...
```

- [ ] **Step 3: Verify histograms in SigNoz**

Check SigNoz Metrics Explorer for `petri.tick.phase_0_duration_ms`.

- [ ] **Step 4: Commit**

```bash
git commit -m "feat: add phase timing histogram metrics"
```

---

### Task 13: Batch Traced Execution in Core

**Prerequisites:** Tick-phase-system refactor complete, Phase 1 cognition exposed as a callable function.

**Files:**
- Modify: `v3/crates/v3-core/src/runtime/trace/recording.rs` (or new file for batch API)
- Modify: `v3/crates/v3-core/src/simulation/tick.rs` (batch traced execution in Phase 1)

- [ ] **Step 1: Write failing test for batch traced execution**

```rust
#[test]
fn batch_traced_execution_returns_traces_for_specified_creatures() {
    // Set up simulation with N creatures
    // Specify a subset of creature IDs to trace
    // Run Phase 1 with batch tracing
    // Assert: traces returned for specified creatures only
    // Assert: all creatures (traced and untraced) have valid MeshOutput
}
```

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement batch traced execution**

The Phase 1 cognition function needs to accept a `HashSet<CreatureId>` of creatures to trace. For traced creatures, it runs `execute_creature_mesh_traced()` and collects the trace data. For untraced creatures, it runs the normal `execute_creature_mesh()`.

Key design considerations:
- Traced creatures must be extracted from the parallel pass (since `execute_creature_mesh_traced()` is sequential) OR the traced path must be made parallel-safe
- The existing `execute_creature_mesh_traced()` runs sequentially — for ~100 creatures this is acceptable
- Strategy: partition creatures into traced and untraced sets. Run untraced in `par_iter_mut`, run traced sequentially. Merge results.

```rust
pub struct BatchTraceResult {
    pub traces: Vec<(CreatureId, Vec<MeshHopTrace>, TerminationReason)>,
}

pub fn run_phase_1_with_tracing(
    sim: &mut Simulation,
    trace_ids: &HashSet<CreatureId>,
) -> BatchTraceResult {
    // 1. Partition creatures into traced / untraced
    // 2. Run untraced creatures in parallel (existing par_iter_mut path)
    // 3. Run traced creatures sequentially with execute_creature_mesh_traced()
    // 4. Merge decisions into single queue sorted by priority bid
    // 5. Return BatchTraceResult with trace data
}
```

- [ ] **Step 4: Run test to verify it passes**

- [ ] **Step 5: Commit**

```bash
git commit -m "feat: add batch traced execution mode to Phase 1 cognition"
```

---

### Task 14: Creature Cognition Span Emission

**Files:**
- Create: `v3/crates/v3-server/src/observability/creature_spans.rs`
- Modify: `v3/crates/v3-server/src/observability/mod.rs` (add module)
- Modify: `v3/crates/v3-server/src/handlers/lifecycle.rs` (emit spans from trace results)

- [ ] **Step 1: Create creature span emission module**

```rust
// v3/crates/v3-server/src/observability/creature_spans.rs

use v3_core::runtime::trace::domain::{MeshHopTrace, TerminationReason};
use v3_core::contracts::CreatureId;

/// Convert a creature's trace data into OTel spans.
/// Creates the creature_tick parent span with child spans for
/// sense, think (with per-node mesh_node spans), decide, and action outcomes.
pub fn emit_creature_trace_spans(
    creature_id: CreatureId,
    trace_data: &[MeshHopTrace],
    termination: &TerminationReason,
    run_id: &str,
    tick_number: u64,
    trace_reason: &str,
    // Additional creature state: energy, age, phenotype, etc.
) {
    let _creature_span = tracing::info_span!(
        "creature_tick",
        "simulation.run_id" = run_id,
        "tick.number" = tick_number,
        "creature.id" = ?creature_id,
        "creature.trace_reason" = trace_reason,
        // ... more attributes from creature state ...
    ).entered();

    // Emit creature_think span with per-node child spans
    {
        let _think_span = tracing::info_span!(
            "creature_think",
            "think.nodes_executed" = trace_data.len() as u32,
        ).entered();

        for hop in trace_data {
            let _node_span = tracing::info_span!(
                "mesh_node",
                "node.id" = hop.node_id,
                "node.execution_order" = hop.hop_index,
                "node.energy_cost" = (hop.energy_before - hop.energy_after),
                // Map hop.upstream_slots, hop.output_slots to attributes
                // Map hop.backend_trace to backend-specific attributes
            ).entered();
            // Span closes on drop
        }
    }

    // Emit termination reason as span event if notable
    match termination {
        TerminationReason::MaxHopsReached => {
            tracing::warn!(event_type = "mesh_hop_limit_hit");
        }
        TerminationReason::EnergyExhausted => {
            tracing::warn!(event_type = "energy_exhausted_during_execution");
        }
        _ => {}
    }
}
```

Note: This is a skeleton. The actual implementation needs to:
- Map `MeshHopTrace` fields to the span attributes defined in the spec
- Handle VM vs graph backend traces differently
- Extract input_sources and output_targets from the hop data
- Map routing decisions from `hop.route`
- Include creature_sense, creature_decide, and creature_action spans (data sources for these come from different parts of the tick — sensor inputs, final actions, action outcomes)

- [ ] **Step 2: Wire into lifecycle.rs**

After Phase 1 returns `BatchTraceResult`, emit spans for each traced creature:

```rust
#[cfg(feature = "otel")]
for (creature_id, hops, termination) in &batch_result.traces {
    crate::observability::creature_spans::emit_creature_trace_spans(
        *creature_id,
        hops,
        termination,
        &handle.run_id,
        handle.sim.tick,
        if handle.otel_config.flagged_creature_ids.contains(creature_id) { "flagged" } else { "sampled" },
    );
}
```

- [ ] **Step 3: Verify creature spans in SigNoz**

Run simulation with tracing enabled. Check SigNoz for `creature_tick` → `creature_think` → `mesh_node` span hierarchy.

- [ ] **Step 4: Commit**

```bash
git commit -m "feat: emit creature cognition OTel spans from trace data"
```

---

### Task 15: Sampling Integration

**Files:**
- Modify: `v3/crates/v3-server/src/handlers/lifecycle.rs` (sampling logic)

- [ ] **Step 1: Implement sampling decision**

Before Phase 1, build the set of creatures to trace:

```rust
#[cfg(feature = "otel")]
let trace_ids: HashSet<CreatureId> = if handle.otel_config.enabled {
    let mut ids = handle.otel_config.flagged_creature_ids.clone();
    // Add random sample
    let sample_rate = handle.otel_config.creature_sample_rate;
    if sample_rate > 0.0 {
        let mut rng = rand::thread_rng();
        for (id, _) in &handle.sim.creatures {
            if rng.gen::<f64>() < sample_rate && !ids.contains(&id) {
                ids.insert(id);
            }
        }
    }
    ids
} else {
    HashSet::new()
};
```

- [ ] **Step 2: Pass trace_ids to Phase 1**

```rust
let batch_result = run_phase_1_with_tracing(&mut handle.sim, &trace_ids);
```

- [ ] **Step 3: Verify sampling works**

Set sample rate to 1% via config panel. Run simulation. Check SigNoz for creature_tick spans — should see ~1% of population traced per tick.

Flag a specific creature via inspector. Verify that creature always appears in traces regardless of sample rate.

- [ ] **Step 4: Commit**

```bash
git commit -m "feat: wire sampling rate and flagged creatures into batch traced execution"
```
