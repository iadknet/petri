# Creature Detail API Optimization

**Goal:** Reduce creature-detail polling bandwidth by adding HTTP compression,
incremental action_log fetching (`since_tick`), sparse field exclusion
(`exclude`), and documenting the endpoint in the API spec.

**Goal IDs:** GP-02, GP-04

**Scope:**
- In: tower-http CompressionLayer on Axum router; `since_tick` query parameter
  for action_log filtering; `exclude` query parameter for sparse field
  selection; frontend `useCreatureDetail` and `api.getCreature()` updates;
  API spec documentation section.
- Out: pagination, binary encoding for creature detail, WebSocket push for
  creature detail, action log persistence, sample endpoint changes.

**Docs Impact:**
- `docs/reference/v3-server-api-protocol-spec.md` — new section 4.11 for
  `GET /v3/simulation/creature/:id`

**Supersedes:** none
**Superseded-By:** none

---

## Goal Alignment

| Goal ID | Alignment |
|---------|-----------|
| GP-02 | Clean boundaries: query parameters parsed in HTTP handler, filtering logic stays in handler, no simulation-layer changes. CompressionLayer applied at transport layer only. |
| GP-04 | Observable behavior: creature detail endpoint is the primary runtime introspection surface. Reducing bandwidth ensures it remains responsive at high tick rates. Documentation closes an API spec gap. |

---

## Boundary Impact

- **v3-server (transport layer only)**: All changes are in the HTTP handler and
  router layers. No changes to v3-core simulation logic, creature state, or
  action log data structures.
- **Dependency addition**: `tower-http` crate added to `v3-server/Cargo.toml`
  (with `compression-gzip` and `compression-br` features). Added as workspace
  dependency in `v3/Cargo.toml`.
- **Frontend**: Changes to `api/rest.ts` (query parameter construction),
  `hooks/useCreatureDetail.ts` (incremental fetch logic), and
  `stores/creatureInspector.ts` (incremental action_log merging, optional
  genome/memory in setDetail). Type updates in `types/genome.ts`.
- **No wire format breaking changes**: All new query parameters are optional.
  Omitting them returns the full existing response (backward compatible).

---

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `v3/crates/v3-core/src/creature/action_log.rs` | keep | Ring buffer and ActionLogEntry stay as-is; filtering is done in the server handler at serialization time. |
| `v3/crates/v3-server/src/handlers/creature.rs` | change | Handler gains query parameter parsing and conditional field inclusion. Stays within its existing responsibility of assembling creature detail responses. |
| `v3/crates/v3-server/src/lib.rs` | change | Router gains CompressionLayer middleware. This is a transport-layer concern that belongs here. |
| `frontend/src/hooks/useCreatureDetail.ts` | change | Hook gains incremental fetch logic (since_tick tracking, exclude parameter). Stays within its existing responsibility. |
| `frontend/src/stores/creatureInspector.ts` | change | Store gains incremental action_log merge in setDetail. Stays within its existing responsibility. |
| `frontend/src/api/rest.ts` | change | API client gains query parameter support for getCreature. Stays within its existing responsibility. |

---

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should `exclude` use whitelist or blacklist pattern? | Blacklist (`exclude=genome,action_log`) for backward compatibility — omitting the parameter returns the full response. | agent | resolved |
| Should `since_tick` return `tick > since_tick` or `tick >= since_tick`? | Strictly greater (`tick > since_tick`) — avoids duplicating the boundary entry the client already has. | agent | resolved |
| Should compression be applied globally or per-content-type? | Global via `CompressionLayer::new()` — tower-http defaults to compressing only compressible content types. | agent | resolved |
| Minimum response size threshold for compression? | Use tower-http default (no minimum) initially. | agent | resolved |
| Should `latest_tick` be top-level or nested? | Top-level `latest_tick` field in the response, always present for convenience. | agent | resolved |

---

## Required Skills

- Rust/backend changes: invoke `rust-skills` BEFORE writing any Rust code and before each review
- Frontend changes: invoke `vercel-react-best-practices` and `vercel-composition-patterns`
  BEFORE writing any frontend code and before each review

---

## TDD Policy

For all behavior changes and bug fixes: write a failing test FIRST, then implement.
A step is not complete until:
1. The failing test exists and is committed
2. The implementation makes it pass
3. No existing tests regress

---

## Code Review Policy

After completing each implementation step:
1. Run a thorough code review (backend: `rust-skills`; frontend: vercel skills)
2. Fix ALL findings
3. Run review AGAIN — repeat until no new findings (clean recursive pass)
4. Only after clean pass: commit the step

---

## Commit Policy

- Commits happen AFTER a clean code review pass, never before
- One commit per implementation step (focused, atomic)
- Do NOT advance to the next step until current step is committed and reviewed clean

---

## Implementation Steps

- [ ] Step 1: **tower-http CompressionLayer** — Add `tower-http` as a workspace
  dependency in `v3/Cargo.toml` with `compression-gzip` and `compression-br`
  features. Add it to `v3-server/Cargo.toml`. Import and apply
  `CompressionLayer::new()` as a layer on the router in `v3-server/src/lib.rs`.
  Write a test that verifies gzip-compressed responses when `Accept-Encoding:
  gzip` is sent. Run `cargo test --workspace`, `cargo clippy`, `cargo fmt`.

- [ ] Step 2: **`since_tick` query parameter (server)** — Add a `CreatureQuery`
  struct with optional `since_tick: Option<u64>` to the `get_creature` handler
  in `v3-server/src/handlers/creature.rs`. When `since_tick` is present, filter
  `action_log` entries to only those with `entry.tick > since_tick`. Add a
  `latest_tick` field to the response (the max tick across all action_log
  entries, or 0 if empty). Write tests: full response without since_tick
  (backward compat), filtered response with since_tick, empty result when
  since_tick is ahead of all entries.

- [ ] Step 3: **`exclude` query parameter (server)** — Extend `CreatureQuery`
  with optional `exclude: Option<String>`. Parse comma-separated field names
  (`genome`, `action_log`, `memory`). When a field is excluded, omit it from the
  JSON response entirely (do not set to null). Use `serde_json::Map` for manual
  response construction to support conditional field inclusion. Write tests:
  exclude=genome omits genome, exclude=genome,action_log omits both, no exclude
  returns full response.

- [ ] Step 4: **Frontend: incremental fetching** — Update
  `frontend/src/api/rest.ts` `getCreature()` to accept optional
  `since_tick` and `exclude` query parameters. Update
  `frontend/src/types/genome.ts` `CreatureDetail` to make `genome`,
  `action_log`, and `memory` optional fields. Update
  `frontend/src/hooks/useCreatureDetail.ts` to: (a) track `latestTick` ref,
  (b) after first successful full fetch, pass `since_tick` and
  `exclude=genome` on subsequent polls, (c) merge incremental action_log
  entries into the store. Update `frontend/src/stores/creatureInspector.ts`
  `setDetail` to handle optional fields and incremental action_log merging.
  Run `npm run build` to verify no type errors.

- [ ] Step 5: **API spec documentation** — Add section 4.11 to
  `docs/reference/v3-server-api-protocol-spec.md` documenting the
  `GET /v3/simulation/creature/:id` endpoint: path parameters, query
  parameters (`since_tick`, `exclude`), full response schema, ActionLogEntry
  field definitions, error cases (404).

- [ ] Review Gate: Interim code review — review Steps 1-5 changes. Invoke
  `rust-skills` for backend, `vercel-react-best-practices` for frontend. Fix
  findings, re-review until clean.

- [ ] Review Gate: Code review — dispatch `superpowers:code-reviewer` subagent
  on full branch diff. Invoke domain skills (backend: `rust-skills`; frontend:
  `vercel-react-best-practices` + `vercel-composition-patterns`). Fix all
  findings. Re-review until clean pass.

- [ ] Review Gate: Architecture & decomposition review — review all changes for
  boundary violations, decomposition opportunities, separation of concerns.
  Re-read `docs/strategy/` and relevant `AGENTS.md` files. Fix easy issues,
  capture larger items in `docs/features/brainstorms/ideas.md`. Repeat until
  clean pass.

- [ ] Completion gate — run all checks from AGENTS.md Completion Gate section:
  1. `scripts/check-doc-harness.sh --mode strict`
  2. `scripts/check-architecture-harness.sh --mode strict`
  3. `scripts/check-plan-harness.sh --mode strict`
  4. `cd v3 && cargo fmt --all -- --check`
  5. `cd v3 && cargo test --workspace`
  6. `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`
  7. `cd frontend && npm run build`

---

**Review cycles:** 3
