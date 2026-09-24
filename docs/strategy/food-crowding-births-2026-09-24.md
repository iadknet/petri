# Food Coverage and Founder Births (2026-09-24)

Investigation of the `founder_only_trajectory_digest_is_pinned` movement at
`a3f26092` (`docs/specs/fix-food-test-fixtures.md` row 13), on main at
`a26f17f1`. No production, test, or roadmap change.

## Conclusion

More food does not give fewer births. The pinned drop (323 → 199 births,
extinction tick 890 → 448) is one draw from a distribution centered on zero:
across 100 paired seeds of the same fixture, coverage 1.0 minus coverage 0.54
is **+5.8 births (sd 148, se 14.8)**, with 1.0 ahead on 58 of 100 seeds. The
pin difference (−124) is 0.84 sd. Coverage changes which cells are seeded, so
the trajectory diverges from tick 0 and any perturbation moves the pin by about
that much; turning off occupancy depletion reverses the sign at seed 2026. The
crowding hypothesis fails on its own metrics: peak occupancy (0.146 vs 0.146),
reproduce rejection share (0.49 vs 0.50), moving share (0.796 vs 0.797), and
deaths (all starvation, none at age 200 or over at 32²) do not depend on
coverage. Every arm has the same boom-and-bust, whatever the coverage: food is
gone by tick ~60 at 32² and in the gate shape, and by tick ~95 at 64² with 10
founders. Grazing depth sets the fixture's viability (off: births ×4.3,
surviving to tick 2,000 goes from 3% to 76–83% of seeds). About half of all
reproduce attempts are rejected because the target is occupied. That comes
from the founder brain, not from food: an above-gate founder never moves, and
it aims North whenever its food ring ties. The same holds at 64² and in the
128² gate shape. The ~14% peak occupancy comes from founder dynamics and does
not vary with coverage at 32², 64², or 128². At goal scale the population cap
binds before occupancy gets that high, so crowding is, if anything, weaker
there. No standard world shows a coverage-driven crowding regime. No action is
warranted; the re-pin stands.

## Question

After `a3f26092` the founder-only viability fixture (`viability.rs:49`
`viability_config`, 32², 10 founders, `mutation.per_unit_rate = 0`, seed 2026,
2,000 ticks) seeds food at coverage 1.0 instead of the effective default 0.54.
Births fell 323 → 199, extinction moved 890 → 448, and `RejectedInvalidTarget`
rose 272 → 995. Hypothesis to test: with food everywhere founders do not
disperse, neighbors fill, reproduction is blocked by crowding, and the lineage
starves or ages out. Alternatives: energy accounting, grazing/regrowth, founder
brain gates, age/death causes, world size, seed.

## Method

- Probe: an uncommitted `#[ignore]` integration test in `crates/v3-core/tests/`
  (deleted before commit), run with
  `cargo test --release -p v3-core --test <probe> -- --ignored --nocapture`.
  It builds `viability_config()` exactly (default config, 32², 10 founders,
  `types[0]` coverage/density overrides, `per_unit_rate = 0`) and calls
  `seed_simulation` + `run_tick`, stopping at extinction.
- Recorded per run: births (`reproduction_actions_spawned_total`), deaths by
  cause (`stats.mortality`), extinction tick, reproduce rejections by reason and
  invalid-target cause, per-creature rejection counts, energy and age every 50
  ticks, successful moves and their direction (position diff with wrap), the
  direction of each birth relative to its parent, peak occupied fraction, the
  first tick total food falls below 5% of cells ("crash"), and parent energy
  right after each birth.
- Arms (32² unless stated): coverage 0.3 / 0.54 / 0.7 / 0.85 / 0.95 / 1.0 at
  density 1.0; coverage 1.0 at density 0.54 (same initial total as 0.54 × 1.0,
  but uniform); grazing off and occupancy depletion off, each at 0.54 and 1.0;
  64² with 10 and 40 founders at 0.54 and 1.0; bench gate shape (128², 256
  founders, default mutation, coverage forced, `normalize()`, as
  `v3-cli/src/bench/profiles.rs:139` and `:276`) at 0.54 and 1.0, 75 and 400
  ticks.
- Seeds: 2026 (the pin) for every 32² arm; seeds 1–100 paired across all ten
  32² arms (1,000 runs); seeds 1–30 for the 64² arms; 11 / 22 / 33 for the gate
  shape.

## Results

### Pin reproduction, seed 2026

Both pins reproduce exactly.

| Metric | cov 0.54 | cov 1.0 |
| --- | ---: | ---: |
| Births | 323 | 199 |
| Extinction tick | 890 | 448 |
| Reproduce attempts / `RejectedInvalidTarget` | 595 / 272 | 1,194 / 995 |
| Invalid-target cause | Occupied 267, Contention 5 | Occupied 994, Contention 1 |
| Rejections by the top 5 creatures | 113 of 272 | 489 of 995 (127, 107, 103, 82, 70) |
| Rejections before tick 100 | 241 | 530 |
| Deaths: lifecycle_decay / action_move / genome_carrying | 229 / 101 / 3 | 145 / 60 / 4 |
| Mean age at death; deaths at age ≥ 200 | 54.5; 0 | 58.9; 0 |
| Peak occupied fraction | 0.146 | 0.146 |
| Food-crash tick; births by then | 59; 95 | 58; 97 |
| Mean energy, ticks 1–50 / 51–100 / 101–150 | 51.7 / 26.1 / 9.6 | 52.3 / 27.7 / 19.2 |
| Successful moves, share North | 14,678, 83% | 8,535, 84% |
| Births aimed North | 213 of 323 (66%) | 145 of 199 (73%) |

Seed 2026 coverage sweep, births / extinction: 0.3 → 179 / 179,
0.54 → 323 / 890, 0.7 → 182 / 186, 0.85 → 187 / 217, 0.95 → 186 / 216,
1.0 → 199 / 448. It is not monotone; 0.54 is the outlier.

### 100 paired seeds, 32²

| Arm | Births mean | Median [IQR] | Survive 2,000 | Extinction median | Rej / attempt | Peak occ | Crash tick | Births by t100 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| cov 0.3 | 230 | 176 [167, 264] | 4 | 304 | 0.44 | 0.141 | 64.2 | 163 |
| cov 0.54 | 241 | 190 [179, 279] | 3 | 398 | 0.49 | 0.146 | 63.0 | 173 |
| cov 0.7 | 226 | 186 [179, 252] | 0 | 247 | 0.50 | 0.147 | 61.3 | 176 |
| cov 0.85 | 240 | 188 [182, 267] | 3 | 290 | 0.51 | 0.147 | 60.3 | 177 |
| cov 1.0 | 247 | 191 [183, 284] | 4 | 302 | 0.50 | 0.146 | 59.9 | 177 |
| cov 1.0, density 0.54 | 229 | 190 [181, 254] | 2 | 254 | 0.52 | 0.146 | 61.1 | 176 |
| grazing off, cov 0.54 | 1,029 | 1,123 [1,012, 1,176] | 76 | >2,000 | 0.37 | 0.202 | 67.6 | 235 |
| grazing off, cov 1.0 | 1,034 | 1,097 [1,002, 1,161] | 83 | >2,000 | 0.38 | 0.199 | 66.9 | 235 |
| depletion off, cov 0.54 | 244 | 190 [180, 293] | 1 | 370 | 0.50 | 0.146 | 62.7 | 174 |
| depletion off, cov 1.0 | 239 | 191 [185, 260] | 1 | 318 | 0.55 | 0.146 | 60.1 | 177 |

Paired births difference (arm B − arm A, same seed):

| A → B | n | Mean Δ ± se | B fewer / more | B extinct earlier |
| --- | ---: | ---: | ---: | ---: |
| cov 0.54 → 1.0 | 100 | +5.8 ± 14.8 | 42 / 58 | 45 |
| cov 0.54 → 0.7 | 100 | −15.2 ± 12.5 | 53 / 47 | 59 |
| cov 0.54 → 0.85 | 100 | −1.0 ± 13.9 | 44 / 55 | 48 |
| cov 0.54 → 0.3 | 100 | −10.8 ± 13.4 | 60 / 37 | 53 |
| cov 0.54 dens 1.0 → cov 1.0 dens 0.54 | 100 | −11.9 ± 12.4 | 48 / 48 | 58 |
| grazing off: cov 0.54 → 1.0 | 100 | +5.4 ± 33.6 | 52 / 48 | 14 |
| depletion off: cov 0.54 → 1.0 | 100 | −5.2 ± 11.5 | 41 / 59 | 47 |
| cov 0.54: grazing on → off | 100 | +787.5 ± 28.0 | 0 / 100 | 6 |
| cov 1.0: grazing on → off | 100 | +787.1 ± 25.7 | 2 / 98 | 5 |

### Larger world and the gate shape

| Arm | n | Births mean | Survive 2,000 | Rej / attempt | Peak occ | Paired Δ births 0.54 → 1.0 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 64², 10 founders, cov 0.54 / 1.0 | 30 | 1,516 / 1,471 | 12 / 7 | 0.47 / 0.48 | 0.126 / 0.122 | −46 ± 137 (15 fewer / 15 more) |
| 64², 40 founders, cov 0.54 / 1.0 | 30 | 1,546 / 1,658 | 7 / 11 | 0.45 / 0.47 | 0.141 / 0.141 | +112 ± 135 (12 / 18) |

Gate shape (128², 256 founders), births at tick 75 for cov 0.54 / 1.0:
seed 11 2,301 / 2,453; seed 22 2,338 / 2,384; seed 33 2,341 / 2,395. Peak
occupancy is 0.140–0.146, the food crash comes at tick 53–57, the population
falls to 10–250 by tick 150–400 in all six runs, and `Occupied` is 45–50% of
attempts. At 400 ticks: 2,844 / 3,032, 3,035 / 2,782, 2,879 / 3,025.

The only monotone coverage signal is small and points the other way: more
initial food gives slightly more early births (163 → 177 by tick 100 from 0.3
to 1.0) and a slightly earlier crash (64.2 → 59.9).

## Mechanism

**Boom and bust (all coverages).** Ten founders grow to ~150 on 1,024 cells
(peak occupancy 0.146) and eat the standing crop by tick ~60. A bite empties
the cell (`kernel/ordinary_food/state.rs:147-150`) and multiplies its grazing
modifier by 0.5, floored at 0.05 (`grazing.rs:11-12`; defaults
`config/simulation.rs:125-136`), recovering 1/1000 per tick (`grazing.rs:27-28`).
An empty cell does not grow on its own (`ecology.rs:449`, `source <= 0.0`). It
is refilled only by spread from a neighbor at or above 0.8 density
(`ecology.rs:475`, spread 0.25 × growth, both scaled by the grazed fertility),
or by recovery spawns once the type's total drops under 1% of capacity
(`ecology.rs:530`; defaults `config/simulation.rs:29-41`). After the crash,
regrowth under the bitten modifier is too slow. Every death is starvation:
`lifecycle_decay` or `action_move` is the first exhausting sink. Mean age at
death is 52–59. At 32² no creature reaches the age-cost cap of 200
(`config/simulation.rs:511-520`); at 64² and in the gate shape a handful do
(at most 7 of ~3,000 deaths). Initial coverage does not change this. The
standing crop is gone by tick 60 either way, and extinction then depends on
which few survivors find regrowth. The Orchards goal case
(`docs/strategy/orchards-collapse-2026-09-17.md`, step 2) shows the same deeper
grazing trough at goal scale.

**Occupied rejections come from the founder brain.** The decision graph votes
`Eat = f − 2g`, `Move[d] = 0.5 + 0.4·ring[d] − 2g`, and
`Reproduce[d] = g·(0.5 + 0.4·ring[d])` (`creature/cgp_founder.rs:144-153`).
Here `g` is 1 when energy is above 0.16 × `max_energy` = 32 and age is at
least 20 (`creature/founder.rs:49-50`, `config/simulation.rs:398-400`). While
`g = 1`, Eat and every Move vote are negative, so the founder never moves or
eats. It aims at the argmax of its food ring, and ties go to the lowest index,
North (`cgp_founder.rs:123-125`). The ring ties whenever it is uniform: all
full at startup at coverage 1.0, and all empty after the crash at any
coverage. That is why 83–84% of moves and 66–73% of births point North in both
arms. The newborn lands on the parent's North cell, and a parent still above
32 retries North into it. The target is invalid when occupied
(`kernel/world.rs:209-210`, `simulation/actions/reproduction.rs:140-148`).
The rejection returns before any reproduce charge (`reproduction.rs:171-174`),
and the failed-action penalty ramps up from 0 over 62,680 ticks
(`config/simulation.rs:592-600`). The parent loses only 0.5 decay per tick and stays stuck until it falls below the gate. In the pin
run, every one of the 995 rejections was a stationary tick. Five creatures
account for 489 of them, so the jump to 995 is a few boxed-in founders on that
one trajectory. Over 100 seeds the rejection share is 0.49 vs 0.50.

**Why the pin moved.** `seed_density` shuffles candidates and takes the first
`round(coverage × n)` (`ecology.rs:221-285`). A coverage change keeps the RNG
draw count but moves which cells hold food. From tick 1 the founders take
different paths, and the post-crash extinction lottery lands differently.
Other perturbations move the same pin just as far (seed 2026, occupancy
depletion off: 0.54 → 191 / 353, 1.0 → 378 / 1,181).

## Relevance to standard worlds

| World | Food types (coverage / density) | Size, founders | Source |
| --- | --- | --- | --- |
| Default / Plains (`plains.json` is `{}`) | one type 0.54 / 1.0 | 1600², 10,000 | `config/simulation.rs:68-82`, `:340-350`, `:928-931` |
| Canyon country | default type 0.54 / 1.0 under PoissonBlobs fertility | 1600², 10,000 (goal) | `experiments/worlds/canyon-country.json` (no `types`) |
| Orchards in grassland | Grass 0.65 / 0.4; Fruit 0.85 / 1.0, `initial_fertility_only` | 1600², 10,000 (goal) | `experiments/worlds/orchards-in-grassland.json:9-26` |
| Confluence | Grass 0.65 / 0.5; Fruit 0.85 / 1.0, `initial_fertility_only` | 1600², 10,000 (goal) | `experiments/worlds/confluence.json:189-205` |
| Bench gate profile | every type forced to 1.0 | 128², 256, 75 ticks | `v3-cli/src/bench/profiles.rs:139-152`, `:284-289` |
| Bench goal profile | recipe values (`food_coverage: None`) | 1600², 10,000, 2,000 ticks | `profiles.rs:156-170` |

No standard world uses coverage 1.0 except the gate profile, and there
coverage 1.0 gives 2–6% more births at tick 75 than 0.54 (three seeds). Peak
occupancy of ~0.14 comes from the founder population itself: it is reached at
32², 64², and 128² at both coverages. At 1600² the Orchards boom stops at the
100,000 population cap (orchards-collapse note, step 1), which is about 0.04
of the cells, so the cap binds before that occupancy. The North-tie rejections
happen wherever the ring is uniform. That includes every post-crash trough, so
they are present in all worlds regardless of coverage. The finding does not
change any standard-world reading.

## Options

No action is warranted on food coverage, and the `a3f26092` re-pin stands.
Two observations, not proposals:

1. The founder-only pin is a trajectory digest. Any change to seeding order
   can move its birth count by about ±150 (1 sd of the paired difference), so
   a pin movement alone is not evidence of a behavior change.
2. The fixture's viability depends mainly on grazing depth and the post-crash
   lottery, not on initial food. This matches the Orchards finding.

## Limits

- Mutation off (the pin's setting). The multi-seed arms use the pin's
  32²/10-founder shape; the 64² and gate-shape arms have 30 and 3 seeds.
- The goal worlds were not re-run at 1600². Relevance rests on recipe values,
  the gate-shape runs, and the committed Orchards investigation.
- Birth direction was inferred from the newborn adjacent to a parent whose
  offspring count rose. Move direction was inferred from position change.
  Neither reads the action log.
- Rust 1.93.0, macOS aarch64, release profile. The release runs reproduced
  the pinned test values exactly.
