# T25.F07 — Group-Scope Learning

**Status**: In Progress
**Last updated**: 2026-10-19
**Feature**: T25.F07
**Track**: [T25 — Edge Groups and Per-Edge Learning](../../roadmaps/t25-edge-groups-and-learning.md)

## Goal

Synapse-type-specific plasticity: a synapse built by a group program learns
for the whole group or for itself. Every member of an edge group carries a
`scope`, `Edge` or `Group`; a plastic member at scope `Edge` learns its own
correction as any edge does, and a plastic member at scope `Group`
contributes its pattern-weighted change to the group's one shared scale, so
every member's applied weight moves in proportion to its pattern. A group
record carries `inherit_scale`, which decides whether a parent's learned
scale seeds the child. The nine plasticity operators admit members, and each
one, like the new `SetScope`, lands on the picked member or on every member
its predicate admits, by one coin; group creation draws one learning mode,
one scope and one inheritance flag for the whole group. The laws, the clamps,
the per-edge traces and the Lamarckian seed rule are unchanged; a static
member contributes nothing.

## Non-Goals

- Randomized creation, `DetachGraphEdge` and the catalog bijection
  (T25.F08); the `GroupKey::Slots` refinement disposition (F08).
- A new rule, outcome channel, learning parameter, or any change to the
  update laws, their clamps or their draw ranges.
- A genome unit or an update cost for scope, `inherit_scale` or a learned
  scale (learning is free, T25.F02; user decision 2026-10-13).
- A new dashboard: the existing plasticity panels gain by-scope and
  learned-scale panels on the same board.
- Reading a stored genome whose membership lacks `scope` or whose group
  lacks `inherit_scale`: it is refused at the load boundary
  (`GENOME_FORMAT` 5), as the lab refuses earlier formats.

## Inputs and Invariants

| Input | Where | What F07 takes from it |
| --- | --- | --- |
| Track row, success criterion 7, Notes | `docs/roadmaps/t25-edge-groups-and-learning.md` | The goal; "scope and `inherit_scale` are heritable; a group-scope update is one pattern-weighted change to the shared scale, bounded once; every attribute mutation draws its scope"; the 2026-10-13 decisions (shared credit is one pattern-weighted change; no unit, no cost; group creation randomizes learning attributes, never the pattern); gate + goal, viability first, mutation gate and slow tests run. |
| Design note | `docs/strategy/edge-groups-and-learning-design-2026-10-13.md` §3 (`Membership`, `Scope`, `inherit_scale`, learned state, clamp rule, detach, I11), §4 "Learning on edges" (D4 scope and credit, seeds), §4 operators (scope coin, `SetScope`, uniform learned creation), §7 (`runtime::plasticity` group items, vocabulary, `graph_weights`), §8 F07 contract (readings: first-update snaps of genomic scales) | The contract below; the track row is authoritative where the two differ. |
| Dependency outputs | T25.F03 (`Membership`, `Group`, `compose`, `member_pattern`, `detach_at`, I10), T25.F04 (`DirectionPair` patterns), T25.F06 (`GraphEdge::plasticity`, `plastic_edges()`, `can_learn`, `LearnedState::{for_genome, capture, applied_weight, edges}`, `Learner`, `BirthSeeds::Rows`, invariants 1–14, `GENOME_FORMAT` 4, `v3alpha9`) | What scope, the scale slots, the group seeds and the payload extend. |
| Today's code | `creature/genome/cgp/{edge,group,container}.rs`, `runtime/plasticity/{learned,rules}.rs`, `runtime/cgp/{execute.rs:48,101-115,258}, effects.rs:49}`, `mutation/graph/{hebbian.rs:136-355,group.rs,mod.rs:62-131}`, `mutation/types/mod.rs:489`, `simulation/tick.rs:1278-1281`, `simulation/actions/cgp_reproduction.rs`, `creature/plasticity_census.rs` | The structures and laws this feature extends; the readers table below. |
| Readers | `v3-telemetry/src/metrics.rs:1048-1097` and `metrics/tests.rs`, `scripts/telemetry-dashboards-check:136-137,208-213`, `telemetry/grafana/dashboards/petri-run.json:2612-2760`, `v3-cli` bench summary (`plasticity_census` block, `SimStats` totals), `v3-lab/src/summary.rs:23`, `v3-server/src/http/creature.rs:79-175`, `mutation_effects/records.rs`, `tests/applied_trajectory.rs`, the frontend files in the readers table | Each reader's post-change expectation. |
| Pins | `tests/viability.rs` founder-only digests (must not move), the rate-0 founder fixtures, every mutation-on pin (`tests/baseline_worlds.rs`, `tests/applied_trajectory.rs`, `mutation/engine/recruitment_tests.rs`, `mutation/graph/tests/group.rs:112`, v3-cli trajectory and deterministic fixtures), `docs/progress/benchmark-series.json` (gate and goal-worlds epochs at F06) | The predeclared re-pins (invariant 11). |
| Primary sources | Larsen & Sjöström 2015 (abstract verified at source 2026-10-19: plasticity "is specific to the synapse type"); tied-weight credit (Rumelhart, Hinton & Williams 1986; summary) | The natural analog: a group is one synapse type whose members learn as a type or as individuals; a shared parameter's change is the sum of its instances' changes, each weighted by its instance's pattern. |

Vocabulary: a **learner** is a plastic edge; a member is an **edge learner**
at scope `Edge` and a **group learner** at scope `Group`; a **scale slot** is
a group's learned scale, which exists iff the group has a group learner; a
group's **bound** is the smallest `weight_clamp` among its group learners; a
**passive member** is a static member of a group with a scale slot; a
**dynamic edge** is a learner or a passive member. A learner's **own
quantity** is the learned weight of an ungrouped learner or the learned own
correction of an edge learner; a group learner has none.

Invariants (the container asserts 1 and 8 at its exits and `validate_graph`
at the load boundary; the proptests in the Verification table assert 6):

| # | Invariant |
| --- | --- |
| 1 | **Placement.** `Membership { group: GroupId, scope: Scope }`, `Scope { Edge, Group }`, JSON key `scope` (`"Edge"` or `"Group"`, always written, `deny_unknown_fields`); `Group` gains `inherit_scale: bool`, JSON key `inherit_scale`, always written; `GraphEdge` stays 36 bytes (const-asserted). `GraphEdge::is_member_of` and every membership comparison in the container compare `membership.group` only, so scope never affects enumeration, removal, relocation or detach. The container builds every member at scope `Edge` and every record with `inherit_scale` false; `EdgeMut::set_scope(Scope)` (a member only; keeps the seed) and `GroupMut::set_inherit_scale(bool)` (keeps every seed) are the only writers; `set_scale` clears the group's seed. `can_learn` keeps one exclusion, a `ClearSlot` edge; `validate_graph`'s `IneligiblePlasticEdge` covers that case only. Scope and `inherit_scale` carry no genome unit and no complexity; `genome_size` and the founder (80, no group) are unchanged. |
| 2 | **Layout.** `for_genome` builds once per backend, with no lazy allocation (the sample scratch keeps F06's grow-at-first-visit behavior, outside the state's identity): the edge slots, every plastic edge in canonical order (F06), each with its genomic own weight, its pattern (0 when ungrouped), its scope and, for a member, its scale slot or genomic scale; the scale slots, one per group with a group learner in `GroupId` order, each with the genomic scale, the bound, the learned scale and a seeded flag; and the passive set, in canonical order, weighed through the learned scale but carrying no samples, no stored value and no view item. A learned own quantity is stored for an ungrouped learner and an edge learner only; a group learner stores none (its own weight is genomic; its `lamarckian` flag is inert while its scope is `Group`). A graph without a plastic edge allocates nothing and runs today's loops. The executor hands the learner each edge's occurrence and value; the learner, not the executor, decides whether the edge is a slot, a passive member or static. |
| 3 | **Applied weight.** `applied = own + scale × pattern`: `own` is the learner's own quantity when it has one, else the genomic own weight; `scale` is the learned scale when the group has a scale slot, else the genomic scale; `pattern` is 0 for an ungrouped edge. `applied_weight(occurrence)` resolves one edge; `LearnedState::applied_weights(node_idx)` yields `(occurrence, applied)` over every dynamic edge in canonical order; the executor's weighing, the view's `applied`, both resolvers and the server payload agree (pinned by a test). The applied weight is never clamped. |
| 4 | **Credit (D4).** After effects, in canonical slot order with visit-start values, each pure-Hebbian learner's `Δ` comes from its own rule and rate and its samples with `w` = its visit-start applied weight; an ungrouped or edge learner applies `Δ` to its own quantity, bounded by its clamp, in that order; a group learner adds `pattern × Δ` to its scale's accumulator. A reward-modulated learner changes no weight at a visit: it accrues its rule's activity (with the same `w`) to its own trace and contributes only in Phase 2.5, where each initialized modulated ungrouped or edge learner applies the three-factor update (F06) and each modulated group learner whose scale slot is initialized adds `pattern × η × signal[source] × trace` to the accumulator. After each event (a committed visit; a backend's Phase 2.5 pass) every scale with a contribution, a zero one included, is written once as `clamp(scale + Σ, ±bound)`, scale slots in order; a write whose stored value lay outside the bound (only the first write of an unseeded scale can) counts one `plasticity_scale_snaps`. A static member contributes nothing; a `Ring` member (pattern 1) contributes `Δ` unchanged; a member with a negative pattern contributes `pattern × Δ` with the opposite sign of `Δ`, so its own applied weight still moves with its rule before bounding, while the net change is the sum over contributors. Traces decay once per tick (F06). A failed visit changes nothing. |
| 5 | **Initialized.** An edge slot: F06 invariant 4. A scale slot: `visited ∨ scale seeded` (visited is set at visit entry, before the pass-cost check, so a failed first visit initializes too); it gates Phase 2.5 contributions to that scale, `capture()` of the scale and the view's `learned` for it. A seeded scale with no initialized edge slot, and a seeded edge slot with an uninitialized scale, are each exposed, updated and captured on their own. |
| 6 | **Seeds.** `BirthSeeds::Rows { compute, sinks, groups }`, `groups` one `Option<f32>` per group record (I11). The container keeps `groups` aligned as it keeps the rows: `add_group` and `add_pair_group` push `None`, dropping a record (`remove_group`, a last member's `remove_edge` or `detach_at`) removes its slot, `relocate_group`, `set_scope`, `set_inherit_scale` and `set_plasticity` keep it, `set_scale` clears it. `capture()` returns `None` when no edge slot and no scale slot is initialized, else rows with the own quantity per initialized ungrouped or edge learner (`None` for a group learner and every static edge) and the learned scale per initialized scale slot. `from_seeds` reads the child's final attributes: a Lamarckian ungrouped or edge learner with a seed starts at the clamped seed and counts as seeded; a group learner ignores any seed on its row; a scale slot starts at `clamp(seed, ±bound)` and counts as seeded iff the child group has `inherit_scale` and a seed, else at the genomic scale, unseeded. A seed a child does not read (a scope change after capture, a group without a learner) is dropped at construction. `detach_at(occurrence, pattern)` composes the seed when the edge or the group has one, `seed := compose(edge seed or own, group seed or scale, pattern)`, reading no flag and clamping nothing (`from_seeds` clamps at construction); with neither seed the edge keeps none. Reproduction writes no runtime field. |
| 7 | **Operators.** The nine plasticity operators keep their keys, weights, predicates and value laws; a member now matches. After the pick, a member draws the scope coin `gen_bool(0.5)`: false, this edge; true, every member of its group that the operator's predicate admits, the rest untouched with no draw; a non-member draws no coin. Value draws follow the coin and are drawn once; each admitted member applies the law to its own config: `EnableHebbian` draws one config and writes it to every admitted (static) member; `EnableRewardModulation` draws one `RewardModulationConfig` and sets `modulation` on every admitted member, keeping each member's rule, rate, clamp and flag; `DisableHebbian` writes `None`; `DisableRewardModulation` clears `modulation` only; `MutateHebbianRule` and `MutateRewardSource` draw among the others of the picked edge's value and write the drawn value; `MutateHebbianRate` and `MutateTraceDecay` draw one `u = gen_range(-1.0f32..=1.0)` and each admitted member applies its own branch (`× (1 + 0.3u)` when the value exceeds 0.01, else `+ 0.05u`, then the `[0, 1]` clamp); `ToggleHebbianLamarckian` writes `!picked.lamarckian`. `SetScope` (key `Graph.SetScope`, weight 2, `Neutral`, after `RemoveGroup` in `ALL`: `ALL` 29 → 30, Graph default-arm weight 58 → 60, `MutationType::all` 62 → 63): picks one member uniformly in canonical order, draws the coin, writes the other of the picked member's scope to the chosen set; no value draw; `applies_to` is "any member". `AddRingGroup` and `AddDirectionPairGroup` keep their shape draws, then draw one mode `gen_range(0..3u8)` (0 static, 1 Hebbian, 2 reward-modulated); a learning mode draws one config by `EnableHebbian`'s law (then `EnableRewardModulation`'s for 2) written to every member; then one scope coin (`gen_bool(0.5)`: true `Group`) written to every member; then one `inherit_scale` coin (`gen_bool(0.5)`). Their weights and complexity effects are unchanged. |
| 8 | **Structure.** Scope travels with the edge through `relocate_group`; a retarget, split or raw-field step on a member detaches it first (F03) and the detached edge keeps its plasticity and has no scope; `RemoveGroup` and a last-member removal take the members' plasticity with them. A member of one group is never a member of another (I5 unchanged). |
| 9 | **Counters.** `plasticity_updates` counts one per contributing learner per event (an ungrouped or edge learner as F06; a group learner one per contribution); `plasticity_changes` counts one per contributing learner whose stored quantity differs after the event (its own quantity; for a group learner its scale, so a moved scale counts every contributor); `plasticity_scale_snaps` as invariant 4. Recorded as one `counter_definition_changes` entry. |
| 10 | **View and census.** The view's edge item carries `occurrence, genomic` (the genomic own weight: an edge learner's correction, not its composed weight), `learned: Option<f32>` (the own quantity when initialized; always `None` for a group learner), `applied`, `trace`, `bound`, `kind`, `scope: Option<Scope>`; `LearnedState::scales(node_idx)` yields `(group, genomic, learned: Option<f32>, bound, inherit_scale)` per scale slot in order. `PlasticityCensus`: every learner counts in `edges`, by rule, by inheritance and in `edges_by_scope` (`ungrouped`, `edge`, `group`); edge `drift_sum` is `|learned − genomic|` over initialized own quantities and `at_clamp` is `|learned| ≥ bound` over them, so a group learner adds neither and an unchanged correction under a nonzero genomic scale drifts 0; scale cells by inheritance (`darwinian` when `inherit_scale` is false, `lamarckian` when true): `scales`, `drift_sum` (`|learned − genomic|` over initialized scale slots, once per group) and `at_bound` (`|learned| ≥ bound`). |
| 11 | **Identity and determinism.** Commit 1 (text only): `scope` and `inherit_scale` exist with their defaults, so the `Debug` text and JSON of every grouped genome move, `GENOME_FORMAT` is 5 and `PROTOCOL_VERSION` `v3alpha10`; no draw and no behavior moves, and with `mutation.per_unit_rate` 0 every founder-based fixture (no group, no plastic edge) is byte-identical. Commit 2 moves the mechanism and the catalog, so every Graph draw changes (the catalog weights, the coin, the one-`u` perturbation, whose rounded single-edge outputs differ from today's) and every mutation-on pin re-pins; the founder-only viability digests do not move. |
| 12 | **Transport.** `graph_weights` lists every member and every plastic edge with `applied` on every listed edge and `learned` for an initialized own quantity; `scales` gains `learned?` for an initialized scale slot; built by one merge-walk over `edge_sites()`, `applied_weights(node_idx)` and the view's edge and scale items, all in canonical order, never by a per-edge search. |

Readers, with their post-change expectations:

| Reader | After F07 |
| --- | --- |
| `companions.has_plasticity`, `cgp_mesh_annotations`, recruitment-paths `stateful`, lab `expressed()` | the same `plastic_edges()` definition; members now count; lab refuses format 4 |
| `plasticity_census` | invariant 10 |
| v3-telemetry | the families in Telemetry; `metrics/tests.rs` expected families; `scripts/telemetry-dashboards-check` `always_names`; `petri-run.json` adds three panels beside `:2612-2760`: plastic edges by scope, mean learned-scale drift by inheritance, learned scales at their bound by inheritance |
| bench summary | the `plasticity_census` block adds the by-scope and scale rows; `plasticity_scale_snaps` joins the `SimStats` totals the gate and goal compare; the closure readings are taken from them |
| mutation-effects state records | the scale items appended after the edge items; `VERSION` `mutation-effects-v4` |
| `tests/applied_trajectory.rs` | hashes the scale items after the edge items (no founder-based change: no plastic edge) |
| server | invariant 12; `v3alpha10` with a §2 history line |
| frontend | `types/genome.ts` (`Membership.scope`, `EdgeGroup.inherit_scale`), `types/creature-detail.ts` (`GroupScaleWeight.learned?`), `EdgeGroupList.tsx` (group row: genomic scale, learned scale, `inherit_scale`; member row: scope, applied), the plasticity badge shows a member's scope, `graphFullModel.ts` draws every listed edge at `applied` and, when the payload is excluded, draws a member at its genomic composition and marks its applied weight unavailable (a regression for each case), fixtures and tests |
| reference specs | `v3-graph-backend-spec.md` §2 (scope, `inherit_scale`), §13 (scope, credit, scale slots, snaps); `v3-genome-spec.md` §3, §5; `v3-reproduction-spec.md` §4 (group and detach seeds); `v3-mutation-spec.md` operator table (`SetScope`, the coin, learned creation, default-arm 60); `v3-server-api-protocol-spec.md` §2, §4.11 |

Options weighed, the sources and the preparatory refactor audit:
[the readings file](../../progress/readings/t25-f07.md).

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.tick.plastic_edges_by_scope` | gauge by `petri.plasticity_scope` (`ungrouped`, `edge`, `group`) | the census's `edges_by_scope` over learners of reachable nodes | none; the scope is heritable, not configured |
| `petri.tick.learned_scales` | gauge by `petri.plasticity_inheritance` | the census's scale slots | none |
| `petri.tick.learned_scale_drift` | gauge (`f64` sum) by `petri.plasticity_inheritance` | `Σ |learned − genomic|` over initialized scale slots | none |
| `petri.tick.learned_scales_at_bound` | gauge by `petri.plasticity_inheritance` | initialized scale slots at their bound | none |
| `petri.run.plasticity_scale_snaps` | counter | scale writes whose stored value lay outside the bound (invariant 4) | none |
| `petri.run.plasticity_updates`, `petri.run.plasticity_changes` | existing counters | invariant 9's definition | unchanged |
| `run.started` body | record | unchanged: the feature adds no config value or switch | — |

Each family is in `always_names` and the expected-families test; the
`creature.genome` record carries `scope` and `inherit_scale` with the genome.

## Verification Plan

**Question.** F07 claims that a plastic member learns for itself or, through
one pattern-weighted bounded change to its group's scale, for its group, and
that a plasticity mutation lands on one member or on all of them at no cost
on a non-learning graph; a gate wall-clock flag, a viability failure, or a
goal world where members never learn would show it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate + goal` | the track plan names F07 "gate + goal, viability first": a production behavior change (phase 0 and 2.5 routing, the operator catalog) whose readings — plastic edges by scope, learned scales, drift, snaps — a 2,000-tick run takes; goal-long is rationed to F02 and F04. |
| Mutation gate | `run` | new production logic in `v3-core` (credit routing, seeds, the coin, `SetScope`, learned creation) that tests must pin. |
| Slow tests (`make rust-test-slow`) | `run` | the operator catalog and every Graph draw change, which moves the recruitment-paths recorded readings the tier pins. |

`cargo test -p v3-core --test viability` runs first: the tick loop's phase 0
and 2.5 routing changes.

## Implementation Tasks

- [x] Commit 1 (text only): `Scope`, `Membership.scope`, `Group.inherit_scale` with serde; `is_member_of` and the container's membership comparisons on `group` only (R5); `GENOME_FORMAT` 5, `PROTOCOL_VERSION` `v3alpha10`; `types/genome.ts`; the identity pins of grouped genomes re-pinned with "the hash without genome text is unchanged" as the check.
- [x] Container: `set_scope`, `set_inherit_scale`, `can_learn` and `validate_graph` (invariant 1); `BirthSeeds::Rows.groups` with the alignment rules and `detach_at`'s seed composition (invariant 6); the alignment proptest and `debug_assert_seed_shape` extended to group slots.
- [x] Refactor R1–R2 (readings file): the learner decides slot membership from each edge's occurrence; one composition used by `weigh`, the view, `applied_weight` and `applied_weights` (invariants 2–3).
- [x] Runtime: scale slots, the passive set, the credit accumulator in `commit` and `reward`, the snap counter, scale initialization, `capture` and `from_seeds` by scope, `scales()`, `applied_weights()` and the view fields (invariants 2–6, 10).
- [x] Refactor R3: each plasticity law split into one draw and a per-config apply; the two perturbations take one `u`; then the scope coin helper, members admitted, `SetScope`, learned creation, the catalog pins (`ALL` 30, default arm 60, `all()` 63) and the weight-sum test (invariant 7).
- [x] Counters (invariant 9) with the `counter_definition_changes` entry; census (R6), the four telemetry families, `always_names`, expected families, the three panels, the bench summary rows and totals (invariant 10).
- [x] Refactor R4 and the payload (invariant 12); `v3alpha10` §2 history; `creature-detail.ts`, `EdgeGroupList.tsx`, the badge, `graphFullModel.ts`, fixtures.
- [x] `mutation-effects-v4`, `applied_trajectory` hash, lab format 5; reference specs per the readers table.
- [ ] Re-pin every mutation-on pin under commit 2 (invariant 11), one cause per commit; viability first.

## Verification

Tests written first (names bind the implementer; design is the implementer's):

| Test | Asserts |
| --- | --- |
| `oja_group_members_use_visit_start_applied_weight` | Oja's `w` is the composed visit-start weight, for an edge learner and a group learner |
| `group_scale_credit_is_pattern_weighted_summed_in_order_and_bounded_once` | the scale delta equals the bounded `Σ pattern × Δ` from the recorded samples; own weights unchanged; a negative-pattern member's contribution is `pattern × Δ` |
| `a_modulated_learner_changes_no_weight_at_a_visit` | under a zero reward, committed visits move traces but neither own quantities nor scales, for both scopes |
| `a_static_member_contributes_nothing` | a static member of a learning group neither samples nor moves the scale, and is weighed through the learned scale |
| `a_mixed_group_updates_edge_learners_and_the_scale_separately` | edge learners move their own corrections, group learners move the scale, in one visit |
| `reward_credit_to_a_scale_is_one_bounded_write_per_tick` | Phase 2.5 sums the modulated group learners' contributions and writes once; gated on the scale slot's initialization |
| `a_first_scale_write_snaps_a_genomic_scale_outside_the_bound` | an unseeded scale at 7 with bound 2 becomes 2 at its first event, a zero-`Σ` event included, and counts one snap; a later write counts none |
| `a_scale_seeds_only_with_a_group_scope_learner` | `inherit_scale` without a group learner seeds nothing; a group learner without `inherit_scale` starts at the genomic scale |
| `a_group_learner_stores_no_own_quantity` | a Lamarckian group learner with a seed on its row starts at its genomic own weight, is captured as `None`, and after `SetScope` to `Edge` in a later child starts at its genomic correction |
| `initialization_is_per_slot_across_edges_and_scales` | a seeded scale with no initialized edge slot and a seeded edge slot with an uninitialized scale; a failed first visit initializes the backend; what each view item exposes and `capture()` returns |
| `a_detached_member_composes_its_seed_with_the_learned_scale` | the composed seed from edge and group seeds, pattern applied, no clamp in the container; a last-member detach drops the group slot |
| `seed_rows_follow_their_edges_through_every_primitive` (extended) | group slots follow their records through add, remove, relocate, detach and `set_scale`; a group with both scopes, distinct edge seeds and a group seed survives a relocation and its inverse unchanged |
| `applied_weight_agrees_with_the_executor_and_the_view` | invariant 3 across an ungrouped learner, an edge learner, a group learner and a passive member, through `applied_weight`, `applied_weights` and the view |
| scope-coin tails | a non-member draws no coin; a member the predicate rejects is untouched and consumes no draw; `EnableHebbian` on a group with a modulated member; `EnableRewardModulation` on a heterogeneous group keeps each member's rule, rate, clamp and flag; `DisableRewardModulation` keeps the config; `Toggle` writes the picked value; a rate step and a decay step apply one `u` per member's branch on both sides of 0.01 |
| `set_scope_flips_one_member_or_every_member` | the coin's two tails; no value draw |
| learned creation | one mode, one config, one scope, one inherit coin per group, both shapes; the draw order pinned |
| counters | invariant 9 on a group of two contributors whose scale moves |
| census | a group learner adds no edge drift; an unchanged correction under a nonzero genomic scale drifts 0; scale drift once per group |
| `validate_graph` | a plastic member passes; a plastic `ClearSlot` edge is still rejected |

The scope-coin tails, `SetScope`, learned creation, counters and census rows
are `a_non_member_draws_no_coin`,
`enable_hebbian_lands_one_config_on_the_admitted_members`,
`group_wide_modulation_changes_keep_each_members_config`,
`group_wide_toggle_and_perturbations_follow_the_picked_value_and_each_branch`,
`group_wide_rule_and_source_redraws_write_the_drawn_value`,
`set_scope_flips_one_member_or_every_member`,
`add_*_draws_*_then_learning_and_is_born_silent`,
`counters_count_every_contributor_of_a_moved_scale`,
`group_scope_learners_and_learned_scales_are_censused_by_scope_and_scale` and
`validate_graph_admits_a_plastic_member_and_rejects_a_plastic_clear_slot_edge`.

Verification run (build, 2026-10-19; `make check`, benchmarks, mutation and
the slow tier not run):

| Command | Result |
| --- | --- |
| `cargo test -p v3-core --test viability` (first, both commits) | 30 passed; founder-only digests unchanged |
| `cargo test --workspace --no-fail-fast` | all pass except `gate_profile_has_no_severe_regression_against_series_references`: `plasticity_updates` 0.104622 per creature-tick against the F06 epoch's 0.012305 (+750%), the predeclared counter-definition change (series entry), awaiting the gate and goal re-pin |
| `PROPTEST_CASES=2000 cargo test -p v3-core --lib seed_rows_follow` | pass |
| `cargo clippy --workspace --all-targets`; `cargo fmt --all -- --check` | clean |
| frontend `tsc -b --noEmit`; `vitest run`; `biome check src/` | clean; 103 files, 666 tests pass |

Identity checks: commit 1 leaves every hash without genome text and every
rate-0 fixture unchanged (`recruitment_tests`, `baseline_worlds`,
`applied_trajectory` comments); commit 2 leaves the mutation-off short-run
identity and the rate-0 sampled trajectory byte-identical.

- [ ] Focused tests: `cargo test -p v3-core --test viability` first, then `make check` -> result recorded here.
- [ ] Mutation gate: fresh `MUTANTS_ITERATE=0 make rust-mutants`, its summary line, output path, and every survivor resolved as killed, equivalent, or deferred.
- [ ] Slow tests: `make rust-test-slow` -> result and any re-pinned recruitment-paths readings.
- [ ] Gate and goal: summaries at `docs/progress/features/t25-f07-group-scope-learning.json` and `-goal.json`, local raw hash and byte count, series entries, no new full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: plasticity set
by synapse type (Larsen & Sjöström 2015), a group being one type whose
members learn as a type or as individuals; the shared scale changes by the
sum of its members' pattern-weighted changes, tied-weight credit. It reaches
creatures through mutation only: the founder has no group and no plastic
edge, so every mutation-off reading is identical.

Compute: a non-learning graph runs today's loops. On a learning graph the
learner's work per weighed edge becomes one occurrence comparison and one
multiply-add, plus per event one accumulator per scale slot and one bounded
write per contributed scale; the layout adds the passive set and the scale
slots per learning backend; `GraphEdge` stays 36 bytes. Gate
`wall_clock_ms_per_creature_tick` (F06 epoch 0.0025164 ms) is predeclared
within the 25% flag threshold, expected within ±5% as a hypothesis; a `flag`
is reported with its mechanism, named from those terms; `severe` is a defect.
`reward_learning_ms` is expected at or below today's on the same hypothesis.
The workflow's telemetry checks (parent comparison, overhead check) stand in
for the gates a telemetry feature skips; F07 skips none, its added run work
is what the gate measures, and its four families and census rows sit behind
the same telemetry build switches as F06's, which the reviewer checks.

Counters: `plasticity_updates` and `plasticity_changes` per creature-tick
have no predeclared direction: members join the plastic set and each
contribution counts (invariant 9's definition change is recorded);
`plasticity_scale_snaps` is new, no reference. Every per-seed counter moves
with the draw stream, with no direction; a `severe` elsewhere is a trajectory
reading reported to the user. The gate and goal-worlds epochs are re-pinned
in the closing commit.

Goal (2,000 ticks): no predeclared direction for any indicator. Readings
from the goal summary per world, including zero: plastic edges by scope,
scale slots by inheritance, learned-scale drift and at-bound share (the
`plasticity_census` block at the last retained sample), and the first-update
snaps of genomic scales (`plasticity_scale_snaps` from the totals against
the scale-slot count, the design note's §8 reading).

**Measured verdict.** Pending.

- Summaries: [gate](../../progress/features/t25-f07-group-scope-learning.json),
  [goal](../../progress/features/t25-f07-group-scope-learning-goal.json).
- Full readings: [`docs/progress/readings/t25-f07.md`](../../progress/readings/t25-f07.md).

## Success Criteria

- [ ] Track criterion 7: scope and `inherit_scale` are heritable; a group-scope update is one pattern-weighted change to the shared scale, bounded once; every attribute mutation draws its scope.
- [ ] A plastic member learns at scope `Edge` or `Group` on both shapes; a static member contributes nothing; a scale seeds a child only with `inherit_scale`.
- [ ] `SetScope` and learned creation are in the catalog with the pinned positions and weights; the nine operators land on one member or on every admitted member by one coin.
- [ ] The census, telemetry families, panels, bench summary, payload and inspector show scope and learned scales; `plasticity_scale_snaps` reaches the `SimStats` totals, its telemetry family and the closure readings.
- [ ] Verification Plan checks run and recorded; the mutation gate has no unresolved survivor; epochs re-pinned.

## Notes for AI Agents

- Decision: the four plasticity laws, the four outcome channels, the clamps, the draw ranges and the Lamarckian seed rule are unchanged (T25.F06); the shared credit is one pattern-weighted change to the scale, bounded once (user, 2026-10-13).
- Decision: refactors R1–R6 in the readings file are in-feature scope; no `WORKAROUND(T25.F07)` marker is written.
- Deferred: the 40,000-tick reading of learned scales (retention, drift) goes to the next goal-long carrier; T25's carriers are F02 and F04 (user, 2026-10-15).
