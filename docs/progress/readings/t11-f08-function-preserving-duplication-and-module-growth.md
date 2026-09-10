# T11.F08 — function-preserving duplication and module growth: measured readings

Measured evidence relocated on 2026-09-09 from
[`docs/specs/roadmap/t11-f08-function-preserving-duplication-and-module-growth.md`](../../specs/roadmap/t11-f08-function-preserving-duplication-and-module-growth.md),
unchanged in content. The spec keeps the predeclaration, the verdict, the
mutation survivor record, and every user decision. Machine-written reports:
[gate](../features/t11-f08-function-preserving-duplication-and-module-growth.json), [goal](../features/t11-f08-function-preserving-duplication-and-module-growth-goal.json).

The tables below name the readings the predeclaration asked for; the reports
are the source for everything else. **Every number below is from the
post-remediation reruns** (both reports overwritten at `df6b444f`, after the
split exclusion was narrowed to `EnergyCurrent`). The first pass's readings
are superseded and are quoted only where they are needed for comparison,
labelled as such.

**Gate counters** (per creature-tick; previous T11.F15 / epoch T11.F04),
`make bench PROFILE=gate` **exit 0**, `severe=false` against both:

| counter | current | vs T11.F15 | vs T11.F04 |
| --- | --- | --- | --- |
| `mesh_hops` | 2.000000 | 0.000000% | +0.022955% |
| `vm_steps` | 28.044355 | 0.000000% | -0.001248% |
| `graph_relax_iters` | 1.000000 | 0.000000% | -66.650826% (T11.F06 definition change) |
| `plasticity_updates` | 0.000899 | 0.000000% | -0.221976% |
| `actions_applied` | 1.000000 | 0.000000% | 0.000000% |
| `births` | 0.001225 | 0.000000% | +5.331040% |

Gate wall 0.005006945 ms/creature-tick: +20.248117% versus T11.F15,
+19.229673% versus T11.F04, both `ok` (the +25% flag is not reached). This is
a host-load reading, not a work change: the whole `deterministic` block of the
rerun is byte-identical to the superseded first-pass gate report, which
measured 0.004238436 ms/creature-tick (+1.791398% / +0.929275%). The rerun
started immediately after a six-minute `cargo-mutants` run on the same host. Every gate counter is byte-identical to T11.F15 as well: the gate
profile is 225 ticks with 75 births, a dormant tail copy costs no VM step, and
no founder edge is affected by the narrowed split exclusion, so the simulation
trajectory is the same one.

**Goal counters** (per creature-tick), `make bench PROFILE=goal` **exit 0**,
`severe=false` against both references:

| counter | current | vs T11.F15 | vs T11.F04 (epoch) |
| --- | --- | --- | --- |
| `mesh_hops` | 2.188660 | +0.377817% | -33.287022% |
| `vm_steps` | 42.819961 | -29.320679% | -96.539913% |
| `graph_relax_iters` | 1.005620 | +0.101932% | -83.246742% |
| `plasticity_updates` | 0.091700 | **+43.861190% (flag)** | -34.498596% |
| `actions_applied` | 1.021652 | -6.102045% | -9.551899% |
| `births` | 0.007653 | -3.236819% | -6.109680% |

Goal wall 0.007124457 ms/creature-tick: +10.055289% versus T11.F15 and
-0.430642% versus T11.F04, both `ok`; total world wall time 708,666 ms
(11.8 minutes, under the 15-minute goal-profile investigation threshold).

**The `plasticity_updates` reading.** The first pass measured 0.104370 here
and `compare_against` returned `severe=true` versus T11.F15, so
`make bench PROFILE=goal` exited 3 and the crossing was escalated as the
review's single P1. That reading is **superseded**. Narrowing the split
exclusion to `EnergyCurrent` (review finding P2-2) restores every
`EnergyConsumedThisTick` and `ReproductiveReserveCurrent` split the first pass
was declining, which changes the evolved trajectory; the rerun reads 0.091700,
+43.861190% versus T11.F15, a `flag` and not a `severe` crossing, and the goal
run exits 0.

*Series context.* This counter has ranged 0.100–0.154 per creature-tick across
the goal reports from T01.F12 through T11.F14 (0.100492, 0.100492, 0.124060,
0.115330, 0.139997, 0.153630, 0.151239, 0.151239). T11.F15's 0.063742 is the
one low outlier in that series; 0.091700 sits between the two and 34.5% below
the T11.F04 epoch.

*Direct mechanism, before any hypothesis about why.*
`apply_hebbian_updates` (`runtime/plasticity/hebbian.rs`) counts one update per
input edge of every non-reward-modulated plasticity node that has inputs and an
initialized weight vector, whether or not anything reads that node. A dormant
plasticity copy therefore raises `plasticity_updates` without ever being
activated, before and after this diff. The counter measures how much plasticity
structure the surviving population carries — population composition — not how
much learning any behavior depends on.

*Hypothesis, not a measurement.* The most likely reason composition moved is
the feature working as intended: a phase-faithful copy of a plasticity module
reproduces its original when something reads it and can then diverge, so
duplicated plasticity structure survives where an appended copy used to read
its inputs in the wrong evaluation phase and be selected away. Nothing in the
diff adds an update to a fixed genome, and the split exclusion still *removes*
split opportunities on plasticity-carrying graphs. This paragraph is an
inference from the mechanism above and from `graph_relax_iters` being flat
(+0.101932%, so the rise is updates per graph visit); it is **not** measured
here.

*What the neighborhood sample cannot show.* In this rerun every plasticity-only
operator skipped all 720 trials (`DisableHebbian`, `EnableRewardModulation`,
`MutateHebbianRate`, `MutateHebbianRule`, `MutateRewardSource`,
`MutateTraceDecay`, `ToggleHebbianLamarckian`, `DisableRewardModulation` all
0/0 applied, 720 skipped), i.e. none of the 36 sampled evolved genomes carries
plasticity, while the population-level counter is 0.091700. The 12-genomes-
per-seed sample is far too small to measure the population's plasticity share,
so it neither supports nor refutes the hypothesis above. The first pass's
"2 of 36 versus 4 of 36 sampled genomes carry plasticity" composition argument
is withdrawn for that reason.

*Status.* The spec predeclared that populations differ and every counter may
move, but did not predeclare a severe allowance. No severe allowance is needed:
the rerun is a `flag`, not a `severe` crossing. No threshold was weakened and
no baseline was re-pinned; an epoch re-pin is not recommended, since the
counter sits 34.5% below the T11.F04 epoch.

**Founder neighborhood versus the predeclaration** (gate report, silent /
applied, T11.F15 → T11.F08). The rerun's founder rows are byte-identical to
the superseded first-pass gate report:

| operator | predeclared | T11.F15 | T11.F08 | met |
| --- | --- | --- | --- | --- |
| `VmCopyInstructionBlock` | 19/50 → 50/50 | 19/50 | 50/50 | yes |
| `VmCopyGeneBackwardSlice` | 27/50, 6 dead → 50/50, 0 dead | 27/50, 6 dead | 50/50, 0 dead | yes |
| `VmCopyGeneForwardSlice` | 22/50 → 50/50 | 22/50 | 50/50 | yes |
| `VmCopyInstructionBlockRemapped` | unchanged | 15/50 | 15/50 | yes |
| `CopyInternalNode` | stays 50/50 | 50/50 | 50/50 | yes |
| `CopySubgraph` | stays 50/50 | 50/50 | 50/50 | yes |
| `AddInternalGraphNode` | 50/50 or skips only on an excluded edge | 50/50, 0 skips | 50/50, 0 skips | yes |

Every other founder operator row is byte-identical to T11.F15. Single-event
silent births 84/164 (predeclared floor: not below 80/164; T11.F15 read
80/164). Founder dead births 0/208, unchanged. Founder observation 57.197 ms,
far under the 10-second cap.

**Evolved neighborhood** (goal report, pooled over 36 genomes, T11.F15 →
T11.F08 rerun): `VmCopyInstructionBlock` 483/720 with 2 dead →
**720/720, 0 dead, 0 skips**; `VmCopyGeneBackwardSlice` 370/610 with 24 dead →
**556/556, 0 dead** (164 skips); `VmCopyGeneForwardSlice` 402/608 →
**582/582, 0 dead** (138 skips) — silent on every applied trial, as
predeclared. `VmCopyInstructionBlockRemapped` stays a behavior-changing macro
(446/720 → 460/720 with 5 dead). `CopyInternalNode` 643/643 and `CopySubgraph`
643/643 (77 skips each, on sampled genomes whose graph backends have no
compute node), `CopyNode` 720/720; the two mesh slices read 716/720 with 2
dead and 712/720 with 3 dead against T11.F15's 717/720 and 718/720 — mesh
topology code is untouched by this feature, so those are sampling differences
between two evolved populations. `AddInternalGraphNode` reads 689/689 silent
with 31 skips (T11.F15: 707/707 with 13); the silent fraction stays 1.00.
`VmCopyConstantBlock` 586/586 silent with 134 skips.

Evolved single-event silence 1890/2736 (T11.F15: 1823/2736; superseded first
pass: 1862/2736). Evolved observation 779.589 ms total
(321.203 / 233.788 / 224.598 ms per seed), far under the 180-second cap.

**One predeclared expectation is missed, recorded rather than glossed.**
Evolved pooled dead births read **28/3300** against the predeclaration
"should not rise above 21/3300" (T11.F15 read 21/3300; the superseded first
pass read 19/3300). Seven extra dead births out of 3300 trials, 0.85% versus
0.64%. By applied-event count they are 18 of 2736 single-event births, 7 of
459 two-event, 1 of 87 three-event, and 2 of 12 four-event.

What can and cannot be attributed. A birth trial replays a real mutation-event
sequence, and the pooled birth tally records no per-operator attribution, so
**no dead birth here can be assigned to an individual operator** — a
multi-event birth may pair a tail copy with an unrelated event that kills the
creature. What the report does show is that all five copy operators this
feature changed (`VmCopyInstructionBlock`, `VmCopyGeneBackwardSlice`,
`VmCopyGeneForwardSlice`, `CopyInternalNode`, `CopySubgraph`) read 0 dead on
every applied operator trial, and that every operator row that does carry dead
trials is an operator this feature leaves untouched (`SwapRouteTargets` 63,
`ChangeEntryNode` 53, `MutateGateBias` 33, `RetargetNodeTarget` 28,
`RemoveNode` 9, the two mesh slices 5, `VmCopyInstructionBlockRemapped` 5,
`VmInstructionMutation` 3). The goal profile runs once and the two populations
are unpaired, so this is a one-run difference of seven births; it is stated
here because the predeclaration named the number, and closing it would need a
paired or repeated run this feature does not perform.

**Persistence and lineage versus T11.F15** (no cognition claim). Final
populations 11,627 / 12,402 / 12,171 (T11.F15: 10,786 / 11,669 / 11,714), no
extinction on any seed, plateau 12,171.430 / 12,406.938 / 12,607.104 versus
10,596.510 / 12,308.016 / 12,191.246, mean energy 66.931 / 63.387 / 59.862
versus 64.769 / 61.161 / 60.371. Births per 100 ticks 12,687.166667 versus
12,963.483333. Reachable structure min/p25/median/p75/max/mean
1/97/106/155/482/126.924144 versus 3/97/104/153/723/125.564576. Lineage
clades/entropy 211/4.374506, 191/4.027032, 207/4.365327 versus 173/4.038022,
212/4.284456, 225/4.251236. Current-memory either counts 0/1/1 versus 3/2/0;
temporal operator-state either counts 91/18/118 versus 37/11/17, persisted
outputs 8/12/17 versus 8/13/25, previous slots 0/0/0 versus 0/0/0. Generation
median/max 22/46, 22/42, 22/46 versus 23/45, 21/52, 22/44. These are one run
per side on unpaired populations; nothing here is a claim about cognition.
