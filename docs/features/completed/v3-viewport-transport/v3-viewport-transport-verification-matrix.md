# V3 Viewport Transport Verification Matrix

**Goal:** Provide one shared command source of truth for the viewport transport refactor program so the main plan and companions do not duplicate long verification blocks.

**Goal IDs:** GP-03, GP-04

**Scope:** Shared verification commands, benchmark checkpoints, and manual performance checks for the viewport transport refactor.

**Docs Impact:** none

**Supersedes:** none

**Superseded-By:** none

## Goal Alignment

- **GP-03:** Shared verification reduces drift and keeps refactor checkpoints reproducible.
- **GP-04:** Manual perf checks remain tied to behavior-backed server/frontend state.

## Boundary Impact

- This file is command-source-only; it does not define feature semantics.
- Companion plans reference this matrix instead of copying commands.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `docs/plans/README.md` verification DRY rule | keep | This matrix is the single command source of truth for the program. |
| Program companion plans | keep | Companions link here instead of duplicating commands. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should companions duplicate the command list? | No. Use this matrix as the canonical source. | user+agent | resolved |

## Baseline Commands

### Server baseline

- `cd v3 && cargo test -p v3-server --test server`
- `cd v3 && cargo bench -p v3-server --bench transport`

### Frontend baseline

- `cd frontend && npm run lint`
- `cd frontend && npm run test`
- `cd frontend && npm run build`

### Core baseline

- `cd v3 && cargo bench -p v3-core --bench tick`

## Phase Gates

### Recursive review gate

For every task in the program:
1. Run the implementation tests and benchmark checks required for that task.
2. Run the recursive review that matches the code touched by the task:
   - `rust-skills` for Rust code
   - `vercel-react-best-practices` plus `vercel-composition-patterns` for frontend code
   - `frontend-design` as part of the same loop when UI behavior or presentation changed
3. If the task touched both Rust and frontend code, run both review families.
4. Fix every new finding introduced by the task.
5. Rerun the same review family or families.
6. Repeat until the review loop produces no new findings.

No task passes or is marked complete until this recursive review gate is clean.

### Rust verification

- `cd v3 && cargo fmt --all -- --check`
- `cd v3 && cargo test --workspace`
- `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`

### Frontend verification

- `cd frontend && npm run lint`
- `cd frontend && npm run test`
- `cd frontend && npm run build`
- `cd frontend && npm run test:e2e`

### Docs and harness verification

- `scripts/check-doc-harness.sh --mode strict`
- `scripts/check-architecture-harness.sh --mode strict`
- `scripts/check-plan-harness.sh --mode strict`

## Benchmark Commands

### Server

- `cd v3 && cargo bench -p v3-server --bench transport`

### Core

- `cd v3 && cargo bench -p v3-core --bench tick`

## Manual Performance Checks

- `curl -s http://127.0.0.1:<port>/v3/simulation/status`
- `curl -s http://127.0.0.1:<port>/v3/simulation/snapshot`
- `/usr/bin/sample <pid> 10 -file /tmp/v3-perf/<phase>/sample.txt`
- `cd v3 && samply record -d 120 --save-only -o /tmp/v3-perf/<phase>/profile.json.gz env V3_SERVER_BIND_ADDR=127.0.0.1:<port> ./target/profiling/v3-server`

## Artifact Policy

- Store all benchmark and profiling artifacts under `/tmp/v3-perf/<phase>/<before|after>/`.
- No phase is complete without both before and after artifacts.

## Risks and Rollback

- If a command becomes stale, update this matrix first and then any references.
- If a benchmark is unstable, fix the fixture before treating results as gating evidence.

**Review cycles:** 1
