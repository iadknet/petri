# Factored Motor Output Research Note (2026-10-11)

The user asked whether per-direction action sinks (`Move(N)`, `Move(E)`, …,
`Reproduce(N)`, …) are a problem. Every direction grows its own circuit, where a
natural design would have one circuit decide "I should move" and another decide
which way. A factored design was considered and rejected once already
(2026-09-21). This note checks that decision against the code, the live world
and the literature, and proposes what to change.

**Decision (user, 2026-10-11): groups connect as groups.**
- A scalar reaches a kind through that kind's drive (T11.F30).
- A neighbor ring reaches a kind's direction group as one matched gene with a
  drawn offset and sign (T11.F31).
- A shared-weight fan-out from a scalar to a direction group adds the same
  amount to every direction. That is exactly a drive edge, so F31 adds no
  fan-out of its own.
- Both rows are placed in the T11 track and appended to the master priority
  order. T20's weight-tying sentence is amended.
- After the adversarial review (user, 2026-10-11):
  - T11.F31 runs first;
  - projections are born at zero weight;
  - F30's ties go to a heading, not North (§7);
  - backward compatibility is not a consideration.

## Answer

1. **Revisit it.** The 2026-09-21 rejection had two reasons:
   - The first ("two directions of one kind cannot be ranked in one pass") was
     written against the per-sink rising bar, which was replaced before T19
     landed. Under the adopted per-kind bar the current catalog has the same
     property (§3).
   - The second reason still holds: a shared direction bank couples all kinds.
2. **Split "whether" from "where", per kind.** Add three kind drives (`Move`,
   `Reproduce`, `StealEnergy`; `Eat` already is one) and keep the 24 directed
   sinks as each kind's own direction channel.
   - With no drive written, a kind commits exactly as it does today.
   - Biology supports this split well (§5). The founder is already written this
     way, four times over (§4).
   - Petri has already had a per-kind whether/where split once: T11.F21's
     `ActionSlot`, a gate plus a direction bank, until T19.F04 deleted it.
     Steering avoidance stayed 0 throughout that era. The split alone is not
     expected to produce steering.
3. **Do not share one "where" across kinds.** Petri's kinds want different
   targets: food for Move, an occupied cell for Steal, an empty cell for
   Reproduce. Modularity theory predicts that coupling separate functions
   hurts. (The tectum evidence of separate approach and escape tracts concerns
   two rules for one motor kind, locomotion, so it does not decide this.)
4. **The bigger lever is a one-gene direction motif (user ruled yes,
   2026-10-11).** A persistent "projection" gene that wires a
   whole sensor ring onto a kind's direction channel, with per-direction edges
   kept as exceptions, has the best support in both literatures (§5).
   - The live world shows the cost. After about 345 generations, the founder's
     four-direction food taxis survives in a median of one direction per
     creature, or two outside one large clade. Nothing direction-general has
     evolved beyond it (§4).
   - Per-direction copies are kept when selection holds them: the founder's
     gate copies survive on all four directions in 185 of 200 creatures. So
     the encoding does not by itself cause loss. What it does is let each
     direction be kept or lost on its own.
   - In-project evidence supports correlated *variation*, not a tied weight
     (§2). The persistent form rests on the literature (§5). F31 is the test.
   - The ruling overrides not-forcing check 3 and the T20 weight-tying sentence
     for the three neighbor rings only.

## 1. Problem, constraints, criteria

**Problem.**
- `VoteSink` (`crates/v3-core/src/creature/genome/vote.rs:105-118`) has 27
  sinks: `Eat`, eight each for `Move`, `Reproduce` and `StealEnergy`, plus
  `Terminate` and `Decide`.
- `runtime/vote_select.rs:43-77` picks each kind's lowest-index argmax sink.
  The kind's effective vote is that sink's vote minus its per-kind bar.
- So a kind's urge to act is only the strength of its best direction. Any rule
  that should apply in every direction has to be wired eight times, once per
  sink:
  - "move toward food", "avoid barriers", "steal from a neighbor";
  - "don't move when full", which today is one edge per wired direction.

**Constraints.**
- R3 (mesh review `:31`): multiple, repeated and ordered actions per tick. The
  user restated it as required on 2026-10-09.
- The per-kind bar with within-kind argmax (T19.F04).
- Determinism, and an RNG-free parallel phase.
- The natural-analog rule.
- Both backends get the same opportunity (T11.F18).
- New forms reachable by the existing operator families.
- Backward compatibility is not a default goal (AGENTS.md). Neutrality at birth
  is still the cheapest way to read an effect.

**Criteria.**
1. Is a direction-general rule (seek, avoid, aim, gate) one selectable step?
2. Does the design keep kind-specific aiming and R3?
3. Natural analog.
4. Precedent and evidence on evolvability.
5. Cost: sinks, hot path, report surface, and the pending rulings.

## 2. What was decided before, and what still holds

| Date | Decision | Status now |
| --- | --- | --- |
| 2026-09-16 | T11.F21 per-direction winner-take-all bank ([motor-output note](motor-output-encoding-research-2026-09-16.md)); "topographic operator bias" recorded as a follow-on | Holds. The bank replaced a scalar decode that no evolved system uses. The follow-on was never built. |
| 2026-09-16 to T19.F04 | T11.F21's `ActionSlot` carried `gate_inputs` (whether) and `direction_bids` (where) per kind | A per-kind whether/where split existed. T19.F04 deleted it (spec `:17`). Steering avoidance was 0 throughout. |
| 2026-09-21 | Mesh review option D, "kind sinks plus one shared direction bank", rejected for R3 ([review](mesh-action-selection-review-2026-09-20.md) `:290`) | Half stale. See §3. |
| 2026-09-21 | Per-kind bar with within-kind argmax replaced the per-sink bar, because per-sink bars made the founder queue `Eat, Move E, Move N, Move S, Move W` (review `:185`) | Holds. This is the rule that made reason 1 of the option-D rejection stale. |
| 2026-09-23 | Ring-wiring harness ([ring-wiring](ring-wiring-experiment-2026-09-23.md) `:40-53`) | From a pre-wired zero 64-edge matrix, hill-climbing on an authored task score: coordinated cyclic-offset updates reached all eight directions in 28/32 runs. The whole-matrix arm reached 0/32 and matched-direction zero edges 3/32. Weights were not permanently tied. This supports correlated *variation*, not a tied weight. |
| 2026-09-23 | Ring-group harness ([ring-group](ring-group-experiments-2026-09-23.md)) | Partial sharing with independent exceptions beat independent and full sharing. This was lifetime learning, not inheritance. |
| T20.F05 | `RefineHeritableStructure`: one coordinated step for an existing group of ring→sink edges | Off by default. Run 3's arm was null. It only acts on edge groups that already exist, and the live census (§4) finds none. |
| 2026-10-06 | E5 `AddProjection` ([diagnosis run](evolvability-diagnosis-run-2026-10.md), E5 row) | Not advanced. It wrote 8 *independent* edges in one event, so later variation was uncorrelated, and it mixed meaningful ring pairings with arbitrary summary pairings. The `aligned` variant, which has the stamp's birth shape at weights in (0, 1], raised harmful births on wall from 22% to 32% (ratio 1.45, lower bound 1.16) and was not above `scrambled` on either assay. Dense variants harmed 4 of 5 fired births on food and 36–43% on wall. A doubling of helpfulness was not excluded. The persistent one-gene form was never tested. |
| 2026-10-06 | Not-forcing check 3 and the index-alignment ruling | Pending with the user ([follow-up](evolvability-diagnosis-run-2026-10-followup.md) `:501-503`). |

## 3. Why the option-D rejection is half stale

Option D was rejected for two reasons:
- "two directions of one kind cannot be ranked in one pass";
- "a direction bid affects every movement kind at once".

Section 4 of the review still labels the *per-sink* rising bar as adopted
(`:284`). The option list therefore reads as written before the switch to
per-kind bars. This is inferred from the stale text: git has both in one commit,
`256d2463`. Under the per-kind bar that T19.F04 adopted:
- A static vote pattern commits `ceil(v)` actions of a kind, all in the
  winning direction (W7, `:155`).
- A turn needs a state-driven vote change between passes (W9, `:157`).
- Section 2.3 of the review says so itself: "the kind vote is how many; the
  argmax is which" (`:172`).

So today's catalog cannot rank two directions of one kind from a static pattern
either. The first reason no longer separates the designs.

The second reason is real. It applies only to sharing one "where" *across
kinds*. It does not apply to splitting "whether" from "where" *within* a kind.

## 4. Measured evidence

**Live-world census.**
- Taken on the user's running v3-server on 2026-10-11:
  - tick about 12,400, population 6,158, 1600×1600 world;
  - 6,587 creature IDs pooled from 16 tiles of 400×400 covering the whole grid;
  - 200 dumps sampled with seed 20261011;
  - generation median 345, maximum 382.
- Scope:
  - Graph-backend wiring on reachable nodes only. Reachable VM nodes carried no
    vote instructions.
  - One world at one time. This is a description, not a closure reading.
- Scripts: Appendix A.

Findings. Ring sources are traced through the bus: a ring copied onto
`CustomOutput(k)` by any reachable node and read back as `UpstreamSlot(k)`.
- **The founder's four-direction food taxis has come apart, one direction at a
  time.**
  - The founder copies food-ring N/E/S/W onto bus slots 2–5. It then votes
    `0.4·slot` into the matching `Move(d)` sink.
  - All four copies survive in 2 of 200 creatures:

    | Food-taxis directions kept | 0 | 1 | 2 | 3 | 4 |
    | --- | --- | --- | --- | --- | --- |
    | Creatures (of 200) | 19 | 94 | 64 | 21 | 2 |

  - By direction it is W 158, N 56, E 51, S 28.
  - The adversarial review found a clade behind most of the loss: 82 creatures
    carry the food-N → `Move(SE)` marker, spatially clustered. They hold 65 of
    the 82 W-only creatures and 17 of the 19 with no taxis. Outside that clade
    the median is 2 directions.
  - This census cannot separate selection from clade drift.
- **The gate copies were mostly kept.**
  - 444 of the founder's −2.0 gate edges into Move now point at a removed
    compute node and read 0.
  - At least one live −2.0 gate remains on all four cardinals in 185 of 200
    creatures.
  - Per-direction copies survive when selection holds them.
- **Method caveat.** Bus tracing counts any reachable writer of
  `CustomOutput(k)` and ignores execution order and overwrites, so the taxis
  counts are an upper bound.
- **Nothing beyond the founder's rule has evolved.**
  - No creature has a diagonal food rule, an aligned barrier→Move rule (5
    barrier→Move edges in all, at offsets 6 and 7), or more than one aligned
    occupied→Steal direction.
  - The commonest evolved ring→Move edge is one clade's single misaligned pair:
    food at N → `Move(SE)` at −0.66, in 78 of 200 creatures.
- **`Reproduce(N/S/E/W)` repeats the founder's per-direction shape.** Diagonal
  sinks are barely used.
- **Aiming fails where it matters.**
  - In the sampled action logs, `StealEnergy` returned `NoVictim` 1,073 times
    against 171 successes.
  - Steal directions are scattered, not pointed at occupied cells.
  - World counters: moves next to a barrier were blocked 56.7% of the time for
    barrier readers and 54.1% for non-readers.
  - steering-v1 barrier avoidance has been 0 at every closure from T11.F21 to
    T16.F02. Evolved exact-hit is 0.36 to 0.38, below the founder's 0.5
    (chance 0.125) ([t16-f02 readings](../progress/readings/t16-f02.md)
    `:391`).

**The founder is already factored.**
- `crates/v3-core/src/creature/cgp_founder.rs:144-243` writes:
  - `Move[d] = 0.5 + 0.4·ring[d] − 2g − 2q`;
  - `Reproduce[d] = g·(0.5 + 0.4·ring[d])`, on the four cardinals;
  - in all, 32 edges plus 4 `Multiply` nodes.
- The same votes in drive-plus-direction form:
  - `drive_Move = 0.5 − 2g − 2q` (3 edges);
  - `drive_Reproduce = 0.9g − 0.4` (2 edges);
  - `dir_K[d] = 0.4·ring[d]` per kind (8 edges);
  - 13 edges and no multiplies, or 9 edges if the two kinds shared one channel.
- A simulation of the commit rule found identical tick queues in 1,024 of
  1,024 cases:
  - lowest-index within-kind argmax, per-kind bar, incumbent kind tie-break;
  - all `f`, `g` in {0,1} and cardinal ring levels in {0, 0.25, 0.5, 1};
  - script: Appendix A.
- Scope of the check:
  - It tests a factored founder port, which is T18's question, not F30.
  - It covers the V3Alpha1 profile only, and it omits Terminate and Steal.
  - It used one shared direction channel, which is numerically equal to the
    per-kind form.
  - The port's g = 0 suppression of Reproduce rests on an exact zero at ring 1
    (−0.4 + 0.4).
- The hand-written controller is a "whether" plus a "where", written out once
  per direction.

**What it costs to reach a rule.**
- This is arithmetic from the draw probabilities, not a measurement.
- `AddGraphEdge` (`mutation/graph/operators.rs:167-187`, `:265-280`):
  - the target is uniform over compute nodes plus 94 sinks;
  - the source is an input leaf 30% of the time;
  - the reference is uniform over the node's references, the sub-value uniform
    over the width, and the weight sign 50/50.
- With the census means (3 compute nodes and 2 references per reachable Graph
  node), one specific correctly signed `ring[d] → Move(d)` edge has probability
  about 1e-4 per `AddGraphEdge` event on a node that declares the ring.
- For comparison, E4 measured that once the food summary is declared on the
  vote node, a correctly signed connection to a cardinal Move arises at 5.5e-6
  per birth ([diagnosis run](evolvability-diagnosis-run-2026-10.md) `:214`).

| Rule | Today | + kind drives | + projection gene |
| --- | --- | --- | --- |
| Toward food in all 8 directions | 8 specific edges, about 2.8e4 edge events if every partial edge were kept | same | 1 gene; about 150 events of the operator on a node that declares all three rings (family 1/3 × kind 1/3 × offset 1/8 × sign 1/2) |
| Away from barriers in all 8 | same, negative | same | 1 gene |
| Steal toward an occupied neighbor | 8 edges | same | 1 gene |
| Stop moving when full | one edge per wired direction (4 in the founder lineage), with direction-biased partial states | 1 edge | 1 edge |

## 5. External evidence

These are summaries in my own words. The research agents read the primary texts
for this note (2026-10-11), and the source list marks which were read in full.

**Biology: "whether" and "where" are separate outputs.**
- In the superior colliculus:
  - the location of population activity on the map sets the saccade vector;
  - the burst rate sets vigor;
  - omnipause neurons hold movement until collicular output crosses threshold
    (Gandhi & Katnani 2011).
- Basal-ganglia output gates whether gaze shifts, and to which stimulus
  (Redgrave et al. 2010).
- In the fly central complex:
  - a goal bump on a columnar ring sets turn direction through PFL3;
  - a global input scale sets vigor (Westeinde et al. 2024);
  - the "where" map alone does not start movement, and the gate sits downstream
    (Mussells Pires et al. 2024).
- Chemokinesis changes the rate of behavior with no direction chosen
  (Cheng 2022). Bacterial run-and-tumble is the limit case.

**Biology: "where" is not shared across behaviors.**
- Zebrafish approach and escape leave the tectum by separate tracts, and only
  the approach tract is ordered by retinal position (Helmbrecht et al. 2018).
- Rat colliculus maps defense and approach into almost entirely segregated
  circuits (Comoli et al. 2012).
- The sensory map is shared. The output channels are per category.

**Biology: direction symmetry is one developmental program.**
- Each central-complex columnar type tiles about nine columns with a repeated
  phase shift. The connectome authors call that topography genetically
  prespecified (Hulse et al. 2021).
- The bridge columns come from four neuroblast lineages. One temporal factor
  shifts subtype ratios across the whole ring (Sullivan et al. 2019).
- Vertebrate retinotopic maps are wired by a few complementary Eph/ephrin
  gradients, then refined by activity (Feldheim & O'Leary 2010).

**Computation.**
- **One gene for all eight directions is the best-supported idea, with
  exceptions.**
  - HyperNEAT's food-gathering task has Petri's shape: 8 sensors, 8 effectors at
    the same angles, argmax. Evolution found "connect same angle, inhibit the
    others" and repeated it around the ring. Direct encodings gain nothing from
    index order (Stanley et al. 2009).
  - Indirect encodings win on regular problems and lose on irregular ones. They
    win most when a direct layer can fix single connections afterwards
    (HybrID). They also produced far more large beneficial mutations: 5.8% of
    offspring more than 20× fitter, against 0.35% for a direct encoding
    (Clune et al. 2011).
  - Shared encodings struggle to prune single connections
    (Huizinga et al. 2014). Keep per-direction edges.
- **Splitting "whether" from "what" has weak support.**
  - Factoring helps in gradient reinforcement learning only for large joint
    action spaces with shared upstream features (Tavakoli et al. 2018).
  - Parameterized actions give each kind its own parameters
    (Hausknecht & Stone 2016).
  - The closest mutation-only analog is a selector neuron ("does this module
    act") beside an output neuron ("what it does"). Modules created by
    duplication specialized in 10 of 10 runs (Calabretta et al. 2000).
- **A shared "where" across kinds is common in artificial life but untested.**
  - Avida, The Bibites and biosim4's kill action share a facing.
  - Wagner & Altenberg (1996) argue that independent functions should be coded
    independently, so coupling Move, Steal and Reproduce through one "where" is
    cross-function pleiotropy.
- **Symmetry built into a policy speeds learning** when the symmetry is exact.
  Invariant kinds plus equivariant directions is the MDP-homomorphic grid-world
  case (van der Pol et al. 2020). The evolutionary analog is untested.
- **Lattice caveat.**
  - A square grid is symmetric only under 90° turns and reflections.
  - Petri moves are king moves with one `move_cost` and no movement corner
    rule, so an index shift of the eight-cell ring is consistent locally.
  - Area summaries and the visibility corner rule are not 45°-symmetric.
    Projections should therefore stay on the three rings.

## 6. Options

| Option | Design | Analog / precedent | Seek-all-8 in one step? | Kind-specific aim? | Verdict |
| --- | --- | --- | --- | --- | --- |
| A. Keep | 27 sinks | NEAT one-output-per-action; T11.F21 | No (8 edges) | Yes | The census shows the cost |
| B. Kind drives, per-kind direction channels | One drive per directed kind, folded into the kind's 8 sinks or as 3 more sinks; `e_K = drive_K + max_d dir_K[d] − bar_K`, direction unchanged | Colliculus gate and vigor; selector neurons; kinesis only with a non-north tie rule | No; a gate becomes one edge | Yes | **Recommended**, with a tie rule (§7). Commits as today when no drive is written. |
| C. Shared "where" across kinds | Drives plus one `Where[8]` (14 sinks), or plus exceptions (38) | Avida or Bibites facing; option D of the review | No (8 edges, shared) | Only by state or exceptions | Not recommended: conflicting targets in Petri, cross-function pleiotropy |
| D. Projection gene | One edge kind: ring family → a kind's direction channel, offset k, one weight, mutated as a unit; per-direction edges stay as exceptions | Central-complex columns; ephrin gradient maps; HyperNEAT plus HybrID; ring-wiring harness (correlated variation) | Yes | Yes | **User ruled yes** (2026-10-11); overrides check 3 and T20 `:181-182` for the rings |
| E. Heading vector (dx, dy) | Two outputs, quantized atan2 | Polyworld, Sims | Only from vector sensors | n/a | Rejected 2026-09-16; still holds |
| F. Egocentric turn and forward | Facing state | Avida (1 of 50 populations steered) | No | Shared facing | Rejected 2026-09-16; still holds |

## 7. Recommendation and decisions for the user

Backward compatibility is not a consideration (user, 2026-10-11). Stored
genomes, sink renumbering and draw re-mapping constrain nothing here. Expected
re-pins are predeclared, not avoided.

1. **Build B.**
   - Commit rule: `e_K = drive_K + max_d dir_K[d] − bar_K`; the argmax inside
     the kind is unchanged.
   - Keeping the direction term in `e_K` keeps T11.F21's one-edge property: a
     strong target both triggers and aims, as a collicular burst does.
   - Form, the spec owner's choice:
     - **Folded** (simplest): a drive edge adds its value to all 8 of the
       kind's sinks at accumulation, so the vote vector stays 27 wide and
       `vote_select` is untouched.
     - **Separate**: three more sinks that decision inputs can read apart.
     - Either way the drives stay out of each kind's direction argmax.
   - Every draw over the sink catalog changes its modulus, so the gate and goal
     epochs are expected to re-pin.
2. **Build D** (user ruled yes, 2026-10-11).
   - What it pairs is body geometry: sensor cell d and motor cell d are the
     same neighbor. Its offset and sign are drawn.
   - Birth weight (user, 2026-10-11): **zero**. Selection raises it. E5's
     `aligned` variant, which has the stamp's shape at weights up to 1, raised
     harmful births on wall 1.45×.
   - Order (user, 2026-10-11): **D first**. It does not need B, it is the
     larger lever, and B alone has precedent for not producing steering
     (T11.F21's per-kind gate era).
3. **Tie rule for B (user, 2026-10-11: option A).**
   - With a positive drive and a flat direction channel, the lowest index (N)
     wins. "Move more when hungry" would then mean "go north when hungry", which
     contradicts kinesis. The diagnosis run's own analog says the opposite: a
     naive forager's undirected search has no fixed compass heading
     (`evolvability-diagnosis-run-2026-10.md:506-516`).
   - **Chosen, option A:** a tie within a kind goes to the creature's heading.
     The heading is the direction of its last committed Move (the "run" of
     run-and-tumble). A newborn's heading points away from its parent, and a
     founder's is set at seeding. This changes the founder, so viability runs
     first.
   - Rejected, option B: keep North and drop kinesis from the analog.

## 8. Rows (placed 2026-10-11; the track rows are authoritative)

Order: T11.F31 first, then T11.F30.

- **T11.F30 — Kind Drives.**
  - Goal: Motor gating apart from aiming, as a collicular gate and burst vigor
    start a saccade, and kinesis with a heading that persists. A kind's urge to
    act is one heritable signal; where it aims stays per kind.
  - Depends on: T19.F06.
  - Scope:
    - one drive per directed kind, folded or separate (§7);
    - the new commit rule;
    - the heading tie (§7).
    - The drives join the destination sets of `RecruitNeutralInput`,
      born-connected `InputRef.Add` and `RefineHeritableStructure`.
  - The founder's wiring is left as it is. Its 13-edge factored port goes to
    T18; it gives identical queues but not an identical vote vector.
  - Readings:
    - predeclared gate and goal re-pins from the changed draw moduli;
    - helpful and harmful share of drive-edge births against matched
      per-direction edge births (a bare drive-edge count rises from supply
      alone and cannot fail);
    - duplicate per-direction gate edges per creature (today about 4 in the
      founder lineage);
    - steering-v1 not down, harmful-per-birth not up;
    - the founder's tie-state moves, now along its heading (viability first).
- **T11.F31 — Ring Projection Motif.**
  - Goal: A topographic map from one developmental program (central-complex
    columns, Eph/ephrin gradients): one gene wires a whole neighbor ring onto a
    kind's direction channel, born silent at zero weight, and single edges
    remain as exceptions.
  - Depends on: T19.F06, T20.F16. It targets today's per-direction sinks.
  - Scope:
    - a Graph edge kind `{ring ref, kind, offset, weight}` and a VM counterpart;
    - unit mutations (weight step, offset ±1, retarget kind, remove);
    - one adding operator that writes weight 0, with dispatch weight set by the
      spec owner;
    - `RecruitNeutralInput`'s ring-into-direction-group case arrives as a
      zero-weight projection rather than a column into one sink;
    - the spec decides whether a born-connected `InputRef.Add` ring landing on a
      direction sink becomes a projection, with harmful-per-birth predeclared.
  - Contracts the spec must settle (adversarial review, 2026-10-11):
    - genome-size units: one unit or eight. Carry cost, replication cost and
      per-unit mutation supply all scale with it, so predeclare the choice;
    - `InputRef.Prune` must count projection references
      (`mutation/input_ref/mod.rs:126-131`). Otherwise a prune renumbers
      references and re-points the projection at another family;
    - VM parity is nominal: no reachable live VM node votes (0 of 1,240). Build
      the opcode, or record an exemption against T11.F18;
    - `RefineHeritableStructure` (T20.F05, `GroupKey::Ring`) refines a
      projection through its own weight mutation; the spec states how its
      grouping skips projections.
  - Readings:
    - selection test: the offset and sign distribution of *retained*
      projections against the uniform 1/8 they are born with;
    - direction-generality census, traced through the bus and across nodes:
      creatures with an aligned ring rule covering at least 6 of 8 directions
      (today 0 of 200; the founder covers 4). Supply alone can meet this, so it
      is descriptive;
    - retention of a rule's directions across generations (today the founder's
      four survive in a median of 1, or 2 outside one clade);
    - steering-v1 exact-hit above the evolved 0.37 and avoidance above 0;
    - helpful share of projection births against ordinary edge births on
      production worlds;
    - harmful-per-birth not up.

## 9. Remaining uncertainty

- No source compares factored with per-direction outputs under mutation-only
  search. B rests on analogy and on removing a duplication cost that §4
  measured. Its evolvability gain is likely modest. D is the larger lever.
- The census is one world at one moment, on Graph wiring only.
- Ecology can still mute seeking (finding 7 of the motor note). E1 measured
  that a single steering edge pays 1.20 to 1.43× over its silent twin, so the
  world does reward aiming.
- E5's harm warning applies to D. A projection carries 8 weights at once, and a
  wrong offset or sign is a large step. The harmful-per-birth reading guards it.
- Do not run a T22 lab for this. The features' own gate and goal readings on
  production worlds are the proof.

## Sources

Read in full unless marked. Research date 2026-10-11.

- Cheng 2022, Proc R Soc B 289:20220237, doi:10.1098/rspb.2022.0237
- Gandhi & Katnani 2011, Annu Rev Neurosci 34:205, doi:10.1146/annurev-neuro-061010-113728 (author manuscript)
- Redgrave et al. 2010, Front Neuroanat 4:132, doi:10.3389/fnana.2010.00132 (abstract and selection sections)
- Helmbrecht et al. 2018, Neuron 100:1429, doi:10.1016/j.neuron.2018.10.021 (dissertation manuscript version)
- Comoli et al. 2012, Front Neuroanat 6:9, doi:10.3389/fnana.2012.00009 (abstract, introduction, discussion)
- Westeinde et al. 2024, Nature 626:819, doi:10.1038/s41586-024-07039-2 (main text)
- Mussells Pires et al. 2024, Nature 626:808, doi:10.1038/s41586-023-07006-3 (main text)
- Hulse et al. 2021, eLife 10:e66039, doi:10.7554/eLife.66039 (targeted sections)
- Sullivan, Warren & Doe 2019, eLife 8:e43482, doi:10.7554/eLife.43482 (partial)
- Feldheim & O'Leary 2010, CSH Perspect Biol 2:a001768, doi:10.1101/cshperspect.a001768
- Dunn et al. 2016, Neuron 89:613, doi:10.1016/j.neuron.2015.12.021 (counter-evidence: lateralized escape command; partial)
- Stanley, D'Ambrosio & Gauci 2009, Artificial Life 15(2):185, doi:10.1162/artl.2009.15.2.15202
- Clune, Stanley, Pennock & Ofria 2011, IEEE TEC 15(3):346, doi:10.1109/TEVC.2010.2104157
- Huizinga, Mouret & Clune 2014, GECCO 697, doi:10.1145/2576768.2598232
- van der Pol et al. 2020, NeurIPS, arXiv:2006.16908
- Cohen & Welling 2016, ICML, arXiv:1602.07576
- Hausknecht & Stone 2016, ICLR, arXiv:1511.04143
- Tavakoli, Pardo & Kormushev 2018, AAAI, arXiv:1711.08946
- Calabretta, Nolfi, Parisi & Wagner 2000, Artificial Life 6(1):69
- Wagner & Altenberg 1996, Evolution 50(3):967
- Kashtan & Alon 2005, PNAS 102(39):13773
- Yaeger 1994, Artificial Life III (behavior section)
- biosim4 `src/executeActions.cpp` at 45e808cc; Avida `cHardwareCPU.cc` at 47f13dad; The Bibites wiki (not code)

Not read: Berg & Brown 1972 (paywalled), Fraenkel & Gunn 1940 (lending only),
Georgopoulos et al. 1986 (abstract only).

## Appendix A: probes

- **Census.**
  - The sampler pages `GET /v3/simulation/snapshot?zoom_tier=detail` in 400×400
    tiles, pools the IDs, shuffles them with seed 20261011, and fetches
    `GET /v3/simulation/creature/:id` until it has 200 dumps.
  - For each reachable Graph node, the census reads every `ActionVote` sink
    edge with |w| > 1e-3:
    - an input leaf resolves through `input_refs[ref_idx]` and `sub_idx`;
    - an `UpstreamSlot(k)` leaf resolves to the ring channels that any
      reachable node writes into `CustomOutput(k)`;
    - a compute-node index at or above the node's compute count reads 0 in
      `runtime/cgp/sources.rs` and is counted as dangling.
  - The ring offset is `(sink direction − sub_idx) mod 8`, signed by the
    product of the two edge signs.
  - A first pass that skipped bus-routed leaves missed the founder's taxis
    entirely. Any census reading of direction rules must trace the bus.
- **Founder check.**
  - It replays the commit rule: lowest-index argmax within a kind, the per-kind
    bar, the kind argmax with the incumbent tie-break, and `q` set after a
    first-pass Reproduce.
  - It compares the founder's votes with the drive-plus-direction form over a
    grid of 1,024 inputs.
- The scripts were session scratch files. Both probes are short enough to
  re-derive from this description.
