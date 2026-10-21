# T26–T28 design review (2026-10-20)

Codex plan-design-review of the
[interaction features design note](../../strategy/interaction-features-design-2026-10-20.md),
the T26, T27 and T28 track rows, the master roadmap changes and the rebase
amendment in `docs/workflow.md`, before the rows were committed.

This is not a feature reading: each feature's spec is written later by the
workflow's Plan step.

- **Job.** A fresh read-only `codex exec` job (`gpt-6.1-sol`, effort
  `high`) applying `$plan-design-review` with all six checks.
- **Inputs held fixed.** The user's decisions in the note's §7.
- **Format.** Findings are quoted in short form, and dispositions name the
  file that changed.

## Round 1

Verdict: `not ready`. 6 blocking, 6 should-fix.

| Check | Result |
| --- | --- |
| C1 Code decomposition | Finding (2, 3, 8, 9) |
| C2 Task decomposition | Finding (1, 5, 10) |
| C3 Domain boundaries | Finding (4, 6, 8, 9, 12) |
| C4 Premises | Finding (1–7, 11) |
| C5 Larger design flaw | Clear |
| C6 Workarounds versus refactors | Finding (2, 3, 6, 9) |

| # | Severity | Finding (short) | Disposition |
| --- | --- | --- | --- |
| 1 | blocking | Wave plan is not collision-separated: T27.F02/F03 share graph catalogs, T26.F02/T27.F02 share Phase 0, RGB caching touches T26.F01's constructors, T28.F01 can start before T26.F02 | Note §4 and master: waves described as concurrent branches reconciled at integration; T28.F01 now depends on T26.F02; T27.F01 derives RGB at perception time (no constructor change); priority order puts T27.F03 before T27.F02 |
| 2 | blocking | The three credit sites are not copies (floors, observation order, signed reverse transfers; birth assigns directly) | T26 notes and criterion: F01 extracts only the shared upper-cap step; each caller keeps its own semantics, with characterization tests; birth unchanged |
| 3 | blocking | Server revision counter already exists: projection compares the barrier mask (`projection.rs:66-112`, from `2dca26ed`) | T28 notes and note §2/§4: counter dropped; F01 adds a creature-edit → publication → client fixture |
| 4 | blocking | RGB-recomputed `PHENOTYPE_SIMILARITY` is still a built-in computation | T27 notes and criterion: similarity retired on the geometry-only rule; comparison left to evolved computation |
| 5 | blocking | "Every feature re-pins gate and goal epochs" conflicts with preserved baselines; T28.F02 widens no draw | Note §4 and all three tracks: each spec predeclares changes by artifact (pins, fingerprints, draws, readings); baselines stay as the workflow keeps them |
| 6 | blocking | Food conservation has no live contract (partial placement, clamp to configured bounds, food death drops, administrative removal, food under barriers) | T28 notes and criterion: F01 owns the live food contract and ledger buckets; F02 owns food displaced by built barriers |
| 7 | should-fix | Newborn capacity below provisioned litter, mutation timing and ledger unowned | T26 notes and criterion 2: F02 owns founder initialization, mutation timing, excess handling and a required fixture |
| 8 | should-fix | New debits and transfers lack ledger ownership; negative upkeep in Phase 0's decay-first partition | T26, T27 and T28 notes: named buckets, Phase 0 netting rule and death attribution per row; success and failure charges stated |
| 9 | should-fix | Inspection, frontend types, trace, VM sampler reachability and sparse signal writes unowned; input-bucket table timing | T26/T27/T28 notes: each row owns its inspection and type surfaces; signal writes are sparse with Graph/VM parity; the bucket table lands with T28.F01, the first row adding self inputs |
| 10 | should-fix | Rebase amendment names gate and goal only; goal-long, provenance and once-per-closure wording | `docs/workflow.md` rebase step: every declared profile, superseded provenance kept, framed as replacing invalidated verification; viability-first added to rows changing tick mechanics |
| 11 | should-fix | Boundary fixation is not proof of runaway capacity | T26 criterion 3 and note §6: descriptive distribution, boundary mass and trend |
| 12 | should-fix | T26.F01 goal line does not name its analog first | T26.F01 goal rewritten to open with heredity of body markings |

The C4 premise table also confirmed four settled user decisions as feasible:
eight signals, indexed slots with 0/1/2 sensors, movable persistent barriers,
and the rebase amendment.

One premise remains unknown: the 256-byte perception ceiling. T27.F01
disposes of it on a measured tick time.

## Round 2

Verdict: `not ready`. 2 blocking, 4 should-fix, 2 notes, all on the newly
added T25.F10 or on scheduling and arithmetic.

Round-1 resolution: 11 of 12 findings resolved. Finding 9 was partly
resolved: the input-bucket table was scheduled after its first consumer
(see 5 below).

| # | Severity | Finding (short) | Disposition |
| --- | --- | --- | --- |
| 1 | blocking | "What the gene built" has no stored baseline: groups do not record member learning defaults, and randomized creation writes per member | T25 notes: the group stores a compact heritable template (learning configuration, scope, retention). Exceptions are members departing from it, including every randomized-creation member. Only genomic state counts. |
| 2 | blocking | Drive facts cannot become pair members without a member-model change (validation allows only direction sinks reading the pair) | T25 notes and criterion: F10 owns a second member role ("fact members", template weight 0, no shared scale) with validation, relocation, deletion, detach and seeds in the container |
| 3 | should-fix | The scope draw keeps access, not per-member supply; intact n-member genes draw about n+1 times fewer events | T25 notes: stated as the intended genomic-bottleneck consequence; the spec characterizes the supply and target law |
| 4 | should-fix | Bloat risk has no deferred long-run owner; size also feeds the size-pressure gate | T25 and T27 notes: deferred to T27.F02's goal-long run (group, exception and member counts, compute, energy); the size-pressure gate is listed among size consumers |
| 5 | should-fix | The input-bucket table is scheduled in T28.F01, but T27.F01's own RGB inputs come first | T27 notes: the table and its sampler/universe agreement coverage move into T27.F01; T28.F01 extends it |
| 6 | should-fix | T25 has two incompatible live criteria (uniform rule against F10) | T25 criterion 2 rewritten: uniform rule marked as F02's outcome, superseded by F10 |
| 7 | note | F10 needs no founder recalibration (the founder has no groups) | T25 notes: remeasure, and recalibrate only if the founder size moves |
| 8 | note | After similarity is retired: 19 fields per slot, 17 facts, 76 fields, 29 recruitment edges | Counts corrected in the note §3 B and §8, T27 and T25 |

## Round 3 (confirm)

Verdict: `ready with fixes`.

- Round-2 findings 2, 3, 4, 6, 7 and 8: resolved.
- Finding 5 was partly resolved: the note's §4 still put the input-bucket
  table in E1. Fixed: §4 now delivers it in A, and E1 extends it.
- Finding 1 was partly resolved. It raised one should-fix: the criterion
  (value-based exceptions) and the notes ("every randomized member pays")
  were different rules. Fixed in the T25 notes:
  - exceptions are judged on current genomic state only, never on
    creation history;
  - randomized creation draws a template and then its members;
  - a member that matches the template is free;
  - attribute or template changes re-evaluate every member.

## User simplification after round 3 (2026-10-20)

The user replaced F10's template-and-exception rule with one unit per group,
with no change tracking. Members cost nothing more, whatever their
corrections or attributes. This retires round-2 finding 1 and the round-3
should-fix rather than resolving them.

The tradeoff is accepted: per-member variation inside a group is free to
carry. The deferred bloat reading on T27.F02 adds the count of members with
nonzero own corrections.

## Round 4 (confirm, after the simplification)

Verdict: `ready`. No blocking or should-fix issues. The simplified F10 is
consistent with T25's live criteria, T27.F02's deferred reading, the note's
§8 and this file, and it is implementable in the current group model.

## Round 5 (adversarial, challenging assumptions)

Verdict: `not ready`. 2 blocking, 7 should-fix, 1 note.

This round asked a fresh Codex job to hunt for load-bearing assumptions,
excluding earlier findings. The full assumption ledger is in the job output
(session scratchpad, `assume-out.md`). Dispositions marked "user" wait for
the user's ruling.

| # | Severity | Finding (short) | Disposition |
| --- | --- | --- | --- |
| 1 | blocking | Priority assumes an interaction deficit without showing that existing opportunities are usable. In F09's cohorts, NearbyCreature has causal parents 0/20, 1/20 and 1/20; F04 shows retained pair groups, not useful fact use | user: a bounded diagnosis before the sequence |
| 2 | blocking | The geometry-only rule is applied selectively: kin affinity (Hamming on kin_tag), lineage match and reproduce-ready remain computed social sensors while similarity is retired | user: a rule for computed social sensors |
| 3 | should-fix | Decision challenge (user): one unit per group discounts independently inherited member parameters; "genomic bottleneck" describes compression, which this is not | user |
| 4 | should-fix | A fixed bite leaves steal's targeting economics untested. At 4–9% victim-hit rates, a 1-energy bite returns 0.04–0.09 per attempt against a 0.4 floor | pending |
| 5 | should-fix | Own-capacity normalization moves inherited fullness thresholds (0.16 × 50 = 8 energy) before storage matters; mean energy is 24–33, far below 200 | pending |
| 6 | should-fix | Food carrying has no demonstrated advantageous sequence; the archived design let creatures eat from storage | user |
| 7 | should-fix | The signal magnitude cost is not an honesty mechanism (Lachmann 2001); amplitude can trade against receiver gain | pending |
| 8 | should-fix | T27's goal-long run can precede giving, since F02 and F03 run concurrently | pending |
| 9 | should-fix | Decision challenge (user): wall count does not monitor fragmentation; connectivity readings are needed | pending |
| 10 | note | The concurrency savings are unmeasured; goal-long runs take about 3 h 12 min | pending |

User rulings on round 5 (2026-10-20):

- **Finding 1:** build as planned, with no diagnosis first. 2,000-tick zero
  use is an integration reading, not a blocker.
- **Finding 3:** keep one unit per group, labelled an explicit economic
  discount in the T25 notes.
- **Finding 6:** food stays ground-only, with no eating from a slot.
- **Finding 2:** pending. The user asked for an explanation of the existing
  social sensors first.

Findings 4, 5 and 7–10 are fixed in the T26, T27 and T28 notes and the
design note §4:

- **4:** a bite-size payoff calculation before T27.F03.
- **5:** an unchanged-brain capacity comparison and a normalization ruling.
- **7:** the cost is metabolic, with an amplitude/gain check and a causal
  receiver witness.
- **8:** T27.F02 depends on T27.F03.
- **9:** connectivity readings.
- **10:** a concurrency cost note.
- **Finding 2, ruled after an explanation:** option B. T27.F01 retires
  `LINEAGE_MATCH` (oracle) and `PHENOTYPE_SIMILARITY` (genotype
  comparison). `KIN_AFFINITY` stays: the master's geometry-only rule now
  names self-referent matching as its one sensed exception.
  `REPRODUCE_READY` is unchanged. Counts: 18 fields per slot, 16 facts, 72
  fields in the family, 28 recruitment edges.

## Round 6 (confirm, after the round-5 rulings)

Verdict: `ready with fixes`. Six should-fix wording mismatches, all fixed:

1. The master and note now say ordinary condition readings (energy,
   readiness) are facts, not comparisons. The A2 rejection cites the
   self-referent exception.
2. The note's dependency table lists T27.F02's dependencies as T27.F01,
   T25.F10 and T27.F03, with F03 then F02 ordered.
3. The note's signal cost is reframed as metabolic, with the amplitude/gain
   check and the causal receiver witness.
4. Self energy normalization depends on the unchanged-brain comparison and
   the spec's ruling.
5. Ground-only eating of carried food is stated in T28 and the note. The
   densest-type pick-up rule is marked a placeholder for the spec.
6. The nearby array grows by 144 B, not 160.
