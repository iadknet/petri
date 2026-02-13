# Heritable Phenotype Color with Gradual Drift

## Problem

The current phenotype color system derives color by hashing the entire controller graph structure (FNV-1a over all nodes, edges, and weights). This means even a single weight mutation completely changes the hash and produces an unrelated color. Parent and child with 99% identical graphs can have completely different colors, making it impossible to visually identify lineage clusters.

## Design

Replace graph-hash-based color derivation with an inherited color model where phenotype color is a heritable trait that drifts gradually across generations.

### Color Model (HSV, stored as heritable traits)

- **Hue** (0.0-360.0): The "family color." Inherited from parent. Perturbed by +/-2 degrees **only when** the child's controller is mutated during reproduction. If no mutation occurs, hue is inherited exactly. Wraps around the color wheel (360 -> 0).
- **Saturation** (0.4-1.0): Individual texture within a family. Inherited from parent with a small random perturbation (+/-0.05) every generation regardless of mutation. Clamped to [0.4, 1.0].
- **Value/Brightness**: Fixed at 0.80. Not heritable, not drifted.

RGB `[u8; 3]` is computed from HSV once at birth and cached for frame emission.

### Drift Characteristics

Hue drift is a mutation-gated random walk. After N mutated generations, expected divergence is ~2 * sqrt(N/3) degrees:
- After 100 mutated generations: ~12 degree std dev (same color family)
- After 500 mutated generations: ~26 degree std dev (noticeably different)
- After 1000+ mutated generations: lineages clearly separated

Saturation drifts every generation but is clamped to a visible range, so it acts as fast-moving visual variety within a lineage cluster.

### Creature Storage

Each creature gains two new fields:
- `phenotype_hue: f32` (0.0-360.0)
- `phenotype_saturation: f32` (0.4-1.0)

The existing `phenotype_color: [u8; 3]` RGB cache remains for frame emission.

### Lifecycle

**Founder spawn:** Random hue (0-360), random saturation (0.4-1.0). Convert to RGB and cache.

**Reproduction:**
1. Child inherits parent's `phenotype_hue` and `phenotype_saturation`.
2. Saturation: always perturbed by +/-0.05, clamped to [0.4, 1.0].
3. Hue: perturbed by +/-2 degrees only if the child's controller was mutated. Wraps modulo 360.
4. Convert HSV (hue, saturation, value=0.80) to RGB and cache.

**Snapshot save/restore:** `phenotype_hue` and `phenotype_saturation` are serialized as part of creature state. RGB is recomputed on restore.

### Wire Format

No changes. `phenotype_color: [u8; 3]` in CreatureSnapshot and CreatureDetail remains the transport format. Hue and saturation are internal Rust state only.

### Removed

- `ComputationGraph::phenotype_color()` method in petri-graph
- `ComputationGraph::phenotype_fingerprint()` method in petri-graph
- `node_signature()` helper in petri-graph
- Graph-hash-based color derivation logic

The `hsv_to_rgb()` helper moves to petri-core (where the color is now computed).

### Test Plan

- Child with no mutation inherits exact parent hue, drifted saturation
- Child with mutation inherits drifted hue and drifted saturation
- Hue wraps correctly around 360 -> 0 boundary
- Saturation clamps to [0.4, 1.0] bounds
- Determinism: same seed produces same color sequence
- Founders get random hue/saturation within valid ranges
- Snapshot round-trip preserves hue/saturation/RGB
