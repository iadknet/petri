//! The discovery baseline and feasibility record of each family: how often
//! one production mutation event declares the family on the founder's vote
//! node, how often one more connects an authored channel to a vote or
//! parameter sink, the Graph cost of the authored controller, and the VM
//! instructions a hand translation would take (counted, not run).

use std::collections::BTreeSet;

use rand::rngs::SmallRng;
use rand::SeedableRng;

use crate::config::{MutationConfig, SimulationConfig};
use crate::contracts::NodeId;
use crate::creature::founder::FOUNDER_GENOME_SIZE_UNITS;
use crate::creature::genome::analysis::{mesh_reachable_nodes, vm_backward_slice};
use crate::creature::genome::cgp::{CgpGraphBackendDef, GraphEdge, GraphSource, OutputSinkKind};
use crate::creature::genome::vote::{ActionParamField, VoteSink};
use crate::creature::genome::{BackendDef, CreatureGenome, NodeGenome, VmInstruction};
use crate::mutation::reachability::ParentExecuted;
use crate::mutation::MutationEngine;
use crate::runtime::mesh::UntracedMeshExecution;
use crate::runtime::vm::{opcode_base_cost, step_charge};
use crate::sensors::perception::genome_uses_extended_perception;

use super::super::input_use::catalog::{addressed, Addressed, Channel, Family as CatalogFamily};
use super::super::mesh_execution::indices_for_node_ids;
use super::super::Battery;
use super::controllers::{controller, AuthoredStructure, Family, VOTE_NODE};
use super::fixtures::competence_contexts;

/// Declare-step seed: `DECLARE_SEED_BASE + 10_000 family_index + i`.
pub const DECLARE_SEED_BASE: u64 = 28_000_000;
/// Connect-step seed: `CONNECT_SEED_BASE + 10_000 family_index + i`.
pub const CONNECT_SEED_BASE: u64 = 29_000_000;
pub const FAMILY_SEED_STRIDE: u64 = 10_000;
pub const DISCOVERY_PROPOSALS: u32 = 10_000;
/// The configured food types of the discovery proposals.
pub const DISCOVERY_FOOD_TYPES: usize = 2;

/// One step's counts over its proposals.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Step {
    pub proposals: u32,
    /// Declare: the child declares the family on the vote node. Connect: the
    /// child gains a live consumer of an authored channel on a vote or
    /// parameter sink.
    pub hits: u32,
    /// Declare: declarations on another executed node. Connect: live
    /// consumers of any channel of the family.
    pub beside: u32,
}

impl Step {
    #[must_use]
    pub fn share(self) -> Option<f64> {
        (self.proposals > 0).then(|| f64::from(self.hits) / f64::from(self.proposals))
    }
}

/// One family's discovery baseline.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Discovery {
    pub declare: Step,
    pub connect: Step,
    /// Requested events per birth at the founder's size: `97 * per_unit_rate`.
    pub events_per_birth: f64,
    /// `(1 / p_declare + 1 / p_connect) / events_per_birth`, an instrument
    /// figure; `None` when either share is zero.
    pub implied_births: Option<f64>,
}

fn vote_node(genome: &CreatureGenome, id: NodeId) -> Option<&NodeGenome> {
    genome.nodes.iter().find(|node| node.node_id == id)
}

fn declares(node: &NodeGenome, family: Family) -> bool {
    node.input_refs.contains(&family.reference())
}

/// Channels read by a consumer that feeds a wired vote or parameter sink:
/// the sink's own edges and the inputs of every compute node a backward walk
/// from those sinks reaches (Graph), or the `ReadInput`s in the backward
/// slices of `AddVote` and `WriteActionParam` (VM).
fn vote_surface_channels(node: &NodeGenome) -> BTreeSet<Channel> {
    let channel = |ref_idx: u16, sub_idx: u16| {
        node.input_refs
            .get(usize::from(ref_idx))
            .and_then(|reference| match addressed(reference, sub_idx) {
                Addressed::Channel(channel) => Some(channel),
                Addressed::OutOfWidth(_) => None,
            })
    };
    match &node.backend_def {
        BackendDef::Graph(graph) => graph_vote_edges(graph)
            .into_iter()
            .filter_map(|edge| match edge.source {
                GraphSource::InputLeaf { ref_idx, sub_idx } => channel(ref_idx, sub_idx),
                _ => None,
            })
            .collect(),
        BackendDef::Vm(vm) => vm
            .program
            .iter()
            .enumerate()
            .filter(|(_, instruction)| {
                matches!(
                    instruction,
                    VmInstruction::AddVote { .. } | VmInstruction::WriteActionParam { .. }
                )
            })
            .filter_map(|(index, _)| vm_backward_slice(&vm.program, index))
            .flat_map(|gene| gene.indices)
            .filter_map(|index| match vm.program[index] {
                VmInstruction::ReadInput {
                    ref_idx, sub_idx, ..
                } => channel(ref_idx, sub_idx),
                _ => None,
            })
            .collect(),
    }
}

fn graph_vote_edges(graph: &CgpGraphBackendDef) -> Vec<&GraphEdge> {
    let mut edges: Vec<&GraphEdge> = graph
        .output_sinks
        .iter()
        .filter(|sink| {
            matches!(
                sink.kind,
                OutputSinkKind::ActionVote(_) | OutputSinkKind::ActionParam(_)
            )
        })
        .flat_map(|sink| &sink.inputs)
        .collect();
    let mut seen = vec![false; graph.compute_nodes.len()];
    let mut queue: Vec<usize> = Vec::new();
    let mut cursor = 0;
    loop {
        while cursor < edges.len() {
            if let GraphSource::ComputeNode(index) = edges[cursor].source {
                let index = usize::from(index);
                if index < seen.len() && !seen[index] {
                    seen[index] = true;
                    queue.push(index);
                }
            }
            cursor += 1;
        }
        let Some(index) = queue.pop() else {
            break;
        };
        edges.extend(&graph.compute_nodes[index].inputs);
    }
    edges
}

/// `founder` with `family`'s reference declared on the vote node and nothing
/// else: the connect step's exact intermediate.
#[must_use]
pub fn intermediate(founder: &CreatureGenome, family: Family) -> CreatureGenome {
    let mut genome = founder.clone();
    genome.nodes[VOTE_NODE].input_refs.push(family.reference());
    genome
}

/// Run `proposals` single-event proposals on `parent`, seeding proposal `i`
/// with `seed_base + i`, and count each child `judge` accepts.
fn step(
    parent: &CreatureGenome,
    battery: &Battery,
    config: &SimulationConfig,
    seed_base: u64,
    proposals: u32,
    judge: impl Fn(&CreatureGenome) -> (bool, bool) + Sync,
) -> Step {
    use rayon::prelude::*;
    let one_event = MutationConfig {
        per_unit_rate: 1.0,
        ..config.mutation.clone()
    };
    let reachable = mesh_reachable_nodes(parent);
    let sets =
        battery.mesh_execution_sets(parent, &config.runtime, config.shared_memory.decay_rate);
    let executed = indices_for_node_ids(parent, &sets.executed);
    let judged: Vec<(bool, bool)> = (0..proposals)
        .into_par_iter()
        .map(|i| {
            let mut child = parent.clone();
            let mut rng = SmallRng::seed_from_u64(seed_base + u64::from(i));
            MutationEngine::apply_mutations_on_units(
                &mut child,
                1,
                &one_event,
                &reachable,
                ParentExecuted::Indices(&executed),
                &mut rng,
                DISCOVERY_FOOD_TYPES,
            );
            judge(&child)
        })
        .collect();
    Step {
        proposals,
        hits: judged.iter().filter(|(hit, _)| *hit).count() as u32,
        beside: judged.iter().filter(|(_, beside)| *beside).count() as u32,
    }
}

/// `family`'s discovery baseline on `founder` with `proposals` per step
/// ([`DISCOVERY_PROPOSALS`] in production).
#[must_use]
pub fn discovery(
    founder: &CreatureGenome,
    family: Family,
    config: &SimulationConfig,
    proposals: u32,
) -> Discovery {
    let battery = Battery::generate(DISCOVERY_FOOD_TYPES);
    let vote_id = founder.nodes[VOTE_NODE].node_id;
    let sets =
        battery.mesh_execution_sets(founder, &config.runtime, config.shared_memory.decay_rate);
    let others: Vec<NodeId> = sets
        .executed
        .iter()
        .copied()
        .filter(|&id| id != vote_id)
        .collect();
    let stride = FAMILY_SEED_STRIDE * family.index();
    let declare = step(
        founder,
        &battery,
        config,
        DECLARE_SEED_BASE + stride,
        proposals,
        |child| {
            let on_vote = vote_node(child, vote_id).is_some_and(|node| declares(node, family));
            let elsewhere = others
                .iter()
                .any(|&id| vote_node(child, id).is_some_and(|node| declares(node, family)));
            (on_vote, elsewhere)
        },
    );
    let authored: BTreeSet<Channel> = family.authored_channels().into_iter().collect();
    let family_key = CatalogFamily::of(&family.reference());
    let parent = intermediate(founder, family);
    let connect = step(
        &parent,
        &battery,
        config,
        CONNECT_SEED_BASE + stride,
        proposals,
        |child| {
            let channels = vote_node(child, vote_id)
                .map(vote_surface_channels)
                .unwrap_or_default();
            (
                channels.iter().any(|channel| authored.contains(channel)),
                channels.iter().any(|channel| channel.family == family_key),
            )
        },
    );
    let events_per_birth = f64::from(FOUNDER_GENOME_SIZE_UNITS) * config.mutation.per_unit_rate;
    let implied_births = match (declare.share(), connect.share()) {
        (Some(p), Some(q)) if p > 0.0 && q > 0.0 => Some((1.0 / p + 1.0 / q) / events_per_birth),
        _ => None,
    };
    Discovery {
        declare,
        connect,
        events_per_birth,
        implied_births,
    }
}

/// Mean work of one execution over the fixture contexts.
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Work {
    pub executions: u32,
    pub mesh_hops: u64,
    pub graph_relax_iters: u64,
    pub vm_steps: u64,
    /// Energy the executions spent, summed.
    pub energy_spent: f64,
}

fn work_on_fixtures(genome: &CreatureGenome, family: Family, config: &SimulationConfig) -> Work {
    let mut work = Work::default();
    for context in competence_contexts(family) {
        let scenario = context.scenario();
        let mut energy = scenario.energy;
        let output = crate::runtime::mesh::execute_creature_mesh_impl(
            genome,
            &scenario.sensors,
            &mut energy,
            &mut [0.0; 16],
            &[0.0; 16],
            &mut crate::creature::state::GraphRuntimeState::new(),
            &config.runtime,
            UntracedMeshExecution,
        );
        let counters = output.work_counters;
        work.executions += 1;
        work.mesh_hops += u64::from(counters.mesh_hops);
        work.graph_relax_iters += u64::from(counters.graph_relax_iters);
        work.vm_steps += u64::from(counters.vm_steps);
        work.energy_spent += f64::from(scenario.energy - energy);
    }
    work
}

/// One family's Graph feasibility record.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Feasibility {
    pub structure: AuthoredStructure,
    pub added_genome_units: u32,
    /// `added_genome_units * genome_carry_cost_per_unit`.
    pub maintenance_per_tick: f64,
    /// Whether the controller's genome makes the host assemble extended
    /// perception (audit S5), for `A_k` and `Z_k` alike.
    pub extended_perception: bool,
    pub founder_work: Work,
    pub authored_work: Work,
    /// Only ordinary Graph structure feeding existing vote or parameter sinks.
    pub feasible: bool,
}

/// `family`'s Graph feasibility.
#[must_use]
pub fn feasibility(
    founder: &CreatureGenome,
    family: Family,
    config: &SimulationConfig,
) -> Feasibility {
    let (authored, structure) = controller(founder, family, false);
    let added = authored.genome_size() - founder.genome_size();
    let BackendDef::Graph(before) = &founder.nodes[VOTE_NODE].backend_def else {
        panic!("the founder's vote node is a Graph node");
    };
    let BackendDef::Graph(after) = &authored.nodes[VOTE_NODE].backend_def else {
        panic!("a controller keeps the vote node a Graph node");
    };
    let sinks_ok = before
        .output_sinks
        .iter()
        .zip(&after.output_sinks)
        .all(|(b, a)| {
            b.inputs == a.inputs
                || matches!(
                    a.kind,
                    OutputSinkKind::ActionVote(_) | OutputSinkKind::ActionParam(_)
                )
        });
    Feasibility {
        structure,
        added_genome_units: added,
        maintenance_per_tick: f64::from(added)
            * f64::from(config.energy.lifecycle.genome_carry_cost_per_unit),
        extended_perception: genome_uses_extended_perception(&authored),
        founder_work: work_on_fixtures(founder, family, config),
        authored_work: work_on_fixtures(&authored, family, config),
        feasible: sinks_ok && authored.nodes.len() == founder.nodes.len(),
    }
}

/// A hand translation of `family`'s controller into VM instructions appended
/// to a VM translation of the founder's vote node, reading the founder's
/// `can reproduce` (register 1) and primary-food flag (register 9) as the
/// founder translation leaves them; the authored reference is index 7.
#[must_use]
pub fn vm_translation(family: Family) -> Vec<VmInstruction> {
    use VmInstruction::*;
    let vote = |sink: VoteSink, src: u8| AddVote {
        sink: sink.index() as u8,
        src,
    };
    let read = |dst: u8, sub_idx: u16| ReadInput {
        dst,
        ref_idx: 7,
        sub_idx,
    };
    match family {
        // Constant pool entry 7 holds the inhibition weight.
        Family::Ring => {
            let mut program = vec![LoadConst {
                dst: 24,
                const_idx: 7,
            }];
            for (register, direction) in [(20u8, 0u8), (21, 2), (22, 4), (23, 6)] {
                program.extend([
                    read(register, u16::from(direction)),
                    Mul {
                        dst: register,
                        a: register,
                        b: 24,
                    },
                    vote(VoteSink::Move(direction), register),
                ]);
            }
            program
        }
        // Constant pool entries 8 and 9 hold the far and gate thresholds.
        Family::Vector => vec![
            read(20, 5),
            read(21, 3),
            read(22, 4),
            LoadConst {
                dst: 23,
                const_idx: 8,
            },
            CmpGt {
                dst: 24,
                a: 20,
                b: 23,
            },
            Sub {
                dst: 24,
                a: 24,
                b: 1,
            },
            Sub {
                dst: 24,
                a: 24,
                b: 9,
            },
            LoadConst {
                dst: 23,
                const_idx: 9,
            },
            CmpGt {
                dst: 25,
                a: 24,
                b: 23,
            },
            Mul {
                dst: 21,
                a: 21,
                b: 25,
            },
            Mul {
                dst: 22,
                a: 22,
                b: 25,
            },
            vote(VoteSink::Move(2), 21),
            vote(VoteSink::Move(4), 22),
            Neg { dst: 21, src: 21 },
            Neg { dst: 22, src: 22 },
            vote(VoteSink::Move(6), 21),
            vote(VoteSink::Move(0), 22),
        ],
        // Constant pool entry 0 holds 0.0 and entry 9 the 0.5 threshold:
        // `fruit` in register 22, `fruit_only` in register 23.
        Family::Scalar => vec![
            read(20, 0),
            LoadConst {
                dst: 21,
                const_idx: 0,
            },
            CmpGt {
                dst: 22,
                a: 20,
                b: 21,
            },
            Sub {
                dst: 23,
                a: 22,
                b: 9,
            },
            LoadConst {
                dst: 21,
                const_idx: 9,
            },
            CmpGt {
                dst: 23,
                a: 23,
                b: 21,
            },
            vote(VoteSink::Eat, 23),
            WriteActionParam {
                field_idx: ActionParamField::EatFoodType.index() as u8,
                src: 22,
            },
        ],
    }
}

/// Steps of the VM translation of the founder's vote node the controller
/// translations follow (`creature::founder::vm_decision_founder_genome`: one
/// straight-line pass).
pub const FOUNDER_VM_TRANSLATION_STEPS: u32 = 48;

/// The counted cost of a VM hand translation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VmConcern {
    pub instructions: u32,
    /// Summed step charges of the instructions as steps `1..=n` of one
    /// dispatch, and as steps after `after_steps` founder steps.
    pub charge_standalone: f64,
    pub after_steps: u32,
    pub charge_after_founder: f64,
    pub step_ramp_allowance: u32,
}

/// Count `family`'s VM hand translation against the runtime's step charges,
/// placed after `after_steps` steps of a founder translation.
#[must_use]
pub fn vm_concern(family: Family, config: &SimulationConfig, after_steps: u32) -> VmConcern {
    let program = vm_translation(family);
    let vm = &config.runtime.vm;
    let charge = |offset: usize| {
        program
            .iter()
            .enumerate()
            .map(|(i, instruction)| {
                step_charge(
                    opcode_base_cost(instruction),
                    vm.opcode_cost_multiplier,
                    offset + i + 1,
                    vm.step_ramp_allowance,
                    vm.step_ramp_cost,
                )
            })
            .sum::<f64>()
    };
    VmConcern {
        instructions: program.len() as u32,
        charge_standalone: charge(0),
        after_steps,
        charge_after_founder: charge(after_steps as usize),
        step_ramp_allowance: vm.step_ramp_allowance,
    }
}
