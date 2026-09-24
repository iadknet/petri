# Unfinished roadmap audit after T19 and T20

**Date**: 2026-09-23
**Scope**: Every track with unchecked executable features, reviewed by three
subagents and reconciled by the parent agent against the current roadmap.

## Result

Retain all unfinished feature IDs. No entire feature is superseded by T19's
implemented execution model or T20's planned input mechanisms. Remove stale
requirements and repair ownership, prerequisite and evidence-gate inconsistencies.
The existing uncommitted T11/T13/T17/T18 cleanup is preserved.

T19 changes how brains execute and select actions. T20 qualifies input access and
inherited refinement, with learning conditional on evidence. Neither supplies
new ecological pressures, whole-module specialization, a modular founder,
memory-motif discovery or general cognition proof. Similar discovery/retention
language in different tracks does not make their measured outcomes identical.

## Per-track dispositions

| Track | Disposition of unfinished work |
| --- | --- |
| [T01 — Experimental Science](../roadmaps/t01-experimental-science-and-causal-evaluation.md) | Retain. Deferred causal evidence and the activity baseline are distinct from T20's bounded engineering qualification. |
| [T02 — Environmental Dynamics](../roadmaps/t02-environmental-dynamics.md) | Retain. Add F01 explicitly to F06's prerequisites: the seasonal-memory campaign lost its seasons dependency when grazing F04 was decoupled. |
| [T03 — Functional Traits](../roadmaps/t03-functional-traits-and-metabolism.md) | Retain. Bodily perception, metabolism and life-history tradeoffs differ from controller input wiring. |
| [T04 — Population Structure](../roadmaps/t04-population-structure-and-diversification.md) | Retain. Ecotypes, habitat connectivity, migration and eventual isolation evidence remain separate ecological work. |
| [T05 — Biotic Interactions](../roadmaps/t05-biotic-interactions-and-coevolution.md) | Retain. Predation, parasitism and coevolution create encounters and outcomes that T20 does not supply. |
| [T06 — Niche Construction](../roadmaps/t06-niche-construction-and-ecological-inheritance.md) | Retain, including the requested four-slot designs. Update the pending F01/F02 specs to votes, effort costs, effective per-type nutrition and the current benchmark contract. |
| [T07 — Communication](../roadmaps/t07-communication-and-social-evolution.md) | Retain. Clarify that identity context uses carried tags and phenotype cues, never invisible lineage IDs. Signal mechanisms remain distinct from T16's existing-cue repair. |
| [T08 — Heredity](../roadmaps/t08-heredity-and-recombination.md) | Retain. Remove F06's deferred T04.F05 proof dependency while preserving bounded compatibility checks and the later-work restriction. Compute mutation supply from current genome size, not the historical 111-unit founder. |
| [T09 — Cognition](../roadmaps/t09-cognition-and-learning.md) | Retain. Route defects to their existing owners instead of automatically creating T11 work. F03 remains the native-plasticity comparator for conditional T20.F14, not a gate on inherited input improvements. |
| [T10 — Experiment Infrastructure](../roadmaps/t10-evolutionary-scale-and-experiment-infrastructure.md) | Retain F01–F08 as deferred proof infrastructure. Retire the full-report-in-Git instruction in favor of T15.F01; future campaign custody remains separate. |
| [T11 — Genotype–Phenotype Map](../roadmaps/t11-brain-genotype-phenotype-map.md) | Retain F10/F12/F13/F20/F24/F25; F11 remains conditional and unscheduled. Remove stale clock/single-visit language and separate F13's rate arms from its retained targeting comparison. |
| [T12 — World Composition](../roadmaps/t12-world-composition-and-baseline-worlds.md) | Retain F05. Narrow the completed recipe criterion to procedural recipes; manual-overlay persistence is still unfinished. |
| [T13 — Module Recruitment](../roadmaps/t13-neutral-module-recruitment.md) | Retain F08–F10. Their outcome is useful new modules, not new input connections. Required negative/inconclusive F08/F09 verdicts must block affected downstream rows and remove them from dispatch priority. |
| [T14 — Telemetry](../roadmaps/t14-runtime-telemetry-and-report-integrity.md) | Retain F09. Clade persistence is distinct from retention of a particular new module or input contribution. |
| [T16 — Cost and Cue Fidelity](../roadmaps/t16-cost-and-cue-fidelity.md) | Retain F02/F03. Vote selection does not remove failed-action penalties or ancestry-oracle sensing. Keep the explicit scoped identity-bank design. |
| [T17 — Brain Boundaries](../roadmaps/t17-brain-boundary-evolvability.md) | Retain F03/F04. Fractional theft and per-type Eat votes remain absent. Check only the introspection criterion already verified by completed F02; keep the third-food-type trigger for F04. |
| [T18 — Founder Architecture](../roadmaps/t18-founder-architecture.md) | Retain F01/F02 under the existing revised vote-based comparison. Both remain unscheduled; founder wiring is not evolved recruitment. |
| [T20 — Input Evolvability](../roadmaps/t20-input-evolvability-and-structured-variation.md) | Retain unchanged. Graph inheritance, later VM support and conditional learning already have distinct deliverables and evidence gates. |

## Removed requirements and corrected boundaries

- The pending [caching spec](../specs/roadmap/t06-f02-food-transport-and-caching.md)
  no longer makes stored food ignore the type's effective `energy_per_unit`.
  The [current food contract](../reference/v3-world-grid-spec.md#5-food-substrate-semantics)
  permits per-type overrides with a shared fallback. Carrying preserves that
  identity and nutrition.
- The pending [carrying spec](../specs/roadmap/t06-f01-material-carrying-and-barrier-construction.md)
  no longer requires an extra failure penalty or an obsolete decoder. It extends
  T19's action interface; T16 still owns global penalty retirement. Both pending
  specs drop the obsolete second goal-run requirement and use the current
  [benchmark artifact contract](../benchmark-artifacts.md).
- T08.F06 no longer waits for the entire deferred transplant/replay proof chain.
  Its foundation, heredity and diagnostic prerequisites remain, as do bounded
  compatibility, viability, retention and cost checks. Confirmatory heredity
  evidence stays in F09. No mating feature is newly scheduled.
- T11's current eligibility clock is world-tick decay plus per-visit activity,
  under the [Graph contract](../reference/v3-graph-backend-spec.md#eligibility-trace-clock-and-activity-phases-0-and-1).
  Old fixed per-birth supply and single-visit assumptions are historical.
  F13 retains the user's paired targeting obligation at a fixed per-unit rate;
  it does not confound that contrast with its rate comparison.
- T13 adopts T20's existing blocked-row/priority convention for failed required
  qualification. F07's honest baseline nulls alone do not block F08. No result
  in this audit changes an actual qualification verdict or the priority order.

[T19](../roadmaps/t19-mesh-action-selection-and-live-state.md) has no unfinished
feature rows, but its T11.F10 comparative discovery/retention criterion is still
pending. Keep the track In Progress. Its active handoff now correctly describes
per-kind habituation, one Eat vote with a scalar food-type parameter, and T18's
completed replanning; it does not claim T17.F04 has already shipped.

The two dependency edits above repair existing planning inconsistencies; they
are not claims that T19 caused them. Closed specs, historical measurements,
feature completion states, current priorities and runtime code are unchanged.
