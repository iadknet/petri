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
- deterministic startup seeding contract for terrain, fertility, food, and founder placement;
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
- `population` (initial counts, limits, and the founder profile; canonical
  defaults in `v3-runtime-config-spec.md` Section 5),
- `world` (`width`, `height`, `edge_mode`, `terrain`, `world_seed`, `food.shared`, `food.types`,
  `food.fertility`),
- `energy` (lifecycle and action-cost controls),
- `runtime` (mesh/vm/graph/mutation controls),
- `startup` (startup-only controls such as early-run ramps).

v3alpha1 policy:
- `population.founder_profile` selects one founder genome profile for the whole
  seeded population (Section 5). It is optional; absent means `v3_alpha1`.
- Unknown/unsupported startup fields and unrecognized profile names are request
  validation failures.

Canonical wire-level field schemas for startup are owned by
`v3-server-api-protocol-spec.md`.

The normalized production food catalog contains one green (`#22c55e`)
`Primary Food`, initial density `1.0`, coverage `0.54`. Food types inherit the shared live
`energy.costs.eat_reward_per_food` reward (default `5.0`) unless
`energy_per_unit` overrides it. Growth and recovery likewise inherit shared
rates unless their optional type fields override them. `initial_fertility_only`
defaults false; true filters tick-zero placement to positive effective fertility
(including annealing), before the existing per-type shuffle and coverage count.
All type fields remain restart-only; shared inherited runtime values remain live.

---

## 4. Deterministic Startup Seeding Contract

For identical startup inputs and seed:
- applied terrain, fertility, and seeded food distribution are deterministic,
- founder spawn attempt ordering is deterministic,
- resulting initial world state and initial creature set are deterministic.

Determinism is required for harnesses and reproducibility-sensitive assertions.
Production runtime determinism remains non-required at product level (canonical
policy in `AGENTS.md`).

Seeding flow contract:
1. Build effective startup config from defaults plus accepted overrides.
2. Create the empty world and apply configured terrain layers in order, before
   fertility, food, or founders. Bounds and union rules are owned by
   `v3-world-grid-spec.md` Section 4. Derive each independent layer seed from its
   explicit seed or `world.world_seed.unwrap_or(run_seed).wrapping_add(index)`.
3. Seed fertility using that effective map seed and its existing layer overrides.
   Terrain consumes no run-RNG draws. Clustered Noise sorts candidate cells
   before seeded shuffle/truncation so hash iteration cannot select barriers.
4. Seed world food using world food initialization semantics from
   `v3-world-grid-spec.md`.
   - v3alpha1 seeds exact coverage over non-barrier cells:
     `round(initial_coverage * eligible_cells)` unique cells.
   - Seeded density uses normalized `f32` food values in canonical `[0.0, 1.0]`
     scale (clamped by `world.food.shared.max_density`).
5. Seed founders from the founder profile selected per Section 5 (default:
   the canonical v3alpha1 genome of Section 5.1).
6. Commit simulation state at tick `0` in `idle` state.

---

`SmallRng` remains the world-generation and run generator: the locked `rand`
0.8.6 wrapper and `rand_xoshiro` 0.6.0 `Xoshiro256PlusPlus` have different
`seed_from_u64` expansion and streams. Reproducibility is for the locked
versions/platform, not a cross-version/platform guarantee. New benchmark
reports record the actual locked `rand_version`; historical absence is
unmeasured. `noise` remains lockfile-pinned. Empty terrain and an absent map seed
preserve production run draws and trajectories.

## 5. Founder Baseline Policy (v3alpha1)

v3alpha1 seeds every founder in a run from one founder genome profile, selected
by `population.founder_profile` (canonical key: `v3-runtime-config-spec.md`
Section 5). The default `v3_alpha1` is the canonical founder genome of
Section 5.1; the four `forage_first_sparse*` profiles are tuned variants of it
tabulated there.

Rules:
- All startup-seeded founders in a run derive from the same founder genome
  profile.
- Startup-seeded founders begin at `generation = 0`.
- Startup-seeded founders receive deterministic identity state per
  `v3-creature-identity-spec.md`:
  - `lineage_id` is assigned from final founder placement order
  - `kin_tag` is derived deterministically from startup seed and `lineage_id`
- Profile selection is a run-level startup input; the API offers no
  per-founder profile selection.
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

The founder genome is a 2-node mesh of two Graph nodes (T19.F04; node 1 was a
VM before), using the CGP-style layered model (see
`v3-graph-backend-spec.md`). Its `genome_size()` is 97 units
(`FOUNDER_GENOME_SIZE_UNITS`). The founder reproduction gate is derived from
runtime config `energy.lifecycle.min_reproduce_age` (default `20` ticks).

**Structural layout:**

```text
CreatureGenome {
  entry_node_id: 0,
  nodes: [Node0, Node1],
}
```

**Node 0 — Graph backend (sensor aggregator)**

The five input references are primary FoodHere, live EnergyCurrent, AgeTicks,
primary NeighborFoodRing, and NeighborOccupiedRing. Three compute nodes apply
strict energy `Threshold` (a fraction of `max_energy`, the scale
`EnergyCurrent` reads on), age `Threshold((min_reproduce_age - 0.5) / age_reference_ticks)` (the scale
`AgeTicks` reads on, both values from the lifecycle config at seeding, exact at
every integer age while `min_reproduce_age <= age_reference_ticks`), and
Multiply of
those gates. Six custom outputs carry food here,
can-reproduce, and N/E/S/W primary food. Its other output sinks remain
unwired. The graph routes to node 1 in slot 0.

**Node 1 — Graph backend (decision by votes)**

Node 1 reads node 0's six slots (`UpstreamSlot(0..6)`) and the
`ActionQueue` compound input (reference 6) and votes; it has no route target,
so every pass ends at it (`NoTargets`). With `f = [food_here > 0]`, `can`
the reproduce gate (slot 1), `ring[d]` the cardinal food slots,
`q = [queue slot 0 holds a Reproduce]` (built as
`Threshold(2.5) - Threshold(3.5)` on the slot's action type), and the
profile's reproduce gate `g` (`can` for V3Alpha1; `can · (1 - f)`, built as
`Threshold(0.5)` over `can - f`, for the ForageFirst profiles):

| Sink | Vote |
| --- | --- |
| `Eat` | `f - 2g - 2q` (V3Alpha1) or `1 - 2g - 2q` (ForageFirst) |
| `Move[d]`, `d` in N, E, S, W | `0.5 + 0.4·ring[d] - 2g - 2q` |
| `Reproduce[d]`, `d` in N, E, S, W | `g·(0.5 + 0.4·ring[d])` |
| `Terminate` | `q` |
| `ActionParam(Reproduce, 1)` | the profile's transfer fraction (a `Constant` node) |

`population.founder_profile` selects one row below by wire name. A profile
changes only the strict energy threshold in Node 0 and the priority and
transfer fraction (the `Constant` wired into `ActionParam(Reproduce, 1)`, the
share of the parent's post-cost energy the child starts with) in Node 1; the 2-node mesh, its input references, compute nodes, and output
wiring are shared by every profile. The energy threshold is the founder's own
constant on the unit scale (it gates on fullness and is not recomputed from
`max_energy`); at the default `max_energy` (200) each sits at least 2.0 above
`min_reproduce_energy`, and every fraction clears the `initial_energy` litter
floor at its own threshold after the age-1.0 reproduce cost, so a founder
attempt at or above its gate is never refused on energy. A lifecycle where
`threshold * max_energy - 1.0 < min_reproduce_energy` pins the founder at its
gate (`v3-runtime-config-spec.md` Section 4).

| Profile | Wire name | Strict energy threshold (fraction of `max_energy`) | Energy at default `max_energy` | Transfer fraction | Priority |
| --- | --- | ---: | ---: | ---: | --- |
| V3Alpha1 | `v3_alpha1` | 0.16 | 32 | 2/3 | Reproduce, local forage, Move fallback |
| ForageFirstSparse | `forage_first_sparse` | 0.16 | 32 | 2/3 | Local forage, Reproduce, forage fallback |
| ForageFirstSparseConservative | `forage_first_sparse_conservative` | 0.30 | 60 | 0.35 | Local forage, Reproduce, forage fallback |
| ForageFirstSparseRichOffspring | `forage_first_sparse_rich_offspring` | 0.20 | 40 | 0.60 | Local forage, Reproduce, forage fallback |
| ForageFirstSparseBalanced | `forage_first_sparse_balanced` | 0.25 | 50 | 0.45 | Local forage, Reproduce, forage fallback |

The votes act as the truth table under the pass loop
(`v3-mesh-execution-spec.md` Section 2): with food and no reproduce gate the
tick commits `Eat` then the best `Move` (pass 3 ends `NoDecision`); without
food, `Move` alone (V3Alpha1; the ForageFirst profiles still commit `Eat`
first, retaining normal failed-Eat accounting); with the gate open, one
`Reproduce`, after which `q` suppresses every other vote and the next pass
ends the tick. Every `Move` and `Reproduce` takes the full primary-food
cardinal maximum with N/E/S/W tie order (the lowest-index argmax over sinks
`0, 2, 4, 6`); `Eat` reads type 0 (its parameter is unwired). With
`max_actions_per_turn = 1` the first commit ends the tick, so capacity one
keeps `Eat`. No second food type is read.

**Determinism note:** The founder genome structure above is canonical for
v3alpha1 seeding. The exact bytewise encoding is an implementation detail, but
the structural layout (node count, node IDs, input_refs, compute nodes,
output sink wiring, and constants) must be identical
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


## Recipe Save and Load

A procedural recipe is a partial or complete `SimulationConfig` JSON object,
with no run seed. App Load and CLI `run --config` share the core startup resolver:
recursive object merge over defaults, strict deserialization, startup ramp
validation, normalization, then startup overrides. The server retains its
injected baseline for tests. Arrays/nonobjects replace wholesale, including null.
Save exports the full applied config, not a frontend subset or pending edits.
With the same run seed and locked dependencies/platform, reloading regenerates
tick-zero terrain, fertility, food and founders. Fixing `world.world_seed` alone
does not fix food or founders. Runtime painting and living/evolved state are
excluded; they require the separately scoped overlay/checkpoint features.
