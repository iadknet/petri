# Petri

This is a pet project of mine to experiment with genetic programming algorithms in an artifical life simulation.  It is based on a program I had fun playing with in college that used genetic programming principals to simulate an ecosystem of "creatures" that would scavenge, reproduce, communicate, and evolve.

Petri is written in rust and is one of the primary projects I use to experiment with new agentic coding workflows and toolsets.

It integrates an array of different artifical life and machine learning approaches with the goal of creating a small ecosystem of diverse creatures that can evolve complex behaviors.


![Petri creatures foraging in a live simulation](docs/assets/petri-creatures.gif)

![Petri dashboard](docs/assets/petri-v3-ui.png)

## Quick start

Install Aqua >= 2.60.1 and Rust 1.93.0 through rustup. `make setup` creates
project-local Aqua proxy links for the pinned Node 24.20.0 (including npm
11.19.0), uv 0.12.1, and repository CLIs; Aqua downloads each pinned tool on
first use after verifying its committed checksum. It then installs hooks and
frontend dependencies. Run `make setup`, then `make run`. Open
<http://localhost:5173>.

## Verification

Run `make check` before completing application or runtime source-code or
build-configuration changes. Use the relevant focused checks for
documentation-only work. `make audit` performs the explicit full-history secret
scan. `make rust-check` runs format validation, the explicit viability merge
gate, `make rust-test-all`, and Clippy; `rust-test-all` runs the complete named
Rust test set. CI fans those named test targets out in parallel. Run `make help`
to see the individual local targets.

## Documentation

- [Documentation index](docs/README.md)
- [Roadmap execution workflow](docs/workflow.md)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)
- [Changelog](CHANGELOG.md)
