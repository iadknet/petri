# V2 Frontend State Matrix

This matrix is the frontend interaction contract companion for:
- `docs/plans/2026-02-14-v2-frontend-wireframe-spec.md`
- `docs/plans/2026-02-14-v2-cp3-api-protocol-spec.md`

## App Modes and Allowed Actions

| simulation state | startup draft edit | start | pause | step | paint stroke | paint clear_all | inspector select | notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `idle` | yes | yes | no | no | yes | yes | yes | startup draft is authoritative source for next run |
| `running` | no | idempotent no-op | yes | no | no | no | yes | paint actions must return `409 invalid_state_transition` |
| `paused` | limited (runtime-safe only) | yes | idempotent no-op | yes | yes | yes | yes | step preserves `paused` |

## Paint Tool Matrix

| tool | effect | phase support | required request fields |
| --- | --- | --- | --- |
| `food` | set food density in brush footprint | `idle`, `paused` | `action=stroke`, `tool`, `brush_half_extent`, `points[]` |
| `barrier` | set barriers in brush footprint | `idle`, `paused` | `action=stroke`, `tool`, `brush_half_extent`, `points[]` |
| `erase_food` | clear food in brush footprint | `idle`, `paused` | `action=stroke`, `tool`, `brush_half_extent`, `points[]` |
| `erase_barrier` | clear barriers in brush footprint | `idle`, `paused` | `action=stroke`, `tool`, `brush_half_extent`, `points[]` |

Paint preview/commit rule:
1. During an active drag, the viewport renders a local preview overlay only.
2. Canonical world state must remain unchanged until the pointer-up batched commit request succeeds.

## Brush Matrix

| brush label | `brush_half_extent` | footprint |
| --- | --- | --- |
| `1x1` | `0` | single cell |
| `3x3` | `1` | centered square radius 1 |
| `5x5` | `2` | centered square radius 2 |

## UX Error Matrix

| condition | expected UI response | transport response |
| --- | --- | --- |
| paint in `running` | non-blocking toast + inline phase badge | `409 invalid_state_transition` |
| invalid brush/tool payload | inline form error + toast | `400 invalid_request` or `422 validation_rejected` |
| protocol version mismatch | blocking banner, disable mutating actions | error envelope or decoder rejection |
| websocket disconnect | reconnect indicator, retain last frame snapshot | connection close/retry cycle |
