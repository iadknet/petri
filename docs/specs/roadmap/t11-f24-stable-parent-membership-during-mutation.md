# T11.F24 — Stable Parent Membership During Mutation

**Status**: In Progress
**Last updated**: 2026-09-24
**Feature**: T11.F24
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

Within one birth, every mutation event's target draw treats a child mesh node
as parent-reachable or parent-executed exactly when it is a node the parent
carried and the parent's set contained it, however earlier events of the same
birth removed or added nodes. A node created during the birth is never a
member, even when it reuses a removed node's `NodeId` or lands at a removed
node's former index, and the target telemetry reports the corrected
membership. This repairs finding M1 of the
[post-T19 mutation audit](../../strategy/post-t19-mutation-audit-2026-09-23.md).

Natural analog: T11.F17's transcription-associated mutagenesis, where a
gene's expression-linked mutation rate belongs to that gene. Deleting a
neighboring gene does not hand its expression state to whatever now sits at
its chromosomal position.

## Non-Goals

- Recomputing the child's reachability or dispatch as targeting policy. The
  parent's sets stay the policy T11.F17 fixed. `RemoveNode`'s own
  eligibility, which reads the child's current reachability, is unchanged.
- Changing `next_node_id`, `NodeId` reuse, or any genome's node ids.
- The drift walk's between-refresh executed-id set, a `NodeId`-keyed
  observation stand-in that spans generations (T11.F17 "Observation
  stand-in"), and T13.F01's id-set module identity rule. Both are cross-birth
  harness conventions. This feature covers the engine's within-birth
  membership, which every caller, including the walk, goes through.
- Audit findings M2 to M6, and T11.F25's parameter targets.
- New telemetry fields, operators, weights, config values, sensors, or
  version strings.

## Inputs and Invariants

- Sources: the T11.F24 track row and the track note "Post-T19 correctness
  prerequisites, 2026-09-23"; audit M1 (its counterexample and
  recommendation); [T11.F17](t11-f17-executed-biased-mutation-targeting.md)
  (parent sets are sorted parent indices, founder short-circuit, telemetry);
  [T13.F01](t13-f01-module-recruitment-observability.md) (per-event
  `MutationEventRecord`, targets translated from pre-event indices to ids).
  The owning track row lists the dependencies.
- Current code (verified 2026-09-24 at `562f5e96`):
  `MutationEngine::apply_mutations_on_units` in `mutation/engine/mod.rs`
  builds `TargetSets` once from `parent_reachable_nodes` and the resolved
  `ParentExecuted`, then builds every event's `TargetSelector` from them.
  `TargetSelector::select` and `classify_target` in `mutation/reachability.rs`
  compare current child indices against those parent-index sets. Only
  `apply_remove_node` (`topology/structural.rs`) removes a mesh node
  (`nodes.remove(idx)`, shifting later indices down). Every node-creating
  topology path appends (`push`/`extend` in `structural.rs` and
  `routing.rs`). `next_node_id` probes upward from the current maximum
  id plus one (wrapping). When a removal leaves a new maximum one below the
  removed id, the next added node reuses the removed id. Each attempted operator runs on a genome snapshot that errors and
  parseability rejection restore, and the parseability gate rejects duplicate
  node ids. Every production birth (`simulation/actions/reproduction.rs`) and
  every reading (`neighborhood::{births, battery, drift,
  recruitment_paths}`) calls this engine.
- Defect, reproduced from the audit: with parent nodes `[A, B, C]` and parent
  sets `[0, 2]`, removing B leaves C at index 1, which is classified as a
  non-member, and a later appended D at index 2 is classified as C's
  membership. This changes the biased draws and misreports
  `TargetReachability`, `mutation_reachable/unreachable_target_total`, and the
  executed-target count.
- Research decision, 2026-09-24:

| Option | Evidence and fit | Disposition |
| --- | --- | --- |
| Birth-local provenance: each child node is either carried from a parent index or created in this birth, and membership is derived for current indices | Exact under removal, append and id reuse. Consumes no RNG and stays local to the engine. Equals today's sets until the first removal. Same principle as NEAT's historical markings (Stanley and Miikkulainen 2002, §3.2), where a gene's identity is its origin, not its position | Adopted |
| Key the parent sets by `NodeId` alone | Fails on reuse (audit M1; track note: "stable IDs alone are insufficient") | Rejected alone. Pairing ids with a created-this-birth set is the adopted option in another representation |
| Recompute child reachability or dispatch after each event | Changes T11.F17's parent-expression policy, and the child has no dispatch record | Rejected (track note) |
| Birth-wide allocator starting at the parent's maximum id plus one, with parent sets keyed by `NodeId` | Also exact, and addition-only births would keep today's ids. However, it changes the genome's ids in births with a removal and a later addition, which is a genome-content change M1 does not need | Rejected: changing id allocation is out of scope, while provenance leaves genomes untouched |

Fixed design:

| Decision | Value |
| --- | --- |
| Membership rule | At every target draw of a birth, a child node is parent-reachable (parent-executed) if and only if it has been present since the birth began, was created by no event of this birth, and its parent index is in the parent's reachable (executed) set. |
| Inputs | Unchanged: the same `parent_reachable_nodes` slice and `ParentExecuted`, resolved at most once per birth that draws events (T11.F17). Callers, `ParentExecuted`, and the domain mutators' signatures stay unchanged. |
| Placement | Inside the engine's event loop. The implementer chooses the representation. |
| Within-event identity | Node ids are unique after every applied event, and one topology event either removes a single mesh node or appends nodes. An implementation that relies on this pins it with a test covering every node-vector-changing topology operator. |
| Rollback | An operator that is discarded, skipped, or rejected by parseability leaves membership as it was before the attempt. |
| RNG and equivalence | Maintaining membership consumes no RNG, and `TargetSelector::select` and `biased_select_from` keep their algorithm. A draw made after a removal may consume different RNG than today, because its sets differ. For example, an eligible set that the corrected membership makes entirely executed now takes the no-roll short-circuit (`reachability.rs` `select`). Every draw made before a birth's first applied mesh-node removal sees exactly today's index sets. A birth with no draw after such a removal is therefore byte-identical to today in genome, `MutationSummary` (event records included), and RNG stream. T11.F17's 3,000-seed founder test stays green without edits. |
| Telemetry | No new fields. `TargetReachability` on each applied event and its event record, the reachable and unreachable target totals, and the executed-target count follow the corrected membership. T13.F01's event-record `NodeId`s already come from pre-event ids and stay unchanged. |
| Cost | At most O(node count) extra work per event after a removal. Zero-event births derive nothing, as before. |

## Implementation Tasks

- [ ] Write failing regressions first. Cover removal followed by a targeted
      event (a carried node after the removed index keeps its membership),
      and removal followed by addition and then a targeted event (the new
      node is a non-member at a reused id and at a former member index).
      Also cover a discarded or parseability-rejected attempt after a
      removal, which leaves membership unchanged and adds no applied-target
      tally. Cover sparse ids, and the executed-set short-circuit transition
      after a removal. Every regression asserts the resulting draw and the
      truthful telemetry: the reachability class, the executed-target count,
      and the event record.
- [ ] Add proptest coverage (v3-core) of the membership rule over generated
      genomes and event sequences, and of the equivalence invariant for
      births with no draw after a mesh-node removal.
- [ ] Implement the membership rule in the engine. Update the engine and
      `reachability.rs` doc comments that describe the parent sets. In
      `docs/reference/v3-mutation-spec.md`, the "Executed layer" item 2
      describes stale indices after `RemoveNode`; replace that sentence with
      the within-birth membership rule, which applies to every engine caller.
- [ ] Re-pin any pinned trajectory, replay, or recruitment-paths/drift test
      value that changes. Before re-pinning, show that a birth in which a
      mesh node was removed before a later draw caused the change. List old
      and new values in the readings file. No predicate may be weakened.
- [ ] Record gate and goal readings as the Performance section requires.

## Verification

- [ ] `cargo test -p v3-core --test viability` first, then
      `cargo test -p v3-core --lib mutation`. Results and the red-run
      transcript go to [readings](../../progress/readings/t11-f24.md).
- [ ] `make check` exits 0 in the worktree.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred. The
      full survivor list stays here.
- [ ] Gate and goal benchmark summaries are stored at
      `docs/progress/features/t11-f24-stable-parent-membership-during-mutation.json`
      and `...-goal.json`. Local raw hash, byte count, and verification time
      are checked, series entries point to the summaries, and no new full
      report is staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** This is a correctness repair to
T11.F17's mechanism, and its natural analog is stated in the Goal. It reaches
creatures only through which child nodes a birth's mutations land on. It adds
no environmental pressure, so the three-world integration rule adds nothing
beyond the ordinary goal run.

Expected compute cost: none measurable. The added work is bounded per event,
and only after a removal. The trajectories diverge from the first birth in
which a mesh node is removed before a later draw, so every counter can move
through the trajectory.

References. Gate: epoch `t19-f04-vote-based-action-selection.json`, latest
closure `t11-f20-per-birth-supply-rule-retirement.json`. Goal (goal-worlds-v1):
epoch `t19-f04-vote-based-action-selection-goal.json`, latest closure
`t11-f20-per-birth-supply-rule-retirement-goal.json`. Standard thresholds
apply: +10%/+50% for work and +25%/+100% for wall time. No epoch re-pin is
budgeted. A severe work counter on either profile, or an extinction in any
goal world, is a user decision under the blocker rule, and wall-time moves
are flag-only. The observation caps stay as they are: founder 10 s, evolved
180 s, drift 30 s per world, and the 15-minute goal investigation threshold.

| Indicator | Predeclared direction |
| --- | --- |
| Founder battery rows (gate founder neighborhood; goal `founder_changed_per_all_births`, `founder_dead_per_all_births` per world) | Unchanged, except births in which a parent node is removed before a later draw. That needs at least three events on the two-node founder. Any difference is reported as a birth count, with no sign predicted |
| Gate and goal work counters (`vm_steps`, `mesh_hops`, `graph_relax_iters`, `plasticity_updates`, `actions_applied`) | No direction; standard thresholds |
| Gate `births` per creature-tick, per-seed `final_population` | Move; no sign |
| Goal per world `mutation_supply.{reachable,unreachable,executed}_target_total`, each divided by `events_applied_total` | No sign predicted; recorded before and after per world, since this is the telemetry the repair makes truthful |
| Goal `final_population`, `plateau_population`, births per creature-tick per world | Move; no sign; an extinction is a blocker |
| Goal evolved changed/dead, drift depth, recruitment paths, lineage diversity | No direction; recorded |

**Measured verdict.** Pending.

- Summaries: [gate](../../progress/features/t11-f24-stable-parent-membership-during-mutation.json),
  [goal](../../progress/features/t11-f24-stable-parent-membership-during-mutation-goal.json).
- Full readings: [`docs/progress/readings/t11-f24.md`](../../progress/readings/t11-f24.md).

## Success Criteria

- [ ] Membership at every draw of a birth follows the parent's carried nodes
      through removal, addition and id reuse. Regressions and proptest are
      green, and the telemetry is truthful.
- [ ] Births with no draw after a mesh-node removal are byte-identical to
      the index rule, and T11.F17's founder test is unchanged.
- [ ] Re-pinned values are attributed and listed. `make check` and the
      mutation gate pass, with every survivor resolved.
- [ ] Gate and goal summaries are stored, and the target telemetry and
      founder rows are read before and after per world.

## Notes for AI Agents

- Decision: Fable credits are exhausted, so this feature's spec owner runs on Opus (Agent model parameter `opus`, high-effort intent) instead of Fable 5.1 `high`, is resumed with `SendMessage`, and no Fable advisor is used anywhere in the run (workflow launch step 3 is skipped; the implementer runs without an advisor).
