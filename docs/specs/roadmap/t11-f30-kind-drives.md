# T11.F30 — Kind Drives

**Status**: In Progress
**Last updated**: 2026-10-12
**Feature**: T11.F30
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

Motor gating apart from aiming, as a collicular gate and burst vigor start a
saccade whose direction the map sets, and as a bacterium's run keeps its
heading until something turns it. The vote catalog gains one drive sink per
directed kind (`Drive(Move)`, `Drive(Reproduce)`, `Drive(StealEnergy)`); a
kind's effective vote is its drive plus its best direction vote minus its bar,
its direction stays the kind's own argmax, and a tie within a kind goes to the
creature's heading: the direction of its last committed `Move`, at birth the
direction away from its parent, drawn at seeding for a founder. "Move more
when hungry" or "stop when full" is one edge into a drive instead of one per
direction. The outcome, read by the goal profile in three worlds, is drive
edges present and carrying weight at tick 2,000, their helpful and harmful
shares read beside matched direction-edge births, harmful births per birth not
up, steering not down, and persistence above its floor.

## Non-Goals

- No shared direction group across kinds; no drive for `Eat` (its one sink is
  already a drive); no change to the within-kind argmax beyond the tie, to the
  across-kind tie (incumbent, then lowest kind index), to the bar, to
  `Terminate`, `Decide` or the commit decoder.
- No founder rewiring: the founder's 32 edges and 4 `Multiply` nodes are kept;
  the 13-edge factored port is T18's. The founder gains only the three unwired
  sinks every Graph node carries and a heading.
- No heading sensor: the genome cannot read its heading. No turn or tumble
  mechanism, no configuration field or switch, no T22 lab run.
- No change to ring projections (T11.F31), `RefineHeritableStructure`'s ring
  grouping or any operator's code path: the drives are reached by the existing
  draws over the catalog, whose moduli move.
- No lineage instrument: fan-out duplicates and drive carriers are read
  cross-sectionally at tick 2,000.

## Inputs and Invariants

Sources of truth: the F30 row and the "Factored motor output, 2026-10-11"
Notes entry (the F30 tie sub-bullet; "the drives join the destination sets of
recruitment, born-connected wiring and refinement"; the founder's wiring left
as it is) in the T11 roadmap; the
[factored motor output note](../../strategy/factored-motor-output-research-2026-10-11.md)
§§4, 5, 6 (option B), 7 and 8 (decisions and readings); the
[T19.F06](t19-f06-retirement-and-observability.md) spec (one execution model;
production, observed and traced agree; inspector performs no float
arithmetic). Code: `creature/genome/vote.rs` (catalog), `runtime/vote_select.rs`
(commit rule), `runtime/mesh.rs` (pass loop), `runtime/action_decode.rs`,
`creature/genome/cgp.rs` (`new_with_fixed_outputs`, 94 sinks),
`mutation/graph/{operators,recruitment,refinement}.rs`,
`mutation/input_ref/mod.rs` (`connect_declared_input`), `mutation/vm/operators.rs`
(`AddVote` draw), `mutation/compound.rs`, `simulation/seeding.rs`,
`simulation/actions/reproduction.rs` (`spawn_child`, `deliver_pending_birth`),
`simulation/tick.rs` (`execute_move`), `neighborhood/{battery,steering}.rs`,
`neighborhood/mutation_effects/`. The previous closure is
`t11-f31-ring-projection-motif`; both epochs are `effort-cost-defaults`.
Planning numbers are in the [readings](../../progress/readings/t11-f30.md).
The option research is the note's §§5 and 6: option B is the user's choice
against A (keep), C (shared "where") and D (F31, closed); no external source
compares factored with per-direction outputs under mutation-only search, so B
rests on analogy, the founder's own factored shape and the §4 duplication cost.

| Question | Decision | Alternatives and why not |
| --- | --- | --- |
| Form | **Separate**: three catalog sinks `VoteSink::Drive(DirectedVoteKind)`, so the vote vector is 30 wide and every existing draw over the catalog (`InputRef.Add`, VM `AddVote`, recruitment destinations, the surface draw, decision-input sub-values) reaches a drive with no new code path; decision inputs read a drive apart; the VM reaches drives through `AddVote`, so T11.F18 parity holds without an opcode | Folded (a drive sink fanned into the eight sinks at accumulation) keeps the vector at 27 but needs a fan-out in Graph effects and a VM opcode or hidden index map, and hides the drive from decision inputs and traces |
| Index order | `Eat` 0, `Move(d)` `1 + d`, `Reproduce(d)` `9 + d`, `StealEnergy(d)` `17 + d`, `Drive(Move)` 25, `Drive(Reproduce)` 26, `Drive(StealEnergy)` 27, `Terminate` 28, `Decide` 29; `VOTE_SINK_COUNT` 30, `FIXED_SINK_COUNT` 97, `FIRST_ACTION_VOTE_SINK` 64 unchanged | Appending after `Decide` keeps two indices but breaks the kind-major order and the `COUNT − 2` / `COUNT − 1` pass-control positions |
| Effective vote | `E[K] = V[Drive(K)] + V[best[K]] − C[K]·unit` for the three directed kinds; `E[Eat] = V[Eat] − C[Eat]·unit`; with no drive written `0.0 + x` is `x` exactly, so a kind without a drive edge commits as today | Drive alone (direction only aims) loses T11.F21's one-edge property: a strong target both triggers and aims (note §7) |
| Within-kind tie | **First tied sink clockwise from the heading, the heading first**: `best[K]` is the argmax over `K`'s eight sinks visited in the order `(h + k) mod 8`, `k` in `0..8`, `h` the heading's `Direction::ALL` index, the first maximum kept (strict `>` replaces). With heading `N` one selection is today's lowest-index rule exactly, so every battery that executes a mesh outside a simulation tick passes `N` and reads as today unless a tick commits a `Move` and then meets a within-kind tie, when the later pass follows the new heading | Nearest-to-heading then clockwise differs only on tie sets with asymmetric gaps and costs an 8-entry rank table; keeping lowest index with the heading only when it is tied reintroduces the North bias for every tie set that excludes the heading, which the founder's cardinal-only channel meets on any diagonal heading |
| Heading storage | `CreatureState.heading: Direction`; the mesh reads it at tick start and every pass-end `Move(d)` commit sets it to `d` for the tick's later passes and for the stored state after the tick (a committed `Move` the world then blocks still turned the creature). Plumbing (an argument and an output field, or `&mut`) is the implementer's | Updating on an applied move in Phase 2 splits the rule across two layers and makes pass 2 of one tick blind to pass 1's commit |
| Heading at birth | The direction from the parent's cell to the cell the child is placed on: `PendingBirth.direction` when that cell is valid at delivery, else the direction of the fallback cell `deliver_pending_birth` finds; a span-0 birth uses the validated `Reproduce` direction | Inheriting the parent's heading is not "away from its parent" |
| Heading at seeding | One `gen_range(0..8)` draw per founder from the seeding RNG, uniform over `Direction::ALL`, taken in placement order immediately before each `CreatureState::new` in `seeding.rs`; positions are already shuffled before, so they are unchanged | A fixed `N` keeps every founder drifting north when no food is in view, the behavior the tie rule removes; `founder_index mod 8` avoids a draw but is a convention nobody asked for |
| Every other construction | `N`: `v3-lab` `eval.rs`, sensor fixtures, tests, and every battery (`battery.rs`, steering, T11.F14 executed set, `mutation_effects`, drift, recruitment paths) executing a genome outside a tick. A live creature's cloned state observed outside the tick (`observe_final_actions`, `observe_temporal_actions`, the `temporal_memory_sensitivity` indicator) reads the creature's stored heading: the observation asks what that creature would do now, and its three arms share it | A drawn heading there would move readings whose meaning is "the genome under a fixed scene" |
| Switch | None; attribution is the comparison with the previous closure, as at T11.F31 | An opt-in flag the track's disposal rule discourages |

| Boundary | Required behavior |
| --- | --- |
| Catalog | `VoteSink::Drive(DirectedVoteKind)`; `kind()` is the drive's `VoteKind`; `DirectedVoteKind::of_sink` is `None` for a drive (it names no direction); `from_index` and `all()` cover 30; `new_with_fixed_outputs` places the three drive sinks between `StealEnergy(7)` and `Terminate`. Serde `{"Drive":"Move"}`. `decode_commit` maps a drive to `NoOp` as it does `Terminate`; the commit rule never selects one as `best`. |
| Accumulation | A drive sink is an ordinary `ActionVote` sink: Graph effects write `sanitize(S)` of its edges to `contribution[index]`; VM `AddVote { sink }` with a drive index adds to it; the pass sums as today. No projection term reaches a drive (`of_sink` is `None`). |
| Commit rule | `vote_select::select(votes, bars, previous, heading)`: per directed kind the argmax runs over its eight direction sinks only (the drive is never a candidate) in the tie order above, then the effective vote above, then the across-kind rule unchanged. `Selection.best[K]` is the tie winner; the decoder reads its direction. The `Decide` guard and the pass end call the same function. Pure, RNG-free. |
| Heading | Updated only by a pass-end `Move` commit; `Reproduce`, `StealEnergy`, `Eat`, `NoOp`, a blocked move, death of a neighbor or a gestation change it never. Never inherited, never a sensor; copied by nothing but the creature's own state. |
| Mutation reach | `connect_declared_input` draws the sink over `0..30`; VM operator 38 draws `0..30`; `recruitment::destinations` lists the drives (a ring family landing on a drive appends its zero-weight edges as for `Eat`, never a projection); `pick_random_surface` runs over compute nodes plus 97 sinks; `refinement::correspondence` yields no ring group for a drive and `Slots` groups keyed by the drive's sink index as for `Eat`; `ActionVotes` and `PreviousPassVotes` sub-value draws run over 30. No operator gains a drive-specific branch. |
| Size and cost | A wired drive sink counts `1 + edges` in `genome_size()` as every wired sink does; the founder's drives are unwired, so `FOUNDER_GENOME_SIZE_UNITS` stays 97. No energy charge beyond the existing per-visit graph charge; the host cost is three more sink iterations per entered visit and the 30-wide vector copies. |
| Traces and protocol | `MeshPassTrace.votes`, `decision_inputs.action_votes` and `previous_pass_votes` are 30 wide (a traced hop carries at most 65 `f32`); the traced tick carries the heading it started with and the creature detail carries the current `heading`, both as a `Direction` string; `PROTOCOL_VERSION` moves to `v3alpha5` in `v3-server/src/types.rs` and `frontend/src/types/protocol.ts`. Production and observed modes are untouched beyond the width. |
| Inspector | `VOTE_SINK_COUNT` 30, `VoteSink` gains `{ Drive: VoteKind }`, `voteSinkFromIndex` and the node formatters name a drive (`Drive Move`); the pass detail shows per directed kind the recorded drive vote, the best direction-sink vote (the drive excluded from that maximum), the bar and the recorded effective vote, performing no float arithmetic (T19.F06 invariant 6); the tick's starting heading and the creature's current heading are shown. Layout is the implementer's. |
| Reference docs | `v3-mesh-execution-spec.md` §2 (pass end: drive term, tie rule, heading), `v3-graph-backend-spec.md` (30 vote sinks, 97 fixed), `v3-vm-isa-spec.md` (`AddVote` sink `0..30`), `v3-sensor-spec.md` §3.6 (30 sub-values), `v3-mutation-spec.md` (`0..30`, 97 sinks), `v3-server-api-protocol-spec.md` (widths, `heading`, version), `v3-startup-seeding-spec.md` (step 5 heading draw; §5.1 tie wording), `v3-reproduction-spec.md` (child heading at delivery). |

| Reader | A drive counts as |
| --- | --- |
| `steering-v1` `move_voted` | a node in the executed set with an edge into `Drive(Move)`, or a VM `AddVote` into it, votes `Move` |
| Functional complexity, sensor census, input-use funnel | a wired drive sink is a sink like `Eat`: its edges' sources are consumed references |
| `mutation_effects` genome identity | distinguishes drive edges (the sink list is part of the genome) |
| `ring_rules` census | unchanged: a drive is no direction path |

The `drives-v1` census is a goal-profile reading over every living creature at
the goal's last persistence sample (tick 2,000), reachable nodes only, both
backends:

| Term | Definition |
| --- | --- |
| Drive path | For kind `K`: a Graph edge with `\|w\| > 1e-3` into `Drive(K)` on a reachable Graph node, or a VM `AddVote` into `Drive(K)` in a reachable VM node's program. Sign is the edge weight's; a VM path is unsigned. |
| Fan-out | On one reachable Graph node and one kind `K`, a `(source, sign)` with edges of `\|w\| > 1e-3` into at least two of `K`'s eight direction sinks; its edges are fan-out edges. The founder has 4 fan-outs holding 16 edges (`Move`: bias, gate, queued-reproduce; `Reproduce`: gate). |
| Fields | `creatures`; per kind `drive_carriers`, `drive_edges`, `drive_edges_positive`, `drive_edges_negative`, `drive_vm_paths`; per kind `fanout_creatures`, `fanouts`, `fanout_edges`; `founder_fanouts` and `founder_fanout_edges` as computed by the same code on the canonical founder. |

The `drive_births` reading is the carrier-outcome instrument keyed by birth
edge class. `add_edge` (`AddGraphEdge`) and `connect_declared_input`
(`InputRef.Add`) report the class of the vote sink they push an edge onto at
the moment they push it, `drive` (a `Drive` sink) or `direction` (a `Move`,
`Reproduce` or `StealEnergy` direction sink); the engine carries it on the
birth's mutation summary, and `spawn_child` stores it on the child as
`birth_edge_class` only when `summary.applied_events == 1`, that one event
is `AddGraphEdge` or `InputRef.Add` (its `applied_by_operator` count exactly
1, since the operator list is the deduplicated keys), and the class comes from
that sole push. The two classes are therefore matched on parent population,
operator and the distribution of the source and weight draws, and differ only
in the drawn destination; no edge can be cancelled within such a birth. A
birth with more than one applied event (a repeated same-operator birth
included), with no vote-sink edge, or landing on `Eat`, `Terminate` or
`Decide` has no class; a test covers the repeated-operator case. At the carrier's removal
`SimStats` records its outcome into
`mutation_value_totals_by_birth_edge_class[class]` exactly as
`mutation_value_totals_by_operator` records it per operator
(`simulation.rs:139-145`): the same `MutationValueTotals`, the same helpful,
neutral and detrimental law. Raw report and summary keep-list carry both
classes.

**Determinism.** No new RNG draw in the mutation stream; seeding gains one
draw per founder from the RNG the simulation keeps (`seeding.rs:124`), so
every later turn shuffle and reproduction seed of a seeded run shifts. Every pin that covers a seeded run or a mutated
birth drawing a sink, a surface or a decision-input sub-value moves once and is
re-pinned with its reason in the readings: the founder digest (three sinks
join every Graph node), `founder_only_trajectory_digest_is_pinned` and the
`baseline_worlds` digests (heading draws and the tie rule), the default-rate
`applied_trajectory` digests (sampled and guard), `legacy_default_short_run_identity`,
drift-walk count pins, `recruitment_paths` accepted seeds, `experiment.rs`
lineage counts, mutation-engine sequence pins, catalog counts (`VOTE_SINK_COUNT`
27 → 30, `FIXED_SINK_COUNT` 94 → 97, `uniform27` → 30), tiny-fixture seeds
whose population no longer reaches a horizon (extinction ticks before and after
in the readings), the founder battery rows of `AddGraphEdge`, `InputRef.Add`, `InputRef.Swap`,
`InputRef.RawFieldMutation` and `RecruitNeutralInput` (draw remaps), and VM
fixtures whose `VmInstructionMutation` draws arm 38 (`AddVote`, sink
`0..30`); the founder has no VM node, so no founder row moves for it.
Unchanged: every config digest, `FOUNDER_GENOME_SIZE_UNITS` (97), the founder
battery rows of every other operator, and every battery reading of a genome
whose ticks meet no within-kind tie after a `Move` commit (one selection under
heading `N` is today's rule; the founder's `Move` effective vote is at most
0.9 and falls below its bar after one commit, so its gate readings are
identical). Traced, observed and production execution agree on every queue,
reason and heading.

**Natural analog.** The superior colliculus separates whether a saccade fires
(gate release, burst vigor) from where it lands (the map's locus), and a
bacterium's run holds its heading until a tumble changes it. A drive is the
gate, the direction channel the map, the heading the run; the founder's own
four-times-repeated gate (note §4) is the shape this encoding writes once.

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.mutation_value_by_birth_edge_class.<field>` | counter (`Sum`), one family per `MutationValueTotals` field, as `petri.run.mutation_value.<field>` is | `mutation_value_totals_by_birth_edge_class` at carrier removal | attribute `petri.birth_edge_class` from the fixed set `drive`, `direction`; a row in the expected families of `crates/v3-telemetry/src/metrics/tests.rs` and its names in `scripts/telemetry-dashboards-check` `keyed_names`; no config value governs it |
| `heading` | creature state on the creature detail (`v3-server`) and in the tick trace, not a metric | the creature's last committed `Move`, birth direction or seeded draw | none |
| configuration | none: no config field or mechanism switch is added, so nothing new reaches `run.started` or a phase group | — | — |

## Implementation Tasks

- [x] Catalog: `VoteSink::Drive`, index order, counts, `new_with_fixed_outputs`,
      frontend `VoteSink`, `VOTE_SINK_COUNT` and formatters; re-pin catalog counts.
- [x] Commit rule: heading parameter and tie order in `vote_select`, the drive
      term in the effective vote, the mesh's heading read and `Move`-commit update,
      heading on `MeshOutput` or by reference; traced and observed modes carry
      the 30-wide vectors.
- [x] Heading state: `CreatureState.heading`, the seeding draw, the birth
      direction in `spawn_child` for both delivery paths, `N` at every other
      construction and in every battery; `execute_move` unchanged.
- [x] Reach: confirm by test that `InputRef.Add`, VM `AddVote`, recruitment and
      `AddGraphEdge` reach a drive; `move_voted` counts a drive.
- [x] Readings: `drives-v1` census; the sink class reported by `add_edge` and
      `connect_declared_input`, carried on the mutation summary and stored as
      `birth_edge_class` for single-operator births;
      `mutation_value_totals_by_birth_edge_class` in `SimStats`, its metric
      family, the raw report and the summary keep-list; the inspector pass
      detail and headings.
- [x] Reference docs, protocol version, re-pins with reasons in the readings.

## Verification

- [ ] `cargo test -p v3-core --test viability` first (founder behavior changes),
      then `make check` -> exit 0; results in the readings.
- [x] Focused tests: the tie rule (heading `N` equals lowest index; a flat
      channel with a positive drive commits the heading; the first tied sink
      clockwise from a diagonal heading), the effective vote with and without a
      drive (an unwired-drive selection under `N` equals today's; a `Move`
      then a tie in one tick follows the new heading), the heading update on
      `Move` commit only, the birth
      heading on both delivery paths and the fallback cell, the seeding draw,
      mutation reach per operator, traced/observed/production agreement, the
      proptest commit-rule property extended to the drive term and the tie
      order; names and results in the readings.
- [x] Build and self-review, 2026-10-12: `cargo test -p v3-core --test
      viability` first (28 passed after the founder-only digest re-pin),
      `cargo test -p v3-core` (2,100 passed), `-p v3-telemetry` (73),
      `-p v3-server` (163), `-p v3-lab` (209), `-p v3-cli` (251 passed, 1
      failed: `gate_profile_has_no_severe_regression_against_series_references`,
      `decided_passes` 0.000313 against the stored 0.000000), `cargo check
      --workspace --all-targets`, `cargo fmt --all --check`, `make
      rust-clippy` (exit 0), frontend `tsc -b --noEmit` (exit 0), `vitest run`
      (383 passed), `biome check src` (no errors); `make check` not run. The
      `v3-server`, `v3-lab` and `v3-cli` counts are from the build commits;
      the self-review commit changes no code in those crates. Re-pins, the
      gate failure and the small-world survival reading are in the readings.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path, and
      every survivor resolved as killed, equivalent, or deferred. The full
      survivor list stays here.
- [ ] Benchmark summaries stored at `docs/progress/features/t11-f30-kind-drives.json`
      and `-goal.json`, local raw hash/byte count and verification time checked,
      series entries point to the summaries, and no new full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: motor gating
apart from aiming and a run that keeps its heading, reaching creatures through
the vote surface at commit and through genome mutation at birth. Expected
compute cost: none worth a budget (three more sink iterations per entered
visit, 30-wide vectors); no severe budgeted for compute. Supply: a drive is 3
of 30 born-connected and VM `AddVote` destinations and 3 of 97 `AddGraphEdge`
surfaces on a node with no compute nodes, so a kind's drive receives about one
eighth of the edges its eight direction sinks receive. Costs carried by
creatures: the ordinary sink and edge units. The catalog moduli move, so
every mutated birth's sink, surface and decision-input draws remap, the
seeding stream gains one draw per founder, and founders with no food in view
walk their heading instead of North, so `deterministic` differs in both
profiles from tick 1. References: `effort-cost-defaults` is the gate and goal
epoch; `t11-f31-ring-projection-motif` is the previous closure and the
reference of every row below. Thresholds: counters flag at +10%, severe at
+50%; wall flags at +25%, severe at +100%. Expected re-pins: the test pins of
the Determinism paragraph; an epoch only on a severe the user accepts. Every
indicator not in the table is reported with no direction. No environmental
pressure is added, so the three-world integration rule does not apply.

| Reading | Reference (Orchards 11 / Canyon 22 / Confluence 33) | Predeclaration |
| --- | --- | --- |
| Goal persistence | final 3,812 / 2,722 / 3,837; plateau 4,571.73 / 2,460.48 / 3,658.42; minimum 3,588 / 394 / 572 | **Floor:** no world extinct; final and plateau each at least half the reference on every world. A breach is a blocker for the user, never rebased. No direction expected. Minimum reported. |
| `drives`: carriers | none exists before this feature | **Fingerprint:** `drive_carriers` and `drive_edges` above 0 for `Move` on every world; a world missing it is a contrary result for the user. Other kinds, signs and VM paths reported, no direction (supply-limited). |
| `drives`: fan-out | founder 4 fan-outs, 16 edges; live world about 4 live gate edges per creature at generation about 345 (note §4) | Descriptive: per-kind `fanouts` and `fanout_edges` per creature reported beside the founder's, no direction; a drive can only replace a fan-out over generations the goal does not reach. |
| `drive_births` (raw `mutation_value_totals_by_birth_edge_class`) | none; `Graph.AddGraphEdge` and `InputRef.Add` carrier rows at the previous closure in the readings (detrimental / helpful 0.33 to 0.41 / 0.39 to 0.50) | **Diagnostic** (note §8): helpful and detrimental shares of the `drive` class reported beside the matched `direction` class with each class's `carriers_observed_total`; no direction. A class under 100 carriers on a world is reported as under-sampled with no share claimed; single-operator `InputRef.Add` and `AddGraphEdge` births at 3 of 30 and 3 of 97 destinations over about 200,000 births make that unlikely. |
| **Harmful per birth:** `neighborhood_read.dead_per_all_births` (5,000 births of the selected parents) | 0.0052 / 0.0072 / 0.0018 | **Not up:** at or below the reference plus two binomial standard errors of a 5,000-birth read, `2√(p(1−p)/5000)`: 0.0072 / 0.0096 / 0.0030. A world above goes to the user. |
| `neighborhood_read.changed_per_all_births` | 0.1432 / 0.1348 / 0.1594; standing floors 0.1464 / 0.1300 / 0.1434 | No reliable direction (a drive edge on a flat channel changes the queue, a direction edge may not). **Bound:** Canyon and Confluence not below their standing floors; Orchards, whose reference already sits below its floor by user acceptance at T11.F31, not below 0.1333 (the reference minus two binomial standard errors of a 5,000-birth read). A breach goes to the user. |
| Founder per-operator battery (raw) | previous closure's rows | Rows of the five remapped Graph and `InputRef` operators move (Determinism paragraph) and are reported; every other row identical. |
| Drift walk `changed_per_all_births`, depths 0 / 22 / 250 / 1,000 / 2,000 (2,000 births each) | Orchards and Confluence 0.1570 / 0.1155 / 0.0450 / 0.0195 / 0.0165; Canyon 0.1495 / 0.1065 / 0.0360 / 0.0140 / 0.0140 | No reliable direction: the walk reads the mutation map, which this feature remaps. **Predeclared regression bound:** not below the reference minus `2√(p(1−p)/2000)`: Orchards and Confluence 0.1407 / 0.1012 / 0.0357 / 0.0133 / 0.0108; Canyon 0.1336 / 0.0927 / 0.0277 / 0.0087 / 0.0087. A reading below goes to the user with the walk's dead and executed rows. |
| `steering_pooled` (raw) exact / within-45 / avoidance | exact 0.4407 / 0.4395 / 0.4205 over 472 / 537 / 459 moves; avoidance 0 on every world (user-accepted at T11.F31) | **Target** (note §8, "not down"): exact-hit not below the reference minus two binomial standard errors of the reference's move count, 0.3950 / 0.3967 / 0.3744, each above the standing 0.37; avoidance reported against the standing target above 0, expected 0 as at T11.F31 (the scene is one-hot, so a drive adds no aim); a world missing either is a contrary result for the user, not absorbed. Within-45 reported. Gate founder `steering-v1` and `mesh_execution` identical to the reference (heading `N`, drives unwired). |
| `ring_rules` fingerprint | carriers 1,106 / 996 / 1,370; nonzero 242 / 351 / 275 | Reported, no direction; F31's fingerprint (each field above 0) still holds on every world. |
| Goal `mean_genome_size`, `mean_mesh_nodes` at tick 2,000 | previous closure's samples (readings) | No reliable direction. |
| Gate (75 ticks, founders), per creature-tick | `actions_applied` 1.628292, `births` 0.027658, `decided_passes` 0, `graph_relax_iters` 5.464431, `mesh_hops` 5.539253, `pass_cap_hits` 0.001468, `passes` 2.683399, `plasticity_updates` 0.016550, `vm_steps` 0.042359 | `deterministic` differs from tick 1 (heading draws and the founder's walk). `actions_applied`, `births`, `graph_relax_iters`, `mesh_hops`, `passes` within flag. `vm_steps`, `pass_cap_hits`, `plasticity_updates` and `decided_passes` are small-base counters a remap or a changed walk can push past flag or severe (`decided_passes` reads severe against zero if any pass decides); such a severe goes to the user with per-seed values as remap movement, a gate-epoch re-pin requested. A severe on another counter does not fit. |
| Goal work counters, per creature-tick | previous closure | No compute severe budgeted. A severe on `vm_steps`, `plasticity_updates`, `decided_passes` or `actions_applied` is read against the per-world evidence and, if clade composition (as at T03.F12, T19.F05, T20.F16), reported for the user's decision on acceptance and re-pin, not attributed to compute. |
| Wall per creature-tick, observation budgets | previous closure; workflow caps | Wall flat within flag; a flag reported with host and phase. Budgets as at the previous closure; the census adds one pass over the terminal population, and the birth class is recorded where the edge is pushed at constant cost, no genome scan. |

Run sequentially after benchmark-affecting work is final, without competing
builds, tests, servers or measurements:

```sh
make bench PROFILE=gate FEATURE=t11-f30-kind-drives
make bench PROFILE=goal FEATURE=t11-f30-kind-drives
```

**Measured verdict (2026-10-12, HEAD `63e03f03`).** Values in the readings.

- Gate: CLI exit 3, make exit 2, `severe` on `decided_passes` (0.000313 against
  0; seeds 11 / 22 / 33: 1 / 44 / 0).
- Goal: CLI exit 3, make exit 2, `severe` on `decided_passes` (0.029780, +221%
  epoch, +172% F31).
- Misses: Canyon final 1,249 and plateau 667.696; Confluence
  `dead_per_all_births` 0.0058; drift depth 250 on every world. No world
  extinct; other readings pass. Summaries 21,216 and 395,892 bytes. Severes and
  misses go to the user, not rebased.

- Summaries: [gate](../../progress/features/t11-f30-kind-drives.json),
  [goal](../../progress/features/t11-f30-kind-drives-goal.json).
- Full readings: [`docs/progress/readings/t11-f30.md`](../../progress/readings/t11-f30.md).

## Success Criteria

- [ ] A directed kind commits on its drive plus its best direction vote; one
      selection of a genome with unwired drives under heading `N` is today's,
      and its tick queue is identical unless a `Move` commit precedes a
      within-kind tie in that tick; a within-kind tie goes to the first tied
      sink clockwise from the heading; the heading follows the last committed `Move`, the birth
      direction and the seeding draw, and nothing else.
- [ ] Every existing draw over the catalog reaches a drive; every reader in the
      Reader table sees drives; production, observed and traced execution agree.
- [ ] Gate and goal ran once each and every row of the predeclaration is read;
      every floor, bound, fingerprint and target is met, or the user has
      explicitly accepted its breach; an unresolved breach stops closure.
- [ ] `make check` exits 0 and the mutation gate is recorded.

## Notes for AI Agents

- Decision: the drives are separate catalog sinks (30-wide vector), the tie
  rule is the first tied sink clockwise from the heading, the heading updates
  on a pass-end `Move` commit, a founder's heading is one seeding-RNG draw and
  every out-of-tick mesh execution uses `N`; taken by the spec owner on
  2026-10-11 under the unattended-run instruction, each grounded in the row,
  the note §7 and the code (Inputs and Invariants, decision table).
- Deferred: review P2, `drift.rs:648` refresh test (readings).
- Exception: closure blocked pending the user's ruling (readings).
