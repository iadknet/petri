//! Input-use funnel (T20.F01, `input-use-v1`): per cohort parent, how far
//! each input family and channel gets from declaration to connection,
//! executed read, causal action effect and one-step retention, over
//! T11.F26's cohorts, contexts and proposals.
//!
//! Stages, per parent and channel:
//! - `declared`: a reachable node's `input_refs` holds the family (for
//!   `UpstreamSlot`, the slot);
//! - `connected`: a consumer on the sensor census's live walk addresses it;
//! - `executed`: a read of it is resolved in some scene execution, recorded
//!   through the executor's own tracer hooks;
//! - `causal`: ablating it (every read returns 0.0, see
//!   [`consumers::ablated`]) changes the committed actions in some scene;
//! - retention: whether a parent-causal channel is still causal in each of the
//!   parent's first applied, non-identical T11.F26 proposals.
//!
//! Observation only: every function runs on clones after the last tick,
//! consumes no simulation RNG, and folds integer counts in a fixed order.

pub mod catalog;
mod consumers;
mod reads;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};

use rayon::prelude::*;

use crate::config::MutationConfig;
use crate::contracts::WorldAction;
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::CreatureGenome;
use crate::mutation::compound::sub_value_count;

use super::battery::{first_action_difference, run_panel, Scenario};
use super::mesh_execution::indices_for_node_ids;
use super::mutation_effects::contexts::{recorded_contexts, Extension};
use super::mutation_effects::{
    genome_identity, proposal_seed_base, Cohort, CohortParent, Proposals, Sizes, WorldInputs,
};
use super::{Battery, EvalContext};
use catalog::{addressed, shared_memory, Addressed, Channel, Declaration, Family};
pub(in crate::neighborhood) use consumers::ablated;
use consumers::{declarations, fixed_channel, Inventory, Target};
use reads::{ReadEvent, ReadRecording};

pub const VERSION: &str = "input-use-v1";
/// Children read for retention per parent with a causal channel.
pub const RETENTION_CHILDREN: u32 = 10;

/// The scenes every stage is read on: `neighborhood-v1` (the original
/// battery), then the T11.F26 extension (recorded and authored contexts and
/// the long sequences), in [`run_panel`] order.
struct Scenes<'a> {
    battery: &'a Battery,
    extension: &'a Extension,
    context: &'a EvalContext<'a>,
}

/// Where a genome's actions first differ from a baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Difference {
    None,
    Original,
    Extended,
}

impl Scenes<'_> {
    fn panels(&self) -> [(&[Scenario], &[Vec<Scenario>]); 2] {
        [
            (self.battery.snapshots(), self.battery.sequences()),
            (&self.extension.singles, &self.extension.sequences),
        ]
    }

    fn original_executions(&self) -> usize {
        self.battery.snapshots().len() + self.battery.sequence_lengths().sum::<usize>()
    }

    fn extended_executions(&self) -> usize {
        self.extension.singles.len() + self.extension.sequences.iter().map(Vec::len).sum::<usize>()
    }

    /// Every scene's actions, and every read site any scene resolved.
    fn observe(&self, genome: &CreatureGenome) -> (Vec<Vec<WorldAction>>, BTreeSet<ReadEvent>) {
        let mut events = BTreeSet::new();
        let mut actions = Vec::new();
        for (singles, sequences) in self.panels() {
            actions.extend(run_panel(
                genome,
                singles,
                sequences,
                self.context.runtime,
                self.context.shared_memory_decay_rate,
                ReadRecording::default,
                |(output, reads), _| {
                    events.extend(reads);
                    output.actions
                },
            ));
        }
        (actions, events)
    }

    /// Every scene's actions, untraced.
    fn actions(&self, genome: &CreatureGenome) -> Vec<Vec<WorldAction>> {
        self.panels()
            .into_iter()
            .flat_map(|(singles, sequences)| {
                run_panel(
                    genome,
                    singles,
                    sequences,
                    self.context.runtime,
                    self.context.shared_memory_decay_rate,
                    || crate::runtime::mesh::UntracedMeshExecution,
                    |output, _| output.actions,
                )
            })
            .collect()
    }

    /// The first part whose actions differ from `baseline`, running no
    /// execution past the first difference.
    fn difference(&self, genome: &CreatureGenome, baseline: &[Vec<WorldAction>]) -> Difference {
        let split = self.original_executions();
        let parts = [
            (Difference::Original, &baseline[..split]),
            (Difference::Extended, &baseline[split..]),
        ];
        for ((part, expected), (singles, sequences)) in parts.into_iter().zip(self.panels()) {
            let differs = first_action_difference(
                genome,
                singles,
                sequences,
                self.context.runtime,
                self.context.shared_memory_decay_rate,
                expected,
            );
            if differs.is_some() {
                return part;
            }
        }
        Difference::None
    }
}

/// One parent's stages. Shared-memory channels enter `connected` and
/// `executed` only.
#[derive(Debug, Clone, Default)]
struct ParentUse {
    declared: BTreeSet<Declaration>,
    /// Every channel of every declared family (for `UpstreamSlot`, the slot),
    /// so declaration-only rows exist.
    declared_channels: BTreeSet<Channel>,
    connected: BTreeSet<Channel>,
    executed: BTreeSet<Channel>,
    /// Channels executed through at least one structurally live consumer.
    executed_live: BTreeSet<Channel>,
    causal: BTreeSet<Channel>,
    causal_original: BTreeSet<Channel>,
    /// Static consumers on reachable nodes reading a decision compound at or
    /// past its width, by family.
    out_of_width: BTreeMap<Family, u32>,
    /// Per parent-causal channel: (pairs, still-causal pairs).
    retention: BTreeMap<Channel, (u32, u32)>,
    children_sampled: u32,
}

/// The channels a read of `inventory` event `event` resolved, with whether
/// the consumer was structurally live.
fn event_channels(inventory: &Inventory, event: ReadEvent) -> Vec<(Channel, bool)> {
    let fixed = |consumer: &consumers::Consumer| {
        fixed_channel(consumer.target).map(|channel| (channel, consumer.live))
    };
    match event {
        ReadEvent::Vm { node, pc, slot } => inventory.nodes[node]
            .vm_at(pc)
            .and_then(|consumer| match consumer.target {
                Target::DynamicSlot => {
                    slot.map(|slot| (shared_memory(usize::from(slot), false), consumer.live))
                }
                _ => fixed(&consumer),
            })
            .into_iter()
            .collect(),
        ReadEvent::Compute { node, index } => inventory.nodes[node]
            .compute
            .get(index)
            .into_iter()
            .flatten()
            .filter_map(fixed)
            .collect(),
        ReadEvent::Sinks { node } => inventory.nodes[node]
            .sinks
            .iter()
            .filter_map(fixed)
            .collect(),
    }
}

/// Every channel of a declared reference.
fn channels_of(reference: &crate::contracts::InputReference) -> impl Iterator<Item = Channel> + '_ {
    (0..sub_value_count(reference).max(1)).filter_map(move |sub_idx| {
        match addressed(reference, sub_idx) {
            Addressed::Channel(channel) => Some(channel),
            Addressed::OutOfWidth(_) => None,
        }
    })
}

/// Read one parent's stages; `proposals` regenerates its T11.F26 proposals.
fn evaluate_parent(
    genome: &CreatureGenome,
    proposals: impl FnOnce(&[usize]) -> Vec<CreatureGenome>,
    scenes: &Scenes<'_>,
) -> ParentUse {
    let reachable = mesh_reachable_nodes(genome);
    let inventory = Inventory::new(genome, &reachable);
    let mut use_ = ParentUse {
        declared: declarations(genome, &reachable),
        declared_channels: reachable
            .iter()
            .flat_map(|&index| genome.nodes[index].input_refs.iter().flat_map(channels_of))
            .collect(),
        connected: inventory.connected(),
        ..ParentUse::default()
    };
    for consumer in inventory.reachable() {
        if let Target::Input(Addressed::OutOfWidth(family)) = consumer.target {
            *use_.out_of_width.entry(family).or_default() += 1;
        }
    }

    let (baseline, events) = scenes.observe(genome);
    for event in events {
        for (channel, live) in event_channels(&inventory, event) {
            use_.executed.insert(channel);
            if live {
                use_.executed_live.insert(channel);
            }
        }
    }

    let candidates: Vec<Channel> = inventory.input_channels().into_iter().collect();
    let differences: Vec<Difference> = candidates
        .par_iter()
        .map(|&channel| scenes.difference(&ablated(genome, |c| c == channel), &baseline))
        .collect();
    for (&channel, difference) in candidates.iter().zip(differences) {
        if difference != Difference::None {
            use_.causal.insert(channel);
        }
        if difference == Difference::Original {
            use_.causal_original.insert(channel);
        }
    }

    if !use_.causal.is_empty() {
        let children = proposals(&reachable);
        use_.children_sampled = children.len() as u32;
        let causal: Vec<Channel> = use_.causal.iter().copied().collect();
        let retained: Vec<Vec<bool>> = children
            .par_iter()
            .map(|child| {
                let base = scenes.actions(child);
                causal
                    .iter()
                    .map(|&channel| {
                        scenes.difference(&ablated(child, |c| c == channel), &base)
                            != Difference::None
                    })
                    .collect()
            })
            .collect();
        for flags in retained {
            for (&channel, still) in causal.iter().zip(flags) {
                let entry = use_.retention.entry(channel).or_default();
                entry.0 += 1;
                entry.1 += u32::from(still);
            }
        }
    }
    use_
}

/// One cohort × family × channel row: parents at each stage. The causal and
/// retention counts are `None` for shared memory, which has no declaration or
/// causal stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub channel: Channel,
    pub declared: Option<u32>,
    pub connected: u32,
    pub executed: u32,
    pub causal: Option<u32>,
    pub causal_original: Option<u32>,
    pub executed_outside_live: u32,
    pub causal_outside_live: Option<u32>,
    pub retention_pairs: Option<u32>,
    pub retained_causal_pairs: Option<u32>,
}

/// One cohort × family row: parents with the stage on at least one of the
/// family's channels (a parent counts once however many qualify). `declared`
/// counts parents declaring the family on a reachable node; it and both causal
/// counts are `None` for shared memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FamilyRow {
    pub family: Family,
    pub declared: Option<u32>,
    pub connected: u32,
    pub executed: u32,
    pub causal: Option<u32>,
    pub causal_original: Option<u32>,
    /// Static consumers on reachable nodes reading this decision compound at
    /// or past its width (a constant 0.0, not a channel), summed over parents.
    pub out_of_width_consumers: u32,
}

/// One cohort's funnel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CohortUse {
    pub cohort: Cohort,
    pub parents_requested: u32,
    pub parents_evaluated: u32,
    /// (parent, channel) pairs causal without an executed read; expected 0,
    /// reported, not asserted.
    pub consistency_violations: u32,
    /// Parents with at least one causal channel, and the retention children
    /// requested of (`RETENTION_CHILDREN` each) and sampled for them.
    pub retention_parents: u32,
    pub retention_children_requested: u32,
    pub retention_children_sampled: u32,
    /// Rows with some parent at some stage, in family then channel order.
    pub rows: Vec<Row>,
    /// One row per family with a channel row, in family order.
    pub families: Vec<FamilyRow>,
}

fn fold_cohort(
    cohort: Cohort,
    parents_requested: u32,
    parents: &[ParentUse],
    retention_children: u32,
) -> CohortUse {
    let keys: BTreeSet<Channel> = parents
        .iter()
        .flat_map(|parent| {
            parent
                .declared_channels
                .iter()
                .chain(&parent.connected)
                .chain(&parent.executed)
                .chain(&parent.causal)
                .copied()
        })
        .collect();
    let count =
        |test: &dyn Fn(&ParentUse) -> bool| parents.iter().filter(|p| test(p)).count() as u32;
    let family_keys: BTreeSet<Family> = keys.iter().map(|channel| channel.family).collect();
    let rows = keys
        .into_iter()
        .map(|channel| {
            let shared = channel.family.is_shared_memory();
            let causal_only = |value: u32| (!shared).then_some(value);
            let pairs = |pick: fn(&(u32, u32)) -> u32| {
                parents
                    .iter()
                    .filter_map(|p| p.retention.get(&channel).map(pick))
                    .sum::<u32>()
            };
            Row {
                channel,
                declared: causal_only(count(&|p| {
                    p.declared
                        .iter()
                        .any(|declaration| declaration.covers(channel))
                })),
                connected: count(&|p| p.connected.contains(&channel)),
                executed: count(&|p| p.executed.contains(&channel)),
                causal: causal_only(count(&|p| p.causal.contains(&channel))),
                causal_original: causal_only(count(&|p| p.causal_original.contains(&channel))),
                executed_outside_live: count(&|p| {
                    p.executed.contains(&channel) && !p.executed_live.contains(&channel)
                }),
                causal_outside_live: causal_only(count(&|p| {
                    p.causal.contains(&channel) && !p.connected.contains(&channel)
                })),
                retention_pairs: causal_only(pairs(|entry| entry.0)),
                retained_causal_pairs: causal_only(pairs(|entry| entry.1)),
            }
        })
        .collect();
    let families = family_keys
        .into_iter()
        .map(|family| {
            let causal_only = |value: u32| (!family.is_shared_memory()).then_some(value);
            let any = |stage: fn(&ParentUse) -> &BTreeSet<Channel>| {
                count(&|p| stage(p).iter().any(|channel| channel.family == family))
            };
            FamilyRow {
                family,
                declared: causal_only(count(&|p| {
                    p.declared
                        .iter()
                        .any(|declaration| declaration.family == family)
                })),
                connected: any(|p| &p.connected),
                executed: any(|p| &p.executed),
                causal: causal_only(any(|p| &p.causal)),
                causal_original: causal_only(any(|p| &p.causal_original)),
                out_of_width_consumers: parents
                    .iter()
                    .filter_map(|p| p.out_of_width.get(&family))
                    .sum(),
            }
        })
        .collect();
    let retention_parents = count(&|p| !p.causal.is_empty());
    CohortUse {
        cohort,
        parents_requested,
        parents_evaluated: parents.len() as u32,
        consistency_violations: parents
            .iter()
            .map(|p| p.causal.difference(&p.executed).count() as u32)
            .sum(),
        retention_parents,
        retention_children_requested: retention_parents * retention_children,
        retention_children_sampled: parents.iter().map(|p| p.children_sampled).sum(),
        rows,
        families,
    }
}

/// The first `wanted` of `source`'s first `proposals` T11.F26 proposals
/// that apply and are not genome-identical, in proposal order.
fn retention_children(source: &Proposals<'_>, proposals: u32, wanted: u32) -> Vec<CreatureGenome> {
    let identity = genome_identity(source.genome);
    (0..proposals)
        .map(|proposal| source.propose(proposal))
        .filter(|(child, summary)| summary.applied_events > 0 && genome_identity(child) != identity)
        .map(|(child, _)| child)
        .take(wanted as usize)
        .collect()
}

fn observe_cohort(
    cohort: Cohort,
    parents: &[CohortParent],
    parents_requested: u32,
    scenes: &Scenes<'_>,
    one_event: &MutationConfig,
    sizes: Sizes,
    wanted: u32,
) -> CohortUse {
    let context = scenes.context;
    let uses: Vec<ParentUse> = parents
        .par_iter()
        .enumerate()
        .map(|(position, parent)| {
            let genome = &parent.genome;
            evaluate_parent(
                genome,
                |reachable| {
                    let sets = scenes.battery.mesh_execution_sets(
                        genome,
                        context.runtime,
                        context.shared_memory_decay_rate,
                    );
                    let executed = indices_for_node_ids(genome, &sets.executed);
                    let source = Proposals {
                        genome,
                        reachable,
                        executed: &executed,
                        seed_base: proposal_seed_base(cohort, position),
                        one_event,
                        food_type_count: context.food_type_count,
                    };
                    retention_children(&source, sizes.proposals, wanted)
                },
                scenes,
            )
        })
        .collect();
    fold_cohort(cohort, parents_requested, &uses, wanted)
}

/// One world's complete `input-use-v1` reading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    pub founder: CohortUse,
    pub drift: CohortUse,
    pub selected: Result<CohortUse, String>,
    pub original_executions: u32,
    pub extended_executions: u32,
    pub recorded_requested: u32,
    pub recorded_contexts: Result<u32, String>,
    pub sequence_source: &'static str,
    pub retention_children: u32,
}

/// Read one world's funnel over T11.F26's cohorts, contexts and proposals.
/// `retention_children` is [`RETENTION_CHILDREN`] in production.
#[must_use]
pub fn observe(
    inputs: WorldInputs<'_>,
    battery: &Battery,
    mutation: &MutationConfig,
    context: &EvalContext,
    sizes: Sizes,
    retention_children: u32,
) -> Reading {
    let one_event = MutationConfig {
        per_unit_rate: 1.0,
        ..mutation.clone()
    };
    let recorded = recorded_contexts(
        inputs.sim,
        inputs.world_seed,
        sizes.recorded_contexts as usize,
    );
    let recorded_contexts = if recorded.is_empty() {
        Err("extinct: no living creature at the terminal tick".to_string())
    } else {
        Ok(recorded.len() as u32)
    };
    let extension = Extension::new(recorded, battery);
    let scenes = Scenes {
        battery,
        extension: &extension,
        context,
    };
    let cohort = |cohort, parents: &[CohortParent], requested| {
        observe_cohort(
            cohort,
            parents,
            requested,
            &scenes,
            &one_event,
            sizes,
            retention_children,
        )
    };
    let founder = [CohortParent {
        index: 0,
        depth_or_generation: 0,
        genome: inputs.founder.clone(),
    }];
    Reading {
        founder: cohort(Cohort::Founder, &founder, 1),
        drift: cohort(Cohort::Drift, inputs.drift, inputs.drift.len() as u32),
        selected: inputs
            .selected
            .map(|parents| cohort(Cohort::Selected, parents, sizes.parents))
            .map_err(str::to_string),
        original_executions: scenes.original_executions() as u32,
        extended_executions: scenes.extended_executions() as u32,
        recorded_requested: sizes.recorded_contexts,
        recorded_contexts,
        sequence_source: extension.sequence_source(),
        retention_children,
    }
}
