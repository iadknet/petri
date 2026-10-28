# T30.F15 — One Definition per Population Statistic

**Status**: In Progress
**Last updated**: 2026-10-28
**Feature**: T30.F15
**Track**: [T30 — Core Boundaries](../../roadmaps/t30-core-boundaries.md)

## Goal

Mean energy, clade entropy and mean capacity each have one definition, in
core's readers (`neighborhood/`): the one the benchmark report uses today.
Every output that carries one of them reads that definition, so the
benchmark report, the NDJSON stream, the server's status and health payloads
and the `petri.tick.*` gauges never disagree on the same quantity at the same
tick beyond an `f32` field's cast or the report's six-decimal formatting. The benchmark report and the committed summary for a given full report
are unchanged; the live outputs change by rounding only; trajectories and
draw streams are unchanged. The three live definitions and the six
`WORKAROUND(T30.F13): one definition of …` markers are gone.

## Non-Goals

- Wire, NDJSON and telemetry-export types other than invariant 2's one
  internal field: an `f32` wire or NDJSON field stays `f32` and casts (the
  row), so the frontend's generated types are unchanged.
- Any other quantity's definition, per-clade means (`CladeProfile`), and
  extinction semantics: the benchmark's means keep `None`/`null` and its
  entropy `UNDEFINED`; the live outputs keep `0`.
- World-input keys and the `WORKAROUND(T30.F13)` marker at
  `v3-server/src/state.rs:574` (T30.F16).
- New signals, censuses or payload fields.

## Inputs and Invariants

Inputs: the T30 row and its Notes entry for F15 (user direction,
2026-10-28; the
[F15–F16 design review](../../progress/readings/t30-f15-f16-design-review.md)),
T30.F13's invariant 2 and audit rows R4–R6, `docs/architecture.md` section 3,
and the code on `2ce6e4632`. The consumer inventory, the pin inventory and the
preparatory audit (A1–A4) are in the
[readings](../../progress/readings/t30-f15.md).

Invariants:

1. **The benchmark's definition is the one definition.** Each stays where it
   is, with its arithmetic unchanged:

   | Quantity | The one definition | Deleted |
   | --- | --- | --- |
   | mean energy | `PopulationCensus::mean_energy`: `f64` sum of the `f32` energies in population order over the count, `None` when empty | `PopulationCensus::mean_energy_live` |
   | clade entropy | `CladeCensus::entropy_nats`: summed in lineage-id order, `None` when empty; the effective lineage count is its `exp`, a reader output `None` when empty | `CladeCensus::effective_lineages_live` |
   | mean capacity | `TraitDistribution::of`'s sorted-order `f64` mean, reached through `distributions::capacity_distribution`, `None` at extinction | `PopulationCensus::mean_capacity_live` |

2. **Live outputs read it and only cast.** An outer crate adds no sum,
   order or `exp` of its own; it casts where its field is `f32` and applies
   its extinction value:

   | Output | Field type | Reads | Empty population |
   | --- | --- | --- | --- |
   | NDJSON `tick_sample.mean_energy`, `run_completed.final_mean_energy` (`v3-cli/src/lib.rs`) | `f32` | mean energy, cast | `0.0` |
   | server `StatusPayload.mean_energy`, `HealthPayload.mean_energy` (`v3-server/src/state.rs`) | `f32` | mean energy, cast | `0.0` |
   | server `HealthPayload.lineages_effective` | `f32` | effective lineage count, cast | `0.0` |
   | telemetry `Census.mean_energy` → `petri.tick.mean_energy` (`v3-telemetry/src/metrics.rs`) | `f64` (was `f32`) | mean energy, uncast | `0.0` |
   | telemetry `Census.mean_capacity` → `petri.tick.mean_capacity` | `f64` | the capacity distribution's mean | `0.0` |

   The benchmark's sample `mean_energy`, `shannon_entropy_nats`,
   `lineage_diversity` and `capacity` blocks keep their code paths. So a live
   `f32` value equals the reader's value for the same population cast to
   `f32`, and a gauge equals it exactly.

3. **Only the named pins move.** The readings' pin table lists every pin
   literal the feature may move, with the fields that may change in each and
   why; a listed pin may also stay. Any other moved pin, or a listed pin
   moving in another field, is a defect. Every benchmark-block pin in
   `v3-cli/src/bench/census_pins.rs` and every empty or extinct pin is
   unchanged.

## Architecture

The feature touches one owner, readers (`neighborhood/`), which
`docs/architecture.md` section 3 makes the home of every population
calculation "computed once with one definition", and the outer crates
section 2 limits to transport and export. The shared rule it changes is the
definition of the three quantities: the owner's operations are
`PopulationCensus::mean_energy` (`neighborhood/population.rs`),
`CladeCensus::entropy_nats` and the effective lineage count
(`neighborhood/clades.rs`) and `TraitDistribution::of` with
`capacity_distribution` (`neighborhood/distributions.rs`); the paths that
interpret them are the benchmark sample (`v3-cli/src/bench/tracking.rs`,
`indicators.rs`, `capacity.rs`), the NDJSON stream (`v3-cli/src/lib.rs`), the
frame builder (`v3-server/src/state.rs`) and the telemetry census
(`v3-telemetry/src/metrics.rs`). Every interpreting path reuses the owner's
operation; no separate policy remains. The paired definitions were T30.F13's
unpaid accepts (R4–R6), not section 5 debts, so no debt line is deleted, no
boundary changes and `docs/architecture.md` is not edited.

## Telemetry

No signal is added or removed and no configuration is added. Two existing
gauges change value by rounding:

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.tick.mean_energy` | gauge | `PopulationCensus::mean_energy`, now the `f64` benchmark mean (was `f32`); `0` at extinction | unchanged: `petri.run_id`, sampled at the snapshot tick |
| `petri.tick.mean_capacity` | gauge | the capacity distribution's sorted-order mean (was a creature-order sum); `0` at extinction | unchanged |

The added work is one allocation and sort of the living population's `f32`
capacities per metrics snapshot (the interval, transition and run-end
snapshots), in place of a fold; the overhead check measures it.

## Verification Plan

**Question.** Do the live outputs now read the benchmark's definitions while
the benchmark report stays unchanged? A gate deterministic block that differs
from the epoch's, a work counter off 0%, a live value that is not the
benchmark value (cast where `f32`), or a pin moving outside the readings'
pin table would show it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate` | The gate seeds confirm on real worlds that the benchmark's deterministic blocks and the nine work counters are unchanged, the comparability the row promises; no population reading is in question. |
| Slow tests (`make rust-test-slow`) | `not run` | No `crates/v3-lab` change. |
| Telemetry overhead check | `run`, default `run` workload | The workflow's telemetry rule 3 runs it when a feature adds work to a run; the capacity sort runs in every metrics snapshot of `v3-cli run`, so `scripts/telemetry-overhead` (T21 workload, ceiling 1.10) measures it on that path. |

## Implementation Tasks

- [x] Record each pin in the readings' pin table as it renders on
      `2ce6e4632` (the JSON a hashed pin prints, the bits a bit pin shows).
- [x] Delete `mean_energy_live`, `mean_capacity_live` and
      `effective_lineages_live` with the six paired-definition markers; add
      the effective lineage count to `CladeCensus`; make the module and item
      docs of `population.rs`, `clades.rs` and `distributions.rs` state one
      definition (audit A1–A3).
- [x] Repoint every live output in invariant 2's table, widen telemetry's
      `Census.mean_energy` to `f64` (A4) and correct its doc, which says it
      copies `tick_sample`'s expressions, and repoint the tests that read the
      deleted functions (`v3-core/tests/viability.rs`,
      `v3-telemetry/src/tests.rs`, the reader tests).
- [x] Replace the two "two definitions" reader tests with tests that each
      live output equals its definition (cast where `f32`) on a living
      population and is `0` on an empty one.
- [x] Re-pin only the moved literals of the pin table, note in each pin
      module's header which literals T30.F15 re-pinned, and record the
      before and after values in the readings.
- [ ] Run the declared `gate` profile and the overhead check.

## Verification

- [x] No paired definition remains:
      `grep -rnE 'WORKAROUND\(T30\.F13\): one definition|mean_energy_live|mean_capacity_live|effective_lineages_live' crates/`
      -> no match.
- [ ] Live outputs equal the one definition: the new reader, CLI, server
      and telemetry tests, in `make check`.
      They pass under `cargo test` of `v3-core --features telemetry-seams
      --lib`, `v3-cli`, `v3-server` and `v3-telemetry`; `make check` is
      not yet run.
- [x] Pins: `cargo test -p v3-cli census_pins`,
      `cargo test -p v3-server census_pins` and
      `cargo test -p v3-telemetry the_tick_census` pass; the before/after
      table in the readings shows only the pin table's fields moved.
      Moved: `NDJSON_PINS` `stream` (three `f32` mean energies) and
      `LIVING_CENSUS_PIN` `mean_energy` (`f64` bits); every other pin holds.
- [x] `cargo test -p v3-core --test viability` passes.
- [x] Build: `cargo check --workspace --all-targets` (and with
      `--all-features`), `cargo fmt --all --check` and `make rust-clippy`
      pass.
- [ ] `make check` exits 0.
- [ ] Overhead: `scripts/telemetry-overhead` (stack up, `run` workload) ->
      pass, its pair count, ratio interval and time used in the readings.
      INCONCLUSIVE, not a pass: spread attempt 1 0.2476, attempt 2 0.0708
      -> 24 pairs at 5303 ms do not fit the 300 s cap (102.6 s used); 0
      pairs, no ratio or interval.
- [x] Gate: summary at `docs/progress/features/t30-f15.json`, raw hash and
      byte count checked, series entry points to it, no full report staged.
      `make bench PROFILE=gate FEATURE=t30-f15` exit 0, `severe=false`;
      raw 161168 bytes, sha256 `df9ed774...7a87`.

## Performance and Goal Impact

**Predeclaration — written before the run.** Not a mechanism feature; no
simulation code changes, so no indicator can move and the epoch is not
re-pinned. Reference: the series' epoch baseline
(`docs/progress/features/t25-f11.json`), at the standard thresholds (flag
10%, severe 50%). Expected: the gate's deterministic blocks identical to the
epoch's and all nine work counters at 0.000000%, since the benchmark's
definitions and code paths are unchanged and the counters count simulation
work. Expected cost: one sort per telemetry metrics snapshot, outside the
tick loop. Wall-clock time per creature tick is compared on the same host
and labelled (flag 25%, severe 100%) but cannot mark the comparison severe;
expected inside the flag threshold. A counter off 0%, or a deterministic
block that differs, is a defect, not a cost.

Overhead check, predeclared: `scripts/telemetry-overhead` with the default
`run` workload (`v3-cli run --seed 7` on `telemetry/overhead-world.json`, `T`
from `docs/progress/readings/t21-f01.md`), the production stack up for both
sides, the pair count set by the script's five-run spread rule (16, 24 or
32), and its statistic: pass when the 90% interval's upper bound of the
median ratio is below 1.10, fail when the lower bound is at or above it,
otherwise inconclusive, as is reaching the 300 s cap. Expected: pass, the
sort being small beside the snapshot's per-genome censuses. An inconclusive
result is reported to the user, never rounded to a pass or extended.

**Measured verdict.** Gate (`make bench PROFILE=gate FEATURE=t30-f15`, exit
0, CLI exit 0): `severe=false` against `t25-f11.json`; deterministic block
identical to the epoch's; all nine work counters at 0.000000%; no wall-clock
reference in the epoch, so none compared. Overhead
(`scripts/telemetry-overhead`, exit 0): inconclusive with 0 pairs (24 pairs
at 5303 ms per run do not fit the 300 s cap; 102.6 s used; first spread
0.2476 on a cold stack, second 0.0708); no ratio or interval. Details in the
readings.

- Summary of the declared profile: [gate](../../progress/features/t30-f15.json).
- Full readings: [`docs/progress/readings/t30-f15.md`](../../progress/readings/t30-f15.md).

## Success Criteria

- [ ] Each of the three quantities has one definition in readers, and every
      output carrying it reads that definition.
- [ ] The benchmark report and summary are unchanged; only the pin table's
      fields moved.

## Notes for AI Agents

- Decision: each quantity keeps the benchmark's definition and the live
  outputs read it, so stored reports and the benchmark series stay
  comparable (user direction, T30 row, 2026-10-28); `f32` wire and NDJSON
  fields cast the `f64` result.
- Decision: the live definitions and their markers are deleted rather than
  kept beside the one definition, and telemetry's census carries mean energy
  as `f64` so its `f64` gauge is not rounded through `f32` (audit A1–A4).
- Decision: final review lens1 P2 (order-insensitive capacity fixture)
  remediated with an order-sensitive fixture; lens2 and lens3 clear.
