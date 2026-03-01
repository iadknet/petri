# V3 Extended Perception Identity Companion

**Goal:** Define creature identity as a first-class lifecycle domain so kin recognition and related perception signals have one canonical owner before implementation begins.

**Goal IDs:** GP-01, GP-02, GP-03

**Scope:** Identity state, founder initialization, offspring inheritance, and ownership of kin-tag mutation semantics. Excludes visibility geometry, runtime trace payloads, and detailed sensor formulas.

**Docs Impact:**

| Doc | Action |
| --- | --- |
| `docs/plans/2026-03-01-v3-extended-perception-identity-companion.md` | Create |
| `docs/plans/2026-03-01-v3-extended-perception-plan.md` | Link as parent/index plan |
| `docs/reference/v3-creature-identity-spec.md` | Create canonical identity contract |
| `docs/reference/v3-reproduction-spec.md` | Add identity inheritance trigger ownership |
| `docs/reference/v3-startup-seeding-spec.md` | Add founder identity linkage |
| `docs/reference/v3-creature-lifecycle-spec.md` | Add lifecycle summary linkage |

**Supersedes:** none

**Superseded-By:** none

**Parent plan:** `docs/plans/2026-03-01-v3-extended-perception-plan.md`

---

## Goal Alignment

- **GP-01:** Kin recognition becomes evolvable through stable inherited identity state rather than ad hoc generation-based heuristics.
- **GP-02:** Identity ownership moves into the creature/lifecycle domain instead of leaking into sensors or runtime tracing.
- **GP-03:** Founder initialization and inheritance rules become explicit, deterministic, and testable.

## Boundary Impact

- `creature/` becomes the canonical owner of `CreatureIdentityState`.
- `reproduction` owns the trigger timing for kin-tag mutation because it already owns the post-mutation child drafting flow.
- `startup seeding` owns founder ordering and startup seed flow, but not identity semantics themselves.
- `sensors/` consumes derived identity similarity values only; raw identity does not become a general-purpose sensor payload.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v3/crates/v3-core/src/creature/state.rs` | change | Creature state needs lifecycle-owned identity storage, not sensor-local scratch data. |
| `docs/reference/v3-reproduction-spec.md` | keep | Reproduction already owns child drafting and mutation trigger timing, so identity-trigger wiring belongs there. |
| `docs/reference/v3-startup-seeding-spec.md` | keep | Startup already owns founder seeding order and is the right consumer of founder identity initialization. |
| `v3/crates/v3-core/src/sensors` | keep | Sensors should read derived similarity signals, not own identity lifecycle state. |

## Identity Domain Overview

Identity adds a stable, inheritable signal for social behavior without exposing raw internal identifiers as general runtime inputs.

Core posture:
- `lineage_id` is stable founder/clade identity.
- `kin_tag` is a drifting family signature that changes slowly across generations.
- `CreatureId` remains the runtime occupancy/turn-order identifier and is not reused for kin semantics.

Canonical formulas and invariants live in `docs/reference/v3-creature-identity-spec.md`.

## New Types and Public Interfaces

Planned type additions:
- `CreatureIdentityState { lineage_id: u32, kin_tag: u32 }`
- `CreatureState.identity: CreatureIdentityState`

Public contract changes to track in implementation:
- offspring drafting/inheritance must produce identity state alongside genome, memory, and phenotype
- startup seeding must assign founder identity deterministically
- sensor-facing identity use must go through derived values such as `lineage_match` and `kin_affinity`

## Founder Initialization Rules

Startup seeding should initialize founder identity in deterministic founder-placement order:
- founders receive unique `lineage_id` values derived from final founder order
- founder `kin_tag` is derived from startup seed plus `lineage_id`
- the canonical algorithm belongs in the identity spec, with startup seeding referencing it rather than redefining it

This preserves reproducibility-sensitive tests without turning identity into a transport-configurable startup concern.

## Offspring Inheritance Rules

Reproduction should apply these rules:
- child inherits `lineage_id` unchanged from parent
- child inherits `kin_tag` unchanged when genome mutation applies zero events
- child mutates `kin_tag` only after reproduction observes `MutationSummary.applied_events > 0`
- the mutation operation itself belongs to `creature::identity`, not to the generic mutation engine

Phenotype and identity trigger timing should remain parallel but separate: both are reproduction-owned triggers driven by the mutation summary, but their mutation algorithms live in their respective domains.

## Ownership of Mutation Trigger vs Mutation Semantics

Ownership split:
- `reproduction` owns **when** kin-tag mutation is evaluated
- `creature::identity` owns **how** the kin-tag mutation is computed

This mirrors the existing phenotype posture and prevents the generic mutation engine from absorbing unrelated lifecycle state.

## Affected Code Areas

| area | expected change |
| --- | --- |
| `v3/crates/v3-core/src/creature/state.rs` | add identity field |
| `v3/crates/v3-core/src/creature/` | add identity module/helpers |
| `v3/crates/v3-core/src/simulation/startup*` or equivalent seeding path | assign founder identity deterministically |
| `v3/crates/v3-core/src/reproduction/` or reproduce action helpers | carry identity through child drafting |
| `docs/reference/v3-creature-identity-spec.md` | canonical state/init/inheritance rules |

## Task List

### Task 1: Add the canonical identity contract

**Files:**
- Create: `docs/reference/v3-creature-identity-spec.md`
- Modify: `docs/reference/v3-startup-seeding-spec.md`
- Modify: `docs/reference/v3-reproduction-spec.md`
- Modify: `docs/reference/v3-creature-lifecycle-spec.md`

- [ ] Define `CreatureIdentityState` and its invariants.
- [ ] Define founder initialization without duplicating startup flow policy.
- [ ] Define offspring identity inheritance and mutation-trigger ownership.

### Task 2: Wire identity into the future implementation plan

**Files:**
- Modify: this companion
- Modify: `docs/plans/2026-03-01-v3-extended-perception-plan.md`

- [ ] Keep code-file targets explicit for creature, startup, and reproduction modules.
- [ ] Keep sensor-facing identity use referenced rather than redefined here.

## Docs Impact

| category | docs |
| --- | --- |
| new canonical reference | `docs/reference/v3-creature-identity-spec.md` |
| updated canonical references | `docs/reference/v3-startup-seeding-spec.md`, `docs/reference/v3-reproduction-spec.md`, `docs/reference/v3-creature-lifecycle-spec.md` |
| companion linkage | this file plus the main plan |
| retired or superseded docs | none |

## Tests

Minimum future test coverage:
- deterministic founder `lineage_id` assignment by placement order
- deterministic founder `kin_tag` seeding from startup seed
- child inherits `lineage_id` unchanged
- child keeps `kin_tag` unchanged when `applied_events == 0`
- child mutates `kin_tag` only when `applied_events > 0`
- raw identity remains internal while sensor-facing similarity values remain bounded

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should raw identity values become public API or frame payload fields? | No. Keep them internal and expose only derived values through sensors when needed. | user+agent | resolved |
| Should the mutation engine own kin-tag mutation semantics? | No. Reproduction owns the trigger and identity owns the pure mutation rule. | user+agent | resolved |
| Should founder identity be configurable through startup requests? | No for v1. Founder identity remains deterministic and internal. | user+agent | resolved |

## Assumptions and Defaults

- `lineage_id` is stable across a lineage
- `kin_tag` is inherited and drifts slowly over generations
- founder identity initialization is deterministic for a fixed startup seed
- identity state is lifecycle-owned, not sensor-owned
- sensor specs consume derived identity similarity values only

## Verification Commands

- `scripts/check-plan-harness.sh --mode strict`
- `scripts/check-doc-harness.sh --mode strict`
- `rg -n "identity|lineage_id|kin_tag" docs/reference/v3-creature-identity-spec.md docs/reference/v3-startup-seeding-spec.md docs/reference/v3-reproduction-spec.md docs/reference/v3-creature-lifecycle-spec.md`

## Risks and Rollback

- Risk: identity semantics drift between startup, reproduction, and sensor docs.
  - Mitigation: keep `v3-creature-identity-spec.md` as the canonical owner and make the others reference it.
- Risk: future implementation treats identity as a sensor convenience field.
  - Mitigation: keep code targets centered on `creature/` and reproduction helpers, not `sensors/`.
- Rollback: remove the identity companion only if the main plan absorbs all identity guidance without reintroducing duplication.

**Review cycles:** 3
