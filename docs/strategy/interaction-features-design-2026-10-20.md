# Interaction Features Design Note (2026-10-20)

User decisions recorded 2026-10-20 (§7). No roadmap rows are committed yet. Prior art,
quotes and citations are in the companion
[interaction features prior-art note](interaction-features-prior-art-2026-10-20.md);
this note cites it by section (PA §n).

## Answer

The user proposed five features: (A) colour vision, (B) signal channels,
(C) giving energy, (D) evolvable energy capacity and (E) carry slots. They
become seven rows in three tracks, grouped by the code surface each one
touches, so that three sessions can run at once:

| Track | Rows | Surface |
| --- | --- | --- |
| T26 Body Traits | F01 one heritable body record, plus the energy-credit and move-cost hooks (behavior-preserving); F02 evolvable energy reserves (D) | `CreatureState`, energy accounting, move cost |
| T27 Social Signals and Exchange | F01 colour vision (A); F02 signals (B); F03 giving energy, with steal taking a fixed bite (C) | perception and NearbyCreature; vote catalog |
| T28 Niche Construction | F01 carrying and food caching (E1); F02 digging and building barriers (E2) | world grid, vote catalog, move cost |

Waves (a wave starts when its dependencies close):

1. T26.F01 ∥ T25.F10 ∥ T27.F01
2. T26.F02 ∥ T27.F03
3. T27.F02 ∥ T28.F01
4. T28.F02

T25.F10 (group genes cost as genes) joined the plan from §8.

Pushback offered and the user's ruling (2026-10-20, §7):

- **B.** The research supports one held broadcast signal; no study compared
  channel counts (PA §2). The user kept **8 held channels**, sensed per
  NearbyCreature slot.
- **C.** Steal's amount parameter is not copied. Give and steal move one
  fixed bite per action, and the multi-action tick lets the amount evolve
  through repetition. Decided on the evidence; this is not a user ruling.
- **D.** The user's neutral point is kept, plus a capacity upkeep that is
  also neutral at 200, so a creature that never moves does not get a large
  store for free (PA §4). Decided on the evidence.
- **E.** The research argues against slot-index parameters. The user kept
  **slot-index parameters and per-slot 0/1/2 sensors**. Directions use the
  drive plus eight direction sinks, which the user accepted. All barriers
  are movable with no decay.

## 1. Problem, constraints, criteria

The goal is richer creature interaction toward the program's success
definition (many coexisting ways of making a living, and cognition):

- seeing each other;
- signalling;
- sharing energy;
- body storage tradeoffs;
- reshaping the world.

Constraints are the standing rules in `docs/roadmap.md`:

- **Natural analog.** It is named first; only geometry is built in, never a
  computation. There is one sensed exception, self-referent matching (user
  ruling B, §7), and ordinary condition readings of another body (energy,
  readiness) are facts, not comparisons.
- **Evolvability, not targets.** A feature never aims at a specific behavior.
- **Cost.** Every capacity has an applied cost.
- **Pressure integration.** Creature-authored pressures go into all three
  goal worlds, with a focused action-to-world fixture, and occurrences are
  reported including zero.
- **No backward compatibility.**

Decision criteria:

- evidence that the mechanism shape becomes behavior in Petri or in the
  literature;
- the size of the code collision surface, which decides concurrency;
- the risk of a runaway free advantage;
- the smallest change that answers the feature's question.

## 2. What the code does today

These facts were mapped on 2026-10-20; paths are under
`crates/v3-core/src`.

- **Colour.**
  - Each creature has six HSL channels (`creature/state.rs:183-187`),
    mapped to RGB by `mutation/phenotype.rs:60-79`.
  - The channels are mutated at each birth when the brain mutated
    (`reproduction.rs:517-528`).
  - Colour is sensed only as `PHENOTYPE_SIMILARITY`, which compares the six
    raw channels, i.e. the genotype, not the visible colour
    (`sensors/reducers.rs:410-432`).
  - The kin tag is separate: `identity.kin_tag`, sensed as `KIN_AFFINITY`.
- **NearbyCreature.**
  - Four slots of nine fields each, width 36 (`contracts/inputs.rs:43-57`).
  - It ranks visible creatures within `vision_radius` 5.
  - Fields reach behavior through T25.F04 direction-pair groups, which were
    retained 30k–40k ticks on all three worlds (T25.F04 readings).
  - A `PerceptionSnapshot ≤ 256 B` test sits at 252 B
    (`sensors/perception.rs:203`). It is an inherited budget with no
    recorded reason.
  - The nearby field names exist only in TypeScript.
- **Memory.** 16 slots, written by `WriteSlot`/`ClearSlot` graph sinks and
  VM opcodes. The value is last-write, held, with optional decay, inherited,
  and visible only to the creature itself.
- **Actions.**
  - There are four vote kinds. The directed kinds (Move, Reproduce, Steal)
    each have a drive plus eight direction sinks, and the selection is drive
    + best direction − bar (`runtime/vote_select.rs:59-96`).
  - Adding a kind touches about 25 sites across core, telemetry, server,
    CLI and frontend. The vote-sink index arithmetic is mirrored by hand in
    `frontend/.../voteSinks.ts`.
  - `StealEnergyAmount` is an overwrite parameter, unbounded and usually
    unwired. A zero-amount steal with a victim counts as `Transferred` and
    as a success (`tick.rs:1101-1120`).
- **Energy.**
  - `max_energy` 200 is global. An upper-cap-and-book-loss step appears at
    three sites (eat, steal attacker ×2). The sites differ in floor,
    observation order and signed flows (review finding 2).
  - The capacity is the denominator for self, nearby and previous-outcome
    sensors.
  - Founder reproduce thresholds are fractions of it.
  - The v3-lab arena suppresses births with `max_energy + 1`.
  - Move costs a flat 1.6 on every attempt, and nothing modifies it.
- **Heritable non-brain state.** Phenotype and identity sit loose on
  `CreatureState`. Its constructor takes 10 positional arguments at 55 call
  sites.
- **World.**
  - Food is a continuous density per type per cell, clamped at 1.0, and
    several types can share a cell.
  - Barriers are a `Grid<bool>`, opaque to sight, never hold food, and
    cannot be occupied.
  - Only the paint tool mutates barriers. The server's projection
    publication already compares the barrier mask and advances its static
    revision (`v3-server/src/query/projection.rs:66-112`), so creature edits
    will reach clients without new machinery (review finding 3).

## 3. Options and precedent per feature

### A — Colour vision (T27.F01)

| Option | Evidence | Verdict |
| --- | --- | --- |
| A1 RGB per NearbyCreature slot (+3 fields), own RGB as self inputs, `PHENOTYPE_SIMILARITY` and `LINEAGE_MATCH` retired, `KIN_AFFINITY` kept as self-referent matching (user ruling B) | User's shape. Slot fields reach behavior via pair groups (T25.F04). Self-referent matching is a real mechanism (PA §1). | **Recommended** |
| A2 per-direction ring of colour distance to self | PA §1 recommends it: kin discrimination with one edge, ring shape proven live | Rejected: the master's self-referent exception covers sensing a carried tag's match (`KIN_AFFINITY`), not a new built-in colour-distance map; raw RGB leaves that comparison to evolution |
| A3 retina (per-direction RGB, 24 inputs) | Polyworld: colour vision produced movement responses only | Deferred |

**Natural analog.** Colour vision; an animal reads a neighbour's markings.

**Cost.** None needs adding: a wired input already costs one genome unit.

**Recorded expectation.** Heritable colour plus vision alone has not
produced social use elsewhere. Tag cooperation needs repeated encounters
(Riolo 2001). A zero-use reading is a valid result.

### B — Signals (T27.F02): user ruled 8 channels

Options offered:

- **B1 display.** Held sinks offset visible RGB. This was recommended.
- **B2 one or two separate channels.**
- **B3 the user's 8 channels.**

The user chose B3, which keeps heritable colour a pure inherited label.

**Contract shape.**

- A creature holds `signal: [f32; 8]`, written by eight `WriteSignal(c)`
  graph sinks and a VM store opcode, last-write like memory.
- The value is held until rewritten, with no decay.
- Others read last tick's value as eight more fields per NearbyCreature
  slot.
- The newborn's signal starts at zero, never inherited.
- Signal values travel in `MeshOutput` and are applied after the mesh, as
  `heading` is; there is no new `&mut` argument (audit #11).

**No cost (user decision, 2026-10-21).** A creature writes sinks and others
read them; nothing else. An earlier draft added a per-tick Σ|signal| cost
from prior art (Wagner 2000, PA §2), which was never a user decision; it is
removed with its amplitude/gain check, rate tuning, ledger bucket and
configurable decay. Signal writes keep the existing graph/VM compute
charging and affordability that memory writes pay; the NoOp effort floor
applies only to world actions.

**Recorded consequences.**

- With A, NearbyCreature grows from 36 to 72 fields (18 per slot: two
  coordinates and 16 facts, after lineage match and similarity are retired).
  A directed slot recruitment builds 12 pair members plus 16 drive facts.
- The perception snapshot's nearby array grows by 144 B, so the spec
  measures tick time.
- Meaning is never encoded in a channel.
- Zero use is a valid result. The reading reports per-channel wiring and
  value variance, not emission rate.

**Natural analog.** Song, scent and display: a creature broadcasts a state
its neighbours can perceive.

### C — Giving energy (T27.F03)

**Shape.** `GiveEnergy` is a directed kind (drive plus eight directions),
the analog of trophallaxis and parental feeding.

- The giver moves one fixed **bite** per action (config), losslessly. The
  recipient is clamped at capacity and the excess is booked as loss.
- `StealEnergy` moves to the same bite rule, and the `StealEnergyAmount`
  parameter is retired. This owns the carried "unbounded steal request"
  finding.
- A zero transfer is a failure, never `Transferred`.

**Precedent.** Taylor-Davies et al. 2024: a zero-sum "feed" to an adjacent
agent evolved as parental care in a neuroevolution grid world (PA §3).
Expect parental care first, and do not target it.

**Rejected.** A free amount parameter: it reads 0 when unwired, which is
the live steal finding.

### D — Evolvable energy reserves (T26.F02)

**Shape.**

- A heritable `capacity` lives in the body record, mutated in log space and
  bounded to [50, 800] (¼× to 4× of 200).
- Move effort is multiplied by `1 + β·(C/200 − 1)`. Per-tick upkeep adds
  `u·(C − 200)`, which is negative for a small body, with total decay
  floored at 0. Both are linear and neutral at 200, so the founder's
  economy is unchanged.
- β and u come from a short default sweep (the noop-floor pattern), not a
  lab.

**Sensors.** Whether the self energy fraction reads own capacity (hunger
relative to fullness) or the global value is decided by the spec after the
unchanged-brain capacity comparison (review round 5 finding 5). Own-capacity
normalization moves inherited fullness thresholds. The spec also rules on
nearby `ENERGY_RATIO` and the previous-outcome scalings. The global value stays the upper bound of
the evolvable range, so the lab sentinel still holds.

**Precedent and rejected options.**

- Polyworld ties size linearly to capacity and move cost (PA §4).
- A stored-energy move cost (load-proportional, Maloiy 1986) is more
  biological. It is rejected because it shifts every creature's economy and
  breaks the user's neutral point.
- Sub-linear costs are rejected (runaway; genome-bloat precedent).

**Natural analog.** Fat reserves survive lean times but cost to carry
(archived T03.F05).

### E — Carrying (T28.F01 food, T28.F02 barriers): user ruled slot index kept

**Shape.**

- Four slots.
- `PickUp` and `Place` are two directed kinds, each a drive plus eight
  direction sinks.
- Each kind has one slot parameter, `PickUpSlot` and `PlaceSlot`, an
  `ActionParamField` rounded to 0..3. An unwired parameter reads 0, which is
  slot 0.
- Pick-up takes what the target cell holds into the named empty slot: a
  barrier, else one bite of food, recorded as `(type, amount)`. The plan's
  "densest type" rule is a placeholder: the spec justifies it as body
  mechanics or lets the type be chosen (review round 5 ledger: density
  order is not nutrition order in Orchards).
- Carried food cannot be eaten from a slot. It must be placed on the
  ground and eaten there (user ruling, review round 5 finding 6).
- Place puts the named slot's item into the target cell. Food adds density,
  capped at 1.0. A barrier needs a free, unoccupied cell.
- Failures are: nothing there, slot occupied or empty, target blocked. Each
  failure pays the effort floor, and no fallback slot is chosen (archived
  T06.F01).

**Sensors.** Four per-slot self inputs carrying 0 (empty), 1 (food) or
2 (barrier), as the user proposed.

**Other rules.**

- A load surcharge per occupied slot goes through T26.F01's move-cost hook.
  The initial value is the archived 0.05; the spec sets the default.
- Death drops follow the archived T06.F01 contract.

**Research objections, kept on record.**

- An unwired scalar reads 0. That is the live steal finding.
- Neuroevolution handles rounding a continuous output to an index badly
  (Kohl & Miikkulainen 2009, PA §Action-space).
- An ordinal 0/1/2 code tells the brain a barrier is "twice" a food.

The spec reports per-slot use, so a later row can revisit this with
evidence.

**Phasing.** E1 covers food only: caching and seed dispersal (placed food
on a fertile cell grows). E2 extends pick-up and place to barriers
(digging and building). E2 adds no new sinks.

**E2 versus the habitat rule (user ruled option 1, 2026-10-20).** Workflow requires
"preserve each world's distinguishing habitat", and dismantlable walls can
erode Canyon. The options are:

1. **All barriers movable, no decay, Canyon wall count reported.** This is
   the archived decision; it accepts habitat drift as a finding.
2. **Movable, with placed barriers decaying back to open ground.** Beaver
   dams need upkeep (PA §5), but dismantled seeded walls never return.
3. **Seeded terrain immovable, so only placed barriers move.** It needs a
   material source; there is none today.

Ruling: option 1, all barriers movable with no decay. Canyon's wall count is
reported over time, and habitat drift is a finding, not a blocker.
Persistence stays a treatment parameter. The archived T06 note says persistence is an
experimental variable.

## 4. Collision map and decomposition

Shared refactors from the preparatory audit:

- **Heritable body record:** one struct replaces 10 positional arguments at
  55 sites.
- **Energy cap function:** one function for the shared upper-cap step;
  each caller keeps its own floor, order and signed flows.
- **Move-cost hook:** one function replaces the flat charge.

They span lanes, so they land first as T26.F01. It is behavior-preserving
in the T25.F01 shape: any pin that moves is a defect.

Surface-local refactors are folded into the first feature on each surface,
as the workflow allows:

- **Into A:** the nearby field table and its fixture, and the 256 B budget
  removed or re-set on a measured tick time.
- **Into C:** a directed-kind catalog (`VoteSink::Directed(kind, d)`, a
  `voteSinks.json` fixture, charge arrays indexed by action type). It is
  sim-preserving, but its serde change re-pins the recruitment-paths
  fingerprints. C also lands the zero-transfer fix as its own commit with a
  predeclared re-pin.
- **Into A:** the input-bucket table with sampler/universe agreement
  coverage, because A's own RGB inputs are the first new self inputs.
- **Into E1:** `WorldState::place_barrier/remove_barrier` (the server's
  existing projection already detects barrier changes), and E1 extends the
  input-bucket table with its slot inputs.

Each spec predeclares its expected changes by artifact: trajectory pins,
serialization fingerprints, mutation draws or catalog positions, and new
benchmark readings. Stored benchmark baselines stay as the workflow keeps
them. T28.F02 adds no sinks; its changes come from applied barrier
behavior. The refactors make each sink or input feature a declaration; they
do not remove its pin changes.

| Row | Depends on | Collides with | Verification |
| --- | --- | --- | --- |
| T26.F01 body record + hooks | — | — | gate; any moved pin is a defect |
| T26.F02 energy reserves | T26.F01 | T28.F01 (move hook, ordered) | viability first; gate + goal + **goal-long** (runaway is long-horizon) |
| T27.F01 colour vision | — | T27.F02 (nearby width, ordered) | gate + goal |
| T27.F02 signals | T27.F01, T25.F10, T27.F03 | T27.F03 ordered before it | gate + goal + **goal-long**; three-world pressure |
| T27.F03 giving + steal bite | T26.F01 | T28.F01 (vote catalog, ordered) | viability first; gate + goal; three-world pressure |
| T28.F01 carrying + food caching | T26.F02, T27.F03 | — | viability first; gate + goal; three-world pressure |
| T28.F02 digging + building | T28.F01 | — | viability first; gate + goal + **goal-long**; three-world pressure |

Goal-long runs at most once per track, following the rationing rule.

Rows in one wave are concurrent branches, not separated surfaces (review
finding 1). They share some files, so the one merged second reconciles them
at integration. Shared files:

- T27.F03 then T27.F02 edit the graph and vote catalogs in order, not
  concurrently: F02 depends on F03.

The amended rebase step re-runs the declared profiles for that merge. A
goal-long run takes about 3 h 12 min (T25.F04), so the time saved by
concurrency is unmeasured (review round 5 finding 10). Branches should land
shared-catalog work on main before starting expensive final profiles.
Collisions that would cost more than a rebase are ordered through
dependencies instead:

- T27.F01 before T27.F02;
- T26.F02 before T28.F01;
- T27.F03 before T28.F01.

T27.F01 derives RGB at perception time rather than caching it on the
creature, so it does not touch the constructors T26.F01 reorganizes.

Waves (corrected after review rounds 1–5): wave 1 T26.F01 ∥ T25.F10 ∥
T27.F01; wave 2 T26.F02 ∥ T27.F03; wave 3 T27.F02 ∥ T28.F01 (T27.F02 waits
for T27.F03 so the track's long run includes giving); wave 4 T28.F02.

## 5. Concurrency (user approved the amendment, 2026-10-20)

Workflow today re-runs only `make check` after a rebase. A feature merged
second therefore keeps benchmark readings that never saw the first feature,
and the next feature compares against them.

The amendment to the rebase step in `docs/workflow.md`:
when the commits `main` gained change simulation behavior, re-run the
feature's declared gate and goal profiles on the rebased tree before
recording the readings (5–17 min each). This amends the existing step and
adds no parallel machinery.

## 6. Remaining uncertainty

- Whether any of A–C becomes behavior at 2,000 ticks. Expect zero-use
  readings; B and E2 carry goal-long.
- Whether β and u stop a capacity runaway. The sweep plus D's goal-long
  capacity distribution informs it. The distribution is read descriptively,
  with boundary mass and trend. Selection toward the cheap bound can be a
  valid outcome, so a broken tradeoff needs more than mass at a bound
  (review finding 11).
- How placed food interacts with grazing state and fertility. This is E1's
  spec, flagged by the audit.
- Several primary texts were read as abstracts only (listed in PA "Not
  retrieved").

## 7. User decisions (2026-10-20)

1. **Signals:** 8 held channels, sensed per NearbyCreature slot (B3). The
   display-colour option B1 was declined. Later the same day the user chose
   option A for wiring: a slot recruitment wires the eight signal fields as
   drive facts with the slot's other facts. The cost this adds per
   recruitment is noted in §3 B. The user also asked to rethink how genome
   size and complexity are calculated (§8).
2. **Carry slots:** slot-index parameters and per-slot 0/1/2 sensors are
   kept. Directions use the drive plus eight direction sinks.
3. **Barriers:** all movable, no decay. Canyon wall drift is reported as a
   finding.
4. **Social identity sensors (after review round 5):** ruling B. Retire
   `LINEAGE_MATCH` (oracle) and `PHENOTYPE_SIMILARITY` (genotype
   comparison); keep `KIN_AFFINITY` as self-referent matching, which the
   master now names as the one sensed exception to the geometry-only rule;
   leave `REPRODUCE_READY` unchanged.
5. **Workflow:** the feature merged second re-runs its declared gate and
   goal profiles after a rebase onto behavior-changing commits.

## 8. Genome size rethink (opened by the user, 2026-10-20)

The trigger is signal wiring option A. One slot recruitment then adds 28
edges: 12 pair members and 16 drive facts, all born at weight 0. Under
T25.F02's uniform rule, each edge is one unit of `genome_size()`
(`creature/genome/mod.rs:298-326`).

That single number drives four costs:

1. mutation supply, `Binomial(size, 0.0060625)`;
2. carry cost, 1e-4 per unit per tick;
3. replication cost, +0.1 per unit above 80;
4. gestation span.

`complexity()`, a reachability-aware count of live components, drives only
the action-cost multiplier, which is disabled by default.

Options considered (the user chose (b) on 2026-10-20; it is placed as
T25.F10, ahead of T27.F02, and the T25 track notes hold the contract
sketch):

- **(a)** Split the costs by biological role. Copying and mutation scale
  with total length (Avida, Tierra). Maintenance scales with expressed
  tissue (`complexity()`, or edges whose weight has moved off birth).
- **(b)** A group gene costs one unit. This reverses T25.F02's "a synapse
  pays wherever it lives". The user first chose one unit plus exceptions,
  then simplified it the same day to one unit per group with no change
  tracking.
- **(c)** Keep the uniform rule and accept the price of option A.

Natural analog for (b): the genomic bottleneck. A genome is far too small
to specify each synapse, so developmental rules wire many synapses from few
genes (Zador 2019, *Nature Communications*; cited from memory, not re-read
this session).

## Next steps

1. Write the T26–T28 track files and the master priority order.
2. Apply the workflow amendment.
3. Run one Codex `plan-design-review` job, recorded in
   `docs/progress/readings/t26-t28-design-review.md`.
4. Get user OK, then commit.
