# Petri V3 Graph Operator Reference

> **Reference specification for v3 graph node operators. Adapted from v2 design.
> This is a reference document, not an implementation plan.**

---

## Graph Operators

The v3 graph backend supports 11 fixed-function operators. Each operator is
dispatched per-node during graph evaluation.

| # | Operator | Parameters |
|---|----------|------------|
| 1 | **Passthrough** | _(none)_ |
| 2 | **WeightedSum** | _(uses coefficients vec)_ |
| 3 | **Threshold** | `threshold: f32` |
| 4 | **Clamp01** | _(none)_ |
| 5 | **DecayIntegrator** | `state_slot: u8`, `alpha: f32` |
| 6 | **Momentum** | `state_slot: u8`, `beta: f32` |
| 7 | **Oscillator** | `phase_slot: u8`, `frequency: f32`, `amplitude: f32`, `bias: f32` |
| 8 | **SumPool** | _(none)_ |
| 9 | **MeanPool** | _(none)_ |
| 10 | **MaxPool** | _(none)_ |
| 11 | **AdaptiveGain** | `gain_slot: u8`, `learning_rate: f32`, `min_gain: f32`, `max_gain: f32` |

---

## Parameter Normalization and Bounds

All operator parameters are clamped at graph construction and mutation time.

| Parameter | Valid Range | Notes |
|-----------|-------------|-------|
| `alpha` | `[0.0, 1.0]` | DecayIntegrator blend factor |
| `beta` | `[0.0, 1.0]` | Momentum blend factor |
| `frequency` | `[0.0, 8.0]` | Cycles per tick |
| `amplitude` | `[0.0, 10.0]` | Oscillator output scale |
| `learning_rate` | `[0.0, 0.1]` | AdaptiveGain adaptation speed |
| `state_slot` | `< state_slot_count` | Must index a valid local state slot |

---

## Graph Local State Contract

- Each graph node owns `state_slot_count` persistent `f32` slots (`0..=8`).
- State persists across ticks for living creatures.
- State resets to zero on birth unless initialized via `local_state_init`.
- State updates are deterministic and operator-local.

---

## Operator Semantics

Exact evaluation formulas for stateful and pooling operators. All inputs are
`f32` values produced by upstream nodes or sensor references.

### DecayIntegrator

```
s = (1 - alpha) * s + alpha * input0
output = s
```

Where `s` is the persistent value in `state_slot`.

### Momentum

```
delta = input0 - input1
s = beta * s + (1 - beta) * delta
output = s
```

Where `s` is the persistent value in `state_slot`.

### Oscillator

```
phase = fract(phase + frequency * dt)    // dt = 1
output = bias + amplitude * sin(2 * pi * phase)
```

Where `phase` is the persistent value in `phase_slot`.

### SumPool

```
output = sum(all inputs)
```

### MeanPool

```
output = mean(all inputs)    // 0.0 for empty input set
```

### MaxPool

```
output = max(all inputs)     // 0.0 for empty input set
```

### AdaptiveGain

```
gain = clamp(gain + learning_rate * input1, min_gain, max_gain)
output = gain * input0
```

Where `gain` is the persistent value in `gain_slot`.

---

## Graph Cost Model

Each graph node incurs an energy cost per tick, computed as:

```
effective_graph_cost = graph_base_tariff * graph_operator_cost_multiplier(operator)
```

### Operator Cost Multipliers

| Operator | Multiplier |
|----------|------------|
| Passthrough | 0.7 |
| WeightedSum | 1.0 |
| Threshold | 0.9 |
| Clamp01 | 0.8 |
| DecayIntegrator | 1.2 |
| Momentum | 1.3 |
| Oscillator | 1.4 |
| SumPool | 1.0 |
| MeanPool | 1.1 |
| MaxPool | 1.2 |
| AdaptiveGain | 1.3 |

> **Note:** v3 uses `u32` energy (not `f32`). The `graph_base_tariff` will be
> scaled to integer energy units. Fractional multiplier products are rounded to
> the nearest integer cost.

---

## Graph Backend Rules

1. **Fixed-function** -- no graph mini-language or bytecode interpreter.
2. **Deterministic** -- identical inputs and state always produce identical outputs.
3. **Bounded-time** -- evaluation is O(nodes + edges) per dispatch; no loops or recursion.
4. **Input references** -- graph inputs may reference `Sensor*` and `Neighbor*` input types.
5. **Output emission** -- the graph emits outputs from `output_definitions` after full evaluation.

---

## GraphBackendDef Structure

The `GraphBackendDef` describes a single graph node within a creature's
controller graph. In v3, `runtime/graph.rs` owns graph execution.

| Field | Type | Description |
|-------|------|-------------|
| `operator` | `GraphOperator` | One of the 11 operators listed above |
| `inputs` | `Vec<InputReference>` | References to sensor values or other node outputs |
| `coefficients` | `Vec<f32>` | Weights for weighted operators; length must equal `inputs.len()` |
| `bias` | `f32` | Additive bias applied after operator evaluation |
| `state_slot_count` | `u8` | Number of persistent local state slots (`0..=8`) |
| `operator_params` | `GraphOperatorParams` | Operator-specific parameters (threshold, alpha, frequency, etc.) |
