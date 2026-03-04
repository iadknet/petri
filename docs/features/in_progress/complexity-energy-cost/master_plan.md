# Complexity Energy Cost

**Goal:** Add a genome-complexity-based energy cost multiplier to all creature actions, creating gradient evolutionary pressure toward efficient genomes.

**Goal IDs:** GP-01, GP-03

**Scope:**
- In: Config struct, multiplier computation, action cost adjustment (all 6 action types + failed_action_penalty), unit/integration tests, spec update
- Out: Server/inspector observability (follow-up), visual indicators, complexity cap changes, reproduction transfer adjustment

**Docs Impact:** `docs/reference/v3-runtime-config-spec.md` (new config fields in Section 4)

**Supersedes:** none

**Superseded-By:** none

---

## Goal Alignment

| Goal ID | Work Items |
|---------|-----------|
| GP-01 | Gradient pressure toward efficient genomes creates richer evolutionary dynamics — creatures must balance cognitive complexity against energy cost, favoring evolution of more efficient decision-making |
| GP-03 | TDD approach with unit tests for multiplier math, integration tests for adjusted costs, and viability gate ensures high-confidence iteration |

---

## Boundary Impact

No crate or module boundary changes. All modifications are within `v3-core`:

- `config/simulation.rs` — new `ComplexityEnergyCostConfig` struct added to existing `EnergyConfig`
- `simulation/actions/mod.rs` — multiplier applied at existing cost deduction sites
- `simulation/actions/reproduction.rs` — multiplier applied to `reproduce_cost`
- `simulation/actions/predation.rs` — multiplier applied to `steal_cost_rate` calculation
- `simulation/tick.rs` — multiplier applied to `failed_action_penalty` deductions

No new modules, crates, or dependency direction changes.

---

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `v3-core/src/config/` | keep | Config struct ownership unchanged; new sub-struct follows existing nesting pattern (`EnergyConfig > ComplexityEnergyCostConfig`) |
| `v3-core/src/simulation/actions/` | keep | Action functions already own cost deduction; multiplier is applied inline at existing deduction sites |
| `v3-core/src/creature/genome/` | keep | Reuses existing `CreatureGenome::complexity()` method — no genome module changes needed |
| `v3-core/src/mutation/pressure.rs` | keep | Mutation complexity pressure is independent; this feature adds energy pressure as a complementary mechanism |

---

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should energy_decay_per_tick also be multiplied? | No — decay is a world-level phase 0 cost, not an action cost. Feature scope is "action energy costs" per refinement doc. | Agent | resolved |
| Should eat_reward_per_food be reduced by complexity? | No — reward is not a cost. Reducing rewards would double-penalize complex creatures. | Agent | resolved |
| Should the multiplier apply to steal_cost_rate? | Yes — complex predators pay more for hunting, consistent with "all action costs" scope. | Agent | resolved |
| Should complexity be cached per-tick or computed inline? | Compute inline — `complexity()` is O(nodes) with typical genomes ~10-100 components; called once per creature per action (max 2 with penalty). Negligible vs mesh execution cost. | Agent | resolved |
| Default threshold value? | 50 — well above founder genomes (~10 complexity) but below where pressure should begin. | Agent | resolved |
| Default scaling_factor value? | 0.002 — at complexity 550 (500 above threshold): 2x cost; at 1050 (near cap): 3x cost. Creates meaningful gradient without instant death. | Agent | resolved |

---

## Design

### Config

New `ComplexityEnergyCostConfig` added to `EnergyConfig`:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComplexityEnergyCostConfig {
    pub enabled: bool,
    pub threshold: u32,
    pub scaling_factor: f32,
}

impl Default for ComplexityEnergyCostConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 50,
            scaling_factor: 0.002,
        }
    }
}
```

### Multiplier Function

Method on `ComplexityEnergyCostConfig`:

```rust
impl ComplexityEnergyCostConfig {
    #[inline]
    pub fn multiplier(&self, complexity: u32) -> f32 {
        if !self.enabled || complexity <= self.threshold {
            return 1.0;
        }
        1.0 + (complexity - self.threshold) as f32 * self.scaling_factor
    }
}
```

### Application Pattern

At each action cost deduction site:

```rust
let mult = config.energy.complexity_cost.multiplier(creature.genome.complexity());
creature.energy -= config.energy.costs.move_cost * mult;
```

### Multiplier Curve (at default settings)

| Complexity | Above Threshold | Multiplier | Effect |
|-----------|----------------|------------|--------|
| 10 (founder) | 0 | 1.0x | No penalty |
| 50 (threshold) | 0 | 1.0x | No penalty |
| 200 | 150 | 1.3x | +30% cost |
| 500 | 450 | 1.9x | +90% cost |
| 1000 | 950 | 2.9x | +190% cost |
| 1200 (cap) | 1150 | 3.3x | +230% cost |

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

- [x] Step 1: Add `ComplexityEnergyCostConfig` struct to `config/simulation.rs` with `Default`, serde derives, normalization in `SimulationConfig::normalize()`, and add `complexity_cost: ComplexityEnergyCostConfig` field to `EnergyConfig`. Write config default tests.
- [x] Step 2: Add `multiplier(&self, complexity: u32) -> f32` method on `ComplexityEnergyCostConfig`. Write TDD unit tests: below threshold returns 1.0, at threshold returns 1.0, above threshold returns correct value, disabled returns 1.0, threshold=0 works correctly.
- [x] Step 3: Write failing integration tests that verify each action type deducts complexity-adjusted costs (use a high-complexity genome fixture vs a low-complexity genome, assert different energy deductions for the same action). Cover: noop, eat, move, reproduce, steal_energy, failed_action_penalty.
- [ ] [parallel] Step 4a: Update `apply_noop` in `actions/mod.rs` to apply complexity multiplier.
- [ ] [parallel] Step 4b: Update `apply_eat` in `actions/mod.rs` to apply complexity multiplier to `eat_cost` (NOT to `eat_reward_per_food`).
- [ ] [parallel] Step 4c: Update `apply_move` in `actions/mod.rs` to apply complexity multiplier to `move_cost`.
- [ ] Step 4d: Update `apply_reproduce` in `actions/reproduction.rs` to apply complexity multiplier to `reproduce_cost`.
- [ ] Step 4e: Update `apply_steal_energy` in `actions/predation.rs` to apply complexity multiplier to the steal cost calculation.
- [ ] Step 4f: Update `failed_action_penalty` deductions in `tick.rs` (4 sites: eat, move, reproduce, steal_energy) to apply complexity multiplier.
- [ ] Step 5: Verify all tests pass — unit tests, integration tests, and `cargo test -p v3-core --test viability` (merge gate).
- [ ] Step 6: Update `docs/reference/v3-runtime-config-spec.md` Section 4 with new `energy.complexity_cost.*` fields, defaults, and normalization rules.
- [ ] Step 7: Run full completion gate: `cargo fmt --all -- --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/check-plan-harness.sh --mode strict`.

---

**Review cycles:** 2
