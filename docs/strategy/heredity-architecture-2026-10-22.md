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

A child gets what it inherits through four separate routes in
`spawn_child`, under three mutation laws:

| Leak | Today |
|---|---|
| L1 | The heritable body (`HeritableBody`: colour, kin identity, capacity, two exchange shares) sits on `CreatureState` outside `CreatureGenome`. It costs 0 genome units and is not part of genome equality. |
| L2 | Three mutation laws. Genome: `Binomial(genome_size, 0.0060625)`. Capacity and both shares: chance 0.1 per birth each. Colour and kin tag mutate only on births where a genome event was applied (`applied_events > 0`, about 39% of founder births), which couples body heredity to brain heredity. |
| L3 | No single owner of what a child receives. `spawn_child` clones and mutates the genome, calls `HeritableBody::inherit`, copies `shared_memory` and captures learned seeds; `cap_energy` runs after construction; seeding (`simulation/seeding.rs`) is a fifth route. |
| L4 | Inherited runtime state follows no rule. Learned weights pass by each edge's heritable `lamarckian` flag; `shared_memory` passes always, with no gene; stateful operator state, `prev_shared_memory` and T27.F02's planned signals start at zero. |
| L5 | The parent's dispatch record (`ParentExecuted::Record`, T11.F17) biases where the child's mutations land. That is runtime state feeding heredity, and the code does not name it as such. |
| L6 | Learned seeds ride inside the genome type: `CgpGraphBackendDef.birth_seeds`, `#[serde(skip)]`, outside equality. They are there so the container's edit primitives keep them aligned with edge indices (T25.F01). |
| L7 | Declaring a sensor (`input_refs`, InputRef domain, 1 unit) and wiring it (bundles, Graph domain, 1 unit each) are two domains. T25.F11's `AddSensor*` joins them for Graph nodes only. |

How it drifted: T26.F01 made the body one record to replace ten positional
constructor arguments at 55 sites (interaction design note §4). That was a
refactor convenience, never a decision that the body stays out of the
genome. T26.F02 and T27.F03 then added traits to the record with their own
per-birth chances.

## Decision (user, 2026-10-22)

- The body lives in the genome, under one mutation law: a body gene
  mutates at the same per-unit rate as a brain unit (multiple choice:
  "Genome, one law"). The user accepted that capacity and the shares then
  mutate about 16 to 18 times less often than today's 0.1 per birth.
- Whether a child receives the parent's memory is heritable: one 16-bit
  gene, 1 genome unit, one bit per `shared_memory` slot; a set bit passes
  that slot, a clear bit gives the child 0; mutation flips one bit; the
  founder has every bit set (multiple choice: "One gene, per-slot mask").

## Model

### Objects and owners

| Object | Holds | Counted | Owner of edits |
|---|---|---|---|
| `CreatureGenome` | `brain` (today's `entry_node_id` and mesh nodes, unchanged) and `body` | yes, `genome_size = brain units + body units` | the mutation engine |
| `BodyGenome` | colour, kin tag, capacity, steal share, give share, memory mask: six genes (the mask's place here is an assistant proposal: a parent's provisioning is a body trait) | 1 unit per gene | a Body mutation domain |
| `BirthPayload` | learned seeds and the parent's memory, captured at delivery | no | the zygote builder |
| `Zygote` | a mutated genome and its payload | — | one builder, for founders and children |

`CreatureState` is built from one `Zygote` plus a position and an energy.
Nothing else is heritable. The body's rules (`move_factor`, `upkeep`,
`energy_scale`, the energy cap) read the body genes and stay where they are.

### Mutation supply: one law

Each part of the genome draws its own events at the shared rate:
`Binomial(brain units, rate)` and `Binomial(body units, rate)`. Their sum
is `Binomial(genome_size, rate)`, today's supply law. Brain dispatch
(mesh layer, then node-internal domains) is unchanged. A body event picks
one body gene uniformly and applies that gene's operator: today's
phenotype step, today's one-bit kin flip, today's log step for capacity and
the shares, and one bit flip for the memory mask. The `applied_events > 0`
gate is deleted. Assistant proposal: per-part draws, so neither the brain's
domain odds nor the body's rate depends on the other.

### The builder

`Zygote::founder` (seeding) and `Zygote::conceive` (delivery) are the only
two constructors. Each produces a genome and a payload; a child's also
produces the `MutationSummary`, which stays the record of applied events.
The constructor consumes the payload by the child's genome:

| Payload part | Child receives it when |
|---|---|
| a learned seed | the child's edge or bundle `Learning` is `lamarckian` (today's rule, after T25.F12) |
| a memory slot | the child's memory-mask bit for that slot is set |

Assistant proposal: the child's genome decides in both cases, as seeds do
today. A maternal-effect reading (the parent's mask decides) is the
alternative; it would make a mask mutation show up only in grandchildren.

Provisioning belongs to the builder: the litter energy and its cap against
the child's capacity (today's `cap_energy` after construction) are one
step, with the overflow booked as today.

L5 stays: the dispatch record is a named input to `Zygote::conceive`'s
mutation step. Natural analog: genes that are actively transcribed mutate
and get repaired at different rates from silent ones.

### Seeds leave the genome type

L6 exists because seeds are keyed by edge index, and only the container's
edit primitives know how indices move. The payload can leave the genome
only if every learnable number has a key that survives edits: a stable id
per edge and per bundle, kept by copies, fresh on creation, dropped on
removal. Assistant proposal: stable ids, which T25.F12's learning layout
(every learner with its identity, derived once at birth) can carry.
The alternative, keeping seeds in the container behind the builder,
keeps L6 and would need a `WORKAROUND` marker.

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
- **The founder grows** from 80 to 86 units (five body traits and the
  memory mask). To keep the founder's supply at 0.485 events per birth,
  `per_unit_rate` becomes `0.485 / 86` (about 0.00564), as T25.F02 did at
  80. The replication-cost and gestation anchors move with the founder
  size.
- **Body genes have a carrying cost**: each pays carry, replication and
  gestation like any unit.
- **Genome equality and serialization include the body**, so the
  no-mutation fast path, mutation-effect screens and genome digests move.
- **A memory mask with every bit set matches today's behavior.** Selection
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
  learning layout this note's stable ids build on. Nothing here changes bundle
  structure.
- L7 overlaps T25.F11 and is not decided here.

## Natural analogs

- One genome encodes both the body plan and the nervous system.
- The egg's cytoplasm (maternal mRNA, mitochondria) is the birth payload,
  separate from the nucleus.
- How much a parent passes through the egg is itself heritable and differs
  between species (maternal effects).

## Shape of the work (assistant proposal)

Three rows, after T25.F12, in a new track, T29 Heredity (user, 2026-10-22,
multiple choice; not yet added). The track states that it supersedes
T26.F01's record criterion:
1. **Zygote builder and birth payload**, behavior-preserving: one builder
   for founders and children, the payload object, stable learnable ids,
   provisioning inside the builder. Every pin holds; any moved pin is a
   defect.
2. **Body genes**: `BodyGenome` in `CreatureGenome`, per-part supply, the
   Body domain and its operators, the gate deleted, `lineage_id` moved to
   observation, founder 86 and the rate re-pinned. Predeclared re-pins:
   draw streams, trajectories, genome format, both epochs.
3. **Memory inheritance mask**: the gene, founder all-on, the constructor
   reading it. Pins move only through the extra unit, if row 2 has not
   already counted it.

Verification for each: viability first (founder and supply change), gate +
goal; mutation gate and slow tests run. Goal readings: body-gene mutation
counts by gene, the capacity, share and colour distributions beside T26.F02's
baseline, kin-tag diversity, and mask bit frequencies.

## Open

- L7: whether a sensor declaration should stop being its own domain and be
  owned by what reads it (the VM node's declaration list, the Graph node's
  bundles). Decide after T25.F11 lands.
- Whether T27.F02's signals join the birth payload under their own mask.
- For the spec owner: the Body operators' draw order and the per-part
  supply's effect on existing draw streams; the stable-id representation.
