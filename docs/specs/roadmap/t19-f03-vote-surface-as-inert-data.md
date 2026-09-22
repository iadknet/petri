# T19.F03 — Vote Surface as Inert Data

**Status**: In Progress
**Last updated**: 2026-09-22
**Feature**: T19.F03
**Track**: [T19 — Mesh Action Selection and Live State](../../roadmaps/t19-mesh-action-selection-and-live-state.md)

## Goal

A motor pool is wired before it is ever driven, like a silent synapse. After
this feature the vote sinks (one `Eat`, eight `Move`, eight `Reproduce`, eight
`StealEnergy`, `Terminate`, `Decide`) and the per-kind parameter sinks exist in
every graph genome's fixed catalog, the `AddVote` opcode exists in the ISA and
the interpreter, the mesh accumulates a vote vector (each node's latest
contribution, summed over nodes) and per-kind commit counters, and traces, the
server sample protocol, the frontend types, and a read-only inspector block
carry them — while nothing reads them and no mutation draws them. Applied
behavior, RNG consumption, and every work counter are identical to T19.F02.

## Non-Goals

- Reading the votes: no commit rule, pass loop, bar, `Decide` or `Terminate`
  semantics, tie rule, or parameter read (T19.F04).
- Lifting the two draw exclusions, deleting the push, pop, execute, bank, and
  gate machinery, re-expressing founders, or counting the new sinks in
  `genome_size` (T19.F04).
- Vote or counter inputs (T19.F05); inspector redesign (T19.F06).
- A VM opcode for the parameter surface: `WriteWorldActionMeta` stays the VM's
  parameter path until T19.F04 decides.
- One `Eat` sink per ordinary food type (T17.F04's bank); config changes;
  migration of stored genomes; re-basing any existing readings series.

## Inputs and Invariants

Sources of truth: the track row and its "Scope, T19.F03", "Decomposition
rules", "Epochs", and natural-analog notes; the
[mesh action-selection review](../../strategy/mesh-action-selection-review-2026-09-20.md)
Sections 2.1, 2.3 (the `V` and `C` lines and "what resets"), 2.6 (exhaustion),
and 5; T19.F02's invariants 3 to 6 (live state, exhaustion matrix); the code
named in each invariant. Options settled from local evidence, no external
research (an inert-data step of a mechanism the note already grounded): one
`Eat` sink with the type as a parameter (food types are a runtime
`world.food.types` `Vec`, so a per-type sink has no compile-time count;
T17.F04 owns the bank) over a fixed maximum; two parameter sinks per kind
mirroring today's `[f32; 2]` decode buffer over three hand-picked ones;
draw exclusion by sink kind over an index bound (survives any catalog length);
a dispatch-local VM vote committed on success over immediate accumulation
(matches the exhaustion matrix). Analog: a silent synapse, wired but not yet
transmitting; the surface reaches creatures through the body, never a sensor.

1. **Sink identity.** A shared, serde-derived catalog in `creature::genome`
   with a fixed index order and no runtime state:

| Item | Definition |
| --- | --- |
| `VoteKind` | `Eat`, `Move`, `Reproduce`, `StealEnergy`; index `0..4`; `VOTE_KIND_COUNT = 4` |
| `VoteSink` | `Eat`, `Move(d)`, `Reproduce(d)`, `StealEnergy(d)` for `d` in `0..8` (`Direction::ALL` index), `Terminate`, `Decide`; index order as listed, `Eat` = 0, `Move(d)` = `1 + d`, `Reproduce(d)` = `9 + d`, `StealEnergy(d)` = `17 + d`, `Terminate` = 25, `Decide` = 26; `VOTE_SINK_COUNT = 27` |
| `VoteVector` | `[f32; VOTE_SINK_COUNT]` |
| `VoteSink::kind()` | `Some(VoteKind)` for the four kinds' sinks, `None` for `Terminate` and `Decide` |
| `VoteSink::from_index` | `None` at or above 27 (the soft default of every index read) |

2. **Graph catalog.** `OutputSinkKind` gains `ActionVote(VoteSink)` and
   `ActionParam(VoteKind, u8)` (`u8` in `0..2`). `new_with_fixed_outputs`
   appends, after today's 64: the 27 `ActionVote` sinks in `VoteSink` index
   order (indices 64..91), then `ActionParam` kind-major (`Eat` 0, `Eat` 1,
   `Move` 0, …; indices 91..99). `FIXED_SINK_COUNT` is 99 and the
   `const` assertion moves with it; the catalog test at `cgp.rs` asserts every
   index. Sink kinds stay structurally immutable; only edges are evolvable.
3. **Draw exclusions (the predeclaration rests on these).**
   `pick_random_surface` skips a sink whose kind is `ActionVote` or
   `ActionParam`, so its surface list, length, and order are identical to
   today's for every genome and `gen_range` draws the same value. Every other
   edge operator enumerates existing edges (`edge_sites`) and is unaffected
   because no vote or parameter sink has an edge. `random_vm_instruction`
   keeps `gen_range(0u8..42)` and no arm yields `AddVote`; no other
   constructor of `VmInstruction` in `mutation/` produces it (the implementer
   greps and the reviewer proves it). Operand-nudge and classification arms
   for `AddVote` exist so the match stays exhaustive (nudge `sink` or `src`
   by one like `WriteDirectionBid`), and are unreachable until T19.F04.
4. **`AddVote { sink: u8, src: u8 }`.** Opcode 42 in the ISA table, base cost
   `0.14` (the write-family cost of `WriteDirectionBid`), one `vm_steps` unit
   like every executed opcode. Semantics: `contribution[sink] += regs[src]`
   on the dispatch-local vector when `sink < 27`; an invalid sink writes
   nothing and still costs. Classification: no destination register, reads
   `src`, an output instruction (`vm_is_output_instruction`), mesh write class
   `Action`, no memory effect for `companions`.
5. **Contribution rule (note 2.1).** A node's contribution vector is set by
   its latest successful visit and replaces that node's previous one; the
   mesh vote vector is the sanitized sum of the latest contributions of every
   node visited this tick. A VM dispatch starts from zeros, sums its own
   `AddVote`s, and commits whenever the dispatch ends other than by energy
   exhaustion (the boundary at which `commit_slots!` fires); a dispatch with
   no `AddVote` commits zeros. A graph visit's contribution is
   the effects pass's sanitized weighted sum per wired `ActionVote` sink
   (0 for an unwired one), applied only when the visit commits (T19.F02's
   exhaustion matrix: graph effects are not applied on exhaustion); its
   `GraphOutputSinkTrace` row reports `applied` true and the sanitized value
   when wired. Each
   entry is `sanitize_f32`'d at commit and the sum is sanitized again.
   Contributions are signed; nothing clamps them.
6. **Parameter surface.** `MeshSideOutputs.action_params: [[f32; 2];
   VOTE_KIND_COUNT]`, zero at evaluation start; a wired `ActionParam(kind, i)`
   sink overwrites `action_params[kind][i]` with its sanitized weighted sum
   in the effects pass (an unwired one leaves it), last visit wins. Nothing
   reads it.
7. **Runtime state.** `MeshSideOutputs` gains `votes: VoteVector`, the
   per-node latest contributions (keyed by genome node index; the
   implementer chooses the container, the sum order is deterministic),
   `commit_counts: [u32; VOTE_KIND_COUNT]` (always zero here: nothing
   commits), and `action_params`; all fresh per evaluation, which is "cleared
   at tick start" while a tick is one pass. `MeshOutput` carries `votes` and
   `commit_counts` for the trace and nothing else reads them.
   `WorkCounters` is unchanged; a vote sink's effects-pass row counts nothing.
8. **Traces and transport.** `TickTrace` gains `votes: VoteVector` and
   `commit_counts: [u32; 4]`; `MeshHopTrace` gains `vote_contribution:
   VoteVector` (the contribution the hop committed; zeros when it did not
   commit). `GraphOutputSinkTrace` rows exist for the new sinks by
   construction (indexed 1:1 with the catalog). The server payloads
   (`sample_protocol.rs`, `sample_assembler.rs`) and `frontend/src/types`
   (`genome.ts` sink kinds and `AddVote`, `trace.ts`) mirror the fields;
   `PROTOCOL_VERSION` stays `v3alpha2` (additive fields, not a breaking
   change). The inspector labels the new sink kinds and the opcode in its
   formatters and shows one read-only block per sampled tick listing the
   non-zero vote entries and the four commit counts; no other UI changes.
9. **Unchanged by construction.** `genome_size` and `cgp_functional_complexity`
   count wired sinks only, so `FOUNDER_GENOME_SIZE_UNITS` stays 111; the
   founder graph gains 35 unwired sinks and its tests are unchanged;
   `founder_only_trajectory_digest_is_pinned` stays
   `63498f8d36346079f8827c382e2978510357b374ca37759af857afa263f2d0be`; every
   `config_digest` is unchanged (no config field moves). Wired-only loops
   (`cgp_analysis`, `companions`, `cgp_mesh_annotations`, `meshSemantics.ts`)
   gain a match arm (`ActionVote`/`ActionParam` are write class `Action`)
   that no genome reaches.
10. **Changed by construction and predeclared.** Serialized genomes grow by
    35 catalog entries, so the Debug-hashed fingerprint
    `legacy_default_short_run_identity` (`tests/baseline_worlds.rs`) moves and
    is re-pinned after reproduction; any other pin that hashes a genome's
    Debug or serde form moves the same way and is listed in the readings
    file; a pin that hashes positions, energy, or ages must not move. A
    genome stored with 64 sinks deserializes and runs with no vote sinks; no
    migration or shim. Per-visit allocation and wall time rise because the
    effects pass iterates and traces every sink (99 rows instead of 64).
11. **Docs.** `v3-graph-backend-spec.md` Sections 5 and 15 (catalog of 99;
    the stale "12 CustomOutput … 52 sinks" corrected to the code's 24 and
    64), `v3-vm-isa-spec.md` Sections 2 and 6 (43 opcodes, 42 drawable
    until T19.F04), `v3-mutation-spec.md` VM-domain and `AddGraphEdge`
    lines (the two exclusions), `v3-mesh-execution-spec.md` Section 1
    (`MeshOutput` fields), `v3-server-api-protocol-spec.md` Section 4 (the
    sample payload bullets), `v3-genome-spec.md` catalog line.

## Implementation Tasks

- [x] Pin the guard first: add a mutation-on seeded short-run trajectory
      digest test (positions, energy bits, ages; no genome bytes) and record
      its value on the base commit `07086ed0` before any production change.
- [x] `VoteKind`, `VoteSink`, `VoteVector` catalog with index tests;
      `OutputSinkKind` variants, `new_with_fixed_outputs`, `FIXED_SINK_COUNT`
      99, catalog test.
- [x] `pick_random_surface` kind exclusion; `AddVote` in `VmInstruction`,
      `random_vm_instruction` unchanged, nudge and classification arms;
      exclusion tests (invariant 3).
- [x] `MeshSideOutputs` fields, VM dispatch-local vote and commit, effects
      pass arms for `ActionVote` and `ActionParam`, sum rule, `MeshOutput`.
- [x] Trace types, server payloads and assembler (with its fixture tests),
      frontend types and the five trace fixtures, formatter labels, the
      read-only inspector block and its test.
- [x] Re-pin `legacy_default_short_run_identity` after reproducing it twice;
      list every moved pin in the readings file.
- [x] Reference docs of invariant 11.

## Verification

- [x] Guard digest test unchanged between `07086ed0` and the feature commit
      -> value and both runs in
      [`docs/progress/readings/t19-f03.md`](../../progress/readings/t19-f03.md).
- [x] Focused tests: catalog indices and count; `pick_random_surface` over
      4,096 seeds never returns a vote or parameter sink and, on the founder
      graph, returns for eight fixed seeds the surfaces recorded on
      `07086ed0`; `random_vm_instruction` over 4,096 seeds
      never yields `AddVote` and the 42 drawable discriminants are all reached;
      per-node-latest replacement on a revisit; signed summation across two
      nodes; sanitization of a non-finite contribution; an exhausted VM dispatch
      and an exhausted graph visit leave the vector untouched; an invalid
      `AddVote.sink` is a costed no-op; parameter overwrite; three-mode parity
      (production, observed, traced) on a voting genome -> test names and
      counts in the readings file.
- [x] Nothing reads the surface: a grep for `.votes`, `commit_counts`,
      `action_params`, `vote_contribution`, `ActionVote`, `ActionParam`,
      `AddVote`, and `is_vote_surface` over `crates/` and `frontend/src` lists
      only writers, traces, formatters, and tests -> the grep output in the
      readings file; the reviewer proves it.
- [x] `cargo test -p v3-core --test viability` -> 28 passed;
      `founder_only_trajectory_digest_is_pinned` unchanged;
      `cargo test -p v3-core --test vm_all_opcodes_e2e` updated for 43
      variants; `make check` exit 0.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred, listed
      here.
- [ ] Benchmark summaries stored at `docs/progress/features/t19-f03-vote-surface-as-inert-data.json`
      and `-goal.json`, local raw hash/byte count and verification time
      checked, series entries point to the summaries, no new full report
      staged, and no epoch re-pinned.

| Command | Result |
| --- | --- |
| `cargo test -p v3-core --test viability` | 28 passed; `founder_only_trajectory_digest_is_pinned` and `FOUNDER_GENOME_SIZE_UNITS` 111 unchanged |
| `cargo test -p v3-core --test applied_trajectory` | 2 passed; guard digest `99ef9a14…` identical before and after; re-pinned accounting digest `09b9e53c…` holds |
| `cargo test -p v3-core --test baseline_worlds` | 19 passed, 1 ignored; re-pinned `legacy_default_short_run_identity` `5811834416729882562` holds |
| `cargo test -p v3-core --test vm_all_opcodes_e2e` | 1 passed, 43 discriminants |
| `cargo test --workspace` | 24 targets pass |
| `npm run test` (frontend) | 329 passed, 62 files |
| `make check` | exit 0 |
| `make roadmap-check` | validation passed |

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: a silent
synapse, a motor pool wired before it is driven; the mechanism reaches
creatures through the body (the genome's sink catalog and ISA), never a
sensor. Predeclared compute: 35 more effects-pass rows per graph visit and a
27-entry vector plus a small contribution container per evaluation; no new
hops, steps, or writes.

References: gate epoch and previous closure are both
`t19-f02-live-internal-state-and-legal-cycles.json`; goal-worlds epoch and
previous closure are both
`t19-f02-live-internal-state-and-legal-cycles-goal.json` (both re-pinned at
T19.F02's closure). Standing thresholds: counters flag at 10%, severe at 50%;
wall flags at 25%, severe at 100%. **This feature must not re-pin either
epoch** (track "Epochs"): it predeclares applied behavior and counters
unchanged, and the goal run is one run.

| Reading | Predeclaration |
| --- | --- |
| Gate and goal work counters (`mesh_hops`, `vm_steps`, `graph_relax_iters`, `plasticity_updates`, `actions_applied`, `births`, `pass_cap_hits`) | Identical to T19.F02's summaries at every case, 0.000000%; any non-zero difference means a draw or a read leaked and is an escalation, not a tolerated flag |
| Goal persistence (final, minimum, plateau), lineage diversity, recruitment paths, memory sensitivity, energy flows (`mesh_ramp`, `priority_bid`) | Identical to T19.F02's goal summary |
| `config_digest`, gate and goal | Unchanged; `inputs_changed` false |
| Founder digest | Unchanged (`63498f8d…`) |
| Debug-hashed trajectory fingerprint (`legacy_default_short_run_identity`) | Moves by construction (catalog grew); re-pinned, not a behavior change |
| Wall time, both profiles | Up; if the host matches T19.F02's (`MacBookPro.lan`) the comparison is live (otherwise `null`, as at T19.F02): a flag (≥25%) is the predeclared cost of the wider effects pass and is tolerated; a severe (≥100%) does not fit and is investigated against the per-visit cost before anything is presented |

**Measured verdict.** One line per profile: CLI and observed outer-process
exit statuses with their sources, the `severe` flag, whether any threshold
was crossed, and whether the epoch was re-pinned (it must not be).

- Summaries: [gate](../../progress/features/t19-f03-vote-surface-as-inert-data.json),
  [goal](../../progress/features/t19-f03-vote-surface-as-inert-data-goal.json).
- Full readings: [`docs/progress/readings/t19-f03.md`](../../progress/readings/t19-f03.md).

## Success Criteria

- [x] Every graph genome built by `new_with_fixed_outputs` has the 99-sink
      catalog in the fixed order; `AddVote` exists at opcode 42; the vote
      vector, per-kind counters, and parameter surface exist in
      `MeshSideOutputs`, `MeshOutput`, the traces, the server payloads, the
      frontend types, and a read-only inspector block.
- [x] No mutation draws a vote or parameter sink or an `AddVote`, and no
      executor, input, or reading consumes the surface (grep and reviewer
      proof).
- [ ] Guard digest, founder digest, `FOUNDER_GENOME_SIZE_UNITS` 111, and
      every `config_digest` unchanged; gate and goal counters identical to
      T19.F02; the moved Debug fingerprint re-pinned and listed.
- [ ] Reference docs of invariant 11 rewritten; mutation gate run with every
      survivor resolved; gate and goal run with no epoch re-pinned.

## Notes for AI Agents

- Decision: one `Eat` vote sink; the food type stays a parameter
  (`ActionParam(Eat, 0)`) until T17.F04 makes eating a per-type bank on this
  surface.
- Decision: the `VoteSink` index order, the catalog positions 64..99, and
  opcode 42 are fixed here; T19.F04 lifts the two draw exclusions without
  renumbering, and T19.F05's `ActionVotes` input reads the same index order.
- Decision: a genome stored with the 64-sink catalog loads and runs with no
  vote sinks; no migration or shim is added.
