use super::{checkpoint, qualifies_score, Panel, Qualification, Reading, FAMILIES};
use crate::config::{MutationConfig, NeutralInputRecruitment};
use crate::contracts::InputReference;
use crate::creature::genome::{analysis::mesh_reachable_nodes, CreatureGenome};
use crate::mutation::{
    graph::refinement::{eligible_focal_groups, AccessStep, RefinementDiagnostic, RefinementStep},
    reachability::ParentExecuted,
    MutationEngine,
};
use crate::neighborhood::recruitment_paths::Event;
use rand::{rngs::SmallRng, RngCore, SeedableRng};
use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Arm {
    B,
    S,
    W,
    C,
    M,
}
impl Arm {
    pub const ALL: [Self; 5] = [Self::B, Self::S, Self::W, Self::C, Self::M];
    pub fn config(self) -> MutationConfig {
        MutationConfig {
            neutral_input_recruitment: match self {
                Self::B => NeutralInputRecruitment::Off,
                Self::S => NeutralInputRecruitment::SingleChannel,
                _ => NeutralInputRecruitment::WholeFamily,
            },
            structured_heritable_refinement: matches!(self, Self::C | Self::M),
            ..MutationConfig::default()
        }
    }
}

pub fn seed(
    panel: u32,
    family: usize,
    arm: Arm,
    lineage: u32,
    generation: u32,
    sibling: u8,
) -> u64 {
    20_090_000_000
        + u64::from(panel) * 1_000_000_000
        + family as u64 * 100_000_000
        + arm as u64 * 1_000_000
        + u64::from(lineage) * 10_000
        + u64::from(generation) * 2
        + u64::from(sibling)
}

pub fn digest(value: &impl Serialize) -> String {
    super::super::recruitment::json_sha256(value)
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Identity {
    pub panel: u32,
    pub family: usize,
    pub arm: Arm,
    pub lineage: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Outcome {
    pub correct: u32,
    pub opportunities: u32,
    pub incumbent_preserved: u32,
    pub incumbent_scenes: u32,
    pub alive: bool,
}
impl From<&Reading> for Outcome {
    fn from(reading: &Reading) -> Self {
        Self {
            correct: reading.correct,
            opportunities: reading.opportunities,
            incumbent_preserved: reading.incumbent_preserved,
            incumbent_scenes: reading.incumbent_scenes,
            alive: reading.alive,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Proposal {
    pub identity: Identity,
    pub generation: u32,
    pub sibling: u8,
    pub seed: u64,
    pub chosen: bool,
    pub parent_digest: String,
    pub digest: String,
    pub rng_after: u64,
    pub units: u32,
    pub requested: u32,
    pub applied: u32,
    pub skipped: u32,
    pub events: Vec<Event>,
    pub access: Vec<AccessStep>,
    pub refinement: Vec<RefinementStep>,
    pub eligible_focal_groups: usize,
    pub focal_declarations: usize,
    pub focal_connected: usize,
    pub outcome: Outcome,
    pub training_qualified: bool,
    pub training_vm_qualified: bool,
    pub mutation_secs: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Frozen {
    pub chosen_focal_structured_events: u32,
    pub identity: Identity,
    pub generation: u32,
    pub seed: Option<u64>,
    pub role: String,
    pub digest: String,
    pub genotype: CreatureGenome,
    pub training: Option<Qualification>,
    pub held_out: Option<Qualification>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Lineage {
    pub identity: Identity,
    pub generations: u32,
    pub proposals: u32,
    pub requested: u32,
    pub applied: u32,
    pub skipped: u32,
    pub focal_access_events: u32,
    pub focal_structured_events: u32,
    pub chosen_focal_structured_events: u32,
    pub eligible_proposals: u32,
    pub proposal_discoveries: u32,
    pub vm_proposal_discoveries: u32,
    pub endpoint_graph_discovery: bool,
    pub endpoint_vm_discovery: bool,
    pub first_training_generation: Option<u32>,
    pub primary_graph_discovery: bool,
    pub primary_vm_discovery: bool,
    pub primary_joint_effect: bool,
    pub endpoint_digest: String,
    pub endpoint_size: u32,
    pub right_censored: bool,
    pub terminal: String,
    pub complete: bool,
}

/// A streamed record; the manifest precedes these in the CLI artifact.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", content = "record", rename_all = "snake_case")]
pub enum Record {
    Proposal(Proposal),
    Frozen(Box<Frozen>),
    Lineage(Lineage),
}

struct Child {
    genome: CreatureGenome,
    reading: Reading,
    record: Proposal,
    qualified: Option<Qualification>,
}

fn primary_eligible(graph_qualified: bool, arm: Arm, chosen_focal_structured_events: u32) -> bool {
    graph_qualified && (arm != Arm::C || chosen_focal_structured_events > 0)
}

pub fn choose(parent: &Reading, children: [&Reading; 2]) -> Option<usize> {
    let mut best = None;
    let mut fraction = parent.fraction();
    // First sibling wins ties; either sibling beats the parent on a tie.
    for (index, child) in children.iter().enumerate() {
        if child.acceptable()
            && (child.fraction() > fraction || best.is_none() && child.fraction() == fraction)
        {
            best = Some(index);
            fraction = child.fraction();
        }
    }
    best
}

fn propose(
    parent: &CreatureGenome,
    reading: &Reading,
    panel: &Panel,
    identity: Identity,
    generation: u32,
    sibling: u8,
    observed: bool,
) -> Child {
    let seed = seed(
        identity.panel,
        identity.family,
        identity.arm,
        identity.lineage,
        generation,
        sibling,
    );
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut genome = parent.clone();
    let reachable = mesh_reachable_nodes(parent);
    let executed: Vec<_> = parent
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| reading.dispatched.contains(&node.node_id).then_some(index))
        .collect();
    let mut diagnostic =
        RefinementDiagnostic::new(identity.arm == Arm::M, seed ^ 0x7c44_3dca_c7ab_9285);
    let config = identity.arm.config();
    let start = Instant::now();
    let summary = MutationEngine::apply_mutations_observed(
        &mut genome,
        parent.genome_size(),
        &config,
        &reachable,
        ParentExecuted::Indices(&executed),
        &mut rng,
        2,
        observed.then_some(&mut diagnostic),
    );
    let mutation_secs = start.elapsed().as_secs_f64();
    let rng_after = rng.clone().next_u64();
    let child_reading = if genome == *parent {
        reading.clone()
    } else {
        panel.evaluate(&genome)
    };
    let qualified = qualifies_score(&child_reading, &panel.baseline)
        .then(|| checkpoint(&genome, panel, child_reading.clone()));
    let reference = FAMILIES[identity.family].reference();
    let InputReference::World(key) = reference else {
        unreachable!()
    };
    let focal_connected = super::super::input_use::connected_channels(&genome).iter().filter(|channel| (0..key.compound_width()).any(|sub| matches!(super::super::input_use::catalog::addressed(&reference, sub), super::super::input_use::catalog::Addressed::Channel(candidate) if candidate == **channel))).count();
    let record = Proposal {
        identity,
        generation,
        sibling,
        seed,
        chosen: false,
        parent_digest: digest(parent),
        digest: digest(&genome),
        rng_after,
        units: parent.genome_size(),
        requested: summary.attempted_events,
        applied: summary.applied_events,
        skipped: summary.skipped_events,
        events: summary.events.iter().map(Event::from).collect(),
        access: diagnostic.access,
        refinement: diagnostic.steps,
        eligible_focal_groups: eligible_focal_groups(parent, key),
        focal_declarations: genome
            .nodes
            .iter()
            .filter(|node| node.input_refs.contains(&reference))
            .count(),
        focal_connected,
        outcome: Outcome::from(&child_reading),
        training_qualified: qualified.as_ref().is_some_and(|q| q.graph_discovery),
        training_vm_qualified: qualified.as_ref().is_some_and(|q| q.vm_discovery),
        mutation_secs,
    };
    Child {
        genome,
        reading: child_reading,
        record,
        qualified,
    }
}

/// One independent lineage. `emit` checks the byte/wall cap before each record;
/// `stop` checks the shared wall budget between generations. A capped lineage is
/// returned as incomplete and cannot contribute a positive result.
#[allow(clippy::too_many_arguments)]
pub fn lineage(
    founder: &CreatureGenome,
    training: &Panel,
    held_out: &Panel,
    identity: Identity,
    generations: u32,
    emit: &mut impl FnMut(Record) -> bool,
    stop: &impl Fn() -> bool,
) -> Lineage {
    let mut parent = founder.clone();
    let mut reading = training.baseline.clone();
    let mut first: Option<Frozen> = None;
    let mut result = Lineage {
        identity,
        generations: 0,
        proposals: 0,
        requested: 0,
        applied: 0,
        skipped: 0,
        focal_access_events: 0,
        focal_structured_events: 0,
        chosen_focal_structured_events: 0,
        eligible_proposals: 0,
        proposal_discoveries: 0,
        vm_proposal_discoveries: 0,
        endpoint_graph_discovery: false,
        endpoint_vm_discovery: false,
        first_training_generation: None,
        primary_graph_discovery: false,
        primary_vm_discovery: false,
        primary_joint_effect: false,
        endpoint_digest: digest(&parent),
        endpoint_size: parent.genome_size(),
        right_censored: false,
        terminal: "completed".into(),
        complete: true,
    };
    let reference = FAMILIES[identity.family].reference();
    for generation in 0..generations {
        if stop() {
            result.complete = false;
            result.terminal = "wall_cap_partial_lineage".into();
            break;
        }
        let mut children = [
            propose(&parent, &reading, training, identity, generation, 0, true),
            propose(&parent, &reading, training, identity, generation, 1, true),
        ];
        let selected = choose(&reading, [&children[0].reading, &children[1].reading]);
        for (index, child) in children.iter_mut().enumerate() {
            child.record.chosen = selected == Some(index);
            result.proposals += 1;
            result.requested += child.record.requested;
            result.applied += child.record.applied;
            result.skipped += child.record.skipped;
            result.focal_access_events += child
                .record
                .access
                .iter()
                .filter(|step| step.channels.iter().any(|(input, _)| *input == reference))
                .count() as u32;
            result.focal_structured_events += child
                .record
                .refinement
                .iter()
                .filter(|step| step.applied && InputReference::World(step.family) == reference)
                .count() as u32;
            result.eligible_proposals += u32::from(child.record.eligible_focal_groups > 0);
            result.proposal_discoveries += u32::from(child.record.training_qualified);
            result.vm_proposal_discoveries += u32::from(child.record.training_vm_qualified);
            if !emit(Record::Proposal(child.record.clone())) {
                result.complete = false;
                result.terminal = "record_cap_partial_lineage".into();
            }
        }
        if !result.complete {
            break;
        }
        if let Some(index) = selected {
            let child = &mut children[index];
            result.chosen_focal_structured_events += child
                .record
                .refinement
                .iter()
                .filter(|step| step.applied && InputReference::World(step.family) == reference)
                .count() as u32;
            if first.is_none()
                && primary_eligible(
                    child.record.training_qualified,
                    identity.arm,
                    result.chosen_focal_structured_events,
                )
            {
                result.first_training_generation = Some(generation);
                first = Some(Frozen {
                    chosen_focal_structured_events: result.chosen_focal_structured_events,
                    identity,
                    generation,
                    seed: Some(child.record.seed),
                    role: "first_training_qualified_primary".into(),
                    digest: child.record.digest.clone(),
                    genotype: child.genome.clone(),
                    training: child.qualified.take(),
                    held_out: None,
                });
            }
            parent = child.genome.clone();
            reading = child.reading.clone();
        }
        result.generations = generation + 1;
    }
    finish_lineage(
        parent, reading, first, result, training, held_out, emit, stop,
    )
}

#[allow(clippy::too_many_arguments)]
fn finish_lineage(
    parent: CreatureGenome,
    reading: Reading,
    mut first: Option<Frozen>,
    mut result: Lineage,
    training: &Panel,
    held_out: &Panel,
    emit: &mut impl FnMut(Record) -> bool,
    stop: &impl Fn() -> bool,
) -> Lineage {
    let identity = result.identity;
    result.endpoint_digest = digest(&parent);
    result.endpoint_size = parent.genome_size();
    // Freeze primary and endpoint before any held-out candidate evaluation.
    if stop() {
        result.complete = false;
        result.terminal = "wall_cap_before_checkpoint".into();
    }
    let training_checkpoint = result
        .complete
        .then(|| checkpoint(&parent, training, reading));
    let mut endpoint = Frozen {
        chosen_focal_structured_events: result.chosen_focal_structured_events,
        identity,
        generation: result.generations,
        seed: None,
        role: "endpoint_diagnostic_only".into(),
        digest: result.endpoint_digest.clone(),
        genotype: parent,
        training: training_checkpoint,
        held_out: None,
    };
    if result.complete && !stop() {
        if let Some(candidate) = &mut first {
            let reading = held_out.evaluate(&candidate.genotype);
            let validation = checkpoint(&candidate.genotype, held_out, reading);
            let structured = identity.arm != Arm::C || candidate.chosen_focal_structured_events > 0;
            let training = candidate
                .training
                .as_ref()
                .expect("primary has training qualification");
            result.primary_graph_discovery =
                training.graph_discovery && validation.graph_discovery && structured;
            result.primary_vm_discovery = training.vm_discovery && validation.vm_discovery;
            result.primary_joint_effect = training.joint_effect && validation.joint_effect;
            candidate.held_out = Some(validation);
        }
        if !stop() {
            let reading = held_out.evaluate(&endpoint.genotype);
            let validation = checkpoint(&endpoint.genotype, held_out, reading);
            let training = endpoint
                .training
                .as_ref()
                .expect("complete endpoint has training qualification");
            result.endpoint_graph_discovery =
                training.graph_discovery && validation.graph_discovery;
            result.endpoint_vm_discovery = training.vm_discovery && validation.vm_discovery;
            endpoint.held_out = Some(validation);
        }
    }
    if stop() {
        result.complete = false;
        result.terminal = "wall_cap_during_checkpoint".into();
    }
    if let Some(candidate) = first {
        if !emit(Record::Frozen(Box::new(candidate))) {
            result.complete = false;
            result.terminal = "record_cap_checkpoint".into();
        }
    }
    if !emit(Record::Frozen(Box::new(endpoint))) {
        result.complete = false;
        result.terminal = "record_cap_checkpoint".into();
    }
    result.right_censored = result.complete && result.first_training_generation.is_none();
    if result.complete && result.first_training_generation.is_none() {
        result.terminal =
            if result.eligible_proposals == 0 && matches!(identity.arm, Arm::C | Arm::M) {
                "no_focal_structured_eligibility"
            } else if result.focal_access_events == 0 && matches!(identity.arm, Arm::S | Arm::W) {
                "no_selected_focal_access_event"
            } else {
                "no_training_causal_discovery"
            }
            .into();
    }
    if !result.complete {
        result.primary_graph_discovery = false;
        result.primary_vm_discovery = false;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn seed_coordinates_do_not_overlap(panel in 0..2u32, family in 0..3usize, arm in 0..5usize, lineage in 0..32u32, generation in 0..256u32, sibling in 0..2u8) {
            let value = seed(panel, family, Arm::ALL[arm], lineage, generation, sibling);
            let mut rest = value - 20_090_000_000;
            prop_assert_eq!(rest / 1_000_000_000, u64::from(panel)); rest %= 1_000_000_000;
            prop_assert_eq!(rest / 100_000_000, family as u64); rest %= 100_000_000;
            prop_assert_eq!(rest / 1_000_000, arm as u64); rest %= 1_000_000;
            prop_assert_eq!(rest / 10_000, u64::from(lineage)); rest %= 10_000;
            prop_assert_eq!(rest / 2, u64::from(generation)); prop_assert_eq!(rest % 2, u64::from(sibling));
        }
        #[test]
        fn neutral_siblings_precede_parent_and_damage_never_selected(score in 0..9u32) {
            let parent = Reading { correct: score, opportunities: 8, ..Reading::default() };
            prop_assert_eq!(choose(&parent, [&parent, &parent]), Some(0));
            let bad = Reading { alive: false, correct: 8, opportunities: 8, ..Reading::default() };
            prop_assert_eq!(choose(&parent, [&bad, &bad]), None);
        }
    }
    proptest! {
        #[test]
        fn vm_only_never_consumes_graph_primary_and_c_requires_chosen_ancestry(events in 0..100u32) {
            for arm in Arm::ALL { prop_assert!(!primary_eligible(false, arm, events)); }
            prop_assert!(primary_eligible(true, Arm::S, events));
            prop_assert_eq!(primary_eligible(true, Arm::C, events), events > 0);
        }
    }
    #[test]
    fn observing_native_births_does_not_change_genomes_or_rng() {
        let config = super::super::scene_config();
        let founder = crate::creature::founder::founder_genome_with_age_gate(
            config.population.founder_profile,
            &config.energy.lifecycle,
        );
        let panel = Panel::new(super::super::Family::Scalar, false, &founder);
        for arm in [Arm::B, Arm::S, Arm::W, Arm::C] {
            let identity = Identity {
                panel: 0,
                family: 0,
                arm,
                lineage: 0,
            };
            for generation in 0..4 {
                let a = propose(
                    &founder,
                    &panel.baseline,
                    &panel,
                    identity,
                    generation,
                    0,
                    true,
                );
                let b = propose(
                    &founder,
                    &panel.baseline,
                    &panel,
                    identity,
                    generation,
                    0,
                    false,
                );
                assert_eq!(a.genome, b.genome);
                assert_eq!(a.record.rng_after, b.record.rng_after);
                assert_eq!(a.record.events, b.record.events);
            }
        }
    }

    #[test]
    fn reduced_chosen_chain_replays_and_partial_lineages_cannot_qualify() {
        let config = super::super::scene_config();
        let founder = crate::creature::founder::founder_genome_with_age_gate(
            config.population.founder_profile,
            &config.energy.lifecycle,
        );
        let training = Panel::new(super::super::Family::Scalar, false, &founder);
        let held_out = Panel::new(super::super::Family::Scalar, true, &founder);
        let identity = Identity {
            panel: 0,
            family: 0,
            arm: Arm::S,
            lineage: 0,
        };
        let run = |stop| {
            let mut records = Vec::new();
            let row = lineage(
                &founder,
                &training,
                &held_out,
                identity,
                3,
                &mut |record| {
                    let mut value = serde_json::to_value(record).unwrap();
                    if value["kind"] == "proposal" {
                        value["record"]
                            .as_object_mut()
                            .unwrap()
                            .remove("mutation_secs");
                    }
                    records.push(value);
                    true
                },
                &|| stop,
            );
            (serde_json::to_value(row).unwrap(), records)
        };
        let first = run(false);
        assert_eq!(first, run(false));
        assert_eq!(first.0["proposals"], 6);
        let partial = run(true);
        assert_eq!(partial.0["proposals"], 0);
        assert_eq!(partial.0["complete"], false);
        assert_eq!(partial.0["right_censored"], false);
        assert_eq!(partial.0["primary_graph_discovery"], false);
    }
}
