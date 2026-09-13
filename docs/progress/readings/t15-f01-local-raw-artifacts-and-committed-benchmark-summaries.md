# T15.F01 readings

[Spec](../../specs/roadmap/t15-f01-local-raw-artifacts-and-committed-benchmark-summaries.md)
and [artifact format/commands](../../benchmark-artifacts.md).

## Implementation verification

| Check | Result |
| --- | --- |
| TDD | Artifact integration suite initially failed on the missing artifact API; page tests initially failed on missing summary support. Bare-main discovery and exact historical floating-point retention received dedicated regressions. |
| `cargo check --workspace --all-targets` | Passed after coherent Rust edits, exact JSON float round trips and self-review remediation. |
| `cargo test -p v3-cli` | Passed after self-review: library 98/98 (8.60 s), binary 11/11, benchmark integration 20/20 (215.47 s), artifacts 18/18 (8.48 s), CLI integration 11/11 (0.20 s), 0 doctests; 158 passed overall, exit 0. |
| `cargo test -p v3-cli --test cli` | 11 passed, 0 failed. |
| `cargo test -p v3-cli --doc` | Passed; 0 doctests. |
| `cargo test -p v3-cli --test bench_artifacts` | 18 passed, 0 failed, 0 ignored; 7.69 s. Includes generated unrounded-ratio parity and fixed checkpoint/claim bounds, deterministic conversion, historical float fidelity, retained recruitment counts, metadata consistency, symlink/hardlink output aliases and write failures. |
| `cargo clippy -p v3-cli --all-targets -- -D warnings` | Passed after self-review remediation; exit 0. |
| `cargo fmt --all -- --check`; `git diff --check` | Passed after self-review remediation. |
| `scripts/dependency-policy-check` | Passed with existing locked `sha2`, direct `same-file` 1.0.6 and `serde_json`'s `float_roundtrip` feature. |
| `cargo test -p v3-cli --test bench_artifacts checked_in_goal_recipe_identities_are_unchanged_by_json_precision -- --nocapture` | Passed both before and after `float_roundtrip`; all three effective identities unchanged (table below). No simulation/observation runs. |
| `make quality-check` | Passed: ShellCheck/actionlint, 3 progress-page/Makefile tests, existing benchmark-wait, mutation-wrapper, development-shutdown and skill-cache tests. Aqua's metadata timestamp writes were unavailable in the sandbox; checks themselves passed. |
| `make roadmap-check` | Passed after workflow/template/output documentation edits. |
| `make check` | Pending orchestrator closure gate. |
| Mutation gate | Pending separate mutation specialist. |

The output-path integration fixture creates temporary main and linked Git
checkouts whose names contain spaces, resolves gate/goal/sweep defaults from
both, and runs one tiny sweep in each. Both use main's raw root and the calling
checkout's summary directory. It covers explicit raw/summary overrides,
missing Git, a bare primary repository, alias rejection and explicit reference
protection. Fixture directories are created under the OS temporary directory as
`petri artifact test <pid> <sequence>` and removed by the test. The existing
Makefile is exercised with a stub of `scripts/bench-wait` in a temporary
`petri make artifact <suffix>` directory; no measured baseline is run by that test.

## Explicit implementation self-review

| Finding | Correction and evidence |
| --- | --- |
| Redundant full-report allocations during output | One borrowed, buffered JSON writer with explicit newline/flush; release live `Report` before conversion and raw bytes before summary writing; verify/hash stored bytes with a 64 KiB buffer. Historical conversion retains identical bytes and the hash below. No memory benchmark or streaming-parser framework added. |
| Summary metadata could disagree with comparison inputs | Reuse `Environment`, `ProfileBlock`, `PerCreatureTick` and measured identity to reject revision/time/host/profile/normalized/wall mismatches and unsupported source versions. Malformed metadata regression was red, then green. |
| Hardlink aliases could truncate evidence; path errors were ignored | Reuse locked `same-file` for existing-file identity and preserve existing-ancestor resolution for future paths. Conversion input/output, raw/summary and raw/reference hardlinks are rejected before truncation; symlink-loop errors propagate. Dedicated regressions were red, then green. |
| Recruitment backend/operator inapplicability was dropped with proposals | Bounded enum-keyed arm cross-tabs plus unresolved count; existing opportunity totals remain untouched. Synthetic first-arm fixture checks 18 selected-inapplicable and 21 unresolved counts independently of random mutations. Regression was red, then green. |
| Review of remediation itself | Checked ownership/drop order, flush/error propagation, alias checks before writes, finite numeric inputs, fixed-buffer byte equality, enum-keyed deterministic ordering, preserved estimates/denominators and historical absence. No additional finding requiring production changes. |

All four correctness regressions failed before implementation; the artifact
suite then passed all 18 tests. The spec owner's fifth consultation approved
these scoped fixes; its sixth required no further correction and confirmed
typed raw field ordering is acceptable when actual written bytes are hashed.
All guidance was accepted. Simulation defaults, counters,
recipes, observation work and threshold rules remain unchanged. Independent
final review, measured baselines, mutation testing and `make check` remain
separate closure gates.

## Historical conversion and consumer check

| Evidence | Reading |
| --- | --- |
| Source | `docs/progress/features/t14-f03-applied-mortality-and-energy-accounting-goal.json` |
| Source bytes | 3,976,198 |
| Source SHA-256 | `c897a562f5c370ddd1388fb2a173e095b0dde72c2be97a3c4c2461887ecfc511` |
| Temporary output/provenance | `/tmp/petri-t15-f01-conversion.i4LuYC/summary.json`, `/tmp/petri-t15-f01-conversion.i4LuYC/provenance.json` |
| Captured verification time | `2026-09-13T15:19:29Z` |
| Summary bytes | 2,275,733 (57.234% of source; bounded aggregate evidence retained) |
| Summary SHA-256, both fixed-input conversions | `9a220da5a9c056886c6dfb1ad7cf99abc8596d3ee828bf232ba2ae2cd36bd5c2` |
| Original measured revision/time | `6ee48ed8a75f7fda2922cd9e7309f182add85b04` / `2026-09-12T17:59:52Z` |
| Repeat conversion and independent consumer comparison | Passed twice with identical summary bytes. Actual page loader reads 2 mixed artifacts, with no raw fetch; all 8 available claims resolve, and complete original environment, comparisons, per-seed readings, totals and normalized counters match independently parsed raw JSON. |

```sh
target/debug/v3-cli bench-summarize \
  --input docs/progress/features/t14-f03-applied-mortality-and-energy-accounting-goal.json \
  --out /tmp/petri-t15-f01-conversion.i4LuYC/summary.json \
  --provenance /tmp/petri-t15-f01-conversion.i4LuYC/provenance.json
shasum -a 256 /tmp/petri-t15-f01-conversion.i4LuYC/summary.json
node /tmp/petri-t15-f01-conversion.i4LuYC/verify.mjs
```

The command is repeated with identical inputs. An independent Node check hashes
the raw bytes with `node:crypto`, resolves every claim pointer against the raw
JSON, checks original measurement identity and stored comparisons, and loads
both artifacts through the actual progress page's loader/mesh extractor. The
mock fetch accepts only the index and the two report paths; accessing local raw
provenance would fail the check.

The independent numeric check found a one-ULP loss in default Serde JSON float
parsing. The existing `float_roundtrip` feature preserves the original values;
the literal regressions include `0.9805590711984373` and
`0.9953555281754939`. Measured identity is retained, not relabelled as the
conversion time. No threshold, recipe, default, counter or observation changed.

| Goal recipe | Effective config SHA-256 before and after JSON precision change |
| --- | --- |
| Orchards in grassland | `8141056a33bc30445fd29340fa40498372835e08459b7c04c347f9724b445423` |
| Canyon country | `ee72c5532cb947fad7349a3a4d3c5a5b5bef2c501ebe5b299b5de12ad26bd99d` |
| Confluence | `25fb4d0baf34719c0f1e657c98b4e8a7932216510d69651c6fd58d4f6d318676` |

| Historical case mesh aggregate, full = summary | Genomes | Total / reachable / executed / knockout | Route varies | Cap hits / executions |
| --- | --- | --- | --- | --- |
| Orchards in grassland | 12 | 54 / 50 / 38 / 16 | 4 | 0 / 960 |
| Canyon country | 12 | 49 / 40 / 33 / 7 | 4 | 0 / 960 |
| Confluence | 12 | 59 / 50 / 40 / 16 | 3 | 0 / 960 |

| Representative claim pointer | Exact source value |
| --- | --- |
| `/deterministic/totals/creature_ticks` | 60,968,679 |
| `/deterministic/totals/births` | 1,198,576 |
| `/deterministic/per_creature_tick/vm_steps` | `"23.478709"` |
| `/deterministic/goal_indicators/population_persistence/per_seed/0/final_population` | 11,313 |
| `/comparison/severe` | `false` |

## Closure measurements

| Profile | Committed summary | Local raw artifact | Status |
| --- | --- | --- | --- |
| Gate | `docs/progress/features/t15-f01-local-raw-artifacts-and-committed-benchmark-summaries.json` | `<main-checkout>/.bench-artifacts/t15-f01-local-raw-artifacts-and-committed-benchmark-summaries/gate.json` | Pending benchmark specialist |
| Goal | `docs/progress/features/t15-f01-local-raw-artifacts-and-committed-benchmark-summaries-goal.json` | `<main-checkout>/.bench-artifacts/t15-f01-local-raw-artifacts-and-committed-benchmark-summaries/goal.json` | Pending benchmark specialist |

Record both byte counts, raw hashes, original revision/time, CLI and observed
outer statuses with their sources, severe/inherited flags and sources,
threshold/cap verdicts, and per-world readings here after measurement. Existing
T13.F02's local-only goal exception is not a substitute reference.
