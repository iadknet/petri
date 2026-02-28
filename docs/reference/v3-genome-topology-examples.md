# V3 Genome Topology Examples

Reference patterns and speculative viability archetypes for V3 mesh genomes.
Each pattern is intentionally small and focused on one behavior.

Status: Active

---

## 1. How to Read These

- `ENTRY` means `CreatureGenome.entry_node_id`.
- Edge labels are `targets[idx]` positions, not node IDs.
- Every hop passes `output_slots: [f32; 12]` to the next node.
- Within a node evaluation, output slots pass through by default; explicit slot
  writes overwrite selected positions.
- VM nodes may emit action or route; Graph nodes route only.

---

## 2. Pattern A: Single-Node VM (Baseline)

Use this as the minimal control case and founder-friendly baseline.

```text
+-------------------------------+
| Node 0 (VM)                   |
| ENTRY                         |
| terminal                      |
+-------------------------------+
```

Topology:

| Node | Backend | `targets` | Role |
|---|---|---|---|
| 0 | VM | `[]` | Emits final `WorldAction` |

Tick trace:
1. Evaluate Node 0 with `upstream_slots = [0.0; 12]`.
2. If VM emits action, return it.
3. If VM halts without action, runtime returns `NoOp`.

---

## 3. Pattern B: Graph Preprocess -> VM Action

Use this for "filter then decide" behavior.

```text
+----------------------------------+      targets[0]      +-------------------------+
| Node 0 (Graph)                   | --------------------> | Node 1 (VM)             |
| ENTRY                            |                       | action node             |
| feature extractor                |                       | terminal                |
+----------------------------------+                       +-------------------------+
```

Topology:

| Node | Backend | `targets` | Role |
|---|---|---|---|
| 0 | Graph | `[1]` | Writes feature signals into output slots |
| 1 | VM | `[]` | Reads slots + sensors, emits action |

Tick trace:
1. Node 0 reads `InputRef { ref_idx, sub_idx }` values and writes derived signals (for example, slots 0 and 1).
2. Routing picks `targets[0]`, so Node 1 executes next.
3. Node 1 reads `UpstreamSlot(slot)`, combines with direct inputs, and emits `Move/Eat/Reproduce/NoOp`.

---

## 4. Pattern C: Multi-Mode Dispatcher

Use this for behavior switching (forage vs reproduce vs explore).

```text
                          +-------------------------------+
                          | Node 0 (Graph)                |
                          | ENTRY                         |
                          | router                        |
                          +-------------------------------+
                           /            |             \
               targets[0] /    targets[1]      targets[2] \
                         v              v                 v
        +---------------------+ +---------------------+ +---------------------+
        | Node 1 (VM)         | | Node 2 (VM)         | | Node 3 (VM)         |
        | forage terminal     | | reproduce terminal  | | explore terminal    |
        +---------------------+ +---------------------+ +---------------------+
```

Topology:

| Node | Backend | `targets` | Role |
|---|---|---|---|
| 0 | Graph | `[1, 2, 3]` | Chooses behavior module |
| 1 | VM | `[]` | Forage action logic |
| 2 | VM | `[]` | Reproduction action logic |
| 3 | VM | `[]` | Exploration action logic |

Tick trace (example route):
1. Node 0 computes `route_target_idx = 1.8`.
2. Runtime maps to `target_idx = floor(1.8) = 1`, then wraps: `1 % 3 = 1`.
3. Node 2 executes and emits `Reproduce`.

---

## 5. Pattern D: Two-Stage Perception (Graph -> Graph -> VM)

Use this when you want layered signal processing before action selection.

```text
+----------------------------------+      targets[0]      +----------------------------------+
| Node 0 (Graph)                   | --------------------> | Node 1 (Graph)                   |
| ENTRY                            |                       | high-level router                |
| low-level features               |                       +----------------------------------+
+----------------------------------+                                   /            \
                                                               targets[0]    targets[1]
                                                                     v            v
                                                            +----------------+ +----------------+
                                                            | Node 2 (VM)    | | Node 3 (VM)    |
                                                            | feed terminal  | | flee terminal  |
                                                            +----------------+ +----------------+
```

Topology:

| Node | Backend | `targets` | Role |
|---|---|---|---|
| 0 | Graph | `[1]` | Encodes raw world signals |
| 1 | Graph | `[2, 3]` | Integrates upstream features, selects mode |
| 2 | VM | `[]` | Feed behavior |
| 3 | VM | `[]` | Flee behavior |

Tick trace:
1. Node 0 produces coarse features (for example, food and danger slots).
2. Node 1 consumes those slots and chooses between feed/flee modules.
3. Selected VM node emits final action.

---

## 6. Pattern E: Junk-DNA-Tolerant Routing

Use this to reason about malformed-but-valid genomes under soft defaults.

```text
+-------------------------------+  targets[0] -> missing node_id 99  +-----------------------+
| Node 0 (VM)                   | -----------------------------------> | missing node          |
| ENTRY                         |                                      +-----------------------+
| router                        |
+-------------------------------+
           |
           | targets[1] -> node_id 2
           v
  +-------------------------------+
  | Node 2 (VM)                   |
  | terminal                      |
  +-------------------------------+
```

Topology:

| Node | Backend | `targets` | Role |
|---|---|---|---|
| 0 | VM | `[99, 2]` | Can route to broken or valid branch |
| 2 | VM | `[]` | Valid terminal branch |

Tick trace:
1. If runtime picks `targets[0]`, routed node lookup fails and returns `NoOp`.
2. If runtime picks `targets[1]`, execution continues to Node 2 normally.

This pattern is expected under mutation-heavy evolution and should never panic.

---

## 7. Speculative Viable Creature Examples

These are design-time hypotheses, not validated outcomes. We have not yet
implemented and measured mesh ecology viability.

### 7.1 Opportunistic Forager (Pattern B)

```text
Node 0 (Graph ENTRY) --> Node 1 (VM terminal)
```

Candidate behavior:
1. Graph node writes slot 0 = local food signal, slot 1 = best direction hint.
2. VM node runs priority logic:
3. if `food_here > eat_threshold` then emit `Eat`
4. else if `energy > reproduce_gate` then emit `Reproduce(direction=slot1, energy=bid)`
5. else emit `Move(direction=slot1)`

Why this could be viable:
- Always has an action path (no dead-end terminal logic).
- Uses direct food conversion when available.
- Converts surplus energy into offspring instead of hoarding.

### 7.2 Energy-Gated Mode Switcher (Pattern C)

```text
                    /-> Node 1 VM (forage)
Node 0 Graph ENTRY -+-> Node 2 VM (reproduce)
                    \-> Node 3 VM (explore)
```

Candidate behavior:
1. Router computes mode from food + energy + local occupancy.
2. Forage VM favors `Eat`/`Move toward food`.
3. Reproduce VM only emits `Reproduce` when `energy_current` exceeds high gate.
4. Explore VM emits movement for dispersal when density is high.

Why this could be viable:
- Separates short-term survival from long-term expansion.
- Avoids frequent low-energy reproduction failures.
- Gives explicit anti-stagnation behavior via explore mode.

### 7.3 Two-Stage Perception with Escape Branch (Pattern D)

```text
Node 0 Graph (feature extraction) -> Node 1 Graph (mode router)
                                      |-> Node 2 VM (feed)
                                      \-> Node 3 VM (flee)
```

Candidate behavior:
1. First graph compresses raw inputs into food/danger/space features.
2. Second graph chooses feed vs flee mode from those features.
3. Feed VM performs local exploitation; flee VM biases movement away from blocked/occupied neighbors.

Why this could be viable:
- Reduces VM complexity by precomputing decision features.
- Supports fast context switching under local threat.
- Maintains explicit terminal action nodes for reliable actuation.

### 7.4 Viability Checklist for New Archetypes

Before treating a topology as "likely viable", verify it has all of:
- At least one frequently reachable terminal VM action path.
- A positive net-energy path (`Eat` opportunity plus bounded action costs).
- A conditional reproduction path that checks energy before emitting.
- Recovery behavior when food is absent (movement or exploration).
- Bounded routing behavior under mutation noise (`max_mesh_hops` + safe defaults).
