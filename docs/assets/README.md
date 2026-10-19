# README captures

Every image the top-level README shows was captured on 2026-10-19. All but
`progress.png` come from one real, running Petri simulation and the telemetry
stack it exported to; `progress.png` renders the committed benchmark
summaries. Values
are simulation-applied state, not placeholders. Captures were taken with
headless Chromium (Playwright) at 1 CSS px per pixel; crops are noted per file.

## Shared run

- **Source build:** `7e8c3b78` (the server's `petri_build_revision`)
- **Runtime:** `MODE=release ./scripts/dev.sh`, telemetry stack up
- **World:** [`experiments/worlds/confluence.json`](../../experiments/worlds/confluence.json),
  run seed 33, loaded through `POST /v3/simulation/startup`
- **Telemetry run id:** `d91887413005fdc4c5cb888b4cac6ff3`
- The run was paused several times (near ticks 3,870, 12,906 and 13,289) to
  select creatures; the dashboards show those pauses as flat stretches.

## `petri-creatures.gif` (hero animation)

- 960 × 540 px, 72 frames, 90 ms per frame, loops; 1:1 crop of the map at
  about 9.5 px per cell, in the region the snapshot API showed held the most
  creatures and hue groups
- Recorded live (not stepped), ending at tick 13,088, so creatures glide
  between ticks as they do in the UI
- Shared 96-color palette, no dithering
- SHA-256 `182e0941f520899bbd747d2180f6fdc0eb2f033fbf1b25ae65d49ece2fb5d07b`

## `readme/closeup.gif`

- 960 × 540 px, 72 frames, 80 ms per frame, loops; 1:1 crop at about
  19 px per cell, recorded live ending at tick 17,541
- SHA-256 `b490c0019d04cc7a2442943308c38293e0b9bf771dc6a9a625aa68f5e84ca539`

## `readme/` screenshots

- `viewer.png` (1600 × 1000): Full UI near tick 13,180, Evolution tab. SHA-256 `8df2311fa2c09bcea8c32fb991143a13c919f02729d540e31cea8bfc1d6dec94`
- `creature-inspector.png` (1200 × 636): Creature #1670742281981 (gen 156), phenotype and diagnostics open. SHA-256 `a9ad0d0ca30dd16c5972b9660ca6796d22de4f70f7f05d050b63f01fe688a13e`
- `node-inspector.png` (955 × 1005): Same creature's mesh and node #0. SHA-256 `aff2701d4143d2fd3a4fab85feddb605f7cba7f920b39bb7a4ee18d9d2d65642`
- `edge-tooltip.png` (955 × 432): Creature #1636382543174 (gen 223): edge groups, plastic-edge tooltip. SHA-256 `e21e76981bdbde025b7ab081b75e28091480c8644cfca198d53b853e7043e395`
- `tick-sampler.png` (955 × 724): Same creature, ten sampled ticks from tick 13,289. SHA-256 `5cdc18bedda82428bf509a62eb2a85d86be2dc4f6b16b73d444dc6c6dde6d807`
- `stats-dock.png` (1600 × 642): Mechanics above Brains at tick 14,982, joined with a 12 px gap. SHA-256 `19957f83cdf988c371387c9312db66c31bc39cc0177613d64cd08dc81d4a75de`
- `config-panel.png` (860 × 1000): Live tab with a setting tooltip open. SHA-256 `d667b736278b8c5866b7d623628504c2fdc12fa41068a86230601532cb53b228`
- `grafana-run.png` (1600 × 940): Petri / Run, Identity panel cropped off. SHA-256 `50b360f53d02cf16b22bf877e38a2bfb32dce23466c01a49abe6473cf3bee59a`
- `grafana-performance.png` (1600 × 1000): Petri / Performance, this run against `3f17b0ff`. SHA-256 `63d6b2d056dd184708c2d3314c573378669fd8305c2dae0564a5918b79daf7a9`
- `grafana-compare.png` (1600 × 935): Petri / Compare runs, this run against `3f17b0ff`. SHA-256 `c1b3b284e4867633c9d3936939cc457a3b39859b3a4fe4d554476d0925f8bf7e`
- `creature-windows.png` (1440 × 1300): Creature windows report, top of page. SHA-256 `afcd5c7c0f69399b1fab38fe317de4158866a5f57c7ce3e3a33ee37332f55877`
- `creature-windows-over-time.png` (1440 × 1160): Creature windows over time, top of page. SHA-256 `5050d6d9248dca9754c964b28c323a7587c189cd2b7f11ba7ef0be2ac892200a`
- `progress.png` (1440 × 1100): `docs/progress/index.html`, first screen. SHA-256 `e9f31e9126ddd23da2e7646b4dab1594982cd00a7e71f8c002404f004bcde7c0`
