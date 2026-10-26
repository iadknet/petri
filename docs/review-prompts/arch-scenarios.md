# Architecture review: planned work and proportion

{{RULES}}

## Target

- Checkout: `{{WORKTREE}}`
- The architecture under review: `docs/architecture.md`, its import checker
  `scripts/architecture-check.mjs` and its debt list
  `scripts/architecture-debts.txt`.
- Every open roadmap row (derived from the unchecked rows under
  `docs/roadmaps/`):
{{OPEN_ROWS}}

## Method

The question is whether the architecture is the right size: clean enough that
the planned work lands without workarounds or tangled code, and no larger than
that work and today's code need. Two parts.

1. Planned work. For each open row, read its row line and the "Notes for AI
   Agents" entries of its track that name it, and any design note the row
   cites; do not read specs or code beyond what settles a question. Name the
   domains the row touches and the rules and prohibitions that bind it, then
   report:
   - under-structure: the row must cross a boundary with no owner operation to
     call, needs an owner the document does not name, or must build on a debt
     before the row that pays it lands, so the document as written pushes it
     toward a workaround or a forbidden import;
   - over-structure: the document requires a layer, view, indirection or owner
     that only this row would use, or that none of the open rows needs.
2. Proportion. For each rule in section 1, prohibition and owner in section 3,
   and debt in section 5, name the current code or open row that needs it,
   with a citation, and say whether the import checker enforces it or only
   review does. Report a rule, prohibition or owner that neither current code
   nor any open row needs as over-structure, with the deletion that removes
   it. Then list the most-changed files under `crates/v3-core/src` since
   2026-09-20 (`git log --since=2026-09-20 --format= --name-only --
   crates/v3-core/src | sort | uniq -c | sort -rn | head -15`) and report a
   file repeatedly patched for a reason no rule or owner covers as
   under-structure. Report detail that belongs in rows and specs rather than
   in the document (it should stay a short map of owners, invariants and
   boundaries) as over-structure.

Grade a finding P1 when the document as written forces an open row into an
unmarked workaround or a forbidden import; P2 when it adds structure no current
code or open row needs, or lacks an owner or rule an open row will hit; P3 for
wording or detail in the wrong place. A larger design flaw that one change to
the document cannot fix is an ungraded `step-back:` line.

## Output

First the coverage: one line per open row, then one per rule, prohibition,
owner and debt, each `clear` or naming its finding and what was examined. Then
any `step-back:` lines. Then one finding per line, most severe first:
`P1: section or row — under-structure|over-structure — issue — evidence —
smallest change` (or `P2:`, `P3:`), ending `[verified]` when you checked it
against the code, the documents or the history, or `[plausible]` when it is
reasoned but not confirmed. Then a `Could not verify:` list. Nothing else.
