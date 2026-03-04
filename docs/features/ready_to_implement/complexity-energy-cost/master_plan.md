# Complexity Energy Cost — Runtime Config UI Controls

**Goal:** Expose the existing `ComplexityEnergyCostConfig` (enabled, threshold, scaling_factor) in the frontend runtime config panel so users can tune complexity pressure at runtime.

**Goal IDs:** GP-01, GP-04

**Scope:**
- In: `ComplexityEnergyCostConfig` TypeScript interface, runtime config panel field definitions (enabled toggle, threshold slider, scaling_factor slider), test fixture updates
- Out: Backend changes (already implemented), creature inspector penalty display (follow-up), curve shape configurability (deferred per refinement doc)

**Docs Impact:** none (no spec changes — backend config already documented in `docs/reference/v3-runtime-config-spec.md`)

**Supersedes:** none

**Superseded-By:** none

---

## Goal Alignment

| Goal ID | Work Items |
|---------|-----------|
| GP-01 | Runtime-tunable complexity pressure lets users adjust evolutionary selection pressure during simulation, enabling experimentation with different complexity cost profiles |
| GP-04 | Exposing complexity cost parameters in the config panel makes the existing complexity pressure mechanism observable and controllable — users can see and adjust threshold/scaling values that were previously hidden in code defaults |

---

## Boundary Impact

No crate or module boundary changes. All modifications within `frontend/src/`:

- `types/config.ts` — new `ComplexityEnergyCostConfig` interface added to existing `EnergyConfig`
- `components/config-panel/runtime/` — new section file with field definitions
- `components/config-panel/runtime/RuntimeConfigPanel.tsx` — register new fields
- Test fixtures — add `complexity_cost` to mock configs

No backend changes needed — `ComplexityEnergyCostConfig` is already serialized via serde with `#[serde(default)]` and accepted by `PATCH /config`.

---

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `frontend/src/types/config.ts` | change | Add `ComplexityEnergyCostConfig` interface and `complexity_cost` field to `EnergyConfig` to match the Rust struct already serialized by the server |
| `frontend/src/components/config-panel/runtime/` | change | New section file following existing `FieldDef`/`BooleanFieldDef` + `RuntimeFieldGroup` pattern — consistent with `MutationSection.tsx` mixed-field approach |
| `v3/crates/v3-core/src/config/simulation.rs` | keep | Backend `ComplexityEnergyCostConfig` already implemented with `enabled: true`, `threshold: 50`, `scaling_factor: 0.002` defaults |
| `v3/crates/v3-server/src/handlers/status.rs` | keep | Server already serializes full `SimulationConfig` including `complexity_cost` and accepts patches via deep merge |

---

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should complexity cost fields go in `EnergyCostsSection.tsx` or a new file? | New `ComplexityEnergyCostSection.tsx` — `complexity_cost` is a distinct sub-config (`energy.complexity_cost`) with its own enabled toggle, not part of `energy.costs` | Agent | resolved |
| What slider ranges for threshold and scaling_factor? | threshold: 0–500 step 10; scaling_factor: 0.0–0.02 step 0.001 — covers the useful range around defaults (50, 0.002) | Agent | resolved |
| Does the backend need any changes? | No — `ComplexityEnergyCostConfig` already exists, is serialized, accepts patches, has normalization | Agent | resolved |

---

## Required Skills

- Frontend changes: invoke `vercel-react-best-practices` and `vercel-composition-patterns` BEFORE writing any frontend code and before each review

---

## TDD Policy

For all behavior changes and bug fixes: write a failing test FIRST, then implement.
A step is not complete until:
1. The failing test exists and is committed
2. The implementation makes it pass
3. No existing tests regress

---

## Code Review Policy

After completing each implementation step:
1. Run a thorough code review (frontend: vercel skills)
2. Fix ALL findings
3. Run review AGAIN — repeat until no new findings (clean recursive pass)
4. Only after clean pass: commit the step

---

## Commit Policy

- Commits happen AFTER a clean code review pass, never before
- One commit per implementation step (focused, atomic)
- Do NOT advance to the next step until current step is committed and reviewed clean

---

## Implementation Steps

- [ ] Step 1: Add `ComplexityEnergyCostConfig` interface to `frontend/src/types/config.ts` with fields `enabled: boolean`, `threshold: number`, `scaling_factor: number`. Add `complexity_cost: ComplexityEnergyCostConfig` to the existing `EnergyConfig` interface. Update all mock/test config fixtures that construct `EnergyConfig` objects (in `ConfigPanel.test.tsx`, `config.test.ts`, `startupConfig.test.ts`, and any other test files) to include `complexity_cost: { enabled: true, threshold: 50, scaling_factor: 0.002 }`.
- [ ] Step 2: Create `frontend/src/components/config-panel/runtime/ComplexityEnergyCostSection.tsx` following the `MutationSection.tsx` mixed-field pattern. Define: `TOGGLES: BooleanFieldDef[]` with `energy.complexity_cost.enabled`, `FIELDS: FieldDef[]` with `energy.complexity_cost.threshold` (min 0, max 500, step 10, default 50) and `energy.complexity_cost.scaling_factor` (min 0, max 0.02, step 0.001, default 0.002). Export `ALL_FIELDS` union and a `ComplexityEnergyCostFieldGroup` component.
- [ ] Step 3: Register the new section in `RuntimeConfigPanel.tsx` — import `ComplexityEnergyCostSection`, spread its `ALL_FIELDS` into `RUNTIME_PATCH_FIELDS`, and render `<ComplexityEnergyCostFieldGroup>` in the panel JSX within the energy config area.
- [ ] Step 4: Verify all tests pass: `cd frontend && npm test`, `cd v3 && cargo test --workspace` (backend unchanged, sanity check). Fix any test fixture issues.
- [ ] Step 5: Run full completion gate: `scripts/check-plan-harness.sh --mode strict`, `cd v3 && cargo fmt --all -- --check`, `cd v3 && cargo test --workspace`, `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`.

---

**Review cycles:** 1
