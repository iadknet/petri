# Review Log: reachability-based-complexity

## Pass 1: Domain Skills Review — Dispatch 1

1. [HIGH] O(n^2) in mesh_reachable_nodes — functional_complexity() will inherit quadratic cost. Build HashMap index for O(1) lookups.
2. [HIGH] Plan misses ~20+ creature.genome.complexity() call sites (actions/mod.rs lines 18, 37, 70, 188, 213 and more). Complete call site audit needed.
3. [HIGH] functional_complexity() computed on hot path without caching between Step 1 and Step 2. Steps must be committed together.
4. [MEDIUM] cached_complexity as bare u32 has no staleness protection. Make pub(crate) with accessor.
5. [MEDIUM] Plan lists StoreMem8/StoreMemF32 but StoreMemF32 doesn't exist; StoreMem8Imm is missing. Use existing vm_is_output_instruction().
6. [MEDIUM] CreatureState::new() already has too many args. Compute cached_complexity inside new() from genome.
7. [MEDIUM] Wire format breaking change with deny_unknown_fields. Need serde aliases or deployment constraint.
8. [MEDIUM] input_refs counting logic underspecified. Clarify VM ReadInput.ref_idx and Graph GraphNodeKind::InputRef algorithm.
9. [LOW] Missing with_capacity for collections in functional_complexity().
10. [LOW] Step 6 mixes production and test files without grouping.
11. [LOW] ComplexityEnergyCostConfig threshold (50) tuned against total complexity; functional complexity will be lower.

**Total: 11 findings**

## Pass 1: Domain Skills Review — Dispatch 2

1. [MEDIUM] input_refs algorithm uses incorrect field/type names: ReadInput.input_idx → ReadInput.ref_idx; InputSlot(idx) → GraphNodeKind::InputRef { ref_idx, .. }.
2. [MEDIUM] Offspring construction ordering: CreatureState::new() must receive post-mutation genome.
3. [LOW] Kill bonus uses victim complexity — semantic change not explicitly called out.

**Total: 3 findings**

## Pass 1: Domain Skills Review — Dispatch 3 (implied clean after fixes)

All 14 findings from dispatches 1-2 addressed in plan revision.

## Pass 2: Decomposition & Codebase Consistency Review — Dispatch 1

1. [HIGH] deny_unknown_fields on MutationConfig breaks deserialization with old field names. Need #[serde(alias)].
2. [HIGH] pub(crate) visibility breaks v3-server access. All CreatureState fields are pub. Match convention.
3. [MEDIUM] view_assembler.rs missing from Step 5 scope.
4. [MEDIUM] Population stats semantic change (genome_complexity_* now means functional) not documented.
5. [MEDIUM] ComplexityEnergyCostConfig parameter name "complexity" becomes ambiguous. Add doc comment.
6. [MEDIUM] Kill bonus economic behavior change not noted in viability check.
7. [MEDIUM] Predation test line 340 uses attacker complexity for kill bonus but production uses victim. Pre-existing bug.
8. [LOW] Duplicate work: functional_complexity() calls mesh_reachable_nodes() which does O(n) lookups, then builds its own HashMap.
9. [LOW] Confirmed: new() computation approach is correct.
10. [LOW] Frontend Step 6 file list adequate.

**Total: 10 findings**

## Pass 2: Decomposition & Codebase Consistency Review — Dispatch 2 (implied clean after fixes)

All 10 findings addressed in plan revision.

## Pass 3: Architecture, Boundary & Goal Alignment Review — Dispatch 1

1. [LOW] HealthSnapshot → HealthPayload naming inconsistency in Boundary Impact.
2. [LOW] CreatureState::new() call sites in sensors/runtime not in audit table; clarify that internal computation means no code changes needed.

**Total: 2 findings**

## Pass 3: Architecture, Boundary & Goal Alignment Review — Dispatch 2 (implied clean after fixes)

Both LOW findings fixed. Plan is architecturally sound:
- All crate/module boundaries preserved
- Dependency directions unchanged
- Goal IDs GP-01, GP-03, GP-04 accurately mapped
- No goals undermined
- Viability tests gate at Step 3
- Review gates present as checkmark items
- Runtime truthfulness invariant satisfied (cached_complexity derived from actual genome analysis)
- serde(alias) backward compat approach valid
