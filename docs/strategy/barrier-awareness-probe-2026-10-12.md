# Barrier awareness probe (2026-10-12)

Question: on the running `v3-server` world (run `eec8aa1c…`, binary built from main at 12:16 on 2026-10-12, current defaults), the dashboard panel "Barrier bumps next to a barrier, by tick" showed creatures with a barrier reader bumping barriers less than creatures without one (gap 0.10 at ticks 7k–15k), then the gap closed (0.01 at tick 23.5k). Did barrier avoidance evolve and then get lost?

## Verdict

No. Real avoiders were not outbred: 93 of 377 sampled creatures (24.7%) steer off a wall they can see. Cutting their barrier inputs raises their lab blocked fraction from 0.00 to 0.24. The gap closed because the dashboard's reader label (`has_barrier_reader`, presence not use) now covers 80% of creatures. Two thirds of those carriers do not use the input, and in the live action logs that non-using majority is blocked more often than creatures without a reader. Whether the avoiders' lab skill pays off in the live maze is not confirmed: their live blocked rate per move is no lower than non-readers'.

**Follow-up the same day (Section 6):** why avoidance was slow to spread, and why the dashboard started trending down after tick 30k. At tick 27.6k most avoiders used per-direction edges, and those give no benefit in the live world. A second route then swept: a T11.F31 ring projection that maps the barrier ring onto the Move sinks. It took avoiders from 24.7% to 41.8% of creatures by tick 37.9k.

## 1. Dashboard series (Prometheus; 5 min windows to tick 23.5k, 3 min after)

Share of moves next to a barrier that walked into it.

| tick | readers | non-readers | gap | reader share of those moves |
| ---: | ---: | ---: | ---: | ---: |
| 2,014 | 0.53 | 0.51 | −0.01 | 5% |
| 6,852 | 0.52 | 0.62 | 0.10 | 15% |
| 8,994 | 0.49 | 0.59 | 0.10 | 26% |
| 15,190 | 0.56 | 0.66 | 0.10 | 55% |
| 20,310 | 0.58 | 0.64 | 0.05 | 73% |
| 23,510 | 0.60 | 0.61 | 0.01 | 79% |
| 29,100 | 0.57 | 0.59 | 0.02 | 81% |
| 33,510 | 0.56 | 0.60 | 0.03 | 87% |
| 36,580 | 0.53 | 0.61 | 0.08 | 90% |
| 37,250 | 0.52 | 0.63 | 0.11 | 91% |

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

## 6. Why avoidance spread slowly, and why it is spreading now

**Live traces.** I traced 120 barrier-adjacent creatures carrying a barrier reader, 10 ticks each, using the inspector's read-only tracer (`POST /v3/simulation/creature/:id/sample`).

**The default traps creatures against walls.** In 40 traces that kept the raw votes, 139 of 154 bumps happened when the Move votes tied at the top, usually the founder's four cardinals at 0.5, with at least one open direction among the tied.

- The 0.5 tie is the founder's own wiring: `Move(d) = 0.4 × food_ring(d) + 0.5`, so with no food in view all four cardinals are equal (`creature/founder.rs`).
- T11.F30 breaks ties clockwise from the heading, starting with the heading itself.
- A committed Move sets the heading whether or not the move succeeds.

So a creature that tied toward a wall picks the wall again on the next tick. In the first 80 traces, 34 creatures bumped on at least 5 of 10 ticks and 26 bumped on all 10. These stuck creatures produced 93% of the bumps. On 81% of bump ticks they also ate, so a stuck creature is a grazer pinned to a wall. Its blocked move costs the same 1.6 as a successful move.

**What escapes the trap.** Any barrier-driven difference among the tied Move sinks breaks the tie. A traced projection avoider's projection shifted each walled direction's vote by about −0.02: enough to turn the four-way tie toward the open side. Ablating one barrier family at a time shows that every avoider in the tick-38k sample (171 of 171) uses the neighbour barrier ring, not the area summary.

**The two routes.** Probe rerun on a fresh whole-world sample of 409 at ticks 37,693–38,126:

| reading | tick ~27.6k (n 377) | tick ~37.9k (n 409) |
| --- | ---: | ---: |
| avoiders, share of all creatures | 93 (24.7%) | 171 (41.8%) |
| through per-direction edges or VM reads | 71 | 50 |
| through a reachable non-zero barrier ring projection | 22 | 121 |
| barrier-reader carriers | 303 (80%) | 387 (95%) |

Live blocked rate per move (bootstrap 95% interval):

| group | tick 27.6k | tick 37.9k |
| --- | --- | --- |
| avoiders through edges or VM | 0.241 [0.16, 0.32] | 0.297 [0.20, 0.38] |
| avoiders through a projection | 0.290 [0.13, 0.45] (n 22) | **0.116 [0.09, 0.15]** |
| readers that do not avoid | 0.309 | 0.318 |
| non-readers | 0.244 | 0.313 |

In the 40 vote traces, projection avoiders bumped on 10 of 126 barrier-adjacent moving ticks (8%) and non-avoiders on 144 of 196 (73%).

**Reading.**

- The edge/VM route is perfect on the lab's single open wall but shows no detectable live advantage. That matches its flat share and its decline, and it is why the first probe found no live benefit.
- The projection route works in the live maze. It is what the dashboard's falling reader line has measured since about tick 30k.
- A plausible reason the projection route arrived late: T11.F31 projections are born with zero weight. Reaching barrier-ring-to-Move with a negative weight takes a retarget and a weight step, and the zero-weight intermediate is neutral. That is a drift wait, not a selection block. This reason is inferred; no lineage was tracked.

**Still open.** Why edge/VM avoiders fail live although they handle all 16 lab walls is not identified. Candidates: they cover only the configurations the lab presents; the lab masks plasticity; or crowding. A fixed-genome read on live barrier patches would separate these.

## Recommendation

No production change. Natural selection is now doing the work. The dashboard needs a use-based split: barrier readers whose readers actually feed an executed path to a move decision, rather than any reachable reader. That would be a T21-style telemetry feature. Open question for a later reading: why non-using carriers are blocked more often live.
