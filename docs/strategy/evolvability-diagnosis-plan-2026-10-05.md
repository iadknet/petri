# Evolvability diagnosis run: plan

Date: 2026-10-05. Status: plan for one unattended run under the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md),
executing the experiment ledger of the
[evolvability diagnosis note](evolvability-diagnosis-2026-10-05.md) (read it first; this
plan does not repeat its argument) and the sibling
[recent-research scan](alife-recent-research-scan-2026-10-05.md) of the same day, whose
fluctuation instrument is row E11. The run writes
`docs/strategy/evolvability-diagnosis-run-2026-10.md` and ends with a plan for what to do
next, not with a feature. Not yet reviewed by Codex; the run's first act is that review.

## Question

Which of three places blocks the evolution of sensor-driven behavior on the current
substrate: the world (the first step toward sensing does not pay), the variation (the
genome cannot build the wire in a reasonable number of births), or the instrument (the
premise was never true at production scale and horizon)? And, given the answer, what
should Petri change?

## What the previous runs' plans still govern

The [run 1 plan](evolvability-exploration-plan-2026-10-01.md) governs everything this
plan does not change: the five not-forcing checks, matched controls, the claim rules,
`lab:` / `instr:` / `proto:` / `docs:` commit kinds and what may land on `main`, the
lab/production boundary, baseline identity hashes for any instrument this run reuses,
the telemetry rules, the Codex review command, and the close order. The
[run 6 plan](evolvability-exploration-plan-2026-10-04-run6.md)'s lessons (one POSIX
`sh` runner with an awake-time killer, byte caps, log-space binomial tails) carry over.
Where this plan and an earlier one disagree, this plan wins.

## Rules

1. **Where.** A Claude worktree from a clean, current `main`, `EnterWorktree` named
   `evolvability-diagnosis`. Record the launch revision. `npm ci` in `frontend/`,
   at least 30 GiB free. Run 6 may still be running in
   `.claude/worktrees/evolvability-exploration-6`: never touch that worktree, its branch
   or its processes.
2. **Host sharing.** Before every heavy job (a world run, a campaign, a twin screen, a
   build of the whole workspace) check for run 6's jobs with
   `pgrep -fl 'v3-lab-aa-instr|v3-cli'`. If any is present, run only the light rows
   (E4, E3's analytic half, probe builds, note writing) and re-check every 20 minutes.
   Never kill another session's process. One heavy job at a time.
3. **Kills and caps.** Each world run or census has a 3 h awake-time kill; each twin
   screen 2 h; each lab campaign 1 h. A killed job is `incomplete` and is rerun once at
   double the limit. Byte cap 1.5 GiB per job; the run's evidence tree stays under
   40 GiB under `.bench-artifacts/lab/diagnosis/`. Telemetry off on every invocation.
4. **Code that may change.** As run 1 defines them, plus: observation-only changes to
   `v3-core`'s `neighborhood` modules (new diagnostic arms for the opportunity assay,
   census checkpoints, a snapshot reading) are `instr:` commits; the `AddProjection`
   operator and the replication-time arm are `proto:` commits behind default-off config
   fields that reproduce `main`. Experiment world recipes are data files under
   `docs/strategy/evolvability-diagnosis-run-2026-10/worlds/`, never under
   `experiments/worlds/`. The pinned production binary for E8's default arm is built
   from the launch revision.
5. **Diagnostic arms.** E1, E7 and E9 seed authored genomes into worlds or scenes. They
   measure the world and the instrument, never discovery; they are labelled
   `diagnostic` in every row and summary and are never recommended as mechanisms.
6. **No `main` changes** beyond the close order: no production default, founder,
   mutation policy, recipe or roadmap row.
7. **Predeclare** every row's summary (hypothesis, method, sizes, prediction,
   falsifier, decision rule) in the note and commit before the row runs. `incomplete`
   rows are kept.
8. **Roles.** The session (Opus 5.5) is the implementer: it writes the probes, runs
   the rows, reads the results and writes the note. The Fable advisor (enabled with
   `/advisor fable` before the goal is pasted) is consulted, through the `advisor`
   tool, at every one of these points, and each consultation's advice and the
   disposition of each point it raises are recorded in the note's "Advice" table:
   - before phase 0 starts (the approach to the whole run, after this plan is read);
   - before each phase's approach is chosen, and when naming the verdict branch;
   - after each row's reading is drafted and before it is recorded, so the
     interpretation is challenged while the raw output is still at hand;
   - whenever a row is `incomplete`, a result contradicts its prediction, or an
     approach is not converging after two attempts;
   - before the plan forward is drafted and again after it is drafted;
   - before closing.
   Fable advises; it does not write code, run rows or edit the note. If the advisor
   tool is unavailable or returns a credit error, the run records that, sends the
   same brief to Codex as an advice job, and continues.
   **Reviews** stay with Codex `gpt-6.1-sol` `high`, read-only, through `codex exec`
   as run 1 describes: (a) this plan before any row runs; (b) after phase 1; (c) the
   draft plan forward before phase 4 closes; (d) closing. A Codex usage limit marks
   the review `pending`, the run continues, and the note recommends nothing until one
   review of the plan forward has completed.
9. **Decide, do not ask.** Choices off the stop list below are the run's; record them.
   Stop and ask only when disk falls under 20 GiB, a step would change `main` outside
   the close order, or a reading needs one of the two user rulings (index alignment,
   the lab's role) to proceed; then record the question in the note and continue with
   rows that do not depend on it.

## The ledger

Rows are the diagnosis note's E1 to E10, made executable. Sizes are fixed here; the
run may shrink a row to fit a kill and must say so.

| ID | Phase | Method, instrument, sizes | Wall | Prediction | Decision rule |
| --- | --- | --- | --- | --- | --- |
| E0 | 0 | Baseline identity: rerun T20.F01's `input-opportunity` pilot (1 replicate per world, 3 threads) at the launch revision; store its summary hash. After the E1 `instr:` commit, the pilot's existing eight arms must reproduce it | 7 min | Hashes equal | Unequal hashes block E1 until fixed |
| E1 | 1 | `instr:` arm set `one-edge` for `input-opportunity` (`neighborhood/opportunity/controllers.rs`): on the founder's vote node, (a) `AreaFoodSummary` `nearest_dx` (sub-value 3, the component `A_vector` steers on, same sign convention as its `Move(E) += gx`) → `Move(E)` at +0.5, ungated; (b) the same edge at −0.5; (c) `NeighborBarrierRing[E]` → `Move(E)` at −0.5 (one cardinal of `A_ring`); each with its zero-weight twin `Z`, plus `F` and `I` as T20.F01 defines them: eight arms. Competence fixtures as T20.F01 (the authored edge may change only the intended vote). Three goal worlds, 8 replicates, 1,000-tick horizon, 3 threads. Read A/Z and A/F birth ratios under T20.F01's verdict rule (positive needs 7 of 8 informative replicates above 1.05) | 30 to 60 min | (a) 1.0 to 1.1, (b) below 1.0, (c) about 1.0 | **World branch** if no one-edge arm is positive in any world where `A_vector` was positive; **variation branch** if (a) or (c) is positive anywhere; otherwise mixed, and E6 decides |
| E2 | 1 | `instr:` probe `census_run` (ignored test or `v3-lab` subcommand): run `Simulation` on Canyon and Confluence at 1600² with the goal seeds to ticks 10,000, 20,000 and 50,000; at each checkpoint call `neighborhood::input_use::observe` exactly as `v3-cli/src/bench/input_use.rs` does and write the family rows and funnel stops; at 20,000 also write the E6 reading. Also `v3-cli run --sample-every 500` samples for population, genome size and generation | about 90 min per world plus censuses | Vector and barrier families declared and connected at every checkpoint; causal use rare and not rising; genome size grows | **Premise false** if `family_causal` for `AreaFoodSummary` or `NeighborBarrierRing` rises across checkpoints and `retained_share` stays above 0.9: the block is horizon, and the plan forward is a horizon change, not a mechanism. **Variation branch** if connected rises and causal does not |
| E3 | 2 | Selection coefficient of intermediates: analytic from carry cost (10⁻⁴ per unit per tick), perception-assembly cost and the E4 removal rates, over the E2 lifetime distribution; then run 5's twin screen (cherry-pick `run5-screen` and run 3's `instr:` commits) on the founder plus one declared reference, the founder plus one zero-weight edge, and the founder plus one wrong-sign edge, 20,000 twins each | 1 h | Declared and zero-weight neutral within 10⁻³; wrong sign harmful 10 to 20 % | A measurable cost on the declared or zero-weight state becomes the first repair candidate |
| E4 | 1 | `lab:` probe `transition_rates`: 10⁶ one-birth children of (a) the founder, (b) founder plus a declared `AreaFoodSummary` on the vote node, (c) founder plus that declaration and one zero-weight edge to `Move(E)`, through the production engine at the production config; count per birth: declarations by family and node, prunes of the new entry, `AddGraphEdge`/`RetargetGraphEdge` landing on that leaf by sink kind, sub-value and sign, `RemoveGraphEdge` and `AlterGraphEdgeWeight` on the new edge. Repeat on three run 4 reference elites | minutes | Within a factor 3 of the note's Section 2.1; prune-to-useful-connect ratio above 10³ | Ratio under 10 refutes the cleanup diagnosis and drops the operator-weight candidate |
| E5 | 2 (variation branch) | `proto:` operator `AddProjection` (field `projection_growth`, default off): one event declares a family if absent and wires every channel to every sink of one vote kind; variants `random` (per-edge weights in [−1, 1]), `shared` (one weight, random sign, all pairs), `aligned` (channel *i* to sink *i* only, one shared weight and sign). Twin screen on the founder and 15 run 4 elites, both repaired assays, 100,000 twins per stratum, A/A check | 1 to 3 h | `random` no better than native; `aligned` several times more often confirmed-helpful and more often harmful | `aligned` not above native drops the projection candidate. `aligned` above native and `shared` not: index alignment is what matters, and the note states the check-3 question with these numbers |
| E6 | 1 (from E2) | At tick 20,000 in each E2 world, for every living creature: `AreaFoodSummary` `nearest_dist` and gradient magnitude from the production sensor assembly; classify as inside food (food here or on a cardinal ring cell), gradient-informative (food visible, none adjacent), or blind (nothing visible); and the energy expected from one step along the gradient against one random step | minutes | Under 20 % gradient-informative | Over 40 % gradient-informative means the world is not the block and E7 is skipped |
| E7 | 2 (world branch) | Experiment worlds at 800² with 2,500 founders (persistence first, 20,000 ticks): food in patches spaced beyond the founder's ring with regrowth that returns on the patch's far side, move cost raised until a random walker's expected intake is under its decay, offspring placed within a radius of 3. Then founder plus 1 % `A_vector` (the opportunity machinery with a recipe path and a 50,000-tick horizon, an `instr:` change), then founder alone for 50,000 ticks with the E2 census | 30 min per read; up to 3 h | Founder alone persists at low density or goes extinct; followers sweep; founder alone evolves causal food reads | Followers not sweeping means demand is still too weak at that geometry: iterate the world twice at most, then record |
| E8 | 3 | `v3-cli run --ticks 20000 --sample-every 500` on Canyon, fresh seeds 1022 and 2022: default; `per_unit_rate` 0.02; `proto:` arm where a `Reproduce` commit occupies `1 + genome_size / 97` ticks (the parent acts on nothing else while it lasts). Read mean genome size, births per creature-tick, persistence | 2 h | Default near 5 × founder by dilution; ×4 grows; time cost holds near founder | Default past 10 × founder by 20,000 ticks moves the size defect ahead of the encoding defect in the plan forward |
| E9 | 3 (if time) | Lab calibration against a known production positive: an authored scene set with fruit only west of the start, comparator = founder plus one `fruit ring[W]` → `Move(W)` edge; the lab at population 32 × 100 generations from the founder, 8 replicates | 1 h | Not reached | Not reached is written into the T22 amendment candidate: no lab negative is admissible about production |
| E10 | 3 | Read run 6's note when it lands on `main` (check at every phase boundary) | — | No ALT excess | A positive ALT reorders the plan forward behind a dense-afferent variant of E5 |
| E11 | 2 (both branches) | `instr:` instrument from the [recent-research scan](alife-recent-research-scan-2026-10-05.md): `food-seeking-hunger` with two mirror-image frozen layouts A and B (the development bank mirrored in the scene frame), switched every k generations with the scan's jitter, k in {3, 30, 300} as a pilot grid with one k chosen for the main batches; arms: switching, random-layout (predictability control), stationary V2 (reference); population 32, 300 generations, 8 replicates, seed batches 3 and 4; calibration, comparator and sterility repair unchanged. Reading: run 5's twin screen on each arm's final elites, scoring confirmed-helpful one-step children on the *other* layout (the alternate-neighborhood analog of Kumawat's reading); reach after a switch and lag descriptive only. T22.F03 ablation sentinels per generation record whether removing each silent node is neutral or deleterious (the Walsh masking reading, no arm) | 3 h | Switching elites have at least 3 × the stationary elites' alternate-layout helpful share | Positive: a lead for T02.F01's priority and for a non-stationary production world in the plan forward, never a stepped season. Negative at that size: the fluctuation lever is closed for this instrument class |

## Phases

0. **Build and freeze.** Worktree, builds, E0. Codex review (a) of this plan.
1. **Premise and gradient.** E4 first (light), then E1, then E2 with E6. Codex review
   (b). Name the branch: world, variation, premise false, or mixed.
2. **The branch.** Variation: E3, then E5. World: E7. Mixed: E7 first, then E5.
   Premise false: skip to phase 3. E11 runs on every branch except premise false,
   after the branch's own rows, because it reads neighborhoods rather than reach.
3. **Size, calibration, run 6.** E8, E9 if time, E10.
4. **The plan forward.** Written into the note as its last section (format below),
   reviewed by Codex (c), revised, closing review (d).
5. **Close** in run 1's order: note and summaries committed on the branch; kept
   evidence copied to the main checkout's `.bench-artifacts/lab/diagnosis/`;
   `ExitWorktree` with keep; recheck `main`; cherry-pick `docs:` commits and `lab:`
   commits that build without `instr:` or `proto:`; `make check` if a `lab:` commit
   landed, else `make check-docs`; remove the worktree, keep the branch.

## The plan forward (format)

The note's last section, titled "Plan forward", holds exactly these parts:

1. **Verdict.** One of: world, variation, premise false, mixed. Two sentences and the
   rows that decided it.
2. **What to change, in order**, as roadmap feature candidates: for each, the track it
   belongs to (T12 or T02 for a world, T11 for operator weights, T11 or T20 for a
   projection unit, T03 for replication time), its natural analog in a sentence, the
   evidence rows, what ordinary variation still has to discover after it, and what
   its goal-profile reading would be expected to show. No spec, no implementation.
3. **The two user rulings**, each with the numbers that bear on it: index alignment
   against not-forcing check 3 (E5), and the lab's role (E9, the Section 1 argument),
   with the T22 contract amendment drafted as the exact sentences to change.
4. **What was refuted** and should not be retried, with the row that refutes it.
5. **The next goal command**, drafted in the workflow's form for the first feature
   candidate, or a second diagnosis run if the verdict is mixed or undecided.
6. **Cost**: wall time per row and the run's turn count.

## Budget

Machine time about 15 h (E1 1 h, E2 3.5 h, E3 and E5 4 h or E7 3 h, E11 3 h, E8 2 h,
E9 1 h);
400 turns, closing from turn 360. Byte cap 40 GiB for the evidence tree.

## Goal command

Open a new session in the main checkout on a clean, current `main`: Opus 5.5, effort
`medium`, auto mode. Run `/advisor fable` first, so the Fable advisor is on before the
goal is pasted. Paste:

```
/goal An evolvability diagnosis run is complete on main. Read docs/strategy/evolvability-diagnosis-2026-10-05.md in full, then docs/strategy/evolvability-diagnosis-plan-2026-10-05.md, and follow the plan exactly; it is the run's contract, and the run 1 plan (docs/strategy/evolvability-exploration-plan-2026-10-01.md) and the T22 exploration contract govern anything it does not cover. You are Opus 5.5 and the implementer: you write the probes, run the rows, read the results and write the note. Confirm the Fable advisor is enabled and consult it through the advisor tool at every point the plan's rule 8 lists (before phase 0, before each phase's approach and the verdict branch, after each row's reading is drafted and before it is recorded, on any incomplete or contradicting row, before and after the plan forward is drafted, before closing), recording each consultation and its dispositions in the note's Advice table; if the advisor is unavailable, record that and send the brief to Codex instead. Confirm you are in the main checkout on a clean main, record the launch revision, create the worktree with EnterWorktree named evolvability-diagnosis, and never touch the run 6 worktree, branch or processes; before every heavy job check for run 6's processes as rule 2 says and wait if they are present. Send this plan to Codex for review (a) before any row runs and record each finding's disposition. Run phases 0 to 4 as predeclared rows without pausing to ask, except where rule 9 says to stop: predeclare and commit each row before it runs, keep incomplete rows, label every diagnostic arm, keep instr and proto commits default-off on the branch, and put experiment worlds under the note's folder, never under experiments/worlds. Name the verdict branch after phase 1 and run that branch's phase 2. Write the note's Plan forward section in the plan's six-part format, with roadmap feature candidates named by track and natural analog, the two user rulings stated with their numbers and the T22 amendment drafted as exact sentences, the refuted ideas listed, and the next goal command drafted; run Codex reviews (b), (c) and (d) and record dispositions. Close in the plan's order from turn 360 at the latest. Done means all of these are shown in this conversation, with command output and the note's ledger, Advice table and Plan forward quoted, not only file paths: docs/strategy/evolvability-diagnosis-run-2026-10.md is on main with at least E0, E1, E2, E4 and E6 completed (not incomplete), the verdict branch named, the Advice table holding a Fable consultation for every point rule 8 lists, and the six-part Plan forward present and Codex-reviewed; make check or make check-docs exited 0 on main; git worktree list no longer lists the evolvability-diagnosis worktree and git branch still lists its branch; git status on main is clean. If a concrete blocker stops the run, record it in the note, land the note as above, report it, and stop. Stop after 400 turns.
```
