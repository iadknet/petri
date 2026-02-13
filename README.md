# Petri: Stage 1a Walking Skeleton

This repository now contains the first runnable vertical slice of the Petri simulation:

- `petri-core`: world state and simulation tick loop
- `petri-server`: REST + WebSocket streaming server
- `petri-cli`: headless runner with periodic stats output
- `web/`: React canvas client consuming MessagePack frames

## Project docs

- `docs/README.md`: canonical docs index
- `docs/strategy/roadmap.md`: implementation roadmap and stage definitions
- `docs/strategy/architecture.md`: architecture and boundary guidance
- `docs/strategy/technology-review.md`: technology posture and tradeoffs
- `docs/strategy/goals.md`: canonical high-level goals catalog with stable IDs
- `docs/reference/creature-controller-reference.md`: creature/controller structure, node semantics, and mutation operator reference

Compatibility note: `petri-roadmap.md`, `petri-architecture.md`, and `petri-technology-review.md` remain as root stubs that point to the canonical `docs/strategy/*` files.

## Cognition-first tick lifecycle (implemented)

Current semantics:
- creatures execute an energy-bounded internal think loop inside each tick
- controller supports explicit `halt` and `no_op` outputs
- at most one world interaction executes per creature per tick
- final action arbitration uses final-thought outputs (with explicit no-op competition)
- cognition energy tuning uses `energy_per_think_step`
- inspector and snapshot payloads include cognition diagnostics (`think_steps`, `halted`, `selected_action`, `selected_confidence`)

## Prerequisites

- `rustup` (recommended) with Rust `1.93.0`
- `volta` with Node `25.6.0` and npm `11.8.0`

## Toolchain setup

```bash
rustup toolchain install 1.93.0
rustup component add rustfmt clippy --toolchain 1.93.0
volta install node@25.6.0 npm@11.8.0
```

The Rust toolchain is pinned via `rust-toolchain.toml`.  
The web toolchain is pinned in `web/package.json` under the `volta` field.

## Run the server

```bash
cargo run -p petri-server
```

Default address: `http://127.0.0.1:4000`

Stage 1e lifecycle note:
- Server boots in `idle` and does not tick until explicitly started.
- Use `POST /simulation/start` (or the web Start button) to begin a run.
- Use `POST /simulation/restart` to rebuild from current startup draft with a new seed.

## Run the web UI

```bash
cd web
npm install
npm run dev
```

Open `http://127.0.0.1:5173`.

The app expects the backend on `127.0.0.1:4000`.

The web control rail now exposes:
- Startup draft: initial creatures, world width/height, sensor radius, initial food density, food spawn/growth, spread threshold, spawn floor density, tick decay, think-step cost, move cost, world wrap
- Runtime controls: pause/resume, ticks-per-second, live sensor radius and food spawn/growth plus spread threshold and spawn floor density
- Advanced Stage 1 controls for energy and mutation tuning
- Viewport paint mode toggle with floating tools (food, barrier, erase food, erase barrier), brush sizes, idle preview mode, and clear paint action (idle/paused only)
- Creature inspector (click creature in viewport), including cognition diagnostics and expanded controller I/O fields
- Live population/average-energy chart
- Snapshot export/import panel (`GET/POST /simulation/snapshot`)
- Idle placeholder before first start, plus pending-restart state when startup-only values change during a run

Creatures now render with heritable phenotype colors (hue drifts on mutation, saturation drifts each generation), and the inspector surfaces the same phenotype color value for the selected creature.

Default simulation world size is `400x400` and can be changed in Startup Draft before starting (or before restarting) a run.

## Run the CLI

```bash
cargo run -p petri-cli -- run --ticks 1000 --sample-every 25
```

Run Stage 1b controller stability ablation:

```bash
cargo run -p petri-cli --bin petri-cli -- ablation --ticks 500
```

Run Stage 1 throughput benchmark (200x200, 5k creatures):

```bash
cargo run -p petri-cli --bin stage1_benchmark -- --ticks 200 --assert-min --min-ticks-per-second 30
```

Stage 1c note: worlds are seeded from a viable founder controller and lightly mutated on spawn/reproduction.
The `ablation` command uses a tuned deterministic config (`founder_survival_config`) to keep populations alive long enough for behavior comparison.

## Verify

```bash
cargo test --workspace
cd web && npm run build
```
