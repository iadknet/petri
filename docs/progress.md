# Benchmark Progress

The compute column reports six normalized counters against the pinned
T10.F10 epoch and the last closed T10.F11 gate report: mesh hops, VM steps,
graph relaxation, actions, and births are 0.000000%; plasticity is both-zero
and therefore `null`/ok. Wall-clock is secondary and shown as
T10.F10 / T10.F11. Historical reports did not run goal indicators, so their
lineage and memory readings are `Undefined`. The deferred indicators are
strategy count, strategy causal distinctness, evolutionary activity, adaptive
novelty, memory dependence, learning dependence, prediction dependence,
information integration, and reciprocal interaction; all are `Undefined` in
every row.

| Feature and date | Evidence | Compute vs T10.F10 / T10.F11 | Indicators |
| --- | --- | --- | --- |
| T10.F09 — 2026-09-04 (closed) | [report](progress/features/t10-f09-throughput-baseline-and-profiling-budget.json) | Counters: 0% / 0%; plasticity both-zero. Wall: -19.388810% / -4.044738%. | Persistence final population: 11=284, 22=260, 33=264; births/100 ticks: 32.888889; structure min/p25/median/p75/max/mean: 96/96/96/96/106/96.012376; lineage: Undefined; memory: Undefined; deferred: Undefined. |
| T10.F10 — 2026-09-04 (closed) | [report](progress/features/t10-f10-deterministic-benchmark-harness.json) | Counters: 0% / 0%; plasticity both-zero. Wall: 0.000000% / +19.033245%. | Persistence final population: 11=284, 22=260, 33=264; births/100 ticks: 32.888889; structure: 96/96/96/96/106/96.012376; lineage: Undefined; memory: Undefined; deferred: Undefined. |
| T10.F11 — 2026-09-04 (closed) | [report](progress/features/t10-f11-cross-process-reproducibility-of-seeded-runs.json) | Counters: 0% / 0%; plasticity both-zero. Wall: -15.990861% / 0.000000%. | Persistence final population: 11=284, 22=260, 33=264; births/100 ticks: 32.888889; structure: 96/96/96/96/106/96.012376; lineage: Undefined; memory: Undefined; deferred: Undefined. |
| T01.F11 — 2026-09-04 (closed) | [report](progress/features/t01-f11-baseline-persistence-characterization.json) | Counters: 0% / 0%; plasticity both-zero. Wall: -15.305404% / +0.815979%. | Persistence final population: 11=284, 22=260, 33=264; births/100 ticks: 32.888889; structure: 96/96/96/96/106/96.012376; lineage: Undefined; memory: Undefined; deferred: Undefined. |
| T01.F12 — 2026-09-04 (pending manual verification/integration) | [gate report](progress/features/t01-f12-goal-profile-basic-indicators-and-progress-table.json); [goal report](progress/features/t01-f12-goal-profile-basic-indicators-and-progress-table-goal.json) | Gate counters: 0% / 0%; plasticity both-zero. Wall: -15.742295% / +0.295879%. | Goal persistence final population and births: 11=5,291/134,117; 22=10,997/179,125; 33=8,130/166,410. Births/100 ticks: 7994.200000. Structure: 1/95/97/124/798/116.771071. Lineage count/entropy: 11=142/2.780730; 22=144/2.947968; 33=140/2.668014. Memory zeroed/scrambled/either: 0/0/0, fraction 0.000000 for every seed (5,291; 10,997; 8,130 creatures); fixed `rotate_left(1)` over 16 slots after the final tick. Deferred: Undefined. Goal-v1 baseline is provisional pending manual verification/integration. |
