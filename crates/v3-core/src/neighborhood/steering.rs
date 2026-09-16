//! Battery `steering-v1` (T11.F21): does a genome's movement follow food and
//! avoid barriers, and does its executed mesh write a direction bank?
//!
//! Separate from `neighborhood-v1`, which is not edited. Six base scenarios
//! (seed 9) are drawn by the same generator; every reading executes one tick
//! from zeroed shared memory and fresh graph state, exactly as the
//! `neighborhood-v1` snapshots run. Three readings per genome:
//!
//! (a) seeking: for each base and each direction `d`, no food here, the
//!     primary ring one-hot at `d`, every other ring zero; a move is a queue
//!     leading with `Move(c)`, an exact hit `c == d`, within-45 `c` in
//!     `{d-1, d, d+1}` mod 8;
//! (b) avoidance: for each (a) scenario leading with `Move(c)`, the same
//!     scenario with `barrier[c] = 1`; avoided when the lead is no longer
//!     `Move(c)`;
//! (c) `bank_written`: a node in the T11.F14 executed set structurally writes
//!     a bank.
//!
//! Observation only, never a fitness signal.

use std::collections::BTreeSet;

use super::battery::{draw_scenarios, execute_scenario_tick, Scenario};
use crate::config::RuntimeConfig;
use crate::contracts::{Direction, NodeId, WorldAction};
use crate::creature::genome::cgp::ActionSlotBehavior;
use crate::creature::genome::{BackendDef, CreatureGenome, VmInstruction};
use crate::runtime::mesh::UntracedMeshExecution;

pub const STEERING_VERSION: &str = "steering-v1";
/// Fixed seed for the base scenarios.
pub const STEERING_SEED: u64 = 9;
/// Number of base scenarios; each yields one (a) scenario per direction.
pub const STEERING_BASE_COUNT: usize = 6;
/// Chance level of an exact hit: one direction in eight.
pub const CHANCE_EXACT: f64 = 0.125;
/// Chance level of a within-45-degree hit: three directions in eight.
pub const CHANCE_WITHIN_45: f64 = 0.375;

/// The fixed `steering-v1` base scenarios for a world with `food_type_count`
/// ordinary food types.
#[derive(Debug, Clone, PartialEq)]
pub struct SteeringBattery {
    bases: Vec<Scenario>,
}

/// One genome's `steering-v1` reading: counts over the (a) and (b) scenarios
/// and the structural bank-written flag.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SteeringReading {
    /// Number of (a) scenarios executed (`STEERING_BASE_COUNT * 8`).
    pub scenarios: u64,
    /// (a) scenarios whose queue leads with a `Move`.
    pub moves: u64,
    /// Moves whose direction is the food direction.
    pub exact_hits: u64,
    /// Moves within one compass step of the food direction.
    pub within_45: u64,
    /// (b) scenarios executed: one per (a) move.
    pub avoidance_trials: u64,
    /// (b) scenarios whose lead is no longer the barred move.
    pub avoided: u64,
    /// Whether a node the battery executes structurally writes a bank.
    pub bank_written: bool,
}

/// The sums of [`SteeringReading`]s over a sample of genomes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SteeringPooled {
    pub genomes: u64,
    pub scenarios: u64,
    pub moves: u64,
    pub exact_hits: u64,
    pub within_45: u64,
    pub avoidance_trials: u64,
    pub avoided: u64,
    /// Genomes whose `bank_written` is true.
    pub bank_written: u64,
}

impl SteeringPooled {
    /// Fold one genome's reading into the pool.
    #[must_use]
    pub fn merge(self, reading: SteeringReading) -> Self {
        Self {
            genomes: self.genomes + 1,
            scenarios: self.scenarios + reading.scenarios,
            moves: self.moves + reading.moves,
            exact_hits: self.exact_hits + reading.exact_hits,
            within_45: self.within_45 + reading.within_45,
            avoidance_trials: self.avoidance_trials + reading.avoidance_trials,
            avoided: self.avoided + reading.avoided,
            bank_written: self.bank_written + u64::from(reading.bank_written),
        }
    }
}

/// Whether a node in `executed` structurally writes a direction bank: a VM
/// program containing `WriteDirectionBid`, or a graph slot with a movement
/// `Emit` behavior and a non-empty bank.
#[must_use]
pub fn writes_bank(genome: &CreatureGenome, executed: &BTreeSet<NodeId>) -> bool {
    genome
        .nodes
        .iter()
        .filter(|node| executed.contains(&node.node_id))
        .any(|node| match &node.backend_def {
            BackendDef::Vm(vm) => vm
                .program
                .iter()
                .any(|instruction| matches!(instruction, VmInstruction::WriteDirectionBid { .. })),
            BackendDef::Graph(graph) => graph.action_bank.iter().any(|slot| {
                matches!(slot.behavior, ActionSlotBehavior::Emit(kind) if kind.is_movement())
                    && !slot.direction_bids.is_empty()
            }),
        })
}

/// The (a) scenario for `base` and food direction `d`.
fn food_at(base: &Scenario, d: usize) -> Scenario {
    let mut scenario = base.clone();
    let mut ring = [0.0f32; 8];
    ring[d] = 1.0;
    let local = &mut scenario.sensors.local;
    local.food_here = 0.0;
    local.neighbor_food = ring;
    local.neighbor_barrier = [0.0; 8];
    local.neighbor_occupied = [0.0; 8];
    let typed = &mut scenario.sensors.typed_local_food;
    typed.food_here_by_type.iter_mut().for_each(|v| *v = 0.0);
    for (type_idx, typed_ring) in typed.neighbor_food_by_type.iter_mut().enumerate() {
        *typed_ring = if type_idx == 0 { ring } else { [0.0; 8] };
    }
    scenario
}

/// The direction of a queue's leading `Move`, if any.
fn leading_move(actions: &[WorldAction]) -> Option<Direction> {
    match actions.first() {
        Some(WorldAction::Move(direction)) => Some(*direction),
        _ => None,
    }
}

fn within_45(chosen: Direction, food: usize) -> bool {
    let c = chosen.to_index();
    c == food || c == (food + 1) % 8 || c == (food + 7) % 8
}

impl SteeringBattery {
    /// Generate the fixed `steering-v1` bases.
    #[must_use]
    pub fn generate(food_type_count: usize) -> Self {
        Self {
            bases: draw_scenarios(STEERING_SEED, STEERING_BASE_COUNT, food_type_count),
        }
    }

    fn run(
        genome: &CreatureGenome,
        scenario: &Scenario,
        runtime: &RuntimeConfig,
    ) -> Vec<WorldAction> {
        execute_scenario_tick(genome, scenario, runtime, UntracedMeshExecution).actions
    }

    /// Read `genome` against the battery; `executed` is the T11.F14 executed
    /// node set for reading (c).
    #[must_use]
    pub fn read(
        &self,
        genome: &CreatureGenome,
        runtime: &RuntimeConfig,
        executed: &BTreeSet<NodeId>,
    ) -> SteeringReading {
        let mut reading = SteeringReading {
            bank_written: writes_bank(genome, executed),
            ..SteeringReading::default()
        };
        for base in &self.bases {
            for d in 0..8 {
                let scenario = food_at(base, d);
                reading.scenarios += 1;
                let Some(chosen) = leading_move(&Self::run(genome, &scenario, runtime)) else {
                    continue;
                };
                reading.moves += 1;
                reading.exact_hits += u64::from(chosen.to_index() == d);
                reading.within_45 += u64::from(within_45(chosen, d));

                let mut barred = scenario;
                barred.sensors.local.neighbor_barrier[chosen.to_index()] = 1.0;
                reading.avoidance_trials += 1;
                let lead = leading_move(&Self::run(genome, &barred, runtime));
                reading.avoided += u64::from(lead != Some(chosen));
            }
        }
        reading
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{OrdinaryFoodTypeId, SimulationConfig};
    use crate::contracts::{InputReference, WorldInputKey};
    use crate::creature::founder::founder_genome;
    use crate::creature::genome::cgp::{
        ActionSlot, CgpGraphBackendDef, ComputeNode, ComputeNodeKind, DirectionBidEdge,
        ExecuteGate, GraphEdge, GraphSource, WorldActionKind,
    };
    use crate::creature::genome::NodeGenome;
    use crate::neighborhood::Battery;

    fn config() -> SimulationConfig {
        SimulationConfig::default()
    }

    fn founder() -> CreatureGenome {
        founder_genome(config().population.founder_profile)
    }

    fn executed(genome: &CreatureGenome, config: &SimulationConfig) -> BTreeSet<NodeId> {
        Battery::generate(config.world.food.types.len()).executed_node_ids(
            genome,
            &config.runtime,
            config.shared_memory.decay_rate,
        )
    }

    #[test]
    fn generate_is_deterministic_and_predeclared() {
        let a = SteeringBattery::generate(2);
        assert_eq!(a, SteeringBattery::generate(2));
        assert_eq!(a.bases.len(), STEERING_BASE_COUNT);
    }

    #[test]
    fn food_at_zeroes_everything_but_the_primary_ring_slot() {
        let base = &SteeringBattery::generate(2).bases[0];
        let scenario = food_at(base, 3);
        let local = &scenario.sensors.local;
        assert_eq!(local.food_here, 0.0);
        assert_eq!(
            local.neighbor_food,
            [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]
        );
        assert_eq!(local.neighbor_barrier, [0.0; 8]);
        assert_eq!(local.neighbor_occupied, [0.0; 8]);
        assert_eq!(local.age_ticks, base.sensors.local.age_ticks);
        assert_eq!(scenario.energy, base.energy);
        let typed = &scenario.sensors.typed_local_food;
        assert!(typed.food_here_by_type.iter().all(|v| *v == 0.0));
        assert_eq!(typed.neighbor_food_by_type[0], local.neighbor_food);
        assert_eq!(typed.neighbor_food_by_type[1], [0.0; 8]);
    }

    #[test]
    fn within_45_is_the_three_neighbouring_compass_steps() {
        assert!(within_45(Direction::N, 0));
        assert!(within_45(Direction::NW, 0));
        assert!(within_45(Direction::NE, 0));
        assert!(!within_45(Direction::E, 0));
        assert!(within_45(Direction::N, 7));
    }

    #[test]
    fn founder_reading_is_deterministic_and_writes_no_bank() {
        let config = config();
        let genome = founder();
        let battery = SteeringBattery::generate(config.world.food.types.len());
        let executed = executed(&genome, &config);
        let reading = battery.read(&genome, &config.runtime, &executed);
        assert_eq!(reading, battery.read(&genome, &config.runtime, &executed));
        assert_eq!(reading.scenarios, 48);
        assert!(!reading.bank_written);
        assert!(reading.exact_hits <= reading.within_45);
        assert!(reading.within_45 <= reading.moves);
        assert!(reading.moves <= reading.scenarios);
        assert_eq!(reading.avoidance_trials, reading.moves);
        assert!(reading.avoided <= reading.avoidance_trials);
    }

    #[test]
    fn pooled_merge_sums_every_count() {
        let a = SteeringReading {
            scenarios: 48,
            moves: 10,
            exact_hits: 2,
            within_45: 5,
            avoidance_trials: 10,
            avoided: 1,
            bank_written: true,
        };
        let b = SteeringReading {
            bank_written: false,
            ..a
        };
        let pooled = SteeringPooled::default().merge(a).merge(b);
        assert_eq!(
            pooled,
            SteeringPooled {
                genomes: 2,
                scenarios: 96,
                moves: 20,
                exact_hits: 4,
                within_45: 10,
                avoidance_trials: 20,
                avoided: 2,
                bank_written: 1,
            }
        );
    }

    // ── One-edge fixtures (T11.F21 contract) ────────────────────────────────

    const FOOD_RING: InputReference = InputReference::World(WorldInputKey::NeighborFoodRing {
        type_idx: OrdinaryFoodTypeId::new(0),
    });
    const BARRIER_RING: InputReference = InputReference::World(WorldInputKey::NeighborBarrierRing);

    /// A one-node graph genome that always emits `Move(scalar)`: the graph
    /// backend's founder-equivalent for the one-edge fixtures.
    fn graph_mover(scalar: usize) -> CreatureGenome {
        let constant = |value: f32| ComputeNode {
            kind: ComputeNodeKind::Constant(value),
            inputs: Vec::new(),
            plasticity: None,
        };
        let edge = |idx: u16| GraphEdge {
            source: GraphSource::ComputeNode(idx),
            weight: 1.0,
        };
        let mut slot = ActionSlot::inert(ActionSlotBehavior::Emit(WorldActionKind::Move));
        slot.gate_inputs = vec![edge(0)];
        slot.param_inputs = vec![edge(1)];
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![FOOD_RING, BARRIER_RING],
                backend_def: BackendDef::Graph(CgpGraphBackendDef {
                    birth_weights: None,
                    compute_nodes: vec![constant(1.0), constant(scalar as f32)],
                    output_sinks: Vec::new(),
                    action_bank: vec![slot],
                    execute_gate: ExecuteGate {
                        inputs: vec![edge(0)],
                    },
                }),
                targets: Vec::new(),
            }],
        }
    }

    /// `graph_mover` with one bank edge `ring[d] * weight -> bid d`.
    fn graph_mover_with_bid(scalar: usize, ring: u16, d: u8, weight: f32) -> CreatureGenome {
        let mut genome = graph_mover(scalar);
        let BackendDef::Graph(graph) = &mut genome.nodes[0].backend_def else {
            unreachable!()
        };
        graph.action_bank[0].direction_bids.push(DirectionBidEdge {
            edge: GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: ring,
                    sub_idx: u16::from(d),
                },
                weight,
            },
            direction: d,
        });
        genome
    }

    /// The founder with `ring[d] -> bid d` (negated when `negative`) written
    /// at the start of its VM decision node: prepending keeps every relative
    /// jump valid, and the bank persists to whichever push the dispatch
    /// reaches.
    fn vm_founder_with_bid(ring: InputReference, d: u8, negative: bool) -> CreatureGenome {
        let mut genome = founder();
        let node = &mut genome.nodes[1];
        let ref_idx = node.input_refs.len() as u16;
        node.input_refs.push(ring);
        let BackendDef::Vm(vm) = &mut node.backend_def else {
            unreachable!()
        };
        let mut prefix = vec![VmInstruction::ReadInput {
            dst: 12,
            ref_idx,
            sub_idx: u16::from(d),
        }];
        if negative {
            prefix.push(VmInstruction::Neg { dst: 12, src: 12 });
        }
        prefix.push(VmInstruction::WriteDirectionBid {
            direction: d,
            src: 12,
        });
        prefix.append(&mut vm.program);
        vm.program = prefix;
        genome
    }

    /// On the `neighborhood-v1` snapshots, `with` differs from `without` only
    /// where the base's `ring[d]` is nonzero, and there only by moving every
    /// movement action to `d`.
    fn assert_only_ring_scenarios_differ(
        with: &CreatureGenome,
        without: &CreatureGenome,
        config: &SimulationConfig,
        ring: impl Fn(&Scenario) -> [f32; 8],
        d: usize,
        expect_difference: bool,
    ) {
        let battery = Battery::generate(config.world.food.types.len());
        let decay = config.shared_memory.decay_rate;
        let with_sig = battery.signature(with, &config.runtime, decay);
        let without_sig = battery.signature(without, &config.runtime, decay);
        let mut differed = 0;
        for ((scenario, a), b) in battery
            .snapshots()
            .iter()
            .zip(&with_sig.snapshots)
            .zip(&without_sig.snapshots)
        {
            if ring(scenario)[d] == 0.0 {
                assert_eq!(a, b, "a zero cue must leave the queue unchanged");
                continue;
            }
            assert_eq!(a.len(), b.len());
            for (x, y) in a.iter().zip(b) {
                assert_eq!(x.action_type(), y.action_type());
                if let Some(direction) = x.direction() {
                    assert_eq!(direction.to_index(), d);
                }
                if x != y {
                    differed += 1;
                }
            }
        }
        assert!(
            differed > 0 || !expect_difference,
            "some snapshot must exercise the bank"
        );
    }

    /// Over the (a) scenarios with food at `d`: how many `genome` leads with
    /// a `Move`, and how many of those are `Move(d)`.
    fn moves_on_food_at(
        battery: &SteeringBattery,
        genome: &CreatureGenome,
        config: &SimulationConfig,
        d: usize,
    ) -> (u64, u64) {
        let mut moves = 0;
        let mut hits = 0;
        for base in &battery.bases {
            if let Some(c) = leading_move(&SteeringBattery::run(
                genome,
                &food_at(base, d),
                &config.runtime,
            )) {
                moves += 1;
                hits += u64::from(c.to_index() == d);
            }
        }
        (moves, hits)
    }

    #[test]
    fn graph_one_positive_food_edge_hits_every_direction_exactly() {
        let config = config();
        let battery = SteeringBattery::generate(config.world.food.types.len());
        let scalar = 2;
        let plain = battery.read(&graph_mover(scalar), &config.runtime, &BTreeSet::new());
        assert_eq!(plain.moves, 48);
        assert_eq!(plain.exact_hits, 6, "the scalar mover hits only food at E");
        assert_eq!(plain.within_45, 18);
        assert!(!plain.bank_written);
        for d in 0..8u8 {
            let genome = graph_mover_with_bid(scalar, 0, d, 1.0);
            let executed = executed(&genome, &config);
            let reading = battery.read(&genome, &config.runtime, &executed);
            assert!(reading.bank_written, "d={d}");
            assert_eq!(reading.moves, 48, "d={d}");
            assert_eq!(
                moves_on_food_at(&battery, &genome, &config, usize::from(d)),
                (6, 6),
                "d={d}: food at d is an exact hit on every base"
            );
            // Elsewhere the bid is 0, the bank all-ties, and the scalar wins.
            let steered = u64::from(usize::from(d) != scalar) * 6;
            assert_eq!(reading.exact_hits, plain.exact_hits + steered, "d={d}");
        }
    }

    #[test]
    fn graph_one_positive_food_edge_changes_nothing_else() {
        let config = config();
        for d in 0..8usize {
            assert_only_ring_scenarios_differ(
                &graph_mover_with_bid(2, 0, d as u8, 1.0),
                &graph_mover(2),
                &config,
                |scenario| scenario.sensors.local.neighbor_food,
                d,
                d != 2,
            );
        }
    }

    #[test]
    fn graph_one_negative_barrier_edge_avoids_and_changes_nothing_else() {
        let config = config();
        let battery = SteeringBattery::generate(config.world.food.types.len());
        let scalar = 2;
        let plain = battery.read(&graph_mover(scalar), &config.runtime, &BTreeSet::new());
        assert_eq!(plain.avoided, 0);
        for d in 0..8u8 {
            let genome = graph_mover_with_bid(scalar, 1, d, -1.0);
            let reading = battery.read(&genome, &config.runtime, &executed(&genome, &config));
            assert_eq!(
                (reading.moves, reading.exact_hits, reading.within_45),
                (plain.moves, plain.exact_hits, plain.within_45),
                "d={d}: no barrier, no change"
            );
            assert_eq!(reading.avoidance_trials, plain.avoidance_trials);
            let expected = if usize::from(d) == scalar {
                plain.avoidance_trials
            } else {
                0
            };
            assert_eq!(reading.avoided, expected, "d={d}");
        }
        // The neighborhood-v1 battery carries no barriers, so the queue is
        // byte-identical to the plain mover on every scenario.
        let neighborhood = Battery::generate(config.world.food.types.len());
        let decay = config.shared_memory.decay_rate;
        assert_eq!(
            neighborhood.signature(
                &graph_mover_with_bid(scalar, 1, scalar as u8, -1.0),
                &config.runtime,
                decay
            ),
            neighborhood.signature(&graph_mover(scalar), &config.runtime, decay)
        );
    }

    #[test]
    fn vm_one_positive_food_edge_hits_every_direction_and_changes_nothing_else() {
        let config = config();
        let battery = SteeringBattery::generate(config.world.food.types.len());
        let founder = founder();
        let plain = battery.read(&founder, &config.runtime, &executed(&founder, &config));
        for d in 0..8u8 {
            let genome = vm_founder_with_bid(FOOD_RING, d, false);
            let reading = battery.read(&genome, &config.runtime, &executed(&genome, &config));
            assert!(reading.bank_written, "d={d}");
            assert_eq!(reading.moves, plain.moves, "d={d}");
            let (moves, hits) = moves_on_food_at(&battery, &genome, &config, usize::from(d));
            assert_eq!(
                (moves, hits),
                (
                    moves_on_food_at(&battery, &founder, &config, usize::from(d)).0,
                    moves
                ),
                "d={d}: food at d is an exact hit wherever the founder moves"
            );
            assert!(reading.exact_hits >= plain.exact_hits, "d={d}");
            assert_only_ring_scenarios_differ(
                &genome,
                &founder,
                &config,
                |scenario| scenario.sensors.local.neighbor_food,
                usize::from(d),
                true,
            );
        }
    }

    #[test]
    fn vm_one_negative_barrier_edge_avoids_where_the_founder_led_and_changes_nothing_else() {
        let config = config();
        let battery = SteeringBattery::generate(config.world.food.types.len());
        let founder = founder();
        let plain = battery.read(&founder, &config.runtime, &executed(&founder, &config));
        assert_eq!(plain.avoided, 0, "the founder ignores barriers");
        let neighborhood = Battery::generate(config.world.food.types.len());
        let decay = config.shared_memory.decay_rate;
        let founder_signature = neighborhood.signature(&founder, &config.runtime, decay);
        let mut avoided_total = 0;
        for d in 0..8u8 {
            let genome = vm_founder_with_bid(BARRIER_RING, d, true);
            let reading = battery.read(&genome, &config.runtime, &executed(&genome, &config));
            assert_eq!(
                (
                    reading.moves,
                    reading.exact_hits,
                    reading.within_45,
                    reading.avoidance_trials
                ),
                (
                    plain.moves,
                    plain.exact_hits,
                    plain.within_45,
                    plain.avoidance_trials
                ),
                "d={d}: an all-zero bank leaves the scalar decode in force"
            );
            let led_with_d = leads_with_direction(&battery, &founder, &config, usize::from(d));
            assert_eq!(reading.avoided, led_with_d, "d={d}");
            avoided_total += reading.avoided;
            assert_eq!(
                neighborhood.signature(&genome, &config.runtime, decay),
                founder_signature,
                "d={d}: no barrier in neighborhood-v1, so nothing changes"
            );
        }
        assert_eq!(
            avoided_total, plain.moves,
            "every founder move is barred exactly once"
        );
    }

    /// How many (a) scenarios `genome` leads with `Move(d)`.
    fn leads_with_direction(
        battery: &SteeringBattery,
        genome: &CreatureGenome,
        config: &SimulationConfig,
        d: usize,
    ) -> u64 {
        let mut count = 0;
        for base in &battery.bases {
            for food in 0..8 {
                let actions = SteeringBattery::run(genome, &food_at(base, food), &config.runtime);
                if leading_move(&actions).is_some_and(|c| c.to_index() == d) {
                    count += 1;
                }
            }
        }
        count
    }
}
