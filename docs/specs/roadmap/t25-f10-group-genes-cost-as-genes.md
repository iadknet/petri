# T25.F10 — Group Genes Cost as Genes

**Status**: Complete
**Last updated**: 2026-10-21
**Feature**: T25.F10
**Track**: [T25 — Edge Groups and Per-Edge Learning](../../roadmaps/t25-edge-groups-and-learning.md)

## Goal

The genomic bottleneck: one developmental gene wires many synapses, so a
creature pays to carry and copy the gene, not every synapse it built. After
this feature `genome_size()` charges one unit per edge-group record and
nothing for its members, whatever their own corrections, learning
configuration, scope or retention: a `Ring` group is one unit (nine today),
a `DirectionPair` group one (thirteen today), a hole nothing, a detached
member an ordinary edge at one unit. A pair recruitment's drive facts become
a second member role of the same gene, **fact members**: one edge per
declared fact field into `Drive K`, born at weight 0, pattern 0, no shared
scale, so a recruitment's twelve direction members and its facts cost one
unit together, not fifteen or nineteen. Only genomic state counts: runtime
learning and birth seeds never change the birth-cached size. The founder has
no group, so its 80 units and the rate `0.0060625` do not move; the spec
remeasures them. This is an explicit economic discount, not compression:
members still store their own weights and settings (user, 2026-10-20).

Natural analog: the genome is far too small to specify each synapse, so
developmental rules wire many synapses from few genes (the genomic
bottleneck, Zador 2019, cited by the design note §8 from memory, not
re-read); an indirect encoding charges the generator, not the phenotype it
expands to. It reaches creatures through the body: the per-unit carry cost,
the replication multiplier, gestation, the size-pressure roll and the
per-unit mutation supply all read `genome_size()`.

## Non-Goals

- A template, exception or change tracking on members (retired by the user's
  simplification, 2026-10-20); a per-group energy charge; a role split of
  the four size costs (design note §8 option a).
- Fact members outside a recruitment: `AddDirectionPairGroup` and its
  randomized twin still build twelve direction members and no facts; a slot
  plan landing on a non-directed sink still appends ungrouped edges.
- Any change to `complexity()`, its readers, the operator catalog's
  membership or weights, the recruitment draw order, the carry, replication
  or gestation formulas, the anchor or the rate.
- A long-run bloat reading: deferred to T27.F02's goal-long run (Notes).

## Inputs and Invariants

| Input | Where | What F10 takes from it |
| --- | --- | --- |
| Track row, criterion 9, Notes (2026-10-20) | `docs/roadmaps/t25-edge-groups-and-learning.md` | The goal and the contract sketch: one unit per group, members free, holes free, a detached member one unit, only genomic state counts, no template or exception tracking, the "explicit economic discount" label; fact members as a second role of the pair gene with validation, relocation, deletion, detach and seeds in the container; the supply-and-target law as the intended consequence; founder remeasured, recalibrated only if it moves; bloat deferred to T27.F02; "viability first, gate + goal"; T25's goal-long allowance spent. |
| Design note §8 and its review | `docs/strategy/interaction-features-design-2026-10-20.md` §8; `docs/progress/readings/t26-t28-design-review.md` rounds 2–5 | The trigger (28 silent edges per signal recruitment under T27.F02), the three options and the user's choice (b); finding 3 of round 5 and its ruling. |
| Size rule today | `creature/genome/mod.rs:298-326` `genome_size` (members counted as sink edges, one unit per record); `creature/founder.rs` `FOUNDER_GENOME_SIZE_UNITS = 80`; `config/simulation.rs` `per_unit_rate` `0.0060625` and its founder-supply test (`:1346`) | What changes and what is pinned. |
| Groups today | `cgp/group.rs` (`Group`, `Shape`, `pattern`: 0 for a direction ≥ 8 or a field that is neither `x` nor `y`; `detach`, `pair_birth_members`, `compose`); `cgp/container.rs` (`validate_graph:428-468` pair role, `check_local_invariants:1382-1480` keyed by `(sub_idx, direction)`, `members:1093`, `add_pair_group:1141`, `insert_member:1309`, `take_members:1328` with direction 0 off a direction sink, `relocate_group:1202`, `detach_at:1233`, `drop_group_record:1351`, `pattern_at:690` with `u8::MAX` off a direction sink, `can_host_group:1077`); `cgp/edge.rs` (`Membership { group, scope }`, 40-byte `GraphEdge`) | The member model fact members extend; the sentinels the role replaces (readings file, audit R1, R5). |
| Recruitment today | `mutation/graph/recruitment.rs:71-96` (`append`: `add_pair_group`, then one ungrouped `GraphEdge::new` per `layout.facts` into `Drive K`), `:141-150` (`hosts_pair_plan`: every kind's `Drive` sink present); `contracts/inputs.rs` (`DirectionPair { x, y, facts }`: area families 3 facts, a `NearbyCreature` slot 7) | The gene recruitment builds at two sites today; the preflight that already requires the drive sink. |
| Mutation today | `mutation/engine/mod.rs:88-145` (supply on `genome_size()`, size-pressure roll on it), `:600-660` (`group_edits`, `WORKAROUND(T25.F08)`); `mutation/graph/hebbian.rs:149-166` (`scope_targets` broadcast over `members`), `operators.rs:287-300` (`detached` before retarget, raw-field step, split, internal-node insertion), `group.rs` (`remove`, `shift_offset`, `retarget_kind`, `detach_member`); `mutation/types/mod.rs` (`ComplexityEffect` per operator); `engine/group_tests.rs:473-500` (`expected_delta` per operator) | The operators whose unit delta changes and the test that pins it. |
| Readers | `v3-cli/src/bench/pair_groups.rs` (`pair-groups-v1`, `members` over `def.members`), `v3-cli/src/bench/artifacts.rs:618-628,723-733` (the summary projection's keep-list), `v3-cli/src/lib.rs:205-230` (`structure_means`, `Ring` only), `creature/drives.rs:110-120` (drive edges by own weight on the `Drive` sink, no membership filter), `creature/ring_rules.rs:180-200`, `v3-server/src/http/creature.rs:130-160` (`graph_weights`, one entry per member), `runtime/plasticity/learned.rs:170-240` (a member's term `pattern × scale`), `frontend/src/components/inspector/EdgeGroupList.tsx:53-82,95` (members by `(channel, direction)`), `neighborhood/mutation_effects/recruitment_reading.rs:49-62`, `neighborhood/recruitment_paths` (slow-tier readings), `v3-telemetry/src/metrics.rs:977` (`petri.tick.mean_genome_size`), `:680` (`petri.run.genome_size_creature_ticks`) | Who moves by construction, who needs the role, who is untouched. |
| Pins | `tests/viability.rs` founder-only digests (must not move in any commit); every mutation-on pin (`tests/baseline_worlds.rs`, `tests/applied_trajectory.rs`, `mutation/engine/recruitment_tests.rs`, `mutation/graph/tests/*`, `runtime/cgp/neutral_connection_tests/recruitment.rs:52,78`, v3-cli trajectory and deterministic fixtures, the slow-tier recruitment-paths readings); `crates/v3-cli/tests/bench_artifacts.rs` digests; `docs/progress/benchmark-series.json` (gate and goal-worlds epochs at F09) | The predeclared re-pins (invariant 7). |
| Reference specs | `v3-genome-spec.md:68-70`, `v3-graph-backend-spec.md:334-335, :352-362, :706-709`, `v3-mutation-spec.md:515, :568, :590, :648-656, :836-839`, `v3-reproduction-spec.md:81-86` (unchanged in form), `frontend/.../EnergyLifecycleSection.tsx:30` (tooltip) | Every sentence that states the member unit or a group's unit count. |

Vocabulary: a **direction member** is a member on a direction sink `K(d)`
(every member today); a **fact member** is a member of a `DirectionPair`
group on the `Drive K` sink of its record's kind, reading one of its pair's
declared `facts` fields; a member's **site** is `Direction(d)` or `Drive`;
a **gene** is a group record with its members of both roles.

| # | Invariant |
| --- | --- |
| 1 | **Unit rule.** `genome_size()` = mesh nodes + input references + route targets + (VM) program length + constants + (Graph) compute nodes + compute edges + sink edges with no membership + one per group record. A member of either role adds nothing; a hole adds nothing; a sink adds nothing of itself (F02). `Δgenome_size` per committed event: `AddRingGroup`, `AddDirectionPairGroup` and their randomized twins +1; `RemoveGroup` −1; `DetachGraphEdge` +1, or 0 when the record goes with its last member; `RemoveGraphEdge` on a member 0, or −1 when the record goes; an implicit detach (`RetargetGraphEdge`, `GraphRawFieldMutation`, `AddInternalGraphNode`) `+1 − records_removed` beside the operator's own delta, so retargeting a sole direction member or a sole fact member into an independent edge adds nothing; recruitment's pair case: its declarations + 1 (today declarations + 1 + 12 + facts). The engine's delta test covers a sole direction member and a sole fact member under each detaching operator, and the container's invariant and seed-alignment property tests (`group_tests.rs`, `container_tests.rs`) generate genes of both roles: fact-bearing genes, fact holes, fact-only survivors and their loaded-genome validation. `ComplexityEffect` labels are unchanged: they name an operator's effect on functional structure, which the size pressure gate restricts, not its unit delta. `cached_genome_size` equals `genome_size()` after every construction and mutation as today; `complexity()` is untouched. |
| 2 | **Fact members.** A fact member is a `GraphEdge` on `Drive(record.kind)` with `source = InputLeaf { ref_idx: shape.ref_idx, sub_idx }`, `sub_idx` one of the pair's declared `facts`, `membership = Some { group, scope }`, no new field: the role is read from the sink (`Drive` against a direction sink), so `Membership`, genome JSON, `GENOME_FORMAT` 6 and `PROTOCOL_VERSION` `v3alpha11` are unchanged. Its pattern is 0 by role (`pattern_at` returns 0 for a `Drive` site before any geometry; `Group::pattern` keeps its direction law for direction members), so its composed weight is its own weight, `AlterGroupScale` and the learned scale never move it, a scope-`Group` fact member credits `0 × Δ` and learns nothing (as a zero-pattern direction member, F04), while attribute broadcasts at `Group` scope (`scope_targets`) reach it as a member of the gene. Validation: `check_local_invariants` keys a `DirectionPair` member by `(sub_idx, site)`, admits a `Drive` site only for the record's kind and only on a `DirectionPair` record (a `Ring` member on a `Drive` sink is `MemberShape`), keeps the I10 sink order with the key `(GroupId, sub_idx)`; `validate_graph` requires a direction member's field in `{x, y}` and a fact member's in `facts`. |
| 3 | **Gene life cycle.** Creation: recruitment's `WholeFamily` pair case builds the gene through one container primitive: the record, the twelve direction members, then one fact member per `facts` field in `facts` order, every member born 0, static, scope `Edge`, no seed; the draw order (family, destination, pair) and draw count are unchanged; `hosts_pair_plan` stays the preflight and reads a container predicate `can_host_facts(kind)` (the kind's `Drive` sink present). Relocation: `RetargetGroupKind` moves fact members to `Drive(kind')` keeping own weight, plasticity, scope and seed; it draws its group and kind as today and, when the gene has a fact member and `can_host_facts(kind')` is false, skips with `NoApplicableTarget` under the engine's gated restore (`applies_to` unchanged, no new draw site; the funnel counts the node applicable and the event skipped; with the fixed sink catalog the skip never fires). `ShiftGroupOffset` leaves fact members on their sink. A `Group`-scope attribute broadcast selected on a direction member lands on the gene's fact members (pinned by a test). Deletion: `RemoveGroup` removes both roles; `RemoveGraphEdge` on a fact member leaves a fact hole; the record goes when its last member of either role goes (a gene may outlive its direction members). Detach: a fact member detaches **own-only**: `detach_at` leaves its `own_weight` bits untouched (no `compose`, so `-0.0` survives), leaves its seed entry exactly as stored (`None` stays `None` even when the group carries a scale seed; the composed-seed rule of F07 invariant 6 applies to direction members only), clears its membership, moves it to the independent prefix of its `Drive` sink, and it costs one unit from then on; the record goes when it was the last member. Seeds: `insert_member`, `take_members` and `detach_at` carry the seed row entry at the fact member's sink as for a direction member. Runtime: a fact member keeps its group term with pattern 0, so a static fact member whose group has a scale slot is a passive dynamic edge in the learned layout as a zero-pattern direction member is (one stored term and one composition per visit); a fact member with no scale slot in its group is a static edge. |
| 4 | **Supply and target law (intended consequence).** Supply stays `Binomial(genome_size(), per_unit_rate)` on the parent's size, read once before any event; every edge-targeting operator still picks uniformly among its eligible materialized edges, members included. So for a genome of `U` ungrouped units and `G` genes with `M` members in all, the expected events per birth fall from `rate·(U + G + M)` to `rate·(U + G)` while the target set is unchanged: a genome dominated by intact `n`-member genes draws about `n + 1` times fewer events for the same targets. Preserved per-member rates are not promised; the goal run reads the resulting supply (`mutation_events_attempted_total` per birth beside `mean_genome_size`). |
| 5 | **Founder.** `founder_genome(V3Alpha1)` carries no group (a test pins it), so `genome_size() == 80`, `FOUNDER_GENOME_SIZE_UNITS == 80`, ForageFirst 83, `per_unit_rate` `0.0060625` and `0.485` expected events per birth are unchanged; the recalibration branch of the track note is not taken. The replication multiplier, gestation and carry cost formulas are unchanged and read the new size. |
| 6 | **Readers.** `pair_groups` becomes `pair-groups-v2`: `members` keeps counting direction members present (the 40,000-tick criterion's reading), and the block and every row gain `facts` (fact members present) and `own_nonzero` (members of either role with `\|own_weight\| > PATH_WEIGHT_FLOOR`): reachable-pair readings, as the block is (unreachable pair genes and `Ring` members' corrections are not counted; the all-genome, both-shape counts belong to T27.F02's run, Notes); the fixture test covers a gene with facts and a fact hole. The block's version and fields change for every sample, zero-group samples included, so the summary schema moves regardless of mutation rate. `structure_means` (`Ring` only), `ring_rules`, the drive census (own weight on the `Drive` sink, no membership filter: a fact member is a drive edge at `\|w\| > 1e-3` as today) and the plasticity census are unchanged in definition. The goal summary's projection (`v3-cli/src/bench/artifacts.rs` keep-list) retains the block's `facts` and `own_nonzero` aggregates as it retains `members`, under the existing last-sample-only row policy, with a conversion fixture proving both survive. `graph_weights` lists fact members as members (genomic = own); the inspector's group card lists fact members on a "facts" line with own weights, no compass coordinate (`edgeGroupWeights.ts` already yields pattern 0 for a member without a sink direction). The lab masks a fact member as any leaf read. The direct `add_pair_group` callers in `v3-server/tests/server.rs`, `v3-lab/src/readings/tests.rs` and `v3-lab/src/ladder/track/tests.rs` follow the primitive's signature. `docs/progress/benchmark-series.json` records one `counter_definition_changes` entry for `genome_size` (before "edges only, group record 1", after "ungrouped edges only, group record 1, members 0") and one for `pair_groups` (`v1` → `v2`). Reference specs and the carry tooltip state the new rule (Inputs table). |
| 7 | **Identity and determinism.** Two commits, one cause each. Commit 1 (fact members): invariants 2, 3 and 6 except the size entry; the genome text of every recruited pair gene changes (its facts carry `membership`) and its size does not (F02 counts a member as an edge). Its one cause is fact membership and the gene life cycle: a picked fact member draws the scope coin, a `Group`-scope broadcast selected on a direction member now reaches the facts, `RetargetGroupKind` moves fact weights between drives, `RemoveGroup` and `DetachGraphEdge` gain targets, and members are inserted in canonical order after the independent prefix, so edge order and every mutation-on pin may move with that cause. Commit 2 (size rule): invariants 1, 4 and the size entry; every genome carrying a group changes size, so the supply draw, carry cost, replication multiplier, gestation and size-pressure roll move for that genome and every mutation-on pin re-pins with that cause. In both commits the founder-only viability digests do not move (invariant 5). No new RNG draw site anywhere (consumption moves on the paths above); no hot-path change beyond invariant 3's passive fact members: `genome_size()` runs at construction and after mutation, one membership test per sink edge. |

Options weighed, the preparatory refactor audit and the Codex rounds:
[the readings file](../../progress/readings/t25-f10.md).

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.tick.mean_genome_size` | gauge (existing; definition moves) | `cached_genome_size` under the unit rule of invariant 1 | `mutation.per_unit_rate`, `genome_carry_cost_per_unit` on `run.started` |
| `petri.run.genome_size_creature_ticks` | counter (existing; definition moves) | the carry charge's unit-ticks | as above |
| bench `pair_groups` (`pair-groups-v2`) | summary block, not a metric | invariant 6 | the goal summary's config identity |

No new metric: a group's unit count is not a bounded-key family, and the
bloat readings are summary blocks.

## Verification Plan

**Question.** F10 claims that a group costs one unit however many members it
has, that a pair recruitment's facts ride in its gene, and that the founder's
80 units and rate do not move; a founder size other than 80, a moved
founder-only digest, a gene whose members still count, or a fact member that
weighs through its scale shows it wrong. A gate counter `severe` is a
trajectory reading under the changed supply and follows the workflow's
acceptance rule; nothing is tuned, and no threshold or baseline is edited.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate + goal`, viability first (`cargo test -p v3-core --test viability` before the gate) | the track note names F10 "viability first, gate + goal": a production cost change whose 2,000-tick readings (`mean_genome_size`, supply per birth, `pair_groups` facts and `own_nonzero`, carry flows) answer whether the discount lands as predeclared; goal-long is spent on F02 and F04 and the bloat question is deferred to T27.F02. |
| Mutation gate | `run`: fresh `make rust-mutants-ci`, with local `MUTANTS_ITERATE=0 make rust-mutants` as the fallback when Actions is unavailable | new production logic in `v3-core` (the unit rule, the fact-member role, validation, relocation, detach) and `v3-cli` (`pair-groups-v2`) that tests must pin. |
| Slow tests (`make rust-test-slow`) | `run` | recruitment code and the per-unit supply change, which move the recruitment-paths recorded readings the tier pins. |

## Implementation Tasks

- [x] Commit 1 (fact members, `9d92ad1b`), tests first: invariants 2, 3 and 6 with audit R1–R5, R8 and R9 (member site, role-based pattern and validation, `can_host_facts`, the one-primitive gene build and its callers, relocation, own-only detach, both-role property generators, `pair-groups-v2` with keep-list and fixtures, `graph_weights`, the inspector facts line); mutation-on pins re-pinned with the fact-member cause.
- [x] Commit 2 (size rule, `61c5b8d1`), tests first: invariants 1 and 5 (`genome_size` unit tests for a `Ring` gene, a pair gene with facts, a hole, a detached member; `expected_delta`; the founder test), the `genome_size` series entry, reference specs and the tooltip; mutation-on pins re-pinned with the size cause.
- [x] Verification Plan as declared; epochs re-pinned in the closing commit.

## Verification

- [x] Focused tests and checks on the build commits (`61c5b8d1`), the self-review pass (`is_fact_member` replaces nine site tests; the broadcast pin `hebbian::tests::a_group_broadcast_from_a_direction_member_reaches_the_fact_members`) and the test-only review remediation (`GroupShape` strategy in `group_tests`): `cargo check --workspace --all-targets`, `cargo fmt --all --check`, `cargo test -p v3-core --no-fail-fast` (lib 2,221, 9 ignored slow), `cargo test -p v3-cli -p v3-server -p v3-lab -p v3-telemetry --no-fail-fast`, inspector and config-panel `vitest` (357) and `biome check` all pass; every moved pin re-pinned with its cause, listed with the lineage search in the [readings](../../progress/readings/t25-f10.md#fixture-re-pin-searches).
- [x] Mutation gate (fresh `make rust-mutants-ci`, run 39573015727, `210228ab` vs `ced01959`): `4 shards, 81 mutants tested: 0 missed, 69 caught, 12 unviable, 0 timeouts`; output `~/.local/share/petri-tools/mutants/t25-f10/mutants.out`; no survivors ([readings](../../progress/readings/t25-f10.md#mutation-gate-2026-10-21)).
- [x] For each declared benchmark profile (gate, goal; 2026-10-21, HEAD `474faf70`): summary stored at
      `docs/progress/features/<id>.json` (gate), `<id>-goal.json` or
      `<id>-goal-long.json`, local raw hash/byte count and verification time
      checked, series entry points to the summary, and no new full report
      staged. Otherwise `Not run: Verification Plan`. Summaries `t25-f10.json` and
      `t25-f10-goal.json`, hashes, bytes, exit statuses and verdicts in
      [Performance and Goal Impact](#performance-and-goal-impact); series `closed`
      entries added, gate and goal-worlds epochs re-pinned to them at closure, no raw report staged.
- [x] Viability first: `cargo test -p v3-core --test viability` before and after each commit -> 30 pass, founder-only digests unchanged; re-run at the final code `474faf70` before the gate (2026-10-21): 30 pass.
- [x] Slow tier: `make rust-test-slow` at `7a23edc3` (2026-10-21) -> exit 0, 3 + 6 pass; the recruitment-paths readings held, so none was re-pinned.
- [x] `make check` at the tested commit `7a23edc3` (2026-10-21) -> exit 0.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: the genomic
bottleneck (Goal); it reaches creatures through the body's size-priced
costs and the per-unit supply. The founder has no group, so every
mutation-off simulation trajectory and the founder-only viability digests
are identical; the goal summary's `pair_groups` schema moves regardless of
rate (invariant 6).

Compute: no per-tick or per-birth work is added; `genome_size()` gains one
membership test per sink edge at construction and after mutation; a static
fact member whose group has a scale slot is a passive dynamic edge
(invariant 3: one stored term and one composition per visit of its node), a
carrier-dependent cost that exists only where a recruited gene also carries a
`Group`-scope learner. Gate `wall_clock_ms_per_creature_tick` (F09 epoch) is
predeclared within the 25% flag threshold, expected within ±5% as a
hypothesis; a `flag` is reported with its mechanism; a `severe` on it is a
defect.

By construction, for the same genome: a gene of `n` members loses `n` units,
so its carry charge per tick falls, its replication multiplier and gestation
are non-increasing (gestation rounds up and may stay equal), and its expected
supply falls by `rate·n`. Across runs these are hypotheses, not identities:
the population reached under a different supply and selection history is
not bound by them. Hypotheses: `mean_genome_size` and
`mutation_events_attempted_total` per birth below F09's in every world where
genes are present at tick 2,000. The `genome_size` series entry marks the
definition change, so cross-definition deltas are not like-for-like. Every
per-seed counter moves with the draw stream (commit 1's life cycle, commit
2's size) with no predeclared direction; a `severe` there is a trajectory
reading that blocks closure until the user accepts it or remediation removes
it (workflow, "Benchmark gate"). When the user accepts, the gate and
goal-worlds epochs are re-pinned in the closing commit.

Goal (2,000 ticks), per world, including zero, from the goal summary:
`mean_genome_size` and `mean_ring_groups` against F09; `pair_groups`
`groups`, `members`, `facts`, `own_nonzero` and the per-row counts;
attempted and applied mutation events per birth; the carry flow. The
hypotheses above are descriptive and not thresholded; the number of genes
per creature is a reading, not a threshold; a fact hole or a detached fact
member appearing is a reading. The long-run bloat question
(cheap silent genes accumulating, free per-member variation) stays open
(Notes).

**Measured verdict (2026-10-21, HEAD `474faf70`, each command run once).** No `severe` on either
profile; no counter flagged on the gate; two goal counters flagged; one predeclared hypothesis
contradicted in one world.

| Command | CLI exit (source) | Outer exit (observed) | `severe` | Verdict |
| --- | --- | --- | --- | --- |
| `cargo test -p v3-core --test viability` | not applicable | 0 | not applicable | 30 pass |
| `make bench PROFILE=gate FEATURE=t25-f10` | 0 (`measurement_evidence.cli_exit`) | 0 | false | wall flag |
| `make bench PROFILE=goal FEATURE=t25-f10` | 0 (`measurement_evidence.cli_exit`) | 0 | false | `mesh_hops` +10.45% and `plasticity_updates` +17.09% flagged |

| Reading (F10 vs F09) | Orchards | Canyon | Confluence | Against the predeclaration |
| --- | --- | --- | --- | --- |
| gate wall per creature-tick | +31.72% (quiet-host re-run, level flag; first run +17.56%) | | | contradicts the +/-5% hypothesis, above the 25% flag, not fatal; readings |
| goal `mesh_hops` / `plasticity_updates` per creature-tick | +4.3% / -10.7% | +4.0% / +83.3% | +21.6% / +72.9% | aggregate +10.45% / +17.09% flagged |
| `mean_genome_size` at tick 2000 | 99.88 vs 103.26 | 94.26 vs 97.66 | 120.60 vs 110.64 | lower in two worlds; Confluence higher, contradicts |
| attempted events per birth | 0.5464 vs 0.5613 | 0.4945 vs 0.5032 | 0.6210 vs 0.5834 | same split |
| `mean_ring_groups` | 0.482 vs 0.410 | 0.330 vs 0.076 | 0.758 vs 0.296 | a reading |
| `pair_groups` groups (F09) | 176 (283) | 0 (30) | 734 (229) | a reading |
| `facts` / `own_nonzero` | 141 / 0 | 0 / 0 | 115 / 9 | a reading; zero read in Canyon |
| `genome_carrying` energy (F09) | 98,977 (104,381) | 18,472 (18,647) | 75,048 (71,241) | per-unit rate unchanged |

The nine gate counters are all ok. The goal flags sit where reachable pair genes rose most
(Confluence), fitting the carrier-dependent cost without isolating it. Goal wall clock per
creature-tick rose +76.33% (flag) in every phase, `food_growth_ms` included, under a host load of 19
to 23 on 8 cores with sibling test runs: host-confounded. The summary carries no fact-hole or
detached-fact-member count.

**Rulings (spec owner, 2026-10-21; reasoning in the readings).** The Confluence rise is a
trajectory reading, not a contradiction: every falsifier of the claim holds, the cross-run direction
was a hypothesis, and the genes carried (734 pair, 0.758 ring per creature, three-fold F09's) account
for about one unit of the ten-unit rise; it is the first short-horizon reading of the track's bloat
risk, carried to T27.F02 (Notes). Not a blocker; no acceptance needed. The goal wall +76% is a non-fatal flag with its host-load
mechanism recorded; not re-run (once per closure; counters deterministic). The gate wall is a flag
(+31.72% on the quiet-host re-run, counters byte-identical to the loaded first run and below F09)
whose mechanism is not isolated, and the predeclaration requires a flag's mechanism, so one paired
attribution reading is taken before the mutation gate: the merge base `ced01959` (F09 code) built in
a scratch worktree and the final code run back to back on the same host, two alternating gate pairs,
the median F10/F09-now ratio recorded in the readings. At or below 1.25 the predeclaration holds
(the excess over the stored F09 is host drift), the flag is recorded with that mechanism, closure
proceeds and both epochs re-pin to F10's summaries in the closing commit without user acceptance, as
every T25 closure has done; above 1.25 it is an unpredeclared compute cost on a feature that adds no
per-tick work, a defect to find before closure (user informed). Reading taken: ratio 0.998, host drift.

- Summaries of the declared profiles: [gate](../../progress/features/t25-f10.json),
  [goal](../../progress/features/t25-f10-goal.json); raw paths, hashes and byte counts in the readings.
- Full readings: [`docs/progress/readings/t25-f10.md`](../../progress/readings/t25-f10.md#benchmark-readings-2026-10-21-head-474faf70-tree-clean-each-command-run-once).

## Success Criteria

- [x] `genome_size()` charges one unit per group record and nothing for a member of either role, a hole or runtime state, and a detached member one unit, pinned by unit tests and the engine's per-operator delta test (invariant 1).
- [x] Recruitment's pair case builds one gene of direction and fact members through one container primitive with an unchanged draw order; fact members validate, relocate, delete, detach and carry seeds inside the container, weigh their own weight only and cost nothing (invariants 2–3).
- [x] The founder has no group, measures 80 units at rate `0.0060625`, and the founder-only viability digests are unchanged in every commit (invariant 5).
- [x] `pair-groups-v2` reports `facts` and `own_nonzero`; the server, inspector and reference specs state the new rule (invariant 6).
- [x] The Verification Plan ran as declared; the goal readings are recorded in the readings file per world, including zero; every re-pin names its cause (invariant 7).

## Notes for AI Agents

- Decision: (user, 2026-10-20, track Notes) a group record costs one unit and its members nothing more, whatever their corrections, learning configuration, scope or retention; no template or exception tracking; labelled an explicit economic discount, not compression (T26–T28 review round 5 finding 3).
- Decision: a fact member's role is read from its sink, not stored; its pattern is 0 by role; it is created only by recruitment's pair case (the track's "ordinary slot-fact wiring outside a recruitment is unchanged").
- Decision: `ComplexityEffect` labels are unchanged; the size pressure gate restricts by functional-structure direction, and `DetachGraphEdge` stays `Neutral` although it adds one unit.
- Decision: the founder's anchor and rate are remeasured, not recalibrated (track Notes: the founder has no group).
- Deferred: the long-run bloat reading (genes per creature, materialized members, members with nonzero own corrections, compute and energy) goes to T27.F02's goal-long run, which names every feature closed in between; `pair-groups-v2` carries the reachable-pair counts, and the all-genome, both-shape counts (unreachable genes, `Ring` members' corrections) are that run's to add. Its 2,000-tick baseline is F10's goal: Confluence 734 reachable pair genes and 0.758 ring genes per creature against F09's 229 and 0.296, `mean_genome_size` 120.60 against 110.64, `own_nonzero` 9.
- Deferred: `mutation/engine/mod.rs` `group_edits` and `mutation/graph/mod.rs` `GraphOperator::random` keep their `WORKAROUND(T25.F08)` markers unchanged; fact members are counted by their membership as any member, so the tally stays correct, neither workaround is extended, and F10 writes no `WORKAROUND(T25.F10)` marker (audit R6, R10; R7 is a test oracle, not a workaround).
- Cost: `/usage` totals not recorded (user command, not run in the session); implementer passes 3 (build, self-review, test-only remediation), advisor consults 2, 2, 2; spec-owner resumes after Plan 3; Codex challenge rounds 2, final `ready`; Codex review findings: correctness 0, design 0, code health 1 P2 (remediated).
