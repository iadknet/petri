---
name: promote-to-refinement
description: Use when a feature idea is ready to be fleshed out with problem statement, acceptance criteria, sizing, and dependencies before planning implementation.
---

# features:promote-to-refinement

**Announce:** "Using features:promote-to-refinement to create a structured refinement doc."

## Overview

Creates `docs/features/needs_refinement/FEATURE-NAME.md` with the full refinement schema. If the idea comes from `ideas.md`, move it to a `## Promoted` section there.

## Steps

1. **Determine FEATURE-NAME**: kebab-case, descriptive (e.g., `storage-slots`, `age-energy-cost`).
2. **Source check**: If idea exists in `docs/features/brainstorms/ideas.md`, move it to a `## Promoted` section at the bottom after creating the refinement file. If new/described directly, skip ideas.md.
3. **Fill schema** interactively with the user:
   - `title`: Human-readable name
   - `tags`: simulation | frontend | core | ui | genome | (etc.)
   - `size`: S | M | L | XL (see size guide below)
   - `depends-on`: list of kebab-case feature names this depends on
4. **XL size nudge**: If size is XL, prompt: "This feature is rated XL. Consider whether it can be split into smaller independent features before proceeding. If not, document the rationale for keeping it unified in the Problem Statement."
5. **Dependency check**: For each entry in `depends-on`, verify the referenced feature exists somewhere in `docs/features/` (any stage). Warn if not tracked.
6. **Write file** with `status: draft`. Fill all schema sections.
7. Set status to `needs-review` once all fields are filled.

## Size Guide

- **S**: <1 day / single crate change
- **M**: 1–3 days / multiple crates
- **L**: 1–2 weeks / cross-cutting
- **XL**: multi-week / major architectural change

## File Schema

```markdown
---
title: Human-readable Feature Name
tags: [simulation | frontend | core | ui | genome | ...]
size: S | M | L | XL
depends-on: []
status: draft
---

## Problem Statement
Why does this feature exist? What problem does it solve?

## User Stories / Acceptance Criteria
- As a [role], I want [X] so that [Y]

## Out of Scope
- Explicit exclusions

## Open Questions
| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| (question here) | (pending) | (owner) | open |
```

## Status Lifecycle

`draft` (created) → `needs-review` (all fields filled) → `ready` (only set by `features:promote-to-ready`)

Never manually set status to `ready`.
