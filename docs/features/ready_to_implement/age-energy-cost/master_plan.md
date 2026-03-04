# Age Energy Cost

**Goal:** Add age-based energy cost multiplier to all creature action costs, creating evolutionary pressure for generational turnover. Includes a targeted helper refactor to centralize multiplier composition.

**Goal IDs:** GP-01, GP-03

**Scope:**
- In: `EnergyConfig::action_cost_multiplier()` helper refactor, `AgeEnergyCostConfig` struct, quadratic multiplier computation, action cost adjustment at all sites (noop, eat, move, reproduce, steal, failed_action_penalty), unit/integration tests, config spec update, normalization
- Out: Hard death at age cap, age-based mutation changes, visual indicators, creature inspector display (follow-up), passive decay adjustment

**Docs Impact:** `docs/reference/v3-runtime-config-spec.md` (new config fields in Section 4)

**Supersedes:** none

**Superseded-By:** none

---

## Goal Alignment

| Goal ID | Work Items |
|---------|-----------|
| GP-01 | Age-based energy cost multiplier creates evolutionary pressure for generational turnover — creatures must balance longevity against reproduction timing, favoring evolution of optimal reproductive strategies |
| GP-03 | TDD approach with unit tests for multiplier math and curve shape, integration tests for adjusted costs across all action types, and viability gate ensures default parameters support population sustainability |

---

## Boundary Impact

No crate or module boundary changes. All modifications within `v3-core`:

- `config/simulation.rs` — new `AgeEnergyCostConfig` struct added to existing `EnergyConfig`; new `action_cost_multiplier()` helper method on `EnergyConfig`
- `simulation/actions/mod.rs` — existing per-site complexity multiplier calls replaced with unified `action_cost_multiplier()` helper
- `simulation/actions/reproduction.rs` — same replacement
- `simulation/actions/predation.rs` — same replacement
- `simulation/tick.rs` — same replacement at 4 `failed_action_penalty` sites

No new modules, crates, or dependency direction changes.

---

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `v3-core/src/config/` | keep | Config struct ownership unchanged; new sub-struct follows existing nesting pattern (`EnergyConfig > AgeEnergyCostConfig`), paralleling `ComplexityEnergyCostConfig`. Helper method lives on `EnergyConfig` alongside the config fields it reads. |
| `v3-core/src/simulation/actions/` | keep | Action functions already own cost deduction; replacing inline multiplier calls with helper is a simplification with no boundary impact |
| `v3-core/src/creature/state.rs` | keep | Reuses existing `CreatureState::age: u64` field — no creature module changes needed |

---

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| How should age and complexity multipliers compose? | Multiplicatively: `base_cost * complexity_mult * age_mult`. Independent pressures compose naturally. | Agent | resolved |
| What quadratic formula for the ramp? | `1.0 + (max_multiplier - 1.0) * min(1.0, age/age_cap)^2`. Returns 1.0 at age=0, max_multiplier at age_cap, clamped beyond. | Agent | resolved |
| Should age_cap use u64 to match creature.age type? | Yes — u64 for type consistency with `CreatureState::age` | Agent | resolved |
| Should the helper take `&CreatureState` or primitive params? | Primitive params `(complexity: u32, age: u64)` — avoids coupling config to creature module, keeps the method testable without constructing creature state | Agent | resolved |
| Where does the helper live? | On `EnergyConfig` as `action_cost_multiplier(&self, complexity: u32, age: u64) -> f32` — it reads from sub-configs it already owns | Agent | resolved |
| Is the call-site refactor behavior-preserving? | For single-action tests with age=0 creatures: yes (age_mult=1.0). For multi-tick viability tests: no — aging creatures will now pay age costs. This is the intended production behavior; viability tests must pass with production defaults per AGENTS.md policy. | Agent | resolved |

---

## Design

### Helper Refactor

New method on `EnergyConfig` centralizes all action cost multiplier composition:

```rust
impl EnergyConfig {
    /// Combined multiplier for all action energy costs.
    ///
    /// Composes complexity-based and age-based multipliers multiplicatively.
    #[inline]
    #[must_use]
    pub fn action_cost_multiplier(&self, complexity: u32, age: u64) -> f32 {
        self.complexity_cost.multiplier(complexity)
            * self.age_cost.multiplier(age)
    }
}
```

All existing `config.energy.complexity_cost.multiplier(creature.genome.complexity())` call sites are replaced with `config.energy.action_cost_multiplier(creature.genome.complexity(), creature.age)`.

### Age Cost Config

New `AgeEnergyCostConfig` added to `EnergyConfig`:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgeEnergyCostConfig {
    /// Whether the age energy cost multiplier is active.
    pub enabled: bool,
    /// Age (in ticks) at which the maximum multiplier applies.
    pub age_cap: u64,
    /// Maximum energy cost multiplier at or beyond age_cap.
    pub max_multiplier: f32,
}

impl Default for AgeEnergyCostConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            age_cap: 500,
            max_multiplier: 10.0,
        }
    }
}
```

### Multiplier Function

```rust
impl AgeEnergyCostConfig {
    /// Returns the energy cost multiplier for a creature of the given age.
    ///
    /// Formula: `1.0 + (max_multiplier - 1.0) * min(1.0, age / age_cap)^2`
    /// Returns 1.0 (no penalty) when disabled or age_cap is 0.
    #[inline]
    #[must_use]
    pub fn multiplier(&self, age: u64) -> f32 {
        if !self.enabled || self.age_cap == 0 {
            return 1.0;
        }
        let ratio = (age as f32 / self.age_cap as f32).min(1.0);
        1.0 + (self.max_multiplier - 1.0) * ratio * ratio
    }
}
```

### Multiplier Curve (at default settings)

| Age | Ratio | Multiplier | Effect |
|-----|-------|------------|--------|
| 0 | 0.0 | 1.0x | No penalty |
| 100 | 0.2 | 1.36x | +36% cost |
| 250 | 0.5 | 3.25x | +225% cost |
| 400 | 0.8 | 6.76x | +576% cost |
| 500 (cap) | 1.0 | 10.0x | +900% cost |
| 600+ | 1.0 | 10.0x | Clamped at cap |

### Application Pattern

At each action cost deduction site, the helper replaces inline multiplier logic:

```rust
let mult = config.energy.action_cost_multiplier(creature.genome.complexity(), creature.age);
creature.energy -= config.energy.costs.move_cost * mult;
```

---

## Required Skills

- Rust/backend changes: invoke `rust-skills` BEFORE writing any Rust code and before each review

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
1. Run a thorough code review (backend: `rust-skills`)
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

- [ ] Step 1: Add `AgeEnergyCostConfig` struct to `config/simulation.rs` with `Default`, serde derives, `#[serde(deny_unknown_fields)]`, and add `age_cost: AgeEnergyCostConfig` field (with `#[serde(default)]`) to `EnergyConfig`. Add normalization for `max_multiplier` (finite, >= 1.0, fallback 10.0) in `SimulationConfig::normalize()`. Write config default tests and normalization tests.
- [ ] Step 2: Add `multiplier(&self, age: u64) -> f32` method on `AgeEnergyCostConfig` with `#[inline]` and `#[must_use]`. Write TDD unit tests: age=0 returns 1.0, mid-range returns correct quadratic value, at age_cap returns max_multiplier, above age_cap returns max_multiplier (clamped), disabled returns 1.0, age_cap=0 returns 1.0.
- [ ] Step 3: Add `action_cost_multiplier(&self, complexity: u32, age: u64) -> f32` helper on `EnergyConfig` with `#[inline]` and `#[must_use]`. Write TDD unit tests: composes complexity and age multipliers multiplicatively, returns 1.0 when both are at baseline.
- [ ] Step 4: Replace all existing inline complexity-multiplier call sites with `action_cost_multiplier()` helper. Sites: `apply_noop`, `apply_eat`, `apply_move` (actions/mod.rs), `apply_reproduce` (reproduction.rs), `apply_steal_energy` (predation.rs), 4x `failed_action_penalty` (tick.rs). Note: this simultaneously activates age cost for aging creatures. Existing single-action tests use age=0 creatures and remain unaffected (age_mult=1.0). Multi-tick tests (viability) will see age effects — this is the intended production behavior.
- [ ] Step 5: Write integration tests that verify each action type deducts age-adjusted costs (create creature with age=400 vs age=0, assert different energy deductions for the same action). Cover: noop, eat, move, reproduce, steal_energy.
- [ ] Step 6: Verify all tests pass — unit tests, integration tests, and `cargo test -p v3-core --test viability` (merge gate). If viability fails, adjust `age_cap`/`max_multiplier` defaults until population sustains — do not weaken viability assertions.
- [ ] Step 7: Update `docs/reference/v3-runtime-config-spec.md` Section 4 with new `energy.age_cost.*` fields, defaults, normalization rules, and `action_cost_multiplier()` composition description.
- [ ] Step 8: Run full completion gate: `cargo fmt --all -- --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/check-plan-harness.sh --mode strict`.

---

**Review cycles:** 3
