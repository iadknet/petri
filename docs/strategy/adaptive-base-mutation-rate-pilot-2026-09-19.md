# Age-10 baseline and mutation-rate pilot

**Date:** 2026-09-19
**Code:** `45c8063722fa3aa509a32d523f5e477e9fcd3853`
**World seed:** `5786586509898421`
**Simulation seed:** `11`
**Horizon:** 6,000 ticks on the full 1,600 × 1,600 world

## Decision

Use `world-recipe-extended-age-drain-age10-baseline.json` as the next base-world candidate.

It differs from the supplied `world-recipe-extended-age-drain-reduced-copy-growth.json` in one behaviorally meaningful setting: `energy.age_cost.max_multiplier` is 10 instead of 305. Keep the per-unit mutation rate at `0.005` for now. The tested half and quarter rates reduced genome growth, but did not improve population persistence or preserve more founder lineages through tick 6,000.

This is a base candidate, not a long-run certification. It has one full-scale simulation seed at 6,000 ticks. The next decision experiment should replicate it across seeds and extend it past the earlier 15,000-tick failure horizon without changing more settings.

## Evidence carried forward

This decision uses the repository's earlier worlds, studies, and task records rather than treating the supplied recipe in isolation.

- The saved baseline-world study established that large, renewable patches can sustain post-boom populations and that the second food is a latent niche rather than guaranteed opportunity. The older Orchards run reached 79,035 creatures at tick 8,500 under a fixed mutation supply of about 0.55 events per birth. See [the baseline-world description](../../experiments/worlds/README.md) and [the per-unit supply study](per-unit-mutation-supply-research-2026-09-14.md).
- The earlier long-simulation task found 53,491 living creatures at tick 52,974 and median generation 561 under the older mutation regime. It also found real behavioral variation but no demonstrated second-food policy. This is evidence that Petri can sustain evolution and that food variety alone does not create adaptation.
- The age-drain investigation found that lowering maximum age cost to 10 preserved late breeding in all eight fresh small-world seeds, whereas the control did so in six. It also found that mutation-disabled founders could still fail, so mutation load was not sufficient as the primary explanation. See [the population-decline mechanism study](population-decline-mechanisms-research-2026-09-19.md).
- The copy-growth study found that reducing large structural copies slowed genome expansion but did not establish an extinction rescue. See [the genome/mutation balance study](genome-mutation-balance-research-2026-09-19.md).

## Controlled full-world comparison

All four runs used the same full world, world seed, simulation seed, reduced large-copy weight of 25, zero executed-target bias, relaxed genome costs, founder profile, food, fertility, barriers, and population settings.

| Arm | Age max | Per-unit mutation rate | Population at 6,000 | Births | Mean genome | Mean generation | Applied events / birth | Surviving founder lineages | Largest lineage |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Supplied source world | 305 | 0.005 | 4,361 | 286,661 | 186.4 | 130.3 | 0.740 | 39 | 49.3% |
| **Age-10 baseline** | **10** | **0.005** | **23,040** | **578,186** | **160.5** | **55.3** | **0.696** | **132** | **6.3%** |
| Half mutation | 10 | 0.0025 | 20,719 | 581,703 | 146.4 | 60.5 | 0.323 | 97 | 6.1% |
| Quarter mutation | 10 | 0.00125 | 21,434 | 590,158 | 127.6 | 61.6 | 0.149 | 84 | 6.4% |

![Full-world population and genome-size trajectories](../../.bench-artifacts/mutation-rate-2026-09-19/trajectory.svg)

The age correction is the large effect. Relative to the exact supplied world, the age-10 baseline had 5.3 times the living population, 3.4 times as many surviving founder lineages, and a far smaller dominant-lineage share at tick 6,000. Its census stayed near 23,000–24,000 from tick 1,000 onward. The supplied world fell from 14,552 at tick 1,000 to 4,361 at tick 6,000.

Lower mutation produced a dose response in genome size, but no persistence advantage:

- Halving the rate reduced mean genome size by 8.8%, final population by 10.1%, and surviving founder lineages by 26.5% relative to the age-10 baseline.
- Quartering the rate reduced mean genome size by 20.5%, final population by 7.0%, and surviving founder lineages by 36.4%.
- The half and quarter arms gained 341 and 1,651 net creatures during ticks 5,000–6,000, while the baseline lost 1,063. All three age-10 arms oscillated, so one interval does not establish a long-run rate advantage.
- Fruit supplied 0.0005% of cumulative food energy in the baseline, 0.0069% in the half-rate arm, and 0.0053% in the quarter-rate arm. These are isolated events, not evidence of a retained fruit-use adaptation.

## Interpretation

The supplied age multiplier is the immediate fragility in this world. At the cap it multiplies action costs by 305, including movement and reproduction. That converts aging into a severe, broad physiological penalty and concentrates the population into fewer lineages. Reducing it to 10 preserves turnover and lineage breadth without removing aging.

The mutation-rate hypothesis was reasonable because the current rule scales event count with genome size. The earlier long-lived system supplied about 0.55 events per birth, while a 400–600-unit genome at rate `0.005` would receive roughly two to three events. In the corrected world, however, reduced copy growth and selection held the mean genome to 160 units at tick 6,000. The observed cumulative supply was only 0.696 events per birth, already close to the older long-lived regime. Halving or quartering the rate pushed supply well below it. The quarter-rate rationale therefore does not fit this corrected genome-size regime.

The evidence supports keeping `0.005` in the base world. A modest reduction such as `0.004` remains a defensible future arm, but the current pilot gives no reason to prefer it before the age-10 baseline is replicated and run longer.

## Options considered

1. **Age 10 with mutation rate 0.005 — selected.** Best population stability and founder-lineage retention in the controlled pilot; event supply remains close to the older long-lived regime.
2. **Age 10 with mutation rate 0.0025 — retain as an experimental arm.** Smaller genomes and slightly more fruit use, but lower census and fewer lineages.
3. **Age 10 with mutation rate 0.00125 — reject as the base.** Strongest genome-size control, but no population benefit and the fewest surviving lineages.
4. **Keep age 305 and tune mutation — reject.** The exact source-world run shows strong population decline and lineage concentration before mutation-rate tuning can solve the broader physiological pressure.

## Limits and next experiment

The mutation comparison has one seed and ends at tick 6,000. It cannot estimate extinction probability or prove indefinite stability. Founder lineage IDs also understate within-lineage genetic diversity.

The next experiment should keep the selected recipe fixed, use paired seeds, and run through at least tick 15,000, preferably 20,000. Predeclare population replacement, births and deaths by interval, surviving lineage count, dominant-lineage share, genome size, and retained food-use innovations as endpoints. Only after that confirmation should mutation rate be revisited.

Raw trajectories and the plot are in `.bench-artifacts/mutation-rate-2026-09-19/`. Machine-readable endpoint results are in [the companion results file](adaptive-base-mutation-rate-pilot-2026-09-19.results.json).
