# Mutation Quality Improvements

**Source:** Branch `codex/inspector-shared-memory-integration` (excluded from merge due to
conflicts with the reachability-aware mutation system on main).

## Context

These changes were developed on a branch that predated the reachability-aware mutation
feature merge to main. They conflict structurally because the branch also refactored
module layout (merging `operators.rs` into `mod.rs`, deleting `reachability.rs`, etc.)
while main added the reachability infrastructure that depends on those same files.

The behavioral improvements are independent of the structural refactoring and should
be re-implementable on top of main's current architecture.

## Behavioral Changes (worth re-implementing)

### 1. Bounded mutation indices

**Problem:** Several graph mutation operators generated indices across the full integer
range (e.g., `rng.gen()` for `u8`) instead of bounding to valid ranges. The runtime
silently handled out-of-range values (resolving to 0.0 or no-ops), making mutations
semantically meaningless.

**Fix applied on branch:**
- `random_graph_node_kind`: `CustomOutput` bounded to `0u8..8` (was `rng.gen()`)
- `mutate_operator_param`: Added modular bounds for `CustomOutput` (`% 8`),
  `WriteActionMeta` (`% 8`), `PushAction` (`% 5`) to prevent wrapping to invalid values
- `apply_graph_raw_field_mutation`: All parameterized kinds bounded to valid ranges

**Where to apply on main:** `v3/crates/v3-core/src/mutation/graph/operators.rs`

### 2. Always-backlink on copy operations

**Problem:** `apply_copy_internal_node` and `clone_and_remap_slice` had a 50% chance
of adding a backlink from existing nodes to the copied structure. Without a backlink,
copied nodes/slices are unreachable and their output is never consumed, making the
copy operation effectively wasted.

**Fix applied on branch:**
- `apply_copy_internal_node`: Always adds a backlink edge from a random existing node
  to the copied node
- `clone_and_remap_slice` (topology): Always adds a backlink from a random pre-existing
  node to a random cloned node

**Where to apply on main:**
- `v3/crates/v3-core/src/mutation/graph/operators.rs` (`apply_copy_internal_node`)
- `v3/crates/v3-core/src/mutation/topology/mod.rs` (`clone_and_remap_slice`)

## Structural Changes (reassess before re-implementing)

### 3. Module consolidation

The branch merged separate `operators.rs` / `tests.rs` files into their parent `mod.rs`
files across `graph/`, `vm/`, `input_ref/`, and `engine/`. This is a stylistic preference
that reduced file count but increased individual file sizes. Orthogonal to the behavioral
improvements above.

### 4. Removal of reachability bias

The branch removed `TargetReachability`, `biased_select_from`, `ReachableBiasConfig`, and
related telemetry. This conflicts directly with main's reachability-aware mutation feature,
which was merged after this branch was created. **Do not re-apply** — main's reachability
system is the current direction.

## Reference

Original branch preserved at: `codex/inspector-shared-memory-integration`
