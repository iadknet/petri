# T13.F04 — Direct Graph Effect Activation readings

Tables and transcripts for the feature spec
[`docs/specs/roadmap/t13-f04-direct-graph-effect-activation.md`](../../specs/roadmap/t13-f04-direct-graph-effect-activation.md).

## Build-pass verification

| Command | Result |
| --- | --- |
| `cargo test -p v3-core --test viability` (run first) | ok, 24 passed, 0 failed |
| `cargo test -p v3-core` | ok, all suites pass (1428 lib, 19 baseline_worlds, 24 viability, remaining suites 0 failed) |
| `cargo test -p v3-core --test reproducibility` | ok, 3 passed |
| `cargo test -p v3-cli` | ok, 163 passed across suites, 0 failed, 1 ignored |
| `cargo clippy --workspace --all-targets` | clean, no warnings |
| `cargo fmt --all` | applied, no further diff |
| `cargo check --workspace --all-targets` | clean |
| `make roadmap-check` | pass |

## Re-pinned expectations

| Test | Change | Reason |
| --- | --- | --- |
| `crates/v3-core/tests/baseline_worlds.rs::legacy_default_short_run_identity` | `12330723111342885916` -> `11753828254793484309` | Wired zero-compute Graph modules now enter their visit, so the 30-tick run with births diverges in trajectory and charge. The new hash reproduced across three separate processes before re-pinning; the identity the test pins is unchanged. `AddRouteTarget` supplies one such genome directly: it attaches a fresh blank Graph detour carrying a weight-1 router-gate edge, whose gate write was skipped before this repair. |
