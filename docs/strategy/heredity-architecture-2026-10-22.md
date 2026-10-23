# Heredity: one genome, one birth payload, one builder

**Date**: 2026-10-22
**Question**: Where do the body, the brain genome and inherited runtime
state belong, who owns each, and how does a child get built from them?
**Method**: the creature, mutation and reproduction code read on `main` at
10519fef (`creature/body.rs`, `creature/identity.rs`, `creature/state.rs`,
`creature/newborn.rs`, `creature/genome/mod.rs`,
`creature/genome/cgp/container.rs`, `mutation/engine/mod.rs`,
`mutation/phenotype.rs`, `simulation/actions/reproduction.rs`,
`simulation/actions/cgp_reproduction.rs`, `simulation/seeding.rs`); the T25,
T26 and T27 roadmaps; the companion
[sensor wiring](group-creation-unification-2026-10-22.md) and
[bundle learning](bundle-learning-ownership-2026-10-22.md) notes.

## Finding

`spawn_child` already coordinates a birth, but it lives in the simulation
layer, has a second builder beside it (seeding), and works under three
mutation laws:

| Leak | Today |
|---|---|
| L1 | The heritable body (`HeritableBody`: colour, kin identity, capacity, two exchange shares) sits on `CreatureState` outside `CreatureGenome`. It costs 0 genome units and is not part of genome equality. |
| L2 | Three mutation laws. Genome: `Binomial(genome_size, 0.0060625)`. Capacity and both shares: chance 0.1 per birth each. Colour and kin tag mutate only on births where a genome event was applied (`applied_events > 0`, about 39% of founder births), which couples body heredity to brain heredity. |
| L3 | The birth transaction is implicit. `spawn_child` (simulation layer) clones and mutates the genome, calls `HeritableBody::inherit` (which draws its own randomness, `identity.rs` `mutate_kin_tag`), copies `shared_memory`, smuggles learned seeds through the genome draft and advances `generation`; the constructors rebuild caches and runtime, or copy the caches on an unchanged-genome birth; `cap_energy` runs after construction; seeding (`simulation/seeding.rs`) is a second builder. |
| L4 | Each inherited store has its own policy, which is legitimate, but only learning's is heritable. Learned weights pass by each edge's `lamarckian` flag; `shared_memory` passes always; stateful operator state, `prev_shared_memory`, `previous_outcome`, the dispatch record and T27.F02's planned signals start at zero. |
| L5 | The parent's dispatch record (`ParentExecuted::Record`, T11.F17) biases where the child's mutations land. That is runtime state feeding heredity, and the code does not name it as such. |
| L6 | Learned seeds ride inside the genome type: `CgpGraphBackendDef.birth_seeds`, `#[serde(skip)]`, outside equality. They are there so the container's edit primitives keep them aligned with edge indices (T25.F01). |
| L7 | Declaring a sensor (`input_refs`, 1 unit) and using it (VM `ReadInput`, Graph bundles and edges) are separate, which is legitimate: VM and Graph consume declarations differently and an unused declaration is a supported state. The open part is narrower: the InputRef mutation category edits both (`mutation/input_ref/mod.rs`), so the node, not a mutation category, should own the cross-reference invariants. |

How it drifted: T26.F01 made the body one record to replace ten positional
constructor arguments at 55 sites (interaction design note §4). That was a
refactor convenience, never a decision that the body stays out of the
genome. T26.F02 and T27.F03 then added traits to the record with their own
per-birth chances. T26.F01 kept the colour and kin-tag gate on brain
mutation deliberately, as inherited coupling to preserve behavior, not as a
design choice; the decision below removes it.

## Decision (user, 2026-10-22)

- The body lives in the genome, under one mutation law: a body gene
  mutates at the same per-unit rate as a brain unit (multiple choice:
  "Genome, one law"). The user accepted that capacity and the shares then
  mutate about 16 to 18 times less often than today's 0.1 per birth.
- Whether a child receives the parent's memory is heritable, slot by slot,
  through 16-bit mask genes, 1 genome unit each, one bit per
  `shared_memory` slot, mutation flipping one bit; the founder has every
  bit set. First decided as one mask (multiple choice: "One gene, per-slot
  mask"); after the architecture review the user chose two ("Both
  evolvable"): the parent's provisioning mask and the child's uptake mask,
  a slot passing when both bits are set.
- Learned seeds cross structural edits by inheritance mappings the Graph's
  edits return, not by stable ids alone (user, after the architecture
  review; [architecture](../architecture.md) A5).

## Model

### Objects and owners

| Object | Holds | Counted | Owner of edits |
|---|---|---|---|
| `CreatureGenome` | a brain chromosome (today's `entry_node_id` and mesh nodes, unchanged) and a body chromosome | yes, `genome_size = brain units + body units`, an explicitly defined economic measure (user decision) | each chromosome's own edit API |
| `BodyGenome` | colour, kin tag, capacity, steal share, give share, memory provisioning mask, memory uptake mask: seven genes (provisioning is a parent's body trait, uptake a child's) | 1 unit per gene | its own edit API; a body mutation policy samples and calls it |
| `BirthPayload` | learned seeds and the parent's memory, captured at delivery | no | the birth transaction |
| `Zygote` | a mutated genome, its payload and its provenance (parent, `lineage_id`, `generation`) | — | the birth transaction, for founders and children |

Mutation categories (Topology, VM, Graph, InputRef and the new body
policy) are how the engine dispatches events, not owners: each chromosome,
and within the brain each node, owns its data and its invariants, and a
mutation policy only samples an edit and calls that owner's API. All
randomness of a birth lives in the mutation policies; no trait draws its
own (today `mutate_kin_tag` and `mutate_phenotype` do).

`CreatureState` is the application aggregate the birth transaction builds
from one `Zygote` plus a position and an energy; it is not a foundational
type other layers build on. Nothing else is heritable. The body's rules (`move_factor`, `upkeep`,
`energy_scale`, the energy cap) read the body genes and stay where they are.

### Mutation supply: one law

Each part of the genome draws its own events at the shared rate:
`Binomial(brain units, rate)` and `Binomial(body units, rate)`. Their sum
is `Binomial(genome_size, rate)`, today's supply law. Brain dispatch
(mesh layer, then node-internal domains) is unchanged. A body event picks
one body gene uniformly and applies that gene's operator: today's
phenotype step, today's one-bit kin flip, today's log step for capacity and
the shares, and one bit flip for either memory mask. The `applied_events > 0`
gate is deleted. Assistant proposal: per-part draws, so neither the brain's
domain odds nor the body's rate depends on the other.

### The builder

One birth transaction coordinates every newborn; specialist components
supply each part, and each store keeps its own policy (one coordinator,
not one universal rule). Assistant proposal: it lives in a creature-birth
module beside the genome, mutation and runtime, and the simulation calls
it with world context (position, litter energy, birth direction).

1. **Draft.** `Zygote::founder` (seeding) or `Zygote::conceive`
   (delivery): the genome draft, the payload, provenance; a child's also
   runs the mutation policies and returns the `MutationSummary`, which
   stays the record of applied events.
2. **Finalize.** Resolve the learning layout from the child's mutated
   genome (T25.F12), apply the child's inheritance settings to the payload
   (table below), build caches, sink weights and runtime (or reuse the
   parent's caches when no event applied, as the fast path does today),
   reset per-life state (`prev_shared_memory`, `previous_outcome`,
   operator state, dispatch history, signals), set the heading from the
   birth direction, and provision energy.

The finalizer consumes the payload by the child's genome:

| Payload part | Child receives it when |
|---|---|
| a learned seed | the child's edge or bundle `Learning` is `lamarckian` (today's rule, after T25.F12) |
| a memory slot | the parent's provisioning bit and the child's uptake bit for that slot are both set |

Seeds follow the child's genome, as today. Memory follows both
generations (user): a provisioning mutation shows in the mutant's
offspring, an uptake mutation in the mutant itself.

Provisioning belongs to the builder: the litter energy and its cap against
the child's capacity (today's `cap_energy` after construction) are one
step, with the overflow booked as today.

L5 stays: the dispatch record is a named input to `Zygote::conceive`'s
mutation step. Natural analog: genes that are actively transcribed mutate
and get repaired at different rates from silent ones.

### Seeds leave the genome type

L6 exists because seeds are keyed by edge index, and only the container's
edit primitives know how indices move. A key alone is not enough: a copy
inside one genome makes two learnable numbers from one, and a detach
composes two seeds into one. So every Graph edit returns an inheritance
mapping (an unchanged locus keeps its identity, a copy gets a new identity
naming its source, a detach composes `scale × pattern + modifier`, a
removal drops, a source change or weight or scale write invalidates);
mappings compose across one birth's committed edits and vanish with a
rejected edit, and the learning layout exposes the identities. Birth
applies the composed mapping to the payload and repeats no graph-edit
semantics (user, after the architecture review).

### Other moves

- `lineage_id` leaves the body: it is an observation label for clade
  telemetry (T27.F01 kept it "observation-only"), not a trait. Assistant
  proposal.
- Colour is one gene: its active channel and polarity stay inside it, as
  today.
- `generation`, `heading`, `energy` and lifetime counters are not
  heritable and stay on `CreatureState`.

## Consequences to accept

- **Body variation slows.** Capacity and each share: 0.1 → about 0.0056 per
  birth. Colour and kin tag: about 0.39 → about 0.0056 per birth at founder
  size. Kin tags then drift about 70 times more slowly, so `KIN_AFFINITY`
  separates lineages later in a run. Step sizes are not retuned here; a
  later sweep may change them.
- **The founder grows** from 80 to 87 units (five body traits and two
  memory masks). To keep the founder's supply at 0.485 events per birth,
  `per_unit_rate` becomes `0.485 / 87` (about 0.00557), as T25.F02 did at
  80. The replication-cost and gestation anchors move with the founder
  size.
- **Body genes have a carrying cost**: each pays carry, replication and
  gestation like any unit.
- **Genome equality and serialization include the body**, so the
  no-mutation fast path, mutation-effect screens and genome digests move.
- **Two memory masks with every bit set match today's behavior.** Selection
  can now clear bits.
- No backward compatibility: genomes, draws and pins move, predeclared.

## Collisions

- T26 (Complete): supersedes T26.F01's criterion that the body is "one
  record passed as one constructor argument and inherited by one call". The
  body rules and T26.F02's capacity law are kept.
- T27.F03: `steal_share` and `give_share` become body genes. T27.F02's
  signals keep their own row; whether signals join the payload under a
  mask is T27's decision.
- T25.F11 and F12 land first: F12 rewrites seed capture and adds the
  learning layout whose identities this note's inheritance mappings use. Nothing here changes bundle
  structure.
- L7 overlaps T25.F11 and is not decided here.

## Review

A Codex architecture challenge of the parent session's domain diagrams
(2026-10-22, forwarded with the user's request; full output outside the
repo) was weighed against the code after this note's first version:

- Mutation domains are dispatch categories, not ownership boundaries
  (`mutation/engine/mod.rs`; `mutation/input_ref/mod.rs` writes a Graph
  edge). Accepted: owners are chromosomes and nodes, policies call their
  edit APIs; L7 narrowed.
- Body/brain separation is reasonable, and charging body fields as genome
  units changes what `genome_size` means, which is the user's model
  decision. Already decided by the user ("Genome, one law"); the note now
  names `genome_size` an explicitly defined economic measure. Accepted:
  trait randomness moves into mutation policies; lineage provenance
  separates from the evolvable kin tag.
- "Two inheritance owners" mistook different payload policies for
  duplicate ownership, and the birth transaction was missing (generation,
  cache reuse, rebuilt caches and learner state, resets, heading).
  Accepted: L3 and L4 restated, the builder became a two-step birth
  transaction with one coordinator and per-store policies.
- `CreatureState` is an application aggregate, not a foundational type;
  the coordinator does not live in the simulation layer. Accepted as an
  assistant proposal.

## Natural analogs

- One genome encodes both the body plan and the nervous system.
- The egg's cytoplasm (maternal mRNA, mitochondria) is the birth payload,
  separate from the nucleus.
- How much a parent passes through the egg is itself heritable and differs
  between species (maternal effects).

## Shape of the work (assistant proposal)

Three rows, after T25.F12, in a new track,
[T29 Heredity](../roadmaps/t29-heredity.md) (user, 2026-10-22, multiple
choice). The track states that it supersedes
T26.F01's record criterion:
1. **Birth transaction and payload**, behavior-preserving: one
   coordinator for founders and children in a creature-birth module, the
   payload object, provenance on the zygote, stable learnable ids,
   provisioning inside finalization, trait randomness moved into mutation
   policies with today's draw order kept. Every pin holds; any moved pin is a
   defect.
2. **Body genes**: `BodyGenome` in `CreatureGenome`, per-part supply, the
   Body domain and its operators, the gate deleted, `lineage_id` moved to
   observation, founder 85 and the rate re-pinned. Predeclared re-pins:
   draw streams, trajectories, genome format, both epochs.
3. **Memory inheritance masks**: the provisioning and uptake genes,
   founder all-on, the finalizer reading both; founder 87 and the rate
   re-pinned. Pins move only through
   the two extra units.

Verification for each: viability first (founder and supply change), gate +
goal; mutation gate and slow tests run. Goal readings: body-gene mutation
counts by gene, the capacity, share and colour distributions beside T26.F02's
baseline, kin-tag diversity, and both masks' bit frequencies.

## Open

- L7: whether every edit touching a node's declarations goes through one
  node edit transaction that owns the declaration–use invariants, so no
  mutation category keeps them by hand. Decide after T25.F11 lands.
- Whether T27.F02's signals join the birth payload under their own mask.
- For the spec owner: the Body operators' draw order and the per-part
  supply's effect on existing draw streams; the stable-id representation.
