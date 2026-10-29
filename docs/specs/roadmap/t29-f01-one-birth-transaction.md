# T29.F01 — One Birth Transaction

**Status**: In Progress
**Last updated**: 2026-10-28
**Feature**: T29.F01
**Track**: [T29 — Heredity](../../roadmaps/t29-heredity.md)

## Goal

An egg is a nucleus and its cytoplasm, assembled once (the row's natural
analog): every creature, founder or child, is built by one birth transaction
in `creature::birth` from a draft (genome, birth payload, provenance) and one
finalization step, with no behavior change. Learned seeds leave the genome
type and travel in the payload keyed by learnable identity; every structural
edit returns the inheritance mapping finalization applies to them; energy
provisioning and its cap run inside finalization; no trait draws its own
randomness outside the mutation policies. Every pin, draw stream, gate
deterministic block and digest is unchanged; a pin that moves is a defect.
Pays G4.

## Non-Goals

- Body genes, per-part supply, deleting the colour and kin-tag gate, moving
  `lineage_id` out of the body (T29.F02, G5); memory masks (T29.F03, G6).
- Route composition through gate writes and VM program repair
  (`mutation::topology::routing`, G23, T30.F05); T30.F11's remaining items (operators
  reporting the nodes they add, the linear `insert_member`, `BundleCreations`); a narrower dispatch-record query (T30.F02 made
  it; F01 only passes it on).
- Making `CreatureGenome.nodes` private (audit A11) or the L7 node edit
  transaction (open in the track).
- Any change to a value law, draw, operator catalog, weight, serialization
  or config field; any new signal; any goal or goal-long run.

## Inputs and Invariants

| Input | Where | What F01 takes from it |
| --- | --- | --- |
| Track row, criterion 1 and Notes | `docs/roadmaps/t29-heredity.md` (F01 row; first criterion; "F01 acceptance"; "F01 is behavior-preserving"; "Verification") | The goal, scope, the four mapping kinds, the preservation cases, the acceptance guard, "F01 gate, pins unchanged", slow tests run |
| Design note | `docs/strategy/heredity-architecture-2026-10-22.md` ("The builder", "Seeds leave the genome type", L3–L6) | Draft and finalize steps, the payload table, mappings over stable ids alone (user) |
| T25.F12 | invariants 4, 5, 7, 8; Architecture | The learning layout (one pure function of the genome), birth values from seeds, "a child consumes a seed only where its own layout has a lamarckian learner there", the detach table; seeds' location was left to F01 |
| T30.F08 | `7fc60133f` | One creature constructor rebuilds derived state on every birth; no cache reuse returns |
| Audit | [readings](../../progress/readings/t29-f01.md), rows A1–A12 | Each `in-feature` row is a Decision below; A10, A11 are Deferred |
| Code | `creature/{state,body,identity,newborn}.rs`, `simulation/seeding.rs:96-158`, `simulation/actions/{reproduction.rs:298-554,cgp_reproduction.rs}`, `simulation/tick/actions.rs:61,189`, `genome/{mod.rs:178-191,node_edit.rs,learning.rs}`, `genome/cgp/container.rs`, `mutation/{engine/mod.rs:464-541,phenotype.rs,graph/operators.rs:20-50,graph/bundle.rs:262,topology/structural.rs,topology/routing.rs:160-290}`, `runtime/plasticity/learned.rs:448-480,580-660`, `v3-lab/src/eval.rs:111-131` | What changes |
| Reference specs | `docs/reference/v3-{reproduction,creature-identity,phenotype}-spec.md` | Sentences naming `spawn_child`, `HeritableBody::inherit`, `mutate_kin_tag` or seeds on the genome |

Research: the options were stable learnable ids stored on edges and bundles
(innovation-number style), today's seed rows aligned inside the container,
and mappings returned by edits. Stored ids are genome data (they would enter
serialization or need a second skip) and cannot express a copy or a detach
composition alone; the rows are L6. The user chose mappings (note, "Seeds
leave the genome type"). No external library applies; this is an internal
ownership move.

Vocabulary. A **learnable key** (`LearnableKey`, genome learning module) is
`Edge(EdgeOccurrence)` or `Scale(BundleId)` within one Graph node revision;
the layout's `EdgeLearner` and `ScaleLearner` expose theirs. A **learnable
identity** is a node index and key in the parent genome as captured. An
**origin** says where a post-edit learnable's inherited value comes from:
`Keep(id)`, `Copy(id)` (a new identity naming its source), `Compose { modifier,
scale, modifier_genomic, scale_genomic, pattern }` over two origins, or
`Fresh`. An **inheritance mapping** gives every learnable key of the post-edit
genome its origin; a pre-edit key with no image is dropped.

| # | Invariant |
| --- | --- |
| 1 | **Birth module.** `creature::birth` owns `Zygote { genome, body: HeritableBody, payload: BirthPayload, provenance: Provenance, inheritance }` (the body leaves at T29.F02), `Zygote::founder(genome, body)`, `Zygote::conceive(parent: &CreatureState, config, rng)` and `Zygote::finalize(id, BirthPlace { position, heading, energy })`. `CreatureState`'s constructor is private to `creature` and called only by finalization; every other builder (seeding, delivery, the lab, every test fixture) goes through the birth API. `creature/newborn.rs` and `simulation/actions/cgp_reproduction.rs` are deleted. |
| 2 | **Payload.** `BirthPayload { seeds, shared_memory }`: `seeds` holds each initialized learner's value keyed by learnable identity, captured by `LearnedState` from the parent's runtime at delivery before mutation (today's capture rule: initialized learners only, none when nothing is initialized); `shared_memory` is the parent's at delivery. A founder's payload is empty and zeroed. The seed value type and its keys live in the genome learning module; no genome type holds a seed. `BirthPayload` is constructible from keyed seeds and memory and `Zygote::with_payload` replaces a draft's payload, for fixtures that seed learners today (`v3-telemetry/src/tests.rs:1414-1472`, `v3-server/tests/server.rs:2842-2851,3005-3014`); seeding and delivery never call it. |
| 3 | **Provenance.** `generation` (founder 0, child parent's + 1) and the birth record `birth_mutation_operators` (applied operators sorted by key) and `birth_edge_class`, derived from the `MutationSummary` at conception; empty for a founder. Finalization writes them; no production birth (seeding, delivery, the lab) writes a creature field after finalization. A test fixture may set world-facing state afterwards (energy, position, memory), so fixtures that start above capacity (`simulation/tick/tests/support.rs:169`, `tick/tests/effort.rs:354`) keep their requested energy and assertions. |
| 4 | **Mappings.** Every container primitive and genome operation that moves, copies, composes, drops or invalidates a learnable returns its inheritance mapping beside its existing return (`BundleEdits`, occurrences); an edit that does none (learning settings, `credits`, kind, `clear_learning`) returns none and keeps every identity. Each primitive's mapping is exactly its seed rule today: a source change (`EdgeMut::set_source`), weight write or scale write gives `Fresh`; `split_edge` keeps the split edge's identity; a retarget or split of a member detaches it inside the container operation (Decision A12), so the operation's mapping composes the detach's; `copy_edges` and `duplicate_compute_nodes` give `Copy`; `detach_at` gives `Compose` with the genomic modifier and scale at the detach and its pattern; removal drops; inserted edges, nodes and bundles are `Fresh`. One type, its composition (`later ∘ earlier`, substituting origins) and its application to a payload live in the genome learning module; application yields `Keep`/`Copy` → the source's seed, `Compose` → `compose(modifier.unwrap_or(modifier_genomic), scale.unwrap_or(scale_genomic), pattern)` when either seed is present (else none), `Fresh` → none: today's values, bitwise. |
| 5 | **Composition across a birth.** Each operator returns its event's composed mapping in its `EventReport`; the engine composes an event's mapping into the birth's only when `apply_gated` commits it, so a rejected event's mapping vanishes with its genome edits. `MutationEngine::apply_mutations_in` returns the birth's composed mapping beside the `MutationSummary`; callers that only screen genomes ignore it. |
| 6 | **Genome mesh edits.** Node removal with successor rewiring, copy-with-attachment (with or without the backend swap), slice clone with id remapping and attachment, and detour append are `CreatureGenome` operations returning their mappings (removal drops the node's learnables and shifts later indices; copies give `Copy` for every learnable of a copied Graph backend; a swapped or appended backend is `Fresh`). Mutation keeps every draw and its order; `clone_and_remap_slice`'s link draw, which reads only the slice length, is taken before the operation is called. |
| 7 | **Node fields.** `NodeGenome`'s `input_refs` and `backend_def` are private: `NodeGenome::new(node_id, input_refs, backend, targets)`, read accessors, and a split-borrow accessor (`&[InputReference]`, `&mut` backend) for backend edits; serde field names and JSON are unchanged. Routing edits its backend in place through that accessor and forwards the Graph edit's mapping (no clone-and-assign). Seeding's bench injection builds node 0 through `NodeGenome::new` with the replaced declarations and the existing backend; `swap` keeps its same-kind, same-width rule. |
| 8 | **Body draws.** A mutation body policy samples the phenotype step (switch draw, its index among the other five channels, polarity draw), the kin bit, then capacity, steal share and give share (chance draw, then step), in today's order and only from config; only the phenotype and kin-tag draws are gated by `applied_events > 0` (A10), and the capacity and both share draws run on every birth, as today; `HeritableBody` applies the sampled choices with today's arithmetic and clamps. `mutate_kin_tag` and the draws in `HeritableBody::inherit` and `mutate_phenotype` are gone, so `mutation::phenotype` no longer imports `creature::phenotype`. |
| 9 | **Conception order.** Clone the parent genome; capture the payload; mutate (the parent's reachable nodes and its dispatch record passed as the named `ParentExecuted` input); sample and apply the body; derive provenance. No other draw; the reproduction stream is consumed exactly as today. The simulation folds the summary into `SimStats` through one `SimStats` operation (A3). |
| 10 | **Finalization.** Apply the birth's mapping to the payload; build derived state, learned state from the applied seeds by the child's layout (T25.F12 invariants 5, 7) and the runtime through the one constructor; take the payload's memory; zero per-life state (`prev_shared_memory`, `previous_outcome`, operator state, dispatch history, lifetime counters); set the heading from `BirthPlace`; provision energy: the given energy capped at the body's capacity, returning the overflow (`applied_debit`, as `cap_energy` does). Delivery books overflow and credit as today; seeding books nothing, and a normalized founder's overflow is zero (`config/simulation.rs:1300-1306`). World placement, the action log and parent counters stay in the simulation. |
| 11 | **Pins.** Unchanged: `tests/viability.rs` digests, `tests/applied_trajectory.rs`, `tests/baseline_worlds.rs`, every draw-stream tail, genome JSON and `Debug` digests (`neighborhood/mutation_effects/mod.rs:58`), census pins, `bench_artifacts` digests, lab fixtures, the slow tier and the gate's deterministic blocks. Tests of the deleted seed rows (`cgp_reproduction.rs` tests, container seed-shape tests) are rewritten against the mapping contract with the same cases; they are test contracts, not pins. |

## Architecture

Owners touched ([`docs/architecture.md`](../../architecture.md) §3): the
creature domain gets the one birth constructor (finalization, payload,
per-life initialization, provisioning) as §3 already assigns it; the genome
owns learnable keys, the mapping type with its composition and application,
the mesh edit operations and private node fields ("edits that validate and
return their effects on inherited values"); mutation keeps supply, dispatch
and every birth draw, including the body policy, and only forwards mappings;
runtime owns capture of learned values and building learners from seeds; the
simulation supplies world context and books flows. Pays G4, deleted at
closure; the body policy also removes `mutation::phenotype`'s import of
`creature::phenotype`, so that clause of G5 and its line in
`scripts/architecture-debts.txt` go at closure. No boundary changes: §3 and
§4 ("Birth") already state this split, so the plan does not edit the file.
Shared rule "where an inherited value goes after an edit": owner the genome
mapping; today it is applied inside the container's primitives
(`genome/cgp/container.rs` seed writes, audit A1) and by whole-backend clones
in `mutation::topology::structural`; both move to it. Shared rule "which
seed a child consumes": owner the learning layout via `LearnedState`
(T25.F12 invariant 7), unchanged. Pays two items of T30.F11's note (the split borrow, A8; owner-complete retarget and split, A12), because F01 rewrites both sites and an accepted deficiency is not accepted twice. Retained debts: G5's gate (A10, marked
`WORKAROUND(G5)` at the body policy), G23 route composition (routing changes
only its backend access), T30.F11's added-node clone in the engine, and the
public `CreatureGenome.nodes` (A11).

## Telemetry

Not applicable: a behavior-preserving restructuring that adds no counter,
signal or config value; every existing signal reads identical values.

## Verification Plan

**Question.** F01 claims every creature is built by one birth transaction
and seeds cross edits by returned mappings with no behavior change; a moved
pin, draw-stream tail or gate deterministic value shows it wrong, and a
gate wall-clock flag shows the mapping or capture costs more than predeclared.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate` | Reproduction and seeding are tick-loop code with no population reading in question; the gate's deterministic blocks are the byte-identity reading and its wall-clock the cost reading, in seconds. |
| Slow tests (`make rust-test-slow`) | `run` | `crates/v3-lab` changes (its creatures build through the birth API), and the slow tier pins the recruitment-path readings a moved draw or seed would change. |

## Implementation Tasks

- [ ] Viability first: `cargo test -p v3-core --test viability` before and after (founder construction changes).
- [ ] Decisions A6, A8, A9: private node fields, `NodeGenome::new`, accessors, the split-borrow accessor at both mutation sites (the `WORKAROUND(T30.F11)` at `graph/operators.rs:36` goes), routing in place, the bench injection.
- [ ] Decision A12: one container retarget operation that detaches a member before a source change (a same-source draw stays a no-op) and an owner-complete `split_edge` that detaches first, each returning its bundle effects and mapping, with split's weight, seed and draw behavior unchanged; `detached` and its `WORKAROUND(T30.F11)` go; at closure the T30.F11 note drops the split borrow, retarget and split items F01 paid.
- [ ] Decision A7: the genome mesh operations; `structural.rs` keeps only draws and eligibility.
- [ ] Decision A1, step 1: mappings and learnable keys beside today's seed rows, with a property test over the Graph, InputRef and Topology catalogs that applying an event's mapping to the parent's seeds equals the rows the container kept; then the payload, the zygote and finalization (A2, A4).
- [ ] Decision A1, step 2: delete `BirthSeeds`, the `birth_seeds` field, `take_birth_seeds`, `capture_birth_seeds` and the step-1 equivalence test; seeds reach the runtime only through finalization.
- [ ] Decision A3: one `SimStats` operation folds a `MutationSummary`; `spawn_child`'s `too_many_lines` allow goes.
- [ ] Decision A5: the mutation body policy and `HeritableBody`'s deterministic application; `WORKAROUND(G5)` at the gate (A10).
- [ ] Deferred A11: `WORKAROUND(T29.F01): CreatureGenome.nodes private behind owner operations` at the field.
- [ ] Remove the three `WORKAROUND(G4)` markers; update the three reference specs; at closure delete G4 and the G5 import clause in `docs/architecture.md` and the debt line.
- [ ] Acceptance guard: shared finalization assertions through the founder, unchanged-child and mutated-child paths (inherited learning, per-life initialization, provisioning and cap); committed and rejected edits exercised against the mapping contract through the existing operator catalogs; a proptest that applying a composed mapping equals applying each mapping in turn.

## Verification

- [ ] Focused tests: `cargo test -p v3-core --test viability` -> pass, digests unchanged; acceptance-guard tests -> pass.
- [ ] `cargo test --workspace` -> every pin in invariant 11 unchanged.
- [ ] `make rust-test-slow` -> pass, readings unchanged.
- [ ] Gate profile: summary at `docs/progress/features/t29-f01.json`, deterministic blocks identical to the epoch, raw hash and byte count checked, series entry points to the summary, no full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: the egg's
nucleus and cytoplasm assembled once; it reaches creatures through every
birth. No trajectory moves: the gate's deterministic blocks are identical to
the gate epoch, and any difference is a defect, never a re-pin. Compute:
births build an explicit mapping per committed structural event and capture
seeds into a keyed payload instead of aligned rows; neither runs per tick.
Gate `wall_clock_ms_per_creature_tick` is predeclared within the 25 % flag
threshold against the gate epoch, ±5 % as a hypothesis; a `flag` is
reported with its mechanism and a `severe` blocks closure until remediation
removes it or the user accepts it. No indicator has an expected direction.

**Measured verdict.** Pending.

- Summary: [gate](../../progress/features/t29-f01.json).
- Full readings: [`docs/progress/readings/t29-f01.md`](../../progress/readings/t29-f01.md).

## Success Criteria

- [ ] Seeding, delivery and the lab build every creature through `Zygote` and one finalization; no other code constructs a `CreatureState`.
- [ ] No genome type holds a seed; seeds cross every edit by returned mappings, composed only for committed events.
- [ ] Provisioning and its cap run inside finalization; no body or identity type draws randomness.
- [ ] `NodeGenome`'s declarations and backend are private; mesh copy, removal and slice cloning are genome operations.
- [ ] Every pin, draw stream and gate deterministic block is unchanged; `make check` passes.

## Notes for AI Agents

- Decision: audit rows A1–A9 and A12 are in-feature refactors of this feature ([readings](../../progress/readings/t29-f01.md)).
- Deferred: A10, the colour and kin-tag gate on `applied_events > 0`, moves with the body draws and is paid by T29.F02 (G5).
- Deferred: unpaid accept, `CreatureGenome.nodes` (`genome/mod.rs:191`); payer proposed to the user at closure.
