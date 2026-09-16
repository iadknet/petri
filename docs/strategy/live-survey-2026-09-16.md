# Live Survey: Mesh Routing and Sensor Reactivity (2026-09-16)

Survey of the world running on `v3-server` at tick 28,315 (binary built from main at 6e9d4db7 on 2026-09-15; T11.F15 to T11.F19 all in). 400 creatures sampled uniformly at random (seed 11) from the 60,074 alive, dumped with `GET /v3/simulation/creature/:id`, and read in vitro with the T11.F14 battery plus a new paired-perturbation probe (Appendix B). The world was paused throughout; nothing was stepped.

## Questions

1. Is complex routing among mesh nodes evolving, and if not, why not?
2. Are creatures reacting to their environment: seeing and seeking food, and do barrier readers react to barriers?

## Verdict

**1. No.** The mesh has grown (median 27 nodes, 10 reachable, 7 executed per creature) and routes do vary with input in 20 to 40% of creatures, but only **2 nodes contribute** to behavior in the median creature (p25 = p75 = 2), the other 5 executed nodes are pass-through detours whose bypass changes nothing, and conditional routing is load-bearing in 13 of 400 creatures (3.3%). The founder is 2 nodes; 252 of 400 creatures run on founder node 1 plus one evolved node (id 4) that swept the population, and that is the whole story of the evolved core. Mutation supply is not the block this time: 15.7% of births change behavior (founder 21.5%; the gen-2,000 population of the 2026-09-07 note was at 1.0%). The block is that nothing a third node could do is rewarded: creatures run fixed action programs whose directions are constants, and the direction encoding makes a sensor-driven direction a needle-in-a-haystack construction (Section 3.4). The one load-bearing inter-node data path in the population is a payload handoff, not a route branch: node 4 copies one fruit-ring slot onto an upstream slot and founder node 1 folds it into a move direction (Section 3.3).

**2. Partly, and narrowly.** Creatures react to what is under their feet (42% change action with `food_here`, 53% with fruit here), to their energy (84%), and to nearby-creature identity (40%), age, and memory. Of the four neighbor rings, three are unconsumed: toggling one primary-food ring slot changes the action in 6 creatures of 400, one occupied slot in 16, one barrier slot in 1. The fruit ring is consumed, through exactly two slots (N and W), and only into a direction parameter. That gives the population one seeking rule: **fruit to the West, move West** (968 of 1,563 moves, 62%; 215 of 400 creatures carry it at over 50%, 205 at over 90%); fruit in any other direction does not steer the move (N is the unwired default; E, SE, S, SW and NW are hit in 0 to 2% of moves), and primary food, a graded ring, and the area-food gradient all land at chance. Barrier readers do **not** react to barriers: a barrier placed on the direction a creature is about to move changes its action in 0 of 12,501 trials, and the live counters agree (creatures with a reachable barrier reader are blocked by a barrier 32.6% of the time a barrier neighbor exists, creatures without one 34.0%). What creatures actually do is run 2 to 5 fixed per-tick programs such as `Move W, Eat, Move S` (the most common tick in the logs) and switch among them by energy and food level.

| Reading | 2026-09-07 note (gen ~1,990, old supply rule) | This world (gen ~410, per-unit supply) |
| --- | ---: | ---: |
| Total mesh nodes, median | 124 | 27 |
| Reachable, median | 17 | 10 |
| Executed over the battery, median | 4 | 7 |
| Contributing (executed minus knockout), median | 3 | 2 |
| Route varies with input (battery) | 50.0% | 19.5% |
| Route varies with input, real memory plus perturbations | not read | 40.3% |
| Route variation load-bearing (3 or more contributing nodes and route varies) | 50.0% | 3.3% |
| Behavior-changing births per all births (60 genomes, 200 births each) | 0.010 | 0.157 |
| Executed node reads barriers | 35 of 400 | 127 of 400 |
| Barrier placed on the chosen move direction: action changes | not read | 0 of 12,501 |
| Move lands on the single primary-food neighbor, given a move | not read | 12.5% (chance 12.5%) |
| Move lands on the single fruit neighbor, given a move: fruit at W / any other direction | not read | 62% / 5% (chance 12.5%) |
| One ring slot toggled changes the action: primary food / fruit / occupied / barrier | not read | 6 / 297 / 16 / 1 of 400 |

## 1. The world at tick 28,315

Read from `/status`, `/config`, and a whole-world `/snapshot` at detail zoom.

| Quantity | Value |
| --- | ---: |
| Population; generation p10 / median / p90 / max | 60,074; 389 / 410 / 468 / 606 |
| Energy p25 / median / p75 of 200 | 14.6 / 21.2 / 28.3 |
| Last tick actions: eat / move / reproduce / noop / steal | 64,036 / 94,740 / 1,447 / 647 / 14 |
| Moves blocked over the run: barrier / occupied | 308.1M / 571.3M |
| Move attempts with a barrier neighbor, blocked by that barrier: has reader / no reader | 32.6% (of 310.2M) / 34.0% (of 609.3M) |
| Reproduce attempts with a barrier neighbor, rejected for barrier: has reader / no reader | 48.7% / 53.2% |
| Reproduction attempts rejected | 121.4M of 141.5M (85.8%); occupied 56.5M, barrier 25.4M |
| Mutation targets over the run: executed / reachable / unreachable / n.a. | 29.7M / 30.1M / 1.9M / 3.5M |
| Supply: `per_unit_supply_enabled`, `per_unit_rate` 0.005, genome size median 475 | about 2.4 requested events per birth |
| Costs: move 0.1, noop 0.05, reproduce 1.0, decay 0.02 per tick, failed action penalty 1.0 ramping to tick 62,680 (0.45 now) | |
| Food: Primary (5 per unit) and Fruit (50 per unit); eats in the sampled logs 1,417 Primary / 33,592 Fruit | |

`has_barrier_reader` in the counters is the reachable-node reading (`creature/state.rs::compute_has_barrier_reader`), so the in-vivo split is by presence of the reference, not by use.

## 2. Question 1: routing

### 2.1 Structure and battery function (400 genomes)

| Quantity | min | p25 | median | p75 | max | mean |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| generation | 374 | 398 | 410 | 425 | 585 | 421.0 |
| total mesh nodes | 5 | 23 | 27 | 32 | 88 | 28.0 |
| reachable | 1 | 8 | 10 | 13 | 32 | 10.8 |
| executed over the battery | 1 | 5 | 7 | 9 | 22 | 7.7 |
| contributing (bypass changes the signature) | 0 | 2 | 2 | 2 | 4 | 2.0 |
| knockout (executed, bypass changes nothing) | 0 | 3 | 5 | 8 | 19 | 5.7 |
| executed nodes with evolved ids | 0 | 4 | 6 | 8 | 21 | 6.5 |
| contributing nodes with evolved ids | 0 | 0 | 1 | 1 | 3 | 0.8 |
| `genome_size` | 186 | 403 | 475 | 581 | 1,296 | 506 |
| chain length, median over 1,036 runs per creature | 1 | 5 | 7 | 9 | 19 | 7.2 |
| distinct discretized action queues over those runs | 1 | 5 | 7 | 8 | 14 | 6.7 |

Backends: 6,281 graph nodes (1,660 executed, 378 contributing) and 4,916 VM nodes (1,424 executed, 423 contributing). Entry node is graph in 358 of 400. Hop-cap hits: 0 in the battery and 0 in 414,400 perturbation runs (T11.F15's single-visit rule holds).

The executed chain is long now (median 7 against 2 in the old regime) because T11.F15's detour form makes a new node execute at birth. Three quarters of reachable nodes execute. But bypassing 5 of the 7 changes nothing on the battery; the executed count is a scaffold count.

### 2.2 The two nodes

The founder has two nodes (ids 0 and 1). Contributing sets over the 400: `{1, 4}` in 252, `{0, 1}` in 82, `{1}` in 23, `{1, 2, 14}` in 10, `{0, 1, 14}` in 7, `{4, 34}` in 5, empty in 4. Node 4 is present in 339 genomes, graph-backed in 337, reads food and introspection in most (`{food, introspection}` 165, `{}` 65, `{food, neighbor, introspection}` 37), and carries two or more route targets in 129. One evolved node replaced founder node 0 in most lineages, and 291 of 400 creatures have at least one contributing evolved node; it is nearly always this one. Contributing sets of three or more nodes: 29 creatures.

### 2.3 Route variation is mostly not load-bearing

| Group | n | contributing, mean | distinct action queues, mean | executed, mean | knockout, mean |
| --- | ---: | ---: | ---: | ---: | ---: |
| route varies (real memory plus perturbations) | 161 | 2.02 | 6.45 | 8.73 | 6.71 |
| route does not vary | 239 | 1.99 | 6.81 | 7.02 | 5.03 |

A varying route selects among detours that produce the same actions. Memory alone (zeroing it) moves a route in 56 creatures. The nodes whose route varies are most often node 4 (27), node 0 (25), and node 1 (13).

### 2.4 Mutational neighborhood (60 genomes, 200 births each, live mutation config)

| Reading | Founder | Live x60 |
| --- | ---: | ---: |
| zero-event births | 116 of 200 | 1,290 of 12,000 (10.7%) |
| mutated births: silent / changed / dead | 41 / 43 / 0 of 84 | 8,763 / 1,889 / 58 of 10,710 |
| conditional silent fraction | 0.488 | 0.818 |
| single-event births silent | 35 of 60 | 2,339 of 2,512 (0.931) |
| behavior-changing births per all births | 0.215 | 0.157 |
| dead births per all births | 0.000 | 0.005 |

At about 30 births per tick this is about 4.7 behavior-changing births per tick. The 2026-09-07 population read 0.010 changed per birth; T11.F17 and T11.F19 repaired that. The live `mutation_value_totals_by_operator` counters read helpful and detrimental at roughly 1:1 for every topology operator (for example `AddRouteTarget` 301,564 helpful / 273,210 detrimental over 633,594 carriers), which says the valuation is noise-level per operator, not that any operator is being selected for.

### 2.5 Why routing is not becoming complex

1. **There is nothing for a third node to decide.** Section 3 shows the population's behavior is a small set of fixed programs switched by energy and food level; the two contributing nodes already implement that switch. A route branch pays only if the branch's destination does something the trunk does not, and every destination that could be reached by a one-event change emits a constant-direction action or nothing.
2. **The direction encoding.** `runtime/action_decode.rs::decode_direction` rounds one scalar parameter and clamps it to 0..7; a graph action slot's parameter is `input x weight` and an unwired parameter is 0, which is North. A move toward food needs an index-of-maximum over an 8-slot ring; the graph kinds available (`Add, Multiply, Min, Max, WeightedSum, Sigmoid, Threshold, GreaterThan, Select, ...`) can express it only as about seven comparisons and seven selects with constants 0 to 7, and a partial construction yields a direction no better than the constant it replaces. There is no fitness gradient along the way past the first slot: the population did find a one-slot rule (fruit at W, go W; Section 3.2), and that is as far as the scalar encoding rewards. No routing decision that depends on a general direction is worth making.
3. **Robustness.** 93% of single-event births are silent (founder 58%). That is the expected consequence of 400 generations at 2.4 events per birth (Wilke and Wagner; Wright and Laue, cited in the 2026-09-06 note) and it means most events land on the scaffold or on introns. It is not starvation: the changed fraction per birth is 16 times the old regime's.
4. **The scaffold is executed but inert.** T11.F15 made growth neutral; T11.F17 targets executed nodes; between them a detour node receives mutations and stays a detour, because acquiring a gate-wired action slot with a useful constant is the same needle as item 2.

## 3. Question 2: reactivity

### 3.1 Paired perturbation (400 genomes, 6 bases each: the creature's real neighborhood and memory plus 5 synthetic)

For each base and each input class, one input was changed and the discretized action queue (kind, direction, food type) compared. "Any" counts creatures whose action changed for at least one perturbation in the class; "pooled" is the fraction of perturbations that changed it.

| Input class | creatures reacting (any) | pooled |
| --- | ---: | ---: |
| energy | 335 (83.8%) | 0.448 |
| one fruit ring slot toggled | 297 (74.2%) | 0.098 |
| fruit here | 213 (53.2%) | 0.142 |
| area food summary | 181 (45.2%) | 0.203 |
| food here | 168 (42.0%) | 0.177 |
| nearby creature identity | 160 (40.0%) | 0.281 |
| memory zeroed | 117 (29.2%) | 0.134 |
| memory scrambled | 100 (25.0%) | 0.107 |
| nearby creature vitals | 96 (24.0%) | 0.175 |
| nearby creature core | 92 (23.0%) | 0.167 |
| age | 90 (22.5%) | 0.033 |
| area occupancy summary | 27 (6.8%) | 0.036 |
| one occupied ring slot toggled | 16 (4.0%) | 0.002 |
| one primary food ring slot toggled | 6 (1.5%) | 0.001 |
| area barrier summary | 2 (0.5%) | 0.001 |
| **one barrier ring slot toggled** | **1 (0.2%)** | **0.000** |

Two scenario classes that reset the whole neighborhood (no food here, one food cell at d, no barriers or occupants) also read high, 72.5% for primary food and 84.2% for fruit, but they change several inputs at once and are confounded with `food_here`; the single-slot rows above are the clean reading. The fruit ring's consumption is two slots wide: toggling slot N changes the action in 232 creatures and slot W in 258, the other six slots in 0 to 50, and every one of the 1,850 changes is a change of direction, never of action kind.

### 3.2 Directional readings (pooled over 400 creatures)

| Test | Result | Chance |
| --- | ---: | ---: |
| food only at direction d, no food here: creature moves | 12,501 of 19,200 (65%) | |
| ... given a move, moves to d | 1,565 of 12,501 (12.5%) | 12.5% |
| ... given a move, moves within 45 degrees of d | 4,691 of 12,501 (37.5%) | 37.5% |
| fruit only at d, given a move: to d / toward d | 13.0% / 29.2% | 12.5% / 37.5% |
| random graded ring, given a move: to the maximum / toward it | 12.6% / 37.5% | 12.5% / 37.5% |
| area food gradient toward d, no local food, given a move: to d / toward d | 12.5% / 37.5% | 12.5% / 37.5% |
| barrier placed on the direction the creature chose: no longer moves there | 0 of 12,501 | |
| ... any change to the action queue at all | 24 of 12,501 | |
| occupied cell placed on the chosen direction: no longer moves there | 200 of 12,501 (1.6%) | |
| reproduce direction occupied: no longer reproduces there | 0 of 532 | |
| `food_here` = 1: leads with Eat | 1,049 of 4,800 (21.9%) | |
| `food_here` = 0: leads with Eat anyway | 301 of 4,800 (6.3%) | |

Per creature: 0 of 400 land on the primary-food cell more than half the time; 0 avoid a barrier more than half the time; 330 never avoid one (70 never move in this test).

The fruit test hides one rule inside its chance-level total. Pooled over 400 creatures and 6 bases, fruit only at row d gives the lead move direction in the columns:

| fruit at | N | NE | E | SE | S | SW | W | NW |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| N | 1,418 | 0 | 0 | 31 | 2 | 26 | 85 | 0 |
| E | 1,382 | 0 | 37 | 31 | 2 | 26 | 85 | 0 |
| S | 1,419 | 0 | 0 | 31 | 2 | 26 | 85 | 0 |
| **W** | **482** | 0 | 20 | 23 | 32 | 26 | **968** | 0 |
| any other | about 1,414 | 0 | 0 to 5 | 12 to 37 | 2 to 6 | 26 | 85 to 98 | 0 |

Fruit at W sends 62% of moves West; fruit anywhere else sends about 90% North (the unwired default) and 5% West. 215 of 400 creatures go West on fruit-at-W in more than half their moves, 205 in more than 90%; 191 of those 215 have the contributing set `{1, 4}`. In a traced carrier (creature 1473173952724), evolved graph node 4 has the sink `CustomOutput 5 <- InputLeaf(fruit ring, sub 6), weight 1.0`, copying `fruit[W]` onto upstream slot 5, and founder VM node 1 reads upstream slot 5 and folds it into the `WriteWorldActionMeta` that sets the Move direction. One sensor slot, one weight, one direction: the scalar encoding permits exactly this, once per parameter, and the population found it once.

Split by barrier-reader state: contributing node reads barriers, 93 creatures, avoidance 0 of 3,144; executed-only reader, 34, 0 of 920; reachable-only, 4, 0 of 136; no reader, 269, 0 of 8,301. In the two contributing graph nodes traced, the barrier `InputRef`s (`AreaBarrierSummary`) sit in the node's reference list and no compute edge, output sink, or action slot reads them (execute gates were not printed); the functional reading above carries the point on its own. `InputRef.Add` is the most-applied operator in the run (1.58M events), so a reference is cheap and free to hold. What is open is why the consumer is never found: for a graph slot, suppressing `Move(d)` on `barrier[d]` is one `AddGraphEdge` onto the slot's gate with a negative weight, and in the VM node 1 that emits most moves it is two or three instructions; blocked moves are about 10% of moves at a penalty of 0.45 now and 1.0 by tick 62,680, against 0.1 per move and 0.02 per tick decay. This survey shows the suppression is absent, not why.

### 3.3 In vivo

- On each creature's real neighborhood and memory (fresh graph state), 203 of 400 lead with a move; 52 of those have a barrier neighbor and 12 move into it; 29 of 203 move into an occupied cell. With some neighbor food present, 26 of 77 moves go toward the best cell (33.8%, chance 37.5%); the rank of the chosen cell's food among the 8 neighbors is uniform. Of 43 creatures standing on food (> 0.05), 23 lead with Eat.
- Action logs (last ~300 actions each, 97,018 entries): Move success 52.3%, Eat success 36.1%, Move blocked 6.1% (10.4% of moves), Eat with no food 4.1%, Reproduce invalid target 0.4%, success 0.3%.
- Per-tick queues in the logs: 2 or 3 actions per tick in 89% of ticks. Most common: `Move W, Eat, Move S` (6,591 ticks), `Eat, Move W` (4,880), `Move N, Eat, Move NE` (4,065), `Move S, Eat, Move NE` (3,033). Each creature used 4 or 5 distinct per-tick queues (median), and 3 to 5 distinct move directions, over its log.
- Under constant blank inputs for 24 ticks with persistent graph state and memory, 326 of 400 creatures move in exactly one direction throughout; constant primary food at SE changes the sequence for 0 of 400; a barrier on every direction used changes it for 0 of 400 (7,836 of 7,836 moves go into the barrier). Sweeping energy alone switches the action kind (move, eat, reproduce) in 296 of 400 but the move direction in 19. So the in-vivo direction variety is the fixed direction constants of 2 to 3 action slots in one tick, and the switching among programs is by energy and food level.

### 3.4 What this says about the environment loop

Creatures are grazers running a fixed diagonal harvest (`move, eat, move` reaps two cells per tick) in a wrap world with 54% food coverage, eating Fruit almost exclusively (96% of eats), reproducing when energy passes the threshold, and paying the blocked-move penalty (0.45 now, 1.0 at tick 62,680) on about one move in ten. Nothing in that loop needs a direction computed from a sensor beyond the one-slot fruit rule, and the encoding makes a general one expensive: a second seeking direction needs a second weight on a second slot and a way to combine the two into one scalar, which is where the argmax construction of Section 2.5 begins. The sensors that would support general seeking and avoidance are referenced (one `InputRef.Add`, free to hold) and, apart from `fruit[W]` and `fruit[N]`, not consumed.

## 4. Caveats

- In-vitro runs start from fresh `GraphRuntimeState`; Hebbian weights and operator state are not in the dump. The 24-tick sequence probe lets state accumulate from fresh and still finds no direction dynamics, but a learned weight set that exists only in the live world would be missed.
- Extended perception cannot be reconstructed from the dump. Base perception was zero; the class perturbations show whether it can matter (area food 45%, nearby identity 40%), not what the creature sees now.
- The battery's `route_varies_with_input` starts from zeroed memory; the 40.3% figure uses real memory and includes energy and memory perturbations as input.
- "Chance" for the directional tests assumes a uniform direction; the population's direction constants are not uniform (W 25%, S 22%, N 18%, NE 17% in the logs), which does not change the conclusion because the exact-hit rate is compared with the probability that a fixed direction happens to be d.
- One world, one sample of 400, one tick. The 2026-09-07 population was under a different supply rule and 5 times deeper; the comparison column is context, not a controlled contrast.

## 5. Where this points (not implemented)

Item 1 became T11.F21 on 2026-09-16 after the [motor output encoding research note](motor-output-encoding-research-2026-09-16.md) compared the alternatives.

1. **Direction representation.** Give a Move or Reproduce slot a per-direction form: eight gate inputs, one per ring slot, winner takes the direction (as a Braitenberg or NEAT-style output bank), so one edge from `NeighborFoodRing[d]` or a negative edge from `NeighborBarrierRing[d]` is a complete, selectable step for every d, not only the one slot the scalar form rewarded. This is the change that would let the sensors already present be consumed and would give a route branch something to select.
2. **A reading that counts consumed sensors, not referenced ones.** The live `has_barrier_reader` split and the `sensor_census` read presence; T11.F14's knockout is the model for a per-input knockout (zero one input class, compare the battery signature), which this probe approximated and which would show the barrier readers as inert in every closure report.
3. **A seeking task in the goal profile.** The battery zeroes barriers and draws food uniformly; a scenario family with one food cell and one barrier per direction would make the exact-hit and avoidance fractions closure readings.

## Appendix A: reproducing the readings

```sh
curl -s http://localhost:3000/v3/simulation/status > status.json
curl -s http://localhost:3000/v3/simulation/config > config.json
curl -s "http://localhost:3000/v3/simulation/snapshot?zoom_tier=detail&x=0&y=0&width=1600&height=1600" > detail.json
# ids in detail.json view.creatures; python: random.Random(11).sample(sorted(ids), 400); then per id
curl -s "http://localhost:3000/v3/simulation/creature/<id>" -o genomes/<id>.json
```

The probe was `crates/v3-core/tests/zz_probe_live_survey.rs` (temporary; removed after the run; source in Appendix B). Each test prints one JSON line per creature; aggregate with any JSON reader.

```sh
PETRI_GENOME_DIR=genomes PETRI_CONFIG=config.json \
  cargo test --release -p v3-core --test zz_probe_live_survey probe_live_survey -- --nocapture      # 3 s
  ... probe_live_sequences        # Section 3.3 constant-input sequences
  ... probe_live_heading_drivers  # Section 3.3 sweeps
PETRI_BIRTHS_N=60 ... probe_live_births   # Section 2.4, 20 s
```

All readings are deterministic (`SmallRng`, per-creature seed `1000 + id`; battery seeds 7 and 8; birth seeds `1_000_000 * (i + 1)`).

## Appendix B: the probe

Dropped into `crates/v3-core/tests/zz_probe_live_survey.rs` and removed afterward, following the 2026-09-07 note's pattern. It depends only on public `v3_core` APIs.

```rust
//! TEMPORARY PROBE (not for commit): live-world survey of mesh routing and
//! sensor reactivity on genomes dumped from a running server
//! (`GET /v3/simulation/creature/:id`). One JSON line per creature on stdout.
//!
//! Readings per creature:
//! - structure and the T11.F14 battery (executed / contributing / route
//!   variation / hop cap / knockout), split founder-era vs evolved node ids;
//! - paired-perturbation reactivity on the creature's real current inputs and
//!   real shared memory, plus synthetic bases: does the discretized action
//!   change when one input class changes;
//! - directional scores: local food seeking (both food types), area-food
//!   seeking, barrier / occupied avoidance, reproduce-into-occupied;
//! - the action it would take right now on its real inputs, and its action log.
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::collections::{BTreeMap, BTreeSet};
use v3_core::config::{FounderProfile, RuntimeConfig, SimulationConfig};
use v3_core::contracts::{Direction, NodeId, WorldAction};
use v3_core::creature::founder::founder_genome;
use v3_core::creature::genome::analysis::mesh_reachable_nodes;
use v3_core::creature::genome::mesh_annotations::{
    derive_mesh_annotations_with_reachable_indices, MeshReadClass,
};
use v3_core::creature::genome::{BackendDef, CreatureGenome};
use v3_core::creature::state::GraphRuntimeState;
use v3_core::neighborhood::Battery;
use v3_core::runtime::trace::domain::TerminationReason;
use v3_core::runtime::traced_mesh::execute_creature_mesh_traced;
use v3_core::sensors::perception::{
    genome_uses_extended_perception, PerceptionSnapshot, SensorSnapshot,
};
use v3_core::sensors::static_inputs::StaticInputs;
use v3_core::sensors::typed_food::{genome_uses_typed_local_food, TypedFoodLocalSnapshot};

#[derive(Clone)]
struct Base {
    sensors: SensorSnapshot,
    energy: f32,
    age: u64,
    memory: [f32; 16],
}

/// Discretized action: kind + direction + food type, ignoring continuous params.
fn disc(actions: &[WorldAction]) -> Vec<String> {
    actions
        .iter()
        .map(|a| match a {
            WorldAction::NoOp => "NoOp".to_string(),
            WorldAction::Eat { type_idx } => format!("Eat{}", type_idx.get()),
            WorldAction::Move(d) => format!("Move{}", d.to_index()),
            WorldAction::Reproduce { direction, .. } => format!("Repro{}", direction.to_index()),
            WorldAction::StealEnergy { direction, .. } => format!("Steal{}", direction.to_index()),
        })
        .collect()
}

struct Run {
    actions: Vec<WorldAction>,
    hops: Vec<NodeId>,
    routes: Vec<(NodeId, usize)>,
    hop_cap: bool,
}

fn run(genome: &CreatureGenome, base: &Base, runtime: &RuntimeConfig) -> Run {
    let mut energy = base.energy;
    let mut shared = base.memory;
    let prev = base.memory;
    let mut gr = GraphRuntimeState::new();
    gr.begin_tick(&genome.nodes, base.age);
    let (out, hops, term) = execute_creature_mesh_traced(
        genome,
        &base.sensors,
        &mut energy,
        &mut shared,
        &prev,
        &mut gr,
        runtime,
    );
    Run {
        actions: out.actions,
        hops: hops.iter().map(|h| h.node_id).collect(),
        routes: hops
            .iter()
            .filter_map(|h| h.route.as_ref().map(|r| (h.node_id, r.selected_target_idx)))
            .collect(),
        hop_cap: matches!(term, TerminationReason::MaxHopsReached),
    }
}

/// First action's move direction, if the queue leads with a Move.
fn lead_move(actions: &[WorldAction]) -> Option<usize> {
    match actions.first() {
        Some(WorldAction::Move(d)) => Some(d.to_index()),
        _ => None,
    }
}
fn lead_repro(actions: &[WorldAction]) -> Option<usize> {
    match actions.first() {
        Some(WorldAction::Reproduce { direction, .. }) => Some(direction.to_index()),
        _ => None,
    }
}
fn lead_eat(actions: &[WorldAction]) -> Option<u16> {
    match actions.first() {
        Some(WorldAction::Eat { type_idx }) => Some(type_idx.get()),
        _ => None,
    }
}

fn unit(d: usize) -> (f32, f32) {
    let (dx, dy) = Direction::ALL[d].delta();
    let n = ((dx * dx + dy * dy) as f32).sqrt();
    (dx as f32 / n, dy as f32 / n)
}

fn toward(chosen: usize, target: usize) -> bool {
    let (ax, ay) = unit(chosen);
    let (bx, by) = unit(target);
    ax * bx + ay * by > 0.01
}

fn set_neighbor_food(s: &mut SensorSnapshot, type_idx: usize, ring: [f32; 8]) {
    if type_idx == 0 {
        s.local.neighbor_food = ring;
    }
    if let Some(r) = s.typed_local_food.neighbor_food_by_type.get_mut(type_idx) {
        *r = ring;
    }
}
fn set_food_here(s: &mut SensorSnapshot, type_idx: usize, v: f32) {
    if type_idx == 0 {
        s.local.food_here = v;
    }
    if let Some(r) = s.typed_local_food.food_here_by_type.get_mut(type_idx) {
        *r = v;
    }
}

fn synthetic_base(rng: &mut SmallRng, food_types: usize, memory: [f32; 16], age: u64) -> Base {
    let nz = |rng: &mut SmallRng, p0: f64| if rng.gen_bool(p0) { 0.0 } else { rng.gen_range(0.05f32..=1.0) };
    let food_here_by_type: Vec<f32> = (0..food_types).map(|t| nz(rng, if t == 0 { 0.5 } else { 0.7 })).collect();
    let neighbor_food_by_type: Vec<[f32; 8]> =
        (0..food_types).map(|t| std::array::from_fn(|_| nz(rng, if t == 0 { 0.5 } else { 0.8 }))).collect();
    let neighbor_barrier: [f32; 8] = std::array::from_fn(|_| if rng.gen_bool(0.15) { 1.0 } else { 0.0 });
    let neighbor_occupied: [f32; 8] = std::array::from_fn(|_| if rng.gen_bool(0.3) { 1.0 } else { 0.0 });
    let energy = [8.0, 15.0, 22.0, 30.0, 45.0, 80.0][rng.gen_range(0..6)];
    Base {
        sensors: SensorSnapshot {
            local: StaticInputs {
                food_here: food_here_by_type[0],
                neighbor_food: neighbor_food_by_type[0],
                neighbor_barrier,
                neighbor_occupied,
                generation: 400.0,
                age_ticks: age as f32,
            },
            typed_local_food: TypedFoodLocalSnapshot { food_here_by_type, neighbor_food_by_type },
            perception: PerceptionSnapshot::zeroed(food_types),
        },
        energy,
        age,
        memory,
    }
}

fn f32s(v: &serde_json::Value) -> [f32; 8] {
    let mut out = [0.0; 8];
    for (i, x) in v.as_array().into_iter().flatten().take(8).enumerate() {
        out[i] = x.as_f64().unwrap_or(0.0) as f32;
    }
    out
}

#[test]
fn probe_live_survey() {
    let dir = std::env::var("PETRI_GENOME_DIR").expect("PETRI_GENOME_DIR");
    let config_path = std::env::var("PETRI_CONFIG").expect("PETRI_CONFIG (live /config dump)");
    let cfg_json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
    let config: SimulationConfig = serde_json::from_value(cfg_json["config"].clone()).expect("config deserializes");
    let runtime = config.runtime.clone();
    let food_types = config.world.food.types.len();
    let decay = config.shared_memory.decay_rate;
    let battery = Battery::generate(food_types);
    let founder = founder_genome(FounderProfile::V3Alpha1);
    let founder_ids: BTreeSet<NodeId> = founder.nodes.iter().map(|n| n.node_id).collect();
    let founder_max_id = founder.nodes.iter().map(|n| n.node_id.0).max().unwrap_or(0);
    eprintln!("founder nodes={} max_id={founder_max_id} food_types={food_types}", founder.nodes.len());

    let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json")).collect();
    files.sort();
    let started = std::time::Instant::now();
    for (i, path) in files.iter().enumerate() {
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let id = v["id"].as_u64().unwrap_or(0);
        let gen = v["generation"].as_u64().unwrap_or(0);
        let age = v["age"].as_u64().unwrap_or(0);
        let energy = v["energy"].as_f64().unwrap_or(0.0) as f32;
        let genome: CreatureGenome = serde_json::from_value(v["genome"].clone()).expect("genome deserializes");
        let mut memory = [0.0f32; 16];
        for (k, x) in v["shared_memory"].as_array().into_iter().flatten().take(16).enumerate() {
            memory[k] = x.as_f64().unwrap_or(0.0) as f32;
        }
        let ci = &v["diagnostics"]["current_inputs"];

        // ---- structure + battery ----
        let reachable = mesh_reachable_nodes(&genome);
        let ann = derive_mesh_annotations_with_reachable_indices(&genome, &reachable);
        let sets = battery.mesh_execution_sets(&genome, &runtime, decay);
        let m = sets.reading;
        let node_by_id = |nid: NodeId| genome.nodes.iter().find(|n| n.node_id == nid);
        let exec_evolved = sets.executed.iter().filter(|n| !founder_ids.contains(n)).count();
        let contrib_evolved = sets.contributing.iter().filter(|n| !founder_ids.contains(n)).count();
        let exec_multi_target = sets.executed.iter().filter(|n| node_by_id(**n).is_some_and(|x| x.targets.len() >= 2)).count();
        let reads = |set: &BTreeSet<NodeId>, class: MeshReadClass| -> usize {
            ann.iter().filter(|a| set.contains(&a.node_id) && a.read_classes.contains(&class)).count()
        };
        let reachable_ids: BTreeSet<NodeId> = ann.iter().filter(|a| a.reachable).map(|a| a.node_id).collect();
        let read_classes = |set: &BTreeSet<NodeId>| -> BTreeMap<String, usize> {
            let mut out = BTreeMap::new();
            for c in [MeshReadClass::Food, MeshReadClass::Neighbor, MeshReadClass::Barrier, MeshReadClass::Occupancy, MeshReadClass::Introspection, MeshReadClass::Upstream, MeshReadClass::ActionQueue] {
                out.insert(format!("{c:?}"), reads(set, c));
            }
            out
        };
        let contributing_ids: Vec<u32> = sets.contributing.iter().map(|n| n.0).collect();
        let executed_ids: Vec<u32> = sets.executed.iter().map(|n| n.0).collect();
        let entry_backend = node_by_id(genome.entry_node_id).map(|n| match n.backend_def { BackendDef::Graph(_) => "graph", BackendDef::Vm(_) => "vm" }).unwrap_or("none");

        // ---- bases ----
        let mut real = Base {
            sensors: SensorSnapshot {
                local: StaticInputs {
                    food_here: ci["food_here"].as_f64().unwrap_or(0.0) as f32,
                    neighbor_food: f32s(&ci["neighbor_food"]),
                    neighbor_barrier: f32s(&ci["neighbor_barrier"]),
                    neighbor_occupied: f32s(&ci["neighbor_occupied"]),
                    generation: gen as f32,
                    age_ticks: age as f32,
                },
                typed_local_food: TypedFoodLocalSnapshot {
                    food_here_by_type: vec![0.0; food_types],
                    neighbor_food_by_type: vec![[0.0; 8]; food_types],
                },
                perception: PerceptionSnapshot::zeroed(food_types),
            },
            energy,
            age,
            memory,
        };
        real.sensors.typed_local_food.food_here_by_type[0] = real.sensors.local.food_here;
        real.sensors.typed_local_food.neighbor_food_by_type[0] = real.sensors.local.neighbor_food;
        let mut rng = SmallRng::seed_from_u64(1000 + id);
        let mut bases = vec![real.clone()];
        for _ in 0..5 { bases.push(synthetic_base(&mut rng, food_types, memory, age)); }

        // ---- real-input prediction ----
        let r0 = run(&genome, &real, &runtime);
        let real_disc = disc(&r0.actions);
        let real_food_here = real.sensors.local.food_here;
        let nf = real.sensors.local.neighbor_food;
        let best_dir = (0..8).max_by(|a, b| nf[*a].partial_cmp(&nf[*b]).unwrap()).unwrap();
        let real_lead_move = lead_move(&r0.actions);
        let real_move_into_barrier = real_lead_move.is_some_and(|d| real.sensors.local.neighbor_barrier[d] > 0.5);
        let real_move_into_occupied = real_lead_move.is_some_and(|d| real.sensors.local.neighbor_occupied[d] > 0.5);
        let real_move_toward_best = real_lead_move.is_some_and(|d| nf[best_dir] > 0.05 && toward(d, best_dir));
        let real_move_food_rank = real_lead_move.map(|d| (0..8).filter(|o| nf[*o] > nf[d]).count());
        let real_repro_into_blocked = lead_repro(&r0.actions).is_some_and(|d| real.sensors.local.neighbor_barrier[d] > 0.5 || real.sensors.local.neighbor_occupied[d] > 0.5);
        let has_barrier_neighbor = real.sensors.local.neighbor_barrier.iter().any(|b| *b > 0.5);

        // ---- perturbation census ----
        let mut all_runs: Vec<Run> = vec![];
        let mut sens: BTreeMap<&str, (u32, u32)> = BTreeMap::new(); // class -> (differing, total)
        let note = |class: &'static str, base_d: &Vec<String>, r: &Run, sens: &mut BTreeMap<&str, (u32, u32)>| {
            let e = sens.entry(class).or_insert((0, 0));
            e.1 += 1;
            if disc(&r.actions) != *base_d { e.0 += 1; }
        };
        // directional tallies
        let (mut food_seek_n, mut food_seek_exact, mut food_seek_toward) = (0u32, 0u32, 0u32);
        let (mut fruit_seek_n, mut fruit_seek_exact, mut fruit_seek_toward) = (0u32, 0u32, 0u32);
        let (mut food_seek_eat_instead, mut food_seek_move_any) = (0u32, 0u32);
        let (mut argmax_n, mut argmax_exact, mut argmax_toward) = (0u32, 0u32, 0u32);
        let (mut area_n, mut area_exact, mut area_toward) = (0u32, 0u32, 0u32);
        let (mut bar_n, mut bar_avoid, mut bar_changed) = (0u32, 0u32, 0u32);
        let (mut occ_n, mut occ_avoid, mut occ_changed) = (0u32, 0u32, 0u32);
        let (mut rocc_n, mut rocc_avoid) = (0u32, 0u32);
        let (mut eat_n, mut eat_when_food, mut eat_when_none) = (0u32, 0u32, 0u32);
        let mut memory_gate_routes = false;
        let mut fruit_matrix = [[0u32; 8]; 8];
        let mut fruit_shift = [[0u32; 8]; 8];

        for b in &bases {
            let rb = run(&genome, b, &runtime);
            let bd = disc(&rb.actions);
            // food_here on/off (type 0)
            for v in [0.0f32, 1.0] {
                let mut p = b.clone(); set_food_here(&mut p.sensors, 0, v);
                let r = run(&genome, &p, &runtime);
                note("food_here", &bd, &r, &mut sens);
                eat_n += 1;
                let ate = lead_eat(&r.actions).is_some();
                if v > 0.5 && ate { eat_when_food += 1; }
                if v < 0.5 && ate { eat_when_none += 1; }
                all_runs.push(r);
            }
            // fruit here
            for v in [0.0f32, 1.0] {
                let mut p = b.clone(); set_food_here(&mut p.sensors, 1, v);
                let r = run(&genome, &p, &runtime); note("fruit_here", &bd, &r, &mut sens); all_runs.push(r);
            }
            // local food seeking, type 0: food only at d, none here
            for d in 0..8 {
                let mut p = b.clone();
                set_food_here(&mut p.sensors, 0, 0.0); set_food_here(&mut p.sensors, 1, 0.0);
                let mut ring = [0.0; 8]; ring[d] = 1.0;
                set_neighbor_food(&mut p.sensors, 0, ring); set_neighbor_food(&mut p.sensors, 1, [0.0; 8]);
                p.sensors.local.neighbor_barrier = [0.0; 8]; p.sensors.local.neighbor_occupied = [0.0; 8];
                let r = run(&genome, &p, &runtime);
                note("food_at_d_scenario", &bd, &r, &mut sens);
                food_seek_n += 1;
                if let Some(c) = lead_move(&r.actions) { food_seek_move_any += 1; if c == d { food_seek_exact += 1; } if toward(c, d) { food_seek_toward += 1; } }
                if lead_eat(&r.actions).is_some() { food_seek_eat_instead += 1; }
                // barrier at the chosen direction
                if let Some(c) = lead_move(&r.actions) {
                    let mut q = p.clone(); q.sensors.local.neighbor_barrier[c] = 1.0;
                    let r2 = run(&genome, &q, &runtime);
                    bar_n += 1; if lead_move(&r2.actions) != Some(c) { bar_avoid += 1; } if disc(&r2.actions) != disc(&r.actions) { bar_changed += 1; }
                    let mut q = p.clone(); q.sensors.local.neighbor_occupied[c] = 1.0;
                    let r3 = run(&genome, &q, &runtime);
                    occ_n += 1; if lead_move(&r3.actions) != Some(c) { occ_avoid += 1; } if disc(&r3.actions) != disc(&r.actions) { occ_changed += 1; }
                    all_runs.push(r2); all_runs.push(r3);
                }
                all_runs.push(r);
                // fruit only at d
                let mut p = b.clone();
                set_food_here(&mut p.sensors, 0, 0.0); set_food_here(&mut p.sensors, 1, 0.0);
                set_neighbor_food(&mut p.sensors, 0, [0.0; 8]); set_neighbor_food(&mut p.sensors, 1, ring);
                p.sensors.local.neighbor_barrier = [0.0; 8]; p.sensors.local.neighbor_occupied = [0.0; 8];
                let r = run(&genome, &p, &runtime);
                note("fruit_at_d_scenario", &bd, &r, &mut sens);
                fruit_seek_n += 1;
                if let Some(c) = lead_move(&r.actions) { if c == d { fruit_seek_exact += 1; } if toward(c, d) { fruit_seek_toward += 1; } fruit_matrix[d][c] += 1; }
                if let Some(c) = lead_move(&rb.actions) { if let Some(c2) = lead_move(&r.actions) { if c2 != c { fruit_shift[d][c2] += 1; } } }
                all_runs.push(r);
            }
            // graded food: random ring, does the move go to the argmax?
            for _ in 0..4 {
                let mut p = b.clone();
                set_food_here(&mut p.sensors, 0, 0.0);
                let ring: [f32; 8] = std::array::from_fn(|_| rng.gen_range(0.0f32..=1.0));
                set_neighbor_food(&mut p.sensors, 0, ring);
                p.sensors.local.neighbor_barrier = [0.0; 8]; p.sensors.local.neighbor_occupied = [0.0; 8];
                let best = (0..8).max_by(|a, b| ring[*a].partial_cmp(&ring[*b]).unwrap()).unwrap();
                let r = run(&genome, &p, &runtime);
                if let Some(c) = lead_move(&r.actions) { argmax_n += 1; if c == best { argmax_exact += 1; } if toward(c, best) { argmax_toward += 1; } }
                all_runs.push(r);
            }
            // area food summary pointing at d (no local food)
            for d in 0..8 {
                let mut p = b.clone();
                set_food_here(&mut p.sensors, 0, 0.0); set_neighbor_food(&mut p.sensors, 0, [0.0; 8]);
                p.sensors.local.neighbor_barrier = [0.0; 8]; p.sensors.local.neighbor_occupied = [0.0; 8];
                let (ux, uy) = unit(d);
                let summary = [0.4, ux, uy, ux * 0.6, uy * 0.6, 0.6, 1.0];
                p.sensors.perception.area_food = summary;
                if let Some(t) = p.sensors.perception.typed_area_food.get_mut(0) { *t = summary; }
                let r = run(&genome, &p, &runtime);
                note("area_food", &bd, &r, &mut sens);
                if let Some(c) = lead_move(&r.actions) { area_n += 1; if c == d { area_exact += 1; } if toward(c, d) { area_toward += 1; } }
                all_runs.push(r);
            }
            // single-slot ring toggles as a class (each direction toggled, nothing else touched)
            for d in 0..8 {
                let mut p = b.clone(); p.sensors.local.neighbor_barrier[d] = 1.0 - p.sensors.local.neighbor_barrier[d];
                let r = run(&genome, &p, &runtime); note("neighbor_barrier", &bd, &r, &mut sens); all_runs.push(r);
                let mut p = b.clone(); p.sensors.local.neighbor_occupied[d] = 1.0 - p.sensors.local.neighbor_occupied[d];
                let r = run(&genome, &p, &runtime); note("neighbor_occupied", &bd, &r, &mut sens);
                all_runs.push(r);
                let mut p = b.clone();
                let mut ring = p.sensors.typed_local_food.neighbor_food_by_type[0]; ring[d] = if ring[d] > 0.5 { 0.0 } else { 1.0 };
                set_neighbor_food(&mut p.sensors, 0, ring);
                let r = run(&genome, &p, &runtime); note("neighbor_food_slot", &bd, &r, &mut sens); all_runs.push(r);
                let mut p = b.clone();
                let mut ring = p.sensors.typed_local_food.neighbor_food_by_type[1]; ring[d] = if ring[d] > 0.5 { 0.0 } else { 1.0 };
                set_neighbor_food(&mut p.sensors, 1, ring);
                let r = run(&genome, &p, &runtime); note("neighbor_fruit_slot", &bd, &r, &mut sens);
                note(["fruit_slot_0","fruit_slot_1","fruit_slot_2","fruit_slot_3","fruit_slot_4","fruit_slot_5","fruit_slot_6","fruit_slot_7"][d], &bd, &r, &mut sens);
                { let rd = disc(&r.actions); if rd != bd { let kind_changed = rd.iter().map(|a| a.trim_end_matches(|c: char| c.is_ascii_digit()).to_string()).collect::<Vec<_>>() != bd.iter().map(|a| a.trim_end_matches(|c: char| c.is_ascii_digit()).to_string()).collect::<Vec<_>>(); note(if kind_changed { "fruit_slot_kind_changed" } else { "fruit_slot_dir_only" }, &bd, &r, &mut sens); } }
                all_runs.push(r);
            }
            // reproduce into occupied: from the base, if it reproduces at d0, occupy d0
            if let Some(d0) = lead_repro(&rb.actions) {
                let mut p = b.clone(); p.sensors.local.neighbor_occupied[d0] = 1.0;
                let r = run(&genome, &p, &runtime);
                rocc_n += 1; if lead_repro(&r.actions) != Some(d0) { rocc_avoid += 1; }
                all_runs.push(r);
            }
            // area barrier / occupancy summaries
            for d in 0..8 {
                let (ux, uy) = unit(d);
                let mut p = b.clone(); p.sensors.perception.area_barrier = [0.3, 0.25, ux, uy, ux * 0.4, uy * 0.4, 0.4];
                let r = run(&genome, &p, &runtime); note("area_barrier", &bd, &r, &mut sens); all_runs.push(r);
                let mut p = b.clone(); p.sensors.perception.area_occupancy = [0.3, ux, uy, ux * 0.4, uy * 0.4, 0.4, 0.5];
                let r = run(&genome, &p, &runtime); note("area_occupancy", &bd, &r, &mut sens); all_runs.push(r);
                // nearby creature slot 0 at d
                let mut p = b.clone();
                p.sensors.perception.nearby_core[0] = 1.0; p.sensors.perception.nearby_core[1] = ux * 0.3; p.sensors.perception.nearby_core[2] = uy * 0.3; p.sensors.perception.nearby_core[3] = 0.3;
                let r = run(&genome, &p, &runtime); note("nearby_core", &bd, &r, &mut sens); all_runs.push(r);
            }
            for v in [0.1f32, 0.9] {
                let mut p = b.clone(); p.sensors.perception.nearby_core[0] = 1.0; p.sensors.perception.nearby_core[3] = 0.3; p.sensors.perception.nearby_vitals[0] = v; p.sensors.perception.nearby_vitals[1] = 1.0 - v;
                let r = run(&genome, &p, &runtime); note("nearby_vitals", &bd, &r, &mut sens); all_runs.push(r);
                let mut p = b.clone(); p.sensors.perception.nearby_core[0] = 1.0; p.sensors.perception.nearby_core[3] = 0.3; p.sensors.perception.nearby_identity[0] = v; p.sensors.perception.nearby_identity[1] = v; p.sensors.perception.nearby_identity[2] = v;
                let r = run(&genome, &p, &runtime); note("nearby_identity", &bd, &r, &mut sens); all_runs.push(r);
            }
            // energy, age, memory
            for e in [5.0f32, 15.0, 45.0, 120.0] {
                let mut p = b.clone(); p.energy = e;
                let r = run(&genome, &p, &runtime); note("energy", &bd, &r, &mut sens); all_runs.push(r);
            }
            for a in [3u64, 25, 300] {
                let mut p = b.clone(); p.age = a; p.sensors.local.age_ticks = a as f32;
                let r = run(&genome, &p, &runtime); note("age", &bd, &r, &mut sens); all_runs.push(r);
            }
            {
                let mut p = b.clone(); p.memory = [0.0; 16];
                let r = run(&genome, &p, &runtime); note("memory_zero", &bd, &r, &mut sens);
                let base_routes: BTreeSet<_> = rb.routes.iter().collect();
                if r.routes.iter().any(|x| !base_routes.contains(x)) { memory_gate_routes = true; }
                all_runs.push(r);
                let mut p = b.clone(); p.memory.rotate_left(1);
                let r = run(&genome, &p, &runtime); note("memory_scrambled", &bd, &r, &mut sens); all_runs.push(r);
            }
            all_runs.push(rb);
        }

        // route variation over the perturbation set (real memory)
        let mut choices: BTreeMap<NodeId, BTreeSet<usize>> = BTreeMap::new();
        let mut exec_all: BTreeSet<NodeId> = BTreeSet::new();
        let mut chain: Vec<usize> = vec![];
        let mut hop_caps = 0usize;
        let mut distinct_actions: BTreeSet<Vec<String>> = BTreeSet::new();
        for r in &all_runs {
            for (n, t) in &r.routes { choices.entry(*n).or_default().insert(*t); }
            exec_all.extend(r.hops.iter().copied());
            chain.push(r.hops.len());
            if r.hop_cap { hop_caps += 1; }
            distinct_actions.insert(disc(&r.actions));
        }
        chain.sort_unstable();
        let route_varies_perturb = choices.values().any(|s| s.len() >= 2);
        let routing_nodes: Vec<u32> = choices.iter().filter(|(_, s)| s.len() >= 2).map(|(n, _)| n.0).collect();

        // action log summary
        let mut log: BTreeMap<String, u32> = BTreeMap::new();
        let mut last_tick_actions: Vec<String> = vec![];
        let mut max_tick = 0u64;
        if let Some(entries) = v["action_log"].as_array() {
            for e in entries {
                let k = format!("{}:{}", e["action_type"].as_str().unwrap_or("?"), e["result"].as_str().unwrap_or("?"));
                *log.entry(k).or_insert(0) += 1;
                max_tick = max_tick.max(e["tick"].as_u64().unwrap_or(0));
            }
            for e in entries {
                if e["tick"].as_u64() == Some(max_tick) {
                    last_tick_actions.push(format!("{}{}:{}", e["action_type"].as_str().unwrap_or("?"), e["direction"].as_u64().map(|d| if d < 8 { d.to_string() } else { String::new() }).unwrap_or_default(), e["result"].as_str().unwrap_or("?")));
                }
            }
        }

        let sens_json: BTreeMap<&str, serde_json::Value> = sens.iter().map(|(k, (d, t))| (*k, serde_json::json!([d, t]))).collect();
        let row = serde_json::json!({
            "id": id, "gen": gen, "age": age, "energy": energy,
            "genome_size": genome.genome_size(), "total_nodes": genome.nodes.len(),
            "reachable": m.reachable_node_count, "executed": m.executed_node_count, "contributing": sets.contributing.len(),
            "knockout": m.knockout_count, "route_varies_battery": m.route_varies_with_input, "hop_cap_battery": m.hop_cap_hits,
            "exec_evolved": exec_evolved, "contrib_evolved": contrib_evolved, "exec_multi_target": exec_multi_target,
            "executed_ids": executed_ids, "contributing_ids": contributing_ids,
            "backends": {"graph_total": m.backends.graph.total, "graph_exec": m.backends.graph.executed, "graph_contrib": m.backends.graph.contributing, "vm_total": m.backends.vm.total, "vm_exec": m.backends.vm.executed, "vm_contrib": m.backends.vm.contributing},
            "entry_backend": entry_backend,
            "reachable_reads": read_classes(&reachable_ids), "executed_reads": read_classes(&exec_all), "contributing_reads": read_classes(&sets.contributing),
            "uses_ext_perception": genome_uses_extended_perception(&genome), "uses_typed_food": genome_uses_typed_local_food(&genome),
            "real": {"actions": real_disc, "food_here": real_food_here, "best_dir_food": nf[best_dir], "lead_move": real_lead_move, "move_into_barrier": real_move_into_barrier, "move_into_occupied": real_move_into_occupied, "move_toward_best": real_move_toward_best, "move_food_rank": real_move_food_rank, "repro_into_blocked": real_repro_into_blocked, "has_barrier_neighbor": has_barrier_neighbor, "last_tick_actions": last_tick_actions, "log_tick": max_tick},
            "sens": sens_json,
            "dir": {"food_seek": [food_seek_exact, food_seek_toward, food_seek_move_any, food_seek_eat_instead, food_seek_n], "fruit_seek": [fruit_seek_exact, fruit_seek_toward, fruit_seek_n], "argmax": [argmax_exact, argmax_toward, argmax_n], "area_food": [area_exact, area_toward, area_n], "barrier": [bar_avoid, bar_changed, bar_n], "occupied": [occ_avoid, occ_changed, occ_n], "repro_occupied": [rocc_avoid, rocc_n], "eat": [eat_when_food, eat_when_none, eat_n]},
            "fruit_matrix": fruit_matrix, "fruit_shift": fruit_shift,
            "route_varies_perturb": route_varies_perturb, "routing_nodes": routing_nodes, "memory_gate_routes": memory_gate_routes,
            "exec_all": exec_all.len(), "chain_med": chain[chain.len() / 2], "chain_max": chain[chain.len() - 1], "hop_caps": hop_caps, "runs": all_runs.len(), "distinct_actions": distinct_actions.len(),
            "log": log,
        });
        println!("{row}");
        if (i + 1) % 50 == 0 { eprintln!("{} genomes in {:.0}s", i + 1, started.elapsed().as_secs_f64()); }
    }
    eprintln!("DONE {} genomes in {:.1}s", files.len(), started.elapsed().as_secs_f64());
}

/// Sequence probe: constant inputs over 24 ticks with persistent graph state
/// and shared memory. Does the move direction cycle on its own (internal
/// pattern generator), and does constant food / a constant barrier at one
/// direction change the sequence?
#[test]
fn probe_live_sequences() {
    use v3_core::simulation::advance_shared_memory;
    let dir = std::env::var("PETRI_GENOME_DIR").expect("PETRI_GENOME_DIR");
    let config_path = std::env::var("PETRI_CONFIG").expect("PETRI_CONFIG");
    let cfg_json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
    let config: SimulationConfig = serde_json::from_value(cfg_json["config"].clone()).unwrap();
    let runtime = config.runtime.clone();
    let food_types = config.world.food.types.len();
    let decay = config.shared_memory.decay_rate;
    const TICKS: usize = 24;
    let seq = |genome: &CreatureGenome, sensors: &SensorSnapshot, energy0: f32, age0: u64, mem0: [f32; 16]| -> Vec<String> {
        let mut energy = energy0;
        let mut shared = mem0;
        let mut prev = mem0;
        let mut gr = GraphRuntimeState::new();
        let mut out = vec![];
        for t in 0..TICKS {
            gr.begin_tick(&genome.nodes, age0 + t as u64);
            let (o, _, _) = execute_creature_mesh_traced(genome, sensors, &mut energy, &mut shared, &prev, &mut gr, &runtime);
            out.push(disc(&o.actions).first().cloned().unwrap_or_else(|| "empty".into()));
            advance_shared_memory(&mut shared, &mut prev, decay);
            energy = energy0; // hold energy constant so only internal state moves
        }
        out
    };
    let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "json")).collect();
    files.sort();
    for path in &files {
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let id = v["id"].as_u64().unwrap_or(0);
        let age = v["age"].as_u64().unwrap_or(0);
        let energy = v["energy"].as_f64().unwrap_or(0.0) as f32;
        let genome: CreatureGenome = serde_json::from_value(v["genome"].clone()).unwrap();
        let mut memory = [0.0f32; 16];
        for (k, x) in v["shared_memory"].as_array().into_iter().flatten().take(16).enumerate() { memory[k] = x.as_f64().unwrap_or(0.0) as f32; }
        let blank = SensorSnapshot {
            local: StaticInputs { food_here: 0.0, neighbor_food: [0.0; 8], neighbor_barrier: [0.0; 8], neighbor_occupied: [0.0; 8], generation: 400.0, age_ticks: age as f32 },
            typed_local_food: TypedFoodLocalSnapshot { food_here_by_type: vec![0.0; food_types], neighbor_food_by_type: vec![[0.0; 8]; food_types] },
            perception: PerceptionSnapshot::zeroed(food_types),
        };
        let s_blank = seq(&genome, &blank, energy, age, memory);
        let mut food3 = blank.clone();
        let mut ring = [0.0; 8]; ring[3] = 1.0;
        set_neighbor_food(&mut food3, 0, ring);
        let s_food3 = seq(&genome, &food3, energy, age, memory);
        // barrier on every direction the blank sequence moved in
        let mut bar = blank.clone();
        for a in &s_blank { if let Some(d) = a.strip_prefix("Move") { if let Ok(d) = d.parse::<usize>() { bar.local.neighbor_barrier[d] = 1.0; } } }
        let s_bar = seq(&genome, &bar, energy, age, memory);
        let moves = |s: &Vec<String>| -> Vec<usize> { s.iter().filter_map(|a| a.strip_prefix("Move").and_then(|d| d.parse().ok())).collect() };
        let mb = moves(&s_blank); let mf = moves(&s_food3); let mbar = moves(&s_bar);
        let distinct = |m: &Vec<usize>| m.iter().collect::<BTreeSet<_>>().len();
        let row = serde_json::json!({
            "id": id,
            "blank": s_blank, "food3": s_food3, "barrier_on_used": s_bar,
            "blank_moves": mb.len(), "blank_distinct_dirs": distinct(&mb),
            "food3_moves": mf.len(), "food3_distinct_dirs": distinct(&mf), "food3_hits_3": mf.iter().filter(|d| **d == 3).count(), "food3_toward": mf.iter().filter(|d| toward(**d, 3)).count(),
            "food_changes_sequence": s_blank != s_food3,
            "barrier_moves": mbar.len(), "barrier_into_blocked": mbar.iter().filter(|d| bar.local.neighbor_barrier[**d] > 0.5).count(), "barrier_changes_sequence": s_blank != s_bar,
        });
        println!("{row}");
    }
}

/// What drives the heading? Sweep one scalar at a time on the creature's real
/// base and record the lead-move direction as a function of it.
#[test]
fn probe_live_heading_drivers() {
    let dir = std::env::var("PETRI_GENOME_DIR").expect("PETRI_GENOME_DIR");
    let config_path = std::env::var("PETRI_CONFIG").expect("PETRI_CONFIG");
    let cfg_json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
    let config: SimulationConfig = serde_json::from_value(cfg_json["config"].clone()).unwrap();
    let runtime = config.runtime.clone();
    let food_types = config.world.food.types.len();
    let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "json")).collect();
    files.sort();
    for path in &files {
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let id = v["id"].as_u64().unwrap_or(0);
        let age = v["age"].as_u64().unwrap_or(0);
        let gen = v["generation"].as_u64().unwrap_or(0);
        let energy = v["energy"].as_f64().unwrap_or(0.0) as f32;
        let genome: CreatureGenome = serde_json::from_value(v["genome"].clone()).unwrap();
        let mut memory = [0.0f32; 16];
        for (k, x) in v["shared_memory"].as_array().into_iter().flatten().take(16).enumerate() { memory[k] = x.as_f64().unwrap_or(0.0) as f32; }
        let ci = &v["diagnostics"]["current_inputs"];
        let mut real = Base {
            sensors: SensorSnapshot {
                local: StaticInputs { food_here: ci["food_here"].as_f64().unwrap_or(0.0) as f32, neighbor_food: f32s(&ci["neighbor_food"]), neighbor_barrier: f32s(&ci["neighbor_barrier"]), neighbor_occupied: f32s(&ci["neighbor_occupied"]), generation: gen as f32, age_ticks: age as f32 },
                typed_local_food: TypedFoodLocalSnapshot { food_here_by_type: vec![0.0; food_types], neighbor_food_by_type: vec![[0.0; 8]; food_types] },
                perception: PerceptionSnapshot::zeroed(food_types),
            },
            energy, age, memory,
        };
        real.sensors.typed_local_food.food_here_by_type[0] = real.sensors.local.food_here;
        real.sensors.typed_local_food.neighbor_food_by_type[0] = real.sensors.local.neighbor_food;
        let lead = |b: &Base| -> String { disc(&run(&genome, b, &runtime).actions).first().cloned().unwrap_or_else(|| "empty".into()) };
        // energy sweep
        let energy_seq: Vec<String> = (1..=30).map(|k| { let mut p = real.clone(); p.energy = k as f32 * 4.0; lead(&p) }).collect();
        // age sweep
        let age_seq: Vec<String> = [0u64, 5, 10, 19, 20, 30, 50, 100, 200, 400].iter().map(|a| { let mut p = real.clone(); p.age = *a; p.sensors.local.age_ticks = *a as f32; lead(&p) }).collect();
        // uniform neighbor food level sweep (all 8 equal)
        let food_level_seq: Vec<String> = (0..=10).map(|k| { let mut p = real.clone(); let lvl = k as f32 / 10.0; set_neighbor_food(&mut p.sensors, 0, [lvl; 8]); set_food_here(&mut p.sensors, 0, 0.0); lead(&p) }).collect();
        let fruit_level_seq: Vec<String> = (0..=10).map(|k| { let mut p = real.clone(); let lvl = k as f32 / 10.0; set_neighbor_food(&mut p.sensors, 1, [lvl; 8]); set_food_here(&mut p.sensors, 0, 0.0); lead(&p) }).collect();
        // food_here sweep
        let here_seq: Vec<String> = (0..=10).map(|k| { let mut p = real.clone(); set_food_here(&mut p.sensors, 0, k as f32 / 10.0); lead(&p) }).collect();
        // memory slot sweep: each slot to 1.0
        let mem_seq: Vec<String> = (0..16).map(|s| { let mut p = real.clone(); p.memory[s] = 1.0; lead(&p) }).collect();
        let dirs = |s: &Vec<String>| -> usize { s.iter().filter_map(|a| a.strip_prefix("Move")).collect::<BTreeSet<_>>().len() };
        let kinds = |s: &Vec<String>| -> usize { s.iter().map(|a| a.trim_end_matches(|c: char| c.is_ascii_digit())).collect::<BTreeSet<_>>().len() };
        println!("{}", serde_json::json!({
            "id": id, "energy_seq": energy_seq.clone(), "energy_dirs": dirs(&energy_seq), "energy_kinds": kinds(&energy_seq),
            "age_dirs": dirs(&age_seq), "age_kinds": kinds(&age_seq),
            "food_level_dirs": dirs(&food_level_seq), "food_level_kinds": kinds(&food_level_seq), "food_level_seq": food_level_seq,
            "fruit_level_dirs": dirs(&fruit_level_seq), "fruit_level_kinds": kinds(&fruit_level_seq),
            "here_dirs": dirs(&here_seq), "here_kinds": kinds(&here_seq),
            "mem_dirs": dirs(&mem_seq), "mem_kinds": kinds(&mem_seq),
        }));
    }
}

/// Per-birth neighborhood (T11.F01 battery) for the first N dumped genomes
/// under the live mutation config: silent / changed / dead per mutated birth.
#[test]
fn probe_live_births() {
    use v3_core::neighborhood::{evaluate_genome, EvalContext};
    let dir = std::env::var("PETRI_GENOME_DIR").expect("PETRI_GENOME_DIR");
    let config_path = std::env::var("PETRI_CONFIG").expect("PETRI_CONFIG");
    let n: usize = std::env::var("PETRI_BIRTHS_N").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
    let cfg_json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
    let config: SimulationConfig = serde_json::from_value(cfg_json["config"].clone()).unwrap();
    let food_types = config.world.food.types.len();
    let battery = Battery::generate(food_types);
    let ctx = EvalContext::from_config(&config);
    let founder = founder_genome(FounderProfile::V3Alpha1);
    let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "json")).collect();
    files.sort();
    let report = |label: &str, genome: &CreatureGenome, seed: u64| {
        let eval = evaluate_genome(genome, &battery, &config.mutation, &ctx, 1, 200, seed);
        let b = &eval.births; let a = &b.any_events;
        let one = b.by_events.get(&1).copied().unwrap_or_default();
        println!("{}", serde_json::json!({"id": label, "nodes": genome.nodes.len(), "genome_size": genome.genome_size(), "births": b.births_total, "zero": b.zero_event_births, "mutated": a.trials - a.skipped, "silent": a.silent, "changed": a.changed, "dead": a.dead, "one_n": one.trials - one.skipped, "one_silent": one.silent, "one_changed": one.changed, "one_dead": one.dead, "by_requested": b.by_requested_events}));
    };
    report("founder", &founder, 0);
    for (i, path) in files.iter().take(n).enumerate() {
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let genome: CreatureGenome = serde_json::from_value(v["genome"].clone()).unwrap();
        report(&v["id"].as_u64().unwrap_or(0).to_string(), &genome, 1_000_000 * (i as u64 + 1));
    }
}
```
