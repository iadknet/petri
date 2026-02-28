# V3 Startup and Founder Seeding Spec

Reference specification for canonical startup seeding behavior, founder baseline
policy, and startup viability posture in V3.

Status: Active

Related references:
- `v3-world-grid-spec.md`
- `v3-runtime-config-spec.md`
- `v3-tick-orchestration-spec.md`
- `v3-reproduction-spec.md`
- `v3-genome-spec.md`
- `v3-server-api-protocol-spec.md`
- `v3-cli-contract-spec.md`
- `v3-phenotype-spec.md`

---

## 1. Purpose and Scope

This document defines:
- startup ownership boundaries for initial world and creature seeding;
- deterministic startup seeding contract for world food and founder placement;
- canonical baseline founder-profile policy for v3alpha1;
- startup viability rejection semantics;
- startup invariants for first state/frame outputs.

This document does not define:
- tick-time action arbitration semantics (owned by `v3-tick-orchestration-spec.md`);
- runtime cognition execution internals (owned by `v3-mesh-execution-spec.md`);
- transport/wire schemas for startup endpoints (owned by
  `v3-server-api-protocol-spec.md`).

---

## 2. Ownership Boundaries

Canonical ownership split:

- Startup seeding behavior and founder baseline policy: this file.
- World defaults, edge behavior, and spatial validity primitives:
  `v3-world-grid-spec.md`.
- Runtime/energy/mutation key defaults and normalization rules:
  `v3-runtime-config-spec.md`.
- Startup endpoint request/response payload schemas:
  `v3-server-api-protocol-spec.md`.

This split keeps startup policy canonical without duplicating world/runtime
configuration semantics.

---

## 3. Startup Request Contract (Conceptual)

Startup uses one required seed with optional structured overrides:

```text
startup(seed, overrides?)
```

Conceptual override domains:
- `population` (initial counts and limits; canonical defaults in
  `v3-runtime-config-spec.md` Section 5),
- `world` (`width`, `height`, `edge_mode`, food parameters),
- `energy` (lifecycle and action-cost controls),
- `runtime` (mesh/vm/graph/mutation controls).

v3alpha1 policy:
- No `founder_profile` request field is part of the canonical startup contract.
- Unknown/unsupported startup fields are request validation failures.

Canonical wire-level field schemas for startup are owned by
`v3-server-api-protocol-spec.md`.

---

## 4. Deterministic Startup Seeding Contract

For identical startup inputs and seed:
- seeded food distribution is deterministic,
- founder spawn attempt ordering is deterministic,
- resulting initial world state and initial creature set are deterministic.

Determinism is required for harnesses and reproducibility-sensitive assertions.
Production runtime determinism remains non-required at product level (canonical
policy in `AGENTS.md`).

Seeding flow contract:
1. Build effective startup config from defaults plus accepted overrides.
2. Seed world food using world food initialization semantics from
   `v3-world-grid-spec.md`.
   - v3alpha1 seeds exact coverage over non-barrier cells:
     `round(initial_coverage * eligible_cells)` unique cells.
   - Seeded density uses normalized `f32` food values in canonical `[0.0, 1.0]`
     scale (clamped by `world.food.max_density`).
3. Seed founders using the canonical founder baseline from Section 5.
4. Commit simulation state at tick `0` in `idle` state.

---

## 5. Founder Baseline Policy (v3alpha1)

v3alpha1 uses one canonical internal founder baseline for startup seeding.

Rules:
- All startup-seeded founders derive from the same baseline founder genome
  profile.
- Startup-seeded founders begin at `generation = 0`.
- Startup does not introduce per-founder profile selection via API.
- Startup phenotype baseline is deterministic and uniform for seeded founders.
  Canonical founder phenotype baseline values (RGB, channel weights, polarity)
  are defined in `v3-phenotype-spec.md` Section 3.
- Diversity is introduced after startup via mutation/reproduction policy, not via
  startup profile randomization.

Founder genome structure, parseability, and runtime interpretation remain owned
by `v3-genome-spec.md`, `v3-mutation-spec.md`, and
`v3-mesh-execution-spec.md`.

### 5.1 Canonical v3alpha1 Founder Genome

The founder genome is a 2-node mesh with both backend types.

**Structural layout:**

```text
CreatureGenome {
  entry_node_id: 0,
  nodes: [Node0, Node1],
}
```

**Node 0 — Graph backend (sensor aggregator)**

```text
NodeGenome {
  node_id: 0,
  input_refs: [
    0: World(FoodHere),
    1: DynamicIntrospection(EnergyCurrent),
    2: World(NeighborCellFood(0)),     // N
    3: World(NeighborCellFood(2)),     // E
    4: World(NeighborCellFood(4)),     // S
    5: World(NeighborCellFood(6)),     // W
    6: World(NeighborCellOccupied(0)), // N
    7: World(NeighborCellOccupied(2)), // E
    8: World(NeighborCellOccupied(4)), // S
    9: World(NeighborCellOccupied(6)), // W
  ],
  backend_def: Graph(GraphBackendDef {
    internal_nodes: [
      // idx 0: food_here signal
      { kind: InputRef(0), inputs: [] },
      // idx 1: energy_current signal
      { kind: InputRef(1), inputs: [] },
      // idx 2: reproduce gate (energy >= 24.0)
      { kind: Threshold(24.0), inputs: [{ source_idx: 1, weight: 1.0 }] },
      // idx 3-6: neighbor food N/E/S/W
      { kind: InputRef(2), inputs: [] },
      { kind: InputRef(3), inputs: [] },
      { kind: InputRef(4), inputs: [] },
      { kind: InputRef(5), inputs: [] },
      // idx 7-10: neighbor occupied N/E/S/W
      { kind: InputRef(6), inputs: [] },
      { kind: InputRef(7), inputs: [] },
      { kind: InputRef(8), inputs: [] },
      { kind: InputRef(9), inputs: [] },
      // idx 11-16: output writers
      { kind: CustomOutput(0), inputs: [{ source_idx: 0, weight: 1.0 }] },
        // slot 0 = food_here
      { kind: CustomOutput(1), inputs: [{ source_idx: 2, weight: 1.0 }] },
        // slot 1 = can_reproduce (0 or 1)
      { kind: CustomOutput(2), inputs: [{ source_idx: 3, weight: 1.0 }] },
        // slot 2 = food_N
      { kind: CustomOutput(3), inputs: [{ source_idx: 4, weight: 1.0 }] },
        // slot 3 = food_E
      { kind: CustomOutput(4), inputs: [{ source_idx: 5, weight: 1.0 }] },
        // slot 4 = food_S
      { kind: CustomOutput(5), inputs: [{ source_idx: 6, weight: 1.0 }] },
        // slot 5 = food_W
      // idx 17: route to node 1
      { kind: RouterOutput, inputs: [] },
    ],
  }),
  targets: [1],
}
```

**Node 1 — VM backend (decision + action emitter)**

```text
NodeGenome {
  node_id: 1,
  input_refs: [
    0: UpstreamSlot(0),  // food_here
    1: UpstreamSlot(1),  // can_reproduce
    2: UpstreamSlot(2),  // food_N
    3: UpstreamSlot(3),  // food_E
    4: UpstreamSlot(4),  // food_S
    5: UpstreamSlot(5),  // food_W
  ],
  backend_def: Vm(VmBackendDef {
    registers: 8,
    constants: [0.5, 1.0, 2.0, 3.0, 20.0],
    program: [
      // Read inputs into registers
      ReadInput(r0, 0, 0),    // r0 = food_here
      ReadInput(r1, 1, 0),    // r1 = can_reproduce
      ReadInput(r2, 2, 0),    // r2 = food_N
      ReadInput(r3, 3, 0),    // r3 = food_E
      ReadInput(r4, 4, 0),    // r4 = food_S
      ReadInput(r5, 5, 0),    // r5 = food_W

      // Priority 1: Reproduce if energy sufficient
      CmpGt(r6, r1, r7),     // r6 = can_reproduce?
      JumpIfZero(r6, +6),     // skip reproduce block

      // Set up reproduce action with direction + energy
      WriteWorldActionMeta(0, r7),  // direction placeholder (N)
      LoadConst(r6, 4),       // const[4] = 20.0 offspring energy
      WriteWorldActionMeta(1, r6),  // offspring energy
      PushAction(3),          // push Reproduce onto action queue

      // Priority 2: Eat if food here
      Sub(r7, r7, r7),        // r7 = 0.0
      CmpGt(r6, r0, r7),     // r6 = (food_here > 0)?
      JumpIfZero(r6, +1),     // skip eat if no food
      PushAction(1),          // push Eat onto action queue

      // Priority 3: Move toward highest food direction
      CmpGt(r6, r2, r4),     // r6 = food_N > food_S?
      Sub(r0, r0, r0),        // r0 = 0 (N)
      JumpIfZero(r6, +2),     // if food_S >= food_N, jump to S
      Jump(+2),               // food_N wins, keep r0=0
      LoadConst(r0, 3),       // const[3] = 3.0 ~ S direction idx
      WriteWorldActionMeta(0, r0),  // set direction
      PushAction(2),          // push Move onto action queue

      // Execute all queued actions
      ExecuteActionQueue,
    ],
  }),
  targets: [],
}
```

**Behavioral intent:**
- Founders build a multi-action queue each tick using `PushAction` + `ExecuteActionQueue`.
- Founders reproduce when energy is sufficient (highest priority).
- Founders eat when standing on food (second priority).
- Founders move toward the cardinal direction with highest visible food.
- Multiple actions can execute per tick (up to `max_actions_per_turn`).

This gives natural selection immediate material to work with: creatures that
find food and reproduce efficiently will out-compete those that do not.

**Determinism note:** The founder genome structure above is canonical for
v3alpha1 seeding. The exact bytewise encoding is an implementation detail, but
the structural layout (node count, node IDs, input_refs, graph internal
topology, VM instruction sequence, and constants) must be identical across
implementations for deterministic seeding with the same seed.

---

## 6. Initial Population Behavior

Population request values are targets subject to world constraints.

Canonical behavior:
- Startup attempts to place founders into valid spawnable cells only.
- Placement uses world validity primitives from `v3-world-grid-spec.md`
  (occupancy, barriers, and edge handling).
- Successful founder count may be less than requested when spatial constraints
  prevent full placement.

Any shortfall between requested and placed founders must be reflected truthfully
in initial status/frame outputs.

---

## 7. Viability and Rejection Semantics

v3alpha1 startup policy rejects invalid or non-viable startup inputs.

Rules:
- No auto-normalization path is canonical for request-level invalid/non-viable
  startup submissions.
- Invalid shape/range/state for startup request fields is rejected.
- Startup configurations that are accepted syntactically but fail viability
  gates are rejected.
- Rejections use canonical `422 validation_rejected` error semantics defined in
  `v3-server-api-protocol-spec.md`.

A viability rejection means startup does not mutate simulation state.

---

## 8. Startup Invariants for First State/Frame

On successful startup:
- simulation state is `idle` at `tick = 0`;
- initial world/frame represents applied startup state, not placeholders;
- initial sparse frame includes seeded creatures and seeded food consistent with
  accepted startup config and seed;
- status/health counters are initialized consistently with zero-tick semantics.

Lifecycle transitions after startup are owned by
`v3-server-api-protocol-spec.md`.

---

## 9. Cross-Spec Ownership Map

- World defaults/edge behavior/validity primitives: `v3-world-grid-spec.md`
- Runtime/energy/mutation defaults: `v3-runtime-config-spec.md`
- Tick queue/arbitration semantics: `v3-tick-orchestration-spec.md`
- Startup/start/pause/step transport contracts: `v3-server-api-protocol-spec.md`
- CLI startup/run outputs: `v3-cli-contract-spec.md`

This file remains canonical for startup seeding policy and founder baseline
posture.

---

## 10. Policy References

- Project-level determinism scope is canonical in `AGENTS.md`
  (`Determinism Scope (Canonical)`).
- Runtime truthfulness invariant is canonical in `docs/strategy/architecture.md`
  (`Runtime Truthfulness Invariant`).
