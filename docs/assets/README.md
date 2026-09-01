# Petri V3 UI capture

Both README images were captured from the same real, running Petri V3
simulation in one session. The simulation was driven through the dashboard via
the repository's browser-automation workflow; all values below are
simulation-applied state, not synthetic placeholders.

## Shared run

- **Capture UTC:** 2026-08-31T01:12:04Z
- **Source commit:** `f0eb6e5326fdf40864858e6dca581575ad866e31`
- **Runtime:** release-mode local Petri V3 application (`MODE=release ./scripts/dev.sh`)
- **World:** 1600 × 1600 cells, `edge_mode: Wrap`
- **Config:** `SimulationConfig` defaults — `initial_creatures` 10,000,
  `max_creatures` 100,000, founder profile `v3_alpha1`
- **Seed:** `195822586` (Config panel value at Restart)
- **Lifecycle:** initialized via **Restart**, run with **Start**, then paused
  and advanced with **Step** for the animation frames.

## `petri-creatures.gif` (hero animation)

- **Dimensions:** 900 × 504 px, 30 frames, ~110 ms/frame, loops forever
- **Frame cadence:** one simulation tick per frame, ticks 53 → 83
- **View:** maximum zoom (20×), centered; each frame is the center 1200 × 672
  region of the 1600 × 672 world viewport, nearest-neighbor downscaled to
  preserve the pixel-cell look
- **Content:** creatures rendered as filled cells (with per-creature energy
  bars in detailed zoom) foraging across shaded food cells
- **Image SHA-256:** `5dcd636a4f48cf602c669711f52555494a64f6723106ebdbcb0aa5c6c2bcac78`

## `petri-v3-ui.png` (dashboard)

- **Dimensions:** 1600 × 1000 px
- **Captured state:** paused at tick 84 (display shows tick 83 for the last
  applied frame) with population at the 100,000 cap; **Stats → Overview** tab
  open showing Mean Energy 24.4 and last-tick actions (Move 68,350, Eat 14,030,
  Reproduce 29,983, Noop 1,246) alongside the population and energy trends
- **View:** moderate zoom (~1.5×) over the full world viewport
- **Image SHA-256:** `3e63710001acefe560811183865945a0db1ff1bcbdb939419385a4ef922fa683`

The unannotated captures were visually inspected at their original resolution;
browser console output was empty except for a single pre-existing, unrelated
error present at page load.
