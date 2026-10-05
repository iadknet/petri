# Evolvability diagnosis run: results (2026-10)

Date: 2026-10-05. Plan:
[evolvability-diagnosis-plan-2026-10-05.md](evolvability-diagnosis-plan-2026-10-05.md)
(the run's contract), executing the ledger of the
[evolvability diagnosis note](evolvability-diagnosis-2026-10-05.md) under the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md);
the [run 1 plan](evolvability-exploration-plan-2026-10-01.md) governs what the
plan does not cover. Status: **in progress**.

## Question

Which of three places blocks the evolution of sensor-driven behavior on the
current substrate: the world (the first step toward sensing does not pay), the
variation (the genome cannot build the wire in a reasonable number of births),
or the instrument (the premise was never true at production scale and horizon)?
And, given the answer, what should Petri change?

## Run setup

- **Roles.** Session on Fable 5.1 (`claude-fable-5-1`), the implementer; the
  Fable advisor is enabled and consulted through the `advisor` tool at every
  rule 8 point (the Advice table below). No subagent had been spawned when
  this note was first committed. Reviews: Codex `gpt-6.1-sol` `high`, read-only,
  through `codex exec` (`codex-cli 0.159.1`).
- **Run 6 close (rule 2).** Done first on `main`: no run 6 process existed
  (`pgrep` empty), the run 6 note is marked incomplete with outcome C (closed by
  user decision), its two completed main batches are read as row E10 below, its
  evidence tree (1.5 GiB) is copied to the main checkout's
  `.bench-artifacts/lab/exploration/run6/` with all 30 ALT draw file hashes,
  16 frozen genome hashes and the two batches' `summary.json` and
  `rows.ndjson` hashes verified (0 mismatches), six `docs:` commits
  cherry-picked onto `main` (`64127352` to `d00e684e`), `make check-docs`
  exit 0, worktree removed, branch `worktree-evolvability-exploration-6` kept.
- **Launch revision.** `main` `d00e684e` (clean). Worktree
  `.claude/worktrees/evolvability-diagnosis` on branch
  `worktree-evolvability-diagnosis`, created with `EnterWorktree` from the
  local `HEAD`. `npm ci` done; `cargo build --release -p v3-cli -p v3-lab`
  2 m 43 s. Disk free at launch 76 GiB.
- **Host.** No `caffeinate` was running when the goal was pasted; the session
  started a session-wide `caffeinate -i` and every heavy job is also wrapped in
  `caffeinate -i`. Telemetry off on every invocation. One heavy job at a time;
  `pgrep -fl 'v3-lab|v3-cli'` before each.
- **Evidence.** Raw output under `.bench-artifacts/lab/diagnosis/` in the
  worktree, copied to the main checkout at closing; compact summaries beside
  this note under
  [`evolvability-diagnosis-run-2026-10/`](evolvability-diagnosis-run-2026-10/).
  Experiment worlds, if any, under that folder's `worlds/`, never under
  `experiments/worlds/`.
- **Commit kinds.** `docs:` (this note and its folder), `lab:` (v3-lab probes),
  `instr:` (observation-only changes to production crates: new diagnostic arms,
  census probe), `proto:` (default-off mechanism prototypes). `instr:` and
  `proto:` stay on the branch.

## Ledger

Rows are the plan's, made concrete. Every row is predeclared here before it
runs; `incomplete` rows are kept. Arms labelled **diagnostic** seed authored
genomes and are never recommended as mechanisms.

| ID | Phase | Question | Method, instrument, sizes (as run) | Prediction / decision rule | Status | Result |
| --- | --- | --- | --- | --- | --- | --- |
| E0 | 0 | Baseline identity: does the launch revision's opportunity pilot reproduce after the E1 `instr:` commit? | `v3-cli --telemetry off input-opportunity --feature diagnosis-e0 --pilot --threads 3` at `d00e684e` (1 replicate per world, 1,000 ticks, the eight T20.F01 arms). Stored: the raw record's sha256 and the sha256 of the summary under [`e0-projection.jq`](evolvability-diagnosis-run-2026-10/e0-projection.jq) (`del(.wall_secs, .threads, .source_revision, .raw.path)`), in [`e0-baseline.json`](evolvability-diagnosis-run-2026-10/e0-baseline.json). After the E1 commit, the same command with the default arm set must give the same two hashes | Hashes equal. Unequal hashes block E1 until fixed | predeclared | |
| E1 | 1 | Does the first step pay? One authored edge from a sensor to a motor, at production scale | `instr:` arm set `one-edge` for the opportunity assay, selected by a new `--arm-set one-edge` option whose default (`t20-f01`) leaves the existing eight arms and their RNG draws untouched. Arms (**diagnostic**): F, I, and three one-edge controllers with zero-weight twins: (a) `AreaFoodSummary(0)` sub-value 3 (`nearest_dx`, the component `A_vector` steers on: `Move(E) += gx`, so +dx is food to the east) → `Move(E)` at +0.5, ungated; (b) the same edge at −0.5; (c) `NeighborBarrierRing[E]` (sub-value 2) → `Move(E)` at −0.5. Each controller appends the family's reference at index 7 of the vote node and one sink edge, exactly as T20.F01's `controller` does. Competence: the Z twin must match the founder's actions on the battery; A may differ from the founder only in `Move(E)` votes. Three goal worlds, 8 replicates, 1,000 ticks, 3 threads, native costs, mutation off. Read A/Z and A/F birth ratios under T20.F01's verdict rule (exposure gate; positive = 7 of 8 informative replicates above 1 and pooled ≥ 1.05; negative = 7 of 8 below 1.05 and pooled < 1.05) | (a) 1.0 to 1.1, (b) below 1.0, (c) about 1.0. **World branch** if no one-edge arm is positive in any world where `A_vector` was positive (Canyon, Confluence); **variation branch** if (a) or (c) is positive anywhere; otherwise mixed and E6 decides. Pooled ratios and per-replicate ratios are reported whatever the verdict, since the verdict rule needs a 5 % pooled effect to call positive | predeclared | |
| E2 | 1 | Is the premise true at production scale and horizon? Where does the funnel stop over time? | `instr:` ignored test `census_run` in `v3-cli` (release, `--ignored`): run `Simulation` on Canyon (seed 22) and Confluence (seed 33) at 1600² from the goal recipes to tick 50,000; at ticks 10,000, 20,000 and 50,000 call `neighborhood::input_use::observe` as the goal bench does (founder cohort plus the selected cohort of 20 parents at production sizes; **the drift cohort is passed empty**, since it is world-independent and already read in the T20.F01 goal summary) and write the family rows and funnel stops; inline samples every 500 ticks of population, mean genome size, generation and births (no second `v3-cli run` pass: the sampling is inline, recorded here); at 20,000 the E6 reading. 3 h awake kill per world; one world at a time | Vector and barrier families declared and connected at every checkpoint; causal use rare and not rising; genome size grows. **Premise false** if `family_causal` for `AreaFoodSummary` or `NeighborBarrierRing` rises across the three checkpoints and `retained_share` stays above 0.9 (the block is horizon). **Variation branch** if connected rises and causal does not. With one run per world and three checkpoints the reading is descriptive; "rises" means monotone across the three checkpoints in both worlds | predeclared | |
| E3 | 2 (variation) | Are intermediates neutral in the production economy? | Analytic from carry cost (10⁻⁴ per unit per tick), the E2 lifetime distribution and the E4 removal rates; then run 5's twin screen (cherry-pick `run5-screen` and the run 3 `instr:` commits) on the founder plus one declared reference, plus one zero-weight edge, plus one wrong-sign edge, 20,000 twins each, 2 h kill | Declared and zero-weight neutral within 10⁻³; wrong sign harmful 10 to 20 %. A measurable cost on the declared or zero-weight state becomes the first repair candidate | predeclared | |
| E4 | 1 | What are the real transition rates on the vote node? | `lab:` probe `transition_rates` (ignored v3-lab test, release): 10⁶ one-birth children each of (a) the founder, (b) founder plus a declared `AreaFoodSummary(0)` at index 7 of the vote node, (c) (b) plus one zero-weight edge `nearest_dx → Move(E)`, through `MutationEngine::apply_mutations_with_food_type_count` at `SimulationConfig::default().mutation` (production), `mesh_reachable_nodes` of the parent and the parent's battery dispatch record as `ParentExecuted` (the production targeting). Per child, by genome diff against the parent: declarations by family and node; prune of the index-7 entry; `AddGraphEdge`/`RetargetGraphEdge` landing on leaf 7 by sink kind, sub-value and weight sign; `RemoveGraphEdge` and `AlterGraphEdgeWeight` on the authored edge; applied counts by operator from the summary. Then the same on three run 4 reference elites (`native-*.json` from `.bench-artifacts/lab/exploration/run4/`, main checkout), food assay, seeds 5 to 7, replicate 0, with their own vote-node declaration | Within a factor 3 of the note's Section 2.1 (declare 3.2 × 10⁻⁴, prune of the unconnected entry 8.6 × 10⁻³, useful connect 4.8 × 10⁻⁷ per birth); prune-to-useful-connect ratio above 10³. Ratio under 10 refutes the cleanup diagnosis and drops the operator-weight candidate | predeclared | |
| E5 | 2 (variation) | Does a projection unit change the helpful rate per birth? | `proto:` operator `AddProjection` (field `projection_growth`, default off): one event declares a family if absent and wires every channel to every sink of one vote kind; variants `random`, `shared`, `aligned`. Twin screen on the founder and 15 run 4 elites, both repaired assays, 100,000 twins per stratum (shrunk to fit the 2 h kill if the pilot throughput requires, recorded), A/A check. The `aligned` variant pairs index with index and is admitted only as a measurement for the user's check-3 ruling, never recommended by this run | `random` no better than native; `aligned` several times more often confirmed-helpful and more often harmful. `aligned` not above native drops the projection candidate; `aligned` above native and `shared` not: index alignment is what matters, and the note states the check-3 question with these numbers | predeclared | |
| E6 | 1 (from E2) | Does the equilibrium world carry a usable gradient? | At tick 20,000 in each E2 world, for every living creature, from the production sensor assembly (`assemble_full_sensor_inputs` and `resolve_input`): `FoodHere(0)`, the four cardinal `NeighborFoodRing(0)` cells, `AreaFoodSummary(0)` `nearest_dx`, `nearest_dy`, `nearest_dist`; classify as inside food (food here or on a cardinal ring cell), gradient-informative (food visible, none adjacent), or blind (nothing visible); and the energy on the cell one step toward the nearest food against the mean over the four cardinal cells (a random step), from the world's food grid | Under 20 % gradient-informative. Over 40 % gradient-informative means the world is not the block and E7 is skipped | predeclared | |
| E7 | 2 (world) | Can a world make sensing the floor without naming it? | Experiment worlds under this note's `worlds/` at 800² with 2,500 founders: food in patches spaced beyond the founder's ring with regrowth returning on the patch's far side, move cost raised until a random walker's expected intake is under its decay, offspring placed within radius 3; founder alone 20,000 ticks (persistence), then founder plus 1 % `A_vector` (**diagnostic**; `instr:` recipe path for the opportunity machinery), then founder alone 50,000 ticks with the E2 census. 30 min per read, 3 h total; the world is iterated at most twice | Founder alone persists at low density or goes extinct; followers sweep; founder alone evolves causal food reads. Followers not sweeping means demand is still too weak at that geometry | predeclared | |
| E8 | 3 | Is the size feedback bounded in production, and by what? | `v3-cli --telemetry off run --ticks 20000 --sample-every 500` on Canyon, fresh seeds 1022 and 2022: default; `per_unit_rate` 0.02 (jq overlay into a recipe under `.bench-artifacts`); `proto:` arm where a `Reproduce` commit occupies `1 + genome_size / 97` ticks. Read mean genome size, births per creature-tick, persistence over ticks 15,000 to 20,000. The default arm runs on the pinned launch-revision binary | Default near 5 × founder by dilution; ×4 grows; time cost holds near founder. Default past 10 × founder by 20,000 ticks moves the size defect ahead of the encoding defect | predeclared | |
| E9 | 3 (if time) | Can the lab reproduce a known production positive? | **Diagnostic** calibration: an authored scene set with fruit only west of the start, comparator = founder plus one `NeighborFoodRing(1)[W] → Move(W)` edge; the lab at population 32 × 100 generations from the founder, 8 replicates, 1 h kill | Not reached. Not reached is written into the T22 amendment candidate | predeclared | |
| E10 | 0 | Run 6's two completed main batches, read as data: does ALT reach more often than N, and are ALT genomes larger? | [`e10-main-batches.json`](../strategy/evolvability-exploration-2026-10-run6/e10-main-batches.json) and [`e10read.py`](../strategy/evolvability-exploration-2026-10-run6/e10read.py) (sha256 `375c…`, bounds from run 6's frozen `stat6.py`): `food-s12` (1,000 generations) and `wall-s12` (500), 8 replicates each; per arm reached count, generation of reach, final best, validation mean, final elite genome size; ALT against N (`native-aa`) by replicate with run 4 rule 4's U; the ALT draws' parity from run 6's F1 | ALT reaches no more often than N; ALT genomes larger. A clear ALT excess on both assays (b − c ≥ 2 of 8 on each) reorders E5 behind a dense-afferent variant | **completed** (descriptive) | **Food-s12:** no arm reached (R 0/8, N 0/8, ALT 0/8); final best means R 2.11, N 2.31, ALT 2.14, founder 2.23, comparator 20.6; final elite genome size medians R 140 (115–282), N 139 (126–165), ALT 204 (139–476); shuffled-score 3,098 (1,201–32,721). **Wall-s12:** N reached 1/8 (replicate 2, generation 216, validation mean 5.68); R 0/8, ALT 0/8; final best means R 0.61, N 1.33, ALT 1.37, founder 0.47; genome size medians R 1,231 (297–8,489), N 336 (146–5,433), ALT 4,419 (290–6,970). **ALT vs N:** food b = 0, c = 0; wall b = 0, c = 1; U = 0.369 on both (8 pairs cannot reach the 0.25 bound). The excess rule is **not met**; prediction held. **Parity:** votes and actions identical in all 32 development scenes on both assays; ALT dies about 1 tick earlier (carry cost), food score equal in 28/32, wall in 31/32. ALT is unresolved at this size; the dense-afferent direction waits on E5 |
| E11 | 2 (both branches) | Does a changing world widen the useful neighborhood? | `instr:` `food-seeking-hunger` with two mirror-image frozen layouts A and B switched every k generations (pilot grid k in {3, 30, 300}, one k for the main batches); arms switching, random-layout, stationary V2; population 32, 300 generations, 8 replicates, seed batches 3 and 4; reading: run 5's twin screen on each arm's final elites scoring confirmed-helpful one-step children on the other layout; T22.F03 ablation sentinels per generation. 3 h | Switching elites have at least 3 × the stationary elites' alternate-layout helpful share. Positive: a lead for T02.F01's priority; negative at this size closes the fluctuation lever for this instrument class | predeclared | |

## Verdict branch

Named after phase 1 (E4, E1, E2 with E6).

## Findings

Filled as rows complete.

## Advice

Every Fable advisor consultation the plan's rule 8 lists, with the disposition
of each point raised.

| # | Rule 8 point | Advice (summary) | Dispositions |
| --- | --- | --- | --- |
| 1 | Before phase 0, after the plan was read | Approach sound; proceed. (1) Define the E0 projection before hashing: strip timing, threads, source revision and the raw path, commit the filter beside the note, reuse it unchanged post-E1. (2) Design E2 once: read the bench caller first; take the population/genome/generation samples inline rather than a second `v3-cli run`; put the E6 reading inside the 20k checkpoint; do not serialize a 60k-creature snapshot. (3) E1's new arms must not disturb the existing eight's RNG draws: put the arm set behind a flag defaulting to the current set; verify the sign convention before hardcoding sub-value 3. (4) E10: reuse the frozen `stat6.py` bounds, read validation means from the last-generation rows, record the reader's sha, label CT `ran, not analysed`. (5) Check which commit created the run 6 note before cherry-picking; use `-x`; verify hashes against the main copy. (6) Unlock first; kill only a broker whose cwd is the dead worktree; expect `.DS_Store`. (7) Commit this note with every row predeclared before E0 runs. (8) Protect the done set (E0, E1, E2, E4, E6, E10); cap phase 2 rows rather than let them eat the plan forward. (9) E4 needs genome diffing, not only operator counts; run 4 elites are under the main checkout. (10) Start a session-wide `caffeinate -i` now and record that none was running | All ten adopted. (1) `e0-projection.jq` committed with this note. (2) E2 row rewritten: inline samples, E6 inside the 20k checkpoint, drift cohort omitted. (3) E1 row: `--arm-set` option with the T20.F01 default; sign convention read from `controllers.rs` (`Move(E) += gx`, `gx = gate · nearest_dx`). (4) Done in `e10read.py`; CT labelled `ran, not analysed` in the run 6 note. (5) The note was born in `a87d90c5`; six commits picked with `-x`; hashes verified on the main copy: 0 mismatches. (6) No broker had the run 6 worktree as cwd; unlock reported "not locked" (the lock had already been released); removal clean. (7) This commit. (8) Adopted as the run's rule. (9) E4 row names the diff. (10) Started; recorded under Run setup |

## Codex reviews

| Review | Job | Finding | Severity | Disposition |
| --- | --- | --- | --- | --- |
| (a) plan, before any row | `.bench-artifacts/lab/diagnosis/codex/plan-review-a.out.md` | pending | | |

## Plan forward

Written in phase 4 in the plan's six-part format.
