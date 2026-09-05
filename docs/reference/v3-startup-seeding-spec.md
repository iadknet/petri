# V3 Startup and Founder Seeding Spec

Reference specification for canonical startup seeding behavior, founder baseline
policy, and startup viability posture in V3.

Status: Active

Related references:
- `v3-creature-identity-spec.md`
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
- `world` (`width`, `height`, `edge_mode`, `food.shared`, `food.types`,
  `food.fertility`),
- `energy` (lifecycle and action-cost controls),
- `nutrition` (startup-only reproductive reserve capacity and reproduction cost),
- `runtime` (mesh/vm/graph/mutation controls),
- `startup` (startup-only controls such as early-run ramps).

v3alpha1 policy:
- No `founder_profile` request field is part of the canonical startup contract.
- Unknown/unsupported startup fields are request validation failures.

Canonical wire-level field schemas for startup are owned by
`v3-server-api-protocol-spec.md`.

The normalized production food catalog contains two complementary roles:
`Maintenance Food` (coverage `0.27`, metabolic yield `10.0`, reserve yield `0.0`)
and `Reproductive Food` (coverage `0.27`, metabolic yield `0.0`, reserve yield
`1.0`). Reserve capacity is `8.0` and a successful reproduction costs `4.0`.

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
     scale (clamped by `world.food.shared.max_density`).
3. Seed founders using the canonical founder baseline from Section 5.
4. Commit simulation state at tick `0` in `idle` state.

---

## 5. Founder Baseline Policy (v3alpha1)

v3alpha1 uses one canonical internal founder baseline for startup seeding.

Rules:
- All startup-seeded founders derive from the same baseline founder genome
  profile.
- Startup-seeded founders begin at `generation = 0`.
- Startup-seeded founders receive deterministic identity state per
  `v3-creature-identity-spec.md`:
  - `lineage_id` is assigned from final founder placement order
  - `kin_tag` is derived deterministically from startup seed and `lineage_id`
- Startup does not introduce per-founder profile selection via API.
- Startup phenotype baseline is deterministic and uniform for seeded founders.
  Canonical founder phenotype baseline values (RGB, channel weights, polarity)
  are defined in `v3-phenotype-spec.md` Section 3.
- Diversity is introduced after startup via mutation/reproduction policy, not via
  startup profile randomization.

Founder genome structure, parseability, and runtime interpretation remain owned
by `v3-genome-spec.md`, `v3-mutation-spec.md`, and
`v3-mesh-execution-spec.md`.
Founder identity state semantics remain owned by
`v3-creature-identity-spec.md`.

### 5.1 Canonical v3alpha1 Founder Genome

The founder genome is a 2-node mesh with both backend types. The graph backend
uses the CGP-style layered model (see `v3-graph-backend-spec.md`).
The founder reproduction gate is derived from runtime config
`energy.lifecycle.min_reproduce_age` (default `20` ticks).

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
    0: World(FoodHere { type_idx: 0 }),
    1: DynamicIntrospection(EnergyCurrent),
    2: StaticIntrospection(AgeTicks),
    3: World(NeighborFoodRing { type_idx: 0 }),      // compound: 8 directions
    4: World(NeighborOccupiedRing),  // compound: 8 directions
    5: World(FoodHere { type_idx: 1 }),
    6: DynamicIntrospection(ReproductiveReserveCurrent),
    7: World(NeighborFoodRing { type_idx: 1 }),      // compound: 8 directions
  ],
  backend_def: Graph(CgpGraphBackendDef {
    compute_nodes: [
      // idx 0: energy gate (energy > 30.0)
      { kind: Threshold(30.0),
        inputs: [{ source: InputLeaf { ref_idx: 1, sub_idx: 0 },
                   weight: 1.0 }] },
      // idx 1: age gate (age >= energy.lifecycle.min_reproduce_age), encoded
      // as max(min_reproduce_age_ticks - 0.5, -0.5) because Threshold is strict >.
      { kind: Threshold(max(min_reproduce_age_ticks - 0.5, -0.5)),
        inputs: [{ source: InputLeaf { ref_idx: 2, sub_idx: 0 },
                   weight: 1.0 }] },
      // idx 2: can_reproduce gate = energy_gate * age_gate
      { kind: Multiply,
        inputs: [
          { source: ComputeNode(0), weight: 1.0 },
          { source: ComputeNode(1), weight: 1.0 }
        ] },
      // idx 3: reserve gate = reserve > immediate predecessor(reserve_cost)
      { kind: Threshold(predecessor(reproductive_reserve_cost)),
        inputs: [{ source: InputLeaf { ref_idx: 6, sub_idx: 0 },
                   weight: 1.0 }] },
      // idx 4: can_reproduce = energy/age gate * reserve gate
      { kind: Multiply,
        inputs: [
          { source: ComputeNode(2), weight: 1.0 },
          { source: ComputeNode(3), weight: 1.0 }
        ] },
      // idx 5: any typed food on the current cell (forage signal)
      { kind: Max,
        inputs: [
          { source: InputLeaf { ref_idx: 0, sub_idx: 0 }, weight: 1.0 },
          { source: InputLeaf { ref_idx: 5, sub_idx: 0 }, weight: 1.0 }
        ] },
    ],

    output_sinks: [
      // Full fixed catalog (64 sinks): 24 CustomOutput, 8 RouterGate,
      // 16 WriteSlot, and 16 ClearSlot sinks. CustomOutput(0..12) are wired;
      // all remaining fixed sinks are empty at founder time.
      // CustomOutput(0): any typed food here (CN5) → slot 0
      { kind: CustomOutput(0),
        inputs: [{ source: ComputeNode(5),
                   weight: 1.0 }] },
      // CustomOutput(1): can_reproduce → slot 1
      { kind: CustomOutput(1),
        inputs: [{ source: ComputeNode(4), weight: 1.0 }] },
      // CustomOutput(2): food_N → slot 2
      { kind: CustomOutput(2),
        inputs: [{ source: InputLeaf { ref_idx: 3, sub_idx: 0 },
                   weight: 1.0 }] },
      // CustomOutput(3): food_E → slot 3
      { kind: CustomOutput(3),
        inputs: [{ source: InputLeaf { ref_idx: 3, sub_idx: 2 },
                   weight: 1.0 }] },
      // CustomOutput(4): food_S → slot 4
      { kind: CustomOutput(4),
        inputs: [{ source: InputLeaf { ref_idx: 3, sub_idx: 4 },
                   weight: 1.0 }] },
      // CustomOutput(5): food_W → slot 5
      { kind: CustomOutput(5),
        inputs: [{ source: InputLeaf { ref_idx: 3, sub_idx: 6 },
                   weight: 1.0 }] },
      // CustomOutput(6): reproductive food here
      { kind: CustomOutput(6),
        inputs: [{ source: InputLeaf { ref_idx: 5, sub_idx: 0 },
                   weight: 1.0 }] },
      // CustomOutput(7): live reproductive reserve
      { kind: CustomOutput(7),
        inputs: [{ source: InputLeaf { ref_idx: 6, sub_idx: 0 },
                   weight: 1.0 }] },
      // CustomOutput(8): live energy
      { kind: CustomOutput(8),
        inputs: [{ source: InputLeaf { ref_idx: 1, sub_idx: 0 },
                   weight: 1.0 }] },
      // CustomOutput(9..12): reproductive-food neighbors N/E/S/W
      { kind: CustomOutput(9),
        inputs: [{ source: InputLeaf { ref_idx: 7, sub_idx: 0 },
                   weight: 1.0 }] },
      { kind: CustomOutput(10),
        inputs: [{ source: InputLeaf { ref_idx: 7, sub_idx: 2 },
                   weight: 1.0 }] },
      { kind: CustomOutput(11),
        inputs: [{ source: InputLeaf { ref_idx: 7, sub_idx: 4 },
                   weight: 1.0 }] },
      { kind: CustomOutput(12),
        inputs: [{ source: InputLeaf { ref_idx: 7, sub_idx: 6 },
                   weight: 1.0 }] },
      // CustomOutput(13..23): empty (inert)
      // RouterGate(0..7): empty (default routing)
      // WriteSlot(0..15): empty (inert)
      // ClearSlot(0..15): empty (inert)
    ],

    action_bank: [
      // action_queue_cap (default 4) empty ActionSlots
      // All start disconnected — no graph-based action emission at founder.
      { behavior: Emit(NoOp), gate_inputs: [], param_inputs: [] },
      { behavior: Emit(NoOp), gate_inputs: [], param_inputs: [] },
      { behavior: Emit(NoOp), gate_inputs: [], param_inputs: [] },
      { behavior: Emit(NoOp), gate_inputs: [], param_inputs: [] },
    ],

    execute_gate: { inputs: [] },  // empty — no graph-based termination
  }),
  targets: [{ target_id: 1, slot: 0, gate_bias: 0.0 }],
}
```

**Node 1 — VM backend (typed nutrition decision + action emitter)**

```text
NodeGenome {
  node_id: 1,
  input_refs: [
    0: UpstreamSlot(0),   // any typed food here
    1: UpstreamSlot(1),   // can_reproduce (CN4)
    2: UpstreamSlot(2),   // maintenance-food neighbor N
    3: UpstreamSlot(3),   // maintenance-food neighbor E
    4: UpstreamSlot(4),   // maintenance-food neighbor S
    5: UpstreamSlot(5),   // maintenance-food neighbor W
    6: UpstreamSlot(6),   // reproductive food here
    7: UpstreamSlot(7),   // live reproductive reserve
    8: UpstreamSlot(8),   // live energy
    9: UpstreamSlot(9),   // reproductive-food neighbor N
    10: UpstreamSlot(10), // reproductive-food neighbor E
    11: UpstreamSlot(11), // reproductive-food neighbor S
    12: UpstreamSlot(12), // reproductive-food neighbor W
  ],
  backend_def: Vm(VmBackendDef {
    registers: 20,
    constants: [0.0, 1.0, 2.0, 4.0, 6.0, reserve_cost,
                offspring_transfer, energy_threshold],
    // The generated VM reads all upstream slots 0..12 and clears r15 to zero;
    // r16 through r19 are spare capacity for later small mutations.
    // Its profile-specific branch order is:
    //   1. canonical profile: reproduce when can_reproduce (slot 1) is true;
    //      forage-first profiles check this after their resource branches;
    //   2. if reserve (slot 7) < reserve_cost, Eat(type 1) when local
    //      reproductive food (slot 6) is present, otherwise seek its typed
    //      neighbor ring (slots 9..12);
    //   3. if energy (slot 8) < energy_threshold, Eat(type 0) when local
    //      maintenance food (slot 0) is present, otherwise seek slots 2..5;
    //   4. reproduce if the graph gate becomes ready, otherwise seek slots 2..5.
    // Every emitted action writes its metadata, uses PushAction, and terminates
    // the turn with ExecuteActionQueue.
    program: [ /* generated instruction sequence */ ],
  }),
  targets: [],
}
```

**Behavioral intent:**
- The graph node aggregates typed local/neighbor food, live energy, age, and
  reserve into fixed output slots using the CGP three-layer model.
- The VM node handles decision-making and action emission using `PushAction` +
  `ExecuteActionQueue`.
- Founders choose the deficient nutrient: reproductive food is preferred while
  reserve is below cost; maintenance food is preferred while energy is below
  threshold. Typed local Eat and typed cardinal movement use the corresponding
  food type only; one action is emitted per decision.
- Each typed movement branch computes the maximum across all four cardinal
  values before selecting a direction. It checks N, E, S, then uses W as the
  final equal-maximum fallback, so ties prefer N, then E, then S, then W.
  Direction metadata is the cardinal index `N=0`, `E=2`, `S=4`, `W=6`.
- Founders reproduce only when graph energy, age, and reserve gates all pass.

The age gate uses the half-tick threshold
`max(min_reproduce_age_ticks - 0.5, -0.5)` for strict `Threshold` semantics.
The reserve gate is separate: it uses the immediate representable `f32`
predecessor of `reproductive_reserve_cost`, so strict `>` is equivalent to
`reserve >= reproductive_reserve_cost`, including fractional costs.

This gives natural selection immediate material to work with: creatures that
find food and reproduce efficiently will out-compete those that do not.

**Determinism note:** The founder genome structure above is canonical for
v3alpha1 seeding. The exact bytewise encoding is an implementation detail, but
the structural layout (node count, node IDs, input_refs, compute nodes,
output sink wiring, VM instruction sequence, and constants) must be identical
across implementations for deterministic seeding with the same seed.

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

- Project-level determinism scope is canonical in root `AGENTS.md`.
- Runtime truthfulness invariant is canonical in `docs/strategy/architecture.md`
  (`Runtime Truthfulness Invariant`).
