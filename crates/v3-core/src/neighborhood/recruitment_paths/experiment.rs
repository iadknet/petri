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

/// The mutation config every proposal runs: the default config on the legacy
/// per-birth supply rule, the fixed-count control the recorded baselines were
/// taken on, like the drift walk (T11.F19).
pub(super) fn proposal_mutation_config() -> crate::config::MutationConfig {
    crate::config::MutationConfig::default().with_legacy_supply()
}

fn propose_siblings(
    start: &Start,
    genome: &CreatureGenome,
    task: &TaskReading,
    tracker: &RecruitmentTracker,
    position: ProposalPosition,
    viable_path: bool,
    config: &crate::config::SimulationConfig,
) -> Vec<Sibling> {
    let ProposalPosition {
        batch,
        lineage,
        generation,
    } = position;
    let mutation = proposal_mutation_config();
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
            Candidate::new(&task, start.task),
            [
                Candidate::new(&children[0].task, start.task),
                Candidate::new(&children[1].task, start.task),
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

/// Retained-parent cost at each checkpoint generation, spread over lineages.
/// Every lineage checkpoints at the same generations.
fn checkpoint_cost(lineages: &[&Lineage]) -> Vec<CheckpointCost> {
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
            let spread = |value: fn(&Checkpoint) -> f64| {
                Spread::of(at.iter().map(|checkpoint| value(checkpoint)).collect())
            };
            Some(CheckpointCost {
                generation: reference.generation,
                genome_size: spread(|checkpoint| f64::from(checkpoint.genome.genome_size()))?,
                modules: spread(|checkpoint| checkpoint.genome.nodes.len() as f64)?,
                carrying_sum: spread(|checkpoint| checkpoint.task.summary().carrying_sum)?,
                ending_energy_sum: spread(|checkpoint| {
                    checkpoint.task.summary().ending_energy_sum
                })?,
            })
        })
        .collect()
}

fn summary(task: Task, lineages: &[&Lineage]) -> Summary {
    let count = lineages.len() as u32;
    let discoveries: Vec<_> = lineages
        .iter()
        .filter_map(|lineage| lineage.retained_discovery.as_ref())
        .collect();
    let mut retention_outcomes = RetentionOutcomes::default();
    for retention in lineages
        .iter()
        .filter_map(|lineage| lineage.retention.as_ref())
    {
        retention_outcomes.record(retention.outcome);
    }
    let useful = retention_outcomes.useful;
    let (proposals, task_dead, task_live_loss) = lineages
        .iter()
        .flat_map(|lineage| &lineage.proposals)
        .fold((0, 0, 0), |(total, dead, loss), proposal| {
            let outcome = &proposal.outcome;
            (
                total + 1,
                dead + u32::from(!outcome.live()),
                loss + u32::from(outcome.live() && outcome.correct(task) < proposal.parent_score),
            )
        });
    let damage = Damage {
        task_dead: estimate(task_dead, proposals),
        task_live_loss: estimate(task_live_loss, proposals - task_dead),
    };
    Summary {
        time_to_first_retained: TimeToFirst::of(lineages.iter().map(|lineage| {
            lineage
                .retained_discovery
                .as_ref()
                .map(|discovery| discovery.generation)
        })),
        time_to_first_proposal: TimeToFirst::of(lineages.iter().map(|lineage| {
            lineage
                .proposal_discovery
                .as_ref()
                .map(|discovery| discovery.generation)
        })),
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
    let mut arms = Vec::with_capacity(ARMS);
    let mut opportunities = Opportunities::default();
    // The eighteen T13.F02 arms keep their order and indices (`arms/0` is
    // `graph_blank` under drift); the nine T13.F06 cost arms are appended.
    let f02 = starts
        .iter()
        .flat_map(|start| Policy::F02.map(|policy| (start, policy)));
    let cost = starts.iter().map(|start| (start, Policy::CostSelection));
    for (start, policy) in f02.chain(cost) {
        let mut lineages = Vec::new();
        for batch in 0..sizes.batches {
            for index in 0..sizes.lineages {
                lineages.push(lineage(
                    start, policy, sizes, batch, index, &battery, &config,
                ));
            }
        }
        let all: Vec<_> = lineages.iter().collect();
        let summary = summary(start.task, &all);
        let batches = (0..sizes.batches)
            .map(|batch| {
                self::summary(
                    start.task,
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
        mutation_context: "Default MutationConfig on the legacy per-birth supply rule (per_unit_supply_enabled forced false); SmallRng seed=13020000+batch*1000000+lineage*10000+generation_zero_based*2+sibling. Reachability and ParentExecuted indices from all eight task ticks recomputed before each sibling pair; observations consume no mutation RNG.".into(),
        construction_resolution: "Constructed authored fixtures and controlled helper seeds; genomes frozen before discovery. Construction-only tracker opportunities are excluded from proposal totals; creation registration precedes preparation.".into(),
        observation_resolution: "Both siblings every generation; first discovery by discovery horizon; retention on same (lineage,node,creation-depth) after followup generations including held parents. Full 80-execution battery/cohort checkpoints at 0/discovery/end; first-fact dates are observation-censored. Replay deltas are complete whole-birth transitions with ordered events, not per-event field causation. Selected-inapplicable backend comes from Graph/VM domain or the target's before/after node; transient deleted targets without backend evidence remain explicitly unresolved.".into(),
        rng_control: "NotApplicable: no added-draw or constructor intervention; observation never advances mutation RNG. Equal seeds need not produce equal transitions after genotype/site divergence; paired fingerprints include complete deltas, ordered events and a draw from a clone of the post-call RNG.".into(),
        limitations: vec!["Task-live is eight-tick survival, not lifetime/ecological viability; drift after task death is genotype drift, not reproduction.".into(),
            "Authored starting forms have matched behavior, unequal size/cost/sites; dormant preparation was constructed, not discovered.".into(),
            "Four batches and eight lineages per batch are replicates; siblings and cross-arm reused seeds are dependent; no positive-discovery floor or pooled-proposal superiority claim.".into(),
            "Target-applicability gaps belong to T13.F03; zero-compute direct Graph effect activation to T13.F04; function-preserving module preparation/recruitment paths to T13.F05. This observation repairs none.".into()],
        total_proposals: opportunities.births, opportunities, constructed, starts, arms, pairs }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn production_lineages(start: &Start, policy: Policy) -> Vec<Lineage> {
        let sizes = Sizes::PRODUCTION;
        let config = task_config();
        let battery = Battery::generate(config.world.food.types.len());
        (0..sizes.batches)
            .flat_map(|batch| {
                let config = &config;
                let battery = &battery;
                (0..sizes.lineages)
                    .map(move |index| lineage(start, policy, sizes, batch, index, battery, config))
            })
            .collect()
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
        let unchanged_use = uses(&start.genome, &unchanged, start.task, &tracker, &config);
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
            crate::contracts::NodeId::new(99),
        ));
        changed.scenes[1]
            .output_slots
            .push((crate::contracts::NodeId::new(99), [1.0; 24]));
        let changed_use = uses(&start.genome, &changed, start.task, &tracker, &config);
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
    /// rows are unchanged.
    #[test]
    fn production_prepared_lineages_match_the_recorded_baseline_and_metadata() {
        let starts = starting_forms();
        let cases = [
            ("graph_prepared", Policy::Drift, [21, 13, 11, 7], [0, 0]),
            (
                "graph_prepared",
                Policy::Selection,
                [22, 22, 22, 22],
                [0, 0],
            ),
            ("vm_prepared", Policy::Drift, [21, 14, 12, 8], [0, 0]),
            ("vm_prepared", Policy::Selection, [21, 21, 21, 20], [0, 0]),
        ];
        for (name, policy, expected, expected_discards) in cases {
            let start = starts.iter().find(|start| start.name == name).unwrap();
            let lineages = production_lineages(start, policy);
            let refs: Vec<_> = lineages.iter().collect();
            let aggregate = summary(start.task, &refs);
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
                    .all(|module| module.lineage == expected_identity)));
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
    #[test]
    fn production_cost_selection_lineages_pin_the_first_reading() {
        let starts = starting_forms();
        for (name, expected, censored) in [
            ("graph_prepared", [22, 22, 22, 22], 10),
            ("vm_prepared", [19, 19, 19, 19], 13),
        ] {
            let start = starts.iter().find(|start| start.name == name).unwrap();
            let lineages = production_lineages(start, Policy::CostSelection);
            let refs: Vec<_> = lineages.iter().collect();
            let aggregate = summary(start.task, &refs);
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
        let make_arm = |policy, lineages: Vec<Lineage>| {
            let refs: Vec<_> = lineages.iter().collect();
            Arm {
                start: start.name.clone(),
                task: start.task,
                policy,
                summary: summary(start.task, &refs),
                batches: vec![],
                opportunities: Opportunities::default(),
                lineages,
            }
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
            proposals,
            checkpoints: Vec::new(),
            proposal_discovery: None,
            retained_discovery: None,
            viable_retained_discovery: false,
            retention: None,
            first_successful_path: Vec::new(),
        };

        // Act
        let aggregate = summary(Task::A, &[&lineage]);

        // Assert
        assert_eq!(aggregate.damage.task_dead, estimate(3, 6));
        assert_eq!(aggregate.damage.task_live_loss, estimate(1, 3));
        assert_eq!(aggregate.retention_outcomes, RetentionOutcomes::default());
        assert!(aggregate.checkpoint_cost.is_empty());
        assert_eq!(aggregate.retained_discovery, estimate(0, 1));
    }
}
