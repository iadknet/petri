# Plan Quality Gate Policy (v1)

This document defines the mechanical planning-quality checks enforced by:

- `scripts/check-plan-harness.sh --mode warn`
- `scripts/check-plan-harness.sh --mode strict`

## Effective Date

Strict blocking for this harness is effective immediately on February 13, 2026.

## Scope

The harness checks:

- Active plan files in `docs/plans/*.md` (excluding `docs/plans/README.md` and `docs/plans/archive/*`)
- `docs/strategy/architecture.md`
- Active plan files matching `*architecture*.md`

Archived plans are out of scope for strict/warn enforcement.

## Required Plan Metadata

Each active plan file must include:

- `**Goal:**`
- `**Goal IDs:**`
- `**Scope:**`
- `**Docs Impact:**`
- `**Supersedes:**`
- `**Superseded-By:**`

## Required Plan Sections

Each active plan file must include:

- `## Goal Alignment`
- `## Boundary Impact`
- `## Existing Boundary Recheck`
- `## Open Questions`

## Architecture-Doc Requirements

`docs/strategy/architecture.md` and active `*architecture*.md` plans must include:

- `**Goal IDs:**`
- `## Goal Alignment`
- `## Boundary Impact`
- `## Existing Boundary Recheck`
- `## Open Questions`

## Goal Catalog Contract

- Canonical source: `docs/strategy/goals.md`
- Goal IDs listed in plan metadata must exist in the goals catalog.
- The `Goal Alignment` section must reference each declared goal ID.

## Boundary Evidence Rule

`## Existing Boundary Recheck` must include at least two reviewed existing areas.  
Each reviewed area must include:

- area identifier (crate/module/file)
- decision (`keep` or `change`)
- rationale

## Ambiguity Rule

`## Open Questions` must use a Markdown table with columns:

- `question`
- `decision`
- `owner`
- `status`

Strict-mode failure conditions:

- any row where `status` is not `resolved`
- unresolved markers in decision-critical fields (`TBD`, `TODO`, `???`)

Warn-mode behavior:

- findings are reported but the command exits `0`

## Summary and Exit Contract

Harness output includes:

- `Plan harness summary: mode=<mode> violations=<n> warnings=<m> files_checked=<k>`

Exit codes:

- warn mode: always `0`
- strict mode: non-zero when violations are present

## Intent Completeness Overlay

Plan-harness conformance does not, by itself, prove implementation completeness.

- Apply `docs/standards/intent-verification-policy.md` alongside this harness policy.
- Completion claims require intent-level and integration-level verification evidence, not metadata/structure compliance alone.
