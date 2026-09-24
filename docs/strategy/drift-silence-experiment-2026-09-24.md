# Behavioral silence after drift: bounded diagnostic pilot

Date: 2026-09-24. Source: `df35975f805e11324b196535d8d391569270f07b`.
Status: exploratory evidence and roadmap proposal; no executable feature or
priority change. [Results](drift-silence-experiment-2026-09-24.results.json),
[replay bundle](drift-silence-experiment-2026-09-24.replay.tar.gz),
[artificial-life research](drift-silence-alife-research-2026-09-24.md).

## Findings

The chart's low changed-birth count mixes **actionless parents** with **parents
that still act but rarely change**. Neither category establishes ecological
viability, cognition, or the cause of silence inside a module.

The native-runtime pilot took **9.483 seconds after compilation**. It reproduced
all five current Orchards checkpoint birth tallies exactly, including the
2,000-depth reading: **774 mutation-bearing births, 769 silent, five changed,
zero dead**. These are the first 20 drift lineages, the exact subset that supplies
that chart's birth reading, not a fresh population sample. Confluence shares this
assay context; it is not a replicate. Canyon was not rerun.

| Depth in birth steps | Parents all-NoOp on battery, of 20 | Parents with one action queue across all 80 executions | Mean total / executed / knockout-contributing nodes | Changed / silent / dead births |
| --- | ---: | ---: | --- | --- |
| 0 | 0 | 0 | 2.00 / 2.00 / 2.00 | 305 / 295 / 0 |
| 22 | 2 | 2 | 3.20 / 2.50 / 1.90 | 244 / 450 / 12 |
| 250 | 7 | 10 | 9.25 / 3.30 / 1.20 | 39 / 709 / 5 |
| 1,000 | 9 | 18 | 33.15 / 6.10 / 0.60 | 6 / 766 / 2 |
| 2,000 | 10 | 18 | 69.35 / 9.05 / 0.60 | 5 / 769 / 0 |

Each row has 2,000 attempted checkpoint births; zero-applied-event births are
excluded from the three outcome counts, but included in the dashboard's plotted
denominator. Node means cover the 20 birth-sampled parents, not the full 50-lineage
mesh census in the standard report. These are unconditional mutation walks,
not walks restricted to neutral changes and not populations under selection.

At depth 2,000:

- The ten all-NoOp parents supply **409 mutation-bearing births, all silent**.
  In the existing classifier, an unchanged all-NoOp signature is Silent. Dead
  requires a differing signature whose actions are all NoOp. Consequently, zero
  dead births does not demonstrate that the parents still act.
- The ten parents with a non-NoOp action supply **365 mutation-bearing births:
  360 silent, five changed, zero dead**. Only **1.37%** change on the battery.
  Actionless parents therefore explain only part of the low aggregate reading.
- Eight of those ten acting parents have exactly one action queue across all 80
  executions. This is finite-battery constancy, not proof of input independence
  over every possible state or ecology.
- Ten parents have no knockout-contributing nodes, coinciding with the all-NoOp
  group here. A knockout-sensitive node is not necessarily useful; redundant or
  conditionally useful nodes can fail this test.

## Fixed-parent targeting comparison

At depths 0 and 2,000, each of the same 20 parents received 100 proposals in each
of three arms. Every proposal used the production engine with exactly **one
requested event** (`units=1`, `per_unit_rate=1`), on a fresh parent copy. Skipped
requests were retained, not retried. This isolates immediate variation on fixed
parents; it does not change the drift history or estimate a production birth rate.

| Targeting arm at depth 2,000 | Changed / 2,000 proposals | Silent | Dead | Skipped |
| --- | ---: | ---: | ---: | ---: |
| Current executed preference, 0.9 | 13 (0.65%) | 1,987 | 0 | 0 |
| Executed preference disabled, 0.0 | 1 (0.05%) | 1,999 | 0 | 0 |
| Knockout-contributing preference, 1.0, diagnostic only | 25 (1.25%) | 1,975 | 0 | 0 |

Disabling the executed layer retains the existing reachable bias and operator
eligibility; it is not uniform over all genomic sites. The third arm substitutes
the battery's contributing set for the engine's parent-executed set, with ordinary
fallback on empty or locally ineligible sets. It uses an observation oracle and
is **not a proposed production mutation policy or a mathematical upper bound**.

All changed proposals came from parents that already acted. At the founder,
all arms give exactly the same 665 changed, 797 silent, and 538 skipped proposals:
the same two nodes are both executed and contributing. These founder observations
repeat one genotype with separate seed streams, not 20 independent founders.

The first-selected-parent-node breakdown under current targeting is:

| First selected node class | Proposals | Changed |
| --- | ---: | ---: |
| Knockout-contributing | 233 | 12 |
| Executed, not knockout-contributing | 1,161 | 1 |
| Not executed | 424 | 0 |
| No node target | 182 | 0 |

Contribution-biased targeting reaches contributing first targets in 460 proposals,
25 changed. Its improvement is consistent with increased exposure: the conditional
changed shares are about 5% in both cases (12/233 and 25/460). Operator and parent
mixtures differ, so this is a hypothesis, not a controlled within-module causal
estimate. The recorded target is the first node selected, not every modified node.
Applied-but-genome-identical events also occur: 19/2,000 under current targeting;
these cannot explain the large majority of silence. Further diagnosis must keep
global phenotype and kin-tag edits, which may be action-silent by design, separate.

The results give no basis to disable executed targeting, increase rates, remove
silent structure, or introduce a contribution-guided mutation oracle. They suggest
both weak exposure to action-influencing material and substantial remaining
silence even when such a node is selected. Threshold masking, unused fields,
state changes, preparatory neutrality, and battery blind spots remain unresolved.

## Protocol and verification

The frozen pre-run protocol is in the replay bundle. The probe resolves the
checked-in Orchards recipe over production defaults, uses the canonical V3Alpha1
founder and original walk seeds 90000–90019, refreshes executed node IDs every ten
birth steps, and uses founder-pinned 97-unit mutation supply. Checkpoint birth
seeds match the existing harness. Control seed formula is
`30000000 + depth*10000 + lineage*100 + trial`; shared seeds do not imply matching
events after different target draws. No observation changes the walk's RNG.

Work cap: one build budget of five minutes and one experiment of at most five
minutes; 20 walks to depth 2,000, 10,000 checkpoint births, 12,000 control proposals.
The run completed within the cap. Wall time is resource accounting, not a
comparative performance benchmark. Analysis uses per-parent records and makes no
population-level significance or optimum claim.

Verification passed:

- A harness test matches its 22-step replay and checkpoint births to native
  `drift::observe`, and checks same-genome silence and founder action capability.
- All five pooled checkpoint silent/changed/dead counts exactly match the
  T11.F24 stored summary; zero-event and event-bearing births sum to 2,000.
- Every control requests exactly one event; tally partitions and founder-arm
  identity hold. Native engine/runtime code was not modified.

The first build tried uncached registry versions and failed on cache permissions;
using the repository lockfile and offline resolution succeeded. Harness verification
caught a node-field typo and test-working-directory assumption before outcomes
were run; both were corrected. The panel is not selected from repeated outcome runs.

For replay, extract the bundle into `.bench-artifacts/` at the repository root,
then run its `reproduce.sh` from the root. It builds a standalone crate against
`crates/v3-core`; use the recorded revision for historical reproduction. The bundle
contains source, dependency lock, protocol, analysis, raw panel, and verification
logs. Reconstructed terminal genomes remain local under the ignored artifact folder.
The summary records the raw panel SHA-256. `make check-docs` passed at
handoff. No production defaults, baseline epochs, roadmap checkboxes, or priority
entries were changed, and nothing was committed.

## Proposed roadmap decision

**Propose one small T11 addition, provisionally F26: Mutation-Effect Attribution
and Observation Coverage. Do not choose a mechanism repair yet.** The pilot
reveals a concrete interpretation gap that does not require ancestry: the current
aggregate mixes already-actionless parents, constant acting parents, and different
target classes. Existing APIs already supply much of the first part.

Proposed placement is **after T11.F25, before T20.F01's baseline**, reusing the
existing neighborhood/drift observations and T14.F12 selected-genome sampling.
F25 already repairs fresh action-parameter targets the decoder does not consume;
its effect should be in the diagnostic baseline. This placement is a proposal,
not a change to the master priority order. T11.F10/F13 are currently earlier in
that order; moving this diagnostic ahead of them would be a separate explicit
sequencing decision. F26 should not make T20 wait for full lineage infrastructure
or a long ecological campaign.

A bounded deliverable would:

1. Preserve existing tallies and add parent all-NoOp, action-diversity, event-count,
   and first-target strata, including genome-identical applied events and an
   unresolved category. Do not call battery all-NoOp ecological death.
2. Compare a fixed selected-genome panel with the drift panel, reporting actual
   generation depth and size differences instead of implying a matched-depth study.
3. Re-evaluate a fixed sample of silent pairs on a separately named coverage
   extension: longer histories and supported contexts absent from the original
   battery, with authored positive controls. Keep original series unchanged.
4. State which suspected bottlenecks are supported, unresolved, or not observed,
   and hand them to existing owners. No automatic policy change follows.

The strongest alternative is to keep this as an ad-hoc report and add the small
reporting/coverage work to an existing planned measurement. That is preferable if
it can preserve a clear general observation owner without expanding T20.F01 from
input-use qualification into whole-brain diagnosis. A new dashboard, experiment
framework, or generic trace platform would be disproportionate.

Keep subsequent work with its existing owners:

- **T11.F12:** multistep neutral potential and robustness. A cheap parent-versus-
  silent-child second-step experiment can establish whether more extensive work
  is warranted; do not add a duplicate neutral-network feature. Its formal
  lineage-linked study still depends on T08.F02 and is unscheduled.
- **T11.F13:** mutation-rate and executed-bias comparisons under selection,
  discovery, retention, cost, and the required 500-generation targeting comparison.
  This pilot does not fulfill that comparison or its default decision rule.
- **T13.F08–F10:** whole-module specialization and retained usefulness.
- **T20:** new-input recruitment and structured refinement, qualified through
  discovery, retention, and ecological transfer. T20 success does not by itself
  explain all deep-drift silence.

Before selecting any new mechanism, the next cheap probes should be coverage of
up to 200 fixed silent pairs, a small selected-genome comparison using available
snapshots, and, only if relevant, equal-budget second-step mutation from parents
and silent children on one existing task. The research note gives bounded designs.
The present pilot supports this diagnostic direction; it cannot establish harm
in selected populations or promise evolved cognition.
