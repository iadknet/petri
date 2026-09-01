---
name: rust-skills
description: >
  Route Rust writing, review, and refactoring work to the relevant focused rules
  for ownership, errors, APIs, async, safety, performance, testing, and related
  concerns without loading the entire rule library.
license: MIT
metadata:
  author: leonardomso
  version: "1.5.1"
  sources:
    - Rust API Guidelines
    - Rust Performance Book
    - Rust 2024 Edition Guide
    - The Rustonomicon
    - ripgrep, tokio, serde, polars, axum, cargo codebases
---

# Rust Skills Router

Use this skill for every Rust code change or Rust-focused review. The detailed
guidance lives in `rules/`; load only the rule files that can change a decision
for the affected code. Do not read all 265 rules by default.

## Workflow

1. Inspect the requested behavior, affected Rust paths, nearby code, compiler
   constraints, and tests.
2. Select the smallest relevant categories from the table below.
3. List matching rules, then read the specific rule files whose names apply:

   ```sh
   rg --files .agents/skills/rust-skills/rules | rg '/(own|err|test)-'
   ```

   Replace the prefixes with those relevant to the work. A filename is a routing
   hint, not permission to assume its contents.
4. Apply repository instructions and existing domain invariants first. Use these
   rules to improve implementation choices, not to introduce unrelated cleanup,
   dependencies, abstractions, or performance work.
5. Verify behavior with the narrowest relevant tests and the repository's normal
   completion gate. For review, report only findings supported by the diff and
   applicable rules.

## Category routing

| Concern | Prefix | Use when |
| --- | --- | --- |
| Ownership and borrowing | `own-` | Choosing borrows, moves, clones, shared ownership, or lifetimes |
| Error handling | `err-` | Designing or propagating recoverable failures |
| Memory optimization | `mem-` | Allocation, representation, reuse, or measured memory pressure matters |
| Unsafe code | `unsafe-` | Adding, changing, or reviewing any `unsafe` operation or contract |
| API design | `api-` | Changing public types, functions, builders, traits, or compatibility |
| Async/await | `async-` | Futures, runtimes, cancellation, tasks, channels, or async I/O |
| Concurrency | `conc-` | Threads, atomics, locks, parallel iteration, or thread-local state |
| Compiler optimization | `opt-` | Profiling identifies code-generation or hot-path concerns |
| Numeric safety | `num-` | Arithmetic, casts, bounds, floats, counters, or simulation values |
| Type safety | `type-` | Modeling states, IDs, validation, nullability, or invariants |
| Traits and generics | `trait-` | Bounds, dispatch, extension points, or generic API design |
| Conversions | `conv-` | `From`, `TryFrom`, parsing, or lossy conversion boundaries |
| Const and compile time | `const-` | Constants, const evaluation, compile-time checks, or static data |
| Serde | `serde-` | Serialization formats, defaults, compatibility, or validation |
| Pattern matching | `pat-` | Exhaustiveness, guards, destructuring, or control flow |
| Macros | `macro-` | Declarative or procedural macro design and diagnostics |
| Closures | `closure-` | Capture, callable bounds, callback storage, or borrowing |
| Collections | `coll-` | Collection choice, entry APIs, lookup, ordering, or iteration |
| Naming | `name-` | Introducing or reviewing public and domain names |
| Testing | `test-` | Any behavior change, regression fix, fixture, or test architecture |
| Documentation | `doc-` | Public APIs, examples, safety/error sections, or hidden doctests |
| Observability | `obs-` | Logs, tracing, metrics, spans, or diagnostic context |
| Performance patterns | `perf-` | Profiling-supported algorithmic or hot-path improvements |
| Project structure | `proj-` | Crate/module boundaries, features, dependencies, or workspace layout |
| Clippy and linting | `lint-` | Lint configuration, suppressions, warnings, or CI policy |
| Anti-patterns | `anti-` | A proposed or reviewed construct resembles a known Rust pitfall |

## Minimum routing expectations

- Behavior changes and bug fixes: inspect applicable `test-` rules plus the
  category owning the behavior.
- Arithmetic or simulation mechanics: inspect applicable `num-`, `type-`, and
  `test-` rules.
- Public API or serialized state: inspect applicable `api-`, `type-`, `serde-`,
  and `test-` rules.
- Async or concurrent code: inspect applicable `async-` or `conc-` rules and any
  ownership/error rules implicated by the design.
- Unsafe code: always inspect every applicable `unsafe-` rule and document the
  safety contract; do not treat this as an optional optimization category.
- Performance changes: require evidence first, then inspect the relevant
  `mem-`, `opt-`, or `perf-` rules. Do not optimize speculatively.

## Decision discipline

- Prefer the repository's established patterns when they satisfy the same safety
  and correctness constraints.
- Keep changes scoped to the requested outcome. A relevant rule may identify a
  review concern; it does not automatically authorize adjacent refactoring.
- Compiler, Clippy, and test output are evidence. Never mark a rule satisfied
  merely because the intended code shape looks idiomatic.
- If rules conflict, prioritize correctness and safety, then public contracts,
  measured performance, and finally style. Surface a material unresolved tradeoff
  instead of silently choosing.
