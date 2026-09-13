use super::super::mesh_execution::{indices_for_node_ids, static_successor_bypass};
use super::super::recruitment::{
    BirthObservation, ModuleBackend, Opportunities, RecruitmentTracker,
};
use super::super::{Battery, Signature};
use super::*;
use crate::creature::genome::{analysis::mesh_reachable_nodes, CreatureGenome};
use crate::mutation::reachability::ParentExecuted;
use crate::mutation::{MutationDomain, MutationEngine, MutationSummary};
use rand::{rngs::SmallRng, RngCore, SeedableRng};
use serde::Serialize;
use sha2::{Digest, Sha256};

pub fn proposal_seed(batch: u32, lineage: u32, generation: u32, sibling: u8) -> u64 {
    13_020_000
        + u64::from(batch) * 1_000_000
        + u64::from(lineage) * 10_000
        + u64::from(generation) * 2
        + u64::from(sibling)
}

fn fingerprint(value: &impl Serialize) -> String {
    hex::encode(Sha256::digest(
        serde_json::to_vec(value).expect("observation values serialize"),
    ))
}

fn initial_tracker(start: &Start) -> RecruitmentTracker {
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
    tracker
}

fn uses(
    genome: &CreatureGenome,
    reading: &TaskReading,
    task: Task,
    tracker: &RecruitmentTracker,
    config: &crate::config::SimulationConfig,
) -> Vec<ModuleUse> {
    let dispatched = reading.dispatched();
    tracker
        .modules()
        .filter(|module| module.is_present() && module.provenance.is_cohort())
        .map(|module| {
            let node = genome
                .nodes
                .iter()
                .find(|node| node.node_id == module.node)
                .expect("present tracker node");
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
            };
            if dispatched {
                let bypass =
                    evaluate_with_config(&static_successor_bypass(genome, module.node), config);
                result.score_loss =
                    i16::from(reading.correct(task)) - i16::from(bypass.correct(task));
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
    Checkpoint {
        generation,
        genome: genome.clone(),
        task: reading.clone(),
        battery_class: format!("{:?}", super::super::classify(baseline, &signature).class),
        battery: signature,
        cohort: tracker.checkpoint(u64::from(generation)),
        modules: tracker.modules().cloned().collect(),
        task_use: uses(genome, reading, start.task, tracker, config),
    }
}

struct Candidate {
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

fn propose_siblings(
    start: &Start,
    genome: &CreatureGenome,
    task: &TaskReading,
    tracker: &RecruitmentTracker,
    position: ProposalPosition,
    viable_path: bool,
    config: &crate::config::SimulationConfig,
) -> Vec<Candidate> {
    let ProposalPosition {
        batch,
        lineage,
        generation,
    } = position;
    let mutation = crate::config::MutationConfig::default();
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
            &mutation,
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
        let module_uses = uses(&child, &child_task, start.task, &child_tracker, config);
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
            viable_path: viable_path
                && parent_live
                && child_task.live()
                && child_task.correct(start.task) + 1 >= parent_score,
        };
        children.push(Candidate {
            genome: child,
            task: child_task,
            tracker: child_tracker,
            record,
            delta,
        });
    }
    children
}

fn lineage(
    start: &Start,
    policy: Policy,
    sizes: Sizes,
    batch: u32,
    lineage: u32,
    battery: &Battery,
    config: &crate::config::SimulationConfig,
) -> Lineage {
    let mut genome = start.genome.clone();
    let mut task = start.task_reading.clone();
    let mut tracker = initial_tracker(start);
    let baseline = battery.signature(&genome, &config.runtime, config.shared_memory.decay_rate);
    let mut result = Lineage {
        batch,
        lineage,
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
    };
    let mut viable_path = task.live();
    let mut history = Vec::new();
    for generation in 1..=sizes.discovery + sizes.followup {
        let parent_score = task.correct(start.task);
        let parent_live = task.live();
        let mut children = propose_siblings(
            start,
            &genome,
            &task,
            &tracker,
            ProposalPosition {
                batch,
                lineage,
                generation,
            },
            viable_path,
            config,
        );
        let winner = choose(
            policy,
            (parent_live, parent_score),
            [
                (
                    children[0].task.live(),
                    children[0].task.correct(start.task),
                ),
                (
                    children[1].task.live(),
                    children[1].task.correct(start.task),
                ),
            ],
        );
        if let Some(index) = winner {
            children[index].record.chosen = true;
        }
        if generation <= sizes.discovery && result.proposal_discovery.is_none() {
            if let Some(child) = children.iter().find(|child| child.record.discovery) {
                result.proposal_discovery = Some(Discovery {
                    generation,
                    sibling: child.record.sibling,
                    module: child.record.useful_modules[0].clone(),
                });
                result.first_successful_path = history.clone();
                result.first_successful_path.push(ReplayStep {
                    generation,
                    sibling: Some(child.record.sibling),
                    seed: Some(child.record.seed),
                    retained: child.record.chosen,
                    events: child.record.events.clone(),
                    delta: child.delta.clone(),
                    outcome: child.task.clone(),
                });
            }
        }
        for child in &children {
            result.proposals.push(child.record.clone());
        }
        if let Some(index) = winner {
            let child = children.swap_remove(index);
            viable_path = child.record.viable_path;
            if generation <= sizes.discovery
                && result.retained_discovery.is_none()
                && child.record.discovery
            {
                result.retained_discovery = Some(Discovery {
                    generation,
                    sibling: child.record.sibling,
                    module: child.record.useful_modules[0].clone(),
                });
                result.viable_retained_discovery = viable_path;
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
        if let Some(discovery) = &result.retained_discovery {
            if generation == discovery.generation + sizes.followup {
                let present = tracker.modules().any(|module| {
                    module.node == discovery.module.node
                        && module.created_depth == discovery.module.created_depth
                        && module.is_present()
                });
                let score_loss = present.then(|| {
                    let bypass = evaluate_with_config(
                        &static_successor_bypass(&genome, discovery.module.node),
                        config,
                    );
                    i16::from(task.correct(start.task)) - i16::from(bypass.correct(start.task))
                });
                let outcome = classify_retention(task.live(), score_loss);
                result.retention = Some(Retention {
                    discovery: discovery.clone(),
                    at_generation: generation,
                    outcome,
                    score: task.correct(start.task),
                    score_loss,
                });
            }
        }
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
    // Each tracker uses local index zero; expose the actual batch/lineage
    // identity on its F01 module/cohort records at the report boundary.
    for checkpoint in &mut result.checkpoints {
        for module in &mut checkpoint.modules {
            module.lineage = batch * sizes.lineages + lineage;
        }
        for row in &mut checkpoint.cohort.lineage_rows {
            row.lineage = batch * sizes.lineages + lineage;
        }
    }
    result
}

fn summary(lineages: &[&Lineage]) -> Summary {
    let count = lineages.len() as u32;
    let discoveries: Vec<_> = lineages
        .iter()
        .filter_map(|lineage| lineage.retained_discovery.as_ref())
        .collect();
    let useful = lineages
        .iter()
        .filter(|lineage| {
            lineage
                .retention
                .as_ref()
                .is_some_and(|retention| retention.outcome == RetentionOutcome::Useful)
        })
        .count() as u32;
    Summary {
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
            .map(|discovery| discovery.generation)
            .min()
            .zip(
                discoveries
                    .iter()
                    .map(|discovery| discovery.generation)
                    .max(),
            )
            .map(|(min, max)| [min, max]),
    }
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

/// One report-level observation; it has no ecological world input or RNG.
#[must_use]
pub fn observe(sizes: Sizes) -> Report {
    assert!(sizes.batches > 0 && sizes.batches <= 4 && sizes.lineages > 0 && sizes.lineages <= 8);
    assert!(
        sizes.discovery > 0 && sizes.discovery <= 32 && sizes.followup > 0 && sizes.followup <= 16
    );
    let config = task_config();
    let battery = Battery::generate(config.world.food.types.len());
    let constructed = constructed_paths();
    let starts = fixtures::starts_from_paths(&constructed);
    let mut arms = Vec::with_capacity(18);
    let mut opportunities = Opportunities::default();
    for start in &starts {
        for policy in [Policy::Drift, Policy::Selection] {
            let mut lineages = Vec::new();
            for batch in 0..sizes.batches {
                for index in 0..sizes.lineages {
                    lineages.push(lineage(
                        start, policy, sizes, batch, index, &battery, &config,
                    ));
                }
            }
            let all: Vec<_> = lineages.iter().collect();
            let summary = summary(&all);
            let batches = (0..sizes.batches)
                .map(|batch| {
                    self::summary(
                        &all.iter()
                            .copied()
                            .filter(|lineage| lineage.batch == batch)
                            .collect::<Vec<_>>(),
                    )
                })
                .collect();
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
    Report { version: VERSION.into(), config_digest: crate::config::config_digest(&config), config, sizes,
        task_definition: "Eight fresh 12x12 one-tick scenes; energy 50; zero learned state; here/east/north food 0/1. A: FoodHere(0)>0; B: NeighborFood(E,0)>0. Exact [Move(E)] plus east displacement or [NoOp] plus no displacement. Score=correct/8; practical margin=1/8.".into(),
        mutation_context: "Default MutationConfig; SmallRng seed=13020000+batch*1000000+lineage*10000+generation_zero_based*2+sibling. Reachability and ParentExecuted indices from all eight task ticks recomputed before each sibling pair; observations consume no mutation RNG.".into(),
        construction_resolution: "Constructed authored fixtures and controlled helper seeds; genomes frozen before discovery. Construction-only tracker opportunities are excluded from proposal totals; creation registration precedes preparation.".into(),
        observation_resolution: "Both siblings every generation; first discovery by discovery horizon; retention on same (lineage,node,creation-depth) after followup generations including held parents. Full 80-execution battery/cohort checkpoints at 0/discovery/end; first-fact dates are observation-censored. Replay deltas are complete whole-birth transitions with ordered events, not per-event field causation. Selected-inapplicable backend comes from Graph/VM domain or the target's before/after node; transient deleted targets without backend evidence remain explicitly unresolved.".into(),
        rng_control: "NotApplicable: no added-draw or constructor intervention; observation never advances mutation RNG. Equal seeds need not produce equal transitions after genotype/site divergence; paired fingerprints include complete deltas, ordered events and a draw from a clone of the post-call RNG.".into(),
        limitations: vec!["Task-live is eight-tick survival, not lifetime/ecological viability; drift after task death is genotype drift, not reproduction.".into(),
            "Authored starting forms have matched behavior, unequal size/cost/sites; dormant preparation was constructed, not discovered.".into(),
            "Four batches and eight lineages per batch are replicates; siblings and cross-arm reused seeds are dependent; no positive-discovery floor or pooled-proposal superiority claim.".into(),
            "Target-applicability gaps belong to T13.F03; zero-compute direct Graph effect activation to T13.F04; function-preserving module preparation/recruitment paths to T13.F05. This observation repairs none.".into()],
        total_proposals: opportunities.births, opportunities, constructed, starts, arms, pairs }
}
