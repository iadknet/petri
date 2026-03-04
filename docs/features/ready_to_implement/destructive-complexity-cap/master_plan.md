# Destructive-Only Complexity Cap

**Goal:** Change restricted mutation to select only Decreasing operators, creating active pruning pressure instead of the current neutral+decreasing ceiling.

**Goal IDs:** GP-01

**Scope:**
- In: Change restricted operator selection from `random_non_increasing()` (Neutral+Decreasing) to `random_decreasing()` (Decreasing-only) in the mutation engine. Add `random_decreasing()` to each domain operator enum. Add `is_decreasing()` helper to `ComplexityEffect`. Skip event when no Decreasing operators exist for a domain.
- Out: Pressure curve changes, cap value changes, config toggles, complexity scoring changes.

**Docs Impact:** None — no canonical docs touched. This is a behavioral change within existing mutation engine internals.

**Supersedes:** none

**Superseded-By:** none

## Goal Alignment

- `GP-01`: Destructive-only pressure creates a grow→prune→grow oscillation cycle. Genomes near the cap are forced to shed junk structure, rewarding efficient cognitive architectures and enabling richer evolutionary dynamics.

## Boundary Impact

No crate or module boundaries change. All modifications are within `v3-core/src/mutation/`:
- `types.rs`: Add `is_decreasing()` method (mirrors existing `is_increasing()`)
- `topology/mod.rs`, `vm/mod.rs`, `graph/mod.rs`, `input_ref/mod.rs`: Add `random_decreasing()` method and `DECREASING_WEIGHT` const to each operator enum
- `engine/mod.rs`: Change restricted branches from `random_non_increasing()` with unrestricted fallback to `random_decreasing()` with skip-event fallback

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `v3-core/src/mutation/pressure.rs` | keep | Pressure probability calculation unchanged — only the operator pool changes |
| `v3-core/src/mutation/engine/mod.rs` | keep | Engine orchestration structure unchanged — only the restricted branch logic changes |
| `v3-core/src/mutation/types.rs` | keep | Adding a helper method, no structural change |
| Domain operator enums (topology, vm, graph, input_ref) | keep | Adding `random_decreasing()` parallel to existing `random_non_increasing()` |

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should there be a config toggle? | No — keep simple, revisit if issues arise | — | resolved |
| What if a domain has 0 Decreasing operators? | Skip that domain's mutation event | — | resolved |
| Does batch restriction behavior change? | No — restriction check still happens once per birth | — | resolved |
| Should `random_non_increasing()` be removed? | No — leave it for potential future use. It has no callers after the change, but removing it is churn with no benefit. | — | resolved |

## Required Skills

- Rust/backend changes: invoke `rust-skills` BEFORE writing any Rust code and before each review

## TDD Policy

For all behavior changes and bug fixes: write a failing test FIRST, then implement.
A step is not complete until:
1. The failing test exists and is committed
2. The implementation makes it pass
3. No existing tests regress

## Code Review Policy

After completing each implementation step:
1. Run a thorough code review (backend: `rust-skills`)
2. Fix ALL findings
3. Run review AGAIN — repeat until no new findings (clean recursive pass)
4. Only after clean pass: commit the step

## Commit Policy

- Commits happen AFTER a clean code review pass, never before
- One commit per implementation step (focused, atomic)
- Do NOT advance to the next step until current step is committed and reviewed clean

## Implementation Steps

- [ ] Step 1: Add `is_decreasing()` helper to `ComplexityEffect` in `types.rs`
  - Add `pub const fn is_decreasing(self) -> bool` to `ComplexityEffect` impl block
  - Add unit test verifying `is_decreasing()` returns true only for `Decreasing`

- [ ] Step 2: Add `random_decreasing()` to all four domain operator enums
  - In each of `TopologyOperator`, `VmOperator`, `GraphOperator`, `InputRefOperator`:
    - Add `DECREASING_WEIGHT` compile-time const (sum of weights for Decreasing-only operators)
    - Add `pub fn random_decreasing(rng) -> Option<Self>` that filters to `ComplexityEffect::Decreasing` only
  - Add unit tests per domain:
    - `random_decreasing` never returns Increasing or Neutral operators
    - `random_decreasing` returns `None` for VM domain (0 Decreasing operators)
    - `random_decreasing` covers all Decreasing operators for Topology, Graph, InputRef over enough seeds

- [ ] Step 3: Change engine restricted branches to use `random_decreasing()` with skip fallback
  - In `engine/mod.rs`, for each domain's restricted branch:
    - Replace `XOperator::random_non_increasing(rng).unwrap_or_else(|| XOperator::random(rng))` with:
      - Call `XOperator::random_decreasing(rng)`
      - If `None`, record attempt as skipped with `MutationSkipReason::NoApplicableTarget` and `continue`
      - If `Some(op)`, proceed as before
  - Update existing test `engine_pressure_at_cap_selects_only_non_increasing` → rename to `engine_pressure_at_cap_selects_only_decreasing` and assert only Decreasing operators are attempted when restricted
  - Add new test: restricted VM mutations are always skipped (VM has 0 Decreasing operators)
  - Add new test: accounting invariant still holds when events are skipped due to no Decreasing operators

- [ ] Step 4: Run viability tests and full test suite
  - `cargo test -p v3-core --test viability` — must pass (viability gate)
  - `cargo test --workspace` — must pass
  - `cargo clippy --workspace --all-targets -- -D warnings` — must pass
  - `cargo fmt --all -- --check` — must pass

**Review cycles:** 2
