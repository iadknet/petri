# Documentation Consistency Policy

This policy defines documentation quality constraints enforced by the doc harness.

## Core Rules

1. Prefer references over repetition.
   - Canonical docs should be linked from secondary surfaces instead of copied.
   - Root compatibility docs remain short pointers to canonical docs.
2. Avoid redundant canonical content.
   - Do not duplicate large narrative blocks across strategy documents.
   - If a concept is already documented in a canonical file, reference it.
3. Avoid conflicting statements.
   - Shared architectural invariants (for example crate dependency direction) must remain consistent across docs.
   - If an invariant changes, update all canonical sources in the same change.

## Enforcement

`scripts/check-doc-harness.sh` enforces:

- compatibility-stub constraints for root strategy docs
- repeated long-block detection across `docs/strategy/*.md`
- conflicting dependency-direction statement detection
- local markdown link integrity

## Authoring Guidance

- Place canonical content once, then link to it.
- Keep summaries short and avoid re-explaining standards in multiple files.
- When in doubt, update one source-of-truth doc and add references elsewhere.
