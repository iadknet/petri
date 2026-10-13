# Why fruit eating appears and fades in the live world (2026-10-12)

Question (user, 2026-10-12): in the running world, fruit eating occasionally evolves and then
quickly dies out. Is fruit growing back too slowly, or is it something else?

Answer: not regrowth. Fruit is never short. The evolved fruit eaters turn fruit into energy
they cannot spend on offspring: they sit near the 500 energy cap, live about twice as long,
and reproduce less often per tick than grass eaters. The net fitness gain is marginal, so
the habit drifts in and out. One episode that started at tick ~37,000 is still growing and
is 20 times larger than any earlier one.

Scope: run `eec8aa1c56b1abc2dc6748426a5bf57c` on the live server (port 3000), ticks 0–40,000,
main at 94d80322. Readings come from Prometheus through the stack's Grafana proxy and from
creature dumps (`GET /v3/simulation/creature/:id`) paged over the full 1600² grid in 200²
tiles. All creature readings are observational, from one lineage and one episode.

## 1. Regrowth is not the limit

- Total fruit density stayed at its ceiling (109,163) for the whole run. The largest earlier
  episode (tick ~14,700, 6 fruit eats per tick) dented it by 22 cell-units.
- Fruit grazing touched at most 0.002 % of cells (`petri_tick_food_grazed_cell_share`,
  type 1); the mean modifier never fell below 0.99999.
- A scan at tick 31,572 found no fruit cell below full density. At tick ~31,500, 436 of
  6,657 creatures (6.5 %) stood on a full fruit cell (200 energy) and none ate it.
- Mechanics agree: recovery spawns (rate 1.0) plus spread from full neighbours refill a
  bitten fruit cell in roughly 5–15 ticks.

## 2. What the episodes look like

- Before tick 37,000, three episodes: ticks ~2,700–4,400, ~14,400–15,000 and ~21,200–21,650,
  peaking at 1–6 fruit eats per tick against ~2,100 grass eats. Fails ran close to
  successes in each (cumulative 4,314 applied and 7,954 failed type-1 eats by tick ~33,000).
  Eats of types 2–5 (no such food) also occur, so the food-type parameter wanders.
- The food type is one scalar (`EatFoodType`, rounded by `decode_food_type_idx`). In the
  captured lineage it comes from 0.72 × node 1's upstream slot 5, the founder's westward
  food score that also feeds the `Move(W)` vote. The type therefore tracks heading: fruit
  is eaten on about half of westward ticks and 11–13 % of other ticks. The creatures switch
  between types; none eats only fruit.
- 32 % of a 272-creature world sample (tick ~40,000) have node 0's food-here sensor reading
  fruit rather than grass. The fruit eaters all carry it (111 of 112 at tick 38,483).

## 3. Why the energy does not become offspring

Captures at ticks 37,257–39,951 (3 → 499 fruit eaters near fruit cells):

| Group | Births per creature-tick | Reproduce attempts per tick | Median energy | Median age |
|---|---|---|---|---|
| Random world sample (tick ~38,500) | 0.0139 | 0.079 | 76 | ~41–54 |
| Fruit eaters, tick 38,483 | 0.0056 | 0.016 | 438 | 92 |
| Same genotype in the same region, ate fruit (tick 38,862) | 0.0077 | 0.032 | 331 | 123 |
| Same genotype in the same region, no fruit eaten | 0.0145 | 0.043 | 86 | 78 |

The last two rows share node 0's fruit sensor and node 1's reproduce gate (`UpstreamSlot 11`).

- **Reproduction is front-loaded and does not read energy.** Across 325 fruit eaters, reproduce
  attempts fall from 0.072 per tick at ages 0–39 to 0.006–0.009 after age 80, and births
  from 0.0105 to 0.003, while those creatures hold 300–440 energy. Standing on fruit does
  not suppress attempts (0.027 per tick on fruit-eating ticks, 0.024 otherwise). The founder
  gates reproduction on slot 1 ("can": energy and age). In a world sample of 272, only 32
  (12 %) still read slot 1; the rest read other slots, which were not traced. The extra
  life fruit buys falls in the ages where these genomes barely try to breed.
- **Most fruit energy is lost to the cap.** A full bite is 200 energy against `max_energy`
  500. About 59 % of the fruit energy the tick-38,483 eaters ate went past the cap
  (93,565 of 158,586). Bites are mostly regrowth trickles (median amount 0.041) from
  camping on the same cells.
- **Fruit eaters clump and get stuck.** In the early captures 57–67 % were on 20 or more
  blocked moves in a row, mostly blocked by occupied neighbours (their own kin). Later
  captures read 19–37 %, against 21 % in a random sample. Some stood on a full fruit cell
  eating type 0 every tick, because the type follows heading and not the cell underfoot.
- Lifetime output is roughly break-even: birth rate × the age of survivors gives ~0.95 for
  fruit eaters and ~1.1 for same-genotype non-eaters. This is a proxy, not a measured
  lifespan.

Reading: fruit is a survival windfall that the evolved reproduction circuits cannot convert
into births. A trait with near-zero net fitness, reached through a fragile route (a rewired
sensor plus a heading-coupled scalar), is gained and lost by drift and mutation; earlier
episodes went extinct. The current episode grows because fruit eaters live long enough for
a lower birth rate to still add up. Standing counts overstate its fitness: long-lived
campers accumulate even at a lower birth rate.

## 4. Options

Pressures stay on. Each option is a way for a windfall to become offspring or to stop
being wasted, not a fruit-specific aid.

1. **Recipe, live and reversible**: lower Fruit `energy_per_unit` (200 → ~50–100) or raise
   `max_energy`. This cuts cap waste. It does not fix the age-front-loaded reproduce gate,
   so expect at most a modest effect.
2. **T02.F08 Partial Bites** (queued): an evolvable leave-share lets a full creature stop
   short of the cap and leaves fruit to regrow in place. Same limit as option 1.
3. **T17.F04 Per-Type Eat Votes**: replaces the rounded scalar with one vote per food type,
   so fruit eating can be wired to the fruit-here sensor instead of riding the heading
   signal. This fixes the fragile route, not the fitness payoff.
4. **Condition-dependent fecundity (T03.F05 storage / T03.F07 litters, or a new row)**: let
   reserves buy offspring. In nature, reproductive output tracks condition: poorly fed
   animals breed later, skip events or produce fewer eggs, and stored reserves raise egg
   production (capital and income breeding; Varpe 2009, Sainmont 2014, Stephens 2014).
   Petri's evolved genomes decouple the reproduce decision from energy (88 % rewired), so
   no rich food niche can pay off. This is the lever that matches the diagnosis; it needs
   its own research note and spec, and production goal evidence before any default changes.

Recommendation: watch whether the current episode holds before acting. If it collapses,
option 4 is the root fix and option 1 is a cheap live probe of the cap-waste share. Options
2 and 3 are already on the roadmap and help, but neither makes fruit pay.

## Limits

- One lineage, one episode, observational. Fruit eaters differ from non-eaters in
  location as well as diet. Tile-edge creatures were classified using fruit from only
  part of their neighbourhood.
- What node 1's rewired reproduce slots (11, 7, 15, 6, …) carry was not traced.
- Earlier episodes were read only through Prometheus; no creature from them survived.

## Sources

- Levins fitness sets and environmental grain (scanned, not decisive here): [Rueffler et al. 2006](https://www.mabs.at/fileadmin/user_upload/p_mabs/Rueffler06AmNat.pdf).
- Diet breadth (scanned, not decisive here): [Pianka, foraging tactics](https://www.bio.utexas.edu/faculty/pianka/foraging.html).
- Capital and income breeding: [Varpe et al. 2009](https://bio.uib.no/te/papers/Varpe_2009_The_adaptive_value.pdf),
  [Sainmont et al. 2014](https://bio.uib.no/te/papers/Sainmont_2014_Capital_versus_income_breeding.pdf),
  [Stephens et al. 2014, "Capital and income breeding: the role of food supply"](https://research-repository.st-andrews.ac.uk/handle/10023/5333).
