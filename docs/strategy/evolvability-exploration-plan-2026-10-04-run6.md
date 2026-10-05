# Evolvability exploration, run 6: plan

Date: 2026-10-04. Status: plan (research workflow, three critics, Codex rounds 1–3, round 3 `ready`), written after
[run 5](evolvability-exploration-2026-10-run5.md) (outcome B), under the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md).
User direction (2026-10-02 side chat, unattended): change the scale, not the
knobs. Run fewer arms over much longer horizons and plot time-to-reach. Change
the starting genome. Check against the real world.

## Why a sixth run

- Run 4 found no arm beating an A/A control at 100 generations. No replicate
  reached on fresh seeds, so reach sat at a base rate of 0.
- Run 5 found that M4 (per-unit rate × 4) gives about three times more
  confirmed-helpful one-step children than the native rate, yet run 4's
  campaigns saw no M4 gain.

Two explanations remain. The horizon may be too short for any arm to reach.
Or one-step supply may not convert into reach at all. Run 6 tests both:
1. **Long horizons:** 1,000 generations with fewer arms.
2. **A different starting genome:** an admissible alternative founder start.
3. **The real world:** M4 against the default in production goal worlds.

## Rules

[Run 5's plan](evolvability-exploration-plan-2026-10-02-run5.md) governs
everything this section does not change, and through it runs 4, 3, 2 and 1.

1. **Branch and builds.**
   - The worktree `evolvability-exploration-6` branches from run 5's tip
     `8fb85fb7`, plus this plan. That tip has `--aa-control`, the fixed
     `r3_masking` probe and the screen probe.
   - **Campaign binary.** The pinned `instr:`-only A/A build
     `v3-lab-aa-instr` (sha256 `10876c1e…`, commit `2b69ee79`), the same
     binary run 4 used. Probes build from the branch.
   - **Production binary.** `v3-cli`, built in release from `main` at this
     plan's commit and pinned by sha256. No exploration code is in it.
   - Telemetry is off on every invocation (`--telemetry off`).

2. **Long-horizon campaigns** (part L, decides the outcome).
   - **One invocation per assay and seed batch**, with these arms:
     - the reference R;
     - the A/A control N (`--aa-control`);
     - M4 (`--arm m4=<run3 overlays>/m4.json`);
     - ALT, the alternative start (`--arm alt=empty.json:alt-<assay>-s<seed>.json`, rule 3);
     - the built-in controls, which cannot be switched off.
   - All arms share one calibration (threshold, grid point and scenes),
     because they run in one invocation.
   - **Sizes.** Population 32, **1,000 generations**, 8 replicates per seed
     batch, retention depth 2, otherwise `--quick`.
   - **Instruments.** `food-seeking-hunger` V2 (lifetime 200) and `wall-v1`
     (scale 2, lifetime 200).
   - **Seeds.** Fresh seeds from 11 upwards that calibrate on each assay,
     chosen in order by `--calibrate-only` before any campaign. Holdout seeds
     (101, 102, 111, 112, 121, 122) are skipped.
     - The first calibrating seed on each assay is the **pilot batch**. It is
       excluded from inference.
     - The next seeds are the main batches.
   - **Sizing from the pilot**, frozen before any main batch runs. The sizing
     script reads only R's and N's reach and the invocation's wall time.
     1. p̂_a is the pooled R and N reached fraction on assay a's pilot.
     2. **Joint model.** On each pair of assay a, N, M4 and ALT reach
        independently with probability p_a = max(p̂_a, 0.05). The shared N
        enters both arms' contrasts. The assays are independent.
     3. Joint B power is the Monte Carlo probability (20,000 draws, recorded
        seed) that all four bounds U < 0.25 hold, for each candidate pair of
        batch counts (k_food, k_wall), each in {6, 8, 10, 12} (48–96 pairs).
     4. The chosen pair is the one with the smallest projected part L time
        whose joint power is at least 0.8 and whose projection fits the 60 h
        cap. The projection is each assay's pilot wall time × k.
     5. If no pair reaches 0.8 within the cap, the affordable pair with the
        highest joint power is used. Its predicted power is recorded in the
        freeze row as an accepted C risk.
     - **Limits of the model**, recorded with it: it ignores dependence that
       scenes and selection streams induce between arms of one replicate, and
       it ignores masking. Masking can only lower B power. This is a planning
       forecast, not a coverage claim.
   - **Kill.** It replaces run 2 rule 6's 900 s awake-time kill for this run
     only (the user's option (a)).
     - Each lab invocation has a **4 h awake-time kill** (estimate 1.5–3 h).
     - A killed invocation is rerun once with an 8 h limit. An invocation
       still killed is `incomplete`.
   - **Byte cap.** `--byte-cap 1610612736` (1.5 GiB) per invocation (estimate
     0.3–0.6 GB).
     - A byte-cap stop (exit 3) is rerun once at twice the cap.
     - Before every invocation the run checks that the whole run 6 evidence
       tree stays under 40 GiB, and that free disk exceeds 20 GiB.
   - **Order.** Seeds run in order, alternating assays.
     - n is fixed by the sizing above. There is no truncation by time,
       because reached lineages stop early, so elapsed time carries outcome
       information.
     - If the frozen n cannot complete within the budget, part L is
       `incomplete`. Its completed batches are then reported descriptively
       only.

3. **ALT: exuberant silent afferents** (part S).
   - **Construction.** The production founder after K = 24 applications of
     T20.F04 `SingleChannel` recruitment's own sampler, each adding a weight-0
     edge.
     - The build calls the production recruitment rule where it is public,
       or reproduces it exactly, including its memory families and its
       ActionQueue channel handling.
     - A fixture checks that every draw lies in that sampler's domain.
     - If the sampler is reproduced rather than called, a distribution test
       compares it with the engine. Over 20,000 production births with
       `neutral_input_recruitment: SingleChannel` that hold exactly one
       recruitment event, the frequencies of (family, channel, sink) must
       match the reproduction's 20,000 draws at a chi-square p of at least
       0.01.
     - Only `main`'s operators and sources are used, so the pinned binary
       loads the result. A load through the pinned binary is part of the
       construction test.
     - There is one draw per assay and seed batch, the pilot and the
       confirmation seeds included.
     - Every draw is generated from a recorded seed, hashed and committed in
       the freeze row before any campaign runs. **No draw is rejected or
       replaced.**
   - **Natural analog.** Developmental exuberance: young nervous systems
     overproduce silent synapses, and experience strengthens or prunes them.
     Here selection and edge removal do that.
   - **Amendment to not-forcing check 4, for ALT only.** Check 4 bans
     authored or pre-wired founders. Run 6 admits one pre-wired start under
     three conditions:
     1. Its wiring is drawn by the production recruitment sampler, a uniform
        rule that names no assay.
     2. Every added edge has weight 0.
     3. Its draws are never selected on behaviour.

     The estimand is conditional on those frozen draws: does this kind of
     silent substrate at birth change time-to-reach?
   - **Parity, reported and never gating.** For each draw, the fixed ALT
     genome is compared with the founder on the development bank (not a
     holdout): identical votes and argmax actions on every tick until any
     death, and the score difference. Under the hunger cap, energy is capped,
     not replenished, so carrying and functional-complexity costs can change
     survival. Any divergence is reported as part of the start's immediate
     effect.
   - **Not-forcing reading.**
     - Check 1: the rule is uniform and names no sensor, sink or assay.
     - Check 3: source and sink are drawn independently. How many edges land
       on task-relevant pairs is reported afterwards.
     - Check 4: the amendment above, with parity reported.
     - Check 5: ordinary variation must still find a non-zero weight with the
       right sign and size on the right edge, plus any gating.
   - **Claim limit.** ALT asks whether path length depends on the founder,
     for these frozen silent-substrate starts, including their cost and
     supply confounds. It is not T18: T18 stays a roadmap feature
     (docs/workflow.md). At most, a positive ALT result is a lead routed to
     T18.F01 and T13.F08, never a founder change.
   - **Matched controls.** ALT's founder-only, mutation-off and
     shuffled-score controls run once, descriptively: one `--genome alt`
     invocation per assay at 100 generations on the pilot seed.
   - **Cost confound.** The 24 edges add about 28 genome units: carrying cost
     of about +0.003 energy per tick, and about 1.3 times the events per birth.
     Realised size, events per birth and energy drain are reported on every
     row.

4. **Statistics** (part L; the single analysis point comes after every
   campaign). For each arm X in {M4, ALT} and each assay, X is compared with
   the A/A control N by replicate.
   - **Why N, not R.** X and R share the same per-child mutation seeds, while
     N's are independent. Pairing X with N avoids that dependence.
   - **Reach** is tri-state: reached, not reached, or unknown for an
     incomplete replicate. An unknown never supplies evidence.
     - b counts pairs where X reached and N did not, with both known.
     - c counts pairs where N reached and X did not.
     - n is the frozen pair count per assay (rule 2).
   - **A candidate.** b − c ≥ ⌈n / 4⌉ on both assays.
   - **B for an arm.** On both assays, U = CP⁺(b, n) − CP⁻(c, n) < 0.25,
     using run 4 rule 4's construction: two one-sided 97.5 % Clopper–Pearson
     bounds, CP⁻ = 0 for c ≤ 1, joint coverage at least 95 %, valid across
     heterogeneous pairs. It also needs masked counts and no incomplete
     replicate.
   - **Masking** follows run 4 rule 4 and applies to B only, with N as the
     reference elite. `r3_masking` gains a reference-arm variable,
     `PETRI_R3_REF`, defaulting to `native`. Scene identity must hold.
   - **Descriptive readings**, which never decide:
     - Kaplan–Meier time-to-reach curves per arm, censored at the horizon,
       with reached fractions at generations 100, 250, 500 and 1,000;
     - R against N discordance;
     - final-best and validation gaps;
     - genome size and energy trajectories;
     - the ladder's first non-pass rung at the horizon.
   - **Tooling.** `analyze6.py`, with synthetic tests, is written and passes a
     Codex code-only review before the analysis point.

5. **Production check** (part P: descriptive, and M4's real-world check).
   - **Command.** For each run:
     `v3-cli --telemetry off run --ticks 20000 --sample-every 500 --seed S --config <recipe> --save-config <raw>/applied.json > <raw>/samples.ndjson`.
     - The sweep profile is not used. Its output omits the mutation counters
       outside goal world-set cases, and its raw reports would reach tens of
       GB.
     - `run` emits a tick sample every 500 ticks: population, mean energy,
       mean genome size, mesh nodes, generation, reproduction attempts and
       spawns, and mutation events attempted and applied (by domain and
       operator).
     - The default arm D uses the goal world file unchanged as the recipe.
     - M4 adds `{"mutation":{"per_unit_rate":0.02}}` through a jq deep merge
       into a recipe kept under `.bench-artifacts`. Only the overlay and the
       world file's hash are committed.
     - The saved applied recipe must show `per_unit_rate` 0.02 for M4 and
       0.005 for D.
     - `scripts/bench-wait` is not used, because it would wait on the user's
       live server.
   - **Worlds and seeds.**
     - The three goal worlds, with fresh seeds only: Orchards 1011 and 2011,
       Canyon 1022 and 2022, Confluence 1033 and 2033.
     - The goal seeds 11, 22 and 33 are never used.
     - That gives 3 worlds × 2 arms × 2 seeds = 12 runs.
   - **Bounds.** A 2 h awake-time kill per run. Output is tiny (41 samples),
     so the byte budget is checked against a 100 MB ceiling for the leg.
   - **Readings**, over ticks 15,000–20,000 (samples 30–40):
     - P1, persistence: mean population over the window, and whether the
       world went extinct (and at which tick).
     - P2, mean energy over the window.
     - P3, births per creature-tick: Δ spawns ÷ (mean population × 5,000).
     - Manipulation check: applied mutation events ÷ spawns over the whole
       run. Counters are cumulative from a known zero at tick 0; `run` emits
       no tick-0 sample. M4's must be at least twice D's in every pair. A run
       with zero spawns makes that pair's check unavailable, which makes part
       P incomplete. It is never passed vacuously.
     - Confounds: mean genome size, mesh nodes, generation.
     - **Extinction** is reported as an interval: from the last sample with a
       positive population to the first sample with zero.
       - Window means use the observed samples, so a population that dies out
         inside the window keeps its earlier positive samples.
       - Energy and births are undefined for samples with zero population.
   - **Reading rule.**
     - Each world's seed spread is |D(s1) − D(s2)|.
     - An M4 effect on a reading counts for a world when both seeds' gaps
       M4(s) − D(s) share a sign and each exceeds that seed spread.
     - A **production signal** is the same sign in at least 2 of 3 worlds.
     - **Harm** is a production signal against M4 on P1.
     - An absent signal is never called corroboration, because 2 seeds per
       world see only gross effects.
   - **Complete** means all 12 runs finished and every manipulation check
     held.
   - **Out of scope here.** The in-vivo helpful share and behaviour readings
     (eats, blocked moves) need goal-only or server instrumentation. Run 6
     leaves production code untouched.

6. **Outcome.** The run ends with exactly one of A, B or C. A takes
   precedence over B, and B over C.
   - **Confirming a candidate.**
     - Every A candidate (rule 4) is frozen, with its ledger row committed,
       before confirmation bank 1 is read.
     - Bank 1 is seeds 101 and 102, 8 replicates each: 16 pairs per assay at
       1,000 generations, in the same invocation shape. The ALT draws for
       those seeds were frozen in phase 1.
     - A candidate confirms when b − c ≥ 4 of 16 against N on both assays.
     - A bank seed that fails calibration on an assay spends the bank, and
       the candidate does not confirm (inherited rule).
   - **A.** At least one arm confirms, and also:
     - **M4:** part P is complete and shows no harm signal. M4's claim is
       stated as a **confirmed general assay improvement**: a supply change,
       with no recruited structure to ablate, so the recruitment ablation of
       run 1's claim rules does not apply. Part P is its real-world check. Its
       natural analog is restated before any T11 suggestion, because a
       uniform rate change is not a heritable mutator background. The note
       may suggest a T11 rate feature, which still needs its own goal-profile
       evidence.
     - **ALT:** the claim is capped as a lead routed to T18.F01 and T13.F08,
       with no recommendation, because production cannot load a genome-file
       founder.
     - A completed Codex review.
   - **B.** No arm is an A candidate, and both arms meet B (rule 4). The note
     states that only the declared effect was excluded.
   - **C.** Anything else, including:
     - a candidate that fails confirmation, when no other arm confirms;
     - part L `incomplete`;
     - an M4 candidate whose part P is incomplete;
     - an arm that is neither a candidate nor B.
   - "More exploration" is never an outcome. Part P changes the outcome only
     through A's harm and completeness requirements.

7. **Lessons from runs 4–5, built in.**
   - Every run 6 command is wrapped by one POSIX `sh` runner, `run6.sh`, with
     the awake-time killer. The host has no `timeout`.
   - The lab caps bytes itself. The production leg's sampled output is
     checked against its 100 MB ceiling.
   - Equivalence samples and checks use the founder strata, never a vacuous
     one.
   - Exact binomial tails above m = 2,000 use log space.
   - Watchers live in `sh` scripts, because zsh aborts on unmatched globs.

8. **Records.**
   - Committed:
     - a compact projection (per arm, assay, replicate and seed: reached,
       generation to threshold, censoring, final best and validation, genome
       size);
     - the analysis output;
     - the ALT draws' hashes;
     - the production projection (one row per sweep);
     - provenance hashes of the raw files.
   - Raw output stays under `.bench-artifacts/`.

9. **Budget.**
   - Machine time:

     | Part | Estimate |
     | --- | --- |
     | Pilot, 2 invocations | 3–6 h |
     | L, 12–24 main invocations (rule 2) | 18–60 h |
     | ALT matched controls | under 1 h |
     | P, 12 runs | about 6 h |
     | Confirmation, if any (4 invocations) | 6–12 h |

     The cap for part L is 60 h, applied through rule 2's sizing.
   - 400 turns, with closing from turn 360.
   - Codex reviews: this plan, the ALT construction and parity test, the
     analysis tooling, and closing.

## Phases

1. **Build and freeze.**
   - The ALT construction and parity test (a `lab:` test).
   - The `r3_masking` reference variable.
   - `run6.sh` and `analyze6.py`.
   - Seed calibration.
   - The freeze row: seeds, ALT draws and hashes, binaries.
2. **Pilot and size.**
   - Run the pilot batches.
   - Size n (rule 2) and commit it before any main batch.
3. **Part L.**
   - Run the main campaigns and the ALT matched controls.
   - Part P runs after part L, so the two do not share the CPU.
4. **Analysis** (the single analysis point), with the Codex review.
5. **Confirmation**, if there is an A candidate.
6. **Note and close.**

## Deliverable

`docs/strategy/evolvability-exploration-2026-10-run6.md`, with summaries in
`docs/strategy/evolvability-exploration-2026-10-run6/`, in run 5's form.
