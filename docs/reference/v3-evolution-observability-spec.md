# V3 Evolution Observability Spec

Reference specification for minimal required mutation/reproduction observability
signals in V3.

Status: Active

Related references:
- `v3-mutation-spec.md`
- `v3-reproduction-spec.md`
- `v3-creature-lifecycle-spec.md`
- `v3-server-api-protocol-spec.md`
- `v3-cli-contract-spec.md`

---

## 1. Purpose and Scope

This document defines the minimal required observability contract for:
- mutation processing outcomes,
- reproduction action outcomes.

Minimal required scope includes:
- required counters,
- required reason enums.

Event records are optional extensions for debugging/tuning and are specified as
"required fields if enabled."

This document is intentionally non-prescriptive about:
- storage format,
- transport/API shape,
- logging backend,
- retention policy.

Canonical v3alpha1 transport mappings for these observability semantics are
specified in `v3-server-api-protocol-spec.md` and
`v3-cli-contract-spec.md`.

---

## 2. Required Counters

Implementations must expose, at minimum:

### Mutation counters

- `mutation_events_attempted_total`
- `mutation_events_applied_total`
- `mutation_events_skipped_total`
- `mutation_events_skipped_total_by_reason`
- `mutation_events_attempted_total_by_domain`
- `mutation_events_applied_total_by_domain`
- `mutation_events_attempted_total_by_operator`
- `mutation_events_applied_total_by_operator`
- `mutation_events_applied_total_semantic_noop`
- `mutation_events_applied_total_semantic_change`

### Reproduction counters

- `reproduction_actions_attempted_total`:
  increments once per emitted reproduce action, regardless of later
  target/energy/mutation/spawn outcome.
- `reproduction_actions_spawned_total`:
  increments when a reproduce action successfully spawns a child.
- `reproduction_actions_rejected_total`:
  increments for reproduce actions that end in a
  `ReproductionActionResult` rejection.
- `reproduction_actions_rejected_total_by_reason`:
  same scope as `reproduction_actions_rejected_total`, partitioned by
  `ReproductionActionResult` reason.

Counter scope (tick-level, run-level, or both) may vary by implementation, but
the semantic meaning of each counter must remain consistent.

Genome complexity stats:
- Population-level `genome_complexity_mean`, `genome_complexity_min`, and
  `genome_complexity_max` report functional complexity (reachability-aware), not
  total genome size. Functional complexity excludes unreachable mesh nodes and
  dead instructions/graph nodes within reachable nodes. This means the stats
  reflect the actual behavioral complexity of creatures, not their total
  structural genome size.

Core accounting representation guidance:
- Core runtime accounting should use typed keys (domain/operator/reason enums)
  on hot paths.
- String-key maps are a transport concern and should be produced at API/CLI
  boundaries only.
- Counters with `last_tick_compute_*` naming represent simulation energy-cost
  accounting, not wall-clock latency.

Mutation accounting invariants:
- `mutation_events_attempted_total =
  mutation_events_applied_total + mutation_events_skipped_total`
- `sum(mutation_events_attempted_total_by_domain) =
  mutation_events_attempted_total`
- `sum(mutation_events_applied_total_by_domain) =
  mutation_events_applied_total`
- `sum(mutation_events_attempted_total_by_operator) +
  domain_level_skips <= mutation_events_attempted_total`
  (Domain-level skips occur when complexity pressure restricts a domain
  that has no eligible operators, e.g. VM under Decreasing-only restriction.
  These events are counted in `attempted_total` and `attempted_by_domain`
  but not in `attempted_by_operator` since no operator was selected.)
- `sum(mutation_events_applied_total_by_operator) =
  mutation_events_applied_total`
- `mutation_events_applied_total_semantic_noop +
  mutation_events_applied_total_semantic_change =
  mutation_events_applied_total`

---

## 3. Required Reason Enums

### `MutationSkipReason` (minimum)

- `ParseabilityViolation`
- `NoApplicableTarget`
- `BudgetExhausted`

### `ReproductionActionResult` rejections (minimum)

- `RejectedInvalidTarget`
- `RejectedEnergyConstraints`

These reason names may be mapped to local naming conventions, but one-to-one
semantic mapping must exist.

Required spawn-rejection semantic:
- `RejectedInvalidTarget`: spawn target was not valid at action-time check.
  This includes out-of-bounds, barrier, occupied, and same-tick contention
  cases under first-processed-wins action ordering.
- `RejectedEnergyConstraints`: spawn target was valid, but reproduce action
  failed energy/transfer validation gates (for example reproduce cost, minimum
  reproduce energy, or transfer constraints).

Optional diagnostic detail:
- Implementations may expose a non-normative local field (for example
  `invalid_target_cause`) for debugging breakdowns, but such sub-causes are not
  required by this contract.

Counter accounting invariant:
- `reproduction_actions_attempted_total =
  reproduction_actions_spawned_total +
  reproduction_actions_rejected_total`.

---

## 4. Optional Event Record Schemas (If Enabled)

Event records are not required by the minimal contract.

If an implementation emits event records, it should provide the following
minimum fields per event type:

### `MutationEvent`

- `tick`
- `event_index`
- `domain` (`Topology`, `Vm`, `Graph`, `InputRef`, or
  implementation-defined equivalent)
- `operator`
- `outcome` (`Applied` or `Skipped`)
- `skip_reason` (when skipped)
- `semantic_category` (`SemanticNoop` or `SemanticChange`, when applied)

### `ReproductionActionEvent`

- `tick`
- `actor_creature_id`
- `target_position`
- `outcome` (`Spawned` or rejection result)
- `rejection_reason` (when rejected)

---

## 5. Consumer Intent

The minimal contract supports:
- test assertions for mutation/reproduction behavior,
- debugging unexpected ecology behavior,
- tuning mutation and reproduction policies over time.
- consistent server/ws and CLI reporting with one-to-one semantic mapping from
  applied runtime behavior.

If an implementation cannot emit full event records, required counters and
reason breakdowns remain mandatory.

---

## 6. Transport-Facing Perf Telemetry

Wall-clock transport/runtime timings are a separate concern from the required
observability counters above.

Rules:
- Wall-clock timing fields (for example projection capture, view assembly, or
  payload size metrics) are transport/runtime telemetry, not simulation
  accounting.
- Implementations must not relabel behavior-backed energy-cost counters as
  wall-clock timings.
- If both are exposed together, APIs should distinguish them clearly, for
  example via separate field groups or unambiguous naming.

---

## 7. Counter Rename Note

This immediate-action model replaces earlier counter names.

Renamed counters:
- `reproduction_attempts_total` -> `reproduction_actions_attempted_total`
- `spawn_committed_total` -> `reproduction_actions_spawned_total`
- `spawn_rejected_total` -> `reproduction_actions_rejected_total`
- `spawn_rejected_total_by_reason` ->
  `reproduction_actions_rejected_total_by_reason`

Removed from minimum required set:
- `spawn_candidates_queued_total`

---

## 8. Policy References

- Project-level determinism scope is canonical in `AGENTS.md`
  (`Determinism Scope (Canonical)`).
- V3 runtime cognition reproducibility controls are canonical in
  `v3-mesh-execution-spec.md` (`Test-Mode Reproducibility Notes`).
- V3 tick ordering/arbitration reproducibility controls are canonical in
  `v3-tick-orchestration-spec.md` (`Test-Mode Reproducibility Notes (Tick
  Arbitration)`).
