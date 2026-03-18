# Stage 2: Server / Projection — Companion Plan

**Parent plan:** `2026-03-18-food-fertility-layer.md`
**Spec:** `docs/superpowers/specs/2026-03-18-food-fertility-layer-design.md`
**Prerequisite:** Stage 1 complete (FoodResource abstraction, fertility seeding, growth integration)

> **For agentic workers:** Use `rust-skills` for all Rust code in this stage.

---

## Task 12: Fertility Quantization

**Files:**
- Modify: `v3/crates/v3-server/src/query/cache.rs`

- [ ] **Step 1:** Write failing tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantize_fertility_maps_range_to_u8() {
        // min=0.0, max=2.0: value 0.0 → 0, value 1.0 → 128, value 2.0 → 255
        assert_eq!(quantize_fertility(0.0, 0.0, 2.0), 0);
        assert_eq!(quantize_fertility(1.0, 0.0, 2.0), 128);
        assert_eq!(quantize_fertility(2.0, 0.0, 2.0), 255);
    }

    #[test]
    fn quantize_fertility_min_equals_max_returns_128() {
        assert_eq!(quantize_fertility(1.0, 1.0, 1.0), 128);
        assert_eq!(quantize_fertility(0.0, 0.0, 0.0), 128);
    }
}
```

- [ ] **Step 2:** Run test to verify it fails
- [ ] **Step 3:** Implement

```rust
/// Quantize an effective fertility value to u8.
/// Two-step pipeline: raw [-1,1] is already mapped to [min,max] by caller.
/// Edge case: min == max → 128 (midpoint).
pub fn quantize_fertility(effective: f32, min: f32, max: f32) -> u8 {
    if (max - min).abs() < f32::EPSILON {
        return 128;
    }
    ((effective - min) / (max - min) * 255.0).round().clamp(0.0, 255.0) as u8
}

/// Build full-world quantized fertility array from FoodResource.
pub fn build_food_fertility_u8(food: &FoodResource) -> Box<[u8]> {
    let config = food.config();
    let fertility = food.fertility();
    let min = config.fertility.min_fertility;
    let max = config.fertility.max_fertility;
    let width = fertility.width() as usize;
    let height = fertility.height() as usize;
    let mut result = Vec::with_capacity(width * height);

    for y in 0..height {
        for x in 0..width {
            let raw = *fertility.get(x as u16, y as u16);
            let t = (raw + 1.0) / 2.0;
            let effective = min + t * (max - min);
            result.push(quantize_fertility(effective, min, max));
        }
    }
    result.into_boxed_slice()
}
```

- [ ] **Step 4:** Run tests — verify pass
- [ ] **Step 5:** Commit

```bash
git commit -m "feat: add fertility quantization for transport"
```

---

## Task 13: Projection and FramePayload

**Files:**
- Modify: `v3/crates/v3-server/src/state.rs`
- Modify: `v3/crates/v3-server/src/query/projection.rs`

- [ ] **Step 1:** Add `food_fertility_u8` to `FramePayload`

In `state.rs`, add to `FramePayload`:

```rust
pub food_fertility_u8: Box<[u8]>,
```

- [ ] **Step 2:** Add `food_fertility_u8` to `ProjectionSnapshot`

In `projection.rs`, add to `ProjectionSnapshot`:

```rust
pub food_fertility_u8: Box<[u8]>,
```

Update `ProjectionSnapshot::from_ws_frame()` to include:

```rust
food_fertility_u8: frame.food_fertility_u8.clone(),
```

- [ ] **Step 3:** Update `same_world_static()`

In `projection.rs`, update `same_world_static()` to include fertility mask comparison:

```rust
pub fn same_world_static(a: &FramePayload, b: &FramePayload) -> bool {
    a.width == b.width
        && a.height == b.height
        && a.barriers == b.barriers
        && a.food_fertility_u8 == b.food_fertility_u8
}
```

- [ ] **Step 4:** Update frame building in `lifecycle.rs`

In `build_ws_frame()`, add fertility quantization:

```rust
let food_fertility_u8 = build_food_fertility_u8(sim.world.food());
```

Pass to `FramePayload`.

- [ ] **Step 5:** Fix all compile errors

- [ ] **Step 6:** Write test for revision change on fertility change

```rust
#[test]
fn same_world_static_detects_fertility_change() {
    let mut a = test_frame_payload();
    let mut b = a.clone();
    b.food_fertility_u8 = vec![128; b.food_fertility_u8.len()].into_boxed_slice();
    assert!(!same_world_static(&a, &b));
}
```

- [ ] **Step 7:** Run tests — verify pass

Run: `cd v3 && cargo test -p v3-server`

- [ ] **Step 8:** Commit

```bash
git commit -m "feat: add food_fertility_u8 to projection and frame payload"
```

---

## Task 14: View Assembly and Endpoints

**Files:**
- Modify: `v3/crates/v3-server/src/transport/view_assembler.rs`
- Modify: `v3/crates/v3-server/src/transport/protocol.rs` (if needed)

- [ ] **Step 1:** Include `food_fertility_u8` in view payloads

Update `assemble_detail_payload()` and `assemble_overview_payload()` to include fertility data from the projection snapshot. For the detail view, extract the subscription rect's fertility slice. For overview, include the full grid or downsample.

- [ ] **Step 2:** Verify all transport paths include fertility

Check:
- Snapshot path (initial connection)
- WebSocket frame path (on world reset)
- HTTP endpoint (if separate)

- [ ] **Step 3:** Write endpoint tests

```rust
#[test]
fn detail_payload_includes_fertility() {
    // Build a projection with fertility data, assemble detail payload,
    // verify food_fertility_u8 is present and correctly sliced
}
```

- [ ] **Step 4:** Run tests

Run: `cd v3 && cargo test -p v3-server`

- [ ] **Step 5:** Commit

```bash
git commit -m "feat: include food_fertility_u8 in all view assembly paths"
```

---

## Task 15: Config Patch Propagation

**Files:**
- Modify: `v3/crates/v3-server/src/handlers/status.rs`

- [ ] **Step 1:** Update `patch_config()` to propagate food config changes

After the config merge and validation, add:

```rust
handle.sim.world.apply_food_config(merged_config.world.food.clone());
```

- [ ] **Step 2:** Write test

```rust
#[test]
fn patch_config_propagates_food_config() {
    // Create simulation, patch food growth_rate, verify FoodResource sees the new value
}
```

- [ ] **Step 3:** Run tests
- [ ] **Step 4:** Commit

```bash
git commit -m "feat: propagate config patches to FoodResource"
```

---

## Stage 2 Gates

- [ ] Run: `cd v3 && cargo test -p v3-server` — all pass
- [ ] Run: `cd v3 && cargo test --workspace` — all pass
- [ ] Run: `cd v3 && cargo fmt --all -- --check` — clean
- [ ] Run: `cd v3 && cargo clippy --workspace --all-targets -- -D warnings` — clean
- [ ] Review Gate: Dispatch `superpowers:code-reviewer` with `rust-skills`. Fix all findings. Re-dispatch on changed code. Repeat until zero new findings.
- [ ] Re-run all Stage 2 tests after review-introduced changes
