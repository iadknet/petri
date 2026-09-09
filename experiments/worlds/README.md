# Baseline worlds

`make bench PROFILE=goal FEATURE=<feature-slug>` runs the three saved worlds
below once each: Orchards in grassland with run seed 11, Canyon country with
22, and Confluence with 33. Each run is 2,000 ticks at 1600² with 10,000
production founders and production creature policies; the recipes carry only
environmental differences and pin `world.world_seed`, so the map (terrain and
fertility) is identical across run seeds and closures while food placement,
founder placement, and the run RNG still follow the run seed. Reports belong
to the `goal-worlds-v1` series and compare per world against the previous
world-set report; a recipe edit is labeled `inputs_changed` in that
comparison rather than treated as the same world. `plains.json` is `{}`, the
production default, kept as a loadable control.

Inspect a world (readings as JSON, preview as PNG):

```sh
cargo run --release -p v3-cli -- world inspect --config experiments/worlds/confluence.json --seed 33 --png experiments/worlds/previews/confluence.png
```

Run one without the goal observations:

```sh
cargo run --release -p v3-cli -- run --config experiments/worlds/canyon-country.json --seed 22 --ticks 2000 --sample-every 100
```

## What the substrate can and cannot do (read before editing a recipe)

- Founders eat food type 0 only. The Eat action's type index is an evolvable
  parameter that the founder emits as 0, and a mutation that changes it was
  not seen in 1,000 mutant births. A second food type is therefore a latent
  niche: it shapes nothing until a lineage learns to eat it, and the goal
  report's typed-eat counter is how that would show.
- An eaten cell holds zero density and only comes back by spread from a
  neighbor at or above 80% of max density, or by the recovery spawn below the
  1% floor. Small fertile patches are grazed to nothing and never return;
  large fertile patches are inexhaustible. Patch size, not fertility alone,
  sets the long-run population.
- Founders are barrier-blind (no barrier input is wired), so a blocked move
  still costs the move and the tick. With food worth little per bite that
  margin is what starves them: poor grass on open ground persists, rich grass
  among barriers persists, poor grass among barriers collapses.
- The tick-zero standing crop (coverage × density × energy per unit) sets how
  long the founders' boom lasts; every world booms to the 100,000 cap within
  100 ticks and crashes as that crop is eaten.

## Orchards in grassland (food differentiation, seed 11)

Grass is the founders' food: diffuse across 65% of the world at low density
(0.4), worth 4 energy per unit, regrowing very slowly (growth 0.05 on a
background fertility of 0.15) except in sixty meadows (fertility up to 2.0,
radius 40–90) where grazed ground returns quickly. Fruit is the latent rich
food: 15 energy per unit, placed only inside twenty-four orchards (radius
50–110, `initial_fertility_only`), regrowing fast within them, edible only by
a lineage that changes its Eat parameter. What the living population feels
is the meadow structure: the diffuse grass feeds the boom and is gone; the
plateau lives on meadows. What the report watches is whether fruit is ever
eaten.

## Canyon country (barrier topology, seed 22)

One whole-world thresholded fBm field (frequency 0.005, threshold 0.02)
makes rock masses with winding passable channels, about 47% of cells barrier
and 98% of passable cells in one component, plus a light rubble field
(3% density, cluster 2) so blocked moves are frequent in the channels too.
It is well looped, not a maze: Codex's perfect maze was a spanning tree and
collapsed to seven creatures. The single production food keeps its default
per-bite value; fertility is forty-five valley meadows (radius 30–70) over a
0.1 background, because the default 5–15-cell blobs are grazed out and half of
them sit under rock. The report's barrier-blocked move fraction and the
barrier-block rate split by barrier-reader state (barrier blocks with a
barrier neighbor over move attempts with a barrier neighbor, per state) are
the barrier-awareness readings.

## Confluence (composite, seed 33)

Bounded edges. A canyon massif in the north-east (one fBm field at four
nested thresholds, 0.02 in the core stepping to 0.32 outward, so rock density
fades into the plain), an archipelago in the south-west (a coarser field,
0.0 to 0.3), a jagged ridge line system across the middle, a scattered
boulder field, and a few rocks everywhere. Both foods from Orchards, with the
grass richer per bite (density 0.5, 5 energy per unit) because poor grass
among barriers does not persist, sixty meadows, eighteen orchards, and a
low-frequency fertility gradient that leaves some regions scarce. Regions
overlap rather than tile: meadows and orchards fall inside and outside the
rock, and the ridge crosses both.

## Applied readings at T12.F04 (2026-09-09, commit `e6757fa3`)

From `world inspect` (tick zero) and the stored closure report
(`docs/progress/features/t12-f04-baseline-world-set-goal.json`, one
2,000-tick run per world, 512.69 s for the whole `make bench PROFILE=goal`
command). Regenerate the previews with `world inspect` after any recipe edit.

| World | Passable | Largest component / passable | Type-0 fertile cells | Type-1 habitat cells | Final / minimum / plateau population | Type-1 share of applied eats | Moves blocked by barrier |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Orchards in grassland | 100% | 100% | all 2,560,000 | 468,253 (18.3%) | 8,818 / 1,273 / 7,805 | 0.031% (2,199 eats) | 0% |
| Canyon country | 52.7% | 98.1% | 1,349,524 (52.7%) | none | 7,715 / 3,202 / 8,144 | – | 15.04% |
| Confluence | 76.6% | 99.5% | 1,961,687 (76.6%) | 208,503 (8.1%) | 21,818 / 1,054 / 12,004 | 4.97% (304,166 eats) | 7.60% |

Confluence's late rise (7,083 at tick 1,600 to 21,818 at 2,000) came with
fruit reaching a 5% eat share and lineage entropy falling to 0.88 nats: one
lineage found the rich food. The per-reader-state barrier-block rates for
Canyon are recorded in the closure report's tracking block and the spec.

![Orchards](previews/orchards-in-grassland.png)
![Canyon](previews/canyon-country.png)
![Confluence](previews/confluence.png)
