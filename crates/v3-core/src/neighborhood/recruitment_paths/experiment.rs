use super::super::mesh_execution::{
    ancestral_payload_replacement, indices_for_node_ids, static_successor_bypass,
};
use super::super::recruitment::{
    BirthObservation, CohortFact, ModuleBackend, Opportunities, RecruitmentTracker,
};
use super::super::{Battery, Signature};
use super::*;
use crate::contracts::NodeId;
use crate::creature::genome::{analysis::mesh_reachable_nodes, CreatureGenome};
use crate::mutation::reachability::ParentExecuted;
use crate::mutation::{MutationDomain, MutationEngine, MutationSummary};
use rand::{rngs::SmallRng, RngCore, SeedableRng};
use serde::Serialize;
use std::collections::BTreeSet;

pub fn proposal_seed(batch: u32, lineage: u32, generation: u32, sibling: u8) -> u64 {
    13_020_000
        + u64::from(batch) * 1_000_000
        + u64::from(lineage) * 10_000
        + u64::from(generation) * 2
        + u64::from(sibling)
}

pub(super) fn fingerprint(value: &impl Serialize) -> String {
    super::super::recruitment::json_sha256(value)
}

pub(super) fn initial_tracker(start: &Start) -> RecruitmentTracker {
    let mut tracker = RecruitmentTracker::new(1);
    tracker.seed_founder(0, &start.creation_base.nodes);
    for (index, stage) in start.history.iter().enumerate() {
        let summary = if index == 0 && start.creation_operator.is_some() {
            let mut base = start.creation_base.clone();
            let summary = fixtures::topology(
                &mut base,
                crate::mutation::topology::TopologyOperator::CopyNode,
                7,
            );
            assert_eq!(
                base, stage.genome,
                "copy registration precedes dormant preparation"
            );
            summary
        } else {
            MutationSummary::zero()
        };
        tracker.record_birth(BirthObservation {
            lineage: 0,
            depth: 0,
            after: &stage.genome.nodes,
            summary: &summary,
        });
    }
    // Authored preparation is construction, not mutation: the form's
    // starting payloads are the birth payloads the ancestral test restores.
    tracker.rebase_birth_payloads(0, &start.genome.nodes);
    tracker
}

/// Every present cohort module's task use, with the bypass and (when the
/// bypass loses) ancestral counterfactuals.
pub(super) fn uses(
    genome: &CreatureGenome,
    reading: &TaskReading,
    task: Task,
    baseline: &TaskReading,
    tracker: &RecruitmentTracker,
    config: &crate::config::SimulationConfig,
) -> Vec<ModuleUse> {
    let dispatched = reading.dispatched();
    let live = reading.live();
    let score = reading.correct(task);
    let score_gain = score > baseline.correct(task);
    let incumbents_preserved = reading.preserves_correct_scenes(baseline, task);
    let current_ending_energy_sum = reading.summary().ending_energy_sum;
    tracker
        .modules()
        .filter(|module| module.is_present() && module.provenance.is_cohort())
        .map(|module| {
            let node = genome
                .nodes
                .iter()
                .find(|node| node.node_id == module.node)
                .expect("present tracker node");
            let birth_payload = tracker
                .birth_payload(module)
                .expect("present tracker module keeps its birth payload");
            let dispatched = dispatched.contains(&module.node);
            let mut result = ModuleUse {
                node: module.node,
                created_depth: module.created_depth,
                created_backend: module.backend,
                current_backend: ModuleBackend::from(&node.backend_def),
                dispatched,
                score_loss: 0,
                queue_effect: false,
                memory_effect: None,
                output_effect: false,
                routing_effect: false,
                payload_changed: node.backend_def != *birth_payload,
                ancestral_loss: None,
                current_ending_energy_sum,
                ancestral_ending_energy_sum: None,
                destination_kind: DestinationKind::of(&node.backend_def),
                specialization: Specialization {
                    task_live: live,
                    score_gain,
                    bypass_loss: false,
                    ancestral_loss: false,
                    incumbents_preserved,
                },
            };
            if dispatched {
                let bypass =
                    evaluate_with_config(&static_successor_bypass(genome, module.node), config);
                result.score_loss = i16::from(score) - i16::from(bypass.correct(task));
                result.memory_effect = reading.memory_effect(&bypass);
                for (before, after) in reading.scenes.iter().zip(&bypass.scenes) {
                    result.queue_effect |= before.actions != after.actions;
                    result.output_effect |= before
                        .output_slots
                        .iter()
                        .filter(|(id, _)| *id != module.node)
                        .ne(after.output_slots.iter());
                    result.routing_effect |= before.routing != after.routing;
                }
                result.specialization.bypass_loss = result.score_loss >= 1;
                if result.specialization.bypass_loss {
                    // A verbatim payload replaced by itself is the identity:
                    // it reads zero without a second evaluation.
                    let (loss, energy) = if result.payload_changed {
                        let replaced = evaluate_with_config(
                            &ancestral_payload_replacement(genome, module.node, birth_payload),
                            config,
                        );
                        (
                            i16::from(score) - i16::from(replaced.correct(task)),
                            replaced.summary().ending_energy_sum,
                        )
                    } else {
                        (0, current_ending_energy_sum)
                    };
                    result.ancestral_loss = Some(loss);
                    result.ancestral_ending_energy_sum = Some(energy);
                    result.specialization.ancestral_loss = loss >= 1;
                }
            }
            result
        })
        .collect()
}

fn checkpoint(
    generation: u32,
    genome: &CreatureGenome,
    reading: &TaskReading,
    start: &Start,
    tracker: &mut RecruitmentTracker,
    battery_context: (&Battery, &Signature),
    config: &crate::config::SimulationConfig,
) -> Checkpoint {
    let (battery, baseline) = battery_context;
    let signature = battery.signature(genome, &config.runtime, config.shared_memory.decay_rate);
    let sets =
        battery.mesh_execution_sets(genome, &config.runtime, config.shared_memory.decay_rate);
    tracker.record_reading(
        0,
        u64::from(generation),
        &sets.executed,
        Some(&sets.contributing),
    );
    let cohort = tracker.checkpoint(u64::from(generation));
    let task_use = uses(
        genome,
        reading,
        start.task,
        &start.task_reading,
        tracker,
        config,
    );
    let mut destination_kinds = DestinationKindCounts::default();
    for module in &task_use {
        destination_kinds.record(module.destination_kind);
    }
    let row = cohort.lineage_rows[0];
    Checkpoint {
        generation,
        genome: genome.clone(),
        task: reading.clone(),
        battery_class: format!("{:?}", super::super::classify(baseline, &signature).class),
        battery: signature,
        modules: tracker
            .modules()
            .map(|module| ModuleRecord {
                birth_payload: module
                    .first(CohortFact::Dispatch)
                    .and_then(|_| tracker.birth_payload(module).cloned()),
                module: module.clone(),
            })
            .collect(),
        task_use,
        route_position_varies: reading.route_position_varies(),
        route_destination_varies: reading.route_destination_varies(),
        destination_kinds,
        eligible_site_fraction: estimate(row.applicable as u32, row.created as u32),
        cohort,
    }
}

struct Sibling {
    genome: CreatureGenome,
    task: TaskReading,
    tracker: RecruitmentTracker,
    record: Proposal,
    delta: GenomeDelta,
}

struct ProposalPosition {
    batch: u32,
    lineage: u32,
    generation: u32,
}

/// Node ids of the genome's statically reachable nodes.
fn reachable_ids(genome: &CreatureGenome, reachable: &[usize]) -> BTreeSet<NodeId> {
    reachable
        .iter()
        .map(|&index| genome.nodes[index].node_id)
        .collect()
}

/// The mutation config the legacy panel's proposals run: the default config
/// on the legacy per-birth supply rule, the fixed-count control the recorded
/// baselines were taken on, like the drift walk (T11.F19).
#[cfg(test)]
pub(super) fn proposal_mutation_config() -> crate::config::MutationConfig {
    Supply::Legacy.mutation_config()
}

/// One arm's context: its start, policy, and the panel it runs under.
struct ArmContext<'a> {
    start: &'a Start,
    policy: Policy,
    panel: Panel,
    mutation: &'a crate::config::MutationConfig,
    battery: &'a Battery,
    config: &'a crate::config::SimulationConfig,
}

fn propose_siblings(
    arm: &ArmContext<'_>,
    genome: &CreatureGenome,
    task: &TaskReading,
    tracker: &RecruitmentTracker,
    position: ProposalPosition,
    viable_path: bool,
) -> Vec<Sibling> {
    let ProposalPosition {
        batch,
        lineage,
        generation,
    } = position;
    let start = arm.start;
    let config = arm.config;
    let starting_score = start.task_reading.correct(start.task);
    let reachable = mesh_reachable_nodes(genome);
    let executed = indices_for_node_ids(genome, &task.dispatched());
    let parent_fingerprint = fingerprint(genome);
    let parent_score = task.correct(start.task);
    let parent_live = task.live();
    let mut children = Vec::with_capacity(2);
    for sibling in 0..2 {
        let seed = proposal_seed(batch, lineage, generation - 1, sibling);
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut child = genome.clone();
        let summary = MutationEngine::apply_mutations_with_food_type_count(
            &mut child,
            arm.mutation,
            &reachable,
            ParentExecuted::Indices(&executed),
            &mut rng,
            config.world.food.types.len(),
        );
        // Inspect a clone, never advance the proposal's mutation stream.
        let rng_after = rng.clone().next_u64();
        let mut child_tracker = tracker.clone();
        child_tracker.record_birth(BirthObservation {
            lineage: 0,
            depth: u64::from(generation),
            after: &child.nodes,
            summary: &summary,
        });
        let child_task = evaluate_with_config(&child, config);
        let module_uses = uses(
            &child,
            &child_task,
            start.task,
            &start.task_reading,
            &child_tracker,
            config,
        );
        let specialized = module_uses
            .iter()
            .any(|module| module.specialization.holds());
        let useful_modules: Vec<_> = module_uses
            .into_iter()
            .filter(|module| child_task.live() && module.dispatched && module.score_loss >= 1)
            .collect();
        let discovery = child_task.live()
            && child_task.correct(start.task) > starting_score
            && !useful_modules.is_empty();
        let delta = GenomeDelta::between(genome, &child);
        let events: Vec<_> = summary.events.iter().map(Event::from).collect();
        let mut opportunities = Opportunities::default();
        opportunities.record(&summary);
        let mut by_backend =
            std::collections::BTreeMap::<_, std::collections::BTreeMap<_, u64>>::new();
        let mut unresolved = 0;
        for event in &summary.events {
            for &(operator, target) in &event.discarded {
                let Some(target) = target else {
                    continue;
                };
                let backend = match operator.domain() {
                    MutationDomain::Graph => Some(ModuleBackend::Graph),
                    MutationDomain::Vm => Some(ModuleBackend::Vm),
                    _ => genome
                        .nodes
                        .iter()
                        .chain(&child.nodes)
                        .find(|node| node.node_id == target)
                        .map(|node| ModuleBackend::from(&node.backend_def)),
                };
                if let Some(backend) = backend {
                    *by_backend
                        .entry(backend)
                        .or_default()
                        .entry(operator)
                        .or_default() += 1;
                } else {
                    unresolved += 1;
                }
            }
        }
        let record = Proposal {
            generation,
            sibling,
            seed,
            parent_live,
            parent_score,
            chosen: false,
            outcome: child_task.summary(),
            modules: child.nodes.len(),
            genome_size: child.genome_size(),
            opportunities,
            mutation_fingerprint: fingerprint(&(&delta, &events, rng_after)),
            parent_fingerprint: parent_fingerprint.clone(),
            rng_after,
            selected_inapplicable_by_backend_operator: by_backend,
            selected_inapplicable_backend_unresolved: unresolved,
            events,
            useful_modules,
            discovery,
            specialized,
            viable_path: viable_path
                && parent_live
                && child_task.live()
                && child_task.correct(start.task) + 1 >= parent_score,
        };
        children.push(Sibling {
            genome: child,
            task: child_task,
            tracker: child_tracker,
            record,
            delta,
        });
    }
    children
}

/// The specialized recruit's current reading on the retained chain.
fn recruit_reading(
    genome: &CreatureGenome,
    task: &TaskReading,
    start: &Start,
    tracker: &RecruitmentTracker,
    discovery: &Discovery,
    config: &crate::config::SimulationConfig,
) -> Option<ModuleUse> {
    uses(
        genome,
        task,
        start.task,
        &start.task_reading,
        tracker,
        config,
    )
    .into_iter()
    .find(|module| {
        module.node == discovery.module.node
            && module.created_depth == discovery.module.created_depth
    })
}

/// One lineage and, in generation order, the whole-birth delta of every
/// chosen child (the compact record keeps these; the legacy report does not).
/// The first retained F06 discovery and the first retained T13.F07
/// specialized recruit, within the discovery horizon.
fn note_retained_discoveries(result: &mut Lineage, record: &Proposal, viable_path: bool) {
    if result.retained_discovery.is_none() && record.discovery {
        result.retained_discovery = Some(Discovery {
            generation: record.generation,
            sibling: record.sibling,
            module: record.useful_modules[0].clone(),
        });
        result.viable_retained_discovery = viable_path;
    }
    if result.specialized_discovery.is_none() && record.specialized {
        let module = record
            .useful_modules
            .iter()
            .find(|module| module.specialization.holds())
            .expect("a specialized proposal carries a specialized useful module")
            .clone();
        result.specialized_discovery = Some(Discovery {
            generation: record.generation,
            sibling: record.sibling,
            module,
        });
        result.ladder.specialized = true;
    }
}

/// The retained chain at one generation.
struct Chain<'a> {
    generation: u32,
    genome: &'a CreatureGenome,
    task: &'a TaskReading,
    tracker: &'a RecruitmentTracker,
}

/// The T13.F06 retention reading at retained discovery + follow-up.
fn legacy_retention(result: &mut Lineage, arm: &ArmContext<'_>, chain: &Chain<'_>) {
    let Some(discovery) = &result.retained_discovery else {
        return;
    };
    if chain.generation != discovery.generation + arm.panel.sizes.followup {
        return;
    }
    let task = arm.start.task;
    let present = chain.tracker.modules().any(|module| {
        module.node == discovery.module.node
            && module.created_depth == discovery.module.created_depth
            && module.is_present()
    });
    let score_loss = present.then(|| {
        let bypass = evaluate_with_config(
            &static_successor_bypass(chain.genome, discovery.module.node),
            arm.config,
        );
        i16::from(chain.task.correct(task)) - i16::from(bypass.correct(task))
    });
    result.retention = Some(Retention {
        discovery: discovery.clone(),
        at_generation: chain.generation,
        outcome: classify_retention(chain.task.live(), score_loss),
        score: chain.task.correct(task),
        score_loss,
    });
}

/// The T13.F07 discovery checkpoint and the recruit's readings at each
/// [`HORIZONS`] offset after the specialized retained discovery.
fn specialized_horizons(
    result: &mut Lineage,
    arm: &ArmContext<'_>,
    chain: &Chain<'_>,
    baseline: &Signature,
) {
    let Some(discovery) = result.specialized_discovery.clone() else {
        return;
    };
    if chain.generation == discovery.generation {
        // Read a clone: the panel's fixed checkpoints keep their cohort
        // retention baselines.
        result.discovery_checkpoint = Some(checkpoint(
            chain.generation,
            chain.genome,
            chain.task,
            arm.start,
            &mut chain.tracker.clone(),
            (arm.battery, baseline),
            arm.config,
        ));
    }
    for offset in HORIZONS {
        if chain.generation != discovery.generation + offset {
            continue;
        }
        let module = recruit_reading(
            chain.genome,
            chain.task,
            arm.start,
            chain.tracker,
            &discovery,
            arm.config,
        );
        let outcome = HorizonOutcome::of(chain.task.live(), module.as_ref());
        if offset == PRIMARY_HORIZON {
            result.ladder.at_primary_horizon = Some(outcome);
        }
        result.horizons.push(Horizon {
            offset,
            at_generation: chain.generation,
            outcome,
            live: chain.task.live(),
            score: chain.task.correct(arm.start.task),
            module,
        });
    }
}

fn lineage(arm: &ArmContext<'_>, batch: u32, lineage: u32) -> (Lineage, Vec<GenomeDelta>) {
    let ArmContext {
        start,
        policy,
        panel,
        battery,
        config,
        ..
    } = *arm;
    let sizes = panel.sizes;
    let mut genome = start.genome.clone();
    let mut task = start.task_reading.clone();
    let mut tracker = initial_tracker(start);
    tracker.record_scene_dispatch(0, &task.scenes_dispatched());
    let baseline = battery.signature(&genome, &config.runtime, config.shared_memory.decay_rate);
    let mut result = Lineage {
        batch,
        lineage,
        task: start.task,
        proposals: Vec::new(),
        checkpoints: vec![checkpoint(
            0,
            &genome,
            &task,
            start,
            &mut tracker,
            (battery, &baseline),
            config,
        )],
        proposal_discovery: None,
        retained_discovery: None,
        viable_retained_discovery: false,
        retention: None,
        first_successful_path: Vec::new(),
        specialized_discovery: None,
        discovery_checkpoint: None,
        horizons: Vec::new(),
        ladder: Ladder::default(),
        classification: LineageClass::NoEligibility,
    };
    let mut reachable_cohort = reachable_cohort(&genome, &tracker);
    result.ladder.eligibility = !reachable_cohort.is_empty();
    result.ladder.expression = expressed(&tracker);
    let mut viable_path = task.live();
    let mut history = Vec::new();
    let mut chosen_deltas = Vec::new();
    for generation in 1..=sizes.discovery + sizes.followup {
        let mut children = propose_siblings(
            arm,
            &genome,
            &task,
            &tracker,
            ProposalPosition {
                batch,
                lineage,
                generation,
            },
            viable_path,
        );
        let winner = choose(
            policy,
            Candidate::new(&task, start.task),
            [
                Candidate::new(&children[0].task, start.task),
                Candidate::new(&children[1].task, start.task),
            ],
        );
        if let Some(index) = winner {
            children[index].record.chosen = true;
        }
        if generation <= sizes.discovery {
            note_proposal_discovery(&mut result, &children, &history);
        }
        for child in &children {
            result.proposals.push(child.record.clone());
        }
        if let Some(index) = winner {
            let child = children.swap_remove(index);
            chosen_deltas.push(child.delta.clone());
            viable_path = child.record.viable_path;
            result.ladder.local_edit |= child.record.events.iter().any(|event| {
                event.outcome.starts_with("Applied")
                    && event
                        .target
                        .is_some_and(|target| reachable_cohort.contains(&target))
            });
            result.ladder.bypass_only |= child
                .record
                .useful_modules
                .iter()
                .any(|module| module.ancestral_loss == Some(0));
            if generation <= sizes.discovery {
                note_retained_discoveries(&mut result, &child.record, viable_path);
            }
            if result.proposal_discovery.is_none() {
                history.push(ReplayStep {
                    generation,
                    sibling: Some(child.record.sibling),
                    seed: Some(child.record.seed),
                    retained: true,
                    events: child.record.events,
                    delta: child.delta,
                    outcome: child.task.clone(),
                });
            }
            genome = child.genome;
            task = child.task;
            tracker = child.tracker;
        } else if result.proposal_discovery.is_none() {
            history.push(ReplayStep {
                generation,
                sibling: None,
                seed: None,
                retained: true,
                events: vec![],
                delta: GenomeDelta::between(&genome, &genome),
                outcome: task.clone(),
            });
        }
        tracker.record_reading(0, u64::from(generation), &task.dispatched(), None);
        tracker.record_scene_dispatch(0, &task.scenes_dispatched());
        reachable_cohort = self::reachable_cohort(&genome, &tracker);
        result.ladder.eligibility |= !reachable_cohort.is_empty();
        result.ladder.expression |= expressed(&tracker);
        let chain = Chain {
            generation,
            genome: &genome,
            task: &task,
            tracker: &tracker,
        };
        legacy_retention(&mut result, arm, &chain);
        specialized_horizons(&mut result, arm, &chain, &baseline);
        if generation == sizes.discovery || generation == sizes.discovery + sizes.followup {
            result.checkpoints.push(checkpoint(
                generation,
                &genome,
                &task,
                start,
                &mut tracker,
                (battery, &baseline),
                config,
            ));
        }
    }
    result.classification = LineageClass::of(result.ladder);
    expose_identity(&mut result, batch * sizes.lineages + lineage);
    (result, chosen_deltas)
}

/// Present cohort modules statically reachable from the entry node.
fn reachable_cohort(genome: &CreatureGenome, tracker: &RecruitmentTracker) -> BTreeSet<NodeId> {
    let reachable = reachable_ids(genome, &mesh_reachable_nodes(genome));
    tracker
        .modules()
        .filter(|module| module.is_present() && module.provenance.is_cohort())
        .map(|module| module.node)
        .filter(|node| reachable.contains(node))
        .collect()
}

/// Some cohort module dispatched in at least one scene of the latest reading.
fn expressed(tracker: &RecruitmentTracker) -> bool {
    tracker
        .modules()
        .any(|module| module.provenance.is_cohort() && module.scenes_dispatched >= 1)
}

/// The first F06 discovery among either sibling, with the replay path that
/// reached it, and the proposal-level specialization flag.
fn note_proposal_discovery(result: &mut Lineage, children: &[Sibling], history: &[ReplayStep]) {
    if result.proposal_discovery.is_none() {
        if let Some(child) = children.iter().find(|child| child.record.discovery) {
            result.proposal_discovery = Some(Discovery {
                generation: child.record.generation,
                sibling: child.record.sibling,
                module: child.record.useful_modules[0].clone(),
            });
            result.first_successful_path = history.to_vec();
            result.first_successful_path.push(ReplayStep {
                generation: child.record.generation,
                sibling: Some(child.record.sibling),
                seed: Some(child.record.seed),
                retained: child.record.chosen,
                events: child.record.events.clone(),
                delta: child.delta.clone(),
                outcome: child.task.clone(),
            });
        }
    }
    result.ladder.proposal_specialized |= children.iter().any(|child| child.record.specialized);
}

/// Each tracker uses local index zero; expose the actual batch/lineage
/// identity on its F01 module/cohort records at the report boundary.
fn expose_identity(result: &mut Lineage, identity: u32) {
    for checkpoint in result
        .checkpoints
        .iter_mut()
        .chain(result.discovery_checkpoint.iter_mut())
    {
        for module in &mut checkpoint.modules {
            module.module.lineage = identity;
        }
        for row in &mut checkpoint.cohort.lineage_rows {
            row.lineage = identity;
        }
    }
}

/// Retained-parent cost at each checkpoint generation, spread over lineages.
/// Every lineage checkpoints at the same generations.
fn checkpoint_cost(lineages: &[LineageFacts]) -> Vec<CheckpointCost> {
    let Some(first) = lineages.first() else {
        return Vec::new();
    };
    first
        .checkpoints
        .iter()
        .enumerate()
        .filter_map(|(index, reference)| {
            let at: Vec<_> = lineages
                .iter()
                .map(|lineage| {
                    let checkpoint = &lineage.checkpoints[index];
                    assert_eq!(checkpoint.generation, reference.generation);
                    checkpoint
                })
                .collect();
            let spread = |value: fn(&CheckpointScalars) -> f64| {
                Spread::of(at.iter().map(|checkpoint| value(checkpoint)).collect())
            };
            Some(CheckpointCost {
                generation: reference.generation,
                genome_size: spread(|checkpoint| checkpoint.genome_size)?,
                modules: spread(|checkpoint| checkpoint.modules)?,
                carrying_sum: spread(|checkpoint| checkpoint.carrying_sum)?,
                ending_energy_sum: spread(|checkpoint| checkpoint.ending_energy_sum)?,
            })
        })
        .collect()
}

fn transitions(lineages: &[LineageFacts]) -> Transitions {
    let mut out = Transitions {
        lineages: lineages.len() as u32,
        ..Transitions::default()
    };
    let mut applicable = (0u64, 0u64);
    for lineage in lineages {
        let ladder = lineage.ladder;
        out.eligibility += u32::from(ladder.eligibility);
        out.local_edit += u32::from(ladder.local_edit);
        out.expression += u32::from(ladder.expression);
        out.specialized += u32::from(ladder.specialized);
        out.proposal_specialized_lineages += u32::from(ladder.proposal_specialized);
        out.specialized_proposals += lineage.specialized_proposals;
        out.bypass_only += u32::from(ladder.bypass_only);
        for &(offset, outcome) in &lineage.horizons {
            *out.retained_at.entry(offset).or_default() +=
                u32::from(outcome == HorizonOutcome::Retained);
        }
        *out.classes
            .entry(class_key(lineage.classification))
            .or_default() += 1;
        applicable.0 += lineage.final_applicable.0;
        applicable.1 += lineage.final_applicable.1;
        out.destination_kinds.merge(lineage.destination_kinds);
    }
    out.eligible_site_fraction = estimate(applicable.0 as u32, applicable.1 as u32);
    out
}

/// One arm's summary over its lineages' facts.
#[must_use]
pub fn summary(lineages: &[LineageFacts]) -> Summary {
    let count = lineages.len() as u32;
    let discoveries: Vec<_> = lineages
        .iter()
        .filter_map(|lineage| lineage.retained_discovery)
        .collect();
    let mut retention_outcomes = RetentionOutcomes::default();
    for outcome in lineages.iter().filter_map(|lineage| lineage.retention) {
        retention_outcomes.record(outcome);
    }
    let useful = retention_outcomes.useful;
    let (proposals, task_dead, task_live_loss) =
        lineages.iter().flat_map(|lineage| &lineage.proposals).fold(
            (0, 0, 0),
            |(total, dead, loss), &(live, score, parent_score)| {
                (
                    total + 1,
                    dead + u32::from(!live),
                    loss + u32::from(live && score < parent_score),
                )
            },
        );
    let damage = Damage {
        task_dead: estimate(task_dead, proposals),
        task_live_loss: estimate(task_live_loss, proposals - task_dead),
    };
    Summary {
        time_to_first_retained: TimeToFirst::of(
            lineages.iter().map(|lineage| lineage.retained_discovery),
        ),
        time_to_first_proposal: TimeToFirst::of(
            lineages.iter().map(|lineage| lineage.proposal_discovery),
        ),
        retention_outcomes,
        damage,
        checkpoint_cost: checkpoint_cost(lineages),
        proposal_discovery: estimate(
            lineages
                .iter()
                .filter(|lineage| lineage.proposal_discovery.is_some())
                .count() as u32,
            count,
        ),
        retained_discovery: estimate(discoveries.len() as u32, count),
        viable_retained_discovery: estimate(
            lineages
                .iter()
                .filter(|lineage| lineage.viable_retained_discovery)
                .count() as u32,
            count,
        ),
        retained_useful: estimate(useful, count),
        retention_among_discoverers: estimate(useful, discoveries.len() as u32),
        discovery_depth_range: discoveries
            .iter()
            .min()
            .zip(discoveries.iter().max())
            .map(|(&min, &max)| [min, max]),
        transitions: transitions(lineages),
    }
}

/// An arm summary and its per-batch summaries.
#[must_use]
pub fn summaries(facts: &[LineageFacts], batches: u32) -> (Summary, Vec<Summary>) {
    let arm = summary(facts);
    let per_batch = (0..batches)
        .map(|batch| {
            let of_batch: Vec<_> = facts
                .iter()
                .filter(|lineage| lineage.batch == batch)
                .cloned()
                .collect();
            summary(&of_batch)
        })
        .collect();
    (arm, per_batch)
}

fn pair(left: &Arm, right: &Arm, left_arm: usize, right_arm: usize) -> Pair {
    let lineages = left
        .lineages
        .iter()
        .zip(&right.lineages)
        .map(|(left, right)| {
            let proposals: Vec<_> = left.proposals.iter().zip(&right.proposals).collect();
            let first = proposals
                .iter()
                .position(|(left, right)| left.mutation_fingerprint != right.mutation_fingerprint);
            let useful = |lineage: &Lineage| {
                lineage
                    .retention
                    .as_ref()
                    .is_some_and(|retention| retention.outcome == RetentionOutcome::Useful)
            };
            PairedLineage {
                batch: left.batch,
                lineage: left.lineage,
                proposal_discovery_difference: i8::from(right.proposal_discovery.is_some())
                    - i8::from(left.proposal_discovery.is_some()),
                retained_discovery_difference: i8::from(right.retained_discovery.is_some())
                    - i8::from(left.retained_discovery.is_some()),
                useful_retention_difference: i8::from(useful(right)) - i8::from(useful(left)),
                first_parent_divergence: proposals
                    .iter()
                    .find(|(left, right)| left.parent_fingerprint != right.parent_fingerprint)
                    .map(|(left, _)| left.generation - 1),
                first_mutation_divergence: first
                    .map(|index| (proposals[index].0.generation, proposals[index].0.sibling)),
                first_rng_divergence: proposals
                    .iter()
                    .find(|(left, right)| left.rng_after != right.rng_after)
                    .map(|(left, _)| (left.generation, left.sibling)),
                matched_proposals_before_divergence: first.unwrap_or(proposals.len()) as u32,
            }
        })
        .collect();
    Pair {
        left_arm,
        right_arm,
        lineages,
    }
}

/// The fingerprint comparison of one replayed lineage.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct ReplayCheck {
    pub proposals: u64,
    pub matched: u64,
    /// The first `(generation, sibling)` whose fingerprint differed.
    pub first_mismatch: Option<(u32, u8)>,
}

impl ReplayCheck {
    pub fn merge(&mut self, other: Self) {
        self.proposals += other.proposals;
        self.matched += other.matched;
        if self.first_mismatch.is_none() {
            self.first_mismatch = other.first_mismatch;
        }
    }
}

/// The fixed arms of one panel: the nine starting forms under Drift,
/// Selection and CostSelection, in the order the legacy report records them
/// (`arms/0` is `graph_blank` under Drift; the nine cost arms follow).
pub struct Assay {
    panel: Panel,
    mutation: crate::config::MutationConfig,
    config: crate::config::SimulationConfig,
    battery: Battery,
    constructed: Vec<ConstructedPath>,
    starts: Vec<Start>,
    arms: Vec<(usize, Policy)>,
}

impl Assay {
    #[must_use]
    pub fn new(panel: Panel) -> Self {
        let config = task_config();
        let battery = Battery::generate(config.world.food.types.len());
        let constructed = constructed_paths();
        let starts = fixtures::starts_from_paths(&constructed);
        let f02 = (0..starts.len()).flat_map(|start| Policy::F02.map(|policy| (start, policy)));
        let cost = (0..starts.len()).map(|start| (start, Policy::CostSelection));
        let arms: Vec<_> = f02.chain(cost).collect();
        debug_assert_eq!(arms.len(), ARMS);
        Self {
            panel,
            mutation: panel.supply.mutation_config(),
            config,
            battery,
            constructed,
            starts,
            arms,
        }
    }

    #[must_use]
    pub const fn panel(&self) -> Panel {
        self.panel
    }

    #[must_use]
    pub fn starts(&self) -> &[Start] {
        &self.starts
    }

    #[must_use]
    pub fn arm_count(&self) -> usize {
        self.arms.len()
    }

    /// The start and policy of arm `arm`.
    #[must_use]
    pub fn arm(&self, arm: usize) -> (&Start, Policy) {
        let (start, policy) = self.arms[arm];
        (&self.starts[start], policy)
    }

    fn context(&self, arm: usize) -> ArmContext<'_> {
        let (start, policy) = self.arm(arm);
        ArmContext {
            start,
            policy,
            panel: self.panel,
            mutation: &self.mutation,
            battery: &self.battery,
            config: &self.config,
        }
    }

    /// Run one lineage of one arm; deterministic in `(arm, batch, lineage)`.
    #[must_use]
    pub fn lineage(&self, arm: usize, batch: u32, lineage: u32) -> Lineage {
        self::lineage(&self.context(arm), batch, lineage).0
    }

    /// Run one lineage and keep its compact record: the full record's
    /// proposal rows reduced to seeds, choices and outcomes, plus the chosen
    /// children's deltas.
    #[must_use]
    pub fn compact_lineage(&self, arm: usize, batch: u32, lineage: u32) -> CompactLineage {
        let (start, policy) = self.arm(arm);
        let (full, deltas) = self::lineage(&self.context(arm), batch, lineage);
        let mut compact = CompactLineage::of(arm, &start.name, policy, &full);
        let mut deltas = deltas.into_iter();
        for proposal in compact.proposals.iter_mut().filter(|p| p.chosen) {
            proposal.delta = deltas.next();
        }
        debug_assert!(deltas.next().is_none());
        compact
    }

    /// Re-run one seed's mutation on `parent` into `child`, returning the
    /// post-call RNG draw and the summary.
    fn mutate(
        &self,
        parent: &CreatureGenome,
        seed: u64,
        child: &mut CreatureGenome,
    ) -> (u64, MutationSummary) {
        let task = evaluate_with_config(parent, &self.config);
        let reachable = mesh_reachable_nodes(parent);
        let executed = indices_for_node_ids(parent, &task.dispatched());
        let mut rng = SmallRng::seed_from_u64(seed);
        let summary = MutationEngine::apply_mutations_with_food_type_count(
            child,
            &self.mutation,
            &reachable,
            ParentExecuted::Indices(&executed),
            &mut rng,
            self.config.world.food.types.len(),
        );
        (rng.clone().next_u64(), summary)
    }

    /// Reconstruct every proposal of a compact lineage from the initial
    /// genome, the chosen chain and the seeds, and compare fingerprints.
    #[must_use]
    pub fn replay(&self, record: &CompactLineage) -> ReplayCheck {
        let (start, _) = self.arm(record.arm);
        let mut check = ReplayCheck::default();
        let mut parent = start.genome.clone();
        for pair in record.proposals.chunks_exact(2) {
            let mut retained = None;
            for proposal in pair {
                let mut child = parent.clone();
                let (rng_after, summary) = self.mutate(&parent, proposal.seed, &mut child);
                let delta = GenomeDelta::between(&parent, &child);
                let events: Vec<_> = summary.events.iter().map(Event::from).collect();
                let replayed = fingerprint(&(&delta, &events, rng_after));
                check.proposals += 1;
                if replayed == proposal.mutation_fingerprint {
                    check.matched += 1;
                } else if check.first_mismatch.is_none() {
                    check.first_mismatch = Some((proposal.generation, proposal.sibling));
                }
                if proposal.chosen {
                    retained = Some(child);
                }
            }
            if let Some(child) = retained {
                parent = child;
            }
        }
        check
    }
}

/// One report-level observation of the legacy panel; it has no ecological
/// world input or RNG.
#[must_use]
pub fn observe(sizes: Sizes) -> Report {
    let assay = Assay::new(Panel::legacy(sizes));
    let config = &assay.config;
    let mut arms = Vec::with_capacity(ARMS);
    let mut opportunities = Opportunities::default();
    for arm in 0..assay.arm_count() {
        let (start, policy) = assay.arm(arm);
        let mut lineages = Vec::new();
        for batch in 0..sizes.batches {
            for index in 0..sizes.lineages {
                lineages.push(assay.lineage(arm, batch, index));
            }
        }
        let facts: Vec<_> = lineages.iter().map(LineageFacts::of).collect();
        let (summary, batches) = summaries(&facts, sizes.batches);
        let mut supply = Opportunities::default();
        for lineage in &lineages {
            for proposal in &lineage.proposals {
                supply.merge(&proposal.opportunities);
            }
        }
        opportunities.merge(&supply);
        arms.push(Arm {
            start: start.name.clone(),
            task: start.task,
            policy,
            lineages,
            summary,
            batches,
            opportunities: supply,
        });
    }
    // Every treatment comparison uses the same original lineage pairs. These
    // rows expose dependent contrasts, never new independent replicates.
    let mut pairs = Vec::new();
    for left in 0..arms.len() {
        for right in left + 1..arms.len() {
            if arms[left].task == arms[right].task {
                pairs.push(pair(&arms[left], &arms[right], left, right));
            }
        }
    }
    Report { version: VERSION.into(), config_digest: crate::config::config_digest(config), config: config.clone(), sizes,
        supply: Supply::Legacy, supply_rule: Supply::Legacy.rule().into(),
        task_definition: "Eight fresh 12x12 one-tick scenes; energy 50; zero learned state; here/east/north food 0/1. A: FoodHere(0)>0; B: NeighborFood(E,0)>0. Exact [Move(E)] plus east displacement or [NoOp] plus no displacement. Score=correct/8; practical margin=1/8.".into(),
        mutation_context: "Default MutationConfig on the legacy per-birth supply rule (per_unit_supply_enabled forced false); SmallRng seed=13020000+batch*1000000+lineage*10000+generation_zero_based*2+sibling. Reachability and ParentExecuted indices from all eight task ticks recomputed before each sibling pair; observations consume no mutation RNG.".into(),
        construction_resolution: "Constructed authored fixtures and controlled helper seeds; genomes frozen before discovery. Construction-only tracker opportunities are excluded from proposal totals; creation registration precedes preparation.".into(),
        observation_resolution: "Both siblings every generation; first discovery by discovery horizon; retention on same (lineage,node,creation-depth) after followup generations including held parents. Full 80-execution battery/cohort checkpoints at 0/discovery/end; first-fact dates are observation-censored. Replay deltas are complete whole-birth transitions with ordered events, not per-event field causation. Selected-inapplicable backend comes from Graph/VM domain or the target's before/after node; transient deleted targets without backend evidence remain explicitly unresolved. T13.F07: specialization is task-live, score >= start+1, bypass loss >= 1, ancestral-payload loss >= 1 and every generation-0 correct scene preserved; the ladder and classification read the retained chain; the +64 horizon is censored on this panel.".into(),
        rng_control: "NotApplicable: no added-draw or constructor intervention; observation never advances mutation RNG. Equal seeds need not produce equal transitions after genotype/site divergence; paired fingerprints include complete deltas, ordered events and a draw from a clone of the post-call RNG.".into(),
        limitations: vec!["Task-live is eight-tick survival, not lifetime/ecological viability; drift after task death is genotype drift, not reproduction.".into(),
            "Authored starting forms have matched behavior, unequal size/cost/sites; dormant preparation was constructed, not discovered.".into(),
            "Four batches and eight lineages per batch are replicates; siblings and cross-arm reused seeds are dependent; no positive-discovery floor or pooled-proposal superiority claim.".into(),
            "Target-applicability gaps belong to T13.F03; zero-compute direct Graph effect activation to T13.F04; function-preserving module preparation/recruitment paths to T13.F05. This observation repairs none.".into()],
        total_proposals: opportunities.births, opportunities, constructed: assay.constructed.clone(), starts: assay.starts.clone(), arms, pairs }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn production_lineages(start: &Start, policy: Policy) -> Vec<Lineage> {
        let sizes = Sizes::PRODUCTION;
        let assay = Assay::new(Panel::legacy(sizes));
        let arm = (0..assay.arm_count())
            .find(|&arm| {
                let (candidate, candidate_policy) = assay.arm(arm);
                candidate.name == start.name && candidate_policy == policy
            })
            .expect("every start and policy is an arm");
        (0..sizes.batches)
            .flat_map(|batch| (0..sizes.lineages).map(move |index| (batch, index)))
            .map(|(batch, index)| assay.lineage(arm, batch, index))
            .collect()
    }

    fn facts(lineages: &[Lineage]) -> Vec<LineageFacts> {
        lineages.iter().map(LineageFacts::of).collect()
    }

    #[test]
    fn fingerprint_is_a_sha256_hex_digest() {
        let digest = fingerprint(&("recruitment", 13_02u32));
        assert_eq!(digest.len(), 64);
        assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }

    #[test]
    fn module_use_reports_dispatch_and_each_observable_effect() {
        let start = starting_forms()
            .into_iter()
            .find(|start| start.name == "graph_copy")
            .unwrap();
        let tracker = initial_tracker(&start);
        let module = tracker
            .modules()
            .find(|module| module.provenance.is_cohort())
            .unwrap()
            .node;
        let config = task_config();
        let bypass = evaluate_with_config(&static_successor_bypass(&start.genome, module), &config);

        let mut unchanged = bypass.clone();
        unchanged.scenes[0].dispatched.push(module);
        let unchanged_use = uses(
            &start.genome,
            &unchanged,
            start.task,
            &start.task_reading,
            &tracker,
            &config,
        );
        assert_eq!(unchanged_use.len(), 1);
        assert!(unchanged_use[0].dispatched);
        assert_eq!(
            unchanged_use[0].current_backend,
            unchanged_use[0].created_backend
        );
        assert_eq!(unchanged_use[0].score_loss, 0);
        assert!(!unchanged_use[0].queue_effect);
        assert_eq!(unchanged_use[0].memory_effect, Some(false));
        assert!(!unchanged_use[0].output_effect);
        assert!(!unchanged_use[0].routing_effect);

        let mut changed = bypass.clone();
        changed.scenes[0].dispatched.push(module);
        for scene in &mut changed.scenes {
            scene.correct_a = true;
        }
        if changed.scenes[1].actions.is_empty() {
            changed.scenes[1]
                .actions
                .push(crate::contracts::WorldAction::NoOp);
        } else {
            changed.scenes[1].actions.clear();
        }
        changed.scenes[1].shared_memory = Some([1.0; 16]);
        changed.scenes[1].routing.push((
            crate::contracts::NodeId::new(98),
            0,
            crate::contracts::NodeId::new(99),
        ));
        changed.scenes[1]
            .output_slots
            .push((crate::contracts::NodeId::new(99), [1.0; 24]));
        let changed_use = uses(
            &start.genome,
            &changed,
            start.task,
            &start.task_reading,
            &tracker,
            &config,
        );
        assert_eq!(changed_use.len(), 1);
        assert!(changed_use[0].score_loss > 0);
        assert!(changed_use[0].queue_effect);
        assert_eq!(changed_use[0].memory_effect, Some(true));
        assert!(changed_use[0].output_effect);
        assert!(changed_use[0].routing_effect);
    }

    /// T11.F22 re-pin: the within-kind `InputRef.Swap` and the
    /// consumer-preserving `InputRef.Prune` draw differently from the old
    /// `Swap`/`Remove`, so every lineage diverges at its first
    /// input-reference event; the selected-inapplicable discard counts stay
    /// exactly zero on both backends (T13.F03's repair, kept by the
    /// per-entry predicates). Re-pinned 2026-09-18 when the topology weight
    /// table was scaled ten-fold around `ChangeEntryNode` (weight 1 of 211):
    /// every topology draw shifts, so every lineage diverges at its first
    /// topology event; the discard counts stay zero. Re-pinned by T11.F23
    /// (scale-relative constant steps): the VM drift row's retained-useful
    /// count moved 7 -> 8 with the applied constant values; the other three
    /// rows are unchanged. Re-pinned by T17.F02 (unit-scale introspection,
    /// `Generation` removed from the catalog): every input-reference draw
    /// shifts, and the two drift rows' retained-useful counts moved 7 -> 6
    /// (graph) and 8 -> 7 (VM); the selection rows are unchanged.
    /// Re-pinned 2026-09-19 for the 25% large-copy weight default: topology
    /// draws and subsequent RNG histories change; applicability stays intact.
    /// Re-pinned by T19.F04 (vote-based action selection, remapping
    /// every VM instruction and Graph edge draw and shrinking the modules;
    /// before/after in `docs/progress/readings/t19-f04.md`); the discard
    /// counts stay zero.
    #[test]
    fn production_prepared_lineages_match_the_recorded_baseline_and_metadata() {
        let starts = starting_forms();
        let cases = [
            ("graph_prepared", Policy::Drift, [18, 13, 13, 1], [0, 0]),
            (
                "graph_prepared",
                Policy::Selection,
                [18, 18, 18, 17],
                [0, 0],
            ),
            ("vm_prepared", Policy::Drift, [19, 16, 16, 8], [0, 0]),
            ("vm_prepared", Policy::Selection, [19, 19, 19, 19], [0, 0]),
        ];
        for (name, policy, expected, expected_discards) in cases {
            let start = starts.iter().find(|start| start.name == name).unwrap();
            let lineages = production_lineages(start, policy);
            let aggregate = summary(&facts(&lineages));
            assert_eq!(
                [
                    aggregate.proposal_discovery.numerator,
                    aggregate.retained_discovery.numerator,
                    aggregate.viable_retained_discovery.numerator,
                    aggregate.retained_useful.numerator,
                ],
                expected,
                "{name} {policy:?}"
            );

            let mut graph_discards = 0;
            let mut vm_discards = 0;
            for lineage in &lineages {
                let expected_identity =
                    lineage.batch * Sizes::PRODUCTION.lineages + lineage.lineage;
                assert!(lineage.checkpoints.iter().all(|checkpoint| checkpoint
                    .modules
                    .iter()
                    .all(|module| module.module.lineage == expected_identity)));
                assert!(lineage.checkpoints.iter().all(|checkpoint| checkpoint
                    .cohort
                    .lineage_rows
                    .iter()
                    .all(|row| row.lineage == expected_identity)));
                assert_eq!(
                    lineage
                        .checkpoints
                        .iter()
                        .map(|checkpoint| checkpoint.generation)
                        .collect::<Vec<_>>(),
                    vec![
                        0,
                        Sizes::PRODUCTION.discovery,
                        Sizes::PRODUCTION.discovery + Sizes::PRODUCTION.followup,
                    ]
                );

                let mut viable_path = start.task_reading.live();
                for siblings in lineage.proposals.chunks_exact(2) {
                    for proposal in siblings {
                        let child_score = match start.task {
                            Task::A => proposal.outcome.correct_a,
                            Task::B => proposal.outcome.correct_b,
                        };
                        assert_eq!(
                            proposal.seed,
                            proposal_seed(
                                lineage.batch,
                                lineage.lineage,
                                proposal.generation - 1,
                                proposal.sibling
                            )
                        );
                        assert_eq!(proposal.mutation_fingerprint.len(), 64);
                        assert_eq!(proposal.parent_fingerprint.len(), 64);
                        assert_eq!(
                            proposal.discovery,
                            proposal.outcome.surviving_scenes == 8
                                && child_score > start.task_reading.correct(start.task)
                                && !proposal.useful_modules.is_empty()
                        );
                        assert_eq!(
                            proposal.viable_path,
                            viable_path
                                && proposal.parent_live
                                && proposal.outcome.surviving_scenes == 8
                                && child_score + 1 >= proposal.parent_score
                        );
                        graph_discards += proposal
                            .selected_inapplicable_by_backend_operator
                            .get(&ModuleBackend::Graph)
                            .into_iter()
                            .flat_map(|operators| operators.values())
                            .sum::<u64>();
                        vm_discards += proposal
                            .selected_inapplicable_by_backend_operator
                            .get(&ModuleBackend::Vm)
                            .into_iter()
                            .flat_map(|operators| operators.values())
                            .sum::<u64>();
                        assert_eq!(proposal.selected_inapplicable_backend_unresolved, 0);
                        assert!(proposal
                            .useful_modules
                            .iter()
                            .all(|module| { module.dispatched && module.score_loss >= 1 }));
                    }
                    if let Some(chosen) = siblings.iter().find(|proposal| proposal.chosen) {
                        viable_path = chosen.viable_path;
                    }
                }
            }
            assert_eq!([graph_discards, vm_discards], expected_discards);
        }
    }

    /// T13.F06 reading, re-pinned at T11.F22 (the within-kind swap and the
    /// prune move every lineage at its first input-reference event): under
    /// cost-visible selection the two prepared forms keep every retained
    /// discovery useful (22/22 and 19/19 against 22/22 and 21/20 under F02
    /// selection; values re-pinned 2026-09-18 with the topology weight
    /// rescale around `ChangeEntryNode`).
    /// Re-pinned 2026-09-19 for the 25% large-copy weight default: 21/21
    /// graph and 20/20 VM retained discoveries remain useful.
    /// Re-pinned by T19.F04 (vote sinks): 16/16 graph and 21/21 VM retained
    /// discoveries remain useful.
    #[test]
    fn production_cost_selection_lineages_pin_the_first_reading() {
        let starts = starting_forms();
        for (name, expected, censored) in [
            ("graph_prepared", [16, 16, 16, 16], 16),
            ("vm_prepared", [21, 21, 21, 21], 11),
        ] {
            let start = starts.iter().find(|start| start.name == name).unwrap();
            let lineages = production_lineages(start, Policy::CostSelection);
            let aggregate = summary(&facts(&lineages));
            assert_eq!(
                [
                    aggregate.proposal_discovery.numerator,
                    aggregate.retained_discovery.numerator,
                    aggregate.viable_retained_discovery.numerator,
                    aggregate.retained_useful.numerator,
                ],
                expected,
                "{name}"
            );
            assert_eq!(
                aggregate.retention_outcomes,
                RetentionOutcomes {
                    useful: expected[3],
                    ..RetentionOutcomes::default()
                },
                "{name}"
            );
            assert_eq!(
                aggregate.time_to_first_retained.censored, censored,
                "{name}"
            );
            super::super::tests::assert_summary_readings_are_consistent(&Arm {
                start: start.name.clone(),
                task: start.task,
                policy: Policy::CostSelection,
                summary: aggregate,
                batches: vec![],
                opportunities: Opportunities::default(),
                lineages,
            });
        }
    }

    #[test]
    fn paired_lineages_report_each_first_divergence_and_signed_difference() {
        let start = starting_forms()
            .into_iter()
            .find(|start| start.name == "graph_prepared")
            .unwrap();
        let left_lineages = production_lineages(&start, Policy::Drift);
        let right_lineages = production_lineages(&start, Policy::Selection);
        let make_arm = |policy, lineages: Vec<Lineage>| Arm {
            start: start.name.clone(),
            task: start.task,
            policy,
            summary: summary(&facts(&lineages)),
            batches: vec![],
            opportunities: Opportunities::default(),
            lineages,
        };
        let left = make_arm(Policy::Drift, left_lineages);
        let right = make_arm(Policy::Selection, right_lineages);
        let paired = pair(&left, &right, 3, 7);
        assert_eq!((paired.left_arm, paired.right_arm), (3, 7));
        for ((left, right), observed) in left
            .lineages
            .iter()
            .zip(&right.lineages)
            .zip(&paired.lineages)
        {
            let proposals: Vec<_> = left.proposals.iter().zip(&right.proposals).collect();
            let first_mutation = proposals
                .iter()
                .position(|(left, right)| left.mutation_fingerprint != right.mutation_fingerprint);
            let useful = |lineage: &Lineage| {
                lineage
                    .retention
                    .as_ref()
                    .is_some_and(|retention| retention.outcome == RetentionOutcome::Useful)
            };
            assert_eq!(observed.batch, left.batch);
            assert_eq!(observed.lineage, left.lineage);
            assert_eq!(
                observed.proposal_discovery_difference,
                i8::from(right.proposal_discovery.is_some())
                    - i8::from(left.proposal_discovery.is_some())
            );
            assert_eq!(
                observed.retained_discovery_difference,
                i8::from(right.retained_discovery.is_some())
                    - i8::from(left.retained_discovery.is_some())
            );
            assert_eq!(
                observed.useful_retention_difference,
                i8::from(useful(right)) - i8::from(useful(left))
            );
            assert_eq!(
                observed.first_parent_divergence,
                proposals
                    .iter()
                    .find(|(left, right)| left.parent_fingerprint != right.parent_fingerprint)
                    .map(|(left, _)| left.generation - 1)
            );
            assert_eq!(
                observed.first_mutation_divergence,
                first_mutation
                    .map(|index| { (proposals[index].0.generation, proposals[index].0.sibling) })
            );
            assert_eq!(
                observed.first_rng_divergence,
                proposals
                    .iter()
                    .find(|(left, right)| left.rng_after != right.rng_after)
                    .map(|(left, _)| (left.generation, left.sibling))
            );
            assert_eq!(
                observed.matched_proposals_before_divergence,
                first_mutation.unwrap_or(proposals.len()) as u32
            );
        }
    }

    fn synthetic_proposal(generation: u32, parent_score: u8, outcome: TaskSummary) -> Proposal {
        Proposal {
            generation,
            sibling: 0,
            seed: 0,
            parent_live: true,
            parent_score,
            chosen: false,
            outcome,
            modules: 0,
            genome_size: 0,
            opportunities: Opportunities::default(),
            events: Vec::new(),
            useful_modules: Vec::new(),
            discovery: false,
            specialized: false,
            viable_path: true,
            mutation_fingerprint: String::new(),
            parent_fingerprint: String::new(),
            rng_after: 0,
            selected_inapplicable_by_backend_operator: Default::default(),
            selected_inapplicable_backend_unresolved: 0,
        }
    }

    fn synthetic_outcome(surviving_scenes: u8, correct_a: u8, correct_b: u8) -> TaskSummary {
        TaskSummary {
            correct_a,
            correct_b,
            surviving_scenes,
            ending_energy_sum: 0.0,
            maintenance_sum: 0.0,
            carrying_sum: 0.0,
            work: Work::default(),
        }
    }

    /// Damage counts task death over every proposal and a score loss against
    /// the parent over the task-live proposals only; the reduced run never
    /// draws a task-dead proposal, so this pins the fold on synthetic lineages.
    #[test]
    fn summary_damage_counts_task_death_over_all_and_loss_over_the_task_live() {
        // Arrange: three task-dead proposals (one of them also below the parent
        // score on Task A), then a live loss, a live neutral step and a live gain.
        let proposals = vec![
            synthetic_proposal(1, 4, synthetic_outcome(7, 4, 0)),
            synthetic_proposal(1, 4, synthetic_outcome(0, 1, 8)),
            synthetic_proposal(2, 4, synthetic_outcome(5, 8, 8)),
            synthetic_proposal(2, 4, synthetic_outcome(8, 3, 8)),
            synthetic_proposal(3, 4, synthetic_outcome(8, 4, 0)),
            synthetic_proposal(3, 4, synthetic_outcome(8, 5, 0)),
        ];
        let lineage = Lineage {
            batch: 0,
            lineage: 0,
            task: Task::A,
            proposals,
            checkpoints: Vec::new(),
            proposal_discovery: None,
            retained_discovery: None,
            viable_retained_discovery: false,
            retention: None,
            first_successful_path: Vec::new(),
            specialized_discovery: None,
            discovery_checkpoint: None,
            horizons: Vec::new(),
            ladder: Ladder::default(),
            classification: LineageClass::NoEligibility,
        };

        // Act
        let aggregate = summary(&[LineageFacts::of(&lineage)]);

        // Assert
        assert_eq!(aggregate.damage.task_dead, estimate(3, 6));
        assert_eq!(aggregate.damage.task_live_loss, estimate(1, 3));
        assert_eq!(aggregate.retention_outcomes, RetentionOutcomes::default());
        assert!(aggregate.checkpoint_cost.is_empty());
        assert_eq!(aggregate.retained_discovery, estimate(0, 1));
    }

    /// Every fixed start's authored module 2 is reachable and undispatched
    /// at generation 0: eligibility is raised before the first proposal and
    /// expression is not.
    #[test]
    fn every_start_reaches_but_does_not_express_its_cohort_module_at_generation_zero() {
        for start in starting_forms() {
            let tracker = initial_tracker(&start);
            assert_eq!(
                reachable_cohort(&start.genome, &tracker),
                BTreeSet::from([NodeId::new(2)]),
                "{}",
                start.name
            );
            assert!(!expressed(&tracker), "{}", start.name);
        }
    }

    /// Two cohort modules born at the same depth are told apart by node: the
    /// recruit reading is the discovered module's, not its earlier sibling's.
    #[test]
    fn recruit_reading_finds_the_discovered_module_among_same_depth_siblings() {
        use crate::mutation::topology::TopologyOperator;
        let start = starting_forms()
            .into_iter()
            .find(|start| start.name == "graph_copy")
            .unwrap();
        let base = fixtures::base(ModuleBackend::Graph, true);
        let mut copied = base.clone();
        fixtures::topology(&mut copied, TopologyOperator::CopyNode, 7);
        let second = fixtures::topology(&mut copied, TopologyOperator::CopyNode, 11);
        assert_eq!(
            copied
                .nodes
                .iter()
                .map(|node| node.node_id.0)
                .collect::<Vec<_>>(),
            [0, 1, 2, 3]
        );
        let mut tracker = RecruitmentTracker::new(1);
        tracker.seed_founder(0, &base.nodes);
        tracker.record_birth(BirthObservation {
            lineage: 0,
            depth: 1,
            after: &copied.nodes,
            summary: &second,
        });
        let config = task_config();
        let reading = evaluate_with_config(&copied, &config);
        let all = uses(
            &copied,
            &reading,
            start.task,
            &start.task_reading,
            &tracker,
            &config,
        );
        assert_eq!(
            all.iter()
                .map(|module| (module.node, module.created_depth))
                .collect::<Vec<_>>(),
            [(NodeId::new(2), 1), (NodeId::new(3), 1)]
        );
        let discovery = Discovery {
            generation: 1,
            sibling: 0,
            module: all[1].clone(),
        };

        let found = recruit_reading(&copied, &reading, &start, &tracker, &discovery, &config);

        assert_eq!(found.map(|module| module.node), Some(NodeId::new(3)));
    }
}
