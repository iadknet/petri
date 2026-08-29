# Mutation Config Surface Cleanup

**Goal:** Remove the obsolete `mutation.input_auto_connect_chance` knob from the active runtime config surface while preserving backward-compatible config ingestion, and document the still-live topology newborn birth controls canonically.

**Goal IDs:** GP-02, GP-03, GP-04

**Scope:**
- In: backend config contract cleanup, transport compatibility handling for the obsolete field, frontend config panel/type cleanup, canonical docs sync for live mutation config fields, regression coverage
- Out: mutation behavior changes, new mutation operators, broad config UX redesign, unrelated runtime-config normalization work

**Docs Impact:**
- Add this implementation plan under `docs/features/in_progress/runtime-config-surface-cleanup/`
- Update `docs/reference/v3-runtime-config-spec.md` to remove stale runtime-config surface claims and add the live `mutation.topology_new_node_birth.*` fields
- Update any affected reference notes that still imply `input_auto_connect_chance` is live behavior

**Supersedes:** none

**Superseded-By:** none

> **Pre-promotion layout:** Checked `cd v3 && ...` verification commands below are preserved as historical evidence. Current Cargo commands run from the repository root.

---

## Goal Alignment

| Goal ID | Work Items |
|---------|------------|
| GP-02 | Remove a dead config knob from the frontend/backend surface so the runtime contract matches the actual mutation architecture instead of carrying stale transport/UI state. |
| GP-03 | Add regression coverage around config serialization/deserialization, transport compatibility, and panel rendering so the cleanup does not silently break startup/PATCH flows. |
| GP-04 | Make the observable config payload truthful by exposing only live controls and documenting the topology newborn policy knobs that actually affect mutation behavior. |

---

## Boundary Impact

- `crates/v3-core/src/config/`
  - Clean up `MutationConfig` so the obsolete field is no longer part of the active emitted config surface.
  - Preserve backward-compatible ingestion of legacy payloads that still include `mutation.input_auto_connect_chance`.
- `crates/v3-server/`
  - Keep HTTP startup/PATCH validation aligned with the effective runtime-config contract.
  - Ensure `GET /v3/simulation/config` and PATCH responses reflect only live config fields.
- `frontend/src/types/` and `frontend/src/components/config-panel/`
  - Remove the obsolete field from the typed config surface and config panel.
  - Keep the frontend aligned with the server-emitted config payload.
- `docs/reference/`
  - Make `v3-runtime-config-spec.md` the truthful canonical owner for the live mutation fields.

Wire-format impact:
- Intentional response-surface change: `mutation.input_auto_connect_chance` stops appearing in emitted config payloads.
- Backward-compatibility requirement: legacy startup/PATCH payloads that still send `mutation.input_auto_connect_chance` should continue to parse and be ignored for one cleanup cycle rather than failing under `deny_unknown_fields`.

---

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `v3-core` mutation engine | keep | No mutation behavior change is needed; the cleanup is contract truthfulness, not operator redesign. |
| `v3-core` config layer | change | This is the correct place to encode legacy-field acceptance versus active emitted runtime config. |
| `v3-server` transport | keep | Server remains a validation/merge layer over the canonical core config contract. |
| `frontend` config panel/types | change | The panel should expose only live knobs and typed payloads should match the server’s effective config schema. |
| `docs/reference/v3-runtime-config-spec.md` | change | The canonical mutation-config table is currently missing live birth-policy fields and should not imply stale UI knobs remain meaningful. |

---

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should the cleanup hard-break legacy configs that still send `mutation.input_auto_connect_chance`? | No. Accept the legacy key on ingress and ignore it, but stop emitting it in config responses and stop exposing it in the frontend. | Agent | resolved |
| Should the topology newborn birth fields remain configurable? | Yes. They are live mutation-policy controls used by topology newborn creation and should be documented canonically instead of treated as UI-only knobs. | Agent | resolved |

---

## Required Skills

- Rust implementation/review: `rust-skills`
- Frontend implementation/review: `vercel-react-best-practices`, `vercel-composition-patterns`, `frontend-design`
- Code review workflow: `superpowers:requesting-code-review`

---

## Task Details

### Task 1: Lock the intended contract with failing tests first

**Files:**
- Modify: `crates/v3-core/src/config/simulation.rs`
- Modify: `crates/v3-server/tests/server.rs`
- Modify: `frontend/src/components/ControlBar.test.tsx`
- Modify: `frontend/src/test/fixtures.ts`

**Steps:**
1. Add a backend/core test that deserializes a legacy mutation config containing `input_auto_connect_chance` and proves it is accepted without affecting the effective config.
2. Add a backend/server test for startup or PATCH showing a legacy payload with `mutation.input_auto_connect_chance` is accepted, while the returned `config` payload no longer includes that field.
3. Add or update a frontend test that fails if the config panel still renders the `Input Auto-Connect` control.
4. Run the targeted tests to capture the expected red state before implementation.

### Task 2: Implement the backend compatibility shim and active config cleanup

**Files:**
- Modify: `crates/v3-core/src/config/simulation.rs`
- Modify: `crates/v3-server/src/handlers/lifecycle.rs`
- Modify: `crates/v3-server/src/handlers/status.rs`
- Modify: `frontend/src/types/http.ts`

**Steps:**
1. Refactor the mutation config serde model so `input_auto_connect_chance` is no longer part of the active serialized config shape.
2. Add a legacy-ingest path in the core config layer so deserialization still accepts `input_auto_connect_chance` while dropping it from the normalized/effective config.
3. Confirm the server’s merge path still accepts legacy startup/PATCH payloads under `deny_unknown_fields`.
4. Update any transport typing that assumed the obsolete field remains in the canonical response payload.
5. Run the targeted backend tests and make them pass.

### Task 3: Remove the obsolete frontend surface and align docs with live behavior

**Files:**
- Modify: `frontend/src/components/config-panel/runtime/MutationSection.tsx`
- Modify: `frontend/src/types/config.ts`
- Modify: `frontend/src/types/http.ts`
- Modify: `frontend/src/test/fixtures.ts`
- Modify: `frontend/src/components/ControlBar.test.tsx`
- Modify: `docs/reference/v3-runtime-config-spec.md`
- Modify: `docs/reference/v3-mutation-spec.md`

**Steps:**
1. Remove the `Input Auto-Connect` field from the runtime config panel and frontend config types.
2. Update fixtures/tests so frontend config objects match the new emitted payload shape.
3. Add the live `mutation.topology_new_node_birth.graph_backend_chance`, `graph_initialized_chance`, and `graph_compute_gate_chance` fields to the canonical runtime-config table with defaults and normalization rules.
4. Update mutation-spec references as needed so runtime-config ownership stays centralized and the obsolete field is not described as live behavior.
5. Run the targeted frontend/doc checks and make them pass.

### Task 4: Regressions and completion verification

**Files:**
- No new product files expected

**Steps:**
1. Re-run all targeted tests that cover the changed backend, transport, and frontend config surface after the final code changes.
2. Re-run full required verification for the touched areas.
3. Document any intentional compatibility note in the completion summary so later cleanup can remove the legacy-ingest shim.

---

## Implementation Steps

- [x] Task 1: Add failing regression coverage for legacy `input_auto_connect_chance` ingestion, emitted config shape, and frontend panel rendering.
- [x] Task 2: Implement the backend config/transport cleanup so the obsolete field is ingress-compatible but no longer part of the active emitted config surface.
- [x] Task 3: Remove the obsolete frontend knob and update canonical docs for the live topology newborn birth fields.
- [x] Review Gate: Code review — run Rust review via `rust-skills`, frontend review via `vercel-react-best-practices`, `vercel-composition-patterns`, and `frontend-design`, request substantive review using `superpowers:requesting-code-review` when available, fix blocking findings, and re-review until clean.
- [x] Review Gate: Architecture & decomposition review — verify the cleanup preserves config ownership boundaries across `v3-core`, `v3-server`, `frontend`, and `docs/reference/`, and that the legacy-ingest shim does not leak transport concerns into mutation behavior.
- [x] Verification: `scripts/check-doc-harness.sh --mode strict`
- [ ] Verification: `scripts/check-architecture-harness.sh --mode strict`
- [x] Verification: `scripts/check-plan-harness.sh --mode strict`
- [x] Verification: `cd v3 && cargo test --workspace`
- [x] Verification: `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`
- [x] Verification: `cd frontend && npm run test`
- [x] Verification: `cd frontend && npm run build`

---

**Review note:** Review cycle completed against `docs/strategy/goals.md`, `docs/strategy/architecture.md`, root `AGENTS.md`, and `frontend/AGENTS.md`. No architecture conflict was found; the only material design choice was preserving legacy ingress compatibility while cleaning the active config surface.

**Execution note:** `check-architecture-harness --mode strict` is currently failing on pre-existing repository-wide violations in many untouched files; this plan leaves that baseline issue unchanged.

**Review cycles:** 1
