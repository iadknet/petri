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

## Run the web UI

```bash
cd web
npm install
npm run dev
```

Open `http://127.0.0.1:5173`.

The app expects the backend on `127.0.0.1:4000`.

## Run the CLI

```bash
cargo run -p petri-cli -- run --ticks 1000 --sample-every 25
```

Run Stage 1b controller stability ablation:

```bash
cargo run -p petri-cli -- ablation --ticks 500
```

## Verify

```bash
cargo test --workspace
cd web && npm run build
```
