//! Compact evidence projection; internal scene vectors stay in the evaluator.
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use v3_core::neighborhood::input_discovery::{
    Arm, Frozen, Identity, Lineage, Panel, Proposal, Qualification, Reading,
};

pub(super) const SCHEMA: &str = "input-discovery-compact-v2";
pub(super) const AUDIT_BYTES: usize = 16 * 1024;

#[derive(Default, Serialize)]
struct Costs {
    carrying: f64,
    learning_charge: f64,
    plasticity_updates: u64,
    graph_charge: f64,
    vm_charge: f64,
    mesh_charge: f64,
    action_charge: f64,
    graph_visits: u64,
    vm_steps: u64,
    mesh_hops: u64,
    perception_assembled_scenes: u64,
}

pub(super) fn reading(value: &Reading) -> Value {
    let mut costs = Costs::default();
    let mut food = Vec::<f64>::new();
    let mut energy = 0.0;
    let mut energy_scenes = 0;
    for scene in &value.scenes {
        costs.carrying += scene.carrying;
        costs.learning_charge += scene.learning_charge;
        costs.plasticity_updates += scene.plasticity_updates;
        costs.graph_charge += scene.graph_charge;
        costs.vm_charge += scene.vm_charge;
        costs.mesh_charge += scene.mesh_charge;
        costs.action_charge += scene.action_charge;
        costs.graph_visits += scene.graph_visits;
        costs.vm_steps += scene.vm_steps;
        costs.mesh_hops += scene.mesh_hops;
        costs.perception_assembled_scenes += u64::from(scene.perception_assembled);
        food.resize(food.len().max(scene.food_intake.len()), 0.0);
        for (total, amount) in food.iter_mut().zip(&scene.food_intake) {
            *total += amount;
        }
        if let Some(after) = scene.energy {
            energy += f64::from(after);
            energy_scenes += 1;
        }
    }
    json!({"correct":value.correct,"opportunities":value.opportunities,
        "incumbent_preserved":value.incumbent_preserved,"incumbent_scenes":value.incumbent_scenes,
        "alive":value.alive,"alive_scenes":value.scenes.iter().filter(|scene|scene.alive).count(),
        "scene_count":value.scenes.len(),"costs":costs,"food_intake":food,
        "ending_energy_sum":(energy_scenes > 0).then_some(energy),"ending_energy_scenes":energy_scenes})
}

pub(super) fn qualification(value: &Qualification) -> Value {
    let intact = reading(&value.reading);
    let ablation = |value: &Reading| {
        let facts = reading(value);
        if facts == intact {
            json!({"same_as":"intact"})
        } else {
            facts
        }
    };
    json!({"learning_mask_removed":value.learning_mask_removed,"inherited_size":value.inherited_size,
        "expressed_size":value.expressed_size,"intact":intact,
        "all_backend_ablated":ablation(&value.all_backend_ablated),"graph_ablated":ablation(&value.graph_ablated),
        "vm_ablated":ablation(&value.vm_ablated),"channels":value.channels,"coverage":value.coverage,
        "graph_discovery":value.graph_discovery,"vm_discovery":value.vm_discovery,"joint_effect":value.joint_effect})
}

pub(super) fn panel(value: &Panel) -> Value {
    json!({"family":value.family,"held_out":value.held_out,"scenes":value.scenes,
        "focal":value.focal,"baseline":reading(&value.baseline)})
}

fn frozen(value: &Frozen) -> Value {
    json!({"identity":value.identity,"generation":value.generation,"seed":value.seed,"role":value.role,
        "digest":value.digest,"genome_size":value.genotype.genome_size(),
        "chosen_focal_structured_events":value.chosen_focal_structured_events,
        "training":value.training.as_ref().map(qualification),"held_out":value.held_out.as_ref().map(qualification)})
}

/// Fixed projection: timing and host state cannot enter the deterministic transcript.
fn proposal(value: &Proposal) -> Value {
    json!({"identity":value.identity,"generation":value.generation,"sibling":value.sibling,
        "seed":value.seed,"chosen":value.chosen,"parent_digest":value.parent_digest,"digest":value.digest,
        "rng_after":value.rng_after,"units":value.units,"requested":value.requested,"applied":value.applied,
        "skipped":value.skipped,"events":value.events,"access":value.access,"refinement":value.refinement,
        "eligible_focal_groups":value.eligible_focal_groups,"focal_declarations":value.focal_declarations,
        "focal_connected":value.focal_connected,"outcome":value.outcome,
        "training_qualified":value.training_qualified,"training_vm_qualified":value.training_vm_qualified})
}

fn canonical(value: Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(&v3_core::config::sort_json_keys_recursive(value))
        .expect("finite native evidence serializes");
    bytes.push(b'\n');
    bytes
}

#[derive(Default, Serialize)]
struct Refinements {
    observed: u64,
    applied: u64,
    requested_norm_max: Option<f64>,
    actual_norm_max: Option<f64>,
    coefficient_count: u64,
    changed_coefficient_count: u64,
}

#[derive(Default)]
pub(super) struct LineageEvidence {
    transcript: Sha256,
    proposals: u64,
    mutation_secs: f64,
    operators: BTreeMap<String, u64>,
    discarded: BTreeMap<String, u64>,
    refinements: Refinements,
    primary: Option<Value>,
    endpoint: Option<Value>,
}

impl LineageEvidence {
    /// Returns only the prospectively selected, bounded audit witness, if any.
    pub(super) fn observe(&mut self, value: &Proposal) -> Option<Value> {
        let projected = proposal(value);
        let bytes = canonical(projected.clone());
        self.transcript.update(&bytes);
        self.proposals += 1;
        self.mutation_secs += value.mutation_secs;
        for event in &value.events {
            *self
                .operators
                .entry(format!(
                    "{:?}/{:?}/{}",
                    event.domain, event.operator, event.outcome
                ))
                .or_default() += 1;
            for (operator, _) in &event.discarded {
                *self.discarded.entry(format!("{operator:?}")).or_default() += 1;
            }
        }
        for step in &value.refinement {
            self.refinements.observed += 1;
            self.refinements.applied += u64::from(step.applied);
            let requested = step
                .coefficients
                .iter()
                .map(|coefficient| f64::from(coefficient.requested).powi(2))
                .sum::<f64>()
                .sqrt();
            self.refinements.requested_norm_max = Some(
                self.refinements
                    .requested_norm_max
                    .map_or(requested, |old| old.max(requested)),
            );
            self.refinements.actual_norm_max = Some(
                self.refinements
                    .actual_norm_max
                    .map_or(step.actual_norm, |old| old.max(step.actual_norm)),
            );
            self.refinements.coefficient_count += step.coefficients.len() as u64;
            self.refinements.changed_coefficient_count += step
                .coefficients
                .iter()
                .filter(|coefficient| coefficient.before != coefficient.after)
                .count() as u64;
        }
        if value.identity.lineage != 0 || value.generation != 0 {
            return None;
        }
        let mut witness =
            json!({"kind":"audit_proposal","details_omitted":false,"proposal":projected});
        if canonical(witness.clone()).len() > AUDIT_BYTES {
            witness = json!({"kind":"audit_proposal","details_omitted":true,"reason":"witness_byte_bound",
                "identity":value.identity,"generation":value.generation,"sibling":value.sibling,"seed":value.seed,
                "digest":value.digest,"parent_digest":value.parent_digest,"chosen":value.chosen,
                "requested":value.requested,"applied":value.applied,"skipped":value.skipped,
                "proposal_sha256":format!("{:x}",Sha256::digest(&bytes)),"replay":"fixed source, manifest and identity"});
        }
        Some(witness)
    }

    pub(super) fn freeze(&mut self, value: &Frozen) {
        if value.role == "first_training_qualified_primary" {
            self.primary = Some(frozen(value));
        } else {
            self.endpoint = Some(frozen(value));
        }
    }

    pub(super) fn record(self, result: &Lineage) -> Value {
        json!({"kind":"lineage","result":result,"transcript":{"proposals":self.proposals,
            "sha256":format!("{:x}",self.transcript.finalize())},"mutation_secs":self.mutation_secs,
            "operators":self.operators,"discarded_operators":self.discarded,"refinement":self.refinements,
            "primary":self.primary,"endpoint":self.endpoint})
    }
}

#[derive(Serialize)]
pub(super) struct Representative {
    identity: Identity,
    generation: u32,
    seed: Option<u64>,
    digest: String,
    genotype: v3_core::creature::genome::CreatureGenome,
}

#[derive(Default)]
pub(super) struct Representatives([Option<Representative>; 5]);

impl Representatives {
    pub(super) fn observe(&mut self, value: &Frozen) {
        let slot = match (value.identity.arm, value.identity.family) {
            (Arm::S, family @ 0..=1) => family,
            (Arm::W, family @ 0..=1) => 2 + family,
            (Arm::C, 2) if value.chosen_focal_structured_events > 0 => 4,
            _ => return,
        };
        if value.role != "first_training_qualified_primary"
            || !value.training.as_ref().is_some_and(|q| q.graph_discovery)
            || !value.held_out.as_ref().is_some_and(|q| q.graph_discovery)
        {
            return;
        }
        let key = (
            value.identity.panel,
            value.identity.lineage,
            value.generation,
        );
        if self.0[slot]
            .as_ref()
            .is_none_or(|old| key < (old.identity.panel, old.identity.lineage, old.generation))
        {
            self.0[slot] = Some(Representative {
                identity: value.identity,
                generation: value.generation,
                seed: value.seed,
                digest: value.digest.clone(),
                genotype: value.genotype.clone(),
            });
        }
    }

    pub(super) fn qualified(
        &self,
        s: bool,
        w: bool,
        ring: bool,
    ) -> impl Iterator<Item = &Representative> {
        self.0.iter().enumerate().filter_map(move |(index, value)| {
            let eligible = match index {
                0 | 1 => s,
                2 | 3 => w,
                _ => w && ring,
            };
            value.as_ref().filter(|_| eligible)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use v3_core::neighborhood::input_discovery::{self as assay, Record, SceneReading};

    fn reduced_records() -> (Lineage, Vec<Record>) {
        let config = assay::scene_config();
        let founder = v3_core::creature::founder::founder_genome_with_age_gate(
            config.population.founder_profile,
            &config.energy.lifecycle,
        );
        let training = Panel::new(assay::Family::Scalar, false, &founder);
        let held_out = Panel::new(assay::Family::Scalar, true, &founder);
        let mut records = Vec::new();
        let result = assay::lineage(
            &founder,
            &training,
            &held_out,
            Identity {
                panel: 0,
                family: 0,
                arm: Arm::S,
                lineage: 0,
            },
            3,
            &mut |record| {
                records.push(record);
                true
            },
            &|| false,
        );
        (result, records)
    }

    fn project(result: &Lineage, records: &[Record]) -> Value {
        let mut facts = LineageEvidence::default();
        for record in records {
            match record {
                Record::Proposal(value) => {
                    facts.observe(value);
                }
                Record::Frozen(value) => facts.freeze(value),
                Record::Lineage(_) => unreachable!(),
            }
        }
        let mut value = facts.record(result);
        value.as_object_mut().unwrap().remove("mutation_secs");
        value
    }

    #[test]
    fn compact_replay_matches_native_results_transcript_and_frozen_genome() {
        let (first, records) = reduced_records();
        let (second, repeated) = reduced_records();
        let compact = project(&first, &records);
        assert_eq!(compact, project(&second, &repeated));
        assert_eq!(compact["result"], serde_json::to_value(&first).unwrap());
        assert_eq!(compact["transcript"]["proposals"], 6);
        let mut transcript = Sha256::new();
        for record in &records {
            match record {
                Record::Proposal(value) => {
                    // Independently build the original deterministic record projection.
                    let mut original = serde_json::to_value(value).unwrap();
                    original.as_object_mut().unwrap().remove("mutation_secs");
                    transcript.update(canonical(original));
                }
                Record::Frozen(value) => {
                    assert_eq!(value.digest, assay::digest(&value.genotype));
                    assert_eq!(compact["endpoint"]["digest"], value.digest);
                    assert_eq!(
                        compact["endpoint"]["training"]["intact"]["correct"],
                        value.training.as_ref().unwrap().reading.correct
                    );
                    assert!(compact["endpoint"].get("genotype").is_none());
                }
                Record::Lineage(_) => unreachable!(),
            }
        }
        assert_eq!(
            compact["transcript"]["sha256"],
            format!("{:x}", transcript.finalize())
        );
        assert!(serde_json::to_vec(&compact).unwrap().len() < 24 * 1024);
        println!(
            "REDUCED_REPLAY transcript={} endpoint={} compact_bytes={}",
            compact["transcript"]["sha256"],
            compact["endpoint"]["digest"],
            serde_json::to_vec(&compact).unwrap().len()
        );
    }

    #[test]
    fn transcript_excludes_timing_but_commits_selection_and_bounded_audit_is_explicit() {
        let (_, records) = reduced_records();
        let Record::Proposal(original) = &records[0] else {
            panic!("first proposal")
        };
        let mut runner = proptest::test_runner::TestRunner::new(proptest::test_runner::Config {
            source_file: Some(file!()),
            ..proptest::test_runner::Config::default()
        });
        runner
            .run(&(0.0f64..1_000_000.0), |timing| {
                let mut candidate = original.clone();
                candidate.mutation_secs = timing;
                prop_assert_eq!(
                    canonical(proposal(&candidate)),
                    canonical(proposal(original))
                );
                candidate.chosen = !candidate.chosen;
                prop_assert_ne!(
                    canonical(proposal(&candidate)),
                    canonical(proposal(original))
                );
                Ok(())
            })
            .unwrap();
        let mut candidate = original.clone();
        candidate.events = vec![v3_core::neighborhood::recruitment_paths::Event {
            domain: v3_core::mutation::types::MutationDomain::Graph,
            operator: None,
            target: None,
            outcome: "x".repeat(AUDIT_BYTES),
            discarded: vec![],
        }];
        let witness = LineageEvidence::default().observe(&candidate).unwrap();
        assert_eq!(witness["details_omitted"], true);
        assert!(canonical(witness).len() <= AUDIT_BYTES);
        candidate.identity.lineage = 1;
        assert!(LineageEvidence::default().observe(&candidate).is_none());
    }

    #[test]
    fn genomes_are_only_lowest_successful_representatives_of_qualified_scopes() {
        let (_, records) = reduced_records();
        let mut frozen = records
            .into_iter()
            .find_map(|record| match record {
                Record::Frozen(value) => Some(*value),
                _ => None,
            })
            .unwrap();
        let mut representatives = Representatives::default();
        representatives.observe(&frozen);
        assert_eq!(representatives.qualified(true, true, true).count(), 0);
        frozen.role = "first_training_qualified_primary".into();
        frozen.training.as_mut().unwrap().graph_discovery = true;
        frozen.held_out.as_mut().unwrap().graph_discovery = true;
        for (arm, family) in [
            (Arm::S, 0),
            (Arm::S, 1),
            (Arm::W, 0),
            (Arm::W, 1),
            (Arm::C, 2),
        ] {
            frozen.identity.arm = arm;
            frozen.identity.family = family;
            frozen.chosen_focal_structured_events = 1;
            for lineage in [2, 0, 1] {
                frozen.identity.lineage = lineage;
                representatives.observe(&frozen);
            }
        }
        assert_eq!(representatives.qualified(false, false, false).count(), 0);
        assert_eq!(representatives.qualified(true, false, true).count(), 2);
        assert_eq!(representatives.qualified(false, true, false).count(), 2);
        let all: Vec<_> = representatives.qualified(true, true, true).collect();
        assert_eq!(all.len(), 5);
        assert!(all.iter().all(|row| row.identity.lineage == 0));
        for row in all {
            let compact = serde_json::to_value(row).unwrap();
            assert_eq!(compact["digest"], assay::digest(&row.genotype));
            assert_eq!(
                compact["genotype"],
                serde_json::to_value(&row.genotype).unwrap()
            );
            assert!(compact.get("training").is_none());
            assert!(compact.get("scenes").is_none());
        }
    }

    proptest! {
        #[test]
        fn native_cost_projection_sums_exact_integer_work_and_preserves_missing_energy(count in 0..64usize, visits in 0..1000u64) {
            let source = Reading { scenes:vec![SceneReading { graph_visits:visits,..SceneReading::default() };count],..Reading::default() };
            let compact = reading(&source);
            prop_assert_eq!(compact["costs"]["graph_visits"].as_u64(),Some(visits * count as u64));
            prop_assert_eq!(compact["scene_count"].as_u64(),Some(count as u64));
            prop_assert!(compact["ending_energy_sum"].is_null());
            prop_assert!(compact.get("scenes").is_none());
        }
    }

    #[test]
    fn checkpoint_projection_keeps_counts_and_native_costs_without_scene_arrays() {
        let source = Reading {
            correct: 2,
            opportunities: 3,
            incumbent_preserved: 7,
            incumbent_scenes: 7,
            scenes: vec![
                SceneReading {
                    alive: true,
                    carrying: 0.25,
                    graph_visits: 4,
                    graph_charge: 0.5,
                    action_charge: 0.125,
                    ..SceneReading::default()
                };
                10
            ],
            ..Reading::default()
        };
        let compact = reading(&source);
        assert!(compact.get("scenes").is_none());
        assert_eq!(compact["correct"], 2);
        assert_eq!(compact["opportunities"], 3);
        assert_eq!(compact["incumbent_preserved"], 7);
        assert_eq!(compact["costs"]["carrying"], 2.5);
        assert_eq!(compact["costs"]["graph_visits"], 40);
        assert_eq!(compact["costs"]["action_charge"], 1.25);
        assert_eq!(compact["alive_scenes"], 10);
        assert!(serde_json::to_vec(&compact).unwrap().len() < 2048);
    }

    #[test]
    fn only_exactly_equal_ablations_reference_intact_and_changed_costs_stay_explicit() {
        let (_, records) = reduced_records();
        let mut value = records
            .into_iter()
            .find_map(|record| match record {
                Record::Frozen(value) => value.training,
                _ => None,
            })
            .unwrap();
        value.graph_ablated = value.reading.clone();
        value.vm_ablated = value.reading.clone();
        value.vm_ablated.scenes[0].action_charge += 0.125;
        let compact = qualification(&value);
        assert_eq!(compact["graph_ablated"], json!({"same_as":"intact"}));
        assert_eq!(compact["vm_ablated"], reading(&value.vm_ablated));
        assert!(compact["intact"].get("applied_actions").is_none());
        assert_eq!(
            compact["channels"],
            serde_json::to_value(&value.channels).unwrap()
        );
        assert_eq!(compact["graph_discovery"], value.graph_discovery);
        assert_eq!(compact["vm_discovery"], value.vm_discovery);
    }
}
