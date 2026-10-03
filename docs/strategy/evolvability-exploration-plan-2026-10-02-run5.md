# Evolvability exploration, run 5: plan

Date: 2026-10-02. Status: written and reviewed while run 4's campaigns run.
A research workflow (code map, prior art, statistics, inherited rules), three
adversarial critics, and three Codex rounds (round 3 `ready`) shaped it.
Run 5 launches after [run 4](evolvability-exploration-2026-10-run4.md) closes,
under the [T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md).
User direction (2026-10-02, unattended): measure mutation effects directly
instead of running evolution, test larger weight jumps first, and give full
campaigns only to mechanisms that pass.

## Why a fifth run

Runs 1–4 place the bottleneck at the benefit rung. Mutations are supplied and
children survive, but almost none help.
- E3: 4 of 161 genome-changed founder children improve the food bank.
- E7 from `native-0`: 0 of 512 grandchildren improve it.
- 15 of 16 food reference replicates stall at the benefit rung.

A campaign reads that rung indirectly, through 100 generations and 16–48 pairs.
A **mutation-effect screen** reads it directly: many children of fixed
parents, each scored alone on fixed scenes, counted as helpful, neutral or
harmful.

The first new mechanism is **larger weight jumps**.
- `alter_edge_weight_in_def` (`crates/v3-core/src/mutation/graph/operators.rs`
  ~729) multiplies a weight by at most 1 ± 0.2, or adds ±0.1 when |w| ≤ 0.01.
- So it never flips the sign of a weight above 0.01.
- Run 3's wall witness path (C2) spends 16 of its 34 edits on re-weights.

Standard neuroevolution operators allow larger moves:
- NEAT replaces 10 % of perturbed weights (Stanley and Miikkulainen 2002).
- CGPANN redraws a weight in ±1 (Turner's CGP-Library).
- Montana and Davis (1989) add a Laplace draw.
- Heavy-tailed steps cross plateaus that small steps cannot (Yao, Liu and Lin
  1999; Doerr et al. 2017).

A screen pass is a benefit-rung lead on fixed parents. One-step measures
predict multi-step search poorly (Wagner 2008; Smith et al. 2002; Hu et al.
2020), so a passing mechanism still has to win a campaign under run 4's rules.

## Rules

[Run 4's plan](evolvability-exploration-plan-2026-10-02-run4.md) governs
everything this section does not change, and through it runs 3, 2 and 1.

1. **Branch and builds.**
   - Run 5's worktree `evolvability-exploration-5` branches from run 4's final
     branch tip. That lineage already holds run 3's `instr:` and `lab:`
     commits, M5's `proto:` commit `a78dc95b` and the A/A control.
   - One build serves the screen and the campaigns: that tip, plus J's
     `proto:` commit, plus the screen's `lab:` commit.
   - Instruments: frozen `food-seeking-hunger` V2 and `wall-v1` (scale 2). No
     instrument changes.
   - Run 5 takes three things from run 4's final note: its outcome, the
     holdout-bank status, and where its evidence lives in the main checkout.
     Run 4's elites must be copied to the main checkout at run 4's close.

2. **Mechanism J: heavy-tailed weight jump** (`proto:`, branch-only, default
   off, `policy-deviation`).
   - **Rule.** For each applied `AlterGraphEdgeWeight` event, J computes the
     native step exactly as now, consuming the same random draw. With
     probability p = 0.1 it then replaces the result with
     w + Laplace(0, 1), clamped to ±max(5, |w|).
   - **Sub-stream.** The jump decision and the jump value come from a
     SplitMix64 sub-stream seeded by three values XORed together:
     - the bits of the native step's sampled `f32` value (`u.to_bits()`;
       in rand 0.8.6 an inclusive `f32` `gen_range` consumes one `u32`);
     - the edge's weight `f32` bits, shifted left 32;
     - the edge's canonical identity, shifted left 48. The identity is
       `surface << 15 | (node_index & 0x7F) << 8 | (edge_index & 0xFF)`, where
       surface is 0 for compute inputs and 1 for sink inputs, all taken at
       event time.
     - J therefore consumes nothing extra from the engine's stream. Both
       native branches consume one `u32`.
     - With p = 0 it reproduces native children byte for byte.
     - With p > 0, a J child in whose birth no jump fired equals the native
       child of the same seed byte for byte. Rule 6 checks this on every
       parent, including births with several events.
   - **Field and overlay.** Field `weight_jump_probability` (default 0,
     skipped when 0 in serialization). Overlay:
     `{"mutation":{"weight_jump_probability":0.1}}`.
   - **Fixed constants.** p, the Laplace scale and the clamp are fixed here
     and never tuned on screen counts. This is the only variant.
   - **Counter.** A plain `pub` counter `weight_jumps_applied` on
     `MutationSummary` identifies jump births in records.
   - **Natural analog.**
     - Mutation effect sizes are heavy-tailed: most are small and a few are
       large (Eyre-Walker and Keightley 2007; Orr 1998).
     - Synaptic strengths are log-normal across orders of magnitude (Buzsáki
       and Mizuseki 2014).
     - A regulatory or dosage mutation can change a synapse's receptor
       expression by a large factor. A few residues can switch a receptor
       from excitatory to inhibitory (Galzi et al. 1992).
     - J reaches creatures only through ordinary mutation of the brain
       genome. It adds no sensor.
   - **Not-forcing checks.**
     1. J is a general rule over every edge weight. It names no sensor,
        arena, action or scene.
     2. Its analog is named above.
     3. Before clamping, its proposal is symmetric and centred on the
        parent's value, so it aims at no value such as −2.0. It pairs no
        source with a sink. The clamp bound is never below |w|.
     4. Its constants are fixed in advance. No proposal is retried or kept
        by score.
     5. Ordinary variation still has to find which edge, the sign and size
        of the change, and all of food's vote-edge structure and wall's
        ring structure.
   - **Matched controls.** Not applicable, because J does not change
     evaluation semantics.
   - **Branch verification** (TDD, `$rust-skills`). Default-off fixtures:
     - p = 0 gives byte-identical children over 10,000 seeds;
     - with p > 0, over 10,000 seeds on the founder, the barrier comparator
       and a grown multi-node genome (12 native births from the founder; it
       stands in for an elite, and rule 6 checks the real elites at run
       time): every birth's applied operator sequence and targets equal the
       native twin's, and only weight values differ;
     - a fixed test vector pins the seed, the identity and the first draws;
     - the clamp bound holds;
     - the Laplace draw uses an open interval;
     - the counter is correct.
     `make check` passes, apart from the known recipe-digest pin.

3. **Strata, parents and banks**, all frozen and hashed in a freeze row
   before any screen read.
   - **Stratum.** One parent on one assay.
   - **Parents per assay:**
     - the founder;
     - 15 distinct reference final elites from run 4's fresh batches
       (`m1-<assay>-s<seed>/elites/native-<i>.json`, seeds 5–10). They are
       taken round-robin over the seeds, lowest replicate first, skipping
       duplicate genome hashes. If fewer than 15 distinct elites exist, the
       rest come from run 3's frozen-size reference elites (seeds 3 and 4)
       in the same order.
     - That gives 32 strata in total.
   - **Setups.** Food uses the hunger setup (energy 20, ceiling 20). Wall uses
     the wall setup (start energy 100, no ceiling).
   - **Bank A (development).** Food `0x3DF00D`, wall `0xB2DE00`, 32 scenes
     each. Each parent's frozen executed record is taken from its run on
     bank A's last scene, as in E4.
   - **Bank B (confirmation).** 32 fresh scenes per assay from the same
     specification, food seed `0x5BF00D`, wall `0x5BBA44`. Bank B is internal
     to the screen, not a holdout.
   - **Never read:** holdout banks 1–3, the instruments' acceptance sets, and
     the probes' `accept` and `accept2` banks.

4. **Children and classes.**
   - **Children.** Every child is a natural production birth:
     `apply_mutations_with_food_type_count` with the arm's resolved `mutation`
     block, from the parent's frozen record.
   - **Seeds.** Child i of a stratum uses a seed from the lab's
     `rng::stream` over (`"run5"`, parent hash, assay, i). Every arm uses
     the same seed for child i, so each arm child has a native twin.
   - **Classes.** Δ_A is the child's bank-A mean minus the parent's.
     - Helpful: Δ_A > 0.05 (the C2 band).
     - Harmful: Δ_A < −0.05.
     - Neutral: otherwise.
     - Identical children are neutral and are not evaluated.
   - **Confirmed helpful:** helpful, with Δ_B > 0.05 on bank B. Only helpful
     children are scored on bank B.
   - **Sterility check.** For every confirmed-helpful child, run 3 rule 3's
     two abandoning copies of the parent are scored on bank A. The child is
     `sterility-attributable` if the parent's better copy gains at least
     Δ_A − 0.05. Such children count as not helpful in every test below.
   - **Forced events.** None are reported. Positive controls (rule 6) are
     diagnostic genomes under run 2 rule 7.

5. **Statistics.**
   - **J's primary test** (decides): a twin sign test.
     - Over all strata, b counts the twin pairs where the J child is
       confirmed helpful and its native twin is not. c counts the converse.
       m = b + c.
     - Pairs are independent across seeds. Under the null, each stratum's
       J-only probability is at most its native-only probability, so b given
       m is stochastically at most Binomial(m, ½), even when strata differ.
     - The p-value is the exact one-sided binomial tail.
     - Only the pairs where a jump fired can be discordant, so this compares
       jump births with step births directly, on natural births.
   - **Estimands for the negative test**, per assay over its n = 16 N twin
     pairs, each the mean over pairs of a per-pair probability:
     - P̄_J = mean P(J-only) and P̄_N = mean P(N-only);
     - the **discordance odds** θ = P̄_J / P̄_N;
     - the **absolute excess** δ = P̄_J − P̄_N.
     - Over the jump-fired births, θ < 2 implies a helpful-rate ratio below
       2. That ratio is (both + J-only) / (both + N-only), which lies between
       1 and θ. Non-fired pairs add zero to both P̄.
   - **Bounds** use run 4 rule 4's conservative construction.
     - CP⁺ and CP⁻ are the exact one-sided 97.5 % Clopper–Pearson bounds on
       b / n and c / n, with CP⁻ = 0 when c ≤ 1 and CP⁺ = 1 when b ≥ n − 1.
     - Run 4 shows these cover a mean of independent, non-identical Bernoulli
       trials (Hoeffding 1956, Theorem 4; Greenberg and Mohri 2014). Joint
       coverage is at least 95 % by Bonferroni.
     - θ_U = CP⁺ / CP⁻, infinite when CP⁻ = 0.
     - δ_U = CP⁺ − CP⁻.
   - **Size.**
     - A J pilot on separate seeds (tag `"run5-pilot"`, 20,000 children per
       stratum, then discarded) counts m₀ on each assay.
     - Per assay, N = clamp(⌈20,000 × 150 / max(m₀, 1)⌉, 20,000,
       1,000,000), the same N for every stratum of that assay.
     - The target m = 150 per assay is chosen so that under θ = 1 (b ≈ c ≈ 75)
       θ_U ≈ 1.6, below 2.
     - Before N is committed, the freeze row records the expected m, the
       sign test's power at θ = 2, and the total runtime from the pilot's
       measured throughput. The runtime covers both twins, the A/A check, the
       PC-J pilot and main draw, and the comparison set.
   - **J secondaries** (reported, never deciding): one-step rates per drawn
     child, the harmful rate, the upper-tail mass (Σ Δ_B over confirmed
     helpful children ÷ N), per-assay and per-stratum b and c, and jump
     births apart from step births.
   - **Comparison set** (descriptive only; never starts a campaign in this
     run): run 3's six M arms, each seed-paired with native.
     - One step for refinement, M4 and M5: the same twin sign test at
       N = 4,096 per stratum.
     - Two steps for M1, M2 and M1 + refinement, which add weight-0 edges:
       - take the first 64 genome-changed neutral children per stratum, for
         the arm and for native;
       - give each 16 grandchildren from its own record, re-frozen on bank
         A's last scene (the E7 pattern), each scored against the original
         parent;
       - the unit is the child, read as having at least one
         confirmed-helpful grandchild;
       - report the rates with a stratified exact (Mantel–Haenszel) p-value.
     - Each M arm's screen reading sits beside its run 4 campaign verdict in a
       cross-table. Agreement neither validates nor refutes the screen.

6. **Validity and positive controls.**
   - **The screen is void** (every J label `inconclusive`) unless:
     - rule 2's fixtures pass;
     - a rerun of both founder strata is byte-identical;
     - **twin identity:** on every parent, for 10,000 seeds, every J child in
       whose birth no jump fired equals its native twin byte for byte. A fired
       birth may still equal its twin, for example when a later event deletes
       the jumped edge.
     - **fire rate:** J's jumps per applied weight event lie within the
       99.9 % binomial range around p;
     - **A/A:** the native arm against native on independent seeds (tag
       `"run5-aa"`, the same N), read with the twin sign test on all strata,
       gives a two-sided p of at least 0.01.
   - **PC-J** decides only whether J may record a supported negative. It is a
     wall diagnostic genome: the built-in barrier comparator with one
     inhibitory ring edge changed. It is constructed before any read, in
     this order:
     1. the first inhibitory ring edge, in C2 order, set to −1.0;
     2. if precheck fails, the same edge sign-flipped.
     - **Precheck** (deterministic, before any read): on both banks, the
       genome scores at least 0.05 below the comparator, and restoring the
       edge's comparator value is confirmed helpful and not
       sterility-attributable.
     - **PC-J pilot.** 100,000 twin pairs on separate seeds (tag
       `"run5-pcj-pilot"`) count b₀ and c₀. Treating b and c at
       N = 1,000,000 as Poisson with means 10 b₀ and 10 c₀, the exact power
       of the twin sign test at p ≤ 0.025 is computed.
     - PC-J is `powered` if that power is at least 0.8. It then holds when
       J's twin test on the main seeds of that one stratum gives p ≤ 0.025,
       with N = 1,000,000.
     - A powered PC-J that fails blocks J's supported negative. A
       comparator-based diagnostic that J cannot repair indicates a pipeline
       fault.
     - If neither genome passes the precheck, PC-J is `not constructible`. If
       the power is below 0.8, it is `underpowered`. In both cases it is
       reported and does not gate, and the negative rests on its bound and
       the void checks above.
   - **PC-F** (descriptive): run 3's C2 food step 9 genome, one edit before
     the +19.26 vote edge. Native's confirmed-helpful count in 4,096
     children is reported, which shows the pipeline can see a known helpful
     step.

7. **Screen outcomes for J.**
   - **Pass**, which needs all of:
     1. the primary test, pooled over both assays, gives p ≤ 0.025;
     2. b / c ≥ 1.5 (b > 0 with c = 0 counts as met);
     3. on each assay with m ≥ 10, b ≥ c;
     4. dropping the stratum with the largest b − c keeps p ≤ 0.05;
     5. a valid screen.
   - **Supported negative**, which needs all of:
     - not a pass;
     - on each assay, either θ_U < 2, or δ_U < 2.5 × 10⁻⁴ per child;
     - PC-J holds, unless it is not constructible or underpowered;
     - a valid screen.
   - **The δ alternative.** It covers an assay where neither arm finds helpful
     weight changes, so m stays small even at the size cap. 2.5 × 10⁻⁴ per
     child is under one extra confirmed-helpful birth in a 3,200-birth
     campaign replicate (32 × 100).
   - **Narrow claim.** Two forms, as recorded per assay:
     - through θ: among births where a jump fired, jumps are not twice as
       often confirmed-helpful as the native step;
     - through δ: J adds under 2.5 × 10⁻⁴ confirmed-helpful children per
       drawn birth, over all natural births and not only fired ones.
   - **Inconclusive:** anything else.
   - **Predicted** (from the prior art):
     - food: no gain at the founder, and more harm (food's helpful steps are
       new vote edges);
     - wall: a gain, if anywhere, at elites and at PC-J.

8. **Campaigns for J**, if J passes the screen.
   - **Arms.** J runs as an overlay arm under run 4 rules 1, 2, 4, 5 and 6
     unchanged: the A/A control, 48 fresh pairs per assay, and rule 4's
     bounds and masking.
   - **Seeds.** The first six seeds from 11 upwards that calibrate on each
     assay, skipping the holdout seeds. The screen's parents come from seeds
     5–10, so no campaign shares a seed with a screen parent.
   - **Invocations.** Two per combination, as in run 4: R with N (the A/A
     control), and R with J.
   - **Identity (rule 5).** Before any campaign read, the build with J off
     reproduces run 4's `native` and `native-aa` rows on `food-s5` and
     `wall-s5` byte for byte, and its projection without `config_digest`.
     Within each new combination, the two invocations' R rows and projections
     must match.
   - **Ablation** (P1's claim rules), built only if J passes the screen and
     before its campaigns run. The lab adds a jump-attribution map per
     individual, a harness change on the branch.
     - A child first copies its parent's marks. Then its birth's events
       update them in order:
       - a structural edit on a node clears the marks on that node's edges,
         which is conservative;
       - a jump on an unmarked edge marks it with the weight before the
         jump;
       - a jump on a marked edge keeps the first mark;
       - a native re-weight keeps the mark.
     - Each elite's map is written with its genome. Only marks are written,
       never per-proposal records.
     - The ablation sets every marked edge back to its recorded pre-jump
       weight. The gain must go under P1's ablation rule.

9. **Run outcome** (about J). The run ends with exactly one of these:
   - **A.** J passes the screen, and then all of the following hold. The note
     may then suggest a roadmap feature in the owning track (T11). Under T22,
     a lab result never justifies a production default alone.
     - its campaign meets run 4 rule 4's A candidate on both assays (strict
       coding, unmasked, b − c ≥ ⌈n / 4⌉ against the A/A control);
     - run 3 rule 3's sterility counterfactual;
     - confirmation under run 4 rule 6 on the next sealed bank that run 4
       left unspent;
     - the ablation, incumbent competence and the natural-world check;
     - a completed Codex review.
   - **B.** Either J's screen is a supported negative, or J passes the screen
     and its campaign meets run 4 rule 4's B. The note states that only the
     declared effects were excluded.
   - **C.** A concrete blocker, a void screen, or J `inconclusive` at its
     screen or campaign.
   - The note also carries an execution status and, if incomplete, a resume
     handoff. "More exploration" is never an outcome.
   - The comparison set's readings are descriptive and never change the
     outcome.

10. **Records.**
    - Aggregates only:
      - counts per stratum × arm × class × applied operator;
      - one record per confirmed-helpful child, holding Δ_A, Δ_B, the jump
        flag, the genome diff, the target node and the sterility reading;
      - fidelity (requested, applied and skipped events, and the identical
        share).
    - Raw output stays byte-capped under `.bench-artifacts/lab/`. Exhaustive
      per-proposal records are never written (T22).

11. **Budget, kill and reviews.**
    - 400 turns from launch; closing starts by turn 360.
    - Every invocation runs under the awake-time 15-minute kill.
      - Each stratum is split into child-index ranges, sized from the pilot's
        throughput to finish within 10 minutes. Ranges are selected by
        environment variable, and each is its own resumable output file.
      - A merge step checks that every range is present exactly once.
      - The test binary is built once with `cargo test --no-run` and run
        directly.
    - Estimate:
      - J pilot and main draw: about 1 hour;
      - comparison set: about 2 hours;
      - campaigns, if any: about 4 hours.
      The pilot's measured throughput replaces this estimate.
    - Codex reviews: at the end of the screen phase, at the end of the
      campaign phase, and before any recommendation. Every finding gets a
      disposition row.
    - Close order as in run 1's plan.

## Phases

1. **Build and freeze.**
   - J with TDD, then the screen probe, then the build.
   - Rule 6's identity fixtures, and rule 8's identity check on `food-s5` and
     `wall-s5`.
   - Construct PC-J and run its precheck.
   - Commit the freeze row before any screen read.
2. **Pilot.** Run the pilot, set N, and commit N.
3. **Screen.**
   - J's main draw, the A/A check, PC-J, PC-F and the comparison set.
   - Analysis, cross-table and the Codex phase review.
4. **Campaigns** (rule 8). Skipped unless J passes.
5. **Confirm** (rule 9), for an A candidate only.

## Deliverable

`docs/strategy/evolvability-exploration-2026-10-run5.md`, with compact
summaries in `docs/strategy/evolvability-exploration-2026-10-run5/`, in run
4's form.
