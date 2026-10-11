# T12.F06 — Fertility Channels

**Status**: In Progress
**Last updated**: 2026-10-10
**Feature**: T12.F06
**Track**: [T12 — World Composition and Baseline Worlds](../../roadmaps/t12-world-composition-and-baseline-worlds.md)

## Goal

A recipe can lay narrow, wavy channels of fertile ground across a world from
ridged fBm noise, a few cells wide with the width set by the recipe. A
fertility layer can also blend by maximum instead of by weighted average, so
a channel adds fertility where it runs without diluting the patches beneath
it. Natural analog: riparian strips along drainage lines. A channel kept at
low fertility refills slowly after grazing, because regrowth scales with
fertility and a thin strip has fewer fertile neighbours to spread from.

## Non-Goals

- No change to production defaults, any existing recipe, or any baseline world.
- No per-layer recovery or regrowth mechanism; slow recovery comes only from
  the channel's low fertility.
- No new terrain pattern, and no change to `Uniform`, `Fbm` or `PoissonBlobs`.
- No new baseline world or recipe in this feature.

## Inputs and Invariants

- Source of truth: the T12.F06 row and its "Fertility channels (T12.F06 ...)"
  note in the [T12 track](../../roadmaps/t12-world-composition-and-baseline-worlds.md).
- Code: `FertilityAlgorithm` and `FertilityLayer` in
  `crates/v3-core/src/config/simulation.rs`; `generate_fertility`,
  `generate_layer` and `mix_layers` in `crates/v3-core/src/kernel/fertility/`;
  per-type layer selection and seed offset in `seed_fertility_maps`
  (`crates/v3-core/src/kernel/ordinary_food/ecology.rs`).
- Today `mix_layers` returns `sum(weight * value) / total_weight`, clamped to
  [-1, 1], over every layer that targets a food type, so a mostly barren
  channel layer pulls blob centres down (weight 1.0 beside two blob layers:
  about 1.0 to about 0.67).
- New algorithm `FertilityAlgorithm::Channels` with fBm shape parameters
  (`octaves`, `frequency`, `lacunarity`, `persistence`), a `width` in cells, a
  raw in-channel `value` in [-1, 1], and `seed: Option<u64>`. The channels
  follow the ridges of ridged fBm (the zero contour of the underlying fBm), so
  `frequency` sets their spacing and the octaves their waviness. Cells on a
  channel take `value`; every other cell is -1, as outside a Poisson blob. The
  implementer chooses how `width` maps to the ridge cut-off, provided the
  measured width follows the parameter (Success Criteria).
- Seeding matches the existing layers: the effective seed is
  `seed.unwrap_or(world_seed.wrapping_add(layer_index))`, with `world_seed`
  already offset per food type by `seed_fertility_maps`. Same config and seed
  give a bit-identical grid in any process.
- New per-layer field `blend` (enum `WeightedAverage` | `Max`), default
  `WeightedAverage`, `#[serde(default)]` and omitted from serialization when it
  is the default. A config with no `Max` layer therefore serializes byte for
  byte as today, so `config_digest`, recipe digests and every existing
  fertility grid stay unchanged.
- Blend semantics: the `WeightedAverage` layers mix exactly as `mix_layers`
  does today (including its zero-total-weight case); each `Max` layer then
  raises a cell to `max(cell, layer value)`. A `Max` layer's weight takes no
  part. With no `WeightedAverage` layer the base is -1. The result is
  independent of layer order and stays in [-1, 1].
- Invalid `Channels` parameters follow the existing normalize style
  (`normalize_food_shared`, `PatternParams::normalize`): a non-finite or
  below-1 `width`, a non-finite `value`, or out-of-range `octaves` fall back
  or clamp in `SimulationConfig::normalize` rather than failing; an unknown
  algorithm or blend name is rejected by serde, as today.
- Serde-facing types elsewhere carry the new fields: the frontend
  `FertilityAlgorithm` and `FertilityLayer` types in
  `frontend/src/types/config.ts`, the startup request built in
  `frontend/src/stores/startupConfig.ts` (which today rebuilds each layer from
  `algorithm`, `weight` and `target` only and would drop `blend`), and
  `FertilitySection.tsx` (whose `algorithmType` today reads any non-`Uniform`,
  non-`Fbm` layer as `PoissonBlobs`). The server and `v3-cli inspect` take the
  config through serde with no algorithm enumeration; struct literals of
  `FertilityLayer` there and in tests gain the field.
- Generation runs once at startup and on layer changes, never per tick.

## Telemetry

Not applicable: the feature only changes initialization-time fertility
generation; the new config fields reach the `run.started` config body with no
edit and govern no traced phase.

## Implementation Tasks

- [ ] Add `FertilityAlgorithm::Channels`, the `FertilityBlend` enum and the
      `blend` layer field, with normalization of the `Channels` parameters.
- [ ] Generate the channel layer (new module beside `fbm.rs`) and wire it into
      `generate_layer` with the shared seeding rule.
- [ ] Apply `Max` layers after the weighted average in fertility mixing.
- [ ] Carry the new algorithm and `blend` through the frontend types, the
      startup request and `FertilitySection.tsx` (algorithm option, width and
      value fields, blend choice).
- [ ] Tests for each Success Criterion; a proptest for the blend invariants.

## Verification

- [ ] `cargo check --workspace --all-targets` -> pass.
- [ ] `cargo fmt --all --check` -> pass.
- [ ] New and existing fertility and config tests in v3-core
      (`cargo test -p v3-core fertility`, `cargo test -p v3-core config`) -> pass.
- [ ] Tests of every other crate the diff touches (for example
      `cargo test -p v3-server`, `cargo test -p v3-cli`) -> pass.
- [ ] `make rust-clippy` -> pass.
- [ ] Frontend type check and tests, only if the frontend changes -> pass.
- [ ] `make check-docs` -> pass.
- [ ] Mutation gate: `Not applicable: user decision 2026-10-10, minimal-overhead run`.
- [ ] Benchmark summary: `Not applicable: no default or existing recipe changes, user decision 2026-10-10`.

## Performance and Goal Impact

Not applicable: no default or existing recipe changes, user decision 2026-10-10

## Success Criteria

- [ ] Default blend reproduces today's mixing exactly: for layers without
      `blend`, the mixed grid is bit-identical to today's weighted average, and
      a config with no `Max` layer serializes with no `blend` key.
- [ ] A `Max` layer never lowers a cell: every cell, a blob centre included,
      is at least its value without that layer (proptest over layer values and
      weights), and the result does not depend on layer order.
- [ ] The channel layer is narrow and its width follows the parameter: on a
      fixed seed and frequency, channel cells are a minority of the grid at
      width 3, and the channel-cell fraction rises strictly with width and
      roughly doubles from width 2 to width 4 (factor in [1.5, 2.5]).
- [ ] The same config and seed give a bit-identical channel grid; a different
      seed gives a different one; an omitted seed follows the shared rule.
- [ ] A recipe with a `Channels` layer and `"blend": "Max"` round-trips
      through `resolve_config` and serialization unchanged, and through the
      frontend startup request with both fields intact.
- [ ] A `width` of 0, a negative or non-finite `width`, and a non-finite
      `value` normalize to valid values; an unknown blend name is rejected.

## Notes for AI Agents

- Decision: Minimal-overhead run (user, 2026-10-10): no Codex rounds, no gate or goal profile, no mutation gate, targeted test subset only.
