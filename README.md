# Petri

Petri is a Rust and React artificial-life simulation with mutable controllers,
a live spatial ecosystem, and a dashboard backed by applied simulation state.

![Petri creatures foraging in a live simulation](docs/assets/petri-creatures.gif)

![Petri dashboard](docs/assets/petri-v3-ui.png)

## Quick start

Required versions are Aqua >= 2.60.1, uv 0.12.3, Rust 1.93.0, Node 24.20.0,
and npm 11.19.0. Run `make setup`, then `make run`. Open
<http://localhost:5173>.

## Verification

Run `make check` before completing work. `make audit` performs the explicit
full-history secret scan. The simulation viability test is run first by
`make rust-check` whenever Rust validation runs.

## Documentation

- [Documentation index](docs/README.md)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)
- [Changelog](CHANGELOG.md)
