# T25.F08 — Creation Forms, Detach and the Catalog

**Status**: In Progress
**Last updated**: 2026-10-19
**Feature**: T25.F08
**Track**: [T25 — Edge Groups and Per-Edge Learning](../../roadmaps/t25-edge-groups-and-learning.md)

## Goal

A mutation in a developmental gene changes every synapse it builds; a
mutation at one synapse changes only that one. Group creation has two forms:
the uniform form (F07) draws one learning mode, one scope and one inheritance
flag for the whole group, and the randomized form, two new operators
`AddRandomizedRingGroup` and `AddRandomizedDirectionPairGroup`, draws a mode,
its fields and a scope independently for every member it builds. A member can
be detached: `DetachGraphEdge` turns one member into an ordinary edge that
keeps the weight it weighed, its plasticity and its composed birth seed, and
a last member's detach removes the group. `AddGraphEdge` never joins a group,
now pinned. The catalog is one consistent set of edge and group operators:
every Graph operator has one key, one weight, one complexity effect and one
engine operator, pinned by a bijection test beside the existing identity
map. The `Slots` inferred-set refinement (`RefineHeritableStructure`) is
deleted on the post-F05 readings; its reporting channel becomes the scope
channel that counts committed attribute changes by the scope they landed
on, and the bench summary exports the per-operator event funnel so the
readings can group it by layer.

The natural analog: an indirect encoding lays down a regular connection
pattern and a direct encoding then refines single connections, which outruns
either alone (HybrID, Clune, Beckmann, Pennock & Ofria, ECAL 2009, LNCS
5778, doi:10.1007/978-3-642-21314-4_17, Crossref-verified 2026-10-19); a
developmental, pleiotropic gene against a synapse-local change. Randomized
creation is a developmental program whose members are born with
synapse-type-specific plasticity each (F07's source, Larsen & Sjöström 2015).

## Non-Goals

- A `Slots` group shape (a later row, design note D7); any other new shape.
- A per-operator breakdown of the attribute-scope counters (invariant 8):
  two totals answer the design note's reading.
- Generating the operator enums from a table (D11): the bijection test pins
  agreement instead.
- The three other domains' test-only `random()` samplers (Deferred below).
- Any change to the learning laws, clamps, traces, seeds or `from_seeds`,
  to the uniform operators' draws, or to catalog weights beyond the three
  additions and the one deletion.
- F09's retention fields; the VM backend; `Drive` sinks as group targets;
  compute-input edges as members (D11).

## Inputs and Invariants

| Input | Where | What F08 takes from it |
| --- | --- | --- |
| Track row, success criterion 8, Notes | `docs/roadmaps/t25-edge-groups-and-learning.md` | The goal; "group creation is uniform or randomized, a member can be detached, and the catalog is one consistent set of edge and group operators with a bijection test"; the 2026-10-13 decision (group creation randomizes learning attributes, never the pattern); "F08 decides the `GroupKey::Slots` refinement disposition from the refinement-by-key readings F04 and F05 record"; verification "F08 gate + goal", mutation gate and slow tests run on all nine; every feature from F02 on predeclares its re-pins, one cause per commit. |
| Design note | `docs/strategy/edge-groups-and-learning-design-2026-10-13.md` §4 (randomized creation, `DetachGraphEdge`, the no-join rule, refinement disposition D7), §8 F08 contract (scope, operators, re-pins, tests first, readings, reference specs) | The contract below; the track row is authoritative where the two differ; §8's catalog totals are corrected in invariant 1. |
| Refinement-by-key readings | F04 `docs/progress/readings/t25-f04.md` (40,000 ticks, pre-F05: thousands of applications per world under whole-family seeding); F05 (`t25-f05.md`, ruling 2026-10-18), F06 and F07 goal summaries: `refinement_applications_by_key` empty on all nine post-F05 world runs at 2,000 ticks | The `Slots` decision (invariant 5). |
| Dependency outputs | T25.F03 (`Membership`, `Group`, `members`, `detach_at`, I10, `group::detach`), T25.F04 (pair groups, `member_pattern`), T25.F06 (`PlasticityConfig`, `draw_hebbian_config`, `draw_modulation`), T25.F07 (`Scope`, `inherit_scale`, `draw_learning`, `scope_targets`, `select_eligible`, `is_member`, invariant 6 detach seeds, invariant 7 learned creation, `SetScope` at `ALL[29]`, catalog 30 / 60 / 63) | What the three operators compose; nothing in the container or the learner changes. |
| Today's code | `mutation/graph/{mod.rs,group.rs,hebbian.rs:179-262,refinement.rs,operators.rs:499-516}`, `mutation/engine/mod.rs:286-291,424-432,631-657,727-748`, `mutation/types/mod.rs:31-46,260-337,415-556,666-672`, `creature/genome/cgp/{container.rs:1080-1273,group.rs:160-173}`, `config/simulation.rs:819-830,871-873`, `v3-cli/src/bench/{schema.rs:1067-1090,tracking.rs:679-699}`, `v3-telemetry/src/metrics.rs:723,793-800` | The structures this feature extends or deletes; the readers table. |
| Pins | `tests/viability.rs` founder-only digests (must not move); every mutation-on pin (`tests/baseline_worlds.rs`, `tests/applied_trajectory.rs`, `mutation/engine/recruitment_tests.rs`, `mutation/graph/tests/group.rs:153`, v3-cli trajectory and deterministic fixtures, the slow-tier recruitment-paths readings); `crates/v3-cli/tests/bench_artifacts.rs` recipe config digests; `docs/progress/benchmark-series.json` (gate and goal epochs at F07) | The predeclared re-pins (invariant 7). |

Vocabulary: a **creation form** is `Uniform` (one mode, one scope, one
`inherit_scale` for the group, F07) or `Randomized` (a mode and a scope per
member, one `inherit_scale`); the **group layer** is exactly these nine
keys: `Graph.AddRingGroup`, `Graph.AddDirectionPairGroup`,
`Graph.AddRandomizedRingGroup`, `Graph.AddRandomizedDirectionPairGroup`,
`Graph.AlterGroupScale`, `Graph.ShiftGroupOffset`,
`Graph.RetargetGroupKind`, `Graph.RemoveGroup` and `Graph.SetScope`
(`SetScope` edits a group attribute although its key lacks "Group"); the
**edge layer** is the other 23 Graph operators, `DetachGraphEdge` and
`RecruitNeutralInput` included; the grouping is applied to the exported keys
in the readings file, never exported.

| # | Invariant |
| --- | --- |
| 1 | **Catalog.** `GraphOperator::ALL` is today's list without `RefineHeritableStructure` and with `AddRandomizedRingGroup`, `AddRandomizedDirectionPairGroup`, `DetachGraphEdge` appended after `SetScope`, in that order: 32 entries, the group slice at `ALL[22..28]`, `SetScope` at 28, the three at 29–31. Weights 2, 2, 2; complexity `Increasing`, `Increasing`, `Neutral`; keys `Graph.AddRandomizedRingGroup`, `Graph.AddRandomizedDirectionPairGroup`, `Graph.DetachGraphEdge`; `MutationOperator` gains the three variants and loses `GraphRefineHeritableStructure`: `all()` is `[Self; 65]`, `random()`'s default arm (excluding `RecruitNeutralInput` only) sums to 66, the engine's enabled Graph weight sum is 67. Had `Slots` stayed: 33 / 66 / `[Self; 66]`; the design note's "one fewer each" is wrong for the default arm, which never held refinement's weight (`TOTAL_WEIGHT` excludes it). A bijection test pins: `ALL` covers every variant; `operator()` is injective and its image is exactly the `Graph`-domain members of `MutationOperator::all()`; every key is unique and `Graph.`-prefixed; `complexity_effect` agrees between the two enums; `all().len()` equals the four domains' `ALL` lengths summed. The hand-written Graph arm of `complexity_effect_cross_consistency_with_domain_operators` stays as the identity oracle (it catches a swap of two same-tier mappings that the bijection properties admit) and gains the three new arms and loses the refinement arm. |
| 2 | **Randomized creation.** `AddRandomizedRingGroup` applies where `AddRingGroup` does (`group::can_add`), `AddRandomizedDirectionPairGroup` where `AddDirectionPairGroup` does (`can_add_pair`). Each makes its uniform twin's shape draws in the same order (ring or pair, kind, offset) and creates the silent group through the same container constructor; then, per member in canonical order (`members(id)`), independently: mode `gen_range(0..3u8)` (0 static, 1 Hebbian, 2 reward-modulated); a mode above 0 draws one config by `EnableHebbian`'s law (`draw_hebbian_config`), mode 2 adds `draw_modulation`, written to that member; then that member's scope coin `gen_bool(0.5)` (true `Group`); after the member loop one `inherit_scale` coin `gen_bool(0.5)`. Scale and own weights stay 0 and the pattern is never drawn: the group is born silent. `Δgenome_size` +9 and +13 as the twins. One creation function takes the form (`Uniform` or `Randomized`), so the shape draws and the container call exist once; the uniform operators' draws are unchanged draw for draw (the existing creation tests still pass unedited). |
| 3 | **Detach.** `DetachGraphEdge` applies to a Graph node with a member (`any_edge(def, is_member)`); it draws one member uniformly in canonical order (`select_eligible(def, is_member)`): one draw, no coin, no value draw; then `group::detach(def, occurrence, input_refs)`, the F03 primitive, so the container does what a retarget's detach does: own weight `:= compose(own, scale, pattern)` bitwise, membership (and scope) cleared, plasticity kept, the edge moved to the end of its sink's independent prefix, its seed composed as F07 invariant 6 defines (a Lamarckian member with an edge or group seed carries the composed seed to the child; a static member's composed row entry is dropped at construction as F06 defines, so it carries none). A last member's detach drops the record and renumbers higher `GroupId`s (`detach_at` returns `removed`). `Δgenome_size` is 0, or −1 when the record goes; the operator is `Neutral`. A committed `DetachGraphEdge` counts one `graph_edges_detached`, and one `groups_removed` when the record went: `group_edits` lists it among the detaching operators, pinned by `a_committed_detach_counts_one_graph_edge_detached`; the whitelist over a tally diff is an accepted workaround (readings R4) and carries `WORKAROUND(T25.F08): operators report their group edits; the container's detach_at already returns whether the record went` at the site. |
| 4 | **No-join.** `AddGraphEdge` (and `InputRef.Add`) push an ordinary edge through `push_edge`, which inserts before a sink's first member (I10); the pushed edge has no membership and the sink's groups keep their member sets and records. No code changes; `add_graph_edge_never_joins_a_group` pins it on a sink that holds a group. |
| 5 | **`Slots` deleted.** Decision (spec owner, from the readings the track row names): on every post-F05 world run (F05, F06, F07 goal summaries, nine runs at 2,000 ticks) `refinement_applications_by_key` is empty; the F05 ruling explains why (a set needs two independent one-slot recruitments of the same field onto one sink); F04's counts came from pre-F05 whole-family seeding that no longer exists; and an inferred set beside explicit groups is two mechanisms for one concept (D7). The mechanism goes: the operator, `refinement.rs` and its tests, `RefinementKey`, the refinement channel (`GraphEventReport.refined`, `SelectedEvent.refined`, `MutationSummary.refinement_applications_by_key` and `record_refinement`, `SimStats.refinement_applications_total_by_key` and its roll-up, the bench field, the telemetry family), which invariant 8's attribute-scope channel takes over site for site, `mutation.structured_heritable_refinement` with its default, the `enabled` arm and the restricted-pressure special case (under restricted size pressure the Graph arm admits decreasing operators only), the telemetry family, the bench field, the frontend toggle and type, the key in the two checked-in recipes (`deny_unknown_fields`), and the reference-spec rows (deletion table below). `RecruitNeutralInput` stays the one config-gated Graph operator. |
| 6 | **Export.** `MutationSupply.operator_funnel_by_operator: BTreeMap<String, OperatorFunnel>`, keyed by `MutationOperator::as_key()`, `#[serde(default)]`, each value the five stages of `MutationOperatorFunnel` (`attempted`, `applicable`, `structurally_valid`, `applied`, `skipped`), projected from `SimStats.mutation_operator_funnel_total_by_operator`, replaces `refinement_applications_by_key` in the `mutation_supply` block; a projection test like F04's keeps it through the summary. The per-operator skip-reason map is not exported. The layer grouping is derived in the readings from the keys. |
| 7 | **Identity and determinism.** No genome field changes: `GENOME_FORMAT` 5 and `PROTOCOL_VERSION` `v3alpha10` stand (the protocol spec does not enumerate `MutationConfig` fields). Commit 1 (deletion) changes every unrestricted Graph draw under a config that enabled refinement (the enabled weight sum 65 → 61: the default) and the resolved config, so every mutation-on pin at the default config and the three recipe config digests re-pin (undo step: re-inserting `mutation.structured_heritable_refinement: true` recovers the T25.F02 commit-3 digests); a fixture that disabled refinement (`mutation/engine/recruitment_tests.rs:9-12`, `tests/mutational_neighborhood.rs:121`) drew from the same enabled set before and after and holds through commit 1, which the implementer asserts, not re-pins. Commit 2 (the three operators, the export and the scope channel) changes every unrestricted Graph draw under every configuration (default 61 → 67, recruitment-Off 60 → 66) and the summary schema, so every mutation-on pin re-pins. The restricted-pressure selection (`engine/mod.rs:283-291`) changes in commit 1 under a config that enabled refinement, since it admitted `RefineHeritableStructure` at weight 4 beside the decreasing operators and no longer does; it is unchanged under a refinement-disabled config and unchanged through commit 2, which adds no decreasing operator. At `per_unit_rate` 0 no simulation behavior, genome text or RNG evidence moves (founder-only viability digests, every rate-0 fixture); the config digests (commit 1) and the summary schema (commit 2) move regardless of rate. Commit 3 is reference text only. Telemetry draws no RNG. |
| 8 | **Scope of attribute changes.** The ten attribute operators (the nine plasticity operators and `SetScope`) report the scope their committed change landed on: `Group` when the picked edge was a member and the coin was true, else `Edge` (a non-member pick draws no coin and lands on itself). `scope_targets` is the one source of that fact and returns it with the targets; the operator passes it up through `GraphEventReport { class: Option<BirthEdgeClass>, attribute_scope: Option<Scope> }`, which takes the deleted refinement channel's place at every site: `SelectedEvent`, `MutationSummary.attribute_changes_by_scope` (`[u32; 2]` or two fields, `record_attribute_scope`), `SimStats.mutation_attribute_changes_total_by_scope`, the reproduction roll-up, `mutation_supply.attribute_changes_by_scope { edge, group }` in the bench summary, and the telemetry family `petri.run.mutation_attribute_changes_by_scope` with `petri.scope` in `{Edge, Group}`. A rolled-back or skipped event reports none; a creation or detach reports none. The share read at closure is `group / (edge + group)` per world. No RNG is drawn and no selection changes. |

Deletion list for invariant 5 (every site `grep` finds today):

| Area | Sites |
| --- | --- |
| v3-core mechanism | `mutation/graph/refinement.rs`, `refinement_tests.rs`, `mutation/engine/refinement_tests.rs`; `mutation/engine/group_tests.rs:146-191` (the refinement-only test) and `:128-141` (the rollback test keeps its `GroupEdits` assertion and drops the refinement channel); `GraphOperator::RefineHeritableStructure` (`ALL`, `weight`, `operator`, `enabled`, `applies_to`, both dispatch arms, `TOTAL_WEIGHT` and `random` exclusions); `MutationOperator::GraphRefineHeritableStructure` (`as_key`, `domain`, `complexity_effect`, `all`); `RefinementKey`; `GraphEventReport`; `MutationSummary.refinement_applications_by_key`, `record_refinement`; `SimStats.refinement_applications_total_by_key`; `reproduction.rs:501-503` roll-up and its test at `:1414-1490`; `engine/mod.rs:290` restricted special case, `:430-432`, `:742` |
| v3-core config and tests | `config/simulation.rs:824-825,871-873,890`; `applicability_tests.rs` config literals; `mutational_neighborhood.rs:121,132` (the exclusion list keeps `RecruitNeutralInput`); `runtime/cgp/neutral_connection_tests/refinement.rs` refinement cases (a case that pins only serde, copy or pruning persistence of plain edges is kept or moved); `graph/tests/group.rs` `add_and_refinement_are_unchanged_by_groups` becomes add-only; `recruitment_tests.rs` mentions |
| v3-cli | `bench/schema.rs:1086-1089`, `tracking.rs:691-695`, `tracking/tests.rs:733-734,797,1402`, `artifacts.rs:1396` fixture; `tests/bench_artifacts.rs` digest chain (`:235-250,286-300,395-404`) |
| v3-telemetry | `metrics.rs:793-800` family `petri.run.refinement_applications_by_key` (its site takes the invariant-8 family); `metrics/tests.rs:168-169,512,922` |
| scripts, frontend, recipes | `scripts/telemetry-dashboards-check:228` comment; `frontend/src/types/config.ts:225`, `config-panel/runtime/MutationSection.tsx:149-155`, `groups.ts:237`, `test/fixtures.ts:98`, `ControlBar.test.tsx:118`; `world-recipe-blobs-and-fruit.json:58`, `experiments/worlds/blobs-and-fruit-substrate.json:58` |
| reference specs | `docs/reference/v3-mutation-spec.md` refinement entry (`:504-547`), `:536`, `:545`, `:628`, `:835` and the operator table; `docs/reference/v3-runtime-config-spec.md:99` |

Readers, with their post-change expectations:

| Reader | After F08 |
| --- | --- |
| `v3_core::neighborhood` screen | iterates `GraphOperator::ALL`: three new rows, one fewer; `op.enabled` gates recruitment only |
| bench summary | `mutation_supply.operator_funnel_by_operator` (invariant 6); `graph_edges_detached_total` and `groups_removed_total` now include `DetachGraphEdge` |
| telemetry | `petri.run.mutation_events_applied_by_operator` carries the three new keys and no refinement key; `petri.run.refinement_applications_by_key` removed with its test rows; `run.started` body loses `mutation.structured_heritable_refinement` |
| frontend | the mutation section has no refinement toggle; `types/config.ts` and fixtures drop the field |
| reference specs | `v3-mutation-spec.md` operator table: the three rows, the randomized-creation paragraph, the detach paragraph, the no-join sentence, catalog counts 32 / 66; `v3-runtime-config-spec.md` without the refinement row |
| `docs/progress/features/*.json` | historical, untouched |

Options weighed, the preparatory refactor audit and the Codex rounds:
[the readings file](../../progress/readings/t25-f08.md).

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.mutation_events_applied_by_operator` | counter (existing family) | committed events per operator; gains `Graph.AddRandomizedRingGroup`, `Graph.AddRandomizedDirectionPairGroup`, `Graph.DetachGraphEdge`, loses `Graph.RefineHeritableStructure` | `petri.operator`; `mutation.per_unit_rate`, `mutation.neutral_input_recruitment` in the `run.started` body |
| `petri.run.graph_edges_detached` | counter (existing) | now also a committed `DetachGraphEdge` | as above |
| `petri.run.groups_removed` | counter (existing) | now also a last member's `DetachGraphEdge` | as above |
| `petri.run.mutation_attribute_changes_by_scope` | counter (new family over two fixed counters, so both series are always exported, zeros included; `metrics/tests.rs` expected-families row; `petri_run_mutation_attribute_changes_by_scope_total` registered in `scripts/telemetry-dashboards-check` `always_names`) | committed attribute changes by the scope they landed on (invariant 8) | `petri.scope` ∈ `{Edge, Group}`; `mutation.per_unit_rate` in the `run.started` body |
| `petri.run.refinement_applications_by_key` | counter, removed | the mechanism is deleted | — |
| `mutation.structured_heritable_refinement` | recorded configuration, removed | the field is deleted from `MutationConfig` | — |

## Verification Plan

**Question.** F08 claims that a group can be born with per-member learning
attributes, that one member can leave its group with the weight it weighed,
and that the catalog is one consistent, bijection-pinned set with the inert
`Slots` refinement gone at no cost to the tick loop; a gate wall-clock flag,
a goal world where neither randomized creation nor detach is ever applied,
or a harmful-per-carrier share that rises would show it wrong. A gate
wall-clock `severe` is a defect; a gate counter `severe` is a trajectory
reading under the changed draw stream (Performance: no predeclared
direction), reported to the user and re-pinned in the closing commit, never
tuned away; the other two are predeclared hypotheses whose failure is
reported with a recorded disposition at closure, not a closure blocker
(Success Criteria require the readings).

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate + goal` | the track plan names F08 "gate + goal": a production change to the operator catalog whose readings (the operator funnel by operator and by layer, detaches, group removals, census by scope, harmful share) a 2,000-tick run takes; goal-long is rationed to F02 and F04. |
| Mutation gate | `run`: fresh `make rust-mutants-ci`, with local `MUTANTS_ITERATE=0 make rust-mutants` as the fallback when Actions is unavailable | new production logic in `v3-core` and `v3-cli` (two creation operators, detach, the engine's detach count, the export) that tests must pin. |
| Slow tests (`make rust-test-slow`) | `run` | the operator catalog and every Graph draw change, which moves the recruitment-paths recorded readings the tier pins. |

## Implementation Tasks

- [x] Commit 1, `Slots` deletion (invariant 5, the deletion table): tests
      first, `restricted_pressure_admits_decreasing_graph_operators_only`
      and the bench projection without the key; delete the mechanism, the
      config field and every site; collapse `GraphEventReport`; drop the
      refinement arm of the identity map and the catalog counts to 29 / 60 /
      62; re-pin every default-config mutation-on pin and the three recipe
      config digests with the undo step, and assert the refinement-disabled
      fixtures unchanged; the `WORKAROUND(T25.F08)` comment on
      `GraphOperator::random` (readings R1).
- [x] Commit 2, the three operators, the export and the scope channel
      (invariants 1–4, 6, 8): tests first,
      `a_committed_attribute_change_reports_the_scope_it_landed_on` (a
      member pick with a true coin, a false coin, a non-member pick,
      `SetScope`; a rolled-back event reports none), its summary projection
      and telemetry rows,
      `randomized_creation_draws_shape_then_per_member_mode_fields_scope_then_inherit_scale`
      (both shapes, replayed draw for draw against a fixture built by the
      container), `uniform_creation_draws_are_unchanged`,
      `detach_draws_one_member_and_keeps_its_composed_weight_plasticity_and_seed`
      (a Lamarckian member with a seed and a static member),
      `a_last_member_detach_removes_the_group_and_counts_one_removal`,
      `a_committed_detach_counts_one_graph_edge_detached`,
      `applicability_follows_members_for_detach`,
      `add_graph_edge_never_joins_a_group`,
      `the_graph_catalog_is_a_bijection_onto_its_engine_operators`, the
      three new arms of the identity map and the
      `the_catalog_keeps_six_operators_at_their_edge_tiers` counts
      (32 / 66 / 65); one creation function parameterized by form (readings
      R6); `DetachGraphEdge` in `group_edits` with its `WORKAROUND(T25.F08)`
      comment (readings R4); `operator_funnel_by_operator` with its
      projection test; re-pin every mutation-on pin.
- [x] Commit 3, reference text only: `v3-mutation-spec.md` (the three rows,
      the randomized-creation and detach paragraphs, the no-join sentence,
      the refinement entry removed, counts 32 / 66) and
      `v3-runtime-config-spec.md` (the refinement row removed).
- [ ] `make rust-test-slow` and re-pin its readings; then one `make check`.

## Verification

- [x] Viability first: `cargo test -p v3-core --test viability` -> 30/30,
      founder digests unchanged at every build commit, after the simplify
      pass and after the review remediation.
- [x] Crate tests: `cargo test --no-fail-fast -p v3-core -p v3-cli` -> every
      target passes (the named tests of Implementation Tasks included) but
      `v3-cli --test bench`
      `gate_profile_has_no_severe_regression_against_series_references`,
      the open-until-closure row of the pins table; `-p v3-telemetry
      -p v3-server` pass at `40ec50fe`; `--features telemetry-seams` window
      seam 20/20 at `40ec50fe`.
- [x] Lints: `cargo fmt --all --check`, `cargo check --workspace
      --all-targets`, `cargo clippy --workspace --all-targets` -> clean.
- [x] Frontend (config panel, control bar): vitest 106/106, `tsc --noEmit`
      and biome clean at `40ec50fe`; the simplify pass and the review
      remediation touch no frontend.
- [ ] Mutation gate: fresh `make rust-mutants-ci` (local
      `MUTANTS_ITERATE=0 make rust-mutants` when Actions is unavailable), its
      summary line, output path, and every survivor resolved as killed,
      equivalent, or deferred.
- [ ] `make rust-test-slow` -> pass with the re-pinned recruitment-paths
      readings.
- [x] Gate and goal (measured 2026-10-20 at `fea7746d`; both `severe=true` as
      predeclared draw-stream trajectory readings, series entries still to
      re-pin in the closing commit): summaries at `docs/progress/features/t25-f08-creation-forms-detach-and-the-catalog.json`
      and `-goal.json`, local raw hash/byte count and verification time
      checked, series entries re-pinned in the closing commit, no full report
      staged; the readings below recorded in the readings file.

Pins moved by the build (each read on two agreeing runs; the founder-only
viability digests hold at every commit):

| Commit | Pin | Was -> now | Cause |
| --- | --- | --- | --- |
| 1 | `bench_artifacts.rs` recipe config digests | `5ab6069b…`, `87ad3188…`, `b33044d5…` -> `62fcf74b…`, `3d595bef…`, `e4d5bef9…` | the resolved `MutationConfig` drops `structured_heritable_refinement`; re-inserting it at `true` recovers the old three (asserted in the chain) |
| 1 | `applied_trajectory.rs` sampled / sampled rate 0 / guard / guard rate 0 | `38f68975…` / `0d037fb0…` / `31bd6e95…` / `ec2e084d…` -> `f55ad21d…` / `872113ec…` / `dc32a1fa…` / `d61cf3eb…` | enabled Graph weight 65 -> 61 remaps every Graph draw |
| 1 | `baseline_worlds.rs` short-run identity, default / rate 0 | `7345675788631625454` / `11318987159188275955` -> `5737684664837370108` / `11540034954633671093` | same |
| 1 | recruitment-paths known-deleting lineage (`--lib`) | arm 12 batch 1 lineage 3 -> batch 22 lineage 0 (generation 7, at 263, depth 47) | same; the first `Deleted` lineage of the recorded search (batches 0-40, lineages 0-15) |
| 1 | v3-cli real-run-path seeds (`bench/tests.rs`, three tests) | 129, 129, 164 -> 73 | same; 73 is the lowest of 0..=400 surviving the horizon (73, 147, 158, 197, 201), over three cells |
| 1 | v3-cli cadence seed (`bench/run/tests.rs`) | 3 -> 9 | same; seed 3 goes extinct at tick 93 |
| 2 | `engine/recruitment_tests.rs` recruitment-Off digest | `6729442929459510286` -> `1771622774480443924` | enabled Graph weight 60 -> 66 under recruitment Off remaps every Graph draw |
| 2 | `applied_trajectory.rs` sampled / sampled rate 0 / guard / guard rate 0 | `f55ad21d…` / `872113ec…` / `dc32a1fa…` / `d61cf3eb…` -> `9458abf6…` / `a187a3fe…` / `3d299de5…` / `7d7ae0e2…` | enabled Graph weight 61 -> 67 remaps every Graph draw |
| 2 | `baseline_worlds.rs` short-run identity, default / rate 0 | `5737684664837370108` / `11540034954633671093` -> `5477079874225389176` / `2120864728947376946` | same |
| 2 | recruitment-paths known-deleting lineage (`--lib`) | batch 22 lineage 0 -> batch 1 lineage 3 (generation 6, at 262, depth 112) | same; the first `Deleted` lineage of the recorded search |
| 2 | `simulation/actions` stats test seed | 101 -> 100 | same; 101 meets a domain-level skip, 100 is the first seed from 100 without one |
| 2 | `priority_bid_reachability.rs` search bound | 1,024 -> 2,048 seeds | same; seed 2,026 is the first lineage reaching a positive bid beside a real action |
| 2 | window-seam equivalence seeds (`--features telemetry-seams`) | graph 12 -> 14, VM 24 -> 25 | same; at 12 and 24 a traced target dies inside its window |
| 2 | v3-cli real-run-path seeds | post-tick and census 73 -> 129, occupancy 73 -> 473 | same; 129 is the only survivor of 0..=400 and ends on one occupancy cell, 473 the first survivor above 400 |
| 2 | v3-cli cadence seed | 9 -> 14 | same; 14 is the lowest of 0..=200 that survives |
| 1, 2 | open until closure | `v3-cli --test bench` `gate_profile_has_no_severe_regression_against_series_references`: severe against F07's series reference, first reported `vm_steps` 0.073814 vs 0.034618 at commit 1 and `decided_passes` 0.000997 vs 0.000140 at commit 2 (reduced-fixture gate) | the series references re-pin with the gate and goal epochs in the closing commit |
| 1 | held, asserted | `engine/recruitment_tests.rs` recruitment-Off digest; `tests/mutational_neighborhood.rs` disabled rows | drew from the same enabled set before and after |

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: HybrID's
indirect-then-direct refinement and the developmental-gene / single-synapse
distinction (Goal); it reaches creatures through mutation at birth only.
The founder has no group and no plastic edge, so every mutation-off reading
is identical.

Compute: no tick-loop, learner or executor code changes; the three
operators run at birth with the cost of their twins (a creation's per-member
draws, a detach's one container move); the export runs once at summary time.
Gate `wall_clock_ms_per_creature_tick` (F07 epoch 0.0038113 ms) is
predeclared within the 25% flag threshold, expected within ±5% as a
hypothesis; a `flag` is reported with its mechanism; `severe` is a defect.

Counters: every per-seed counter moves with the draw stream, with no
direction; `graph_edges_detached_total` and `groups_removed_total` are
expected up (a new detaching operator at weight 2) and are not thresholded;
`plasticity_*` have no direction. A `severe` elsewhere is a trajectory
reading reported to the user. The gate and goal-worlds epochs are re-pinned
in the closing commit.

Goal (2,000 ticks), per world, including zero, from the goal summary:
`operator_funnel_by_operator` for the three new operators (five stages
each) and the funnel grouped by layer (edge layer, group layer, from keys);
`graph_edges_detached_total` and `groups_removed_total` against F07's; the
share of attribute changes applied at group scope,
`attribute_changes_by_scope.group / (edge + group)` (invariant 8), beside
the plasticity census by scope (`edges_by_scope`, the prevalence of stored
learner scope); harmful share per carrier,
`mutation_outcome_summary.detrimental_total / carriers_observed_total`,
against F07's per world, hypothesis not up, reported either way; the `Slots`
firing rate is recorded as "deleted on nine empty post-F05 readings".

**Measured verdict (2026-10-20, HEAD fea7746d, each profile run once).**
Gate: v3-cli exit 3 (source: `measurement_evidence.cli_exit`, successful
artifact pair with a severe), outer `make` exit 2; `severe=true`, one severe,
`decided_passes` 0.000997 vs 0.000140 (+612%), the draw-stream trajectory
reading ruled by the spec owner; `wall_clock_ms_per_creature_tick` 0.0025481
vs epoch 0.0038113 (-33.1%), inside the 25% band, no flag. Goal: exit 3 /
make 2, `severe=true`: `vm_steps` 2.0196 vs 0.5709 (+253.7%),
`plasticity_updates` 3.2016 vs 1.7537 (+82.6%), `decided_passes` 0.002550 vs
0.001407 (+81.2%), all trajectory readings re-pinned at closure; flags on
`mesh_hops`, `graph_relax_iters`, `actions_applied`, `passes`. Group share of
attribute changes: Orchards 0.257, Canyon 0.116, Confluence 0.371. Detached
edges 400 / 12 / 932 (F07: 112 / 33 / 195), groups removed 244 / 5 / 608
(F07: 250 / 69 / 365). Harmful share 0.360 / 0.477 / 0.438 (F07: 0.360 /
0.434 / 0.399), up in two worlds. New operators applied: randomized ring
1,687 / 411 / 1,453, randomized pair 156 / 17 / 211, detach 262 / 6 / 633.
Raw gate 118,732 B, sha256 `b19ce575...b9f8`; raw goal 699,473,823 B, sha256
`76586a4a...f8baf8`; summaries 23,772 B and 482,625 B.

- Summaries: [gate](../../progress/features/t25-f08-creation-forms-detach-and-the-catalog.json),
  [goal](../../progress/features/t25-f08-creation-forms-detach-and-the-catalog-goal.json).
- Full readings: [`docs/progress/readings/t25-f08.md`](../../progress/readings/t25-f08.md).

## Success Criteria

- [ ] Invariants 1–8 hold and their named tests pass; the uniform
      operators' creation tests pass unedited.
- [ ] `RefineHeritableStructure`, `structured_heritable_refinement` and
      `refinement_applications` appear in no production code, config,
      recipe, script, frontend source or reference spec; test code may name
      them only in the config-digest undo chain invariant 7 requires, in an
      assertion that the old field or key is absent, and in pin-history
      comments; documents under `docs/archive`, `docs/strategy`,
      `docs/progress` and closed specs are historical.
- [ ] The goal run reports the three new operators' funnels, the funnel by
      layer, detaches and removals, the share of attribute changes at group
      scope, the census by scope and the harmful share per world in the
      readings file.
- [ ] Gate and goal summaries stored, epochs re-pinned, the mutation gate
      and the slow tier recorded.

## Notes for AI Agents

- Decision: `Slots` is deleted on the post-F05 readings (invariant 5); a
  `Slots` group shape would be a new row, never a revival of inferred sets.
- Decision: the attribute-scope channel (invariant 8) is two totals that
  reuse the deleted refinement channel's sites; a per-operator breakdown is
  a Non-Goal (KISS over verification machinery, 2026-09-07).
- Deferred: `group_edits`'s operator whitelist over a tally diff (readings
  R4) is an accepted, marked workaround; the same area-mode row as R1 pays
  it by having operators report their group edits.
- Decision: the design note's F08 catalog totals are corrected to
  32 / 66 / 65 (delete) and 33 / 66 / 66 (keep): the default arm never held
  refinement's weight.
- Decision: the two creation forms share one creation function per shape
  (readings R6); a later form is a variant, not a copy.
- Deferred: the four domains' test-only `random()` / `TOTAL_WEIGHT`
  samplers duplicate the engine's `select_weighted_index` draw; an area-mode
  `$refactor-audit` row over `crates/v3-core/src/mutation` removes all four
  at once (readings R1); F08 marks the Graph one.
- Deferred: a long-run reading of detached members' retention and of
  randomized groups' survival goes to the next goal-long carrier; T25's
  carriers (F02, F04) are spent.
