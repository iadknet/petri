---
name: rust-skills
description: Use whenever writing, reviewing, or refactoring Rust code; routes to focused local rules for the changed code.
license: MIT
metadata:
  author: leonardomso
  version: "1.5.1"
---

# Rust rules router

Use this skill for every Rust implementation, refactor, and review. Read the
relevant rule files in `rules/` before changing code; do not load the whole
catalogue by default. Apply the highest-risk route first, then only categories
that affect the changed behavior. Repository instructions and the task's
acceptance criteria take precedence where they are more specific.

## Always route

- Ownership and mutation: `own-`, `type-`, and `anti-`.
- Errors or fallibility: `err-`, `api-`, and `pat-`.
- Tests: `test-`; read `doc-` when public API documentation changes.
- Public interfaces: `api-`, `trait-`, `conv-`, `name-`, and `doc-`.
- Review: route to the changed code's categories plus `anti-` and `lint-`.

## Route by change

| Change | Read these rule families |
| --- | --- |
| Async tasks, channels, or cancellation | `async-`, `own-`, `conc-` |
| Threads, locks, atomics, or Rayon | `conc-`, `async-`, `own-`, `test-` |
| `unsafe`, FFI, or manual `Send`/`Sync` | `unsafe-`, `type-`, `test-`, `doc-` |
| Allocation or hot-loop work | `mem-`, `perf-`, `opt-`, `anti-` |
| Numeric values or conversions | `num-`, `conv-`, `type-` |
| Serde or external data | `serde-`, `type-`, `api-`, `err-` |
| Macros or generated code | `macro-`, `anti-`, `doc-` |
| Closures and callbacks | `closure-`, `type-`, `own-` |
| Collections and iteration | `coll-`, `perf-`, `anti-` |
| Logging or diagnostics | `obs-`, `err-` |
| Crate/workspace layout or lint policy | `proj-`, `lint-` |

## Reading rules

Rule files are named `<family>-<topic>.md`; use `rg --files rules` or a targeted
prefix search to locate the needed files. Read an individual rule before relying
on it, especially for `unsafe-`, `async-`, `conc-`, `err-`, and `api-` choices.
Do not introduce performance machinery without an observed need or a task
requirement, and do not treat a lower-priority style rule as grounds to expand a
focused change.

## Completion check

Before handing off Rust work, confirm the applicable focused tests and project
checks ran, no selected rule was knowingly violated without a documented reason,
and the change did not add avoidable cloning, panics for expected failures, lock
holding across `.await`, or undocumented unsafe invariants.
