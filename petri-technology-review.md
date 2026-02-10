# Petri — Technology Stack Review

## Summary

This document evaluates library and technology choices for the Petri project. The guiding principle: use a library when it genuinely saves effort and is well-maintained; roll our own when a library adds more complexity than it solves or doesn't fit our specific model.

---

## Rust Backend Dependencies

### Entity Storage: `slotmap` (HopSlotMap)

**Recommendation: Use `slotmap` crate, specifically `HopSlotMap`.**

Our creatures have a uniform component set — every creature has the same fields. This means a full Entity-Component-System framework (Bevy ECS, hecs, legion, specs) is architectural overkill. ECS shines when you have heterogeneous entity types with different component combinations; we don't.

HopSlotMap gives us O(1) insertion/removal, O(1) lookup by stable ID, and fast iteration that skips vacant slots. For 10,000–50,000 creatures iterated every tick, this is the right tradeoff.

Status: `slotmap` v1.1 is actively maintained with millions of downloads.

```toml
slotmap = "1.1"
```

### Spatial Indexing: Hand-rolled grid

**Recommendation: Don't use a spatial indexing library. Our grid IS the spatial index.**

Libraries like `rstar` (R*-trees) and `kiddo` (k-d trees) are designed for irregular point clouds in continuous space. Our world is a fixed discrete grid. The grid cells themselves provide O(1) lookup for "what's at this position?" For nearest-neighbor queries (nearest food, nearest creature), a simple scan of surrounding cells within sensing radius is efficient and straightforward.

Implementation: a flat `Vec<Cell>` for the grid, plus a separate `HashMap<(i32, i32), Vec<CreatureId>>` or chunk-based spatial hash for creature-to-creature proximity queries if the sensing radius is large. The `fxhash` crate can speed up the hash map.

```toml
rustc-hash = "2.1"  # fxhash for fast grid coordinate hashing
```

### Computation Graphs: `petgraph`

**Recommendation: Use `petgraph` for graph storage and algorithms.**

Each creature's brain is a directed graph with 5–200 nodes. We need node/edge addition and removal, topological sort (for evaluation order), cycle detection, and serialization. `petgraph` provides all of this with good performance and built-in serde support.

At 10,000+ creatures with small graphs each, the per-graph overhead is negligible. We'll use `petgraph::Graph` (adjacency list) rather than `petgraph::GraphMap` for better cache locality.

Status: `petgraph` v0.8.3, actively maintained with frequent 2025 releases.

```toml
petgraph = { version = "0.8", features = ["serde-1"] }
```

### Serialization: `postcard` (save/load) + `rmp-serde` (WebSocket)

**Recommendation: Two formats for two purposes.**

For **save/load** (Rust-to-Rust), use `postcard`. It's ~30% smaller than bincode, fast, and actively maintained. Notably, `bincode` has a RUSTSEC-2025 advisory marking it as unmaintained — that rules it out for new projects.

For **WebSocket frames** (Rust-to-JavaScript), use `rmp-serde` (MessagePack). The frontend needs to deserialize these frames, and MessagePack has mature JavaScript libraries (`msgpack-lite`, `@msgpack/msgpack`). Postcard's format is Rust-specific and has no JS decoder.

```toml
postcard = { version = "1.3", features = ["alloc"] }
rmp-serde = "1.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"  # for REST API responses
```

### Random Number Generation: `rand` with `SmallRng`

**Recommendation: Use `SmallRng` from the `rand` crate.**

We need fast, seedable RNG. `SmallRng` (currently Xoshiro-based) is extremely fast and passes standard statistical tests — more than sufficient for creature behavior and mutation. `ChaCha8Rng` is ~10x slower and provides cryptographic security we don't need.

Store RNG runtime state in save files so a loaded simulation can continue naturally.

For parallelism (rayon), avoid shared RNG state. Keep randomness on the world/creature data model (for example, per-creature RNG + world RNG) so evaluation work can remain parallel without contention.

```toml
rand = "0.8"
```

### Parallelism: `rayon`

**Recommendation: Use `rayon` for data-parallel creature evaluation.**

Phase 2 (graph evaluation) is embarrassingly parallel — each creature's evaluation is independent given read-only world state. Rayon's work-stealing scheduler handles load balancing automatically. The API change is minimal: `.iter()` becomes `.par_iter()`.

Gotchas to plan for: RNG state should be owned by world/creature state (not shared mutable global state), and the spatial grid must be read-only during evaluation (rebuild it in a separate sequential phase). Profile to confirm parallelism actually helps at our creature counts — for very small graphs (<10 nodes), the scheduling overhead could dominate.

```toml
rayon = "1.11"
```

### Web Server: `axum` + `tokio`

**Recommendation: Use `axum` for both REST and WebSocket serving.**

Axum is built by the Tokio team, has first-class WebSocket support via extractors, and handles REST and WebSocket on the same server cleanly. It's the current community standard for new Rust web services.

The alternative would be `actix-web`, which has slightly higher raw throughput in benchmarks but uses its own runtime and has a steeper learning curve. For our use case (single-user local simulation with one active UI client), the performance difference is irrelevant.

`warp` is in maintenance mode — skip it.

For the current single-user model, keep streaming simple: one active WebSocket connection receives simulation frames directly from the server loop.

```toml
axum = { version = "0.8", features = ["ws"] }
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.6", features = ["cors"] }  # CORS for dev
```

---

## Neuroevolution: Build Our Own

**Recommendation: Roll our own graph evolution system on top of `petgraph`.**

This is the most important build-vs-buy decision in the project, and the answer is clearly build.
It is also an explicit **neurosymbolic** design choice: one graph may combine neural-style approximation nodes with discrete logic/arithmetic control nodes.

### Why existing libraries don't fit

Existing Rust NEAT implementations (`rustneat`, `OxiNEAT`, `neuralneat`, etc.) are all designed for standard neural networks — homogeneous node types with weighted-sum-plus-activation-function semantics. Our computation model is a hybrid graph with heterogeneous node types: neural nodes, arithmetic nodes, logic nodes, memory nodes, gate nodes. No existing library supports this mix.

The same applies to CGP (Cartesian Genetic Programming) libraries — they assume fixed mathematical operator palettes and don't support neural-style weighted connections.

General evolutionary algorithm frameworks (`genevo`, `RsGenetic`) handle population management and selection, but know nothing about graph topology. They'd save maybe 100 lines of selection code while constraining our architecture.

### What we build

Our `petri-genome` crate implements:

**Mutation operators** (~400–600 lines): weight perturbation, add/remove node, add/remove edge, change node type, perturb node parameters, duplicate subgraph. These operate directly on `petgraph::Graph` instances. Mutation policies are split by type: frequent continuous parameter perturbation, lower-rate discrete logic/type mutation, and lower-rate topology edits.

**Innovation numbering** (~200 lines): Borrowed from the NEAT algorithm. A global counter assigns unique IDs to structural mutations (new nodes, new edges). This allows crossover to align two graphs with different topologies by matching innovation IDs rather than node positions. Reference: Stanley & Miikkulainen, "Evolving Neural Networks through Augmenting Topologies" (2002).

**Crossover** (~300–500 lines): For sexual reproduction. Align parent graphs by innovation ID, inherit matching genes from either parent, inherit excess/disjoint genes from the fitter parent. Handle mixed node types at conflict points.

**Distance metric** (~100 lines): Measure genome similarity for kin recognition and speciation. Count excess genes, disjoint genes, and average weight difference between matching genes.

Total estimated implementation: 1,000–1,400 lines of focused, domain-specific code. This is less effort than adapting an existing NEAT library to support hybrid semantics, and we control every line.

### Hybrid Graph Risk Controls

To make hybrid neural+logic evolution practical, include these guardrails in architecture and mutation policy:

1. **Explicit interface nodes:** include neural->logic and logic->neural converters (for example `Threshold` and `Select`/mux-like routing) so mixed subgraphs communicate through clear semantics.
2. **Topology bias as a soft prior:** initialize and mutate with preference for `sensor -> neural/perception -> logic/decision -> output`, but do not hard-lock this shape.
3. **Separated mutation timescales:** keep weight/parameter perturbation rates higher than logic-node type and structural mutation rates.
4. **Ablation as validation, not proof of superiority:** run neural-only, logic-only, and hybrid palettes under fixed configs to confirm runtime stability and instrumentation quality.

### References to study

- The NEAT paper (Stanley & Miikkulainen, 2002) for innovation numbering and crossover alignment
- "Learning to Fly" tutorial series (pwy.io) for practical Rust neuroevolution architecture
- Self-adaptive mutation rates paper (PLOS One, 2024) for preventing topology bloat

---

## Frontend Stack

### Framework: React + TypeScript + Vite

**Recommendation: React for UI controls, Canvas/WebGL for rendering.**

The framework choice matters less than you'd think here, because the performance-critical rendering (world grid, evolutionary tree) runs on its own Canvas/WebGL animation loop completely outside the framework's reactivity system. React handles the controls, panels, and layout; Canvas handles the pixels.

React wins over Svelte or SolidJS primarily on ecosystem: ReactFlow (for graph visualization) is React-native, the library ecosystem is larger, and React + Canvas is a well-documented pattern.

Vite as the build tool for fast dev experience.

### World Rendering: ImageData Buffer

**Recommendation: Render the grid by writing directly to an ImageData buffer, then `putImageData()` to Canvas.**

For a 1000×1000 grid, this is a 4MB RGBA buffer updated per frame. The Rust backend sends creature positions, food levels, and barrier locations as binary data over WebSocket. The frontend writes pixel colors into the buffer and blits to canvas in one operation.

This is faster than any higher-level approach (PixiJS, WebGL) for our use case because we're doing 1:1 pixel mapping with no transforms, no sprites, no blending. Each cell is one pixel (or a small square at zoom levels).

If we later want smooth zoom, camera effects, or multi-pixel creature rendering, PixiJS (WebGL-based 2D renderer) is a clean upgrade path that wouldn't require rearchitecting.

### Creature Brain Inspector: ReactFlow

**Recommendation: Use ReactFlow for the computation graph visualization.**

When inspecting a creature, we render its 5–200 node computation graph as an interactive node-and-edge diagram. ReactFlow handles this well: it's React-native, supports custom node rendering (we can color-code by type and show activation values), and handles zoom/pan/interaction out of the box.

For graphs up to 200 nodes, performance is not a concern with any library. ReactFlow wins on developer experience and React integration.

### Evolutionary Tree: d3-hierarchy + Custom Canvas

**Recommendation: Use `d3-hierarchy` for layout computation, custom Canvas renderer for drawing.**

The evolutionary tree could have tens of thousands of nodes. SVG-based rendering breaks down at this scale. The approach: d3-hierarchy computes the tree layout (node positions, parent-child links) offline, then a custom Canvas renderer draws only the nodes visible at the current zoom level. This is a standard pattern for large hierarchical visualizations.

Implement level-of-detail filtering: at high zoom, show all nodes. As the user zooms out, progressively hide nodes with few descendants, showing only major lineage branches.

### Live Charts: uPlot

**Recommendation: Use uPlot for population, energy, and diversity time series.**

uPlot is purpose-built for real-time time series with minimal overhead. Benchmarks show 10% CPU and 12MB RAM for 3,600 points at 60fps, compared to 40% CPU and 77MB for Chart.js. It handles incremental data appending (new point per frame) without re-rendering the full history.

It's framework-agnostic, so it works alongside React without conflict.

### Frontend Dependencies Summary

```json
{
  "dependencies": {
    "react": "^18",
    "react-dom": "^18",
    "@msgpack/msgpack": "^3",
    "reactflow": "^11",
    "d3-hierarchy": "^3",
    "uplot": "^1.6"
  },
  "devDependencies": {
    "typescript": "^5",
    "vite": "^6",
    "@types/react": "^18"
  }
}
```

---

## Deployment Considerations (Native + Containers)

The deployment target is a Rust backend with a browser frontend. No in-browser simulation runtime is required.

**Primary runtime:** Native Rust process (`cargo run` for local development, packaged `--release` binary for production).

**Container runtime:** Docker/OCI image for reproducible deployment. Persist save/load data to a mounted volume, and expose both HTTP and WebSocket traffic on the same service port.

**Architecture implication:** Keep `petri-core`, `petri-graph`, and `petri-genome` as pure computation crates without server/runtime dependencies (`tokio`, `axum`). This preserves clean layering, keeps CLI/headless workflows straightforward, and simplifies testing.

---

## Architecture Impact on Design Documents

Based on this review, the following updates should be made to the architecture document:

1. **Replace the generic "binary format" references** with specific choices: `postcard` for save/load, `rmp-serde` (MessagePack) for WebSocket frames.

2. **Add `fxhash`/`rustc-hash`** to the spatial indexing section, noting we use a hand-rolled grid hash rather than a library.

3. **Add a core/runtime boundary note** to the crate structure: `petri-core`, `petri-graph`, and `petri-genome` should not depend on server/runtime frameworks (`tokio`, `axum`). Keep them reusable from both server and CLI.

4. **Clarify the innovation numbering system** in the genome crate description — this is borrowed from NEAT and is the key mechanism enabling crossover between topologically different graphs.

---

## Full Rust Dependency Summary

```toml
[workspace.dependencies]
# Core simulation
slotmap = "1.1"
petgraph = { version = "0.8", features = ["serde-1"] }
rand = "0.8"
rustc-hash = "2.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
postcard = { version = "1.3", features = ["alloc"] }
rmp-serde = "1.3"

# Server (petri-server only)
axum = { version = "0.8", features = ["ws"] }
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.6", features = ["cors"] }
rayon = "1.11"

```

Total direct dependencies for the core simulation: 6 crates. Total for the server: 10 crates. This is lean and maintainable.
