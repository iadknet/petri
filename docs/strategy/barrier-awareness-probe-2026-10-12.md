# Barrier awareness probe (2026-10-12)

Question: on the running `v3-server` world (run `eec8aa1c…`, binary built from main at 12:16 on 2026-10-12, current defaults), the dashboard panel "Barrier bumps next to a barrier, by tick" showed creatures with a barrier reader bumping barriers less than creatures without one (gap 0.10 at ticks 7k–15k), then the gap closed (0.01 at tick 23.5k). Did barrier avoidance evolve and then get lost?

## Verdict

No. Real avoiders were not outbred: 93 of 377 sampled creatures (24.7%) steer off a wall they can see. Cutting their barrier inputs raises their lab blocked fraction from 0.00 to 0.24. The gap closed because the dashboard's reader label (`has_barrier_reader`, presence not use) now covers 80% of creatures. Two thirds of those carriers do not use the input, and in the live action logs that non-using majority is blocked more often than creatures without a reader. Whether the avoiders' lab skill pays off in the live maze is not confirmed: their live blocked rate per move is no lower than non-readers'.

## 1. Dashboard series (Prometheus, 5 min windows)

Share of moves next to a barrier that walked into it.

| tick | readers | non-readers | gap | reader share of those moves |
| ---: | ---: | ---: | ---: | ---: |
| 2,014 | 0.53 | 0.51 | −0.01 | 5% |
| 6,852 | 0.52 | 0.62 | 0.10 | 15% |
| 8,994 | 0.49 | 0.59 | 0.10 | 26% |
| 15,190 | 0.56 | 0.66 | 0.10 | 55% |
| 20,310 | 0.58 | 0.64 | 0.05 | 73% |
| 23,510 | 0.60 | 0.61 | 0.01 | 79% |

Over the same window, mean genome size went from 165 to 438. Drive carriers: steal went from 3% to 33%, reproduce from 9% to 39%, and move peaked at 26% around tick 9k–11k. Raw rates are per wall-clock minute and the run slowed about 4×, so only the ratios compare across ticks.

## 2. Sample and structure

- **Sampling:** every 200² detail tile over the 1,600² world (7,008 creatures), with each tile's share of a 400-creature draw dumped immediately (`GET /v3/simulation/creature/:id`).
- **Result:** 377 dumped and 25 died before their dump, so the sample slightly under-represents creatures about to die. Ticks 27,421–27,823. The world was not paused.
- **Structure:**
  - 303 of 377 (80%) carry a reachable barrier reader, matching the dashboard.
  - All 303 also have a reachable barrier decision writer.
  - Median reader nodes per carrier: 1.
  - Carriers are not larger (median genome 481 against 507 for non-carriers).

## 3. Causal probe

- **What it compares:** each genome against a copy with its barrier inputs ablated, on the 16 calibration scenes of `wall-v1` (scale 2, lifetime 200, seed 1).
- **Ablation:** covers both `Family::NeighborBarrierRing` and `Family::AreaBarrierSummary`, through all three read paths: graph input edges, VM `ReadInput`, and T11.F31 ring projections (see side finding 1).
- **Controls:** determinism held (377 of 377 originals identical on rerun). Non-readers showed no use (0 of 74).
- **Code:** probe source is `crates/v3-lab/tests/live_barrier_use.rs`. One run takes about 42 s.

| group | n | lab blocked, sensor on | lab blocked, sensor cut | live blocked per move (moves) | snapshot occupied / barrier neighbours | 11×11 barrier density |
| --- | ---: | ---: | ---: | ---: | --- | ---: |
| avoiders (Δblocked ≥ 0.05) | 93 | 0.000 | 0.238 | 0.254 (4,664) | 0.45 / 0.91 | 0.127 |
| use, Δblocked < 0.05 | 17 | 0.196 | 0.238 | 0.277 (741) | not read | not read |
| readers not using | 193 | 0.235 | 0.235 | 0.312 (8,651) | 0.40 / 0.85 | 0.096 |
| non-readers | 74 | 0.239 | 0.239 | 0.244 (2,653) | 0.36 / 0.77 | 0.086 |

- **Avoidance effect:** Δblocked is 0.237 at the median for users, and the avoider count does not move between thresholds 0.02 and 0.10. No creature steers into walls more with the sensor on.
- **Predeclared rule:** at least 15% functional avoiders means the label is filling with non-users; under 5% means the avoiders were outbred. Avoiders are 24.7% of all creatures, about the reader share of moves at the gap peak (26% at tick 9k). That peak-era share is inferred from the gap, not measured.

## 4. Limits

- **Lab conditions:** creatures run alone in an open wall arena with the founder phenotype and plasticity masked. Every creature dies in every scene after about 50 ticks and nobody eats (side finding 2). The paired contrast is valid: same genome, same scenes, only the ablation differs. It shows "this genome steers off a visible wall", not that steering pays in the live world.
- **Live proxy:** comes from each creature's action log. `Blocked` merges barrier and occupied cells, and the denominator is all moves rather than barrier-adjacent moves.
- **Why non-users are blocked more is unexplained.** Their excess (0.312 against 0.244, about 7 standard errors) is not explained by snapshot crowding or barrier exposure, both of which are similar or lower for them. Something inherited alongside the unused reader may be bad for movement; this probe does not identify it.

## 5. Side findings for other owners

1. **`v3_lab::readings::ablate` misses ring projections.** It leaves `RingProjection.ref_idx` reads live. 12 of the 110 users are only detected once projections are ablated (`ablated_channels = 0xFF`, as `neighborhood/input_use/consumers.rs` does). T22.F03 causal readings are affected for any family read through projections.
2. **The `wall-v1` calibration predates the 2026-10-10 energy-cost tuning.** Its config digest was `2d9975…` at 809a7b48 and is now `19e743…`. The founder's score moved from +0.61 to −0.24 and the comparator's from 10.42 to 9.17. Recalibration is needed before any barrier campaign.

## Recommendation

No production change. The dashboard needs a use-based split: barrier readers whose readers actually feed an executed path to a move decision, rather than any reachable reader. That would be a T21-style telemetry feature. Open question for a later reading: why non-using carriers are blocked more often live.
