# T30.F01 — Inspector Brain View

**Status**: In Progress
**Last updated**: 2026-10-26
**Feature**: T30.F01
**Track**: [T30 — Core Boundaries](../../roadmaps/t30-core-boundaries.md)

## Goal

A person inspecting a creature sees its brain as the simulation applies it:
every edge's genomic, composed and applied weight, every member's site and
pattern, every learner's initialization and inheritance, resolved by the
genome's and the runtime's read APIs and carried by wire types `v3-server`
owns. The frontend's types for every HTTP and WebSocket payload are generated
from those Rust types and checked by a test; the paint tools read
server-owned types. No simulation behavior changes.

## Non-Goals

- Changing what the inspector shows beyond what the view resolves: no new
  panels, layouts or metrics. T25.F11, T25.F12 and T29.F02 change the view
  with their semantics.
- Deriving the action log from the applied action result (G18, T30.F06); this
  feature ships the server-owned entry with today's fields.
- Moving `PatternParams` into `config` (G10, T30.F04) or the genome out of
  `creature/` (G24, T30.F07); the derive goes on the types wherever they live
  at integration.
- Generating request types the frontend composes from generated ones
  (`StartupRequest`'s `Partial<…>` nesting stays a frontend composition).
- Snapshot, genome JSON (`GENOME_FORMAT`) and telemetry formats.

## Inputs and Invariants

| Input | Where | What it fixes |
| --- | --- | --- |
| Row and Notes (2026-10-25, 2026-10-26) | `docs/roadmaps/t30-core-boundaries.md` | Applied semantics, not a future model; heterogeneous scale contributors and `DirectionPair` fact members as preservation cases; `memberPattern` and `learnerInitialized` deleted; `ts-rs` in `v3-server` with a comparing test, an explicit write flag, 64-bit integers as `number`, Biome-formatted output; exempt-type handling the spec owner's; proof of concept on `ServerMessage`, `serialize_with` arrays and `skip_serializing_if` fields; the sampler route and paint (G22) in scope. |
| Desired state | `docs/architecture.md` sections 2, 5 (G17, G22) | The server owns query projections and its wire types; only `Direction`, `Position`, the id types and the config schema cross the wire as core structs; the frontend never copies a core type or recomputes a rule. |
| Genome read APIs | `cgp/container.rs` (`edge_sites`, `edge_at`, `genomic_weight`, `pattern`, `member_site`, `is_fact_member`, `groups`), `cgp/group.rs` (`Group::pattern`, `compose`, `pair_birth_members`, `ring_offset`, `MemberSite`) | Member and composition facts come from here; the frontend's `compassX`, `compassY`, `compose`, `memberPattern`, `PAIR_BIRTH` and relative-direction arithmetic copy them. |
| Runtime read APIs | `runtime/plasticity/learned.rs` (`LearnedState::{edges, scales, applied_weights, applied_weight}`, `LearnedEdge`, `LearnedScale`) | Learner facts; `learned` is `None` for a group learner, so initialization at group scope is readable only through `scales()` (audit R3). |
| Today's wire | `http/creature.rs` (`genome`, `mesh_annotations`, `graph_weights`, `action_log`), `transport/sample_protocol.rs` (`serde_json::Value` aliases), `http/paint.rs`, `http/pattern.rs` | What the view replaces; `graph_weights`' merge-walk in canonical order (T25.F07 invariant 12) is the model for the `learning` payload. |
| Drift evidence | `types/creature-detail.ts:77-87` vs `http/creature.rs:210-221` | Two diagnostics fields missing from the hand copy since `a8822192`; the generated types make this a failing test. |
| Tooling | ts-rs README (`main`, 2026-10-26), crates.io 12.0.1 | `export_to_string`, `TS_RS_LARGE_INT=number`, supported serde attributes (`tag`, `rename_all`, `skip_serializing_if`, `flatten`, `default`), `#[ts(optional)]`. |
| Options and audit | [readings](../../progress/readings/t30-f01.md) | The alternatives weighed and the preparatory refactors R1–R10. |

| # | Invariant |
| --- | --- |
| 1 | **No behavior change.** `v3-core` changes are read APIs and `cfg_attr` derives only: `LearnedEdge` gains `initialized` and `inherits` (R3), the genome gains a group-coordinates read API (invariant 3) whose ring rule, `ring_birth_members`, `add_group` iterates in place of its own loop (one rule for creation and for the view, the same channels in the same order), the exempt types gain a `ts`-feature derive, and `ActionType` and `ActionResult` lose `#[non_exhaustive]` while `ActionResult`, `HebbianRule`, `OutcomeChannel` (with a compile-time completeness check), `MeshReadClass` and `MeshWriteClass` gain a public `ALL` list in declaration order so the wire enums can pin them (invariants 8, 9); `mutation/graph/hebbian.rs` draws from the public `HebbianRule::ALL` and `OutcomeChannel::ALL` in place of its private copies of the same content and order, so no draw changes. No tick, mutation, sensor, birth or energy rule changes; the `ts` feature is off by default, no binary or default build enables it, and enabling it adds derives only, so `--no-default-features` and default builds run the same code; the gate's pins and `make check`'s viability and baseline-world digests are unchanged. The server's view construction draws no RNG and mutates nothing. |
| 2 | **The brain view.** `GET /v3/simulation/creature/{id}` carries `brain` in place of `genome` and `mesh_annotations`: `entry_node_id` and one node per genome node with its id, the annotation facts (`reachable`, read and write classes, `has_stateful_behavior`, live instruction or internal-node indices), its input references (invariant 6), its backend and its route targets. A Graph backend lists compute nodes and output sinks, each with its kind and its edges; an edge carries its source, `own`, `genomic` (the composed weight, `genomic_weight`), `applied` (`applied_weight`) and, when a member, `member { group, scope, site (direction d or drive), field, pattern, scale }` and, when plastic, `learner` (invariant 4). A VM backend carries `register_count`, `constants` and `program` as a server-owned instruction type, one variant per opcode. Every number comes from a genome or runtime read API; `v3-server` and the frontend contain no `scale × pattern + own`, no compass, no offset arithmetic and no initialization or inheritance rule. `brain` is omitted by `exclude=brain`. |
| 3 | **Groups.** A Graph backend's `groups` lists each record in `GroupId` order: `shape`, `kind`, `offset`, `scale`, `inherit_scale`, `retention`, `learned_scale` (an initialized scale slot's value, else absent) and `coordinates`: its birth coordinates with presence, read from one genome API this feature adds (a `Ring` group's eight cells with the sink direction `add_group` uses, `(channel + offset) mod 8`; a `DirectionPair` group's twelve `pair_birth_members`) and, for a pair, `facts`: the declared facts with presence. A direction member outside the birth coordinates is read from the edge list (`site` a direction, `pattern` 0). The frontend renders presence; it holds no coordinate table. |
| 4 | **Learners.** An edge's `learner` carries `rule`, `learning_rate`, `weight_clamp`, `modulation` (source and trace decay, or absent), `retention`, `scope` (ungrouped, edge or group), `initialized`, `inherits` (the `lamarckian` flag for an ungrouped or edge learner; the group's `inherit_scale` for a group learner, as `capture()` reads them), `learned` (the own quantity when initialized, never at group scope) and `trace` (when initialized and modulated), from `LearnedEdge`. A scale whose contributors differ in rule, rate, reward source and trace is shown as those learners with one `learned_scale` on the group (T25.F07 invariant 4); a fact member is a member with site `drive`, pattern 0 and `genomic == own` (T25.F10 invariant 2); a zero-pattern direction member is listed as present outside the coordinates; an uninitialized learner shows `initialized: false` with `applied` as the runtime resolver returns it, which may differ from its composed weight (an unseeded ungrouped or edge learner starts at its genomic own weight clamped to its bound, T25.F09 invariant 2; a group learner's correction is never clamped, T25.F07 invariant 3). These four and invariant 5's passive member are the preservation cases, each pinned by a server test and a frontend test. |
| 5 | **Learning payload.** `learning`, computed on every poll unless `exclude=learning`, lists per Graph node every dynamic edge of `applied_weights()`, learners and passive members alike (`surface`, `container`, `edge`, `applied`), each learner's entry adding `learned?`, `trace?` and `initialized` from `edges()`, and the scale view (`group`, `learned?`, `initialized`) from `scales()`, all in canonical order by one merge-walk over the three iterators, never by a per-edge search; omitted when no node carries a dynamic edge. A passive member's applied weight follows its group's learned scale, so it is in `learning`, not only in `brain`: the fifth preservation case, pinned by a server test (today's `graph_weights` fixture with genomic 0.5 and applied 1.5) and a frontend test. The frontend joins `learning` to `brain` by `(surface, container, edge)` and draws a listed edge at its `applied`, else at `brain`'s; it invents no runtime value. `graph_weights` is deleted. |
| 6 | **Input references.** One server-owned reference type across `brain` and the sampler: `family` (`world`, `static`, `dynamic`, `upstream`, `action_queue`, `decision`), `key` (the key's variant name, as the catalog fixtures key it), `food_type` for a typed food sensor and `slot` for an upstream slot. The frontend's `parseWorldInputRef` is deleted; its label tables stay keyed by `key`. |
| 7 | **Sampler.** `sample_protocol.rs` has no `serde_json::Value` field: the instruction, the input reference, the world action, the vote sink and the direction are typed; `DirectionPayload` is `Direction` (exempt). Every sample field keeps its JSON name and value except a hop's `input_refs`, which adopts invariant 6's reference type in place of the core enum's externally tagged JSON (`{"UpstreamSlot":3}`, `"ActionVotes"`); its shape fixtures and frontend consumers move with it under `v3alpha14`. One field is added: `GraphTracePayload.temporal_committed`, which the core trace records and the frontend always read but the server never sent (the inspector showed "not committed" for every pass); it is the applied value. |
| 8 | **Paint and action log.** `POST /v3/simulation/paint` takes a server-owned tool enum, and its response and the pattern-apply response share a server-owned stats type, with today's names and values, converted to and from `kernel::paint` at the route (`query/cache.rs` keeps the kernel types; the pattern request takes no tool). The action log entry is a server-owned type with today's fields and variant names, built from `creature::action_log::ActionLogEntry`; `ActionType` and `ActionResult` are server enums converted from the core enums by exhaustive matches with no wildcard arm, and a test pins them over `ActionType::ALL` and `ActionResult::ALL`, so a new core variant fails to compile and to pin. |
| 9 | **Wire ownership.** Every `v3-server` request, response and WebSocket message type derives `TS` under `cfg_attr(test)`, with `ts-rs` a dev-dependency and `v3-core`'s `ts` feature enabled only by `v3-server`'s dev-dependency, so no binary carries either; no core type appears in one except `Direction`, `Position`, the id types and the config schema (the `ts` feature of `v3-core`); no `serde_json::Value` field; every `#[ts(type)]`, `#[ts(as)]` or `#[ts(skip)]` is listed in the readings file with its reason; every server enum that mirrors a core enum is pinned by one test over the core `ALL` list, variant names and order alike. The proof of concept converts `ServerMessage` (internally tagged), the `[f32; N]` fields serialized with `serialize_with` (as `number[]`) and the `skip_serializing_if` fields (`#[ts(optional)]`) before the rest. `PROTOCOL_VERSION` becomes `v3alpha14`. |
| 10 | **Generated types.** One `v3-server` test (in `make check` through `rust-test-server`) renders every wire type and its dependencies with `export_to_string` from one listed root set, with 64-bit integers as `number`, and compares the declarations with the committed files under `frontend/src/types/generated/` with whitespace and member separators normalized; it fails on any added, removed, renamed or retyped type, field or variant, lists the differing files, and rewrites them only when `PETRI_WRITE_TS_TYPES=1` is set. `cargo test` without the flag never writes. The committed files are Biome-formatted and pass `npm run lint`. |
| 11 | **Frontend.** `frontend/src/types/` keeps only `generated/`, the catalog fixtures and their readers (`vote-sinks.ts`, `vote-sinks.json`), runtime constants and compositions over generated types; `genome.ts`, `creature-detail.ts`, `action-log.ts`'s types, `paint.ts`, `pattern.ts`, `protocol.ts`'s types, `config.ts`, `http.ts`'s response types, `errors.ts`'s types and `trace.ts` are replaced. `edgeGroupWeights.ts`'s `compassX`, `compassY`, `compose`, `memberPattern`, `sinkDirection`, `graphFullModel.ts`'s `learnerInitialized`, `PlasticEdgeList.tsx`'s `inheritsLearned`, `EdgeGroupList.tsx`'s `PAIR_BIRTH` and relative-direction arithmetic `inputRefUtils.ts`'s `parseWorldInputRef` and `mesh/meshAnalysis.ts`'s reachability traversal (with its `reachableNodeIds` and `unreachableNodeIds`; the mesh view reads each node's `reachable` from `brain`) are deleted; the composition tooltip prints the view's `scale`, `pattern`, `own` and `genomic`. `directionPairs.json` and `nearbyFields.json` stay as pinned catalog data for labels. |

## Architecture

Owners touched: the genome (a group-coordinates read API beside `pattern`,
`member_site` and `genomic_weight`), the runtime (`LearnedEdge.initialized`
and `.inherits`, resolved where `capture()` applies the same rules), `v3-server`
(the brain view, the learning payload, the sampler, paint and action-log wire
types, the generator test) and the frontend (generated types, deleted
rules). Debts paid: G17 (the genome's layout as the frontend's API, the
hand-copied `genome.ts`, the recomputed rules) and G22's paint half; G22's
action-log half gains its server-owned wire type here, and its derivation
from the applied result stays with T30.F06 (G18), so the plan commit edits
the G22 row's payer. Section 3's order is untouched: the server reads the
creature's `graph_runtime.learned()` and `cached_reachable_nodes` as today;
T30.F02 moves the runtime state out of the creature record, and whichever
lands later re-points that accessor. The exempt types' `ts` derive is a
build-time attribute under an optional feature no binary enables, which keeps
`v3-core` free of transport; `PatternParams` is derived where it lives when
this lands (T30.F04 moves it). No boundary changes; the edit to
`docs/architecture.md` is the G22 payer line.

## Telemetry

Not applicable: the feature changes nothing a run does; it adds no counter,
config value or mechanism switch, and the server's view construction is
observation of applied state.

## Verification Plan

**Question.** The brain view shows what the core applies and the generated
types match the server's; a moved gate pin or a differing generated file
would show it wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate` | The diff adds fields to the runtime's learned view and read APIs to the genome; the gate's unchanged pins, at seconds of cost, prove no trajectory moved, as T24.F03's byte-identical seeded output did. |
| Slow tests (`make rust-test-slow`) | `not run` | No mutation operator, draw, catalog or lab change. |

## Implementation Tasks

- [x] Core read APIs, tests first: `LearnedEdge.initialized` and `.inherits`
      (R3); the group birth-coordinates API (invariant 3); the `ts` feature with
      optional `ts-rs` and `cfg_attr` derives on `Direction`, `Position`,
      `NodeId`, `CreatureId` if it is ever rendered, and the config schema
      (`config/`, `PatternParams`, `PatternBounds`).
- [x] Proof of concept (invariant 9): `ts-rs` in `v3-server`, `ServerMessage`,
      the `serialize_with` arrays and the `skip_serializing_if` fields rendered,
      the generator test and `frontend/src/types/generated/` with the write flag
      (invariant 10), `TS_RS_LARGE_INT=number` set in `.cargo/config.toml`
      `[env]` so it holds at build and at test time.
- [x] The brain view and learning payload (invariants 2–6, R1–R3, R10) with
      server tests for the five preservation cases and the `exclude` tokens;
      `graph_weights` tests replaced.
- [x] The sampler types (invariant 7, R4), paint and action-log types
      (invariant 8, R5, R6), the remaining response, request and error types
      (R8, R9); `v3alpha14`.
- [x] Frontend: adopt the generated types, delete the copies and the recomputed
      rules (invariant 11), the inspector reads `brain` and `learning`, the
      paint store reads the generated types; component and store tests updated
      with view fixtures, one per preservation case.
- [x] `frontend/AGENTS.md` gains one line pointing at `src/types/generated/`
      and the write flag (edit under `.agents/skills/agents-md`).
- [x] At close: delete G17 from `docs/architecture.md` and reduce G22 to the
      action-log derivation T30.F06 pays; fill the readings file's
      review-items table.

## Verification

- [x] Core: `cargo test -p v3-core` -> 2305 lib tests and every
      integration target pass, the learned-view and coordinates tests among
      them, the viability target first; `ActionResult::ALL` and
      `OutcomeChannel::ALL` are checked complete and in discriminant order at
      compile time (mutation draws rules and channels from `HebbianRule::ALL`
      and `OutcomeChannel::ALL`);
      `cargo build -p v3-core --no-default-features` and
      `cargo build -p v3-core --features ts` both succeed; `cargo tree -e
      normal,build -i ts-rs` finds no `ts-rs` under `v3-server` or `v3-cli`.
- [x] Server: `cargo test -p v3-server` -> 53 unit, 106 `server`, 11
      `telemetry` tests pass (generator parity, the five preservation cases,
      the `exclude` tokens, paint and pattern round trips, sampler shapes, the
      `ActionType`/`ActionResult` pin over both core `ALL` lists, whose
      conversions have no wildcard arm, and the mirror parity test: every
      core-mirroring enum of `genome_wire.rs` and `brain.rs` has the core
      enum's serialized variant names, read from both derived `Deserialize`
      impls, and every value of a core `ALL` list or the fixed sink catalog
      serializes identically); test builds print no ts-rs serde
      warning (`no-serde-warnings`, the skipped attributes listed in the
      readings); deleting two generated files, then
      `PETRI_WRITE_TS_TYPES=1 cargo test -p v3-server --lib generated_types`
      and `npm run lint:fix` -> `git status` clean.
- [x] Frontend: `npm run lint` (0, four pre-existing warnings),
      `npm run test` (673 tests, among them a node the routes reach but the
      annotation marks unreachable classifying as unreachable), `npm run build` in `frontend/` -> 0;
      `grep -rn "compassX\|compassY\|learnerInitialized\|memberPattern\|parseWorldInputRef\|PAIR_BIRTH" frontend/src` -> none.
- [x] `cargo clippy --workspace --all-targets -- -D warnings` and
      `cargo fmt --all --check` -> 0.
- [ ] `make check` -> 0; the gate summary stored at
      `docs/progress/features/t30-f01.json` with the series entry pointing to
      it and no full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** This feature adds no mechanism
and reaches no creature. Expected compute cost: none; the gate's per-tick
cost and every pin are compared against the current epoch and are expected
identical, with no indicator moving in any direction. A moved pin is a defect
(track criterion 3), not a cost to accept; no re-pin is justified. The
`learning` payload costs the inspector one merge-walk per poll, as
`graph_weights` does today, and is not benchmarked.

**Measured verdict.** Gate, 2026-10-26, HEAD 1038e63d: no pin moved and
compute cost is zero, as predeclared. `make bench PROFILE=gate
FEATURE=t30-f01`: outer `make` exit 0, CLI exit 0 (the recipe passed and the
CLI wrote both files). `severe=false`; threshold verdict `ok` on all nine
indicators, each delta 0.000000% against
`t27-f03-giving-energy.json`; the whole `deterministic` object equals the
reference (final populations 622, 632, 645). Raw report 132637 bytes, sha256
`2833000d811388c62e37a976f63d63652f17e01276fd914e3e24de439f6de6e5`, at the
main checkout's ignored `.bench-artifacts/t30-f01/gate.json`; summary 27481
bytes. No benchmark-series entry was added (none exists for the reference
summary either).

- Summary of the declared profile: [gate](../../progress/features/t30-f01.json).
- Full readings: [`docs/progress/readings/t30-f01.md`](../../progress/readings/t30-f01.md).

## Success Criteria

- [x] The inspector renders the five preservation cases from `brain` and
      `learning` with every number from a core read API, and no frontend
      file computes a composition, pattern, coordinate, initialization or
      inheritance.
- [x] Every `v3-server` wire type derives `TS`, the committed generated types
      match the test's rendering, and the frontend builds against them with no
      hand-copied Rust type left under `frontend/src/types/`.
- [x] Paint and the pattern routes use server-owned tool and stats types;
      the sampler has no `serde_json::Value` field.
- [ ] The gate's pins are unchanged; `make check` passes; G17 is deleted from
      `docs/architecture.md` and G22 keeps only the derivation T30.F06 pays.

## Notes for AI Agents

- Decision: (user, 2026-10-25, track Notes) frontend wire types are
  generated with `ts-rs` in `v3-server`; a core struct inside a wire type is a
  compile error, the enforcement this feature relies on.
- Decision: (spec owner, 2026-10-26, under the track's delegation) the
  exempt types get a feature-gated `TS` derive in `v3-core`, never a server
  mirror; every other core type stays underived.
- Deferred: the action log entry's derivation from the applied result is
  T30.F06's (G18); this feature's entry mirrors today's fields.
