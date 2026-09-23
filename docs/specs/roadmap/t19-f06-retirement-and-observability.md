# T19.F06 — Retirement and Observability

**Status**: In Progress
**Last updated**: 2026-09-23
**Feature**: T19.F06
**Track**: [T19 — Mesh Action Selection and Live State](../../roadmaps/t19-mesh-action-selection-and-live-state.md)

## Goal

Removal of a rule with no natural analog: one execution model remains
wherever a reader looks. `mutation.action_queue_cap` is retired and the
`ActionQueue` input width is a constant (four slots, 12 sub-values). The push,
pop, execute, bank, and gate names are gone from `crates/`, `frontend/src`, and
`docs/reference`, and a check keeps them gone. Records built on the retired bank
are marked historical. Route variation is reported both within a snapshot
(state-driven) and across snapshots (input-driven). The inspector shows a tick
as passes: hops grouped by pass, votes, bars, effective votes, `Terminate` and
`Decide`, and the decision-state inputs each dispatch read. Applied behavior
and every work counter are identical; the config digest changes.

## Non-Goals

- Widening the `ActionQueue` input past four slots (a later remap); any change
  to a draw, the pass loop, a founder, or a cost; any new config field or input.
- Rewriting closed specs, readings files, or stored summaries: history keeps
  its names, and the marking is an index.
- Consolidating the hand-mirrored reason enums (T19.F04 review P3); version
  bumps of `recruitment-paths-v1`, `steering-v1`, `mesh-execution-v2`, `drift-depth-v3`, or
  the summary schema.
- Frontend work outside the inspector's sampler and action-selection views;
  the benchmark dashboard (`docs/progress/index.html`); T11.F10's protocol;
  the T18 re-plan.

## Inputs and Invariants

Sources of truth: the track row, criterion 2, and the "Scope, T19.F06",
"Epochs", "Contract text", and "Readings that become historical" notes; the
[mesh action-selection review](../../strategy/mesh-action-selection-review-2026-09-20.md)
Section 3 retirement table and the Section 5 F06 row; T19.F02's deferral of the
legacy convergence trace fields to this feature (its Notes); T19.F04 invariants
4, 8, 10, and 11 and its deferred P3 on the traced diagnostic route
(`runtime/mesh.rs:429`); T19.F05 invariants 1 to 3. No external research
applies: every choice is internal to the codebase.

| Option | Disposition |
| --- | --- |
| `ActionQueue` width: a constant 12, keep the cap, or tie it to `max_actions_per_turn` | Constant (track scope); tying it to `max_actions_per_turn` makes it 30 at the default 10, a draw remap |
| Enforcement: a scan in `scripts/policy-check`, a Rust test, or a CI-only job | `policy-check`: it runs in both `make check` and `make check-docs` and sees docs and frontend, which a Rust test does not |
| Decision-state values in the inspector: carried on the trace or re-derived in TypeScript | Carried: a vote vector is an `f32` sum the frontend would recompute in `f64`, so a derived value could differ from what the node read |
| Historical marking: an index in `docs/progress/readings/README.md`, a banner in each closed file, or version bumps | Index: the README says readings files are not rewritten after closure, JSON summaries cannot carry a banner, and a version bump moves every report |
| Route split: add within-snapshot flags beside the existing ones, or rename | Add: `route_varies_with_input` already compares per-node route sets across the 48 independently reset snapshots (`neighborhood/mesh_execution.rs:130`), so it keeps its name, value, and series |
| Legacy `converged`, `stable_passes_count` trace fields | Deleted: constant false and zero since T19.F02 (graph spec Section 11), shown by the inspector as "not converged" on every graph hop; `max_delta` stays |

1. **Config retirement.**

| Surface | Change |
| --- | --- |
| `MutationConfig` (`config/simulation.rs`) | Field, `default_action_queue_cap`, and the normalization clamp deleted; `deny_unknown_fields` rejects a stored config carrying it, with no alias or ignore shim (the T19.F02 precedent). The normalization, proptest, and default tests drop the field |
| `mutation/compound.rs` | A named constant for four input slots; `ActionQueue` width is `slots × 3` = 12 on every config; `sub_value_count` loses its config parameter and its callers stop passing one |
| Seven tracked recipes | The field is stripped from `world-recipe-extended-age-drain.json`, `-age10-baseline`, `-age10-mutation-half`, `-age10-mutation-quarter`, `-reduced-copy-growth`, `-relaxed-genome-costs`, and `world-recipe-fertility-zones.json`; each still loads |
| Frontend | `types/config.ts`, `test/fixtures.ts`, `ControlBar.test.tsx`, the `MutationSection` control, and both `bounds.ts` rules and their test; `runtime.max_actions_per_turn` keeps its static bounds |
| v3-server | `patch_config_names_each_bound_constrained_field_patched_alone` loses the cap case; its `max_actions_per_turn` case (a patch of `2` is no longer rewritten) takes a value normalization still rewrites |
| `config_digest` | Changes for every config, because the serialized `MutationConfig` loses a key; the default config's digest, the three goal cases' digests, and the recruitment-paths config digest, before and after, are recorded in the readings file |

   `max_actions_per_turn` is the one cap. The width bounds the `sub_idx` draw
   (`mutation/graph/operators.rs:183`), so behavior is identical exactly where
   the old normalized cap was 4: the defaults, both bench profiles, the goal
   recipes (which set neither field), and all seven recipes (cap 4,
   `max_actions_per_turn` 10). Elsewhere, a config with `max_actions_per_turn`
   below 4 or a hand-set cap other than 4, the width moves to 12 and mutated
   births remap by the draw; the extra sub-values read the queue's
   out-of-range `0.0`. That is the track's instruction ("held at today's four
   slots"), and its unchanged-behavior predeclaration concerns the profiles,
   which the remap does not reach. A test that runs mutation at
   `max_actions_per_turn` below 4 and pins an outcome moves by this remap and
   is re-pinned with the reason in the readings file. Old saved recipes
   carrying the field fail to load; backward compatibility is not a goal
   (`AGENTS.md`).

2. **Retired-name scan.** `scripts/policy-check` fails, printing each
   `file:line`, when `git grep --untracked -E` (tracked and non-ignored
   untracked files) finds the pattern below anywhere under `crates/`,
   `frontend/src/`, or `docs/reference/`, with no exclusion. The pattern holds
   criterion 2's names in their Rust, snake, camel, and prose spellings, the
   retired edge surface `ActionBid`, the retired tick reasons, and the retired
   config field. It is case-sensitive and `push_action` must not be followed by
   `_` or a letter, so `push_action_log` and the stats store's `pushActions`
   stay legal:

```text
PushAction|PopAction|ExecuteActionQueue|WriteDirectionBid|WriteWorldActionMeta|DirectionBank|ActionSlot|ExecuteGate|ActionBid|ActionEmitted|MaxHopsReached|push_action([^_a-z]|$)|pop_action|execute_action_queue|write_direction_bid|write_world_action_meta|direction_bank|action_bank|action_slot|execute_gate|action_queue_cap|actionBank|actionSlot|executeGate|[Aa]ction[ -]bank|[Aa]ction[ -]slot|[Ee]xecute[ -]gate|[Dd]irection[ -]bank
```

   The one hit in a committed seed record,
   `crates/v3-core/proptest-regressions/mutation/input_ref/tests.txt:7`, is a
   shrunk-value comment naming a deleted field; its comment is reworded and
   its `cc` seed kept. Other hits on 2026-09-23 outside the config retirement,
   each removed or reworded
   so the reference states the current model without naming the old one: the
   "deleted by T19.F04" and removed-opcode passages in `v3-vm-isa-spec.md`
   (Section 2 removed-opcode list), `v3-graph-backend-spec.md` (Sections 6 and 7),
   `v3-mutation-spec.md`, `v3-genome-spec.md`, and
   `v3-server-api-protocol-spec.md` (Section 2); the `genome_size()` unit list
   in `v3-runtime-config-spec.md` (it names what is counted today); the comments
   in `recruitment_paths/experiment.rs`, `recruitment_paths/tests.rs`, and
   `mutation/graph/tests/{f08,operators}.rs`; and
   `graph_operator_catalog_has_no_action_slot_operator`, deleted because the
   compiler and the scan now enforce it. Stale prose outside the pattern goes
   too: `SteeringReading`'s "structural bank-written flag" (`steering.rs:49`),
   the "bank edge" panic text (`mutation/graph/operators.rs:913`), and the
   push-model test doc at `runtime/mesh.rs:1155`.

3. **Historical records.** `docs/progress/readings/README.md` gains a
   "Historical records" section; no closed file is edited:

| Record | Historical because | Since |
| --- | --- | --- |
| T13.F02 to T13.F07 recruitment-path readings and records (`t13-f0[2-7]*` readings, `t13-f02`, `t13-f07`, `-s0`, `-s0-pilot` summaries) and every `recruitment_paths` block in summaries closed before T19.F04 | Activation used hand-built action banks; the same `recruitment-paths-v1` string names a different construction after T19.F04 | T19.F04 closure, `4bbe3d4c` |
| Steering structural readings `bank_written`/`bank_written_fraction` (T11.F21 to T19.F03 summaries) | "Executed node writes a bank"; T19.F04's `move_voted` ("contributes to a `Move` sink") is a different structure under the same `steering-v1` | T19.F04 closure |

4. **Route variation split.** The existing position and destination flags keep
   their definitions (some node's non-empty set of applied routes differs
   between two snapshots: input-driven, because perception differs and every
   snapshot starts from the same reset state). Two flags join them: some node
   applied two different route positions (respectively destinations) within one
   snapshot's execution, across passes or revisits: state-driven, because
   perception is frozen within a tick. The observed mode already records applied
   routes only (`ObservedMeshExecution`, `RECORDS_HOPS = false`). Every place
   that carries or aggregates `route_varies_with_input` carries the
   within-snapshot flags beside it, `#[serde(default)]` where stored: the
   reading, the bench `MeshExecution` block, the drift walk's `MeshTotals`,
   and the summary's `mesh_summary` projection (`bench/artifacts.rs`), which
   omits the key when a source row lacks the field, so a record measured before
   this feature reads unmeasured, never false. The flags are computed from the
   route sets the battery already collects; no execution is added. Doc comments
   name which flag is which.

5. **Trace and protocol.** Additions are captured in traced mode only; the
   production and observed modes are untouched.

| Surface | Change |
| --- | --- |
| `MeshHopTrace` (domain, sample protocol, `trace.ts`) | `decision_inputs`: the values the dispatch's `ResolveCtx` supplied for `ActionVotes` (27), `PreviousPassVotes` (27), `CommitCounts` (4), and `HopsThisTick` (1), as `f32` exactly as `resolve_input` returns them |
| `StaticInputsSnapshot` | `previous_outcome` (4), copied from `StaticInputs`, so it is the scaled value the genome reads |
| Hop `route` | Recorded exactly when the production loop resolves one for that dispatch (neither energy-exhausted nor `Decided`), as the observed mode records; otherwise `null` (T19.F04 P3). A route toward a missing node or taken before a pass cap is kept; a ramp-unaffordable hop is counted in the pass's `hops` but never recorded |
| Graph trace | `converged` and `stable_passes_count` deleted end to end; `max_delta` kept |
| `PROTOCOL_VERSION` | `v3alpha4` in `v3-server/src/types.rs` and `frontend/src/types/protocol.ts`: fields and a config key are removed; the protocol spec's Sections 2 and 4 say so |

   A traced hop adds at most 59 `f32`; a tick has at most
   `max_actions_per_turn × max_mesh_hops` hops.

6. **Inspector.** The sampler's views (`SamplerBar`, `MeshHopTimeline`,
   `VoteSurfaceBlock`, the node views they select) show what the runtime
   recorded: a view may count, group, and select recorded values (bars are
   commit counts; a kind's best sink vote is the lowest-index maximum of the
   pass's recorded votes) but performs no float arithmetic, so the effective
   vote shown is the recorded `effective_votes`. Component layout is the
   implementer's.

| View | Shows |
| --- | --- |
| Pass timeline | One group per record in `tick.passes`, in order, including a pass with no recorded hop (`MissingNode`, ramp exhaustion); each headed by pass index, end reason, the recorded `hops` count, and committed action or none; its recorded hops under it; the tick's termination reason and final queue after the last pass |
| Pass detail | Per kind: the bar at pass start, the best sink vote, and the effective vote (`raw − bar`), the committed kind marked; `Terminate` and `Decide` shown as their own fields, never as kind votes; on the pass that ends a `TerminateVoted` tick, `Terminate` against the winning effective vote; on a `Decided` pass, its `Decide` vote |
| Selected hop | The node's non-zero vote contribution; the five decision-state values available to this dispatch, labeled by input name (vectors as non-zero sinks, `CommitCounts` per kind, `HopsThisTick`, `PreviousOutcome` per channel) |
| Routes and graph hops | No route on a hop whose dispatch applied none; no convergence label |

7. **Reference and cross-references.** Besides invariant 2: the runtime config
   spec loses the cap row and its invariants and records the removal without
   the name; the graph backend spec (Section 15) and the sensor spec
   (Section 3, `ActionQueue`: 12 sub-values, four slots) state the constant; the graph
   spec drops the legacy-fields sentence; the protocol spec documents
   invariant 5. The T11.F15 spec gains one `Decision:` bullet: its single-visit
   rule and visit-filtered fallback were retired by T19.F02 at the user's
   requirement (2026-09-21), and the live contract is
   `v3-mesh-execution-spec.md`; nothing else in it is rewritten.
8. **Unchanged.** No RNG draw, founder, or tick-loop line changes. These pins
   hold unedited: `legacy_default_short_run_identity`,
   `mutation_on_applied_trajectory_guard_is_pinned`,
   `founder_only_trajectory_digest_is_pinned`, the founder digest,
   `FOUNDER_GENOME_SIZE_UNITS` (97). Traced, observed, and production execution
   still agree on every queue and reason.

## Implementation Tasks

- [ ] Config retirement, width constant, recipes, frontend config surfaces,
      server test (1); tests first where behavior is asserted.
- [ ] Route split through every carrier (4).
- [ ] Trace and protocol (5), then the inspector views (6) with their tests.
- [ ] Retired-name scan and the removals it forces (2); reference docs and the
      T11.F15 bullet (7); the README index (3).
- [ ] Readings file: digests before and after, recipe loads, the scan's
      failing and passing runs, the browser check, pins.

## Verification

- [ ] `cargo test -p v3-core --test viability`, then `make check` exit 0
      (counts in the readings file).
- [ ] Scan: `scripts/policy-check` passes on the final tree; it fails naming
      `file:line` for a retired name in a tracked file and in a newly created
      untracked file, and passes with `push_action_log` and `pushActions`
      present (temporary edits, reverted; transcript in readings).
- [ ] Config: each of the seven recipes loads through `v3-cli run --ticks 1
      --config <recipe>`; a config carrying the field is rejected; the
      `ActionQueue` width is 12 at `max_actions_per_turn` 1, 4, 10, and 20; the
      pins of invariant 8 pass unedited; digests before and after in readings.
- [ ] Route split fixtures: a node routing two ways within each snapshot and
      identically across snapshots reads within true, across false; a node
      routing one way per snapshot but differently across snapshots reads
      within false, across true; the founder reads both false. The flags reach
      the bench block, drift totals, and `mesh_summary`; a source row without
      them projects no key.
- [ ] Trace: a traced fixture whose VM node reads each of the five inputs shows
      its read values (VM register writes) equal to the hop's
      `decision_inputs` and the tick's `previous_outcome`; the hop ending a
      pass `Decided` and an energy-exhausted hop record no route, while a hop
      routing to a missing node and the hop before a pass cap keep theirs;
      protocol version tests read `v3alpha4`.
- [ ] Inspector: one frontend test per row of invariant 6; a live check on the
      dev stack sampling a creature, screenshot path and tick in readings.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred.
- [ ] Summaries stored at
      `docs/progress/features/t19-f06-retirement-and-observability.json` and
      `-goal.json` from `make bench PROFILE=gate
      FEATURE=t19-f06-retirement-and-observability` and `make bench
      PROFILE=goal FEATURE=t19-f06-retirement-and-observability` (once); raw
      hash, byte count, and verification time in readings; a JSON diff of each
      summary's `deterministic` section against T19.F05's, recorded in
      readings, shows only the within-snapshot fields and embedded config identity
      (every case `config_digest`, and `recruitment_paths.config`, which loses
      the retired key, with its `config_digest`); both series list them with
      `epoch_baseline` unchanged; no full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** No natural analog applies: this
retires a rule that had none, adds no sensor, and adds no environmental
pressure, so the three-world integration rule does not apply. Predeclared
compute: none on the production path (the width is a constant the mutation
draw reads); traced mode copies at most 59 values per hop, and neither bench
profile traces.

References: gate epoch `t19-f04-vote-based-action-selection.json`, previous
closure `t19-f05-decision-state-inputs.json`; goal epoch and previous closure
are their `-goal.json` counterparts. Standing thresholds: counters flag at 10%,
severe at 50%; wall flags at 25%, severe at 100%. The goal profile runs once.
Neither epoch is re-pinned (track "Epochs").

| Reading | Predeclaration |
| --- | --- |
| Every deterministic work counter, gate and goal, against T19.F05 | Identical per seed and per world, as exact integers (the `deterministic`-section diff of Verification, not the comparison's rounded deltas); any difference does not fit and is escalated before anything is presented |
| The same counters against the epoch (T19.F04) | Exactly T19.F05's comparison: gate `pass_cap_hits` flag, not severe; goal `severe=true` on `decided_passes`, so the goal CLI exits 3 and `make` exits 2. This is the severe the user accepted on 2026-09-22 as draw-remap movement; it is reported with the zero-delta evidence, not remediated |
| Goal indicators: persistence, lineage diversity, sensor census with `decision_inputs`, births probe, drift walk, T11.F14 and steering halves (existing fields) | Identical to T19.F05 |
| Within-snapshot route flags | First reading, no direction; founder false |
| Config identity | Changes: goal case digests in all three worlds (`inputs_changed` true), `recruitment_paths.config` and its digest, and the gate's `measurement_evidence.effective_config_digest`; the gate profile block carries no recipe digest, so the comparison still loads |
| Wall and cognition per creature-tick | No direction; the production path adds no work and the battery adds one set pass per node per snapshot, so a flag is reported with host identity and the phase it falls in, attribution pending investigation, not dismissed |
| Pins of invariant 8 | Unchanged |

**Measured verdict.** Pending.

- Summaries: [gate](../../progress/features/t19-f06-retirement-and-observability.json),
  [goal](../../progress/features/t19-f06-retirement-and-observability-goal.json).
- Full readings: [`docs/progress/readings/t19-f06.md`](../../progress/readings/t19-f06.md).

## Success Criteria

- [ ] `mutation.action_queue_cap` exists nowhere live; the `ActionQueue` width
      is 12 on every config; the seven recipes load; the digest change is named.
- [ ] The retired-name scan runs in `make check` and `make check-docs` and
      passes; criterion 2's grep finds nothing.
- [ ] The historical index names the T13 recruitment records and the
      pre-cutover steering structural readings.
- [ ] Within-snapshot and across-snapshot route variation are both reported.
- [ ] The inspector shows every row of invariant 6 from recorded values.
- [ ] Gate and goal counters are identical to T19.F05 and neither epoch moves.

## Notes for AI Agents

- Decision: model substitution for this run (Fable credits exhausted): every
  Claude role runs on Opus 5.5, the spec owner in place of Fable 5.1 `high`,
  the benchmark specialist in place of Sonnet 5; the implementer has no Fable
  advisor. Codex roles are unchanged.
- Decision: a stored config carrying the retired queue-cap field is rejected
  by `deny_unknown_fields`; no alias or ignore shim is added.
