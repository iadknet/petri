# Petri: Stage 1a Walking Skeleton

This repository now contains the first runnable vertical slice of the Petri simulation:

- `petri-core`: world state and simulation tick loop
- `petri-server`: REST + WebSocket streaming server
- `petri-cli`: headless runner with periodic stats output
- `web/`: React canvas client consuming MessagePack frames

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
- Startup draft (Core 6): initial creatures, initial food density, food spawn/growth, tick decay, move cost
- Runtime controls: pause/resume, ticks-per-second, live food spawn/growth
- Advanced Stage 1 controls for energy and mutation tuning
- Creature inspector (click creature in viewport)
- Live population/average-energy chart
- Snapshot export/import panel (`GET/POST /simulation/snapshot`)
- Idle placeholder before first start, plus pending-restart state when startup-only values change during a run

## Run the CLI

```bash
cargo run -p petri-cli -- run --ticks 1000 --sample-every 25
```

Run Stage 1b controller stability ablation:

```bash
cargo run -p petri-cli -- ablation --ticks 500
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
