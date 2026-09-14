# T13.F05 — Function-Preserving Module Recruitment

**Status**: In Progress
**Last updated**: 2026-09-13
**Feature**: T13.F05
**Track**: [T13 — Neutral Module Recruitment](../../roadmaps/t13-neutral-module-recruitment.md)

## Goal

Gene duplication and circuit growth: for every duplication form in the fixed
fixture family below, and for every growth form the existing operators reach
within the bound, a path of at most six production mutation events leads
from inert new tissue to a useful contribution, every step before the last
retains the incumbent's behavior, and the last step is one bounded connection
or parameter edit that exposes the effect. Each path uses the existing
split, copy/diverge and connection operators through their production entry
points with recorded seeds; a transition no production operator can make is
repaired in that operator, and paths that already exist become maintained
regression tests with the record that no operator changed. It reaches
creatures only through the body: birth mutation of the genome.

## Non-Goals

- No blank-template catalog, macro operator, authored controller, novelty
  reward, speciation, fan-out, or guarantee that every neutral node becomes
  useful.
- No change to operator weights, domain shares, the executed/reachable bias,
  T11.F18's constructors or odds, T13.F03's applicability rule or T13.F04's
  entry rule; T13.F03's deferred executed-applicable bias question stays a
  track decision.
- No change to the T13.F02 experiment's parameters, seeds, tasks or margins;
  no discovery or retention floor (T13.F06).
- No shortening of a path that exists but is long.
- No environmental pressure: nothing to integrate into the three standard
  goal environments.

## Inputs and Invariants

- Owning row: T13.F05; the track's **F05 scope** note defines acceptance.
  [T13.F03](t13-f03-mutation-target-applicability.md): the applicability
  predicate is the single source of truth for an operator's sites; no
  select-then-fail path. [T13.F04](t13-f04-direct-graph-effect-activation.md):
  the derived Graph entry rule; a wired zero-compute module is an admitted
  intermediate; no stored activation flag.
  [T11.F18](t11-f18-backend-neutral-mesh-node-growth.md): blank detours are
  `blank_graph_backend` / `minimal_vm_backend` at 0.5/0.5.
  [T13.F02](t13-f02-recruitment-paths-and-replicated-baseline.md) supplies,
  unchanged: Task A and B, the eight scenes, task-live, the 1/8 margin,
  usefulness by `static_successor_bypass`, the battery signature and the
  stage record in `neighborhood/recruitment_paths/fixtures.rs`.
- Research, checked 2026-09-13 against local evidence. The
  [recruitment research note](../../strategy/neutral-module-recruitment-research-2026-09-08.md)
  fixed the direction: reuse safe connected growth and copy/diverge paths,
  add only a path the baseline shows missing, prefer a small change to an
  existing operator over a macro catalog. Options: (1) seed-selected
  production-operator paths, F02's pattern for `CopyNode` and
  `SwapRouteTargets` — adopted, each step is proved with the operator that
  makes it in production; (2) automated operator × seed search — rejected, a
  transition's existence is decided by site enumeration; (3) new template or
  motif operators — excluded by the track.
- Baseline facts. After T13.F03/F04 every blank, copy, split and unprepared
  experiment arm is still null (0/32, Wilson 0.000–0.107) while prepared arms
  discover 17–19/32; Graph `contributing` stayed 0 in 5 of 6 drift cells.
  F02's constructed paths take three stages after creation for blank forms
  and two for copies, but the preparation stage is one authored compound
  edit, never shown as production operator steps.

**Fixture family (fixed before any operator changes).** Nine starting forms,
each frozen with its genome, creation edit and Task/battery reading before
qualification, never chosen for an observed outcome:

| Form | Backend | Task | Creation and incumbent |
| --- | --- | --- | --- |
| Blank branch | Graph, VM | A | F02 blank start: current constructor, tied later route target of the entry node; incumbent always-NoOp, 4/8. |
| Dormant copy | Graph, VM | A | F02 copy start: production `CopyNode` of the dead reactive module, dormant by the T11.F15 attachment proof. |
| Neutral split | Graph | A | F02 split start: `split_existing_edge` inside the dormant copy. |
| Unprepared copy | Graph, VM | B | F02 changed-task unprepared arm: exact copy of the Task A-correct incumbent (8/8 A, 4/8 B) at the recorded fork. |
| Inline detour | Graph, VM | A | Production `AddNode` on the entry-to-incumbent edge with a recorded seed; the detour is dispatched every tick and forwards to the incumbent. |

The inline detour is the only form whose neutral steps run under execution;
the others are neutral by non-dispatch until activation.

**Path contract.** A path is an ordered list of at most **six** production
events after the starting form, each one call of `TopologyMutator`,
`GraphMutator`, `VmMutator` or the InputRef mutator's `apply` with an explicit
operator and a recorded seed, and each recorded as an F02 `ConstructionStage`
with its `GenomeDelta`. Six is twice F02's longest constructed stage count after
creation and about a third of the ~16 applied events a lineage retains over
the 32-generation discovery horizon.
For every step before the last the subject is task-live, loses at most 1/8 of
the active task score against the preceding stage, and keeps the incumbent's
complete battery signature; where the module writes shared memory, queues an
action or writes a route gate, those surfaces are compared separately and
must be unchanged. Real charges are recorded per step (energy, VM steps,
`graph_relax_iters`, genome size) and never vetoed unless they make the
subject task-dead. The last step is exactly one connection or parameter edit
(a topology `SwapRouteTargets`, `MutateGateBias` or `AddRouteTarget`; a Graph
edge, weight, param or action-slot edit; a VM instruction, raw-field or
delete edit) after which the module is dispatched, its bypass loses at least
1/8, and the active task score is at least 1/8 above the starting form's. A
seed is found by a one-off search of at most 1,000,000 seeds per step, recorded
in the readings with the odds the operator's draws imply; only the pinned seed
replays in the maintained test. Exhausting the range is not evidence of a
missing transition.

**Missing transition.** A path step is missing when the genome delta it needs
is one that no production operator can produce from the preceding genome:
shown by enumerating the operator's applicable sites (the F03 predicate and
the operator's own site enumeration) and citing the documented range of each
draw it makes, recorded as the operator, the site or range it lacks and the
enumeration that proves it. A transition is also missing in practice when
the operator draws a field over values the runtime decodes to nothing
(`PushAction.action_type` as a full `u8` where `decode_world_action` admits
0..=4): the repair draws from the decodable range only, leaving existing
out-of-range genome values and the raw-field unit step unchanged. A repair
aligns that operator's site set or draw with the values it admits. A site
change extends the predicate in the same change and carries the F03
invariants 1–3; a draw change carries a property test that the draw covers
exactly the admitted range; either carries a function-preservation property
wherever a path uses the repaired step as a neutral one. A repair that
would need a new operator, a template, or a change to weights or bias is out
of scope and is reported, not built.

**Growth gap.** A form whose shortest complete path exceeds six events is not
missing a transition; its path, length and lengthening step are recorded in
the readings and it fails no criterion below. Copy and split forms are
expected to qualify in at most three; one that exceeds six with no missing
transition is a blocker for the user's decision, not a repair.

**Regression coverage.** Every qualified path is a maintained test in
`crates/v3-core/src/neighborhood/recruitment_paths/` replaying each step
through the production operator with its recorded seed, asserting the
per-step facts, and replaying through `GenomeDelta::apply`. Mutation, runtime
and simulation never depend on observation types.

**Measured, not preserved.** With a repair: applied mixes, RNG consumption
per event, every evolved trajectory after the first affected birth, drift,
neighborhood and experiment readings; cross-process determinism
(`tests/reproducibility.rs`) and the gate two-run check still hold; a pinned
expectation that encoded the old draw is re-pinned with its reason. Without
one, every simulation counter and observation reading equals the previous
closure's exactly.

**Reference.** A repair updates the operator's paragraph in
`docs/reference/v3-mutation-spec.md` §3; path lengths live in the readings.

## Implementation Tasks

- [x] Freeze the nine starting forms and add the seed-selected step harness
      (`recruitment_paths/qualification.rs`) on F02's stage seams.
- [x] Qualify each form: record the shortest path found, per-step readings,
      and every missing transition or growth gap in the readings.
- [x] TDD each missing transition's repair in the existing operator with
      viability run first; one found, the `PushAction.action_type` draw.
- [x] Turn every qualified path into the maintained regression tests above.
- [ ] `cargo check --workspace --all-targets`, `cargo clippy`, `cargo fmt`,
      `make roadmap-check`; update `docs/progress.md`,
      `docs/progress/benchmark-series.json` and the owning row at closure.

## Verification

- [x] `cargo test -p v3-core recruitment_paths` (ok: 25 lib, 3 integration)
      -> every path replays its pinned seeds through the production operators
      and `GenomeDelta::apply`; seven of nine forms qualify, both blanks are
      complete 7-event growth gaps; path, per-step and seed-search tables in
      [readings](../../progress/readings/t13-f05-function-preserving-module-recruitment.md).
- [x] One production draw repaired (`PushAction.action_type` in `0..=4`,
      readings "Seed search"): viability ok before and after, `cargo test -p
      v3-core`, `--test reproducibility` and `-p v3-cli` ok, one test re-pinned.
- [ ] `make check` on the final feature code -> tested commit recorded below.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants` -> summary line, run mode,
      output path and every survivor resolved in the closure record below.
- [x] Benchmark summaries at
      `docs/progress/features/t13-f05-function-preserving-module-recruitment.json`
      and `-goal.json`, raw hash/byte count and verification time checked,
      series entries point to them from the closing commit, no full report
      staged; exits, `severe`, caps, floors and experiment fractions in the
      readings. One goal run per the 2026-09-05 decision.

```sh
make bench PROFILE=gate FEATURE=t13-f05-function-preserving-module-recruitment
make bench PROFILE=goal FEATURE=t13-f05-function-preserving-module-recruitment
```

| Closure record | Value |
| --- | --- |
| Tested commit | pending |
| Mutation gate | pending |
| Closure documentation checks | pending |
| Mutation output audit | pending |

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: gene duplication
followed by divergence of the copy, and growth of new circuit tissue that
passes signals through unchanged until one connection makes it matter; it
reaches creatures through the body (birth mutation of the genome) and through
no sensor. Expected compute cost: none. A repair, if any, extends an
operator's site set or draw inside the mutation path, which is a small share
of tick work; the qualification itself is test-only. No severe allowance,
threshold change or epoch re-pin is predeclared. Both profiles compare against
the series index at run time: gate epoch `remove-complementary-nutrition.json`
and previous `t13-f04-direct-graph-effect-activation.json`; goal-worlds epoch
`t13-f03-mutation-target-applicability-goal.json` and previous
`t13-f04-direct-graph-effect-activation-goal.json`, all under
`docs/progress/features/`. The +10%/+50% work and +25%/+100% wall flags
stand; inherited epoch flags are recorded apart from change against the
previous closure; new flags are investigated. Caps: founder observation
under 10 s, evolved under 180 s summed across seeds, drift under 30 s per
world, T13.F02 experiment under 120 s, goal profile under 15 minutes. Drift
floors are the T11 track's re-based 0.0015 at depth 1,000 and 0.005 at depth
2,000, strict not-below per world; T13.F03/F04's acceptances of readings
below 0.005 do not extend here, so such a reading is escalated.

| Indicator | Predeclared direction |
| --- | --- |
| Qualified path length per form | Copy and split forms at most 3 events; blank and inline forms recorded, growth gaps named. Deterministic, not a floor. |
| Six normalized simulation counters, wall/creature-tick | No operator repaired: identical to the previous closure. Repaired: no predeclared direction, inside the flags. |
| Founder neighborhood: `VmInstructionMutation` silent share, per-birth silent and dead fractions | The `PushAction` draw repair makes inserted pushes decode 5/5 instead of 5/256, so that operator's silent share falls by at most its `PushAction` share of inserts (about 1/41); per-birth silent and dead stay within the T11 floors (d) at most 5% dead, (e) at least 60% silent. Justified as a draw alignment; no other component is expected to move. |
| Drift changed / all births at depths 1,000 and 2,000 | Floors 0.0015 and 0.005 apply. Without a repair the readings equal T13.F04's (7/9/7 per 2,000, below the floor) and are escalated as such; with one, expected to hold or rise. |
| T13.F02 in-report experiment fractions (blank, copy, split, unprepared arms) | Identical without a repair; move with one, reported as consequences, no floor. |
| T13.F01 rungs and Graph/VM contributing counts | Same rule: identical or measured consequence, no floor. |
| Neighborhood, diversity and cognition indicators | No predeclared direction. |

**Measured verdict.** Gate: exit 0, `severe=false`, no flag, counters
byte-identical to T13.F04. Goal: exit 0, `severe=false`, `plasticity_updates`
+22.75% flag against T13.F04 only, every cap held, founder
`VmInstructionMutation` silent share and dead unchanged as predeclared. The
founder row above mis-cited T11 floor (e): it is the single-event floor and,
like (d), track-level, met by T11.F10; the per-feature rule is no regression,
and the founder readings (per-birth silent 54.3/57.7/54.3%, single-event
59.1/62.2/59.1%) are byte-identical to T13.F04's, so this is not a miss.
Depth-2,000 drift 7/9/7 per 2,000 is identical to T13.F04's and below the
0.005 floor; the T13.F03/F04 acceptances do not extend, so closure and merge
wait for the user's decision while review and the mutation gate proceed.

- Summaries: [gate](../../progress/features/t13-f05-function-preserving-module-recruitment.json),
  [goal](../../progress/features/t13-f05-function-preserving-module-recruitment-goal.json).
- Full readings: [`docs/progress/readings/t13-f05-function-preserving-module-recruitment.md`](../../progress/readings/t13-f05-function-preserving-module-recruitment.md).

## Success Criteria

- [ ] Every dormant-copy, unprepared-copy and neutral-split form has a
      qualified path of at most six production events whose neutral steps keep
      the incumbent's battery, queue, memory and routing effects and whose
      last step is one connection or parameter edit exposing a useful,
      bypass-sensitive contribution.
- [ ] Every blank and inline form has a qualified path or a recorded shortest
      path with its growth gap named; every missing transition is repaired
      inside the existing operator with the predicate extended and property
      coverage, or the spec records that no production operator changed.
- [ ] Every qualified path is a maintained replaying test; reference specs
      state any repaired operator's semantics.
- [ ] Benchmark evidence stored, drift floors read, and the mutation gate
      recorded.

## Notes for AI Agents

- Decision: Module identity and usefulness are T13.F02's (one mesh node,
  `static_successor_bypass`, 1/8 margin); intra-module split and copy
  operators are path steps, not separately scored modules.
- Decision: 2026-09-13, build-pass escalation. A path's node draw may use the
  uniform `TargetSets::new(&reachable, &[]).selector(0.0, 0.0)` F02 used:
  since T13.F03 the biases only reweight the applicable set, so they change
  no path's existence. `graph_blank` and `vm_blank` at seven events are
  growth gaps, not failures. The full-`u8` `PushAction.action_type` draw is a
  transition missing in practice and is repaired as above, justified by
  decodability (5/256 to 5/5) alone. Of the other exhausted steps, the meta
  slot draw was rare and is met by the larger seed search; the jump steps
  were a harness acceptance defect (reference repair rewrites offsets),
  corrected in the harness, not a draw defect.
