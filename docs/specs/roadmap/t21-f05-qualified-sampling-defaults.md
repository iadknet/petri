# T21.F05 — Qualified Sampling Defaults

**Status**: In Progress
**Last updated**: 2026-09-30
**Feature**: T21.F05
**Track**: [T21 — Run Observability and Execution Tracing](../../roadmaps/t21-run-observability-and-execution-tracing.md)

## Goal

Telemetry is on by default in `v3-cli`, `v3-server` and `v3-lab`, and the
default configuration is the named preset `standard`, whose cost against the
reference build was measured by one 60-minute measurement on a default-size
world and a small one with the stack healthy, stopped and slow, from which
the user set the ceiling at 10% of elapsed time with 5% as the target. Four named presets (`minimal`, `phases`, `standard`,
`dense`) select the sampling settings T21.F02–F04 delivered, each with its
measured time and storage cost. Tests that start a binary pass
`--telemetry off`, so `make check` needs no Docker; closure benchmark runs
export and still write their stored reports. With telemetry off, on, or on with no stack listening, a run's deterministic
output is identical.

**D5 (user decision, goal of 2026-09-30).** "If F05's qualifying measurement
puts the default under 5%, record 5% as the ceiling, otherwise stop and
report the measurements for the user's decision." Resolved 2026-09-30
(Notes): five cells under 5%, W1 `s` inconclusive at [1.0232, 1.0505]; the
user recorded 10% as the ceiling and flipped the default on; 5% stays the
target.

## Non-Goals

- No adaptive governor: the track holds it in reserve until fixed caps are
  shown to fail, and this measurement is what shows it.
- No new signal, span, attribute or dashboard beyond the preset name on
  `run.started`; no change to any cap, queue bound or cadence semantics of
  F02–F04; no change to what a preset's settings mean.
- No CPU-time reading, no thread-count change and no second reference: the
  ceiling governs elapsed time against the `--no-default-features` build,
  as the workflow says.
- The bench, assay and lab paths are not re-measured: they export only after
  their last timed region (F06), so the qualifying workload is `v3-cli run`.
- No edit to the assay modules, `v3-core` or `docs/workflow.md`.

## Inputs and Invariants

| Input | Where | What F05 takes from it |
| --- | --- | --- |
| Track row and notes | `docs/roadmaps/t21-run-observability-and-execution-tracing.md` | F05 row; Overhead ceiling (5% target, F05 measures cost against detail, the user sets the final ceiling; not default-on at the interim ceiling); Telemetry flag (F05 flips the default in every binary in the same change; tests and closure measurements pass off); Verification; Overhead check (F05 covers a default-size and a small world, three stack states; self time is a lower bound); Sampling contract; Budget arithmetic; Decisions (D5). |
| Benchmark gate | `docs/workflow.md`, "Benchmark gate" | The 25% ceiling until F05 closes and the ceiling F05 records after; one ratio, one reference; the method requirements (interleaved pairs, spread-chosen pair count, predeclared inconclusive, `scripts/bench-wait`, nothing alongside); the routine 5-minute cap and F05's one 60-minute allowance; a cap reached is inconclusive; the flip is a process-level default, not a simulation default. |
| Research note | `docs/strategy/run-observability-research-2026-09-29.md` | Staged ceiling; guardrail 4 (a preset over the ceiling is opt-in and labelled); D5. |
| F01 spec | Goal, Inputs and Invariants, Performance | `resolve_switch` (flag beats `PETRI_TELEMETRY` beats the default); the reference build; the stderr run line (`self_time_us`, `flush_ms`, `dropped`, `abandoned`); queue bounds (5 s request, 10 s flush); `scripts/telemetry-overhead` and its scratch stack; T = 6908 and the calibration rule. |
| F02 spec | as above | `PETRI_TELEMETRY_METRICS_INTERVAL_MS` (default 1000, 10–3600000); the settings pattern (read only when on, invalid refuses to start, recorded on `run.started`); a snapshot is about 195 KB on the wire (F02 readings). |
| F03 spec | as above | `PETRI_TELEMETRY_TICK_TRACES` (default on); about 5–6 KB per trace, one per snapshot. |
| F04 spec | as above | `PETRI_TELEMETRY_CREATURE_WINDOWS` (on), `_WINDOW_TICKS` (8, 1–64), `_WINDOW_INTERVAL_MS` (10000, 10–3600000); about 80 KB per window plus a 12 KB genome record; 42 µs per recorded creature-tick; windows off against the reference read 1.04 at spread 15–16%. |
| F06 spec | as above | Measurement commands hold export until the last timed region; `make bench` passed `--telemetry off` from F06 until the user's decision of 2026-09-30 (invariant 2), which supersedes D2's second half. |
| Switch and settings | `crates/v3-telemetry/src/lib.rs` `resolve_switch`, `Options::from_env`; `metrics.rs` `resolve_metrics_interval`; `trace.rs` `resolve_tick_traces`; `windows.rs` `WindowSettings::from_env` | `resolve_switch(None, None)` is `Off`; each setting resolves its own variable with a built-in default; a preset slots in as the source of those defaults. |
| Binaries | `crates/v3-cli/src/main.rs` `Cli`, `crates/v3-server/src/bin/server.rs` `Args`, `crates/v3-lab/src/{main,cli}.rs` | The flag's help says "default off"; each binary has a `…_which_beats_default_off` test. |
| Binary-spawning tests | `crates/v3-cli/tests/{cli,bench,bench_artifacts}.rs` (2, 6 and 4 `CARGO_BIN_EXE` sites without the flag) and `crates/v3-lab/tests/lab_run.rs` (one site without it, line 814); the telemetry fixtures in `crates/v3-cli/tests/telemetry.rs`, `telemetry/measurement.rs` and `lab_run.rs`'s telemetry test pass `--telemetry <switch>` explicitly on every leg | The thirteen sites that gain `--telemetry off`; the fixtures' explicit `on` legs are the exception. |
| Scripts and Makefile | `Makefile` `bench` (off), `lab` (no flag), `rust-test-*`; `scripts/dev.sh` (starts `v3-server`); `scripts/telemetry-parent-compare` (off); `scripts/telemetry-dashboards-check` (on) | Which invocations follow the default and which pass a flag. |
| Host and stack (probed 2026-09-30, scratch project `petri-telemetry-f05plan`) | 8 cores, Docker Desktop 8 CPUs, the stack limited to 2 CPUs; a paused container's published port still accepts the connection and never answers (curl times out); a container at `--cpus 0.05` answers in 50–190 ms; a stopped stack refuses in 26 ms; the reference binary runs the default world's ticks 0–40 at about 75–130 ms per tick under load 9–13 with about 0.14 s of start-up, and `run --ticks 0` exits 1 | The slow state's definition, the default-world tick count and the init reading below. |

Invariants:

1. No `SimulationConfig` or `RuntimeConfig` field, no production RNG draw,
   no simulation default, no stored summary content and no deterministic
   output changes; `v3-core` is not edited; every change to the binaries
   sits behind their `telemetry` feature. The telemetry default is a
   process-level setting.
2. Default. `resolve_switch(None, None)` is `On`; the flag beats
   `PETRI_TELEMETRY`, which beats it; the three binaries' help text says
   so. The invocations that pass `--telemetry off` explicitly are every
   test that spawns a binary (the thirteen `CARGO_BIN_EXE` sites above; the
   only spawns without `off` are the telemetry fixtures' legs, which pass
   `on` explicitly) and `scripts/telemetry-parent-compare`, which checks
   behavior on both sides. Everything else, `make bench` included, follows
   the default: closure benchmark runs export and still write their stored
   reports (Notes). `make check` passes with no stack listening and
   `PETRI_TELEMETRY` unset.
3. Presets. `PETRI_TELEMETRY_PRESET` names the base values of the five
   settings (table below); default `standard`; read only when telemetry
   resolves on; anything but the four names refuses to start, as an
   invalid interval does. An explicitly set variable overrides the preset's
   value for that setting only. `run.started` carries `petri.preset` beside
   the effective values it already carries, and F01's stderr start line
   gains them too
   (`telemetry: on endpoint=<url> preset=<name> interval_ms=<n> tick_traces=<on|off> windows=<on|off> window_ticks=<n> window_interval_ms=<n> invocation=<id> run=<id>`),
   so a run whose records never reach a stack still shows what it ran with.
   The telemetry fixtures in `crates/v3-cli/tests/telemetry.rs`,
   `telemetry/measurement.rs` and `crates/v3-lab/tests/lab_run.rs` remove
   `PETRI_TELEMETRY_PRESET` from the spawned environment as they remove
   `PETRI_TELEMETRY` and the endpoint, and a test of one preset sets it
   explicitly. Presets change no cap or queue bound.
4. Qualification. The predeclared method in Performance and Goal Impact is
   run once, inside the 60-minute allowance, by the implementer's
   verification pass, with nothing else measuring on the host; its verdict
   applies D5 as the Goal states. No cell is rerun or extended.
5. The ceiling in force after this closure is 10%, ratio 1.10, the user's
   D5 decision recorded in the Measured verdict, and
   `scripts/telemetry-overhead` carries it in its one `ceiling` variable,
   read by both the routine and the qualify mode, so a later qualify run
   would be judged at 1.10 while the recorded qualifying run was judged at
   the predeclared 1.05; 5% stays the target the Expected row is written
   against. The routine check keeps F01's workload,
   states, ratio and 300 s cap and adopts the statistic, pair table,
   environment isolation and run timeout below, because F01's
   75%-of-pairs rule cannot decide at 5% on this host (readings file), and
   drops F01's one rerun of an inconclusive state, an extension that rarely
   fits the cap. A pair count the 300 s cap cannot
   hold makes the routine check inconclusive before any pair. Routine
   calibration, unused while T = 6908 is pinned, accepts the first in-band
   run of up to three, the same acceptance the qualifying search uses.

| Preset | `METRICS_INTERVAL_MS` | `TICK_TRACES` | `CREATURE_WINDOWS` | `WINDOW_TICKS` | `WINDOW_INTERVAL_MS` |
| --- | --- | --- | --- | --- | --- |
| `minimal` | 1000 | off | off | 8 | 10000 |
| `phases` | 1000 | on | off | 8 | 10000 |
| `standard` (default) | 1000 | on | on | 8 | 10000 |
| `dense` | 250 | on | on | 16 | 2000 |

`standard` is today's defaults, so a run that sets no variable behaves as
at F04. `dense` is the opt-in detail level; it is labelled with its
measured cost and is expected over the ceiling.

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.preset` | log record attribute on `run.started` (recorded configuration) | the resolved `PETRI_TELEMETRY_PRESET` | beside `petri.metrics_interval_ms`, `petri.tick_traces`, `petri.creature_windows`, `petri.window_ticks`, `petri.window_interval_ms`, whose values are the effective ones |

No other signal changes. The flip changes which runs export, not what a run
exports; the counts, caps and sampling policies stay F02–F04's.

## Implementation Tasks

- [x] Preset resolution in `v3-telemetry` (`Options::from_env`,
      `WindowSettings`): the four presets, per-setting override, invalid
      refuses, `petri.preset` on `run.started`, the effective settings on
      the stderr start line, preset scrubbing in the three fixture helpers;
      unit tests first.
- [x] `scripts/telemetry-stats.mjs` (median, order-statistic interval,
      verdict, the T1 calibration rule) with `scripts/telemetry-stats.test.mjs` under `make check`;
      `scripts/telemetry-overhead` gains `PETRI_OVERHEAD_MODE=qualify`
      (worlds, cells, presets, storage blocks, cap 3,600 s,
      `TELEMETRY_USED_MS` carry, the readings lines it prints), the
      environment scrub, the run timeout and the export check, and the
      shared statistic and pair table in routine mode, verdicts against
      the ceiling, 1.10 after the user's decision;
      the `s` flush flag above 10,050 ms; the `dense` export check
      (`snapshots` ≥ 12, `traces` = `snapshots`, `windows` ≥ 2).
- [x] Default flip: `Switch::default` is `On` and `resolve_switch` falls
      back to it; the three help texts say "default on"; the three
      `…_default_on` tests also check the help text; the routine ceiling is
      1.10, pinned by `telemetry-stats.test.mjs`; `--telemetry off` at the
      thirteen spawn sites (a per-file `TELEMETRY_OFF` constant, empty
      without the `telemetry` feature).
- [x] Parent comparison against `8b0a158f`, T = 6908.
- [x] The qualifying measurement, once, recorded in
      `docs/progress/readings/t21-f05.md`; D5 applied (not under 5%: user
      stop, then the user's 10% ceiling and the flip); the Measured verdict
      and the preset cost table filled in.

## Verification

- [x] `cargo test -p v3-telemetry` -> preset resolution (each preset's
      values, override precedence, invalid refuses, default `standard`),
      `petri.preset` on `run.started` and the stderr start line's settings;
      the neutrality and measurement fixtures pass with
      `PETRI_TELEMETRY_PRESET=minimal` exported in the caller's environment.
      Ran `PETRI_TELEMETRY_PRESET=minimal cargo test -p v3-telemetry -p v3-cli
      -p v3-lab -p v3-server`: all pass (v3-telemetry lib 65, v3-cli
      telemetry 13 with `a_preset_sets_the_base_settings_…`); without the
      scrub four v3-cli fixtures failed under that export.
- [x] `cargo test -p v3-cli`, `-p v3-server`, `-p v3-lab` -> the switch
      tests: `resolve_switch(None, None)` is `On`, `--telemetry off` beats
      `PETRI_TELEMETRY=on`, `PETRI_TELEMETRY=off` beats the default; the
      neutrality test unchanged. The four switch tests (with v3-telemetry's
      `switch_defaults_on_…`) pass;
      `cargo build --release -p v3-cli -p v3-server -p v3-lab
      --no-default-features` compiles with 0 `v3-telemetry`/`telemetry-seams`
      lines in `cargo tree -e features`; clippy `-D warnings` passes with and
      without default features.
- [x] Every `CARGO_BIN_EXE` spawn in `crates/*/tests` passes
      `--telemetry off`, or `--telemetry <switch>` explicitly in a
      telemetry fixture (reviewer grep against the diff); `make check`
      passes with the stack down and `PETRI_TELEMETRY` unset.
      `.args(TELEMETRY_OFF)` at the thirteen sites; `env -u PETRI_TELEMETRY
      make check` exit 0 with no stack running.
- [x] `node --test scripts/telemetry-stats.test.mjs` -> median, interval
      ranks and verdict on fixed inputs, including the strict comparison
      at exactly the ceiling 1.10, read from `scripts/telemetry-overhead`,
      and a voided pair entering as an infinite ratio.
      18 tests pass (the `dense` minimum counts and the T1 calibration
      rule among them); `make
      quality-check` runs the file and exits 0.
- [x] `scripts/telemetry-parent-compare 8b0a158f` -> identical canonical
      output; transcript in the readings file. Exit 0, `identical: 3
      canonical lines, T=6908`.
- [x] `PETRI_OVERHEAD_MODE=qualify scripts/telemetry-overhead` -> the
      readings file's qualification section (per-cell ratios, intervals,
      verdicts, flush and self time, presets, storage, measured time); the
      verdict copied to Performance and Goal Impact. Attempt 2
      (`TELEMETRY_USED_MS=11917`, bracketing T1 rule) exit 0,
      `Qualification: not pass: user stop (D5)`, 2,174.1 s of the cap used.
- [ ] Mutation gate: `Not applicable: observability feature` (the
      workflow's exemption; no run).
- [ ] Benchmark summary: `Not applicable: observability feature`.
- [x] After the flip: `env -u PETRI_TELEMETRY make check`,
      `make check-docs` and `make roadmap-check` exit 0 (no stack running).

## Performance and Goal Impact

**Predeclaration — written before the run.** Profiles: `Not applicable:
observability feature`; the diff changes no simulation behavior, adds no
`SimulationConfig` or `RuntimeConfig` field, draws no production RNG and
writes no stored summary; `v3-core` is not edited; the reviewer checks
that, and that the flip is the process-level default the workflow names.
Measurement feature: the natural-analog rule and the environmental-pressure
rule do not apply, and no indicator can move.

Parent comparison: not triggered by its rule (nothing in `v3-core` or a
tick loop) but run because it costs seconds; `BASE` is `8b0a158f`,
T = 6908. Both sides pass `--telemetry off`. Expected: identical canonical
output.

**Slow-state quantity (user decision, 2026-09-30).** In `s` the verdict
ratio subtracts `flush_ms`, and the flush is bounded separately at F01's
10 s deadline read with 50 ms of scheduling slack; the healthy and stopped
states keep the flush inside their ratio. A paused stack runs every request
to its 5 s timeout and the flush to its bound, which a 5 s workload cannot
amortize.

Qualifying measurement, method fixed before any run:

| Item | Predeclared |
| --- | --- |
| Command | `PETRI_OVERHEAD_MODE=qualify scripts/telemetry-overhead`, every timed run under `scripts/bench-wait`, on the scratch stack `petri-telemetry-overhead`; the orchestrator has stopped the host's other containers, and no build, test, server or other measurement runs alongside |
| Isolation | the script removes every `PETRI_TELEMETRY*` and `OTEL_*` variable from the timed runs' environment, sets `OTEL_EXPORTER_OTLP_ENDPOINT` to the scratch stack's `http://127.0.0.1:4318` and `PETRI_TELEMETRY_PRESET` to the preset under test, and checks every telemetry run's stderr start line against the preset table (invariant 3), voiding the pair on a mismatch; the first line of each cell is printed for the readings |
| Worlds | W1, default-size: `v3-cli run --seed 7 --ticks T1 --sample-every T1` with no `--config` (the built-in default: 1600×1600, 10,000 founders); W2, small: F01's workload, `--config telemetry/overhead-world.json` (128×128, 256 founders), T = 6908 |
| T1 | calibrated once, charged to the cap, by a bracketing search on the reference build aimed at 4 to 6 s: three runs as F01's rule, starting at 20 ticks, each at the count the previous run scales to 5 s; if the third run is outside the band, the search continues by bisection between the nearest counts measured below and above the band (when no run has landed above it, the next count is the one the last run scales to 6 s; when none has landed below it, inconclusive), up to five more runs; the first count whose run lands in 4 to 6 s is T1, printed as `- Default-world ticks T1: N` for the readings file and fixed for later closures; if none lands in the band within eight runs the attempt stops as inconclusive for the user. Linear scaling alone fails on this world because its per-tick cost steps up near tick 41 (ticks 1–40 about 97 ms, 41–48 about 255 ms): attempt 1, under F01's third-run rule, ran 20 → 2,083 ms, 48 → 5,939 ms, 40 → 3,895 ms and stopped; attempt 2, under this rule, ran 20 → 2,023 ms, 49 → 6,001 ms, 41 → 4,119 ms and took T1 = 41, the first in-band run |
| Init | three reference runs of `--ticks 1` per world; the median is an upper bound on a run's start-up share (it holds one tick: about 0.25 s, 5% of a W1 run; under 10% of a W2 run), recorded so a reader sees the most the ratio can be diluted; no correction is applied |
| Reference | the same commit built with `--no-default-features`, always run with the stack stopped |
| Spread | five consecutive reference runs per world after calibration; spread = (max − min) / median |
| Pairs (n) | spread ≤ 5%: 16; ≤ 10%: 24; ≤ 20%: 32; above 20%: the spread block is repeated once after 60 s idle (charged; a wait the remaining cap cannot hold is inconclusive), and if still above 20% that world's cells are inconclusive and it runs no pair |
| Cells and pairs | per world, three cells of preset `standard`: `b` on with the stack stopped, `a` on with the stack started and ready (`scratch_stack_ready` plus 5 s before every telemetry run), `s` on with the stack paused (`docker compose pause lgtm`; the port accepts and never answers, so every export request runs to the 5 s timeout and the end flush to its 10 s bound); a pair is one telemetry run and one reference run adjacent in time, order alternating pair by pair as F01's; the reference always runs with the stack stopped, so `a` starts and stops the stack once per pair and `s` pauses, then unpauses and stops it |
| Ratio | per pair, telemetry process wall / reference process wall, F01's and F06's quantity, the end flush included, in `a` and `b`; in `s` the verdict ratio is (telemetry process wall − `flush_ms` from the stderr line) / reference process wall, the flush-included ratio recorded beside it (user decision above); the median `flush_ms`, `dropped` and `abandoned` per cell are recorded, and every cell's `flush_ms` stays within F01's 10 s deadline plus 50 ms of scheduling slack (the reading includes the return from the abandoned wait; a cell above 10,050 fails, `fail-flush-bound`) |
| Export check | in `a`, a telemetry run whose line shows `failed`, `dropped` or `abandoned` above zero, or counts inconsistent with its preset (`standard`: `snapshots` ≥ 4, `traces` = `snapshots`, `windows` ≥ 1; `phases`: `windows` = 0; `minimal`: `traces` = `windows` = 0; `dense`: `snapshots` ≥ 12, `traces` = `snapshots`, `windows` ≥ 2, from its 250 ms interval and 2 s window interval over a run of at least 4 s), voids its pair |
| Statistic | per cell, the median of its n ratios with its 90% distribution-free interval: the lower bound is the (k + 1)-th smallest ratio and the upper the (n − k)-th, k the largest count with P(Binomial(n, ½) ≤ k) ≤ 0.05 (n = 16: k = 4; 24: k = 7; 32: k = 10), computed by `scripts/telemetry-stats.mjs` with no random draw |
| Verdict per cell | pass when the interval's upper bound is below 1.05 (D5's "under 5%", a strict comparison on unrounded values); fail when its lower bound is at or above 1.05; else inconclusive |
| Qualification | pass when all six cells pass and the cap was not reached; the margin is the interval, not the point estimate: the upper bound sits under the ceiling, and the median is recorded as the headroom; D5 then records 5% as the ceiling; any fail or inconclusive cell, or the cap reached anywhere in the measurement, is a user stop with every reading reported |
| Self time | `self_time_us` / ticks per cell, a lower bound |
| Idle stack (c) | per world, three reference runs with the stack started against the world's `b` reference runs; recorded, no ceiling |
| Presets, time | after the six cells, per world, `minimal`, `phases`, `dense`: 6 pairs of state `a`, the same ratio, statistic and export check, no ceiling; a preset whose lower bound is at or above 1.05 is labelled "exceeds the ceiling; opt-in" |
| Presets, storage | wire bytes per run and per hour (`bytes=` ÷ run wall × 3,600) from each preset's `a` runs; disk growth from one storage block per preset and world, separate from the pairs: with the stack started and 60 s settled, `du -sk` of `/data/prometheus`, `/data/tempo` and `/data/loki` in the container, three telemetry runs back to back, 60 s settle, `du -sk` again; the session's idle growth rate, read once from a 60 s idle window at the start, is subtracted pro rata; disk per run is the corrected growth over three, per hour scaled by run wall; every per-hour figure is labelled a repeated-short-invocation rate, since a fresh run starts a window at once and its per-run records recur, and is not the growth of one continuous hour-long run |
| Order | W1 cells, W2 cells, (c) for both, preset pairs, storage blocks; every cell completes before the next starts |
| Time cap | 3,600 s of measured process wall over every attempt: calibration, init, spread and its repeat, pairs, (c), preset pairs and storage runs; each attempt's used time is recorded and passed to the next as `TELEMETRY_USED_MS`; every run is spawned with a timeout equal to the remaining budget, a killed or failed run is charged at its full duration, and reaching the cap ends the measurement as inconclusive (Qualification row) |
| Void | a non-zero exit, a timeout, a settings mismatch or a failed export check voids its pair, which is not replaced and whose time stays charged; a voided pair enters its cell's sample as an infinite ratio, so n and the ranks are unchanged and a void can only move the verdict towards fail or inconclusive; a voided pair's cause is a defect fixed before closure |
| Projection | at n = 32: W1 pairs about 10 s in `b` and `a` and 20 s in `s` (1,280 s), W2 about 8, 8 and 18 s (1,088 s), calibration, init, spread and (c) about 130 s, preset pairs about 320 s, storage runs about 110 s: about 2,930 s; at n = 24 about 2,340 s; at n = 16 about 1,590 s. The `s` runs' 10 s flush is inside those figures; elapsed session time adds the stack's two state changes per `a` and `s` pair (about 10–20 s each, roughly 50 minutes at n = 32) and the settle waits, so about 2 to 2.5 hours |
| Expected | `standard`: every cell's median within 1.00–1.02 (F01–F02 measured 0.994–1.010 healthy and stopped; self time ≤ 0.7 µs per tick at F04 against a tick of about 0.55 ms on W2 and 100 ms on W1), `s` equal to `b` within noise on its verdict ratio, with `flush_ms` at 10,000, `abandoned` > 0, and its flush-included ratio about 3 on W1 and 3.5 on W2 (the 10 s flush over a 5 or 4 s run); (c) ≤ 1.02; `minimal` ≈ `phases` ≈ `standard`; `dense` 0–5 points above `standard`; wire bytes for `standard` about 200 KB per wall second (0.7 GB per hour), disk about 15 MB per hour in Prometheus and about 70 MB per hour in Tempo (F03's 22 MB of tick traces and F04's 50 MB of windows), `dense` about four times that |

**Measured verdict: not under 5%; the user recorded 10% as the ceiling and
flipped the default on (D5 resolved 2026-09-30).** Attempt 2,
2026-09-30, load average 3.1–5.8;
T1 = 41 (calibration 20 → 2,023 ms, 49 → 6,001 ms, 41 → 4,119 ms); W1
spread 0.0221 (n = 16), W2 spread 0.1538 (n = 32); init 201 ms (W1) and
9 ms (W2). Five cells pass; W1 `s` is inconclusive, its upper bound 1.0505
not below 1.05. No void pair, cap not reached: 2,174.1 s of 3,600 s used
over both attempts. Against Expected: W1 `a` and `s` medians sit above
1.00–1.02, (c) on W1 is 1.0881 against ≤ 1.02, and W1 `minimal`'s upper
bound is 1.0531. Under the user's 10% ceiling every cell passes: the
largest upper bound of the six is 1.0505, and of the preset `a` cells
1.0531; routine mode's ceiling is 1.10 and the default is on.

| World | Cell | n | Median | 90% interval | Verdict at 1.05 | Verdict at 1.10 | Flush-included median | `flush_ms` median (max) | `abandoned` | Self time µs/tick |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| W1 | b | 16 | 1.0063 | [0.9901, 1.0132] | pass | pass | – | 0 (0) | 0 | 87.0 |
| W1 | a | 16 | 1.0249 | [1.0170, 1.0347] | pass | pass | – | 5.5 (7) | 0 | 100.1 |
| W1 | s | 16 | 1.0335 | [1.0232, 1.0505] | inconclusive | pass | 3.2505 | 10,003 (10,006) | 12 | 119.5 |
| W2 | b | 32 | 1.0005 | [0.9851, 1.0258] | pass | pass | – | 0 (0) | 0 | 0.596 |
| W2 | a | 32 | 1.0051 | [0.9959, 1.0184] | pass | pass | – | 7 (19) | 0 | 0.660 |
| W2 | s | 32 | 0.9919 | [0.9878, 1.0085] | pass | pass | 3.4853 | 10,005 (10,011) | 10 | 0.622 |

Idle stack (c): W1 1.0881, W2 0.9982. Presets, state `a` (the `standard`
rows are the qualifying `a` cells; no preset's lower bound reaches 1.05;
per-hour figures are repeated-short-invocation rates):

| World | Preset | n | Median | 90% interval | Wire bytes per run | Wire MB per hour | Disk KB per run (prom, tempo, loki) | Disk MB per hour |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| W1 | minimal | 6 | 1.0247 | [1.0072, 1.0531] | 241,956 | 187 | 21, −3, 4 | 17.5 |
| W1 | phases | 6 | 1.0146 | [1.0005, 1.0269] | 233,904 | 196 | 24, 111, 4 | 114.6 |
| W1 | standard | 16 | 1.0249 | [1.0170, 1.0347] | 319,309 | 266 | 65, 127, 16 | 174.1 |
| W1 | dense | 6 | 1.0090 | [0.9815, 1.0202] | 992,664 | 837 | 84, 162, 32 | 230.5 |
| W2 | minimal | 6 | 0.9853 | [0.9649, 1.0206] | 781,038 | 732 | 113, −3, 4 | 103.7 |
| W2 | phases | 6 | 1.0252 | [0.9961, 1.0295] | 805,492 | 752 | 72, 111, 4 | 168.7 |
| W2 | standard | 32 | 1.0051 | [0.9959, 1.0184] | 894,750 | 836 | 71, 677, 16 | 702.4 |
| W2 | dense | 6 | 1.0171 | [0.9857, 1.0449] | 3,419,363 | 3,184 | 225, 167, 29 | 381.7 |

- Readings: [`docs/progress/readings/t21-f05.md`](../../progress/readings/t21-f05.md).

## Success Criteria

- [ ] Every binary defaults to telemetry on, the flag beats the variable
      beats the default, the thirteen test spawn sites and the parent
      comparison pass `--telemetry off`, and `make bench` follows the
      default.
- [ ] `PETRI_TELEMETRY_PRESET` selects `minimal`, `phases`, `standard` or
      `dense`, an explicit variable overrides one setting, and `run.started`
      carries the preset.
- [x] The qualifying measurement ran once under the predeclared method,
      every cell's reading is in the readings file, and the Measured verdict
      records the D5 outcome and the ceiling the user set from it (10%).
- [x] Each preset's measured time and storage cost is recorded.
- [ ] `make check` passes with no stack listening; the neutrality test and
      the parent comparison hold.

## Notes for AI Agents

- Decision: D5, verbatim in the Goal, resolved by the user on 2026-09-30:
  the measurement was not under 5% (W1 `s` inconclusive at
  [1.0232, 1.0505]); the ceiling is recorded as 10% (ratio 1.10), the
  default flips to on, 5% stays the target, and later T21 closures and any
  later feature that adds work to a run are held to 10%.
- Decision: (user, 2026-09-30, after the flip) closure benchmark runs
  export telemetry to the stack and the stored benchmark reports are still
  written as before; `make bench` no longer passes `--telemetry off`, and
  tests that start a binary still do. Reason, quoted: "The performance
  parts of the benchmark are only to catch severe regressions, I think it
  is fine to export telemetry to grafana and do the stored benchmark
  reports." This supersedes the second half of F06's D2.
- Decision: (user, 2026-09-30) in the paused stack state the verdict ratio
  subtracts `flush_ms` and the flush is bounded separately at 10 s; the
  healthy and stopped states keep the flush inside their ratio.
- Decision: (user, 2026-09-30) the qualifying measurement runs on the
  loaded host as it is, and the user accepts whatever verdict comes out;
  the outcome stopped for the user, who then set the ceiling (first
  bullet).
