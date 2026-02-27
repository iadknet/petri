//! Hebbian-specific mutation operators for graph internal nodes.
//!
//! Five operators that evolve per-node Hebbian learning parameters:
//! - EnableHebbian: add HebbianConfig to a non-Hebbian node
//! - DisableHebbian: remove HebbianConfig from a Hebbian node
//! - MutateHebbianRule: change the learning rule variant
//! - MutateHebbianRate: perturb the learning rate
//! - ToggleHebbianLamarckian: flip the inheritance flag

use rand::Rng;

use crate::creature::genome::{BackendDef, CreatureGenome, HebbianConfig, HebbianRule};
use crate::mutation::types::MutationSkipReason;

const ALL_RULES: [HebbianRule; 4] = [
    HebbianRule::Classic,
    HebbianRule::Oja,
    HebbianRule::AntiHebb,
    HebbianRule::Covariance,
];

/// Add HebbianConfig to a random non-Hebbian internal node.
pub fn enable_hebbian(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        // Two-pass: count eligible, then select.
        let eligible_count = g
            .internal_nodes
            .iter()
            .filter(|n| n.hebbian.is_none() && !n.inputs.is_empty())
            .count();
        if eligible_count == 0 {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        let target = rng.gen_range(0..eligible_count);
        let int_idx = g
            .internal_nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.hebbian.is_none() && !n.inputs.is_empty())
            .nth(target)
            .map(|(i, _)| i)
            .expect("target within counted range");

        let rule = ALL_RULES[rng.gen_range(0..ALL_RULES.len())];
        g.internal_nodes[int_idx].hebbian = Some(HebbianConfig {
            rule,
            learning_rate: rng.gen_range(0.01f32..0.2),
            weight_clamp: rng.gen_range(1.0f32..5.0),
            lamarckian: rng.gen_bool(0.5),
        });
    }
    Ok(())
}

/// Remove HebbianConfig from a random Hebbian internal node.
pub fn disable_hebbian(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        let eligible_count = g
            .internal_nodes
            .iter()
            .filter(|n| n.hebbian.is_some())
            .count();
        if eligible_count == 0 {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        let target = rng.gen_range(0..eligible_count);
        let int_idx = g
            .internal_nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.hebbian.is_some())
            .nth(target)
            .map(|(i, _)| i)
            .expect("target within counted range");

        g.internal_nodes[int_idx].hebbian = None;
    }
    Ok(())
}

/// Change the HebbianRule variant on a random Hebbian internal node.
pub fn mutate_hebbian_rule(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        let eligible_count = g
            .internal_nodes
            .iter()
            .filter(|n| n.hebbian.is_some())
            .count();
        if eligible_count == 0 {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        let target = rng.gen_range(0..eligible_count);
        let int_idx = g
            .internal_nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.hebbian.is_some())
            .nth(target)
            .map(|(i, _)| i)
            .expect("target within counted range");

        if let Some(ref mut cfg) = g.internal_nodes[int_idx].hebbian {
            // Pick a different rule.
            let others: Vec<HebbianRule> = ALL_RULES
                .iter()
                .copied()
                .filter(|r| *r != cfg.rule)
                .collect();
            if !others.is_empty() {
                cfg.rule = others[rng.gen_range(0..others.len())];
            }
        }
    }
    Ok(())
}

/// Perturb the learning_rate on a random Hebbian internal node.
///
/// Uses ~30% multiplicative perturbation when rate is above a threshold,
/// otherwise a small additive perturbation.
pub fn mutate_hebbian_rate(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        let eligible_count = g
            .internal_nodes
            .iter()
            .filter(|n| n.hebbian.is_some())
            .count();
        if eligible_count == 0 {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        let target = rng.gen_range(0..eligible_count);
        let int_idx = g
            .internal_nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.hebbian.is_some())
            .nth(target)
            .map(|(i, _)| i)
            .expect("target within counted range");

        if let Some(ref mut cfg) = g.internal_nodes[int_idx].hebbian {
            if cfg.learning_rate.abs() > 0.01 {
                cfg.learning_rate *= 1.0 + rng.gen_range(-0.3f32..=0.3);
            } else {
                cfg.learning_rate += rng.gen_range(-0.05f32..=0.05);
            }
            cfg.learning_rate = cfg.learning_rate.clamp(0.0, 1.0);
        }
    }
    Ok(())
}

/// Flip the `lamarckian` flag on a random Hebbian internal node.
pub fn toggle_hebbian_lamarckian(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        let eligible_count = g
            .internal_nodes
            .iter()
            .filter(|n| n.hebbian.is_some())
            .count();
        if eligible_count == 0 {
            return Err(MutationSkipReason::NoApplicableTarget);
        }

        let target = rng.gen_range(0..eligible_count);
        let int_idx = g
            .internal_nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.hebbian.is_some())
            .nth(target)
            .map(|(i, _)| i)
            .expect("target within counted range");

        if let Some(ref mut cfg) = g.internal_nodes[int_idx].hebbian {
            cfg.lamarckian = !cfg.lamarckian;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::NodeId;
    use crate::creature::genome::{
        GraphBackendDef, GraphInput, GraphInternalNode, GraphNodeKind, NodeGenome,
    };
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn rng(seed: u64) -> SmallRng {
        SmallRng::seed_from_u64(seed)
    }

    fn genome_with_graph(nodes: Vec<GraphInternalNode>) -> CreatureGenome {
        CreatureGenome {
            entry_node_id: NodeId::new(0),
            nodes: vec![NodeGenome {
                node_id: NodeId::new(0),
                input_refs: vec![],
                backend_def: BackendDef::Graph(GraphBackendDef {
                    internal_nodes: nodes,
                }),
                targets: vec![],
            }],
        }
    }

    fn node_without_hebbian() -> GraphInternalNode {
        GraphInternalNode {
            kind: GraphNodeKind::Add,
            inputs: vec![GraphInput {
                source_idx: 0,
                weight: 1.0,
            }],
            hebbian: None,
        }
    }

    fn node_with_hebbian() -> GraphInternalNode {
        GraphInternalNode {
            kind: GraphNodeKind::Add,
            inputs: vec![GraphInput {
                source_idx: 0,
                weight: 1.0,
            }],
            hebbian: Some(HebbianConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: false,
            }),
        }
    }

    #[test]
    fn enable_hebbian_adds_config() {
        let mut genome = genome_with_graph(vec![node_without_hebbian()]);
        let mut r = rng(42);
        let result = enable_hebbian(&mut genome, 0, &mut r);
        assert!(result.is_ok());

        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            assert!(g.internal_nodes[0].hebbian.is_some());
        } else {
            panic!("expected Graph backend");
        }
    }

    #[test]
    fn enable_hebbian_skips_when_all_already_hebbian() {
        let mut genome = genome_with_graph(vec![node_with_hebbian()]);
        let mut r = rng(42);
        let result = enable_hebbian(&mut genome, 0, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    #[test]
    fn disable_hebbian_removes_config() {
        let mut genome = genome_with_graph(vec![node_with_hebbian()]);
        let mut r = rng(42);
        let result = disable_hebbian(&mut genome, 0, &mut r);
        assert!(result.is_ok());

        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            assert!(g.internal_nodes[0].hebbian.is_none());
        } else {
            panic!("expected Graph backend");
        }
    }

    #[test]
    fn disable_hebbian_skips_when_none_hebbian() {
        let mut genome = genome_with_graph(vec![node_without_hebbian()]);
        let mut r = rng(42);
        let result = disable_hebbian(&mut genome, 0, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    #[test]
    fn mutate_rule_changes_variant() {
        let mut genome = genome_with_graph(vec![node_with_hebbian()]);
        let original_rule = HebbianRule::Classic;
        let mut r = rng(42);
        let result = mutate_hebbian_rule(&mut genome, 0, &mut r);
        assert!(result.is_ok());

        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            let new_rule = g.internal_nodes[0].hebbian.as_ref().unwrap().rule;
            assert_ne!(new_rule, original_rule);
        }
    }

    #[test]
    fn mutate_rate_stays_in_bounds() {
        let mut genome = genome_with_graph(vec![node_with_hebbian()]);
        for seed in 0u64..100 {
            let mut r = rng(seed);
            mutate_hebbian_rate(&mut genome, 0, &mut r).unwrap();

            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                let rate = g.internal_nodes[0].hebbian.as_ref().unwrap().learning_rate;
                assert!(
                    (0.0..=1.0).contains(&rate),
                    "rate {rate} out of bounds at seed {seed}"
                );
            }
        }
    }

    #[test]
    fn toggle_lamarckian_flips_flag() {
        let mut genome = genome_with_graph(vec![node_with_hebbian()]);
        let mut r = rng(42);

        // Initially false
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            assert!(!g.internal_nodes[0].hebbian.as_ref().unwrap().lamarckian);
        }

        toggle_hebbian_lamarckian(&mut genome, 0, &mut r).unwrap();

        // Now true
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            assert!(g.internal_nodes[0].hebbian.as_ref().unwrap().lamarckian);
        }

        toggle_hebbian_lamarckian(&mut genome, 0, &mut r).unwrap();

        // Back to false
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            assert!(!g.internal_nodes[0].hebbian.as_ref().unwrap().lamarckian);
        }
    }

    #[test]
    fn toggle_lamarckian_skips_when_no_hebbian() {
        let mut genome = genome_with_graph(vec![node_without_hebbian()]);
        let mut r = rng(42);
        let result = toggle_hebbian_lamarckian(&mut genome, 0, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    #[test]
    fn enable_hebbian_skips_nodes_without_inputs() {
        // A node with no inputs should not get Hebbian enabled.
        let node = GraphInternalNode {
            kind: GraphNodeKind::Constant(1.0),
            inputs: vec![],
            hebbian: None,
        };
        let mut genome = genome_with_graph(vec![node]);
        let mut r = rng(42);
        let result = enable_hebbian(&mut genome, 0, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }
}
