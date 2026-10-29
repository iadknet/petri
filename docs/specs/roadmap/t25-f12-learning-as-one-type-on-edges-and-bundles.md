# T25.F12 — Learning as One Type on Edges and Bundles

**Status**: In Progress
**Last updated**: 2026-10-28
**Feature**: T25.F12
**Track**: [T25 — Edge Groups and Per-Edge Learning](../../roadmaps/t25-edge-groups-and-learning.md)

## Goal

Synapses and the developmental programs that build them learn by one
machinery, as one family of plasticity rules serves every synapse type (the
row's natural analog). After this feature one `Learning` value (rule, rate,
clamp, reward modulation, retention, `lamarckian`) is stored on a lone edge,
on a member as its override, and in a bundle's two slots: `scale_learning`
("learns as a group") and `member_default` ("learns per edge"), either, both
or neither present. Every member weighs `scale × pattern + modifier`; a
member's policy is `Inherit`, `Static` or `Own(Learning)`, and its `credits`
switch feeds the scale whatever its policy. A genome-owned learning layout, one
pure function of the finalized genome that the runtime builds once per birth
and never resolves during a visit, resolves every policy and lists every
learner; the
runtime, structural classification, the census and the inspector read it, and
one update law serves every learnable number with no scope branch. `Scope`,
`inherit_scale`, the bundle's `retention`, `SetScope` and the three bundle
retention operators are gone, paying debt G3.

## Non-Goals

- The birth transaction and where seeds ride (G4, T29.F01): `BirthSeeds`
  keeps one row entry per edge and one slot per bundle.
- Any value law: the four Hebbian rules, the three-factor step, `relax`, the
  enable draws, the rate, trace-decay and retention perturbations, I1's
  ranges (`rules.rs`, `hebbian.rs` laws) are unchanged.
- The plasticity update cost (0, user 2026-10-13).
- A counter of which `Learning` an edit landed on; the goal run reads state.
- Any goal-long run (T25's allowance is spent).

## Inputs and Invariants

| Input | Where | What F12 takes from it |
| --- | --- | --- |
| Track row, criterion and Notes | `docs/roadmaps/t25-edge-groups-and-learning.md` (F12 row; the "Learning is one type" criterion; Decided 2026-10-22 on F12; F12 acceptance guard) | The goal; the user's model; "supersedes member-owned scale-learning configuration, not only the names"; the acceptance guard; "viability first, gate + goal; slow tests run" |
| Design note | `docs/strategy/bundle-learning-ownership-2026-10-22.md` | The user's words, the objects, the who-learns table, layout and runtime split, update law, detach table, mutation draws, two-commit shape, acceptance cases; open items resolved in invariants 9–10 ([readings](../../progress/readings/t25-f12.md) records the options) |
| T25.F11 | its invariants 2–8, 10–11 | The bundle record, one member law and composition, the birth policy whose learning step invariant 10 replaces, the operator catalog, the `bundles-v1` block, the pin classes |
| T30.F10 | its invariants 1, 4, 5 | One catalog: an operator is added in its domain enum's tables and dispatch; `EventReport` is the one report |
| T25.F06–F09 | their specs | I1, the initialization rule (visited or seeded), birth values clamped at construction, decay toward the birth value, F07 invariant 9's counts |
| Code | `genome/mod.rs:136-215`, `genome/cgp/edge.rs`, `genome/cgp/bundle.rs:300-360`, `genome/cgp/container.rs:279-381,618-646,1237-1258,1318-1365,1474-1552`, `runtime/plasticity/learned.rs`, `runtime/cgp/execute.rs:30-260`, `mutation/graph/{hebbian,bundle,mod}.rs`, `mutation/event.rs:150-160`, `simulation/actions/{reproduction,cgp_reproduction}.rs`, `simulation/stats.rs:340-350`, `genome/companions.rs:57`, `genome/cgp_mesh_annotations.rs:33`, `neighborhood/plasticity_census.rs` | What changes; sites in the audit |
| Readers | `v3-server/src/transport/brain.rs:180-580`, `frontend/src/components/inspector/{PlasticEdgeList,EdgeBundleList,graphInternalsEdges,edgeBundleWeights}.tsx/.ts`, `v3-telemetry/src/metrics.rs:816-826,1070-1190`, `telemetry/grafana/dashboards/petri-run.json`, `scripts/telemetry-dashboards-check`, `v3-cli/src/bench/{tracking,schema,bundles,census_pins,artifacts}.rs`, `v3-lab/src/{eval,summary}.rs` | Who moves |
| Reference specs | `docs/reference/v3-{mutation,graph-backend,genome,reproduction,server-api-protocol,cli-contract,evolution-observability}-spec.md` | Every sentence naming scope, `inherit_scale`, bundle retention or the removed operators |

Vocabulary. A **learner** is one learnable number: a lone edge's weight, a
member's **modifier** (its `own_weight`), or a bundle's scale. A member's
**effective `Learning`** is its override for `Own`, `member_default` for
`Inherit` when present, none otherwise. A **contributor** is a member with
`credits` of a bundle with `scale_learning`. A **target** of a learning edit
is a lone edge, a member, or one of a bundle's two slots.

| # | Invariant |
| --- | --- |
| 1 | **One type.** `PlasticityConfig` becomes `Learning`, fields and `in_range` unchanged; it moves with `HebbianRule`, `RewardModulationConfig` and `retention_decays` into one `genome` learning module, which also holds `MemberPolicy` and the layout (invariant 4). The genome JSON key `plasticity` becomes `learning`. |
| 2 | **Storage.** By type, an edge is lone with `Option<Learning>` or a member with `Membership { bundle, policy: MemberPolicy, credits: bool }`, `MemberPolicy::{Inherit, Static, Own(Learning)}`: a lone edge cannot hold `Inherit` or `credits`, an `Own` member always holds its `Learning`. `GraphEdge` keeps its 40-byte assertion; a layout that cannot fit is a requirement question for the spec owner. `Bundle { source, wiring, scale, scale_learning: Option<Learning>, member_default: Option<Learning> }`; `inherit_scale` and `retention` are deleted with `Scope`. Composition (F11 invariant 3) and cost (one unit per bundle, F10) are unchanged; no learning setting changes size. Validation checks I1 on every stored `Learning` (lone, `Own`, both slots). Owner writers: lone learning, member policy, credits, each slot (`EdgeMut`, `BundleMut`); none changes a seed. A new member is born `Static`, `credits` false, before the birth draws. |
| 3 | **Who learns.** Per the note's table: an edge learner exists for every lone edge with `Learning` and every member with an effective `Learning`; a scale learner exists for every bundle with `scale_learning`, whether or not it has contributors. A contributor's activity feeds the scale whatever its own policy, so a member may learn its modifier and credit at once. |
| 4 | **Learning layout (genome).** One pure analysis of a Graph node (`def`, `input_refs`), in the genome learning module, yields in canonical order: every edge learner with its occurrence, effective `Learning`, source (`Lone`, `Own`, `MemberDefault`) and, for a member, its bundle and pattern; every scale learner in `BundleId` order with its `Learning`, its contributors (occurrence and pattern, canonical order, possibly empty) and its members (occurrence and pattern, the edges whose weight reads the learned scale). Readers given only a genome (mesh annotations, `structural_companions`) call the same function; sharing one built layout across readers waits for the birth constructor (G4, T29.F01). It is the only code that resolves `Inherit` or reads a policy or `credits` to decide what learns, and its emptiness is the one definition of "carries learning": runtime construction, `StructuralCompanions.has_plasticity`, mesh annotations' `has_stateful_behavior` (an edge learner into a live compute node or a sink, or any scale learner), the census carriers and the lab read it. `plastic_edges()` is deleted. A genome owner operation `clear_learning()` makes every lone edge static, every member `Static` and both slots absent; the lab's `expressed` calls it. |
| 5 | **Runtime construction.** `LearnedState::for_genome` builds each Graph node's layout once per birth (after the child's mutation, before its first visit) and stores per learner its effective `Learning`, value, birth value, trace and seeded flag, and the contributor and member indices; `runtime::plasticity` names no `MemberPolicy`, `credits` or slot. Birth values: an edge learner `clamp(seed if its Learning is lamarckian else genomic own, ±clamp)`; a scale learner the same with the bundle's seed, genomic scale and `scale_learning`. Initialization stays visited-or-seeded. A scale learner without contributors keeps its value, decays, is captured and is seeded like any other; it only receives no credit. |
| 6 | **One update law.** Per committed visit, from visit-start state, two steps per learner: aggregation, then one scalar update. An edge learner evaluates its `Learning`'s rule on the edge's `pre`, `post` and composed visit-start weight. A scale learner evaluates `scale_learning`'s rule on each contributor's `pre`, `post` and composed visit-start weight and sums the results times each contributor's pattern, unnormalized. Pure Hebbian: add the (summed) delta at the learner's rate, clamp by its clamp, write once. Reward-modulated: add the (summed) activity to the learner's one trace; the reward step is `three_factor` per initialized learner at its own source, rate and clamp. Each tick every trace decays by its learner's trace decay and every initialized learner with retention below 1 relaxes toward its birth value. `LearningCounts.updates` counts learner updates (an edge learner per commit, a scale learner per commit with a contributor, a reward step per initialized modulated learner) and `changes` those whose stored value moved; a scale counts once, not per contributor (a definition change, invariant 11). |
| 7 | **Seeds.** Capture stores each initialized edge learner's value at its occurrence and each initialized scale learner's at its bundle; a child consumes a seed only where its own layout has a learner there whose `Learning` is lamarckian. The container's seed alignment is unchanged. |
| 8 | **Detach.** When a member leaves its bundle (`DetachGraphEdge`, a source-changing edit, a split) its weight composes (F11 invariant 4) and its lone `Learning` follows the note's table: `Own` keeps the override; `Inherit` takes a copy of `member_default` when present, else none; `Static` none. `credits` is dropped; the seed composes as today; the last member removes the bundle. Relocation and removal keep policies and `credits`. |
| 9 | **Learning edits (mutation).** One draw picks the target uniformly among the node's eligible targets: edges in `edge_sites` order, then bundle slots in `BundleId` order, `scale_learning` before `member_default`. A picked member then draws the broadcast coin `gen_bool(0.5)`: true lands the edit on every member of its bundle the same predicate admits, in canonical order; lone edges and slots draw none and report `Broadcast::One`, so every committed learning edit is counted. The value is drawn once after the coin from the picked target's `Learning` (today's `Law`). `EventReport.attribute_scope` becomes `broadcast: Option<Broadcast>`, `Broadcast::{One, Members}` in `mutation::event`. The catalog follows the table below; Graph total weight 78 → 73. |
| 10 | **Birth draws.** The F11 birth policy's learning step, per new bundle in order: the form coin `gen_bool(0.5)` (true `Randomized`); `scale_learning` by F07's `draw_mode` (absent, Hebbian config, Hebbian then modulation, by `gen_range(0..3)`); `member_default` by `draw_mode`; then `Uniform` draws one policy and one `credits` coin `gen_bool(0.5)` once and applies both to every member, `Randomized` per member in canonical order a policy then a `credits` coin. A policy draw is `gen_bool(0.5)` true `Inherit`, else `draw_mode`: absent `Static`, present `Own`. Modifiers are 0 and the pattern is never drawn; the scale follows the birth law. |
| 11 | **Readers.** Census: carriers are creatures with a learner on a reachable node; edge learners keep their pathway × surface, rule, inheritance (effective `lamarckian`) and retention cells, and `edges_by_scope` (`ungrouped`, `edge`, `group`) becomes `edges_by_source` (`lone`, `own`, `member_default`); scale cells key on `scale_learning.lamarckian`, bound and retention are its clamp and retention, and every scale learner counts. View: each `LearnedEdge` and `LearnedScale` carries its learner's effective `Learning` once, with no copied retention, bound, kind or inheritance field (the kind is `PlasticityKind::of` it); `LearnedEdge` drops `scope`, gains `source`, and `learned` is `Some` whenever initialized. Telemetry and bench: the Telemetry table. `bundles` block `bundles-v2`: a separate per-wiring `learning` map with `scale_learning` and `member_default` (bundles with the slot), `members_by_policy` (`inherit`, `static`, `own`) and `crediting`, kept on each world's last checkpoint only (the `wirings` aggregates stay on every checkpoint). `docs/progress/benchmark-series.json` records `counter_definition_changes` for `plasticity_updates` and `plasticity_changes`, `plasticity_census` (source keys, carriers), `attribute_changes_by_scope` → `attribute_changes_by_broadcast` and `bundles` (`v1` → `v2`). Server: `BrainBundle` replaces `inherit_scale` and `retention` with `scale_learning` and `member_default` (one wire `Learning`); a member carries `policy` (`inherit`, `static`, `own`) and `credits`; `EdgeLearner` carries its effective wire `Learning` (flattened) and `source`, `lamarckian` replacing `scope` and `inherits`; the wire `Scope` and `LearnerScope` go; `PROTOCOL_VERSION` bumps and the generated frontend types follow; the inspector renders policy, credits and both slots from them and drops the group-scope sentences. Lab: `GENOME_FORMAT` 10 → 11. |
| 12 | **Two commits.** Commit 1 (seams, behavior-preserving): the learning module, the layout and the update-law owner over today's representation and execution: each contributor keeps its own configuration, trace, source and rate, a scale keeps the smallest-member clamp and exists only with a group-scope contributor, a group-scope member's correction stays unlearned; these semantics live only in the layout's builder; every pin, draw stream and gate deterministic block holds (a moved pin is a defect). Commit 2 (the model): invariants 1–11, commit 1's old semantics deleted with no switch; every Graph draw, every mutation-on pin, genome JSON and text digests, census pins, `bench_artifacts` digests, slow-tier readings, lab fixtures, fixture seeds re-chosen under the new stream with their precondition and assertions unchanged, `PROTOCOL_VERSION`, generated types, the reference specs and both epochs (gate, goal-worlds) re-pin with that one cause. The founder carries no learning and no bundle: founder-only viability digests do not move in either commit. |

| Operator | Weight | Targets (eligible) | Effect |
| --- | --- | --- | --- |
| `EnableHebbian` | 1 | lone edge without `Learning` that `can_learn`; member not `Own`; absent slot | `draw_hebbian_config`; a member becomes `Own` |
| `DisableHebbian` | 1 | lone edge with `Learning`; member not `Static`; present slot | lone and slot none; a member becomes `Static` |
| `ClearLearningOverride` (new, after `DisableHebbian`) | 1 | member not `Inherit` | `Inherit`; no value draw |
| `MutateHebbianRule`, `MutateHebbianRate`, `ToggleHebbianLamarckian` | 2, 4, 2 | stored `Learning` (lone, `Own`, present slot) | today's laws |
| `EnableRewardModulation`, `DisableRewardModulation`, `MutateRewardSource`, `MutateTraceDecay` | 1, 1, 2, 4 | stored `Learning` without / with modulation | today's laws |
| `EnableRetention`, `DisableRetention`, `MutateRetention` | 1, 1, 4 | stored `Learning` at / below retention 1 | today's laws (T25.F09) |
| `ToggleCredits` (new, in `SetScope`'s place) | 2 | member | the other of the picked member's `credits` on every target; no value draw |
| Removed | — | `SetScope`, `EnableBundleRetention`, `DisableBundleRetention`, `MutateBundleRetention` | their weights (2, 1, 1, 4) leave the catalog |

`ComplexityEffect`: `ClearLearningOverride` and `ToggleCredits` `Neutral`;
the rest unchanged.

## Architecture

Owners touched ([`docs/architecture.md`](../../architecture.md) §3): the
genome owns `Learning`, `MemberPolicy`, the bundle's slots, the learning
layout (its "deterministic analysis, the learning layout among it"), the
detach table and `clear_learning`; mutation owns the target draw, the
broadcast coin and the birth draws, and writes through the genome's owner
writers; the runtime owns learned values and the one update law; readers own
the census; outer crates own the wire, telemetry export, the bench blocks and
the lab. Pays G3, deleted at closure; no boundary changes, since §3 and §4
already state this split. Shared rule "what learns": owner the layout
(invariant 4); its interpreters today, all moved to it, are
`runtime/plasticity/learned.rs:166-320,569-574`, `genome/companions.rs:57`,
`genome/cgp_mesh_annotations.rs:33`, `neighborhood/plasticity_census.rs:244-262`,
`v3-lab/src/eval.rs:219-229` and `v3-server/src/transport/brain.rs:556-580`.
Shared rule "the update target": owner the runtime's update law; the scope
branches in `learned.rs` go. Shared rule "a detached member's learning":
owner `CgpGraphBackendDef::detach_at` (`container.rs:1318`), which every
detaching path reaches. Whether to detach before a source change or a split
is still decided by mutation (`detached`, `mutation/graph/operators.rs:366`,
before `EdgeMut::set_source` and `split_edge`); T30.F11's row owns making those
owner-complete, so F12 leaves it in place and marks it. The feature uses G4
as it stands: seeds ride in the container (`genome::cgp::container`),
captured before mutation, because T29.F01 owns where they live; the runtime
constructor reads them, and the layout never does.

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.tick.plastic_edges_by_source` (`petri.plasticity_source`: `lone`, `own`, `member_default`), replacing `plastic_edges_by_scope` | gauge | the census over the runtime view of applied learners | none beyond the run attributes |
| `petri.tick.plasticity_carriers`, `plastic_edges*`, `learned_scales*` | gauge | the census; carriers and scale cells redefined by invariant 11 | none beyond the run attributes |
| `petri.run.mutation_attribute_changes_by_broadcast` (`petri.broadcast`: `one`, `members`), replacing `…_by_scope` | counter | committed learning edits by `EventReport.broadcast` | the mutation config in `run.started` |
| `plasticity_updates`, `plasticity_changes` work counters | counter | `LearningCounts` (invariant 6) | none |

The dashboard panels and `scripts/telemetry-dashboards-check` follow the
renames; no new family, attribute set or cap.

## Verification Plan

**Question.** Edges and bundles learn by one `Learning` type through one
layout and one update law, so a bundle-only learner learns, `Inherit` members
learn with `member_default`, and every reader agrees with the layout; shown
wrong by a failing acceptance case, by a goal world where bundles carry
`scale_learning` yet no learned scale moves off its birth value, or by a
compute cost outside the predeclaration.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate + goal` | a production behavior change to learning and its mutation draws, read at 2,000 ticks; goal-long is spent on this track |
| Slow tests (`make rust-test-slow`) | `run` | the lab changes (`expressed`, `GENOME_FORMAT`) and the track's F12 note declares it |

## Implementation Tasks

- [x] Viability first (`cargo test -p v3-core --test viability`) before and after each commit.
- [x] Commit 1: the genome learning module and layout, `clear_learning`, the runtime's one update-law owner over today's semantics, every "carries learning" site reading the layout (audit R1, R2); preservation cases first: a randomized bundle whose contributors differ in rule, rate, source and trace; mixed reward sources on one scale; group-scope members with nonzero corrections. Pins hold.
- [ ] Commit 2: invariants 1–11: `Learning`, storage, slots, policies and `credits`; `Scope`, `inherit_scale`, bundle retention and their setters deleted (R4); `Broadcast` (R3); the catalog table; birth draws (R5); detach table; the update law without commit 1's semantics; readers, view, wire, inspector, telemetry, bench, lab (R6); reference specs; predeclared re-pins.
- [x] Acceptance cases written first (Verification).
- [x] Write `WORKAROUND(T30.F11): source changes and splits detach inside the container's owner operations` at `detached` (`mutation/graph/operators.rs:366`).

## Verification

- [ ] Track guard: each learning-edit operator applied on its lone-edge, member and both slot targets, checking the broadcast (one, members) and that unaffected settings, siblings and the other slot do not change; runtime construction, census and structural classification agree with the layout, including bundle-only learners and scales without contributors.
- [ ] Note's cases: a bundle-only learner learns; an `Inherit` member under a learning scale moves only through it; `member_default` serves `Inherit` and leaves `Own` and `Static`; the slots evolve independently; a member that learns and credits updates both from one visit; disabling one member leaves siblings and the scale's credit; a child whose last contributor was switched off keeps, decays and passes on the scale; each detach row; Lamarckian seeding of a scale and a modifier; the layout built once per birth, with `Inherit` never resolved during a visit (invariant 5: `runtime::plasticity` names no `MemberPolicy`, `credits` or slot).
- [ ] Proptests (v3-core): detach preserves the composed weight bitwise and yields the table's `Learning` for any policy and slots; the layout's effective `Learning` is the override when `Own`, `member_default` or none when `Inherit`, none when `Static`.
- [ ] `make check`; `make rust-test-slow`.
- [ ] Gate and goal summaries at `docs/progress/features/t25-f12.json` and `t25-f12-goal.json`, raw hash, byte count and time checked, series entries, no full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: one family of
plasticity rules for every synapse type (Goal); it reaches creatures through
the genome's learning settings, the Graph catalog and bundle birth. Commit 1
moves no trajectory. Commit 2 moves every mutation-on trajectory with one
cause.

Compute: no new per-tick pass; per visit a scale learner evaluates one rule
per contributor as today's group-scope members did, and the layout replaces
`from_seeds`' own scan at birth. More members may learn (`Inherit` under
`member_default`, about 2/3 of bundles carry it) and scales learn without a
plastic member, so cognition cost per visit may rise. Gate
`wall_clock_ms_per_creature_tick` is predeclared within the 25 % flag
threshold against the gate epoch, ±10 % as a hypothesis; a `flag` is reported
with its mechanism, a `severe` blocks closure until the user accepts it or
remediation removes it. `plasticity_updates` and `plasticity_changes` change
definition (invariant 6) and are read as definition changes, not compared
like-for-like; no direction is predeclared.

Goal hypotheses, descriptive, no threshold: in every world bundles carry
`scale_learning` and `member_default` near the 2/3 birth share, drifting with
selection; `learned_scales` counts at least the bundles with
`scale_learning` on reachable nodes, some with nonzero drift; `own` and
`member_default` edge learners both present. Persistence and every per-seed
counter move with the draw stream with no predeclared direction; a `severe`
there blocks closure until the user accepts it or remediation removes it.
The epochs re-pin at closure with commit 2's cause.

**Measured verdict.** Not yet measured.

- Summaries of the declared profiles: [gate](../../progress/features/t25-f12.json),
  [goal](../../progress/features/t25-f12-goal.json).
- Full readings: [`docs/progress/readings/t25-f12.md`](../../progress/readings/t25-f12.md).

## Success Criteria

- [ ] The track's "Learning is one type" criterion holds in the code: one `Learning` on edges and both bundle slots, `scale × pattern + modifier` for every member, policies and `credits`, a contributor-independent scale, one layout read by runtime, structure checks and the census, one update path.
- [ ] G3 is deleted from `docs/architecture.md`.
- [ ] The acceptance cases, `make check`, the slow tier, gate and goal pass the Verification section.

## Notes for AI Agents

- Decision: the model is the user's (2026-10-22): bake in as few assumptions as possible and allow evolution maximum flexibility within clean layers; both slots at once, the explicit `Static` policy and contributor-independent scale state are assistant proposals under that rule (track Notes).
- Deferred: mutation decides when a member detaches before a source change or a split (`mutation/graph/operators.rs:366`, audit R7); T30.F11 pays.
- Deferred: the long-run selection of learning policies, slots and `credits` is read by T27.F02's goal-long run, naming every feature closed in between.
