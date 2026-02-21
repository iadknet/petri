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
- `population` (initial counts and limits),
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
- Diversity is introduced after startup via mutation/reproduction policy, not via
  startup profile randomization.

Founder genome structure, parseability, and runtime interpretation remain owned
by `v3-genome-spec.md`, `v3-mutation-spec.md`, and
`v3-mesh-execution-spec.md`.

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
