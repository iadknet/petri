# Slower deletion: research and experiment plan (2026-10-09)

Date: 2026-10-09. Status: plan, not yet run; the user chose option B on
2026-10-09. Question raised by the user:
recent research suggested deletion mutations remove structure before it can
collect enough further mutations to become useful. Is testing that a
configuration change or a mutation code change? The answer is code, so the
user asked for this plan: a stricter re-screen of the follow-up run's N2 lead
(T11.F28, "stays a lead"), on the T20.F16 baseline, with the loss routes
widened past `Prune`. It runs under the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md),
in the form of the [output-wiring plan](output-wiring-experiment-plan-2026-10-09.md).

## Problem and constraints

- **Problem.** If deletion operators remove new structure faster than
  refining operators can tune it, it never becomes useful. Neutral drift
  never gets to work on it.
- **Constraints.**
  - The not-forcing checks and a natural analog.
  - Branch-only `proto:` code, default off, classified `policy-deviation`.
    The unchanged-policy arm must match `main`'s output.
  - No production change outside a roadmap feature.
  - The three conditions the follow-up run set before T11.F28 can become a
    feature ([follow-up note](evolvability-diagnosis-run-2026-10-followup.md),
    plan forward item 2):
    - a non-loss control arm declared in advance;
    - every checkpoint read on every seed, under an aggregation rule declared
      in advance;
    - a minimum effect size declared in advance.
- **Decision criteria.**
  - Is deletion still a material hazard on `main` after T20.F16, and for
    which structure?
  - Does slowing deletion, and nothing else, raise executed and causal
    sensor use by more than trajectory noise?
  - What does that cost persistence and genome size?

## Local evidence (inspected 2026-10-09)

**No configuration controls deletion.**
- Operator weights are hard-coded `const fn weight()` tables: InputRef
  ([input_ref/mod.rs:163](../../crates/v3-core/src/mutation/input_ref/mod.rs)),
  Graph ([graph/mod.rs:73](../../crates/v3-core/src/mutation/graph/mod.rs)),
  topology ([topology/mod.rs:60](../../crates/v3-core/src/mutation/topology/mod.rs))
  and VM ([vm/mod.rs:60](../../crates/v3-core/src/mutation/vm/mod.rs)).
- The deletion operators are those whose `complexity_effect()` is
  `Decreasing`:

  | Domain | Operator | Weight |
  | --- | --- | --- |
  | InputRef | `Prune` | 2 of 12 |
  | Graph | `RemoveGraphEdge` | 2 of 44 |
  | Graph | `RemoveInternalGraphNode` | 1 of 44 |
  | Graph | `DisableHebbian` | 1 of 44 |
  | Graph | `DisableRewardModulation` | 1 of 44 |
  | Topology | `RemoveNode` | 10 of 211 |
  | Topology | `RemoveRouteTarget` | 20 of 211 |
  | VM | `VmDeleteInstruction` | 1 of 32 |

  `DisableHebbian` sets the node's plasticity config to `None`, and
  `DisableRewardModulation` sets its modulation to `None`; either way the
  tuned parameters are lost.
- `MutationConfig`
  ([config/simulation.rs:822](../../crates/v3-core/src/config/simulation.rs))
  has one per-operator setting, `large_copy_weight_percent`, and it covers
  only the copy operators. `per_unit_rate` scales every operator at once.

**What deletion removes, on the founder.** Hazards per birth are from E4
([diagnosis run](evolvability-diagnosis-run-2026-10.md), 10⁷ births). The
"survives" column assumes about 400 births along a line of descent. B1 and
W1 report a mean generation of 346 to 457 at 20,000 ticks on Canyon.

| Structure | Deletion hazard per birth | Other loss per birth | Survives 400 births (deletion only) |
| --- | ---: | ---: | ---: |
| Silent declaration (no consumer) | 9.9 × 10⁻³ (`Prune`) | 8.7 × 10⁻³ (retype 24 %, `Swap` 16 %, mixed 7 % of the total; not counted in the last column) | about 2 % |
| Connected declaration | about 0 (T11.F22 makes it unprunable) | 7.96 × 10⁻³ (retype 57 %, `Swap` 37 %) | about 100 % |
| A sensor edge | 1.1 × 10⁻⁴ (`RemoveGraphEdge`) | re-weighted 2.2 × 10⁻⁴; its reference retyped or swapped 8 × 10⁻³ | about 96 % |

Three readings follow from this table:

1. **T20.F16 changed the premise.** Since 2026-10-09 an `Add` on a Graph
   node arrives wired to a vote sink
   ([input_ref/mod.rs:302](../../crates/v3-core/src/mutation/input_ref/mod.rs)).
   It is therefore addressed and `Prune` cannot touch it. The
   declare-then-prune race the diagnosis measured now arises only in these
   cases:
   - an `Add` on a VM node, which is still unwired;
   - a reference whose only edge was removed or retargeted;
   - the founder's own silent ring entry.
2. **Deletion is fast only where `Prune` concentrates.**
   - `Prune` draws only among a node's silent entries, so a lone silent
     entry takes every `Prune` event on its node. Its per-site hazard is
     about 1/k for k silent siblings.
   - Every other deletion operator draws uniformly over all its sites:
     `pick_random_edge` draws over every edge
     ([graph/operators.rs:651](../../crates/v3-core/src/mutation/graph/operators.rs)).
   - A sensor edge is deleted about once in 9,000 births.
3. **Retype and `Swap` are drift, not deletion.**
   - They keep the entry or edge and change what it reads.
   - E4 counted them as loss of one family, `AreaFoodSummary(0)`. They are
     the edits the hypothesis wants to accumulate.
   - These rates are for the founder only. Evolved genomes carry many more
     sites, and an executed bias of 0.9 puts most events on executed nodes.
     No measurement exists for them; gate S0 provides one.

**N2, the earlier test.**
- Prototype: `mutation.silent_entry_decay_rate`, commit `dcbee45e`, branch
  `worktree-evolvability-diagnosis-2`. Ordinary events that hit a silent
  entry were skipped. A per-birth pass then removed each silent entry at
  rate 0.005, by prune, swap or retype.
- Outcome: it passed its screen, but specificity was not shown.
  - The `hold2` arm, which changed no loss rate, matched the declared-count
    gain.
  - The direction reversed at 10,000 on seed 2022.
  - `slow` fell to 52 living creatures on that seed.
  - Its genomes were smaller than the default's, against the prediction
    that silent cargo would accumulate.
- It cannot be rebased as file state: it predates T03.F12's gestation, and
  `apply_add` changed for T20.F16. It stays a design reference. Two lessons
  carry over:
  - Skip an event terminally, never redraw it; a redraw hands the draw to
    other operators.
  - A deletion drawn on a node where it has no site is still discarded and
    redrawn, as today.

**Structure accumulates anyway.** Mean genome size on Canyon at 20,000 ticks
is about 5.5 to 6 × the founder's under T20.F16 production (B1's `born`
arm, which W1's `default` reproduced). Deletion does not
stop growth in bulk. The question is whether particular new structure
survives long enough to be refined.

## External evidence (searched 2026-10-09)

- **NEAT has no deletion mutation.**
  - [Stanley and Miikkulainen 2002](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf),
    *Evolutionary Computation* 10(2), section 3.1, has only two structural
    mutations, and "each mutation expands the size of the genome by adding
    gene(s)".
  - Add-node disables the split connection instead of deleting it.
  - Speciation (section 3.3) exists so that new structure gets "time to
    optimize" before it competes.
  - This is precedent for protecting new structure, and for dropping
    deletion as a mutation.
- **CGP keeps inactive genes.**
  - [Turner and Miller 2015](https://gpbib.cs.ucl.ac.uk/gp-html/Turner_2015_GPEMa.html),
    *Genetic Programming and Evolvable Machines* 16(4):531–558, found that
    the benefit of explicit redundancy (inactive genes kept in the genotype)
    to neutral drift is additive to the benefit of implicit redundancy.
  - Petri's Graph backend is CGP-shaped (`runtime/cgp`).
  - Only the abstract was verified; nothing from the body is cited.
- **Counter-case: bacteria delete pseudogenes fast.**
  - [Kuo and Ochman 2010](https://pmc.ncbi.nlm.nih.gov/articles/PMC2916853/),
    *PLoS Genetics* 6(8):e1001050: young *Salmonella* pseudogenes are
    removed, chiefly by deletions, faster than a neutral model allows.
  - The authors attribute this to selection against harmful pseudogenes.
    The pseudogenes that persist are weakly expressed and weakly connected.
  - Reading for Petri: fast loss in nature is selection removing harmful
    structure, and inert structure lingers. Petri already selects. The arms
    below change only the mutational deletion rate.
- **Already in the repo.**
  - Wagner 2008 §2c: the size of the neutral set buys access to new
    phenotypes ([diagnosis note](evolvability-diagnosis-2026-10-05.md)).
  - Walsh 2026: copies become essential only if they survive long enough
    ([literature scan](alife-recent-research-scan-2026-10-05.md)).

## Options

| Option | What | For | Against |
| --- | --- | --- | --- |
| A. N2 rebased, plus edge decay (the approved shape) | Re-implement N2's silent-entry decay on `main`, plus a per-birth decay pass over edges | Per-site decay is the faithful analog for `Prune`'s 1/k concentration | There is no silent set for edges, so a silence criterion would have to be invented. Edges already survive about 96 % of a run. It also slows retype and `Swap`, which are the drift the hypothesis wants. It is a new build either way |
| **B. Deletion keep probability** | `mutation.deletion_keep_probability: Option<f64>`. An applicable `Decreasing` operator applies with probability s; otherwise it is skipped terminally with its own skip reason, never redrawn | One knob over an existing classification, in all four domains. Encodes the hypothesis literally. Leaves retype and `Swap` alone. At s = 1.0 it gives a non-loss control that runs the same code | Keeps `Prune`'s 1/k concentration (scaled by s). Also slows removal of harmful structure, which then falls to selection |
| C. Lower the hard-coded weights | Edit the `const` tables | No new concept | Every other operator's share shifts (fixed totals, redraws). Changes production policy. Has no null arm |
| D. Record and stop | Write up the post-T20.F16 reading only | E4 already suggests deletion is minor except for silent entries | E4 is founder-only and predates T20.F16; evolved genomes are unmeasured |

## Recommendation: B, behind a measurement gate (S0)

B tests the hypothesis as stated ("deletion at its current rates is the
block") with one rule and a control that runs the same code. If S1
validates it, the feature row decides the analog's form: per-site decay or a
scaled rate. A is still the better answer to `Prune`'s concentration, but
only once deletion is shown to matter. S0 comes first because E4 already
suggests that, on `main`, deletion is fast only for silent entries.

### The prototype (`proto:`)

- **Field.** `MutationConfig.deletion_keep_probability: Option<f64>`.
  - `None` is the default. It is absent from the serialized config, so
    recipe digests are unchanged and the frozen instrument reads the field
    through the serialized config.
  - Where it applies: after an operator passes the T13.F03 applicability
    check and before it is applied.
  - When set: if the operator is `Decreasing`, draw `u = rng.gen::<f64>()`
    and apply only if `u < s`. Otherwise record a terminal skip with a new
    reason, `DeletionNotKept`.
  - Use an explicit draw: rand's `gen_bool(1.0)` consumes no draw, so it
    would make the s = 1.0 arm identical to `None`. With the explicit draw,
    `del-1.0` is the stream-perturbed, policy-identical control.
  - Size pressure stays off, as in production.
- **Tests.**
  - `None` reproduces `main` draw for draw.
  - `Some(1.0)` never records the new skip.
  - `Some(0.0)` applies no `Decreasing` operator, still discards and redraws
    an inapplicable one, and leaves the requested count equal.
  - `Some(0.5)` skips at the rate within a CP interval and is deterministic.
  - Serialization: absent at `None`, round-trips at `Some`.
  - A proptest: applied events plus skips equal the requested events.

### Gate S0: does deletion still matter on `main`? (go/no-go)

- **Parents.** The founder plus the 20-parent selected cohort of `default`
  at 20,000 on seeds 1022 and 2022: 40 evolved parents. The census probe
  dumps their genomes (a small `instr:` addition).
- **Probe.** The E4 probe, extended as a `lab:` addition with per-class
  deletion hazards for genome-file parents.
- **Births.** 10⁶ one-birth children per parent, through
  `apply_mutations_with_food_type_count`, at `None` and at 0.03.
- **Per parent and per deletion class**, each class being the site a
  `Decreasing` operator removes (a silent input entry, a graph edge, a
  compute node, a plasticity or reward config, a mesh node, a route target,
  a VM instruction):
  - h_c = applied deletions ÷ (eligible sites × births);
  - L_c = 1 − (1 − h_c)^400.
- **Edits before deletion (R), descriptive only.**
  - For edges: re-weight, retarget and re-source events per edge ÷ removals
    per edge.
  - For entries: retype and `Swap` events per entry ÷ prunes per silent
    entry.
- **Check against E4.** On `main` the founder plus a declared entry loses it
  at E4's 0.018643 (the CP interval contains it).
- **Manipulation check at 0.03.**
  - Each `Decreasing` operator's applied count: the CP interval contains
    0.03 × its count at `None`.
  - Each other operator's applied rate: within 5 % of `None`.
  - Requested events: identical.
- **Go** if some class has a median L_c ≥ 0.5 at `None`. The median is
  taken over the evolved parents that carry the class, and a class counts
  only if at least 10 of the 40 carry it.
- **No-go** otherwise. The note then records that deletion is not the block
  at a 20,000-tick horizon on `main`, lists the hazards, and S1's treatment
  arms do not run.

### Gate S0.2: identity

The census probe on the prototype binary at `None`, against the instrument
built without the prototype: samples and timing-free checkpoint blocks
hash-equal on seeds 1022 and 2022 to 20,000, as in W0.2.

### Row S1 (draft)

- **Design.** Canyon goal recipe, 20,000 ticks, the census probe with
  checkpoints at 10,000 and 20,000. At both checkpoints: the input funnel,
  the vote-delta reading and the births block; E6 at 20,000.
- **Seeds.** 1022 and 2022 (comparable with N2, B1 and W1), plus 3022 and
  4022. The seed is the replicate.
- **Limits.** 3 h kill per run, one job at a time.
- **Arms.**
  - `default` (`None`), which also supplies S0's parents;
  - `del-1.0`, the non-loss control: the same code runs and nothing is
    skipped;
  - `del-0.3`;
  - `del-0.03`.

  No mutation-off pairs: at `per_unit_rate` 0 no deletion is drawn, so they
  would be identical by construction.

| Reading | `del-1.0` | `del-0.3` | `del-0.03` |
| --- | --- | --- | --- |
| P = Σ over the three typed target rows of `family_executed` + `family_causal` (B1's measure) | differs from `default` by trajectory noise only | between | above `default` if the hypothesis holds |
| Vote-delta target channels that change a committed action | noise | not predicted | above `default` if the hypothesis holds |
| Mean genome size and size per generation | as `default` | higher | highest; N2 saw the opposite, so a contradiction goes to the advisor |
| Window-mean population and its lowest sample | as `default` | not predicted | at least 0.5 × `default`'s; the lowest sample is recorded (N2 fell to 52) |
| Births block: dead, silent, changed | as `default` | not predicted | not predicted |

**Validation rule (draft, for the user to fix in the goal command).**
- **Per seed and checkpoint t:**
  - noise N(t) = |P(`del-1.0`) − P(`default`)|;
  - effect E(t) = P(`del-0.03`) − P(`default`).
  - Scale: P was 22 and 19 at 20,000 in B1's `born` arm, which is today's
    production, so 5 is roughly a quarter of the baseline.
- **A seed supports the hypothesis** if, at both checkpoints, E(t) ≥ 5 and
  E(t) > N(t). The checkpoints of one world are dependent samples, so they
  must agree; seeds are what get counted.
- **`del-0.03` is validated as a lead** if all of these hold:
  - S0 was a go;
  - `del-0.03` persists to 20,000 on all four seeds, with window-mean
    population at least 0.5 × `default`'s on each;
  - at least 3 of 4 seeds support;
  - at 20,000, P(`del-0.03`) ≥ P(`del-0.3`) ≥ P(`default`) on at least 3 of
    4 seeds (dose order);
  - on each supporting seed, the vote-delta block shows at least one target
    channel whose ablation changes an action under `del-0.03`.
- Otherwise it is not validated.
- With 20 parents per cohort this is a lead-selection screen, stated as
  such.
- **What each outcome means.**
  - A validated S1 makes a deletion-rate feature a candidate: T11.F28
    retitled, with its form (per-site decay or scale) left to the spec.
  - An unvalidated S1, or a no-go at S0, retires T11.F28's premise on
    `main`.

## Cost

- **Code.**
  - The prototype: one engine branch per domain loop, the config field and
    its tests. Smaller than N2's, because there is no decay pass.
  - The S0 extension to the probe and the cohort dump in the census probe:
    about one session together.
- **Runs.**
  - S0.2: two frozen-instrument runs.
  - Four `default` runs.
  - S0: 40 parents × 2 settings × 10⁶ births. Founder births take 2.2 s per
    10⁶; evolved genomes are larger, so time a smoke run before choosing
    the count.
  - Twelve treatment runs if S0 is a go.
  - About 18 runs at 11 to 15 min each: roughly 4 h of compute.
  - Dropping seeds 3022 and 4022 halves the runs, and the 3-of-4 clauses
    become 2-of-2.
- **Session.** One goal session of about 150 to 200 turns.

## Remaining uncertainty

- The executed bias of 0.9 may shield inert structure from deletion and
  from refinement alike. Slowing deletion then buys no refinement. S0's R
  readings show this.
- B keeps `Prune`'s 1/k concentration. At 0.03, a lone silent entry's
  founder deletion hazard is about 3 × 10⁻⁴, so it survives 400 births about
  nine times in ten. Larger silent sets fare better still.
- Slower deletion may inflate genomes and slow runs. The 3 h kill caps
  this, and a killed run counts as not persisting.
- P counts target-family reads only. Gains on other families show up in the
  vote-delta block and the births block, but are not validated.

## Goal command

Open a new session in the main checkout on a clean `main`, model Opus 5.5,
with the advisor enabled on Opus 5.5 and Codex for reviews (the user's
choice on 2026-10-09). Paste:

```
/goal A deletion-rate experiment run is complete on main. Read docs/strategy/deletion-rate-experiment-plan-2026-10-09.md in full (its prototype, gates S0 and S0.2, row S1 and validation rule are the contract's form), then docs/strategy/evolvability-diagnosis-run-2026-10-followup.md rows G3 and N2 and its plan forward item 2 (the three conditions), then docs/strategy/output-wiring-run-2026-10.md (gates, frozen-instrument pattern, Advice table). Confirm Opus 5.5 with the advisor enabled on Opus 5.5 and Codex for reviews, a clean main, and create the worktree with EnterWorktree named deletion-rate. Take the census probe and the diagnosis probe as file state from branch worktree-output-wiring (instr and lab commits only, no proto). Build the proto field mutation.deletion_keep_probability as the plan states it, the cohort genome dump as instr, and S0's per-class hazards as lab, reading the prototype field through the serialized config so the instrument builds without it. Predeclare and commit S0, S0.2 and S1 before any of them runs: Canyon goal recipe, seeds 1022, 2022, 3022 and 4022, 20,000 ticks, checkpoints 10,000 and 20,000, arms default, del-1.0, del-0.3 and del-0.03, 3 h kill per run, one job at a time. Run S0.2 and the four default runs first, then S0; if S0 is no-go, record it and do not run the treatment arms. The validation rule is the plan's draft rule as written. Consult the advisor at every rule 8 point and record it; send the predeclaration to Codex before any row runs and the note's reading before closing. Write docs/strategy/deletion-rate-run-2026-10.md with the gates, the row, the Advice table and a plan forward that says whether a deletion-rate feature is recommended (and in which form), or T11.F28's premise is retired. Land docs only; keep proto, instr and lab on the branch. Done means the note is on main, make check-docs exited 0 on main, the worktree removed and its branch kept, main clean. Stop after 200 turns.
```
