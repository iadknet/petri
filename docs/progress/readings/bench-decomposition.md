# bench-decomposition readings

[Spec](../../specs/bench-decomposition.md) and
[artifact format/commands](../../benchmark-artifacts.md).

## Closure measurement

| Profile | Committed summary | Local raw artifact | Status |
| --- | --- | --- | --- |
| Gate | `docs/progress/features/bench-decomposition.json` (96,820 bytes; SHA-256 `692364769f09522ae0c1072959dc68ecbdf03a968cdf3a093c7c92ea1584bc49`) | `<main-checkout>/.bench-artifacts/bench-decomposition/gate.json` (94,024 bytes; SHA-256 `ac386a135c1593a043304b19f879f2dd7fee6712d4e13ba64a53fb75f5011292`) | Pass: `comparison.severe=false`; all 12 metric levels (6 counters x 2 references) `ok`. |

- Command: `make bench PROFILE=gate FEATURE=bench-decomposition`; observed outer `make` exit 0 (shell exit status); CLI exit 0 from `measurement_evidence.cli_exit` (`v3-cli` successful artifact-pair completion, `dirty=false`).
- Commit measured: `a8611434` (production code final `aec1954a`, plus one docs-only commit).
- Compared against `docs/progress/features/remove-complementary-nutrition.json` (epoch): `mesh_hops` +0.083512% ok, `vm_steps` -1.429996% ok, `graph_relax_iters` -0.031550% ok, `plasticity_updates` -11.556709% ok, `actions_applied` -0.208934% ok, `births` +0.455564% ok.
- Compared against `docs/progress/features/t15-f01-local-raw-artifacts-and-committed-benchmark-summaries.json` (last-closed): all 6 counters 0.000000% delta, `ok`.
- No goal profile required by this spec (Performance and Goal Impact: none intended or expected).

## Deterministic-block identity check

| Item | Value |
| --- | --- |
| Pre-move summary | `/tmp/bench-decomposition-before.json`, 96,493 bytes, SHA-256 `0d12e669d95eb6edcc45ae0989a1ed9f849f00c20832829f25ac90af2af5c86a` (measured on starting commit `c100db3b`) |
| Post-move summary | `docs/progress/features/bench-decomposition.json`, 96,820 bytes, SHA-256 `692364769f09522ae0c1072959dc68ecbdf03a968cdf3a093c7c92ea1584bc49` (measured on `a8611434`) |
| Command | `node -e '...JSON.stringify(...deterministic...)' /tmp/bench-decomposition-before.json docs/progress/features/bench-decomposition.json && cmp /tmp/bench-decomposition-before.det /tmp/bench-decomposition-after.det` (verbatim per spec) |
| `node` exit | 0 |
| `cmp` exit | 0 (byte-identical `deterministic` blocks) |

Wall-clock fields are not compared, per the spec's Performance and Goal Impact
section.
