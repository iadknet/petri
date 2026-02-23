# V3 Remaining Docs Completion Plan

**Goal:** Complete the remaining canonical V3 docs required to start a clean
implementation path by adding startup/founder policy, full server
API/protocol, and minimal CLI output contracts.
**Goal IDs:** GP-01, GP-02, GP-03, GP-04
**Scope:** Docs-only work: create three new reference specs and apply targeted
integration edits in existing V3 reference docs; excludes code changes and broad
strategy rewrites.
**Docs Impact:**

| Doc | Action |
| --- | --- |
| `docs/reference/v3-startup-seeding-spec.md` | Create canonical startup/founder seeding policy owner |
| `docs/reference/v3-server-api-protocol-spec.md` | Create canonical v3alpha1 HTTP/WS/error/config transport contract |
| `docs/reference/v3-cli-contract-spec.md` | Create canonical minimal local-runner CLI NDJSON contract |
| `docs/reference/v3-creature-lifecycle-spec.md` | Targeted ownership links to startup/server/CLI specs |
| `docs/reference/v3-world-grid-spec.md` | Targeted ownership links for startup and server transport surfaces |
| `docs/reference/v3-runtime-config-spec.md` | Targeted ownership links for startup/config patch transport |
| `docs/reference/v3-evolution-observability-spec.md` | Targeted ownership links for server/CLI observability surfaces |
| `docs/reference/v3-tick-orchestration-spec.md` | Targeted ownership links for external lifecycle controls |
| `docs/plans/archive/2026-02-21-v3-remaining-docs-completion.md` | Create docs-only execution + readiness gate plan |

**Supersedes:** none

**Superseded-By:** none

## Goal Alignment

- `GP-01`: enables clean implementation of startup and lifecycle controls
  without relying on legacy behaviors.
- `GP-02`: clarifies ownership boundaries across startup, transport, and runtime
  contracts.
- `GP-03`: makes implementation planning high-confidence by removing contract
  ambiguity.
- `GP-04`: ensures observability surfaces are explicitly specified across
  server/CLI contracts.

## Boundary Impact

- Startup/founder policy is owned by `v3-startup-seeding-spec.md`.
- Server HTTP/WS/error/config transport schema is owned by
  `v3-server-api-protocol-spec.md`.
- CLI run-output schema is owned by `v3-cli-contract-spec.md`.
- Existing core runtime/world/tick/reproduction specs keep current behavioral
  ownership and consume new docs by reference.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `docs/reference/v3-world-grid-spec.md` | keep | World defaults/edge/validity ownership stays canonical in world spec. |
| `docs/reference/v3-runtime-config-spec.md` | keep | Runtime/energy/mutation defaults remain canonical here; transport schema moves to server API spec. |
| `docs/reference/v3-tick-orchestration-spec.md` | keep | Tick still owns queue/arbitration semantics; server lifecycle controls remain transport-level ownership. |
| `docs/reference/v3-evolution-observability-spec.md` | keep | Observability semantic requirements remain canonical while transport mappings link to server/CLI specs. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should protocol docs be minimal or full-surface in this pass? | Full v3alpha1 surface (HTTP + WS + config + errors). | user+agent | resolved |
| Should startup expose founder profile selection in v3alpha1? | No founder-profile request field in v3alpha1. | user+agent | resolved |
| Should invalid/non-viable startup requests normalize or reject? | Reject (`422 validation_rejected`) with no auto-normalization. | user+agent | resolved |
| Lifecycle endpoint posture? | `start`/`pause` idempotent, `step` state-restricted. | user+agent | resolved |
| Frame stream posture? | Sparse full-frame payloads for `GET /frame` and ws `frame` events. | user+agent | resolved |
| Snapshot endpoints in scope? | Out of scope for v3alpha1. | user+agent | resolved |
| Include CLI docs now? | Yes, minimal local-runner NDJSON contract. | user+agent | resolved |

## Task List

### Task 1: Create startup/founder canonical spec

**Files:**
- Create: `docs/reference/v3-startup-seeding-spec.md`

- [x] Define startup ownership boundaries vs world/tick/runtime/server transport.
- [x] Define deterministic startup seeding and founder baseline policy.
- [x] Define viability rejection semantics (`422`) with no normalization path.
- [x] Define startup invariants for first state/frame outputs.

### Task 2: Create full v3alpha1 server API/protocol spec

**Files:**
- Create: `docs/reference/v3-server-api-protocol-spec.md`

- [x] Define protocol version posture (`v3alpha1`) and state model.
- [x] Define canonical HTTP endpoint schemas:
      `/startup`, `/start`, `/pause`, `/step`, `/status`, `/frame`,
      config read/patch.
- [x] Define ws event envelope/payload mapping + ordering guarantees.
- [x] Define normative error envelope and required error codes.
- [x] Define out-of-scope snapshot posture.

### Task 3: Create minimal CLI contract spec

**Files:**
- Create: `docs/reference/v3-cli-contract-spec.md`

- [x] Define minimal local-core runner mode.
- [x] Define NDJSON schemas for `run_started`, `tick_sample`,
      `run_completed`.
- [x] Define protocol version and deterministic test expectations.

### Task 4: Apply targeted integration edits in existing specs

**Files:**
- Modify: `docs/reference/v3-creature-lifecycle-spec.md`
- Modify: `docs/reference/v3-world-grid-spec.md`
- Modify: `docs/reference/v3-runtime-config-spec.md`
- Modify: `docs/reference/v3-evolution-observability-spec.md`
- Modify: `docs/reference/v3-tick-orchestration-spec.md`

- [x] Add related-reference links to new canonical docs.
- [x] Add ownership-link language only (no broad rewrites).
- [x] Remove or defer duplicated startup/transport details.

### Task 5: Readiness gate and recursive review

**Files:**
- Modify: `docs/plans/archive/2026-02-21-v3-remaining-docs-completion.md`

- [x] Run self-review passes for cross-spec contradictions until stable.
- [x] Record readiness checklist and verification outcomes.

## Implementation Readiness Checklist

- [x] Startup/founder behavior is canonically owned by
      `v3-startup-seeding-spec.md`.
- [x] Transport/error semantics are canonically owned by
      `v3-server-api-protocol-spec.md`.
- [x] CLI output semantics are canonically owned by
      `v3-cli-contract-spec.md`.
- [x] World defaults remain canonical in `v3-world-grid-spec.md`.
- [x] Runtime defaults remain canonical in `v3-runtime-config-spec.md`.
- [x] No founder request field exists in v3alpha1 startup contract.
- [x] Invalid/non-viable startup requests reject with canonical 422 envelope.
- [x] Snapshot import/export endpoints are explicitly out of scope in v3alpha1.
- [x] Active docs contain no requirement to adapt prior `v3` code as migration
      input.
- [x] `scripts/check-doc-harness.sh --mode warn` passes.
- [x] `scripts/check-architecture-harness.sh --mode warn` passes.
- [x] `scripts/check-plan-harness.sh --mode strict` passes.

## Verification Commands

- `scripts/check-doc-harness.sh --mode warn`
- `scripts/check-architecture-harness.sh --mode warn`
- `scripts/check-plan-harness.sh --mode strict`
- `rg -n "v3-startup-seeding-spec.md|v3-server-api-protocol-spec.md|v3-cli-contract-spec.md" docs/reference/v3-*.md`
- `rg -n "founder_profile|validation_rejected|protocol_version|/v3/ws|/v3/simulation/startup" docs/reference/v3-*.md`

## Verification Results (2026-02-21)

- `scripts/check-doc-harness.sh --mode warn`: pass (`violations=0`)
- `scripts/check-architecture-harness.sh --mode warn`: pass (`violations=0`, baseline warnings only)
- `scripts/check-plan-harness.sh --mode strict`: pass (`violations=0`, `warnings=0`)

## Risks and Rollback

- Risk: duplicate ownership language between startup/server/runtime specs.
  - Mitigation: enforce canonical owner statements and reduce duplicate
    behavioral text to references.
- Risk: accidental behavioral drift from prior docs through broad edits.
  - Mitigation: limit edits to explicit file scope and targeted ownership links.
- Rollback: revert only touched reference docs and this plan file; preserve
  unaffected strategy/docs.

**Review cycles:** 2

Cycle 1: Derived decision-complete docs scope from active reference set and
locked protocol/startup/CLI defaults for v3alpha1.

Cycle 2: Cross-spec consistency + harness verification pass; no further
conflicts found after targeted ownership-link edits.
