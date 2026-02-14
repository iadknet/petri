# Petri — V2 Target Architecture

## Purpose

This document defines the active target architecture for the greenfield rewrite under `v2/`.
Legacy root crates and `web/` remain reference-only during this program.

**Goal IDs:** `GP-01`, `GP-02`, `GP-03`, `GP-04`

## Goal Alignment

- `GP-01`: center richer cognition/runtime behavior in a dedicated v2 runtime.
- `GP-02`: keep ownership boundaries explicit across runtime, transport, CLI, and web.
- `GP-03`: lock protocol contracts through checkpoint-scoped tests and fixtures.
- `GP-04`: surface deterministic run/status/frame/health telemetry to product surfaces.

## Boundary Impact

- Active dependency direction: `v2-core -> v2-server/v2-cli`.
- `v2-web` consumes protocol contracts from `v2-server`.
- No dependency from `v2/*` into legacy `petri-*` crates or root `web/`.
- Reuse is copy-only per `v2/docs/COPY_POLICY.md`.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/crates/v2-core` policy ownership | `keep` | Runtime semantics and deterministic policy remain core-owned. |
| `v2/crates/v2-server` transport ownership | `keep` | HTTP/WebSocket contracts and lifecycle error mapping remain server-owned. |
| legacy `crates/petri-*` + root `web/` | `keep` | Historical reference only; no active implementation coupling. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should legacy runtime contracts stay backward compatible with `v2`? | No; `v2` can break independently (`v2alpha1` contract ownership). | user+agent | resolved |
| Should snapshot endpoints be part of initial `v2` protocol? | No; out of scope for current checkpoint set. | user+agent | resolved |

## Repository Architecture

```text
petri/
├── v2/
│   ├── crates/
│   │   ├── v2-core/     # runtime/simulation policy
│   │   ├── v2-server/   # HTTP + WebSocket contracts for v2
│   │   └── v2-cli/      # deterministic NDJSON run/ablation surfaces
│   ├── web/             # v2 desktop client + protocol decoders
│   └── docs/            # local v2 boundary/copy-policy docs
├── crates/              # legacy runtime stack (reference-only for rewrite)
├── web/                 # legacy client (reference-only for rewrite)
└── docs/                # canonical strategy/plan/operations docs
```

## Runtime/Protocol Contract Surfaces

- `v2-core`:
  - mesh queue execution semantics
  - energy/backends primitives
  - deterministic runtime policy contracts
- `v2-server`:
  - `/v2/simulation/*` lifecycle/status/frame/paint contracts
  - non-2xx CP-3 error envelope
  - WebSocket event ordering and event/payload mapping
- `v2-cli`:
  - `run` and `ablation` command surfaces
  - deterministic NDJSON event shapes and field ordering
- `v2-web`:
  - typed models and runtime-safe decoders for `v2alpha1`
  - fixture-locked protocol parser tests

## Compatibility Posture

- Root compatibility stubs are pointer-only:
  - `petri-architecture.md`
  - `petri-roadmap.md`
  - `petri-technology-review.md`
- Legacy implementation/docs are retained for traceability, not as active targets.
