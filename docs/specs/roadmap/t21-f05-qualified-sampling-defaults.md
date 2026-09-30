# T21.F05 — Qualified Sampling Defaults

**Status**: In Progress
**Last updated**: 2026-09-30
**Feature**: T21.F05
**Track**: [T21 — Run Observability and Execution Tracing](../../roadmaps/t21-run-observability-and-execution-tracing.md)

## Goal

Telemetry is on by default in `v3-cli`, `v3-server` and `v3-lab`, and the
default configuration is the named preset `standard`, whose cost against the
reference build is qualified under the 5% ceiling by one 60-minute
measurement on a default-size world and a small one with the stack healthy,
stopped and slow. Four named presets (`minimal`, `phases`, `standard`,
`dense`) select the sampling settings T21.F02–F04 delivered, each with its
measured time and storage cost. Tests that start a binary and invocations
that produce stored closure measurements pass `--telemetry off` explicitly,
so `make check` needs no Docker and closure evidence is taken in isolation.
With telemetry off, on, or on with no stack listening, a run's deterministic
output is identical.

**D5 (user decision, goal of 2026-09-30).** "If F05's qualifying measurement
puts the default under 5%, record 5% as the ceiling, otherwise stop and
report the measurements for the user's decision." The default flips to on
only in a closure whose qualifying verdict is pass; any other outcome leaves
every default off, and the feature stops for the user under the blocker rule.

## Non-Goals

- No adaptive governor: the track holds it in reserve until fixed caps are
  shown to fail, and this measurement is what shows it.
- No new signal, span, attribute or dashboard beyond the preset name on
  `run.started`; no change to any cap, queue bound or cadence semantics of
  F02–F04; no change to what a preset's settings mean.
- No CPU-time reading, no thread-count change and no second reference: the
  ceiling governs elapsed time of the telemetry build against the
  `--no-default-features` build of the same commit, as the workflow says.
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
| F06 spec | as above | Measurement commands hold export until the last timed region; `make bench` already passes `--telemetry off`; D2: closure measurement runs stay off. |
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
   `on` explicitly), `make bench` and `scripts/telemetry-parent-compare`. `make lab`, `scripts/dev.sh` and a
   hand-run assay or lab command follow the default; one whose output will
   be cited as closure evidence is started with `--telemetry off` by
   whoever runs it. `make check` passes with no stack listening and
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
   explicitly. Presets change no cap: the
   per-run caps of F03 and F04 and the queue bounds of F01 hold under every
   preset.
4. Qualification. The predeclared method in Performance and Goal Impact is
   run once, inside the 60-minute allowance, by the implementer's
   verification pass, with nothing else measuring on the host; its verdict
   applies D5 as the Goal states. No cell is rerun or extended.
5. The ceiling in force after this closure is the one recorded in the
   Measured verdict, and `scripts/telemetry-overhead` carries it as its
   routine ceiling; the routine check keeps F01's workload, states, ratio
   and 300 s cap and adopts the statistic, pair table, environment
   isolation and run timeout below, because F01's 75%-of-pairs rule at a
   fixed n cannot decide at 5% on a host whose pair ratios spread 6–12%
   (readings file). A pair count the 300 s cap cannot hold makes the
   routine check inconclusive before any pair.

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

- [ ] Preset resolution in `v3-telemetry` (`Options::from_env`,
      `WindowSettings`): the four presets, per-setting override, invalid
      refuses, `petri.preset` on `run.started`, the effective settings on
      the stderr start line, preset scrubbing in the three fixture helpers;
      unit tests first.
- [ ] `scripts/telemetry-stats.mjs` (median, order-statistic interval,
      verdict) with `scripts/telemetry-stats.test.mjs` under `make check`;
      `scripts/telemetry-overhead` gains `PETRI_OVERHEAD_MODE=qualify`
      (worlds, cells, presets, storage blocks, cap 3,600 s,
      `TELEMETRY_USED_MS` carry, the readings lines it prints), the
      environment scrub, the run timeout and the export check, and the
      shared statistic and pair table in routine mode, ceiling 1.05.
- [ ] Default flip: `resolve_switch`, `Switch::default`, the three help
      texts, the three `…_default_off` tests become `…_default_on`;
      `--telemetry off` at the thirteen spawn sites.
- [ ] Parent comparison against `8b0a158f`, T = 6908.
- [ ] The qualifying measurement, once, recorded in
      `docs/progress/readings/t21-f05.md`; D5 applied; the Measured verdict
      and the preset cost table filled in.

## Verification

- [ ] `cargo test -p v3-telemetry` -> preset resolution (each preset's
      values, override precedence, invalid refuses, default `standard`),
      `petri.preset` on `run.started` and the stderr start line's settings;
      the neutrality and measurement fixtures pass with
      `PETRI_TELEMETRY_PRESET=minimal` exported in the caller's environment.
- [ ] `cargo test -p v3-cli`, `-p v3-server`, `-p v3-lab` -> the switch
      tests: `resolve_switch(None, None)` is `On`, `--telemetry off` beats
      `PETRI_TELEMETRY=on`, `PETRI_TELEMETRY=off` beats the default; the
      neutrality test unchanged.
- [ ] Every `CARGO_BIN_EXE` spawn in `crates/*/tests` passes
      `--telemetry off`, or `--telemetry <switch>` explicitly in a
      telemetry fixture (reviewer grep against the diff); `make check`
      passes with the stack down and `PETRI_TELEMETRY` unset.
- [ ] `node --test scripts/telemetry-stats.test.mjs` -> median, interval
      ranks and verdict on fixed inputs, including the strict comparison
      at exactly 1.05 and a voided pair entering as an infinite ratio.
- [ ] `scripts/telemetry-parent-compare 8b0a158f` -> identical canonical
      output; transcript in the readings file.
- [ ] `PETRI_OVERHEAD_MODE=qualify scripts/telemetry-overhead` -> the
      readings file's qualification section (per-cell ratios, intervals,
      verdicts, flush and self time, presets, storage, measured time); the
      verdict copied to Performance and Goal Impact.
- [ ] Mutation gate: `Not applicable: observability feature` (the
      workflow's exemption; no run).
- [ ] Benchmark summary: `Not applicable: observability feature`.
- [ ] `make check`, `make check-docs`, `make roadmap-check` exit 0.

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

**Slow-state quantity (user decision, 2026-09-30).** The workflow's
ceiling is one ratio, telemetry on against the reference, measured by F01
and F06 as process wall time, flush included. With the slow state defined
as a paused stack, every export request runs to its 5 s timeout and the
end-of-run flush to its 10 s bound, which a 5 s workload cannot amortize.
The user decided option (a): in `s` the verdict ratio subtracts `flush_ms`,
and the flush is bounded separately at F01's 10 s. The healthy and stopped
states keep the flush inside their ratio. The 60-minute allowance is
unblocked.

Qualifying measurement, method fixed before any run:

| Item | Predeclared |
| --- | --- |
| Command | `PETRI_OVERHEAD_MODE=qualify scripts/telemetry-overhead`, every timed run under `scripts/bench-wait`, on the scratch stack `petri-telemetry-overhead`; the orchestrator has stopped the host's other containers, and no build, test, server or other measurement runs alongside |
| Isolation | the script removes every `PETRI_TELEMETRY*` and `OTEL_*` variable from the timed runs' environment, sets `OTEL_EXPORTER_OTLP_ENDPOINT` to the scratch stack's `http://127.0.0.1:4318` and `PETRI_TELEMETRY_PRESET` to the preset under test, and checks every telemetry run's stderr start line against the preset table (invariant 3), voiding the pair on a mismatch; the first line of each cell is printed for the readings |
| Worlds | W1, default-size: `v3-cli run --seed 7 --ticks T1 --sample-every T1` with no `--config` (the built-in default: 1600×1600, 10,000 founders); W2, small: F01's workload, `--config telemetry/overhead-world.json` (128×128, 256 founders), T = 6908 |
| T1 | calibrated once by F01's rule aimed at 5 s, starting at 20 ticks (not F01's 1,000, which is minutes on this world): three reference runs, each at the tick count the previous one scales to 5 s, the third must take 4 to 6 s; charged to the cap; printed as `- Default-world ticks T1: N` for the readings file and fixed for later closures (about 45–60 from the probe) |
| Init | three reference runs of `--ticks 1` per world; the median is an upper bound on a run's start-up share (it holds one tick: about 0.25 s, 5% of a W1 run; under 10% of a W2 run), recorded so a reader sees the most the ratio can be diluted; no correction is applied |
| Reference | the same commit built with `--no-default-features`, always run with the stack stopped |
| Spread | five consecutive reference runs per world after calibration; spread = (max − min) / median |
| Pairs (n) | spread ≤ 5%: 16; ≤ 10%: 24; ≤ 20%: 32; above 20%: the spread block is repeated once after 60 s idle (charged), and if still above 20% that world's cells are inconclusive and it runs no pair |
| Cells and pairs | per world, three cells of preset `standard`: `b` on with the stack stopped, `a` on with the stack started and ready (`scratch_stack_ready` plus 5 s before every telemetry run), `s` on with the stack paused (`docker compose pause lgtm`; the port accepts and never answers, so every export request runs to the 5 s timeout and the end flush to its 10 s bound); a pair is one telemetry run and one reference run adjacent in time, order alternating pair by pair as F01's; the reference always runs with the stack stopped, so `a` starts and stops the stack once per pair and `s` pauses, then unpauses and stops it |
| Ratio | per pair, telemetry process wall / reference process wall, F01's and F06's quantity, the end flush included, in `a` and `b`; in `s` the verdict ratio is (telemetry process wall − `flush_ms` from the stderr line) / reference process wall, the flush-included ratio recorded beside it (user decision above); the median `flush_ms`, `dropped` and `abandoned` per cell are recorded, and every cell's `flush_ms` stays within F01's 10 s bound |
| Export check | in `a`, a telemetry run whose line shows `failed`, `dropped` or `abandoned` above zero, or counts inconsistent with its preset (`standard`: `snapshots` ≥ 4, `traces` = `snapshots`, `windows` ≥ 1; `phases`: `windows` = 0; `minimal`: `traces` = `windows` = 0), voids its pair |
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

**Measured verdict.** Filled in at closure: one row per cell (median,
interval, n, verdict, `flush_ms`, self time), (c), the preset cost table,
measured time used, and the D5 outcome.

- Readings: [`docs/progress/readings/t21-f05.md`](../../progress/readings/t21-f05.md).

## Success Criteria

- [ ] Every binary defaults to telemetry on, the flag beats the variable
      beats the default, and the twelve test spawn sites, `make bench` and
      the parent comparison pass `--telemetry off`.
- [ ] `PETRI_TELEMETRY_PRESET` selects `minimal`, `phases`, `standard` or
      `dense`, an explicit variable overrides one setting, and `run.started`
      carries the preset.
- [ ] The qualifying measurement ran once under the predeclared method,
      every cell's reading is in the readings file, and the Measured verdict
      records the D5 outcome and, on pass, the 5% ceiling.
- [ ] Each preset's measured time and storage cost is recorded.
- [ ] `make check` passes with no stack listening; the neutrality test and
      the parent comparison hold.

## Notes for AI Agents

- Decision: D5, verbatim in the Goal; the flip lands only with a pass.
- Decision: (user, 2026-09-30) in the paused stack state the verdict ratio
  subtracts `flush_ms` and the flush is bounded separately at 10 s; the
  healthy and stopped states keep the flush inside their ratio.
- Decision: (user, 2026-09-30) the qualifying measurement runs on the
  loaded host as it is, and the user accepts whatever verdict comes out;
  D5 applies as written: the flip lands only on a pass, and any other
  outcome stops for the user.
