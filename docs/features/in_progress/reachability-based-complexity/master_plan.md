# Reachability-Based Complexity: Implementation Plan

**Goal:** Split the complexity metric into `genome_size()` (total) and `complexity()` (reachability-aware functional) so creatures are not penalized for junk DNA.

**Goal IDs:** GP-01, GP-03, GP-04

**Scope:**
- In: `genome_size()` method, reachability-aware `complexity()`, config renames, call site migration, intra-node dead code analysis (VM + Graph), API + frontend updates, cached complexity on CreatureState
- Out: Caching reachable set on CreatureState (deferred to reachability-aware-mutation), new mutation operators, genome viewer junk DNA fading, active mesh observability panel

**Docs Impact:**
- `docs/reference/v3-runtime-config-spec.md` — update `complexity_cap` → `genome_size_cap` and complexity cost semantics
- `docs/reference/v3-evolution-observability-spec.md` — note that `genome_complexity_*` stats now reflect functional complexity
- `docs/reference/v3-mutation-spec.md` — note pressure gate uses `genome_size`, not `complexity`

**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- **GP-01** (Evolve Richer Creature Decision-Making): Removing junk-DNA energy penalties lets creatures accumulate exploratory genetic material, increasing the search space for richer cognition without paying action-cost penalties for inert structure.
- **GP-03** (Keep Iteration High-Confidence): TDD approach with tests for each metric; viability tests validate that the economic change doesn't break founder population sustainability.
- **GP-04** (Keep Behavior Observable): Functional complexity is a more meaningful metric for observing actual creature behavior than total genome size. API and frontend updates surface this.

## Boundary Impact

- **`creature/genome/mod.rs`**: Add `genome_size()` method, refactor `complexity()` to call new reachability-aware computation. Public API change — both methods are `pub`.
- **`creature/genome/analysis.rs`**: Add `functional_complexity()` function composing mesh reachability + intra-node backward slicing. Internal to `creature` module.
- **`config/simulation.rs`**: Rename `complexity_cap` → `genome_size_cap`. Wire format change (serde).
- **`mutation/engine/mod.rs`**: Change pressure gate from `genome.complexity()` to `genome.genome_size()`.
- **`simulation/tick.rs`**: Action cost call sites switch to `creature.cached_complexity` (reading cached functional complexity).
- **`simulation/actions/mod.rs`**: Action cost call sites (`apply_noop`, `apply_eat`, `apply_move`, and test helpers) switch to `creature.cached_complexity`.
- **`simulation/actions/predation.rs`**: Kill bonus and steal cost use `creature.cached_complexity`.
- **`simulation/actions/reproduction.rs`**: Reproduce cost uses `creature.cached_complexity`.
- **`v3-server/src/handlers/creature.rs`**: API response emits both `complexity` and `genome_size`.
- **`v3-server/src/handlers/lifecycle.rs`**: Population stats compute complexity from cached value.
- **`v3-server/src/state.rs`**: `HealthPayload` field types unchanged (still `genome_complexity_*`).
- **Frontend**: Type updates, store updates, display updates — no new components.
- **Dependency direction**: All changes flow within existing boundaries. `creature` module owns both metrics. No new cross-crate dependencies.

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `creature/genome` owns complexity computation | keep | Both `genome_size()` and `complexity()` are genome-level metrics. Analysis utilities in `creature/genome/analysis.rs` provide the reachability primitives. No reason to move computation elsewhere. |
| `config/simulation.rs` owns config fields | keep | Renaming `complexity_cap` → `genome_size_cap` stays within the existing config module. No boundary change. |
| `mutation/engine` consumes genome metrics | keep | Engine calls `genome.genome_size()` instead of `genome.complexity()`. Same dependency direction (engine depends on creature). |
| `v3-server` computes stats from v3-core | keep | Server iterates creatures and reads `.cached_complexity`. Same pattern, just different field. |
| Frontend consumes API response | keep | Frontend reads `complexity` field from API. Adding `genome_size` is additive. |

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should Store* memory operations count as VM output instructions? | Yes — use existing `vm_is_output_instruction()` which includes StoreMem8 and StoreMem8Imm. No StoreMemF32 exists in the codebase. | agent | resolved |
| Cache functional complexity at birth? | Yes — compute once after mutation, store as `pub cached_complexity: u32` on `CreatureState` (matching existing field visibility convention). Genome doesn't change after birth so this is safe. | agent | resolved |
| Frontend display genome_size? | No — only functional complexity in existing displays. genome_size is available via API for future features. | user | resolved |
| Rename complexity_cap → genome_size_cap? | Yes — config is not versioned, breaking changes acceptable. Server and frontend must be updated together. | user | resolved |
| Performance of population-wide stats? | Acceptable — with cached complexity on state, population stats are field reads. | agent | resolved |
| Performance of functional_complexity()? | Build a `HashMap<NodeId, usize>` index once inside `functional_complexity()` for O(1) node lookups (same pattern as `mesh_backward_slice`). Cached at birth so computed once per creature. | agent | resolved |
| Complexity cost threshold tuning? | The threshold default (50) was tuned against total complexity. Functional complexity will be lower (junk DNA excluded), so fewer creatures exceed threshold. Viability tests will catch if this breaks economics. If needed, threshold can be adjusted post-merge. | agent | resolved |

## Required Skills

- Rust/backend changes: invoke `rust-skills` BEFORE writing any Rust code and before each review
- Frontend changes: invoke `vercel-react-best-practices` and `vercel-composition-patterns` BEFORE writing any frontend code and before each review

## TDD Policy

For all behavior changes and bug fixes: write a failing test FIRST, then implement.
A step is not complete until:
1. The failing test exists and is committed
2. The implementation makes it pass
3. No existing tests regress

## Code Review Policy

After completing each implementation step:
1. Run a thorough code review (backend: `rust-skills`; frontend: vercel skills)
2. Fix ALL findings
3. Run review AGAIN — repeat until no new findings (clean recursive pass)
4. Only after clean pass: commit the step

## Commit Policy

- Commits happen AFTER a clean code review pass, never before
- One commit per implementation step (focused, atomic)
- Do NOT advance to the next step until current step is committed and reviewed clean

## Call Site Audit

Complete inventory of `.complexity()` call sites and their target migration:

### Production call sites

| File | Line(s) | Current call | Migration target |
|------|---------|-------------|-----------------|
| `creature/genome/mod.rs` | 319 | `complexity()` method def | Refactor to delegate to `functional_complexity()` |
| `simulation/actions/mod.rs` | 18, 37, 70 | `creature.genome.complexity()` in apply_noop/eat/move | `creature.cached_complexity` |
| `simulation/actions/mod.rs` | 188, 213 | `sim.creatures[id].genome.complexity()` in test helpers | `sim.creatures[id].cached_complexity` |
| `simulation/actions/reproduction.rs` | 139 | `sim.creatures[parent_id].genome.complexity()` | `sim.creatures[parent_id].cached_complexity` |
| `simulation/actions/predation.rs` | 64 | `sim.creatures[attacker_id].genome.complexity()` steal cost | `sim.creatures[attacker_id].cached_complexity` |
| `simulation/actions/predation.rs` | 112 | `sim.creatures[victim_id].genome.complexity()` kill bonus | `sim.creatures[victim_id].cached_complexity` (intentional: kill reward reflects functional value, not junk DNA) |
| `simulation/tick.rs` | 346, 374, 419, 470 | `creature.genome.complexity()` failed action penalties | `creature.cached_complexity` |
| `mutation/engine/mod.rs` | 38 | `genome.complexity()` pressure gate | `genome.genome_size()` |
| `v3-server/.../creature.rs` | 89 | `creature.genome.complexity()` API response | `creature.cached_complexity` (+ add `genome_size`) |
| `v3-server/.../lifecycle.rs` | 240 | `creature.genome.complexity()` pop stats | `creature.cached_complexity` |

### Test-only call sites

| File | Line(s) | Migration |
|------|---------|-----------|
| `creature/genome/mod.rs` | 528, 557, 600, 635 | Become `genome_size()` tests; add parallel `complexity()` tests |
| `simulation/tick.rs` | 898, 983, 1005 | Use `cached_complexity` or `genome.complexity()` as appropriate |
| `simulation/actions/mod.rs` | 769, 777 | Update to use `genome_size()` or `complexity()` per test intent |
| `simulation/actions/predation.rs` | 304, 329, 340, 360, 399 | Use `cached_complexity` |

## Implementation Steps

### Step 1: Add `genome_size()` and `functional_complexity()` with tests

**Files:** `v3/crates/v3-core/src/creature/genome/mod.rs`, `v3/crates/v3-core/src/creature/genome/analysis.rs`

This step implements both metrics and wires `complexity()` to use functional complexity. Steps 1 and 2 must be implemented and committed together (or in immediate sequence) to avoid exposing the expensive `functional_complexity()` on every hot-path `.complexity()` call without caching.

1. **TDD first**: Write failing tests for:
   - `genome_size()` returns same value as current `complexity()` for any genome
   - `complexity()` excludes unreachable mesh nodes (genome with entry→A→B and disconnected C; complexity should not count C)
   - `complexity()` excludes dead VM instructions within reachable nodes (VM program with output instruction + dead instructions; only output chain counts)
   - `complexity()` excludes dead Graph internal nodes within reachable nodes (Graph with output kind + disconnected internal nodes; only output chain counts)
   - `complexity()` counts StoreMem8/StoreMem8Imm as live outputs (via `vm_is_output_instruction()`)
   - `complexity()` for a fully-connected genome equals `genome_size()` (no junk → same result)
   - `complexity()` counts only input_refs consumed by live backend components

2. **Implement `genome_size()`**: Rename current `complexity()` body to `genome_size()`. Existing complexity tests become `genome_size()` tests.

3. **Implement `functional_complexity()` in `analysis.rs`**:
   - Signature: `pub fn functional_complexity(genome: &CreatureGenome) -> u32`
   - Build a `HashMap<NodeId, usize>` index once for O(1) node lookups (same pattern as `mesh_backward_slice` at analysis.rs:449)
   - Inline the mesh reachability BFS using the pre-built index (avoid calling `mesh_reachable_nodes()` which does its own O(n) linear scans — build index first, then BFS with O(1) lookups)
   - For each reachable node:
     - Count +1 for the node itself
     - Count targets on this node
     - For VM backend:
       - Find output instructions using existing `vm_is_output_instruction()` — do NOT re-enumerate the list
       - Run `vm_backward_slice` from each output instruction, union results into a `HashSet<usize>`
       - Count live instructions (those in the union set)
       - Count constants referenced by live instructions (trace `LoadConst { const_idx }` in the live set)
     - For Graph backend:
       - Find output internal nodes by kind: CustomOutput, RouterOutput, WriteActionMeta, PushAction, PopAction, ExecuteActionQueue
       - Run `graph_backward_slice` from each, union results into a `HashSet<usize>`
       - Count live internal nodes + their inputs (edges)
     - Count consumed input_refs:
       - VM: collect `ref_idx` values from `ReadInput { dst, ref_idx, sub_idx }` instructions in the live set
       - Graph: collect `ref_idx` values from internal nodes with `kind = GraphNodeKind::InputRef { ref_idx, sub_idx }` that are in the live set
   - Use `with_capacity` for internal collections where sizes are known
   - Return total score

4. **Wire `complexity()` to call `functional_complexity()`**: The `complexity()` method on `CreatureGenome` now delegates to `analysis::functional_complexity()`.

- [x] Step 1: Add `genome_size()` method and reachability-aware `complexity()` with TDD

### Step 2: Add `cached_complexity` to `CreatureState` and wire caching

**Files:** `v3/crates/v3-core/src/creature/state.rs`, `v3/crates/v3-core/src/simulation/actions/reproduction.rs`, `v3/crates/v3-core/src/simulation/seeding.rs`

**Important:** This step MUST be committed in the same session as Step 1 to avoid hot-path performance regression. Between Step 1 (complexity() becomes expensive) and Step 2 (caching), every action cost calculation would recompute full reachability analysis.

1. Add `pub cached_complexity: u32` field to `CreatureState` (matching existing field visibility convention — all `CreatureState` fields are `pub`). Document the invariant: "Genome is immutable after creation; cached value is always current."
2. At creature creation (seeding): compute `genome.complexity()` and pass to state constructor.
3. At offspring creation (reproduction): construct `CreatureState::new()` AFTER `MutationEngine::apply_mutations()` completes, so the genome passed to `new()` already contains mutations and `cached_complexity` reflects the offspring's actual functional complexity.
4. Compute `cached_complexity` inside `CreatureState::new()` from the genome parameter rather than accepting it as a separate argument (prevents passing incorrect values). Since the computation is internal to `new()`, most `CreatureState::new()` call sites (including test fixtures in `sensors/`, `runtime/`, etc.) require no code changes — they recompile automatically.
5. **No-mutation fast path:** In the reproduction path, check `MutationSummary::applied_events`. If zero, the offspring's genome is identical to the parent's — copy the parent's `cached_complexity` directly instead of recomputing `functional_complexity()`. This avoids the expensive mesh BFS + backward slicing for the common case where `mutation_probability` gates most births. Use a separate constructor or setter (e.g., `CreatureState::new_with_cached_complexity(genome, complexity)`) for this path, keeping the default `new()` always-compute path for seeding and test fixtures.
6. Update any test fixtures that explicitly assert on or construct `CreatureState` fields to account for the new `cached_complexity` field.

- [x] Step 2: Cache functional complexity on CreatureState at birth

### Step 3: Migrate ALL action cost call sites to cached complexity

**Files:** `v3/crates/v3-core/src/simulation/tick.rs`, `v3/crates/v3-core/src/simulation/actions/mod.rs`, `v3/crates/v3-core/src/simulation/actions/predation.rs`, `v3/crates/v3-core/src/simulation/actions/reproduction.rs`

Migrate every production call site per the Call Site Audit table above:

1. `actions/mod.rs` lines 18, 37, 70: `creature.genome.complexity()` → `creature.cached_complexity`
2. `actions/mod.rs` lines 188, 213: `sim.creatures[id].genome.complexity()` → `sim.creatures[id].cached_complexity`
3. `actions/reproduction.rs` line 139: `sim.creatures[parent_id].genome.complexity()` → `sim.creatures[parent_id].cached_complexity`
4. `actions/predation.rs` lines 64, 112: steal cost and kill bonus → `creature.cached_complexity`. Note: kill bonus now scales with victim's functional complexity instead of total genome size — this is an intentional behavioral change (less reward for killing junk-heavy creatures).
5. `tick.rs` lines 346, 374, 419, 470: failed action penalties → `creature.cached_complexity`
6. Update all test call sites in these files per the audit table. **Note:** predation test at line 340 has a pre-existing bug — it computes expected kill bonus using attacker's complexity instead of victim's complexity. Fix this during migration by using `sim.creatures[victim_id].cached_complexity`.
7. Run viability tests (`cargo test -p v3-core --test viability`) to verify economic balance. Kill bonus economics are changing (functional complexity is lower than total for junk-heavy creatures), so viability tests are critical here.
8. **Threshold check**: Verify that the complexity cost threshold (default 50) still produces reasonable pressure. Functional complexity will be lower than total complexity for creatures with junk DNA, meaning fewer creatures exceed the threshold. If viability tests pass, the economics are acceptable.

- [x] Step 3: Migrate all action cost call sites to cached functional complexity

### Step 4: Rename `complexity_cap` → `genome_size_cap` and update mutation pressure

**Files:** `v3/crates/v3-core/src/config/simulation.rs`, `v3/crates/v3-core/src/mutation/engine/mod.rs`, `v3/crates/v3-core/src/mutation/pressure.rs`

1. Rename `MutationConfig::complexity_cap` → `genome_size_cap`. Add `#[serde(alias = "complexity_cap")]` for backward compatibility with serialized configs (`MutationConfig` uses `deny_unknown_fields`).
2. Rename `MutationConfig::complexity_pressure_enabled` → `genome_size_pressure_enabled`. Add `#[serde(alias = "complexity_pressure_enabled")]`.
3. Update `pressure::is_restricted()` call in `engine/mod.rs` to use `genome.genome_size()` instead of `genome.complexity()`.
4. Update all config tests that reference these fields.
5. Update config normalization comments.

- [x] Step 4: Rename config fields and update mutation pressure gate

### Step 5: Update server API and population stats

**Files:** `v3/crates/v3-server/src/handlers/creature.rs`, `v3/crates/v3-server/src/handlers/lifecycle.rs`, `v3/crates/v3-server/src/state.rs`, `v3/crates/v3-server/src/transport/view_assembler.rs`

1. Creature API response (creature.rs line 89): emit `"complexity"` from `creature.cached_complexity` and add `"genome_size"` from `creature.genome.genome_size()`.
2. Population stats (lifecycle.rs line 240): switch from `creature.genome.complexity()` to `creature.cached_complexity`. The `genome_complexity_*` field names in `HealthPayload` stay unchanged — they now report functional complexity. This semantic change is documented in the reference spec update (Step 7).
3. `view_assembler.rs` lines 236-238: update zero-default construction if `HealthPayload` fields change.
4. Add doc comment to `ComplexityEnergyCostConfig` noting it operates on functional complexity (not genome size).
5. Update server tests.

- [x] Step 5: Update server API response and population stats computation

- [ ] Review Gate: Interim code review — review Steps 1-5 changes. Fix findings, re-review until clean.

### Step 6: Update frontend

**Production files:**
- `frontend/src/types/genome.ts` — add `genome_size: number` to `CreatureDetail`
- `frontend/src/types/config.ts` — rename `complexity_cap` → `genome_size_cap`, `complexity_pressure_enabled` → `genome_size_pressure_enabled`
- `frontend/src/components/config-panel/runtime/MutationSection.tsx` — update field paths and labels

**Test fixtures:**
- `frontend/src/test/fixtures.ts` — update config fixture field names
- `frontend/src/hooks/useViewSubscription.test.ts` — update fixture
- `frontend/src/stores/worldView.test.ts` — update fixture
- `frontend/src/stores/creatureInspector.test.ts` — update fixture
- `frontend/src/components/ControlBar.test.tsx` — update fixture

**No display changes needed:** InspectorHeader and EvolutionTab read `complexity` which retains its field name. The semantic change (now functional complexity) is transparent.

- [x] Step 6: Update frontend types, config panels, and test fixtures

### Step 7: Update reference docs

**Files:** `docs/reference/v3-runtime-config-spec.md`, `docs/reference/v3-mutation-spec.md`, `docs/reference/v3-evolution-observability-spec.md`

1. `v3-runtime-config-spec.md`: Update `complexity_cap` → `genome_size_cap` in the config table; update complexity cost semantics to note it uses functional complexity (reachability-aware).
2. `v3-mutation-spec.md`: Note that pressure gate uses `genome_size()`, not `complexity()`.
3. `v3-evolution-observability-spec.md`: Note that `genome_complexity_*` stats reflect functional complexity.

- [x] Step 7: Update reference specification docs

- [ ] Review Gate: Code review — dispatch `superpowers:code-reviewer` subagent on full branch diff. Invoke domain skills (backend: `rust-skills`; frontend: `vercel-react-best-practices` + `vercel-composition-patterns`). Fix all findings. Re-review until clean pass.
- [ ] Review Gate: Architecture & decomposition review — review all changes for boundary violations, decomposition opportunities, separation of concerns. Re-read `docs/strategy/` and relevant `AGENTS.md` files. Fix easy issues, capture larger items in `docs/features/brainstorms/ideas.md`. Repeat until clean pass.
- [ ] Completion gate — run all checks from AGENTS.md Completion Gate section

**Review cycles:** 3
