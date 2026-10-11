# T11.F31 — Ring Projection Motif

**Status**: In Progress
**Last updated**: 2026-10-11
**Feature**: T11.F31
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

A topographic map from one developmental program, as central-complex columns
repeat one circuit around the heading ring and Eph/ephrin gradients wire a whole
retinotopic map. A Graph node carries ring projections: one gene names a
neighbor ring the node declares, one directed action kind (`Move`, `Reproduce`
or `StealEnergy`), a rotation offset and one weight, and adds
`weight · ring[(d − offset) mod 8]` to that kind's direction-`d` vote for all
eight directions. A new operator adds a projection born silent at weight 0;
its weight, offset, kind and existence then change only as a unit, so a rule to
seek, avoid or aim in every direction arises, shifts or is lost in one event,
while per-direction edges remain independent exceptions. The intended outcome,
read by the goal profile in three worlds, is projections present and carrying
weight at tick 2,000, born silent, with harmful births per birth not up and
persistence above its floor; ring rules are read through the bus.

## Non-Goals

- No kind drive, no scalar fan-out to a direction group, no change to the
  commit rule or `vote_select` (T11.F30); no direction group shared across
  kinds; no projection from area summaries, nearby-creature or other non-ring
  families.
- No change to `InputRef.Add`'s born-connected edge, the founder, the sink
  catalog, or any existing operator's draws, weight or code path (the Graph
  domain's modulus changes).
- No VM opcode (Exception in Notes); VM nodes are unchanged.
- No configuration field or switch, no T22 lab, no factored founder port (T18).
- No lineage-tracking instrument: retention is read cross-sectionally, as the
  note's census reads it. No compute-node path tracing in the ring-rule census.
  The T20.F01 opportunity reading, T13 recruitment-path fixtures and `v3-lab`
  readers are not extended to projections (struct literals only).

## Inputs and Invariants

Sources of truth: the F31 row and the "Factored motor output, 2026-10-11"
Notes entry with its sub-bullets and its closing paragraph (readings trace ring
rules through the bus) in the T11 roadmap; the
[factored motor output note](../../strategy/factored-motor-output-research-2026-10-11.md)
§§4, 5, 7 and 8 (user decisions, E5's 1.45× harmful-birth ratio, rows); the
graph backend spec §6 zero-vote contract; the
[T20.F16](t20-f16-born-connected-input-declaration.md) and
[T19.F06](t19-f06-retirement-and-observability.md) specs (born-connected
`Add`; one execution model with traced, observed and production agreeing).
Code: `creature/genome/vote.rs` (27 sinks; `Move(d)` at `1 + d`,
`Reproduce(d)` `9 + d`, `StealEnergy(d)` `17 + d`), `creature/genome/cgp.rs`,
`runtime/cgp/effects.rs`, `mutation/graph/{mod,operators,recruitment,refinement}.rs`,
`mutation/input_ref/mod.rs`, `creature/genome/mod.rs` (`genome_size`). The
previous closure and both epochs are `effort-cost-defaults`. Planning numbers
are in the [readings](../../progress/readings/t11-f31.md).

| Question | Decision | Alternatives and why not |
| --- | --- | --- |
| Representation | `CgpGraphBackendDef.projections: Vec<RingProjection>`, `RingProjection { ref_idx: u16, kind: Move \| Reproduce \| StealEnergy, offset: u8, weight: f32 }`; serde default empty and omitted when empty; `PartialEq` and `Debug` include it (`Debug` may omit an empty list). Not an edge of any sink | Eight weight-tied sink edges: every edge operator would need group awareness and could split the group, the per-direction loss the row removes; a new `GraphSource` is still eight edges; the sink catalog is fixed |
| Size | One `genome_size()` unit per projection; a projection wires no sink | Eight units charge the motif as eight edges and remove the one-gene economy of the user ruling; the size law counts heritable components, the ring's declaration counted separately |
| Adding operator | `AddRingProjection`, dispatch weight 2 (`AddGraphEdge`'s tier), writes weight 0.0 | A nonzero birth weight: user decision (E5 `aligned`, weights up to 1, raised harmful births 1.45× on wall) |
| Unit operators | `AlterRingProjectionWeight` 4, `ShiftRingProjectionOffset` 2, `RetargetRingProjectionKind` 2, `RemoveRingProjection` 2: each the tier of the edge operator it parallels | Weights normalized by site count: no existing operator does so; one combined operator loses per-operation attribution in `mutation_effects` and carrier outcomes |
| Weight law | `AlterGraphEdgeWeight`'s: `\|w\| > 0.01` → `w · (1 + U[−0.2, 0.2])`, else `w + U[−0.1, 0.1]` | A new law is untested; this is the row's "refines a projection through its own weight mutation" |
| Born-connected `InputRef.Add` ring landing on a direction sink | Stays one ordinary edge at the new-edge law's weight: T20.F16 unchanged, draw for draw, so `Add`'s per-birth harm is unchanged by construction | A zero-weight projection retracts F16's live-at-birth wiring for the ring families (`NeighborBarrierRing` is an F16 target row); a projection at the drawn weight contradicts the zero-birth-weight decision and is E5's harmful shape |
| `RecruitNeutralInput` ring case (Off by default) | Ring family and a directed destination `K(d)`: declare as today, then append one projection `{ref_idx, K, (d − channel) mod 8, 0.0}` instead of the edge or column (`channel` 0 under `WholeFamily`); no new draw; other families and destinations unchanged | Keeping the column contradicts the user decision sub-bullet |
| `RefineHeritableStructure` | Unchanged: `groups()` reads `ActionVote` sink edges only, so a projection is never a group member; its refinement is `AlterRingProjectionWeight` | Group-aware refinement of projections duplicates one weight operator |
| VM parity | Recorded T11.F18 exemption (Notes); a VM node reaches projections by `SwapNodeBackend` to Graph | An opcode: 0 of 1,240 reachable live VM nodes vote (note §8), and the VM has no per-instruction weight, so it would be a new design |
| Switch | None; attribution is the comparison with the previous closure, as at T20.F16 | An opt-in flag the track's disposal rule discourages |

| Boundary | Required behavior |
| --- | --- |
| Ring | A reference is a ring when it is `World(NeighborFoodRing { .. } \| NeighborBarrierRing \| NeighborOccupiedRing)`: width 8, sub-value `c` is `Direction::ALL[c]` (N, then clockwise), the same index as the vote direction. |
| Evaluation | In the effects pass of an entered visit, for each directed kind `K` and direction `d`: `P_K[d] = Σ w_p · r_p((d + 8 − k_p) mod 8)` over `K`'s projections in genome order, `r_p(c)` the value `InputLeaf { ref_idx, sub_idx: c }` resolves to; a projection whose `ref_idx` is out of range or not a ring contributes 0, and `k_p` is read mod 8. Sink `K(d)`'s contribution is `sanitize(S + P_K[d])`, `S` the sink's edge sum as today (0.0 without edges); a sink with neither stays 0 and unapplied. One implementation serves production, observed and traced modes; a traced sink record carries the applied value including projection terms. An exhausted visit stages nothing. RNG-free. |
| Cost | No per-projection energy charge (the graph charge stays per compute node). `enters_visit()` is true when a projection exists, so a projection on a dormant graph pays one node-equivalent per visit, as F02's first zero vote edge does. Host work is eight reads and multiply-adds per projection per entered visit. |
| Zero weight | For finite ring values and equal native context, a weight-0 projection leaves the visit's vote contribution, output slots, parameters, shared memory, route scores and carried state equal (exact numeric equality, signed zero allowed); dormant activation is its only effect. |
| Equivalence | On a kind with no sink edges, a projection `(w, k)` contributes exactly what eight edges `InputLeaf { ref_idx, (d − k) mod 8 }` → `K(d)` at weight `w` would. |
| `AddRingProjection` | Applies to a Graph node with at least one ring at an index ≤ 65,535. After node selection, three draws in order: the ring uniformly over those indices ascending; kind `gen_range(0..3)` over `[Move, Reproduce, StealEnergy]`; offset `gen_range(0..8)`. Pushes `{ref_idx, kind, offset, 0.0}`; duplicates are allowed and each owns its weight. `Increasing`. |
| Unit operators | Apply to a Graph node with at least one projection. First draw: the projection, uniformly. Then `Alter`: the weight law; `Shift`: `gen_bool(0.5)` chooses +1 or −1, mod 8; `RetargetKind`: `gen_range(0..2)` over the other two kinds in catalog order; `Remove`: `Vec::remove`, order kept. `Alter`, `Shift`, `RetargetKind` `Neutral`; `Remove` `Decreasing`. No operator changes part of a projection or its `ref_idx`. |
| Catalog | Appended to `GraphOperator::ALL` after `RefineHeritableStructure` in the order of the decision table; keys `Graph.AddRingProjection`, `Graph.AlterRingProjectionWeight`, `Graph.ShiftRingProjectionOffset`, `Graph.RetargetRingProjectionKind`, `Graph.RemoveRingProjection`; default-arm total weight 44 → 56; restricted size pressure admits only `RemoveRingProjection` of the five. Existing operators' code paths and the founder battery's per-family seeds are unchanged. |
| References | `referenced_indices` counts every projection `ref_idx`, so `Prune` never deletes a projection's ring; `reindex_input_refs_after_removal` decrements projection references above the removed index and removes a projection equal to it, as for edges; recruitment admission treats a dangling projection reference as a dangling leaf. `Swap` and `RawFieldMutation` keep a ring a ring of the same class (`input_ref_kind`). |
| Copies | Cloning, `CopyNode`, slice copies and serde carry projections; `SwapNodeBackend`'s blank backends carry none. |

| Reader | A projection counts as |
| --- | --- |
| `genome_size()` | one unit |
| Functional complexity (reachable nodes) | one live component; its ring among the consumed references |
| Mesh annotations | its ring's read class; write class `Action` |
| Sensor census (persistence samples) | a consumer of its ring on the live walk |
| Input-use funnel | a consumer of all eight channels of its ring, live on a reachable node, read whenever its node's effects pass runs; ablating one channel zeroes that channel's term in every projection that reads it, ordinary consumers of that channel are ablated as today, and unrelated channels, consumers, weights and `genome_size()` are unchanged; any ablation-only state stays out of genome identity, serialization and size |
| `steering-v1` `move_voted` | a `Move` projection votes `Move` |
| `mutation_effects` genome identity | distinguishes projections |
| Creature dump and inspector | `GraphBackendDef.projections?` in `frontend/src/types/genome.ts`; the Graph node detail lists each projection (ring, kind, offset, weight); layout is the implementer's |

The ring-rule census (`ring-rules-v1`) is the note's Appendix A census made a
goal-profile reading, with projections added:

| Term | Definition |
| --- | --- |
| Scope | Every living creature at the goal's last persistence sample (tick 2,000); its reachable Graph nodes only (no reachable live VM node votes, note §8). |
| Rule | `(family, kind, offset, sign)`: a ring's sensor census key (e.g. `NeighborFoodRing:0`), a directed kind, an offset in 0..8, a sign. |
| Path for direction `d` | On a reachable node, an `ActionVote` edge into `K(d)` with `\|w\| > 1e-3` whose source is (i) `InputLeaf` of a ring of `family` at channel `(d − offset) mod 8`, sign of `w`; or (ii) `InputLeaf` of `UpstreamSlot(j)` while some reachable Graph node has an edge with `\|w'\| > 1e-3` into `CustomOutput(j)` from that ring channel, sign of `w · w'` (one bus hop; execution order and overwrites ignored, so an upper bound, as in the note); or (iii) a projection `(family, kind, offset)` with `\|w\| > 1e-3`, which is a path for every `d`, sign of `w`. Compute-node paths are not traced. |
| Coverage | The number of directions of a rule with at least one path; the founder's `NeighborFoodRing:0 Move 0 +` covers 4 (N, E, S, W through slots 2 to 5). |
| Fields | `creatures`; `direction_general` (some rule covers at least 6) and `direction_general_edges_only` (at least 6 without a projection path); `projection_carriers`, `projections`, `projections_nonzero` (reachable nodes) and `projections_unreachable`; `projection_rows` `family kind offset count positive negative`; `rule_rows` `family kind offset sign n1 … n8` (creatures by coverage 1 to 8; rules no creature covers omitted). |

**Determinism.** New draws occur only in the mutation stream, in the pinned
orders above. Every pin that covers a mutated birth drawing a Graph-domain
operator moves once and is re-pinned with its reason in the readings: the
default-rate `applied_trajectory` digests (sampled and guard), the default
`legacy_default_short_run_identity`, drift-walk count pins, `recruitment_paths`
accepted seeds (re-seeded, plans unchanged), `experiment.rs` lineage counts,
mutation-engine sequence pins and catalog counts (`ALL` 23 → 28,
`TOTAL_WEIGHT` 44 → 56, the `MutationOperator` list), and tiny-fixture seeds
whose population no longer reaches a horizon (extinction ticks before and after
in the readings). Unchanged: founder-only and rate-0.0 pins, the founder
digest, `FOUNDER_GENOME_SIZE_UNITS` (97), every config digest, the founder
battery rows of existing operators.

**Natural analog.** Central-complex columnar types tile the heading ring with
one repeated phase shift (Hulse et al. 2021) and a few Eph/ephrin gradients
wire a retinotopic map (Feldheim and O'Leary 2010): one program places a whole
map. The offset is that phase shift, the shared weight its gain, the zero birth
weight a program expressed before selection tunes it, and per-direction edges
the activity-refined exceptions (HybrID, Clune et al. 2011). The ruling
overrides not-forcing check 3 and the T20 weight-tying sentence for the three
neighbor rings only.

## Telemetry

No metric family, counter or configuration is added; existing families gain
keys and the goal summary gains one block.

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.mutation_events_applied`, `_attempted`, `_skipped`, `petri.run.mutation_operator_funnel` by operator | counters, existing families, five new `Graph.*` keys | applied and attempted projection events | `run.started` body with the unchanged `mutation` config |
| `cases[].ring_rules` (`ring-rules-v1`) | goal summary block, new | the ring-rule census over the terminal population (Inputs) | case `config_digest`, unchanged |
| `cases[].input_use`, persistence `sensor_census`, `steering_pooled`, `mutation_effects`, `neighborhood_read`, `drift_depth` | existing blocks | projections enter as consumers and writers per the Reader table | same |
| Inspector Graph node detail | existing view | the creature dump's `projections` | none |

## Implementation Tasks

- [x] `cargo test -p v3-core --test viability` passes before any edit and after
      every production edit.
- [x] Failing tests first for the Boundary and Reader tables, with proptest
      properties in v3-core for zero-weight equality, eight-edge equivalence,
      one-step rotation by `Shift`, one size unit per projection, `Prune` and
      reindex keeping each projection on the same `InputReference`, and serde
      round trip with omission when empty; each operator's draw order pinned; one
      `AlterRingProjectionWeight` on a zero-weight `Move` projection over a food
      ring moves the `Move` vote in every direction holding food; a nonfinite
      weight or ring value is sanitized with the sink's edge sum, as an edge
      term would be; duplicate projections and incumbent edges sum in the
      stated order; refinement groups identical with and without projections;
      `InputRef.Add` and VM paths draw for draw unchanged.
- [x] Genome type, serde, identity and size; evaluation, `enters_visit` and
      traces.
- [x] The five operators, catalog and keys; the `RecruitNeutralInput` ring
      case; reference coverage (`Prune`, reindex, admission).
- [x] Every reader in the Reader table.
- [x] `ring_rules` in the goal profile and the summary projection, with a
      founder fixture (one rule, `NeighborFoodRing:0 Move 0 +`, coverage 4)
      and fixtures for each path kind and for a projection.
- [x] Frontend type and Graph node detail.
- [x] Reference documents: graph backend spec (data model, §6, effects,
      liveness, soft defaults), mutation spec (catalog, `Add` unchanged),
      runtime config spec (`genome_size()` units).
- [x] Re-pins of the Determinism paragraph, each with its reason, in the
      readings.
- [ ] `make check` exits 0 in the worktree.
- [ ] Gate and goal profiles by the benchmark specialist, then the mutation gate
      by the mutation specialist, sequentially and alone.

## Verification

- [x] `cargo test -p v3-core --test viability` -> exit 0 (28 passed), before
      any edit and after the last production edit (the self-review pass).
- [x] Focused tests -> names and counts in the
      [readings](../../progress/readings/t11-f31.md). On the current tree:
      `cargo test -p v3-core` (2,069 passed), `cargo test -p v3-cli
      ring_rules` -> exit 0; `cargo check --workspace --all-targets`, `make
      rust-clippy` and `cargo fmt --all --check` clean. On the build commit
      `cf7fa109`: `cargo test -p v3-cli`, `cargo test -p
      v3-server -p v3-lab -p v3-telemetry` -> exit 0; `npm run lint`, `npm
      run test` (380 passed) and `npm run build` in `frontend/` -> exit 0.
- [ ] `make check` -> exit 0, tested commit in the readings.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      every survivor resolved as killed, equivalent, or deferred.
- [x] Benchmark summaries stored at the paths below, series entries added
      (`gate.closed`, `goal_worlds.closed`), raw hashes and byte counts in the
      readings, goal run once, no full report staged. Exit statuses, `severe`
      false on both, and the two missed predeclared targets are in the
      measured verdict above.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: one
developmental program wiring a whole topographic map, reaching creatures
through genome mutation at birth and the existing vote surface. Expected
compute cost: none worth a budget, eight reads and multiply-adds per
projection per entered visit; no severe budgeted for compute. Supply: about
0.54 attempted events per birth at the previous closure, 0.8 × 1/3 of them in
the Graph domain, and `AddRingProjection` about 2 of roughly 32 applicable
weight there, so about 1% of births add a projection. Costs carried by
creatures: one size unit per projection (carrying, gestation, replication,
requested supply). The Graph-domain modulus moves from 44 to 56, so every
mutated birth's Graph draw remaps and `deterministic` differs in both profiles.
References: `effort-cost-defaults` is the gate and goal epoch and the previous
closure. Thresholds: counters flag at +10%, severe at +50%; wall flags at +25%,
severe at +100%. Expected re-pins: the test pins of the Determinism paragraph;
an epoch only on a severe the user accepts. Every indicator not in the table is
reported with no direction. No environmental pressure is added, so the
three-world integration rule does not apply.

| Reading | Reference (Orchards 11 / Canyon 22 / Confluence 33) | Predeclaration |
| --- | --- | --- |
| Goal persistence | final 4,283 / 2,354 / 4,213; plateau 4,536.27 / 2,500.07 / 3,883.59; minimum 3,702 / 342 / 586 | **Floor:** no world extinct; final and plateau each at least half the reference on every world. A breach is a blocker for the user, never rebased. No direction expected. Minimum reported. |
| `ring_rules`: projections | none exists before this feature | **Fingerprint:** `projection_carriers`, `projections` and `projections_nonzero` each above 0 on every world; a world missing one is a contrary result for the user. Carrier share supply-limited, no direction. **Selection test** (note §8): offsets of nonzero projections reported against the uniform 1/8 they are born with, and their signs against the 1/2 of the first weight step from zero, no direction (a screen at generation about 47). |
| `ring_rules`: direction generality | none (first reading); live world 0 of 200 at generation about 345; founder covers 4 | Descriptive (note §8: supply alone can meet it): `direction_general` and `direction_general_edges_only` reported, no direction. |
| `ring_rules`: retention | none (first reading); live world median 1 of the founder's 4 at generation about 345 | Descriptive: coverage histogram and median of `NeighborFoodRing:0 Move 0 +` reported, no direction; a projection is kept or lost whole by construction. |
| **Harmful per birth:** `neighborhood_read.dead_per_all_births` (5,000 births of the selected parents) | 0.0040 / 0.0110 / 0.0062 | **Not up:** at or below the reference plus two binomial standard errors of a 5,000-birth read, `2√(p(1−p)/5000)`: 0.0058 / 0.0140 / 0.0084 (the read moved 0.0008 to 0.0110 over the last four closures with no harmful mechanism). A world above goes to the user. `InputRef.Add` is unchanged, so its share of this is unchanged by construction. |
| `neighborhood_read.changed_per_all_births` | 0.1568 / 0.1608 / 0.1668; standing floors 0.1464 / 0.1300 / 0.1434 | Flat or slightly down (a silent operator takes about 6% of applicable Graph weight); not below the standing floor on any world, a breach goes to the user. |
| Founder per-operator battery (raw) | previous closure's rows | Existing operators' rows identical; `AddRingProjection` silent 1.00; the four unit operators have no founder site. |
| `mutation_effects.cohorts[].operators` (founder, drift and selected cohorts) | none for the new keys | `Graph.AddRingProjection`: `action_changed` and `action_dead` 0 (born silent); a nonzero count is checked against the dormant-graph charge before it is reported as contrary. Unit operators' `action_dead` per proposal reported beside `AlterGraphEdgeWeight`, `RetargetGraphEdge` and `RemoveGraphEdge`, no direction. |
| Carrier outcomes by operator (raw `mutation_value_totals_by_operator`) | detrimental shares in the readings | Diagnostic (note §8 "helpful share of projection births against ordinary edge births"): helpful and detrimental shares of each projection operator reported beside `AddGraphEdge`, `AlterGraphEdgeWeight`, `RetargetGraphEdge` and `RemoveGraphEdge`, no direction; lifetime outcomes credit every operator a carrier was born with, so this is not the harm measure. |
| Drift walk `changed_per_all_births`, depths 0 / 22 / 250 / 1,000 / 2,000 (2,000 births each) | Orchards and Confluence 0.1555 / 0.1145 / 0.0355 / 0.0165 / 0.0090; Canyon 0.1500 / 0.1080 / 0.0195 / 0.0165 / 0.0070 | No reliable direction: the walk reads the mutation map alone, which this feature remaps and gives a silent operator. **Predeclared regression bound:** not below the reference minus two binomial standard errors, `2√(p(1−p)/2000)`: Orchards and Confluence 0.1393 / 0.1003 / 0.0272 / 0.0108 / 0.0048, Canyon 0.1340 / 0.0941 / 0.0133 / 0.0108 / 0.0033. A reading below goes to the user with the walk's dead and executed rows. |
| Input-use ring family rows, selected and drift cohorts | selected `NeighborOccupiedRing` declared / connected 13 / 2, 10 / 2, 4 / 0 | Reported, no direction; the census is the fingerprint. |
| `steering_pooled` (raw) exact / within-45 / avoidance | 0.3894 / 0.6416 / 0; 0.4449 / 0.6949 / 0; 0.4150 / 0.6675 / 0 | **Target** (note §8): exact-hit above 0.37 and avoidance above 0 on every world. Expected: exact-hit met (the reference already is); avoidance likely 0 at tick 2,000, since projections are born at zero weight, few carry a weight step by then, and few vote nodes declare a barrier ring. A world missing either is a contrary result for the user, not absorbed. Within-45 reported. Gate founder `steering-v1` and `mesh_execution` identical to the reference. |
| Goal `mean_genome_size`, `mean_mesh_nodes` at tick 2,000 | 118.26 / 125.57 / 123.50; 3.80 / 3.55 / 3.57 | No reliable direction. |
| Gate (75 ticks, founders), per creature-tick | `effort-cost-defaults` gate: `vm_steps` 0.095108, `pass_cap_hits` 0.002030, `plasticity_updates` 0.026053, `decided_passes` 0 | `deterministic` differs. `actions_applied`, `births`, `graph_relax_iters`, `mesh_hops`, `passes` within flag. `vm_steps`, `pass_cap_hits`, `plasticity_updates` and `decided_passes` are small-base counters a remap alone can push past flag or severe (`decided_passes` reads severe against its zero reference if any pass decides, as at T20.F16); such a severe goes to the user with per-seed values as remap movement, a gate-epoch re-pin requested. A severe on another counter does not fit. |
| Goal work counters, per creature-tick | `effort-cost-defaults` goal | No compute severe budgeted. `vm_steps`, `plasticity_updates`, `decided_passes` and `actions_applied` have read severe from clade composition before (T03.F12, T19.F05, T20.F16); a severe here is read against the per-world evidence and, if composition, reported for the user's decision on acceptance and re-pin, not attributed to compute. |
| Wall per creature-tick, observation budgets | previous closure; workflow caps | Wall flat within flag; a flag reported with host and phase. Budgets as at the previous closure; the census adds one pass over the terminal population. |

Run sequentially after benchmark-affecting work is final, without competing
builds, tests, servers or measurements:

```sh
make bench PROFILE=gate FEATURE=t11-f31-ring-projection-motif
make bench PROFILE=goal FEATURE=t11-f31-ring-projection-motif
```

**Measured verdict (2026-10-11, code `329637b3`).** Both profiles ran once, sequentially.
`make bench PROFILE=gate` and `PROFILE=goal`: outer exit 0 (observed by the
benchmark specialist from the shell; summaries' `outer_exit` is `null`), CLI
exit 0 (summary `measurement_evidence.cli_exit`, source: v3-cli status on
successful artifact-pair completion). Gate: `severe` false, no counter flag.
Goal: `severe` false; flags `vm_steps` +13.75% and `decided_passes` +18.03%
only. Raw (ignored, main checkout `.bench-artifacts/t11-f31-ring-projection-motif/`):
gate 105,028 bytes (`08204beb...`), goal 681,246,074 bytes (`00be755a...`).
Summaries: gate 19,839 bytes (`d79ba8c1...`), goal 392,679 bytes (`0a847496...`).
Full hashes, counters, wall readings and every predeclared reading are in the
readings. Predeclared targets **missed**, both accepted by the user below:
(1) Orchards `neighborhood_read.changed_per_all_births` 0.1432, below its
standing floor 0.1464 (Canyon 0.1348 and Confluence 0.1594 are above their
floors); (2) steering avoidance 0 on all three worlds (exact-hit 0.4407 /
0.4395 / 0.4205 met). The founder per-operator battery (raw gate
`mutational_neighborhood.founder.operator_rows`) is met: all 56 existing rows
identical to the reference, `AddRingProjection` silent 50 of 50, the four unit
operators skipped 50 of 50; founder `steering` and `mesh_execution` are
identical in the raw gate. The summary's `mutation_effects` founder cohort rows
move with the draw remap; that row predeclared only the new keys. Met:
persistence floor, ring_rules fingerprint on every world, `dead_per_all_births`
0.0052 / 0.0072 / 0.0018 against bounds 0.0058 / 0.0140 / 0.0084, drift-walk
bounds, `AddRingProjection` silent (0 changed, 0 dead) in every cohort.

**User-accepted breaches (user, 2026-10-11, answer "Accept" to each).**

| Breach | Measured | Attribution |
| --- | --- | --- |
| Orchards `neighborhood_read.changed_per_all_births` below its standing floor | 0.1432 against floor 0.1464 (reference 0.1568). Drops against the reference: −8.7% Orchards, −16.2% Canyon (0.1348, floor 0.1300), −4.4% Confluence (0.1594, floor 0.1434) | Silent supply, not harm. The predeclared "slightly down" counted only `AddRingProjection`, at about 6% of the applicable Graph weight. `ShiftRingProjectionOffset`, `RetargetRingProjectionKind` and `RemoveRingProjection` are also silent on the zero-weight projections most carriers hold, so a larger share of births changes nothing. `dead_per_all_births` and the drift walk met their bounds. The floor and its value are unchanged. |
| Steering avoidance 0 on all three worlds against the note §8 target (above 0) | Avoidance 0 of 472 / 537 / 459 moves; exact-hit 0.4407 / 0.4395 / 0.4205 met the 0.37 target | Predeclared as likely: projections are born at zero weight, few carry a weight step by tick 2,000, and few vote nodes declare a barrier ring. The target is unchanged. |

- Summaries: [gate](../../progress/features/t11-f31-ring-projection-motif.json),
  [goal](../../progress/features/t11-f31-ring-projection-motif-goal.json).
- Full readings: [`docs/progress/readings/t11-f31.md`](../../progress/readings/t11-f31.md).

## Success Criteria

- [ ] A ring projection is one heritable gene: one size unit, born at weight 0
      by its own operator, and every change to its weight, offset, kind or
      existence acts on all eight directions in one event; per-direction edges
      and `InputRef.Add` are unchanged.
- [ ] Every reader in the Reader table sees projections, `Prune` never orphans
      one, and production, observed and traced execution agree.
- [ ] Gate and goal ran once each and every row of the predeclaration is read;
      every floor, guard, fingerprint and target is met, or the user has
      explicitly accepted its breach; an unresolved breach stops closure.
- [ ] `make check` exits 0 and the mutation gate is recorded.

## Notes for AI Agents

- Exception: T11.F18 backend parity does not cover ring projections. VM nodes
  get no projection opcode, the option the user allowed on 2026-10-11 (track
  Notes), because 0 of 1,240 reachable live VM nodes vote; a VM node reaches
  projections only through `SwapNodeBackend` to Graph.
