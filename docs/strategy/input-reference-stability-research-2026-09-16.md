# Input Reference Stability Research Note (2026-09-16)

Grounds roadmap feature T11.F22. The question, raised by the user after the [live survey](live-survey-2026-09-16.md): should a mesh node's sensor references be fixed once the node exists, with new reference sets arriving only through new nodes, so each node evolves against inputs that do not move under it?

Research was a direct read of five primary sources on 2026-09-16 (no adversarial verification; one source at abstract level only), plus the live world's per-operator counters and the mutation spec. Depth is proportionate to a reversible operator-set change.

## Problem, constraints, decision criteria

**Problem.** A Petri node holds `input_refs`, a shared indirection table: every graph edge and VM `ReadInput` on the node addresses a sensor by `ref_idx` into that table (`v3-genome-spec.md`, "shared indirection for VM and Graph backends"). Two operators rewrite the table under its consumers: `InputRef.Swap` replaces entry k with a random sensor, so every consumer of k reads a different sensor at once; `InputRef.Remove` deletes entry k, strips every consumer of k, and renumbers the entries above it. `InputRef.Add` appends an entry nothing reads.

**Local evidence.** In the live world's `mutation_value_totals_by_operator` (tick 28,315, 15.4M carriers), helpful share = helpful / (helpful + detrimental); the population-wide baseline is 0.50 to 0.53:

| Operator | applied | helpful | detrimental | helpful share |
| --- | ---: | ---: | ---: | ---: |
| `InputRef.Swap` | 3,158,683 | 1,162,280 | 1,442,395 | **0.446** |
| `InputRef.Remove` | 1,578,132 | 626,561 | 745,038 | **0.457** |
| `InputRef.RawFieldMutation` | 3,158,166 | 1,232,898 | 1,359,641 | 0.476 |
| `InputRef.Add` | 1,577,699 | 711,329 | 640,505 | 0.526 |
| `Graph.AlterGraphEdgeWeight` (reference) | 1,002,204 | 464,898 | 411,614 | 0.530 |

`Swap` and `Remove` are the two lowest non-topology operators in the run (only `ChangeEntryNode` at 0.243 is lower). The InputRef domain draws 26.7% of all events (9.47M of 35.5M), so these two operators are about 13% of every birth's exposure.

**Cheap proof, run 2026-09-16 (100 of the survey's 400 dumped genomes, T11.F01 battery, 20 trials per operator per genome, live mutation config; probe in the appendix).** Per applied trial:

| Operator | applied | silent | changed | dead |
| --- | ---: | ---: | ---: | ---: |
| `input_ref.Swap` | 2,000 | 0.789 | **0.208** | 0.003 |
| `input_ref.Remove` | 2,000 | 0.799 | **0.201** | 0.000 |
| `input_ref.RawFieldMutation` | 2,000 | 0.806 | 0.194 | 0.000 |
| `input_ref.Add` | 2,000 | 1.000 | 0.000 | 0.000 |
| `graph.RetargetGraphEdge` | 2,000 | 0.887 | 0.113 | 0.000 |
| `graph.AddGraphEdge` | 2,000 | 0.955 | 0.045 | 0.000 |
| `graph.AlterGraphEdgeWeight` | 2,000 | 0.982 | 0.018 | 0.000 |
| `topology.ChangeEntryNode` | 2,000 | 0.357 | 0.580 | 0.063 |

Weighting each operator's changed fraction by its share of applied events in the live run gives 0.0715 changed events per event overall, of which `Swap` supplies 25.9%, `RawFieldMutation` 24.1%, `Remove` 12.5%, and the next largest, `ChangeEntryNode`, 7.6%. **`Swap` and `Remove` together supply 38.4% of all behavior-changing events in this population** (ten VM copy and motif operators could not be name-matched between the probe and the counters; they are mostly silent, so the share is slightly overstated, not understated). Two facts hold at once: these operators are the population's largest source of behavioral variation, and the variation they supply is net detrimental. Retiring them outright would cut behavior-changing births by about a third, from the survey's 15.7% toward 10%, and the no-regression rule would read that as a regression.

**Constraints.** T11's node-type contract (neutral growth, function-preserving copies from T11.F08, one small step per connection operator); the natural-analog rule; `InputRef.Add` is the cheapest neutral step in the system and the route by which the survey's one sensor-to-direction rule was acquired.

**Criteria.** (1) Does a reference keep its meaning for its consumers? (2) Is there still a one-event path to a new sensor? (3) Precedent. (4) Cost: operators retired or added, indicator effect.

## Options considered

| Option | What changes | Meaning stable? | New-sensor path | Precedent |
| --- | --- | --- | --- | --- |
| A. Freeze the set: no `Add`, `Swap`, or `Remove` on existing nodes; new nodes draw random references; copies re-draw | the user's first form | Yes | Only through a new node whose random draw happens to include the sensor and whose internals then wire it | None found; breaks T11.F08 copy neutrality |
| B. Append-only, meaning-stable references: retire `Swap` and `Remove`; keep `Add`; new nodes draw random references; copies inherit; unused references may be dropped as a neutral cleanup | the pure form; removes 38% of behavioral variation (cheap proof) | Yes (an entry never changes meaning; the table only grows) | `Add` (one event, neutral) then a consumer edge or instruction (one event) | NEAT: structural mutation only adds genes; gene identity is historical origin |
| C. Per-consumer rewiring only: retire `Swap` and `Remove`, keep the existing edge- and instruction-level retargets (`RetargetGraphEdge` moves one edge's `ref_idx` by one; VM operand mutation on `ReadInput`) | same as B plus explicit keeping of the fine-grained rewires | Yes for the table; a single consumer may still move | as B | CGP point mutation on one connection gene; LGP micro-mutation on one register operand |
| D. Gradual swap: keep `Swap` but restrict it to a same-shape reference (same compound width and read class, e.g. food ring type 0 to type 1); replace index-shifting `Remove` with a neutral prune | recommended: meaning stable within kind, variation channel kept | Within kind: a consumer keeps reading the same kind of signal | as today | SignalGP tags: inexact matching required, exact matching hurts |
| E. Keep as is | | No | `Add` | Petri today |

B and C are the same feature: the fine-grained retargets already exist and are not table rewrites, so any form keeps them. The cheap proof above moved the choice from B to D: B is recorded as the arm to test only if D's readings show the within-kind changes are still net detrimental.

## Material findings

1. **NEAT never retargets an existing gene; it only adds.** "Structural mutations occur in two ways ... Each mutation expands the size of the genome by adding gene(s)." A gene's identity is its origin: "Two genes with the same historical origin must represent the same structure (although possibly with different weights), since they are both derived from the same ancestral gene." Stanley and Miikkulainen 2002, Sections 3.1 to 3.2. Confidence high (primary text read).
2. **CGP rewires in place, but one consumer input at a time.** "In a point mutation, an allele at a randomly chosen gene location is changed to another valid random value ... if an input gene is chosen for mutation, then a valid value is the address of the output of any previous node" (Miller, CGP chapter, Section 2.6.1), with the caveat that "a small change in the genotype can sometimes produce a large change in the phenotype" (Fig. 2.7). CGP's evolvability rests on neutrality: "in genotypes having 4000 nodes, the percentage of inactive nodes is approximately 95%" and neutral drift "has been shown to be extremely beneficial" (Section 2.7). Confidence high. Reading for Petri: the harm in `Swap` is its granularity (a shared entry with many consumers), not rewiring as such; Petri's per-edge retarget is the CGP form and stays.
3. **Reference binding that changes gradually beats exact binding.** In SignalGP, tags "allow for inexact references; that is, a referring tag need not exactly match its referent." Across nine minimum-similarity thresholds (0% to 100%, 30 replicates each), the 87.5% and 100% conditions were significantly worse than every other (Kruskal-Wallis chi-squared 161.27, p < 2.2e-16); 0% to 62.5% did not differ. Lalejini and Ofria 2019 and the published analysis. Confidence high for the result; its transfer is to option D (a rewire whose step is small), not to B directly.
4. **Duplication then divergence outperforms hand-wired modules.** Calabretta, Nolfi, Parisi and Wagner 2000 compared a non-modular network, a hardwired modular network, and a duplication-based modular network on a robot task; both modular forms adapted faster and further, and the duplication-based form reached "a much higher degree of functional specialization." Abstract level only (full text unreachable on 2026-09-16); whether the copy inherits its source's input weights is not confirmed from the text, so this supports "copies inherit, new function arrives by divergence" only at the level of the abstract. Confidence medium.
5. **LGP changes one operand at a time.** Micro-mutations alter a single register, constant, or operator of one instruction, and effective-code analysis separates neutral from effective changes (Brameier and Banzhaf 2007, from the publisher summary and the effective-LGP paper). Confidence medium (secondary summaries). Same reading as finding 2.
6. **No system found rewrites a many-consumer indirection entry in one event.** Every precedent either adds (NEAT), retargets one consumer (CGP, LGP, TPG instructions), or binds gradually (SignalGP). `Swap` and `Remove` have no counterpart; the live counters above are the local measurement of that gap.

## Recommendation

**Build option D as T11.F22, "Meaning-Stable Input References": a reference keeps its kind for life, and its consumers keep reading that kind of signal.** `Swap` becomes a within-kind swap (same read class and compound width: a food ring may become a fruit ring, a barrier ring an occupied ring, one area summary another, one introspection key another) so the step is small and a consumer tuned to an 8-slot ring never wakes up reading a scalar; `Remove` becomes a neutral prune of unreferenced entries, so a consumer never loses its sensor and indices never shift under it. `Add` stays. New nodes draw at creation; copies inherit.

Against the strongest alternative, B (retire both, the user's intent taken literally): B gives the stability but removes 38% of the population's behavioral variation in one step, and the survey's reading that supply is no longer the block rests on that variation being there. D keeps the channel and shrinks its step, which is what SignalGP's threshold result rewards (exact binding lost, inexact binding won, 0 to 62.5% indistinguishable) and what CGP's per-input rewiring already does on the graph side. Against A (freeze the set and re-draw on copy): A loses `Add` and breaks copy neutrality. If D's closure readings show the within-kind swaps still net detrimental, B is the next arm, read the same way.

The analog: a neuron keeps the afferents it was wired with, and the modality of an input does not change once a circuit is built on it; a new modality reaches a circuit through new cells and through duplication followed by divergence (Ohno's gene duplication; Calabretta's duplicated modules). What a mature input may do is shift within its modality, the way a receptive field drifts; that is the within-kind swap.

## Contract for T11.F22 (decisions the spec owner works from)

1. **Reference kind.** Define a reference's kind as its `MeshReadClass` plus its compound width (`WorldInputKey::compound_width`), so the kinds are: scalar food here (per type), 8-slot food ring (per type), 8-slot barrier ring, 8-slot occupied ring, 7-slot area summaries (food per type, barrier, occupancy), nearby-creature banks (16, 8, 12), introspection scalars, upstream slots, action-queue. The spec fixes the exact partition; the rule is that a swap never changes width and never crosses read class.
2. **`InputRef.Swap` becomes within-kind.** The replacement is drawn from the kind of the entry being replaced; when the kind has one member the swap is not offered (not a skip that counts against the funnel: the operator is inapplicable for that entry, as T13.F03 defined). `clamp_sub_idx_after_swap` is no longer needed because width is preserved; keep it as a debug assertion.
3. **`InputRef.Remove` becomes `InputRef.Prune`:** removes only an entry with no consumer on either backend (no graph edge with that `ref_idx`, no VM `ReadInput` with it, live or not), then renumbers consumers above it; neutral by construction and covered by a property test; inapplicable when no such entry exists. The genome carry cost (T03.F08) is the reason pruning is worth offering at all.
4. **`InputRef.Add` and `RawFieldMutation` unchanged.** Per-consumer retargets (`RetargetGraphEdge` on `InputLeaf.ref_idx`, VM `ReadInput` operand mutation) unchanged.
5. **Creation and copy.** New nodes draw their references at creation as the growth operators do today (spec owner confirms in `AddNode`'s detour form and `AddInternalGraphNode`); copies inherit their source's references (T11.F08 neutrality). No change expected in either; the spec states it.
6. **Readings.** Founder operator rows: `Swap` and `Remove` rows are replaced by the within-kind swap and prune rows. Predeclared on the goal profile: dead-per-birth not up; changed-per-birth may fall by at most `Remove`'s 12.5% share of changed events plus the cross-kind fraction of today's `Swap` changes (the spec computes that fraction with the appendix probe extended to classify each swap as within- or cross-kind, before the operator is changed); the live-run `mutation_value_totals_by_operator` helpful share for the swap is read at closure and is predeclared to rise from 0.446 toward the 0.50 to 0.53 operator baseline. If changed-per-birth falls by more than the predeclared bound, the swap partition is too narrow and the spec widens it (for example, all 8-slot rings as one kind) in its one revision rather than closing.
7. **Out of scope:** any change to which sensors exist, to `sub_idx` sampling, to the graph or VM edge operators, and to T11.F21's bank (independent; neither depends on the other). Option B, full retirement, is recorded as the follow-on arm and is not part of this feature.
8. **Relation to T11.F11.** "Kind" is a hand-written partition; nearest-match tag resolution (T11.F11's mechanism, SignalGP's for every reference) is its general form, under which a small tag mutation rebinds gradually and no index can shift. If T11.F11 ships a tag scheme, sensor references adopt that same scheme and the within-kind swap and prune retire in its favor; T11.F22 does not design a second matching scheme, and nothing in it depends on T11.F11 or blocks it.

## Remaining uncertainty

- The cheap proof measured how often `Swap` and `Remove` change behavior, not whether within-kind swaps are less detrimental than cross-kind ones; the battery reads changed, silent, and dead, never helpful. The helpful-share prediction in item 6 is the feature's own test, read on the goal run's live counters.
- The changed-event shares assume the probe's per-operator rates transfer from 100 genomes at 20 trials to the whole population; ten VM operators were unmatched by name and excluded, which overstates the InputRef shares slightly.
- Finding 4 is abstract-level; it supports the analog, not a design detail.

## Appendix: the cheap-proof probe

Dropped into `crates/v3-core/tests/zz_probe_operator_rows.rs` and removed afterward; the genome dumps and `config.json` are the survey's (its Appendix A).

```rust
//! TEMPORARY PROBE (not for commit): per-operator silent/changed/dead rows
//! (T11.F01 battery) on dumped live genomes under the live mutation config.
use std::collections::BTreeMap;
use v3_core::config::SimulationConfig;
use v3_core::creature::genome::CreatureGenome;
use v3_core::neighborhood::{evaluate_genome, Battery, EvalContext};

#[test]
fn probe_operator_rows() {
    let dir = std::env::var("PETRI_GENOME_DIR").expect("PETRI_GENOME_DIR");
    let config_path = std::env::var("PETRI_CONFIG").expect("PETRI_CONFIG");
    let n: usize = std::env::var("PETRI_N").ok().and_then(|v| v.parse().ok()).unwrap_or(100);
    let trials: u32 = std::env::var("PETRI_TRIALS").ok().and_then(|v| v.parse().ok()).unwrap_or(20);
    let cfg_json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
    let config: SimulationConfig = serde_json::from_value(cfg_json["config"].clone()).unwrap();
    let battery = Battery::generate(config.world.food.types.len());
    let ctx = EvalContext::from_config(&config);
    let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "json")).collect();
    files.sort();
    let mut totals: BTreeMap<String, (u32, u32, u32, u32, u32)> = BTreeMap::new(); // trials, skipped, silent, changed, dead
    for (i, path) in files.iter().take(n).enumerate() {
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let genome: CreatureGenome = serde_json::from_value(v["genome"].clone()).unwrap();
        let eval = evaluate_genome(&genome, &battery, &config.mutation, &ctx, trials, 0, 1_000_000 * (i as u64 + 1));
        for row in eval.operator_rows {
            let e = totals.entry(format!("{}.{}", row.family, row.operator)).or_default();
            e.0 += row.tally.trials; e.1 += row.tally.skipped; e.2 += row.tally.silent; e.3 += row.tally.changed; e.4 += row.tally.dead;
        }
    }
    for (op, (t, s, si, c, d)) in &totals {
        println!("{}", serde_json::json!({"op": op, "trials": t, "skipped": s, "silent": si, "changed": c, "dead": d}));
    }
}
```

## Sources

- Stanley, K. O. and Miikkulainen, R. (2002). Evolving neural networks through augmenting topologies. Evolutionary Computation 10(2). https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf
- Miller, J. F. (2011). Cartesian genetic programming (chapter). https://harm.work/downloads/Cartesian_Genetic_Programming.pdf
- Lalejini, A. and Ofria, C. (2019). What else is in an evolved name? Exploring evolvable specificity with SignalGP. GPTP XVI. https://link.springer.com/chapter/10.1007/978-3-030-04735-1_6 ; repository and analysis: https://github.com/amlalejini/GPTP-2018-Exploring-Evolvable-Specificity-with-SignalGP , https://lalejini.com/GPTP-2018-Exploring-Evolvable-Specificity-with-SignalGP/analysis/stats.html
- Calabretta, R., Nolfi, S., Parisi, D. and Wagner, G. P. (2000). Duplication of modules facilitates the evolution of functional specialization. Artificial Life 6(1). https://pubmed.ncbi.nlm.nih.gov/10943666/ (abstract)
- Brameier, M. and Banzhaf, W. (2007). Linear Genetic Programming. Springer. https://link.springer.com/book/10.1007/978-0-387-31030-5 ; Effective linear genetic programming: https://d-nb.info/1106387333/34
