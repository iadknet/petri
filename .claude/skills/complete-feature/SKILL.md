---
name: complete-feature
description: Use when all implementation steps in an in-progress feature's master_plan.md are checked off and the feature is ready for final quality gates.
---

# features:complete-feature

**Announce:** "Using features:complete-feature to run final gates and close out FEATURE-NAME."

## Overview

Runs a post-implementation architecture review, executes the full AGENTS.md completion gate, finishes the development branch, then moves the feature directory to `docs/features/completed/`.

## Post-Implementation Architecture Review (Before Final Gates)

1. Review the completed implementation holistically for decomposition opportunities, separation of concerns, and domain boundaries uncovered during implementation.
2. **Easy fixes** (cosmetic, obvious): fix immediately and commit.
3. **Larger enhancements or refactors** revealed by the implementation: add each as a new entry in `docs/features/brainstorms/ideas.md` with context ("uncovered during FEATURE-NAME implementation").
4. Incorporate any easy-fix commits (post-review).

## Completion Gate (Full AGENTS.md Gate)

Run ALL of the following and confirm they pass:

```bash
# Harness checks
scripts/check-plan-harness.sh --mode strict
scripts/check-doc-harness.sh --mode strict
scripts/check-architecture-harness.sh --mode strict

# Rust quality gates
cd v3 && cargo fmt --all -- --check
cd v3 && cargo test --workspace
cd v3 && cargo clippy --workspace --all-targets -- -D warnings
```

If frontend changes were made: confirm all `agent-browser` e2e tests pass.

**REQUIRED SUB-SKILL:** Run code review via `superpowers:requesting-code-review`.

**REQUIRED SUB-SKILL:** Use `superpowers:verification-before-completion` — confirm all gate output before claiming completion.

**Do not claim completion until all gates pass with confirmed output.**

## Finish Development Branch

**REQUIRED SUB-SKILL:** Use `superpowers:finishing-a-development-branch` to handle merge, PR creation, and worktree cleanup.

## Move to Completed

After the worktree is merged and closed:

```bash
mv docs/features/in_progress/FEATURE-NAME/ docs/features/completed/FEATURE-NAME/
git add docs/features/completed/FEATURE-NAME/
git commit -m "feat: mark FEATURE-NAME as completed"
```

Commit the completed/ move on main after merge.
