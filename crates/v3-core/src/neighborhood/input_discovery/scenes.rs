use crate::config::{FoodTypeConfig, OrdinaryFoodTypeId, SimulationConfig};
use crate::contracts::{Direction, NodeId, Position, WorldAction};
use crate::creature::action_log::ActionLogEntry;
use crate::creature::genome::{BackendDef, CreatureGenome};
use crate::creature::identity::CreatureIdentityState;
use crate::creature::state::CreatureState;
use crate::kernel::WorldState;
use crate::neighborhood::opportunity::fixtures::Local;
use crate::runtime::trace::domain::{BackendTrace, TickTrace};
use crate::runtime::trace::recording::ActiveTrace;
use crate::simulation::{run_tick, Simulation};
use serde::Serialize;
use slotmap::SlotMap;
use std::collections::BTreeSet;

/// Fixed native world settings; mutation and all runtime/energy settings stay native.
pub fn scene_config() -> SimulationConfig {
    let mut config = super::super::recruitment_paths::task_config();
    config.world.food.types = vec![FoodTypeConfig::default(); 2];
    for food in &mut config.world.food.types {
        food.initial_coverage = 0.0;
    }
    config
}

/// A clone-only expression mask, never inherited by offspring.
pub fn learning_off(genome: &CreatureGenome) -> (CreatureGenome, usize) {
    let mut masked = genome.clone();
    let mut removed = 0;
    for node in &mut masked.nodes {
        if let BackendDef::Graph(graph) = &mut node.backend_def {
            for compute in &mut graph.compute_nodes {
                removed += usize::from(compute.plasticity.take().is_some());
            }
        }
    }
    (masked, removed)
}

#[derive(Debug, Clone, Serialize)]
pub struct Scene {
    pub local_grass: bool,
    pub local_fruit: bool,
    pub ring_food: Option<u8>,
    pub eligible: bool,
    pub barrier: Option<u8>,
    pub far_food: Option<u8>,
    pub far_offset: Option<(i32, i32)>,
    pub diagonal_barrier: bool,
    pub distance: i32,
    pub position: Position,
    pub energy: f32,
}

impl Scene {
    pub fn new(
        local: Local,
        ring_food: Option<u8>,
        eligible: bool,
        barrier: Option<u8>,
        far_food: Option<u8>,
        held_out: bool,
    ) -> Self {
        Self {
            local_grass: matches!(local, Local::Grass | Local::GrassAndFruit),
            local_fruit: matches!(local, Local::Fruit | Local::GrassAndFruit),
            ring_food,
            eligible,
            barrier,
            far_food,
            far_offset: None,
            diagonal_barrier: false,
            distance: if held_out { 3 } else { 2 },
            position: if held_out {
                Position::new(5, 5)
            } else {
                Position::new(6, 6)
            },
            energy: match (held_out, eligible) {
                (false, false) => 20.0,
                (false, true) => 80.0,
                (true, false) => 25.0,
                (true, true) => 90.0,
            },
        }
    }

    pub fn world(&self) -> WorldState {
        let config = scene_config();
        let mut world = WorldState::new(12, 12, config.world.edge_mode);
        world.reconfigure_food(config.world.food);
        world.set_food_type(
            self.position,
            OrdinaryFoodTypeId::new(0),
            f32::from(self.local_grass),
        );
        world.set_food_type(
            self.position,
            OrdinaryFoodTypeId::new(1),
            f32::from(self.local_fruit),
        );
        if let Some(direction) = self.ring_food {
            world.set_food(self.offset(direction, 1), 1.0);
        }
        if let Some(direction) = self.far_food {
            world.set_food(self.offset(direction, self.distance), 1.0);
        }
        if let Some((dx, dy)) = self.far_offset {
            world.set_food(
                Position::new(
                    u16::try_from(i32::from(self.position.x) + dx).expect("fixed scene x"),
                    u16::try_from(i32::from(self.position.y) + dy).expect("fixed scene y"),
                ),
                1.0,
            );
        }
        if let Some(direction) = self.barrier {
            world.set_barrier(self.offset(direction, 1), true);
        }
        if self.diagonal_barrier {
            let (dx, dy) = self
                .far_offset
                .expect("ring diagonal has a frozen off-axis cue");
            world.set_barrier(
                Position::new(
                    u16::try_from(i32::from(self.position.x) - dx.signum())
                        .expect("fixed diagonal x"),
                    u16::try_from(i32::from(self.position.y) - dy.signum())
                        .expect("fixed diagonal y"),
                ),
                true,
            );
        }
        world
    }

    pub fn offset(&self, direction: u8, distance: i32) -> Position {
        let (dx, dy) = Direction::ALL[usize::from(direction)].delta();
        Position::new(
            u16::try_from(i32::from(self.position.x) + dx * distance)
                .expect("fixed scene stays in bounds"),
            u16::try_from(i32::from(self.position.y) + dy * distance)
                .expect("fixed scene stays in bounds"),
        )
    }

    pub fn run(&self, genome: &CreatureGenome) -> SceneReading {
        let (masked, _) = learning_off(genome);
        let mut world = self.world();
        let mut creatures = SlotMap::with_key();
        let id = creatures.insert_with_key(|id| {
            let mut creature = CreatureState::new(
                id,
                masked,
                self.position,
                self.energy,
                0,
                [0; 6],
                0,
                [true; 6],
                CreatureIdentityState::default(),
                [0.0; 16],
            );
            creature.age = 50;
            creature
        });
        world.place_creature(self.position, id);
        let mut simulation = Simulation::new(world, creatures, 0, scene_config(), 41);
        simulation
            .action_logs
            .insert(id, crate::creature::action_log::ActionLog::new(64));
        let mut trace = Some(ActiveTrace::new(id, 1));
        trace
            .as_mut()
            .expect("active native trace")
            .include_perception_debug = true;
        run_tick(&mut simulation, &mut trace);
        let tick = trace.as_ref().and_then(|trace| trace.ticks.first());
        let creature = simulation.creatures.get(id);
        let flows = &simulation.stats.energy_flows;
        let mut reading = SceneReading {
            actions: tick.map_or_else(Vec::new, |tick| tick.final_actions.clone()),
            applied: simulation
                .action_logs
                .get(id)
                .map_or_else(Vec::new, |log| log.entries().iter().copied().collect()),
            food_intake: flows.food_intake_by_type.clone(),
            carrying: flows.genome_carrying,
            plasticity_updates: simulation.stats.plasticity_updates_total,
            learning_charge: flows.hebbian_learning + flows.reward_learning,
            alive: creature.is_some_and(|creature| creature.energy > 0.0),
            position: creature.map(|creature| creature.position),
            energy: creature.map(|creature| creature.energy),
            mesh_hops: simulation.stats.mesh_hops_total,
            graph_visits: simulation.stats.graph_relax_iters_total,
            vm_steps: simulation.stats.vm_steps_total,
            graph_charge: flows.graph_compute,
            vm_charge: flows.vm_compute,
            mesh_charge: flows.mesh_ramp,
            perception_assembled: crate::sensors::perception::genome_uses_extended_perception(
                genome,
            ),
            action_charge: flows.action_charges.noop
                + flows.action_charges.eat
                + flows.action_charges.r#move
                + flows.action_charges.reproduce
                + flows.action_charges.steal_energy
                + flows.failed_action_penalty,
            dispatched: tick.map_or_else(BTreeSet::new, |tick| {
                tick.hops.iter().map(|hop| hop.node_id).collect()
            }),
            area_food: tick.and_then(|tick| {
                tick.debug_perception
                    .as_ref()
                    .map(|perception| perception.area_food)
            }),
            reads: BTreeSet::new(),
        };
        if let Some(tick) = tick {
            reading.reads = executed_reads(genome, tick);
        }
        assert_eq!(
            reading.plasticity_updates, 0,
            "expression mask must prevent learning"
        );
        assert_eq!(
            reading.learning_charge, 0.0,
            "expression mask must prevent reward updates"
        );
        reading
    }
}

/// Semantic reads reconstructed from existing native backend execution traces.
fn executed_reads(genome: &CreatureGenome, tick: &TickTrace) -> BTreeSet<(NodeId, u16, u16)> {
    use crate::creature::genome::{cgp::GraphSource, VmInstruction};
    let mut reads = BTreeSet::new();
    for hop in &tick.hops {
        let Some(node) = genome.nodes.iter().find(|node| node.node_id == hop.node_id) else {
            continue;
        };
        match (&node.backend_def, &hop.backend_trace) {
            (BackendDef::Graph(graph), BackendTrace::Graph(trace)) => {
                let mut record = |source| {
                    if let GraphSource::InputLeaf { ref_idx, sub_idx } = source {
                        reads.insert((hop.node_id, ref_idx, sub_idx));
                    }
                };
                for evaluated in trace.passes.iter().flat_map(|pass| &pass.node_evaluations) {
                    for edge in &graph.compute_nodes[evaluated.node_index].inputs {
                        record(edge.source);
                    }
                }
                if !trace.output_sinks.is_empty() {
                    for edge in graph.output_sinks.iter().flat_map(|sink| &sink.inputs) {
                        record(edge.source);
                    }
                }
            }
            (BackendDef::Vm(_), BackendTrace::Vm(trace)) => {
                for step in &trace.steps {
                    if step.energy_after > 0.0 {
                        if let VmInstruction::ReadInput {
                            ref_idx, sub_idx, ..
                        } = step.instruction
                        {
                            reads.insert((hop.node_id, ref_idx, sub_idx));
                        }
                    }
                }
            }
            _ => unreachable!("native trace backend matches genome"),
        }
    }
    reads
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SceneReading {
    pub actions: Vec<WorldAction>,
    pub applied: Vec<ActionLogEntry>,
    pub food_intake: Vec<f64>,
    pub carrying: f64,
    pub plasticity_updates: u64,
    pub learning_charge: f64,
    pub alive: bool,
    pub position: Option<Position>,
    pub energy: Option<f32>,
    pub mesh_hops: u64,
    pub graph_visits: u64,
    pub vm_steps: u64,
    pub graph_charge: f64,
    pub vm_charge: f64,
    pub mesh_charge: f64,
    /// Native perception is conditionally assembled but has no separate energy debit.
    pub perception_assembled: bool,
    pub action_charge: f64,
    pub dispatched: BTreeSet<NodeId>,
    pub area_food: Option<[f32; 7]>,
    pub reads: BTreeSet<(NodeId, u16, u16)>,
}
