# T20.F01 — Input-Use Baseline and Ecological Opportunity

**Status**: In Progress
**Last updated**: 2026-09-25
**Feature**: T20.F01
**Track**: [T20 — Input Evolvability and Structured Variation](../../roadmaps/t20-input-evolvability-and-structured-variation.md)

## Goal

Two readings, both observation only. First, every goal-profile closure
reports, per world and per T11.F26 cohort, how far each input family and
channel gets from declaration to connection, executed read, causal action
effect and one-step retention, on the original battery and on T11.F26's
nonzero-barrier, extended-perception and outcome contexts, so a reader can
see where input use stops. Second, a bounded, seeded competition assay in
Orchards, Canyon and Confluence at native costs records whether competent
authored use of three differently shaped families (a ring, a heterogeneous
vector and a scalar) changes actual offspring against a matched ablation,
with exposure and controller adequacy tested before any null is read, and
gives each family a positive, negative or inconclusive opportunity verdict
and a Graph feasibility record.

## Non-Goals

- No change to the mutation engine, operators, targeting, supply, runtime,
  sensors, costs, founder, world recipes or any default; no new ecology,
  environmental pressure or production mutation policy; production execution
  gains no branch or work for the ablation.
- No change to `neighborhood-v1`, `neighborhood-coverage-v1`,
  `mutation-effects-v1`, `steering-v1`, the structural sensor census or any
  existing report field, key or chart. No progress-page section.
- Not owned here: the neutral-consumer contract (F02), declare-and-connect
  recruitment (F04), structured variation (F05), discovery (F09), retention
  under native mutation (F10), ecological transfer (F11), VM input corrections
  S1/M2 (F03), learning (F06–F08, F13–F15). VM feasibility is recorded, not a
  gate.
- Authored controllers are instruments, never production genomes, mutation
  targets or fitness signals. A failed authored controller does not show that
  a family has no use.
- Historical or pre-repair results (the 2026-09-23 ring experiments, the
  T19.F06 and earlier steering and census readings) are cited, never pooled
  into this baseline.
- Readable state outside `InputReference` other than shared memory (VM
  indexed queue opcodes, graph operator state, plasticity) is not catalogued.

## Inputs and Invariants

Sources: the track's F01 row, its first two success criteria, the Notes
entries on general access, ownership, handoffs, the F01 deliverable row and
evidence gates; T11 Notes on F24/F25 and F26 coverage/closure; the
[sensor audit](../../strategy/post-t19-sensor-integration-audit-2026-09-23.md)
(inventory, S2, S3, S5) and the T11.F26 spec and closure reading.

**Existing seams, verified in code.** `mutation::sampling::random_input_reference_for_food_types`
draws 19 named buckets plus eight `UpstreamSlot` buckets (24 slots).
`runtime::inputs::resolve_input` is the one resolver for both backends: world
compounds wrap `sub_idx` modulo width; decision compounds read 0.0 at or past
width; `ActionQueue` reads past its mutation width 12 (audit S4). Graph reads
are `GraphSource::InputLeaf { ref_idx, sub_idx }`, VM reads
`ReadInput { dst, ref_idx, sub_idx }`; traced execution records graph passes
and VM steps with `pc` and instruction. T11.F26's `neighborhood::mutation_effects`
holds the founder, drift (20 depth-2,000 lineages) and selected (`min(20, s)`
T14.F12 genomes) cohorts, the single-event proposal seeds
(`PROPOSAL_SEED_BASE` 20,000,000) and the `recorded` and `authored` coverage
contexts. `simulation::seeding::seed_simulation` places founders on shuffled
cells and `seed_simulation_with_perception_mix` replaces seeded genomes after
seeding. `genome_size()` counts input references and edges. `v3-cli
recruitment` (T13.F07) is the capped-experiment precedent. The Orchards recipe has no
barriers and two food types (Grass 4, Fruit 15 energy); Canyon has barriers
and one food type; Confluence has barriers and both types.

**Research decision.** Extend T11.F26's cohorts and contexts and the
seeding seam; add no dependency. Alternatives: a standalone harness like the
2026-09-23 experiments (reproduces nothing at closure and cannot feed F02–F09)
and monoculture runs per arm (confound arm with population size and double the
runs). The assay follows the head-to-head competition design of
[Lenski et al. (1991)](https://doi.org/10.1086/285289): arms share one world
and fitness is read relative to a matched competitor, paired within a
replicate. Arm identity is fixed by setting the per-unit mutation rate to
zero, so the assay reads opportunity, not heritability, which F10 owns.

**1. Goal block `input-use-v1`.** New per-case
`cases[].input_use: Indicator<InputUse>`, `#[serde(default)]`, on
`goal-worlds-v1` only (`Undefined` elsewhere, gate included), projected whole
into the committed summary; timing in `environment.input_use_wall_clock_ms_per_seed`
and `_total`, outside every existing timer.

| Item | Definition |
| --- | --- |
| Catalog | the 19 named families (food families per configured type, plus any type a genome declares) and `UpstreamSlot` (channel = slot); shared memory (16 current, 16 previous slots) is a separate inventory with no declaration stage and no causal stage |
| Channel | the value the resolver actually addresses: scalar 0; world compound `sub_idx mod width`; `ActionQueue` the raw `sub_idx` (slot `sub_idx / 3`), with channels at or past the mutation width 12 labelled `beyond_draw_width` and kept through every stage; decision compound `sub_idx` below its width, and reads at or past it (constant 0.0) counted per family in `out_of_width_consumers`: consumers, not parents, on parent-reachable nodes, live or dead, summed over the cohort's parents |
| Parents | T11.F26's three cohorts, same identities and order |
| Scenes | `original`: the `neighborhood-v1` battery exactly as `Battery` runs it (48 fresh-state snapshots, 8 four-tick sequences carrying state under production bookkeeping); `extended`: T11.F26's `recorded` (up to 32) and `authored` (24) contexts from fresh state and its 4 × 32-tick `sequences`, run as T11.F26 defines them |
| `declared` | a parent-reachable node's `input_refs` holds the family (per food type) |
| `connected` | a structurally live consumer addresses the channel (graph edge on the census's live walk; VM `ReadInput` in the live instruction set) |
| `executed` | a read of the channel is actually resolved in at least one scene execution, counted independently of `connected` (Graph evaluates every compute node of a dispatched node, `runtime/cgp/execute.rs`); `executed_outside_live` counts parents whose only executed reads are on structurally dead consumers |
| `causal` | ablating the channel changes the committed action queue in at least one scene; `causal_original` counts the original scenes only; `causal_outside_live` counts causal parents with no structurally live consumer (the census's VM slice omits control dependencies such as `JumpIfZero`), a legitimate observation, not an error |
| `retention` | retained causal influence, not retained behavior: for a parent with at least one causal channel, its first 10 applied, not genome-identical T11.F26 proposals in proposal order, regenerated from T11.F26's seed formula; `retention_pairs` (parent-causal channel × child) and `retained_causal_pairs` (still causal in the child on the same scenes) |
| Ablation | one channel at a time, every read of it returns 0.0; nothing else is altered at its source, and execution proceeds under native semantics and charges, so downstream branches, steps, charges and live reads may change as consequences |
| Row | per cohort × family × channel: parents at each stage, `causal_original`, `executed_outside_live`, `causal_outside_live`, `retention_pairs`, `retained_causal_pairs`; rows with no parent at any stage are omitted; per cohort × family (per food type; `UpstreamSlot` and each shared-memory bank as one family): `family_declared`, `family_connected`, `family_executed`, `family_causal`, `family_causal_original`, each the number of cohort parents for which the stage holds on at least one of the family's channels (a parent counts once however many channels qualify; `family_declared` is the family's `declared` count; shared memory has none), plus `out_of_width_consumers`; bounds: each union is at least the family's largest channel row and at most `min(channel-row sum, family_declared)`, and `family_declared ≥ family_connected`, `family_declared ≥ family_executed`, `family_causal ≥ family_causal_original`, and `family_executed ≥ family_causal` apart from parents in `consistency_violations`; the readings' stopping stage is read on these unions; rows may use a compact encoding whose format string is stored in the block and round-trip tested; per cohort: parents evaluated and `consistency_violations` (causal without executed; expected 0, reported not asserted) |

Invariants per row: `declared ≥ connected`, `declared ≥ executed ≥ causal ≥
causal_original` (shared memory has no `declared`); `connected` and `causal`
are not nested. Observation runs after the last tick on
clones and consumes no simulation RNG; integer counts folded in a fixed order;
the block is byte-identical across thread counts. An extinct world yields an
`Undefined` selected cohort and recorded group with the reason; short samples
store requested and actual counts; zero denominators are `Undefined`.

**2. Opportunity assay `input-opportunity-v1`** (`v3-cli input-opportunity`).

| Parameter | Value |
| --- | --- |
| Worlds | the three `goal-worlds-v1` recipes, production config, recipe map; world index 0 Orchards, 1 Canyon, 2 Confluence |
| Arms (8) | `F` founder; `I` incumbents; `A_ring`, `Z_ring`; `A_vector`, `Z_vector`; `A_scalar`, `Z_scalar` |
| Assignment | the k-th seeded creature in creature order carries arm `k mod 8` |
| Replicates | 8 per world; run seed `26_000_000 + 1_000 × world_index + replicate` |
| Horizon | 1,000 ticks |
| Mutation | `per_unit_rate = 0.0`; every other setting (maintenance, ramped compute, replication, grazing, action costs) at recipe and production values |
| Incumbents | per world, one run of the goal profile's config for that world at its goal seed (11, 22, 33) to tick 1,000; up to 20 living genomes drawn uniformly without replacement from the id-sorted population with `SmallRng::seed_from_u64(27_000_000 + goal_seed)`; `I` creatures cycle through them in draw order; with no survivors they carry the founder genome, labelled `I (founder fallback)`, and every `I` comparison is `Undefined` |
| Per arm and replicate | founders, cumulative births (a newborn alive at the end of its birth tick, credited to the arm it inherits from its founding creature, which with mutation off is its parent's arm), living at every 100th tick, extinction tick; per replicate `births_total` and `unattributed_births` (newborns that die within their birth tick), with per-arm births plus `unattributed_births` equal to `births_total` |
| Exposure samples | at every 100th tick, up to 32 living creatures per `A`/`Z` arm drawn with `SmallRng::seed_from_u64(29_000_000 + 16 × (run_seed − 26_000_000) + tick / 100)`; snapshot from the production assemblers built unconditionally; `exposed` when any authored channel reads nonzero; `applied` when ablating the authored channels changes the committed actions of one execution from fresh cognition state |

Arm identity passes to every descendant through inheritance, never by
re-classifying genomes. The assay is byte-identical across thread counts and
reruns, consumes no RNG outside its seeds, and runs replicates in any order
that yields the same summary.

**Authored controllers.** Each is the canonical founder plus one declared
reference on its vote node and ordinary Graph structure (edges, and compute
nodes of existing kinds if gating needs them) feeding existing vote or
parameter sinks. Weights and structure are fixed constants, recorded in the
readings before the pilot, and never tuned on assay outcomes. `Z_k` is `A_k`
with every authored edge weight set to 0.0: same references, nodes,
`genome_size()` and perception assembly, no signal.

| Arm | Family and shape | Authored use | Adequacy fixture |
| --- | --- | --- | --- |
| `A_ring` | `NeighborBarrierRing`, cyclic 8 | inhibit `Move(d)` when `barrier[d]` for the founder's four cardinal moves | `steering-v1` (b): avoids in every founder-leading scenario; the founder avoids none |
| `A_vector` | `AreaFoodSummary(0)`, heterogeneous 7 | nearest-food dx/dy drive the cardinal `Move` votes when the four cardinal primary-ring cells the founder reads are empty and no primary food is here | empty cardinal ring with nearest food in each cardinal direction: leads with a move toward it in all four, the founder does not; food in any cardinal ring cell (also with barriers on the diagonals) or primary food here: the founder's action; an empty cell and a fruit-only cell, otherwise identical: the same action |
| `A_scalar` | `FoodHere(1)`, scalar | fruit here raises `Eat` and sets `EatFoodType` to 1 | fruit only and fruit with grass: `Eat(1)`; grass only: the founder's action |

Competence checks, for every `A_k` and `Z_k`: on every `neighborhood-v1`
execution and fixture context whose authored channels read zero, its actions
equal the founder's; `Z_k` equals the founder on every fixture context. On
signal-present fixture contexts crossing local food (none, grass, fruit),
primary-ring food (empty, present) and reproductive eligibility (eligible,
not), `A_k`'s committed queue differs from the founder's only by its intended
change: `A_ring` drops or replaces a move into a barrier; `A_vector` adds or
redirects a move only with an empty cardinal ring and no primary food here
(diagonal ring cells, which the founder does not read, do not gate it); `A_scalar` sets the
eaten type to fruit, or adds an `Eat` where fruit is the only food here. Every
`Reproduce` and every other `Eat` the founder commits is kept.

**Applicability.** `A_ring` in Orchards and `A_scalar` in Canyon have nothing
to sense; those pairs are predeclared null references, reported as
`not applicable` with their ratio, never as a verdict.

**Verdicts**, per family and applicable world, with `a`, `z` the `A_k`, `Z_k`
cumulative births in one replicate:

| Verdict | Rule |
| --- | --- |
| exposure gate | pooled over replicates and samples, `exposed ≥ 5%` and `applied ≥ 1%` of sampled `A_k` creatures; otherwise, or with no samples, `inconclusive (exposure)` |
| informative replicate | `a + z ≥ 20`; ratio `a / z`, `+∞` when `z = 0 < a` |
| positive | gate met, at least 7 of the 8 replicates both informative and `a / z > 1`, and `Σa / Σz ≥ 1.05` |
| negative | gate met, at least 7 of the 8 replicates both informative and `a / z < 1.05`, and `Σa / Σz < 1.05` |
| inconclusive | every other case, including fewer than 7 informative replicates; an uninformative replicate never counts toward a verdict and the sign-test n is 8 |
| uncertainty | per-replicate ratios, the pooled ratio and the exact one-sided sign-test p-value of each count (7 of 8 is 0.035) are stored with every verdict |

A family is positive when any applicable world is positive, negative when all
are negative, otherwise inconclusive; per-world verdicts stay beside it. The
F02–F05 gate is favorable when at least two families are positive, one of them
non-ring, each with a Graph-feasible controller; an unfavorable gate is
recorded and applied at closure under the master roadmap's standing rule for
unfavorable qualification verdicts. Per-replicate paired ratios `A_k / F`
and `A_k / I` (net reproductive change after native costs), `Z_k / F` (the
carried cost of inert structure) and `I / F` are reported beside the verdicts,
not as verdicts: `A_k / Z_k` isolates the signal within the scaffold.

**3. Discovery baseline and feasibility**, in the assay summary.

| Item | Definition |
| --- | --- |
| Declare step | 10,000 single-event proposals on the founder (units 1, rate 1.0, production operator mix and executed set, two configured food types), seed `28_000_000 + 10_000 × family_index + i`; `p_declare` is the share whose child declares the family on the vote node; declarations on other executed nodes reported beside it |
| Connect step | the same on the exact intermediate, the founder plus that one declaration on the vote node and nothing else, seed `28_000_000 + 1_000_000 + 10_000 × family_index + i`; `p_connect` is the share whose child gains a live consumer of an authored channel on a vote or parameter sink; any channel of the family reported beside it |
| Implied births | `(1 / p_declare + 1 / p_connect) / 0.485` (requested events per birth at 97 units): expected waiting time for the two steps in one unselected lineage, labelled an instrument figure; `Undefined` when either share is 0 |
| Graph feasibility | per family: authored edges and compute nodes, added `genome_size()` units and their per-tick maintenance charge, extended-perception assembly (host work, audit S5), work counters per execution against the founder on the fixture contexts; feasible when only ordinary Graph structure and existing sinks are used |
| VM concerns | the instruction count and step charges of a hand translation of each controller, counted, not run; S1 and M2 noted for F03 |

## Implementation Tasks

- [x] Core: the input-use catalog, stages, ablation and one-step retention
      over T11.F26's cohorts, contexts and proposals, in `neighborhood`.
- [x] Bench: `input_use` per case, timing fields, summary projection.
- [x] Core: authored controllers, ablation genomes, fixtures, arm seeding,
      birth attribution, exposure samples, verdicts, discovery baseline and
      feasibility record; CLI `input-opportunity` with `--pilot`, wall and
      byte caps, and a committed summary.
- [ ] Readings in `docs/progress/readings/t20-f01.md`: the controller
      constants, the funnel per world × cohort × family naming the first stage
      whose parent count falls below half the previous stage (or none), the
      opportunity and Graph-feasibility tables, the discovery baseline and the
      VM concerns. The funnel table awaits the family unions from the goal
      rerun; the rest is recorded.

## Verification

- [x] Focused tests (names and results in readings): each stage on
      constructed genomes (declared only; connected in an unexecuted node;
      executed without effect; an executed read on a structurally dead
      consumer; a VM read used only as a branch condition that changes the
      queue (`causal_outside_live`); causal on extended scenes only; causal on
      the original battery), channel wrap, `ActionQueue` channels past 12, decision
      out-of-width reads, ablation
      identity for an unread and an all-zero channel on both backends,
      retention on a constructed pair, the stage invariant, zero-denominator
      and short samples, byte identity across thread counts; the three
      adequacy fixtures (`A_vector`: food on any cardinal ring cell, also
      with diagonal barriers, keeps the founder's action; diagonal ring food
      opens the gate) and competence checks, equal `genome_size()` for each
      `A_k`/`Z_k` pair, arm assignment and birth attribution on a small world
      with the per-replicate `births_total` identity, the verdict rules on
      constructed tallies (an uninformative replicate never counts; sign-test
      `n` 8), the row-string round trip as a proptest, two reduced assay runs
      byte-identical, and the family unions (bounds; a multi-channel family
      whose union is below its channel sum).
- [ ] Existing blocks unchanged: gate `deterministic` equal to the T11.F26
      gate summary's; goal `deterministic` equal to the T11.F26 goal
      summary's after removing only `cases[].input_use`; method in readings.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` clean and
      `make check` exits 0 (2026-09-25, results in readings).
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      every survivor resolved.
- [ ] Gate and goal summaries at `docs/progress/features/t20-f01-input-use-baseline-and-ecological-opportunity.json`
      and `...-goal.json`, raw hash/byte count and verification time checked,
      series entries added, no full report staged; the second goal run is not
      required (workflow, 2026-09-05). Gate recorded; goal rerun pending for
      the family unions.
- [x] Assay pilot and full summaries at `...-opportunity-pilot.json` and
      `...-opportunity.json`, run through `scripts/bench-wait` after all
      code is final, alone on the host; the complete 3-thread run and the
      voided 8-thread attempt are in readings.

## Performance and Goal Impact

**Predeclaration — written before the run.** Observation only: no natural
analog is required, no environmental pressure is added and no simulation
behavior changes.

| Item | Predeclared |
| --- | --- |
| References | gate: previous closure T11.F26, gate epoch T11.F25; goal: previous closure T11.F26, goal-worlds epoch T11.F27 (`docs/progress/benchmark-series.json`) |
| Deterministic counters, persistence, existing indicators | no change in either profile; any movement is a defect |
| Gate | untouched; `input_use` absent |
| Goal observation cost | expected under 30 s summed over three worlds; cap 60 s in `environment.input_use_wall_clock_ms_total` |
| Existing caps | founder 10 s, evolved 180 s, read 10 s and `mutation_effects` 60 s unchanged; the 15-minute goal investigation threshold is judged on end-to-end time (T11.F26: 684 s) |
| Summary size | committed goal summary grows by at most 300 KB |
| Assay cost | 27 runs of 1,000 ticks (24 replicates, 3 incumbent sources), expected about 90 min from T11.F26's 630 s for 2,000 ticks in three worlds, most of it in the boom before tick 300; wall cap 7,200 s, summary cap 300 KB; the pilot (one replicate per world) projects the full run, and a projection over the cap is escalated to the spec owner, never met by cutting replicates, horizon or arms |
| Expected readings, sanity only | selected cohorts show causal use of the founder's own families (`FoodHere(0)`, `NeighborFoodRing(0)`, `EnergyCurrent`, `AgeTicks`) and sparse causal use of barrier, area, nearby and decision families; drift cohorts about half actionless; null-reference pairs near a 1.0 ratio; no direction is predeclared for any verdict |
| Epoch | no re-pin expected or authorized |

Exceeding a cap is resolved before closure, never absorbed. Commands, in
order, sequential and after all benchmark-affecting work:

```sh
make bench PROFILE=gate FEATURE=t20-f01-input-use-baseline-and-ecological-opportunity
make bench PROFILE=goal FEATURE=t20-f01-input-use-baseline-and-ecological-opportunity
scripts/bench-wait cargo run --release -p v3-cli -- input-opportunity --feature t20-f01-input-use-baseline-and-ecological-opportunity --pilot
scripts/bench-wait cargo run --release -p v3-cli -- input-opportunity --feature t20-f01-input-use-baseline-and-ecological-opportunity
```

**Measured verdict (2026-09-25).** Gate and goal: both not severe against
their epochs, all deterministic counters unchanged, all caps under budget,
goal summary grew ≈99.1 KB (cap 300 KB). The goal run took ≈518 s through
its final report output (benchmark log creation to last write,
11:45:16–11:53:54, filesystem-derived, not a process-exit time) against the
15-minute threshold; `wall_clock_ms_total` (465.8 s) is simulation time
only. Founder families show causal use in
all three worlds as expected. **Assay: complete, under cap.** The first
(8-thread) attempt overran the cap and was voided by the spec owner's ruling
(pilot/full concurrency mismatch; see Verification). The 3-thread rerun
completed: `wall_secs=1,692.015355` against the 7,200 s cap, `incomplete:
false`, 24/24 replicates, CLI exit 0. Family verdicts unchanged from the
void run: `ring` inconclusive, `vector` positive, `scalar` positive,
`gate_favorable: true`; unattributed births 0 in every world. Determinism
check: the void run's 23 raw records are identical to the rerun's
corresponding records. Full detail, projection method and per-world tables
in readings.

- Summaries: [gate](../../progress/features/t20-f01-input-use-baseline-and-ecological-opportunity.json),
  [goal](../../progress/features/t20-f01-input-use-baseline-and-ecological-opportunity-goal.json),
  [opportunity](../../progress/features/t20-f01-input-use-baseline-and-ecological-opportunity-opportunity.json).
- Full readings: [`docs/progress/readings/t20-f01.md`](../../progress/readings/t20-f01.md).

## Success Criteria

- [ ] Each world's goal report carries the input-use funnel by cohort, family
      and channel on original and extended scenes, with every existing block
      unchanged.
- [ ] The readings name where input use stops per world, cohort and family.
- [ ] The assay records, per family, a positive, negative or inconclusive
      opportunity verdict with exposure and adequacy, Graph feasibility, the
      two-step discovery baseline and VM concerns, and the F02–F05 gate.
- [ ] Required checks, mutation evidence, independent review and closure
      records are complete; the row is checked and the spec Complete on main.

## Notes for AI Agents

- Decision: run substitutions (user, 2026-09-25): no Fable model anywhere in this run, so the spec owner runs on Opus; every advisor consult is replaced by a fresh read-only Codex Astra `high` task through the Codex channel.
- Decision: the user authorized a second post-review remediation pass (2026-09-25) to store exact per-family unions in `input_use`; only the goal profile is rerun.
