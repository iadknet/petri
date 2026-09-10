# Neutral Module Recruitment: From Silent Structure to Useful Behavior

**Status**: Research (evidence for [T13](../roadmaps/t13-neutral-module-recruitment.md); not a feature spec)
**Date**: 2026-09-08
**Code inspected**: initial main at `d14136db`; T11.F18 measured implementation at `b16f2820`. Closure status checked on main at `e61343db`.

## Question and decision

How does a neutral brain module acquire useful computation, and can growing
scaffold deprive existing computation of mutation opportunities? The user
requested a dedicated track after reviewing the T11.F18 drift miss and the
research below. T13 first makes recruitment observable, then repairs concrete
targeting and activation barriers, qualifies viable recruitment paths, and
measures discovery and retention. T11.F13 retains mutation-rate calibration.

Neutral creation is one step. A new module must receive mutations, acquire
inputs and effects, influence behavior, and persist when that influence is
useful. Selection cannot improve an entirely hidden function for its eventual
benefit: hidden variation drifts until a connection, another mutation, or a
changed environment exposes it. Action neutrality also does not imply equal
fitness when carrying and executing structure costs energy.

## Research and applicability

Primary sources checked 2026-09-08; read depth and transfer limits follow.

| Source | Finding used | Application to Petri and limit |
| --- | --- | --- |
| [Stanley and Miikkulainen, NEAT, 2002](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf), sections 2.3 and 3.1 and ablation discussion | New nodes split an existing connection, reducing initial disruption; the paper warns that disconnected additions may never join the working network. Speciation protects innovations while they improve. | Test connected, function-preserving growth beside blank growth. NEAT's split is approximately neutral because its new nonlinearity changes the function; it is not a proof of Petri's exact state contract. Do not import speciation or its fitness system. |
| [Atkinson, Plump and Stepney, Semantic Neutral Drift, 2021](https://link.springer.com/article/10.1007/s11047-019-09772-4), methods and results | Equivalent rewrites of active circuits create different mutation neighborhoods; several rewrites improved search, with task-dependent results. Rules are selected among those with valid matches. | Prefer existing safe split/copy machinery and measure the resulting access to useful changes. Their Boolean equivalences are not automatically valid for Petri's floats, recurrence, memory, queues or costs. |
| [Goldman and Punch, Reducing Wasted Evaluations, 2013](https://github.com/brianwgoldman/ReducingWastedEvaluationsCGP), author implementation/documentation; [Pedroni, An Explicitly Neutral Mutation Operator](https://eddy.dev/neutral-mutations-cgp.pdf), methods and results | CGP can distinguish active and inactive genes, preserve mutations in dormant material, and ensure an active gene is reached; active-only mutation did not universally outperform preserving neutral exploration. | Compare separate opportunities for refinement and scaffold in T11.F13. CGP's structural activity is not Petri's mesh dispatch or a battery knockout result; mutating an active gene does not guarantee useful behavior. No mutate-until-behavior-changes rule. |
| [Hayden, Ferrada and Wagner, 2011](https://www.nature.com/articles/nature10083), abstract and figure descriptions | RNA-enzyme populations that accumulated cryptic variation adapted faster to a changed substrate. | Include a changed-task reading and a matched control without accumulated scaffold variation. This does not prove arbitrary empty modules are useful or justify a production curriculum. |
| [Wagner, 2008](https://pubmed.ncbi.nlm.nih.gov/17971325/), abstract | Genotype robustness can reduce immediate variation while population movement through a neutral network increases accessible phenotypes. | Separate short-term changed-birth fraction from discovery and retention over lineages; neither silence nor change alone is a cognition score. |
| [Knibbe et al., 2007](https://academic.oup.com/mbe/article/24/10/2344/1074262), methods and discussion of noncoding sequence | With per-site mutation and rearrangements, noncoding sequence changes mutational load; low and high mutation rates favor different genome structures. | Per-node supply is a credible comparison with a carrying cost, already owned by T11.F13 and T03.F08. Petri's heterogeneous mutation events are not interchangeable nucleotide sites. |

## Local evidence and unresolved causes

- The [backend-bias note](mesh-backend-bias-research-2026-09-08.md) established
  VM-only detour creation on main. T11.F18 supplies equal Graph/VM creation
  opportunity; it is not a demonstration that new Graph modules become useful.
- Its implementation report at `b16f2820` measured 10/2,000 changed births
  at drift depth 2,000 against 16/2,000 previously, but 20/2,000 against
  3/2,000 at depth 1,000. At depth 2,000, total action-contributing modules
  across 50 lineages rose from 31 to 36. These observations do not establish
  the mechanism of the drift regression. T11.F18 subsequently closed on main
  with a [recorded feature-specific user exception](../specs/roadmap/t11-f18-backend-neutral-mesh-node-growth.md#performance-and-goal-impact)
  for the 0.005 reading; the standing 0.008 floor remained for later features
  until 2026-09-09, when the user re-based it permanently to 0.005 at
  T12.F04's closure (dated note in the
  [T11 track](../roadmaps/t11-brain-genotype-phenotype-map.md#notes-for-ai-agents)).
- The walk retains every descendant without ecological selection. Birth
  observations use 100 offspring from each of 20 parent lineages. A new RNG
  draw changes subsequent lineage histories. Preserve per-lineage results;
  an aggregate count cannot separate parental variation, mutation sampling,
  and substrate changes. Matched seeds alone do not synchronize semantic
  events across versions ([simulation experimental design](https://informs-sim.org/wsc03papers/008.pdf),
  section 4). Statistical units must reflect the sampled lineages
  ([Lazic et al., 2018](https://journals.plos.org/plosbiology/article?id=10.1371/journal.pbio.2005282),
  experimental-unit discussion).
- In [Graph target selection](../../crates/v3-core/src/mutation/graph/mod.rs),
  every Graph module is considered before checking the chosen operator's
  internal requirements. Selecting an edgeless module for an edge mutation
  returns `NoApplicableTarget`; the [engine](../../crates/v3-core/src/mutation/engine/mod.rs)
  discards that operator for this event even if another module has an edge.
  Adding inapplicable modules can therefore change realized operator supply.
  This path is established by inspection; its contribution to the drift miss
  is not measured. Audit analogous VM and input-reference cases only where
  the same defect occurs.
- The [Graph executor](../../crates/v3-core/src/runtime/cgp/execute.rs) returns
  immediately on zero compute nodes, even when effects have wired sources.
  A direct input-to-output connection can consequently need an unrelated
  compute node before it runs. T13 separates truly inert modules from modules
  with wired effects, preserving actual cost and state accounting.
- Existing [input-reference growth](../../crates/v3-core/src/mutation/input_ref/mod.rs)
  is unwired; [Graph growth](../../crates/v3-core/src/mutation/graph/operators.rs)
  offers disconnected, bootstrap and edge-split forms. A possible recruitment
  path adds an input reference, bootstraps a compute node from it, then connects
  its output to a surface the working controller uses. This is a possible
  path, not a measured waiting time or guaranteed fitness improvement.

## Options and decisions

| Option | Decision and reason |
| --- | --- |
| Observe actual recruitment using existing runtime traces, neighborhood batteries, benchmark reports and progress rows | Adopt first. Extant counts cannot identify a newly recruited module or explain its loss. Keep the bounded experimental provenance out of production mutation decisions. |
| Select operator-specific applicable targets before applying existing execution bias | Implement a bounded repair. A failed local target is not proof an operator is globally unavailable; keep empty modules eligible for operations that can grow or connect them. |
| Allow wired Graph effects without an unrelated compute node | Implement explicit activation semantics, with blank neutrality, reference updates and real energy accounting. Do not add a dummy neuron just to bypass the guard. |
| Reuse safe connected growth and copy/diverge paths | Qualify both backends, adding only a missing path established by the baseline. Prefer a small change to existing operators over a new macro-operator catalog. |
| Increase all mutation rates, prune every silent node, or target only battery-proven contributors | Do not adopt as a default. These can destroy or starve the intermediates recruitment needs, and battery sensitivity is not ecological value. |
| Separate refinement and scaffold opportunities; per-node supply with carrying cost | Compare under existing T11.F13. Measure discovery, retention, damage and cost before choosing a production policy. |
| Replace the mesh, add fan-out, a novelty reward, controller templates or global speciation | No demonstrated need; outside this track. |

## Evidence required before stronger claims

Use the existing measurement path, not a new campaign framework. First prove
short viable mutation paths with constructed fixtures, then measure their
frequency through production mutators on declared seed batches. Follow new
module cohorts through first mutation, first input/effect, contribution,
retention and loss; handle copies explicitly. Report observation cutoff and
deletion separately from failure to recruit. A knockout proves sensitivity
on the named battery, not usefulness; usefulness needs paired performance
on an existing task or ecological survival/reproduction, including cost.

Baseline comparisons separate drift from selection, blank from copy/split
growth, and RNG-history changes from backend choice where a synchronized
control is feasible. Predeclare batches, denominators, task objectives,
retention horizons, practical-loss margins and compute budgets before each
measurement. Keep every result, including nulls, and estimate uncertainty
across lineages rather than treating sibling trials as independent lineages.

T11.F18 closed independently while this track was being drafted. This track
preserves its recorded acceptance exception, original miss and standing floor.
Later measurements can inform an explicit prospective contract decision;
choosing a favorable seed or raising a floor to each new sample maximum is
not calibration. No new simulation was run for this note.
