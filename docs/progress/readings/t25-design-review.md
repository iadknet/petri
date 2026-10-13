# T25 design review (2026-10-13)

Adversarial Codex Astra review of the
[edge groups and learning design note](../../strategy/edge-groups-and-learning-design-2026-10-13.md)
before the T25 rows were committed. Not a feature reading: the track's specs
are written later, one per feature, by the workflow's Plan step. Each round
is a fresh read-only `codex exec` job (`gpt-6-astra`, effort `high`) over the
note, the rows and the code, with the user's decisions fixed and the design
under attack. Findings are quoted in short form; dispositions name the note
section that changed.

## Round 1

Verdict: `not-ready`. 17 blocking, 7 advisory.

| # | Severity | Finding (short) | Disposition |
| --- | --- | --- | --- |
| 1 | blocking | A genome is immutable after birth (`state.rs:182`); one container owning structural edits, birth correspondence and learned-state alignment conflates three concerns | Edited §7: the container owns structural edits and emits edit records; reproduction owns birth correspondence; runtime owns learned-state construction |
| 2 | blocking | `scale × pattern + weight` moves `DirectionPair` members by different amounts and signs, contradicting "changes every member equally" | Rebutted, user to confirm: §4 keeps one change to the shared scale as the meaning of equal credit; an equal additive change across directions is a drive, a separate gene by the 2026-10-11 decision |
| 3 | blocking | `effective_weight` takes genomic values only; the runtime substitutes learned values (`hebbian.rs:50`) and the census reads runtime storage | Edited §3: pure `genomic_weight` plus a runtime `applied_weight` resolver; structural readers use the genomic one |
| 4 | blocking | Group learning: no equation uses the group's own configuration; `Static` has no Lamarckian flag | Edited §3–§4: the group record owns no learning configuration, only `inherit_scale`; each member computes Δ with its own configuration and routes it by scope; static members contribute nothing; contributions sum in genome order |
| 5 | blocking | `weight_clamp` has no owner after `PlasticityConfig` goes | Edited §3–§4: `clamp` on each learning variant; bounds the learned quantity the update writes; Oja reads the applied weight |
| 6 | blocking | `Member` undefined; retarget, split and raw-field edits can break geometry | Edited §3: membership is a validated `(source, target)` coordinate; such edits detach first, atomically; compute-input edges cannot be members |
| 7 | blocking | Group removal and reference pruning need one atomic reindex | Edited §4: `remove_group` is atomic and reindexes every link; runs from `reindex_input_refs_after_removal`; `InputRef.Prune` counts shape references; empty groups removed; property tests |
| 8 | blocking | Offset shift and kind retarget relocate materialized members; holes, exceptions, order and birth correspondence unspecified | Edited §4: members relocate with pattern, correction, learning and learned state; appended in genome order; holes stay holes; nothing regenerated |
| 9 | blocking | Detach loses learned values or absorbs them into the genome; inheritance flags | Edited §3: detach is a genomic transformation and a runtime transformation that keeps the applied weight; inheritance keyed on the child's post-mutation flags per edge and per group |
| 10 | blocking | Learning after effects cannot re-resolve sources: effects write shared memory during the sink loop (`effects.rs:192, 202`) | Edited §4: updates consume the evaluation-time samples the executor records; one convention for both update kinds |
| 11 | blocking | "Mask edges uniformly" unspecified; today's ablation rewrites references on copies | Edited §4, §7: an observation-only evaluation-context mask; genome, order, learned-state correspondence, size and complexity untouched |
| 12 | blocking | Stored membership does not replace ring-rule and drive analyses, which cover independent edges and bus paths | Edited §7 and the track criterion: explicit membership versus derived wiring analyses; analyses keep their definitions over members and genomic weights |
| 13 | blocking | `genome_size` charges one unit per wired sink (`mod.rs:280`): eight members onto unwired sinks cost 17, not 9 | Edited §3: the wired-sink unit is dropped for a uniform rule and the founder anchor re-pinned; flagged as a change beyond the accepted decision, user to confirm |
| 14 | blocking | F01 already needs group removal; F03's attribute operators need the scope helper, not F04 | Edited §8: F01 owns removal and structural invariants; F03 owns attribute operators and the scope draw; F04 owns creation forms, detach, catalog |
| 15 | blocking | The family merge changes the default-on `Slots` refinement key before F04 decides its disposition | Edited §4, §8: F02 re-keys `Slots` on the merged layout; F04 decides with a reading |
| 16 | blocking | Draw contracts: "scope first" versus "pick an edge first"; creation forms leave member scope or group configuration undefined | Edited §4: per-feature draw contracts (edge first, then a scope coin; uniform and randomized creation fully specified; `inherit_scale` a coin flip) |
| 17 | blocking | F01 and F04 declare `gate` where the workflow's `gate + goal` row applies; "unchanged live avoidance" is not a gate reading | Edited §8: F01 and F04 declare `gate + goal`; F01's equivalence reading is a fixed fixture, not an evolved fraction |
| 18 | advisory | Displayed weights: genomic versus applied; transport derived values | Edited §7: the creature endpoint adds applied weights and learned values as derived fields; the inspector shows both |
| 19 | advisory | Pair metadata belongs with the sensor layout; input references are shared by both backends | Edited §4: eligible pairs declared beside the field indices in `perception.rs`; F02 owns the merged layout as the shared contract |
| 20 | advisory | "Same answer before and after" contradicts fixing readers that omit projections | Edited §7: explicit post-migration expectations for the three readers; unaffected readers pinned separately |
| 21 | advisory | "Retained" pair groups and nonzero scales can come from supply alone | Edited §8 F02 readings: retention across an observation interval against creations and recorded draws |
| 22 | advisory | "Edges at the clamp within a life" needs history the census does not keep | Edited §8 F03 readings: sampled clamp occupancy |
| 23 | advisory | Separate CPU and memory predeclarations for F01 and F03; no membership search in per-edge evaluation | Edited §3, §8, §9: indexed lookup, sparse learned storage, per-feature predeclarations |
| 24 | advisory | Identity must include heritable attributes and exclude learned values and birth payloads | Edited §3: identity invariant |
| 25 | advisory | "A term that is always 0" is false under an additive correction | Edited §3: omitted members are geometrically unnecessary at birth, not impossible |

## Round 2

Verdict: `not-ready`. Of round 1: 21 confirmed, 4 partial (1, 9, 11, 16),
0 rejected. 7 new blocking findings.

| # | Severity | Finding (short) | Disposition |
| --- | --- | --- | --- |
| 1 | partial | §7 still had reproduction building runtime state and the runtime consuming edit records | Edited §7: reproduction folds published records into seed values; the runtime owns a constructor from seeds and knows nothing of records |
| 9 | partial | A static member's learned group contribution has no slot to survive detach | Edited §3: detach is genomic only; seeds follow the child's flags; a static member starts from its genomic weight and the parent's learned-scale contribution ends with the membership |
| 11 | partial | The mask omitted VM reads, which today's ablation rewrites (`consumers.rs:289`, lab `readings.rs:633`) | Edited §4: the mask keys `(mesh node, input reference)` for both backends, plus per-edge triples and a learning flag |
| 16 | partial | Creation left "or Static", scope probability and reward-modulated draws unspecified | Edited §4: static creation (F02, F03), uniform learned creation (F04) and randomized creation (F05) with every draw and probability named |
| 26 | blocking | "What moved where" cannot carry today's inheritance semantics (weight steps invalidate, identity splits preserve) | Edited §7: a record table (created, copied, removed, relocated, invalidated, detached, plus mesh-level records) with each kind's inheritance rule; reproduction resolves seeds; a runtime-owned constructor |
| 27 | blocking | Birth correspondence outside the genome loses the engine's snapshot rollback (`engine/mod.rs:647`) | Edited §7: event-local batches published only after validation, discarded with the rollback; a regression for rejection followed by an applied event |
| 28 | blocking | The graph container misses whole-module ancestry: topology copies backends, removes mesh nodes, replaces backends (`structural.rs:75-236`; tested at `reproduction.rs:981`) | Edited §7: mesh-level records from the topology operators in the same event batch; the graph container owns no topology |
| 29 | blocking | F01 combined a separately shippable infrastructure migration with the representation change | Edited §8 and the track: new F01 (container, edit records, birth seeds; behavior-preserving; gate with identical deterministic output); the ring replacement is F02; five features |
| 30 | blocking | F01/F02 creation drew learning attributes that only F03 introduced | Edited §4, §8: creation is static with no learning draw until the learning feature (now F04), which owns uniform learned creation; randomized creation stays last |
| 31 | blocking | Writing an attribute to heterogeneous `Learning` variants was undefined | Edited §4: a total transition table per (operator, variant), draws once before the member loop |
| 32 | blocking | "Carrying cost falls for everyone" omitted materialization; supply and gestation also read size; complexity treatment of groups unspecified | Edited §3: a complete accounting contract (`Δgenome_size = 8P − S`, every reader of size, functional complexity rules, F02 predeclares and reads) |

## Round 3

The first attempt at 11:23 on 2026-10-13 hit the Codex usage limit before
any output; the retry ran after the 11:44 reset. Verdict: `not-ready`. Of
round 2: 7 confirmed, 5 partial (11, 16, 26, 31, 32), 0 rejected. 3 new
blocking findings and 1 advisory. The workflow's loop is three rounds, so
the dispositions below were applied to the note but not re-reviewed by
Codex; the user decides whether a fourth round runs.

| # | Severity | Finding (short) | Disposition |
| --- | --- | --- | --- |
| 11 | partial | The mask cannot distinguish VM sub-values, which today's ablation does (`consumers.rs:295`) | Edited §4: mask keys carry a sub-value or "all" |
| 16 | partial | `SetScope`, clamp mutation and shared parameter-step laws incomplete | Edited §4: `SetScope` writes the picked edge's new scope; `clamp` uses the rate law bounded to `[0.01, 10]`; one drawn step applied to each member's own value |
| 26 | partial | The record table omitted source-deletion invalidation (`cgp.rs:382`) and the no-op retarget rule (`operators.rs:686`) | Edited §7: `ComputeInserted`/`ComputeRemoved` records with the dangling-source invalidation; a retarget to the same source changes and invalidates nothing |
| 31 | partial | "Keep rule, rate, clamp and flag" contradicted "every member receives the same values"; same-mode transitions unspecified | Edited §4: drawn fields are shared, retained fields stay each member's own; a member already in the target mode is unchanged |
| 32 | partial | Accounting omitted size-pressure eligibility (`engine/mod.rs:130`) and the predation kill bonus (`predation.rs:127`) | Edited §3: both added to the reader lists |
| 33 | advisory | F05 owned the catalog triplet although F03 and F04 add or replace operators under exhaustive dispatch (`graph/mod.rs:333`) | Edited §8: each feature integrates the operators it introduces; F05 keeps its own additions and a final reconciliation |
| 34 | blocking | Removing an empty compute node shifts later edge surfaces with no edge record (`cgp.rs:377`); batch addressing undefined | Edited §7: compute insertion and removal records; sequential addressing; relocation as an ordered record sequence |
| 35 | blocking | `Detached { occurrence }` lacked the operands to compose a seed after earlier invalidation or when the last member deletes its group | Edited §7, §3: `Detached { occurrence, group, pattern }`; composition from current seed state with genomic fallbacks; nothing parental resurrected |
| 36 | blocking | Oja on a shared scale could read a scale an earlier member changed in the same visit (`hebbian.rs:136`, `traces.rs:151`) | Edited §4: every Δ, Oja's term included, uses the recorded visit-start applied weight; contributions and clamps then apply in genome order |
