# T11.F25 — Meaningful Action-Parameter Targets

**Status**: In Progress
**Last updated**: 2026-09-24
**Feature**: T11.F25
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

Every fresh mutation draw that creates a Graph or VM action-parameter
connection targets a parameter field that the body's current action decoder
(`decode_commit`) reads: `Eat[0]` (food type), `Reproduce[1]` (transfer
fraction) or `StealEnergy[1]` (amount). The set of decoded fields is one
explicit catalog, next to the vote catalog, and a test ties it to the decoder.
Storage stays eight slots, so existing genomes, runtime writes and pruning are
unchanged. This repairs finding M3 of the
[post-T19 mutation audit](../../strategy/post-t19-mutation-audit-2026-09-23.md).

Natural analog: neuromuscular synapse formation. A growing motor axon forms a
terminal where the muscle presents a receptor cluster (the agrin–MuSK
acetylcholine-receptor prepattern), so a new motor connection lands on muscle
that can contract. This is a correctness repair to the mutation operators, and
it reaches creatures only through the parameter wiring their offspring
inherit, with no sensor.

## Non-Goals

- Re-encoding or removing the five undecoded storage fields: the 8
  `ActionParam` sinks, `FIXED_SINK_COUNT` 99, `WriteActionParam.slot_idx` in
  `0..8`, its runtime addressing and cost (track note: no storage rewrite).
- Mutation supply, operator weights, the opcode draw (still uniform over 39),
  or any config value.
- Existing structure. `RemoveGraphEdge`, `AlterGraphEdgeWeight`,
  `RetargetGraphEdge`, `GraphRawFieldMutation`, the edge split, and VM
  delete, replace and copy operators keep reaching edges and instructions on
  undecoded fields, so old inactive structure can still be pruned or
  faithfully copied.
- `VmInstructionRawFieldMutation`. It stays a one-unit move of one encoded
  operand for tolerant decoders (`v3-mutation-spec.md`). It moves an
  existing write rather than drawing a fresh target. From a decoded slot, it
  can remove that write's contribution to a decoded parameter. From an
  undecoded slot, it can land on a decoded one.
- `ReadActionQueueParam.param_slot`. It reads the queue's own `param` layout
  and is not a parameter target (audit M6).
- The structural census and mesh annotations. A wired undecoded parameter sink
  or write keeps its `Action` class, and causal-use measurement belongs to
  T20.
- Other sink families (`CustomOutput`, `WriteSlot`, `ClearSlot`,
  `RouterGate`, vote sinks), audit findings M1, M2, M4, M5 and M6, and new
  telemetry fields or instrument version strings.

## Inputs and Invariants

- Sources: the T11.F25 track row, the track note "Post-T19 correctness
  prerequisites, 2026-09-23" (derive fresh Graph/VM parameter-target draws
  from the active decoder catalog, check every creation path, preserve
  pruning, no storage rewrite and no supply change), audit M3 and its
  recommendation, and the T19.F04 spec's commit decode (its invariant 3:
  undecoded slots stay unread until T17.F03/F04 name them) and surfaces
  (its invariant 8: both draw exclusions lifted, slot draw `0..8`). The
  owning track row lists the dependencies.
- Current code (verified 2026-09-24 at `df35975f`):
  - Decoder: `runtime/action_decode.rs` `decode_commit` reads `params[0]` for
    `Eat`, nothing for `Move`, and `params[1]` for `Reproduce` and
    `StealEnergy`. No other code reads the parameter surface: queue entries
    hold decoded `WorldAction`s, so an undecoded field is write-only.
  - Catalog: `creature/genome/vote.rs` holds `VoteKind`, `VoteSink`, and
    `VOTE_PARAM_SLOTS` = 2. `creature/genome/cgp.rs`
    `new_with_fixed_outputs` appends the 8 `ActionParam(kind, slot)` sinks
    kind-major after the 27 vote sinks.
  - Graph creation path: `AddGraphEdge` → `add_edge` →
    `pick_random_surface` (`mutation/graph/operators.rs`) makes one
    `gen_range` over compute nodes plus all of the def's sinks. With `c`
    compute nodes, `5/(c + 99)` of its draws land on an undecoded parameter
    sink. `can_add_edge` is true when any compute node or sink exists. No
    other graph operator creates a sink edge. Split, copy and bundle operators
    keep or duplicate existing destinations, and topology `AddRouteTarget`
    wires only `RouterGate` sinks.
  - VM creation path: `random_vm_instruction` (`mutation/vm/operators.rs`)
    draws opcode 25 as `WriteActionParam { slot_idx: gen_range(0u8..8), .. }`.
    Its callers are `apply_instruction_mutation`'s empty-program seed, insert,
    replace, and single-instruction replace. `mutate_one_instruction_field`
    is the raw-field nudge (Non-Goals). No motif or topology path emits
    `WriteActionParam`.
  - The founder (a 97-unit vote graph with no VM node) wires only
    `ActionParam(Reproduce, 1)` and leaves `Eat[0]` unwired.
- Research decision, 2026-09-24:

| Option | Evidence and fit | Disposition |
| --- | --- | --- |
| Explicit decoded-parameter catalog; derive the graph surface draw and the VM slot draw from it | Audit M3's recommendation. Local, two draw sites, storage untouched. This follows CGP's distinction between active outputs and inactive structure ([CGP-Library](https://www.cgplibrary.co.uk/files2/CartesianGeneticProgramming-txt.html)) | Adopted |
| Remove the five undecoded fields (re-encode sinks and `slot_idx`) | Changes genome encoding, the ISA, `FIXED_SINK_COUNT` and stored genomes. The track note forbids a storage rewrite | Rejected |
| Rejection sampling: redraw when an undecoded target is drawn | Same distribution as filtering but consumes a variable amount of RNG, which makes the code harder to reason about | Rejected |
| Raise supply or weights so decoded targets are hit more often | The track note and audit rule out a supply change, and it does not fix the mapping | Rejected |

Fixed design:

| Decision | Value |
| --- | --- |
| Catalog | One `pub` constant in `creature/genome/vote.rs` lists the decoded `(VoteKind, slot)` pairs in kind-major order: `(Eat, 0)`, `(Reproduce, 1)`, `(StealEnergy, 1)`. The implementer names it. A flat VM index is `kind.index() * VOTE_PARAM_SLOTS + slot`, which gives 0, 5 and 7. |
| Decoder agreement | A test covers every `(kind, slot)` in `VoteKind::ALL × 0..VOTE_PARAM_SLOTS`. Some pair of finite values in that slot alone, other slots fixed, changes `decode_commit`'s action for the kind's sinks if and only if the pair is in the catalog. A later feature that decodes another slot (T17.F03/F04) must extend the catalog and the decoder together. |
| Graph draw | `pick_random_surface` makes one uniform `gen_range` over the def's compute nodes followed by its `output_sinks` in their existing vector order, skipping each `ActionParam(kind, slot)` sink not in the catalog. A chosen sink keeps its original vector index in `EdgeSurface::SinkInput`. On the full fixed catalog that is `c + 94` surfaces. When nothing is drawable, it returns `None` without consuming RNG, and `can_add_edge` is true exactly when the draw has a candidate. The source and weight draws that follow are unchanged. |
| VM draw | Opcode 25 draws `slot_idx` uniformly over the catalog's flat indices in catalog order with one `gen_range`. `src` and every other opcode's operands are unchanged. |
| Runtime | Unchanged. A write to an undecoded slot or an unwired sink behaves as today. |
| Eligibility | `AddGraphEdge` eligibility changes only for a graph with no compute node whose sinks are all undecoded parameter sinks. No graph on the fixed catalog has that shape, so it arises only in partial test defs. |
| Determinism and equivalence | Draws remain pure functions of the seeded RNG. Consider a birth from the same parent, context and RNG that selects no `AddGraphEdge` operator, meets no changed `AddGraphEdge` eligibility, and makes no fresh opcode-25 draw. That birth is byte-identical to `df35975f` in genome, `MutationSummary` and RNG stream. Every pinned value that changes must be attributed to a birth that does one of those things, or to the trajectory downstream of such a birth. |
| Cost | Negligible: the graph draw filters at most 99 sinks per `AddGraphEdge` event. |

## Implementation Tasks

- [x] Write failing tests first: the catalog–decoder agreement test, and
      draw tests showing that fresh Graph and VM parameter targets are only
      decoded fields while each decoded field stays drawable. Also show that
      every non-parameter sink and every compute node stays drawable, and that
      `can_add_edge` agrees with the draw on partial and mixed sink lists,
      including a def whose only sinks are undecoded parameters, where the
      draw consumes no RNG.
- [x] Add the catalog (`DECODED_ACTION_PARAMS`, flat
      `DECODED_ACTION_PARAM_FLAT_SLOTS`) and derive both draws from it (Fixed design). Update
      the doc comments that say "every sink" and the test
      `pick_random_surface_draws_uniformly_over_compute_nodes_and_every_sink`
      to the new candidate list, with the uniform-draw assertion kept.
- [x] Add a regression test showing that existing structure on undecoded fields
      is still reachable: `RemoveGraphEdge` can remove an edge on an undecoded
      `ActionParam` sink, and VM delete can remove a `WriteActionParam` to an
      undecoded slot.
- [x] Update `docs/reference/v3-mutation-spec.md` (the VM fresh-instruction
      draw near "`WriteActionParam { slot_idx in 0..8, src }`" and
      `AddGraphEdge`'s "all 99 sinks"), `v3-graph-backend-spec.md` (the
      `pick_random_surface` sentence), and `v3-vm-isa-spec.md` (the slot-index
      list: storage `0..8` stays, and mutation draws only decoded slots).
- [x] Re-pin any trajectory, replay, drift or recruitment-paths test value
      that changes. List each old and new value in the readings with the
      attributing draw. No predicate may be weakened.
- [ ] Record gate and goal readings as Performance requires.

## Verification

- [x] `cargo test -p v3-core --test viability` first, then
      `cargo test -p v3-core --lib mutation` and the new tests, with the red
      run and green run in [readings](../../progress/readings/t11-f25.md).
- [ ] `make check` exits 0 in the worktree.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred.
- [x] Gate and goal summaries are stored at
      `docs/progress/features/t11-f25-meaningful-action-parameter-targets.json`
      and `...-goal.json`. Local raw hash, byte count and verification time are
      checked, series entries point to the summaries, and no new full report
      is staged. Gate series `epoch_baseline` re-pinned per the user decision
      below; goal series `epoch_baseline` left unchanged (goal severe result
      is unresolved, see Measured verdict).

## Performance and Goal Impact

**Predeclaration — written before the run.** The natural analog and the path
to creatures are in the Goal. The feature adds no environmental pressure, so
the three-world rule adds nothing beyond the ordinary goal run. Expected
compute cost: none measurable. Trajectories diverge from the first birth that
makes an `AddGraphEdge` surface draw or a fresh opcode-25 draw. The removed
sinks shift the index mapping of every later surface, so most such draws land
elsewhere, not only those that used to hit an undecoded field. Every counter
can therefore move through the trajectory.

References. Gate: epoch `t19-f04-vote-based-action-selection.json`, latest
closure `t11-f24-stable-parent-membership-during-mutation.json`. Goal
(goal-worlds-v1): epoch `t19-f04-vote-based-action-selection-goal.json`,
latest closure `t11-f24-stable-parent-membership-during-mutation-goal.json`.
Standard thresholds apply: +10%/+50% for work and +25%/+100% for wall time.
No epoch re-pin is budgeted. A severe work counter on either profile, or an
extinction in any goal world, is a user decision under the blocker rule, and
wall-time moves are flag-only. The observation caps are unchanged.

| Indicator | Predeclared direction |
| --- | --- |
| `config_digest`, founder digest, `FOUNDER_GENOME_SIZE_UNITS` | Unchanged |
| Founder half: `mesh_execution`, `steering`, `reachable_node_count` (no mutation) | Unchanged |
| Founder half `operator_rows` (per-operator seeds, `neighborhood/operators.rs` `per_operator_rows`) | Only the `AddGraphEdge` row moves; no sign. The founder has no VM node, so no other row can move |
| Founder half `births`; goal `founder_changed_per_all_births`, `founder_dead_per_all_births` per world | Move only through births that attempt `AddGraphEdge` or make a fresh opcode-25 draw; no sign |
| Gate and goal work counters (all bench `COUNTER_NAMES`: `mesh_hops`, `vm_steps`, `graph_relax_iters`, `plasticity_updates`, `actions_applied`, `births`, `pass_cap_hits`, `passes`, `decided_passes`) | No direction; standard thresholds |
| Gate per-seed `final_population`; goal `final_population`, `plateau_population`, births per creature-tick per world | Move; no sign; an extinction is a blocker |
| Goal evolved half, drift depth rows, recruitment paths, lineage diversity, memory and temporal memory sensitivity, reachable structure size, structural companions, `mutation_supply` operator mix | No direction; recorded. Instrument versions stay, because the production draw changed and not the instrument |

No goal indicator counts parameter-target decoding. The draw tests are the
evidence that the Goal holds.

**Measured verdict.**

Gate: `make bench PROFILE=gate FEATURE=t11-f25-meaningful-action-parameter-targets`
(`v3-cli` exit 3, severe). Against the T19.F04 epoch, `pass_cap_hits` is
0.001364 per creature-tick vs 0.000752 (+81.4%, severe); all other eight
`COUNTER_NAMES` are `ok`. Against the latest closure, T11.F24's 0.001016, it
is +34.25% (flag); all others `ok`. The gate series test
`gate_profile_has_no_severe_regression_against_series_references` failed
before the re-pin below and now passes (`cargo test -p v3-cli --test bench
gate_profile_has_no_severe_regression_against_series_references`: `ok`,
1 passed). This is the rare-event `AddGraphEdge`-filter divergence
predeclared above.

User decision (verbatim, 2026-09-24, in reply to the severe gate
`pass_cap_hits` 0.001364 vs the T19.F04 epoch 0.000752, +81.4%): "accept and
re-pin". The gate series `epoch_baseline` in
`docs/progress/benchmark-series.json` is re-pinned from
`docs/progress/features/t19-f04-vote-based-action-selection.json` to
`docs/progress/features/t11-f25-meaningful-action-parameter-targets.json`.

Goal: `make bench PROFILE=goal FEATURE=t11-f25-meaningful-action-parameter-targets`
(`v3-cli` exit 3, severe). Against the T19.F04 epoch (goal-worlds-v1):
`vm_steps` 2.403726 vs 1.024805 (+134.55%, severe), `decided_passes`
0.000148 vs 0.000092 (+60.87%, severe), `mesh_hops` +10.68% (flag),
`plasticity_updates` +42.41% (flag); `graph_relax_iters`, `actions_applied`,
`births`, `pass_cap_hits`, `passes` are `ok`. Against the latest closure,
T11.F24: `vm_steps` +202.71% (severe), `decided_passes` +34.55% (severe),
`mesh_hops` +26.74%, `graph_relax_iters` +20.19%, `plasticity_updates`
+21.18%, `actions_applied` +26.34%, `passes` +16.51%, `decided_passes` flag
(all flag); `births` and `pass_cap_hits` `ok`. No extinction: `final_population`
4877/3221/1219 across the three goal-worlds-v1 seeds, `extinction_tick` null
on all three. `neighborhood_evolved_wall_clock_ms_total` 5317.7 ms, well
under the 180 s cap; `neighborhood_founder_wall_clock_ms` 280.1 ms, under the
10 s cap. This `vm_steps`/`decided_passes` severity is not covered by the
Predeclaration table above (which named only `AddGraphEdge`-filter and
opcode-25-draw trajectory shifts with no predicted sign) and is not covered
by the user decision above, which addressed only the gate `pass_cap_hits`
counter. It is reported to the orchestrator as an unresolved, unexpected
result; the goal series `epoch_baseline` is left unchanged pending a
separate user decision.

- Summaries: [gate](../../progress/features/t11-f25-meaningful-action-parameter-targets.json),
  [goal](../../progress/features/t11-f25-meaningful-action-parameter-targets-goal.json).
- Full readings: [`docs/progress/readings/t11-f25.md`](../../progress/readings/t11-f25.md).

## Success Criteria

- [ ] Every fresh Graph and VM parameter-target draw lands on a field in the
      decoded catalog, each decoded field stays drawable, and the catalog is
      tested against `decode_commit`.
- [ ] Storage, runtime, pruning of existing structure, and births without a
      changed draw are unchanged, and every re-pinned value is attributed.
- [ ] `make check` and the mutation gate pass with every survivor resolved,
      and the gate and goal summaries are stored and read against the
      predeclaration.

## Notes for AI Agents

- Decision: Fable credits are exhausted, so this feature's spec owner runs on Opus (high-effort intent) instead of Fable 5.1 `high`, is resumed with `SendMessage`, and no Fable advisor is used anywhere in the run (workflow launch step 3 is skipped).
