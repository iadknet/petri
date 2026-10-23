# Petri architecture

**State**: desired. This file describes where the system is going, not what
the code does today. [A8 Gaps](#a8-gaps) lists every place the code differs
and the roadmap row that closes it.
**Last updated**: 2026-10-22

## Use this file

1. **Plan** a track, row or spec: read A1–A7 and cite the sections the work
   touches and the A8 gaps it closes or preserves. Work that changes the
   desired contract (A1–A7) edits this file in the same change and adds an A8
   line naming its row; work the desired state already allows only cites it.
2. **Implement** toward the cited sections, never toward a shape A8 lists as
   today's.
3. **Review**: a change that adds an import A3 does not allow, gives a concept
   a second owner (A5) or leaks one domain's internals into another is a
   finding. Code that already used an A8 gap is pre-existing; a new use of a
   gap is added coupling unless the spec declares it with the gap's owning
   row ([Review](workflow.md#review) grades both).
4. **Close**: delete the A8 lines, or parts of lines, the feature closed. A8 is
   empty when the code matches this file.

The workflow enforces each step: [Plan, Review, Close](workflow.md). Sections
keep their `A` numbers so specs can cite them; gaps keep their `G` numbers
until deleted. How the code drifted lives in the strategy notes A8 links,
never here.

## A1 Rules

| Rule | Here |
| --- | --- |
| One owner per concept | One module builds, validates and edits a concept; every route to it calls that owner. Two routes never carry two rule sets. |
| Dependencies form a graph | A3's arrows are the allowed imports. No cycle between domains; siblings with no path between them never import each other. |
| Composition over inheritance | Aggregates are composed of parts (genome = brain + body chromosome; creature = genome + runtime + world-facing state). Variation is a closed enum with its behavior in one module (`Wiring`) or one value type reused in several slots (`Learning`); no trait hierarchies for domain objects. |
| Fewest assumptions, most evolvability | When two designs are plausible, make both evolvable (live and silent births, two learning slots), inside clean layers. |
| Sampling apart from structure | Mutation samples; the owning domain validates and applies deterministically. A birth's randomness lives only in mutation policies. |
| Derive, don't duplicate | Analysis results (learning layout, caches) derive from the genome once per birth and are never stored in the genome type. Runtime state and telemetry derive from applied behavior. |
| Natural analog | Every mechanism names one ([master roadmap](roadmap.md)). |
| No backward compatibility | Genomes, configs and pins move when the design moves, predeclared. |

## A2 System

```mermaid
flowchart LR
  FE["frontend"] -->|"HTTP + WebSocket, typed protocol"| SRV["v3-server"]
  SRV --> CORE["v3-core"]
  CLI["v3-cli"] --> CORE
  LAB["v3-lab"] --> CORE
  TEL["v3-telemetry"] -->|"telemetry-seams"| CORE
  SRV -.->|"telemetry feature"| TEL
  CLI -.-> TEL
  LAB -.-> TEL
  TEL -->|"OTLP"| STACK["telemetry/ stack"]
```

| Part | Owns | Never |
| --- | --- | --- |
| `v3-core` | the simulation library: world, genome, mutation, runtime, creatures, readers | I/O, transport, telemetry export, lab selection, a dependency on another crate here |
| `v3-telemetry` | run identity, OTLP export, metrics, traces and windows read through `telemetry-seams` | changing a run or drawing a simulation RNG |
| `v3-cli` | commands, benchmarks and measurements (`bench`, `measure`, `inspect`) | simulation rules |
| `v3-server` | HTTP and WebSocket transport, sessions, query projections (the creature inspector's brain view among them), its own wire types | simulation rules; a core struct reused as a wire shape beyond the shared shapes below |
| `v3-lab` | capability assays under the [lab contract](lab.md) | being depended on; lab selection, tasks or solutions reaching production |
| `frontend` | rendering and controls over the typed server API ([frontend rules](../frontend/AGENTS.md)) | fabricating runtime state |

**Shared shapes.** Transport may carry exactly these core types as they
serialize: `Direction`, `Position`, the id types and the config schema (a
recipe is that schema). The list is explicit; moving a type into contracts
does not add it. Everything else, the genome included, reaches the frontend
through a projection `v3-server` owns. The genome and runtime owners resolve
the facts it shows (composed weights, patterns, learners) through their own
read APIs; the server selects, structures and serializes them and computes
no simulation rule, and no frontend code recomputes one; the frontend's types are generated from that projection or held
by a field-by-field parity test, never a hand copy. Telemetry
may also export the genome's canonical serialization as an identity record
for tools, never for the frontend.

## A3 v3-core domains

An arrow is a direct import a domain may make; nothing is allowed
transitively. Every domain may import `contracts`; every domain except
`contracts`, `genome` and the energy ledger may import `config`; readers may
import every domain. The unit is the domain, not the module path:
`creature/genome/` is the genome domain, so `crate::creature::genome` is a
genome import. Creature and simulation each have internal parts with their
own arrows.

```mermaid
flowchart BT
  ledger["energy ledger"]
  kernel["world (kernel)"]
  patterns["patterns"] --> kernel
  genome["genome"]
  mutation["mutation"] --> genome
  sensors["sensors"] --> kernel
  runtime["runtime"] --> genome
  runtime --> sensors
  runtime --> ledger
  subgraph creature["creature"]
    body["body"]
    aggregate["aggregate"]
    birth["birth"]
  end
  body --> genome
  aggregate --> body
  aggregate --> genome
  aggregate --> runtime
  aggregate --> ledger
  aggregate -->|"summary types"| mutation
  birth --> aggregate
  birth --> mutation
  subgraph simulation["simulation"]
    scheduling["scheduling"]
    mechanics["mechanics"]
  end
  mechanics --> aggregate
  mechanics --> kernel
  mechanics --> ledger
  mechanics --> birth
  scheduling --> mechanics
  scheduling --> sensors
  scheduling --> runtime
  scheduling --> aggregate
  scheduling --> kernel
  scheduling --> birth
  scheduling --> patterns
  scheduling -->|"summary types"| mutation
```

| Domain | Owns | Today |
| --- | --- | --- |
| contracts | shared vocabulary: ids, positions, directions, world actions, action queue, routing; **sensor descriptors** (fields, field sets, geometry, descriptor-to-source addressing, sensor key sets, the perception-requirements value); **brain I/O vocabulary** (one catalog of action kinds, their vote sinks, parameters and output capabilities, every width derived from it; today four directed kinds of nine plus `Eat`, `Terminate`, `Decide`); the dispatch-history query mutation reads; deterministic hashing | `contracts/`; G1, G2, G8, G11 |
| config | configuration schema and defaults, pattern parameters included | `config/`; G10 |
| energy ledger | energy-accounting primitives the runtime, creature and simulation share: death cause, debit and observation helpers, cognition energy observation | G7 |
| world (kernel) | grid, food resources, fertility, occupancy, paint; world-edit primitives | `kernel/` |
| patterns | world pattern generators | `patterns/` |
| genome | heritable data and its deterministic analysis: A4's chromosomes, edit transactions and the inheritance mappings they return, wiring, structural annotations (declared and wired sensor keys, perception requirements), validity, the learning layout | `creature/genome/`; G12, G15 |
| mutation | supply law, event dispatch, operators and birth policies; samples edits and calls genome edit APIs | `mutation/`; G2, G13 |
| sensors | perception assembled from the world, creature-visible views and the perception-requirements value, all passed in explicitly | `sensors/`; G9, G20 |
| runtime | brain execution: mesh pass loop, VM, graph evaluation, vote selection, action decode, plasticity values and update law, its own state types, brain-execution traces | `runtime/`; G7, G8, G16 |
| creature: body | body rules (move factor, upkeep, energy scale, energy cap) read from body genes | `creature/body.rs`, `phenotype.rs`, `colour.rs`, `identity.rs`; G4, G5 |
| creature: aggregate | `CreatureState`: composes the genome, runtime state and world-facing state (held signals and carried slots included) and keeps its per-life invariants | `creature/state.rs`; G8, G9 |
| creature: birth | the birth transaction and the founders | `simulation/actions/reproduction.rs`, `simulation/seeding.rs`, `creature/founder.rs`, `creature/newborn.rs`; G4 |
| simulation: scheduling | tick order and phases (A6), RNG stream allocation, seeding with world context, records derived from applied results (action history, reward inputs, run totals) | `simulation/tick.rs`, `simulation.rs`, `outcomes.rs`, `stats.rs`; G18, G19 |
| simulation: mechanics | action resolution as transactions (move, eat, predation, giving, delivery, and T28's carrying and building), the action cost law, removal settlement, one applied result per action | `simulation/actions/`, executors in `simulation/tick.rs`; G18, G19 |
| readers | censuses and indicators over structural and runtime views; simulation behavior never depends on them (telemetry, server, CLI and lab read them) | `neighborhood/`, `creature/*_census.rs`, `creature/drives.rs`, `creature/ring_rules.rs`; G14 |

## A4 Creature composition

```mermaid
flowchart TD
  CS["CreatureState (aggregate)"] --> G["CreatureGenome"]
  CS --> RT["runtime state: learned values, traces, memory, compute state, dispatch record"]
  CS --> W["world-facing: position, heading, energy, age, generation, lineage label, held signals, carried slots"]
  G --> BR["brain chromosome"]
  G --> BO["body chromosome: colour, kin tag, capacity, steal share, give share, memory provisioning mask, memory uptake mask"]
  BR --> N["mesh nodes"]
  N --> D["sensor declarations"]
  N --> B["backend: VM or Graph"]
  B --> GR["Graph (aggregate root)"]
  GR --> E["edges on surfaces: compute inputs and sinks"]
  GR --> BU["bundles: records with wiring, scale, learning slots"]
  E -.->|"membership by occurrence"| BU
```

- `genome_size` = brain units + body units: one explicitly defined economic
  measure that drives costs and mutation supply.
- A bundle never stores its members: a member is an edge linked to the bundle
  record by its occurrence (surface and position). The Graph is the one
  structural owner.
- A member's weight is `scale × pattern + modifier` for every wiring.
- Three addresses, three lifetimes: an occurrence (surface, position) is valid
  for one graph revision; a learnable identity lasts as long as structural
  correspondence; a runtime slot is private to one compiled layout. Inspectors
  read resolved views, never one of these addresses across its lifetime.

## A5 Owners

| Concept | Owner | Never |
| --- | --- | --- |
| sensor fields, field sets, geometry | contracts sensor descriptor | rediscovered by mutation, wiring or readers |
| vote and action vocabulary | contracts brain I/O | defined inside one backend |
| declaration–use cross-references on a node | the node's edit transaction | kept by hand inside a mutation category |
| edges | Graph backend, through its edit transactions (validation, inheritance mappings) | edited outside the container |
| inheritance correspondence | every Graph edit returns how inherited values map onto the edited structure: an unchanged locus keeps its identity, a copy gets a new identity naming its source, a detach composes `scale × pattern + modifier` into the lone edge, a removal drops it, and a source change or a weight or scale write invalidates it; mappings compose across the committed edits of one birth and vanish with a rejected edit; the learning layout exposes identities; birth applies the composed mapping to the payload | stable keys alone; one identity shared by two learnable numbers; a birth-side branch per structural operator |
| bundle and membership | Graph backend: bundle record, member link by occurrence | a bundle holding its members |
| wiring (`Plain`, `Ring`, `DirectionPair`) | genome: closed enum; member coordinates, pattern, destination requirements, plan enumeration and legality in one module | the input contract knowing which action receives a field |
| learning settings | one `Learning` value type: lone edge `Option<Learning>`; bundle `scale_learning` and `member_default`; member policy `Inherit`, `Static` or `Own` plus `credits` | per-edge-only config; parallel flags for one idea |
| learning layout (learners, identities, effective `Learning`, contributors) | genome analysis, once per birth after mutation | runtime discovery; edge-only discovery; per-visit `Inherit` lookup |
| learned values, traces, update law | runtime plasticity | genome type |
| mutation supply | mutation engine: `Binomial(units, rate)` per chromosome, one rate | a per-trait chance |
| event dispatch (Topology, VM, Graph, InputRef, Body) | mutation engine: selection only | owning data or invariants |
| birth randomness (live or silent, learning draws, trait steps) | mutation policies | a trait drawing its own RNG |
| offspring construction | creature birth transaction: draft (`Zygote::founder`, `Zygote::conceive`), then finalize | the simulation layer; a second builder |
| birth payload (learned seeds keyed by learnable identity, parent memory) | birth transaction | the genome type |
| parent dispatch history | runtime records it and implements a narrow query (executed node identities in a requested window, resolved lazily) defined in contracts; the birth transaction passes it to mutation as a named input | mutation importing runtime or the aggregate; the record's storage layout in contracts |
| inherited-store policy | each store owns its policy and names whose genes act: seeds by the child's `Learning.lamarckian`; a memory slot passes when the parent's provisioning bit and the child's uptake bit are both set (user, 2026-10-22) | one universal rule |
| provenance (parent, generation, lineage label) | the zygote; the lineage label is observation-only | the body |
| energy cap and provisioning at birth | birth finalize | after construction |
| energy debit and death cause | energy ledger | simulation-owned types imported by runtime |
| world state | world (kernel) | |
| perception | sensors, from views and the perception-requirements value genome analysis derives | a sensor reading the genome or the whole aggregate |
| tick order and phase contracts (A6) | simulation scheduling | a mechanism choosing its own phase |
| action resolution | simulation mechanics: each action is one transaction | resolution code beside the scheduler |
| applied action result | mechanics returns one per resolved action; scheduling derives action history, reward inputs and run totals from it; runtime owns learning updates; telemetry reads it. Lifecycle events with no resolved action (a delayed delivery, a world-update removal) are records of their own | a second success classification |
| multi-party transactions (give, steal, carry, build, death drops) | mechanics, over kernel world-edit primitives and aggregate operations: the transfer or material operation is atomic (a rejected one moves nothing) and settles exactly once; the attempt's effort is charged and committed either way, and the applied result carries it | the kernel or one creature enforcing a multi-party invariant |
| action cost law | mechanics composes it from config and body modifiers (the body supplies factors such as the move factor, never a composed effort); the ledger applies the arithmetic; scheduling aggregates totals | costs composed on the aggregate, in body rules or in runtime |
| mechanics inputs | scheduling hands mechanics narrow context (the actor, its neighbours, the world cells and the ledger an action touches); mechanics settles removals it causes | mechanics taking the whole simulation or calling back into scheduling |
| material stocks (food and barriers carried, placed, dropped, lost) | mechanics, with typed stock, transfer and sink records apart from energy | material booked as energy |
| held signals | runtime emits committed writes; the aggregate stores them per life; sensors read the previous tick's values, frozen | a signal visible in the tick it is written |
| carried slots | the aggregate stores them; mechanics changes them only through transactions | the world or sensors editing inventory |
| RNG streams | scheduling allocates named streams at run start (queue order, reproduction, food growth); each mechanism draws from its own | one stream seeding another per tick; a reader or telemetry advancing a simulation stream |
| traces | runtime owns brain-execution traces; scheduling composes tick records (applied results, phase timings) around them | runtime types carrying simulation or aggregate records |
| telemetry export | `v3-telemetry` through `telemetry-seams` | core code exporting |
| wire shapes | `v3-server` transport | core structs on the wire |

## A6 Flows

**Tick.** Scheduling owns the order and each phase's contract; a phase may
coordinate several owners, each through its own API.

| Phase | Reads | Writes and commits | Transitions |
| --- | --- | --- | --- |
| `food_growth` | world | food, from its own stream | — |
| `world_update` | bodies, world | upkeep, ageing, gestation progress, the shared-memory snapshot and decay | removals settle once; reward energy is snapshotted after this phase |
| `sensor_assembly` | a frozen world and the previous tick's held signals | perception per creature | queue order drawn from its stream |
| `cognition` | perception, own runtime state | own runtime state, shared memory and cognition energy debits (through the ledger), and one decision per creature (votes, parameters, memory and signal writes), in parallel | — |
| `actions` | decisions, in queue order | in each parent's queue slot a due birth is delivered or lost before its action; mechanics transactions, each with one applied result; signal writes commit | newborns join the next tick's queue; committed signals become readable next tick |
| `reward_learning` | applied results, energy since the snapshot | learning updates (runtime) | — |

**Birth.**

```mermaid
flowchart LR
  P["parent + world context"] --> C["Zygote::conceive: genome draft, mutation policies (dispatch record as a named input), payload capture, provenance"]
  F0["seeding"] --> Z0["Zygote::founder"]
  C --> FIN["finalize: learning layout, payload by the child's genome, genome-derived caches (reused when no event applied), runtime built fresh through the payload policies, per-life resets, heading, energy provisioning"]
  Z0 --> FIN
  FIN --> CS["CreatureState"]
```

**Mutation.** The engine draws events per chromosome, dispatches each to a
category, and the operator samples one edit. For a new sensor or bundle the
birth policy asks the Graph for the admissible wiring plans, samples one and
draws the birth law and learning; the Graph validates and applies the plan
atomically.

**Learning.** Genome `Learning` settings → learning layout (pure, once per
birth) → runtime values and update law (contribution aggregation, then one
scalar update per learner). Readers read the layout.

## A7 Boundary rules

Reviewers cite these by number.

1. Imports follow A3's direct arrows only; no import cycle between domains
   (A3's unit).
2. `genome` imports only `contracts`.
3. `mutation` and `runtime` never import each other; `sensors` imports none
   of `genome`, `runtime` and `creature`; creature body imports neither
   `mutation` nor `runtime`; simulation calls mutation only through birth.
4. Each concept in A5 has its one owner; a new route to it calls that owner.
5. Mutation samples, owners apply; no birth randomness outside mutation
   policies; every mechanism draws from its own named stream.
6. Derived data never lives in the genome type; runtime state and telemetry
   derive from applied behavior.
7. Simulation rules, energy accounting, telemetry, transport, I/O and
   configuration parsing stay apart; telemetry and test-only hooks sit behind
   `telemetry` and `telemetry-seams`.
8. A2's crate rules: `v3-core` depends on no other workspace crate; lab code
   never reaches production. An internal struct reused as a transport or
   telemetry shape beyond A2's shared shapes, or a field read raw at many sites where one resolver
   belongs, is a boundary finding.

## A8 Gaps

Reconciliation, not history. Each line is a difference between today's code
and A1–A7, with the row that closes it. Delete the line when the code matches.
Design notes: [sensor wiring](strategy/group-creation-unification-2026-10-22.md),
[bundle learning](strategy/bundle-learning-ownership-2026-10-22.md),
[heredity](strategy/heredity-architecture-2026-10-22.md).

| Gap | Today | Desired | Closed by |
| --- | --- | --- | --- |
| G1 | Three routes wire a sensor with three rule sets: `mutation/input_ref/mod.rs:310` `apply_add`, `mutation/graph/recruitment.rs:125` `Admission`, `mutation/graph/group.rs:121,159`; fact-only branches in `creature/genome/cgp/container.rs:695` `pattern_at` and `:1304` `detach_at`; hosting split into `:1105` `can_host_group` (every directed kind's direction sinks) and `:1117` `can_host_facts`; non-world sensor widths in `mutation/compound.rs:21-24` | A5 wiring, bundle, sensor descriptor | T25.F11 |
| G2 | Brain I/O vocabulary outside contracts: votes in `creature/genome/vote.rs`; `OUTPUT_SLOT_COUNT` in `runtime/types.rs`, imported by `mutation/sampling.rs:7`, `mutation/graph/recruitment.rs:16`, `mutation/input_ref/mod.rs:16`; route resolution and jump targets imported from runtime by `mutation/topology/routing.rs:12` and `mutation/vm/operators.rs:728` | A3 contracts; A7 rule 3 | the vocabulary the sensor descriptor needs (vote and outcome counts, output slot count): T25.F11; route resolution and jump targets: T30.F05 |
| G3 | Learning configured on edges only (`creature/genome/mod.rs:180` `PlasticityConfig`); group `inherit_scale` and `retention` beside it (`creature/genome/cgp/group.rs:122`); `Scope::Group` branches in `runtime/plasticity/learned.rs:166-278`; learners discovered from plastic edges (`creature/genome/companions.rs:57`, `learned.rs:563`) | A5 learning settings, learning layout | T25.F12 |
| G4 | No birth transaction: `spawn_child` in `simulation/actions/reproduction.rs:376` coordinates a birth; `simulation/seeding.rs:95` is a second builder; learned seeds ride in the genome (`creature/genome/cgp/container.rs:245-246`, captured by `simulation/actions/cgp_reproduction.rs:15`); `cap_energy` runs after construction (`reproduction.rs:568`); traits draw their own RNG (`creature/identity.rs:43`, `creature/body.rs:80`) | A5 offspring construction, payload, inheritance correspondence, randomness; A6 birth | T29.F01 |
| G5 | Body outside the genome: `HeritableBody` (`creature/body.rs:26`) on `CreatureState`; capacity and shares mutate by a per-birth chance (`body.rs:127`); colour and kin tag gated on brain mutation (`reproduction.rs:519`); `lineage_id` in the body's identity (`creature/identity.rs:7`); mutation imports the body's `Phenotype` from creature (`mutation/phenotype.rs:4`) | A4 body chromosome; A5 supply, provenance | T29.F02 |
| G6 | Every memory slot passes to the child (`reproduction.rs:386`) | A4 body chromosome masks; A5 inherited-store policy | T29.F03 |
| G7 | Energy primitives live in `simulation/energy_accounting.rs` and are imported by runtime (`runtime/mesh.rs:25`, `runtime/vm.rs:11`, `runtime/cgp/execute.rs:14`, `runtime/types.rs:76`) and creature (`creature/state.rs:171,315-317`): with G16, runtime↔simulation and creature↔simulation cycles | A3 energy ledger; A7 rule 1 | T30.F02 |
| G8 | Runtime state types live in the aggregate: `GraphRuntimeState` and `DispatchRecord` in `creature/state.rs:77,33`, imported by runtime (`runtime/mesh.rs:16`, `runtime/cgp/execute.rs:4`, `runtime/traced_mesh.rs:7`); mutation embeds `&DispatchRecord` in `ParentExecuted::Record` (`mutation/reachability.rs:21,33`). With the aggregate's own imports of runtime and mutation (`creature/state.rs:17-18`: runtime, and mutation summary types, both allowed) these close creature↔runtime and creature↔mutation cycles | A3 runtime, creature; A4 runtime state; A5 parent dispatch history | T30.F07 (after T29.F01, which keeps the record a named mutation input) |
| G9 | Sensors read the whole aggregate and a creature helper (`sensors/reducers.rs:8-9`, `sensors/static_inputs.rs:3-5`) while the aggregate imports `sensors::reads` (`creature/state.rs:19`): a creature↔sensors cycle | A3 sensors (the key-set type in contracts); A7 rule 3 | T30.F03 |
| G10 | `config/simulation.rs:360-361` imports pattern types, `patterns/mod.rs:18` imports kernel, `kernel/fertility/mod.rs` imports config: a config→patterns→kernel cycle | A3 config | T30.F04 |
| G11 | `kernel/ordinary_food/stream.rs:10` imports `creature::identity::splitmix64` | A3 contracts | T30.F04 |
| G12 | The genome validity gate lives in `creature/parseability.rs`, imported by `mutation/engine/mod.rs:9` | A3 genome | T30.F05 |
| G13 | The InputRef category edits declarations and Graph edges by hand (`mutation/input_ref/mod.rs:338` `connect_declared_input`); no node edit transaction owns their invariants | A5 declaration–use cross-references | T25.F11 (the minimum transaction its sensor additions need); T30.F06 (every other edit path; heredity note L7) |
| G14 | Production code reads readers: the aggregate computes its sensor caches through `creature::sensor_census` at construction (`creature/state.rs:16,250`); the pair-group inheritance witness reads `creature::drives::PATH_WEIGHT_FLOOR` (`simulation/actions/reproduction.rs:128`) | A3 genome (structural analysis) under readers | witness: T25.F11 (moves it to applied outcomes or retires it); sensor caches: T30.F03 |
| G15 | Genome analysis draws randomness: `vm_backward_slice_random` and `vm_forward_slice_random` (`creature/genome/analysis.rs:180,229`), called by `mutation/vm/operators.rs:879,898` | A1 sampling apart from structure; A7 rule 5 | T30.F05 (mutation picks the anchor and calls the deterministic slice) |
| G16 | A runtime trace carries simulation and aggregate records: `creature::action_log::ActionLogEntry` and `simulation::stats::TickPhaseTimings` (`runtime/trace/domain.rs:63,107`) | A5 traces | T30.F02 |
| G17 | The genome's internal layout is the frontend's API: the inspector endpoint serializes `CreatureGenome` (`v3-server/src/http/creature.rs:67,505`) and step traces serialize `VmInstruction` (`v3-server/src/transport/sample_assembler.rs:162`); `frontend/src/types/genome.ts` copies `Membership`, `Scope`, `PlasticityConfig` and `EdgeGroup` by hand, read by five inspector files, with no fixture keeping them in step | A2 shared shapes | T30.F01 |
| G18 | Two result vocabularies: `ActionResult` (`creature/action_log.rs:15`) for history and a success boolean for rewards (`simulation/outcomes.rs:53` `record_action_result`); no single applied result that history, rewards and totals derive from | A5 applied action result | T30.F09 |
| G19 | Scheduling and mechanics share `simulation/tick.rs`: per-action executors (`execute_eat` `:940`, `execute_move` `:995`, `execute_reproduce` `:1059`, `execute_steal_energy` `:1149`, `execute_give_energy` `:1184`) sit beside `run_tick` (`:1385`), and action energy flooring and removal run in the scheduler (`:1303-1320`); mechanics in `simulation/actions/` take the whole `Simulation` (`giving.rs:7`, `predation.rs:4`, `reproduction.rs:15`) and predation calls `sim.remove_creature`; move effort is composed on the aggregate (`creature/state.rs:302` `move_effort`) | A3 simulation parts; A5 action resolution, mechanics inputs, action cost law | T30.F08 |
| G20 | Sensors read the genome to decide what to assemble (`sensors/perception.rs:182` `genome_uses_extended_perception`, `sensors/typed_food.rs:72` `genome_uses_typed_local_food`) | A3 sensors; A7 rule 3 | T30.F03 |
| G21 | One stream seeds another every tick: the queue shuffle draws from `sim.rng` (`simulation/tick.rs:1435`) and reproduction is reseeded from it (`:1451`), so reproduction draws depend on the queue's | A5 RNG streams | T30.F10 |
