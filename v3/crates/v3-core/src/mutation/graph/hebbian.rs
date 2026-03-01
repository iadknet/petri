//! Plasticity mutation operators for graph internal nodes.
//!
//! Nine operators that evolve per-node plasticity learning parameters:
//! - EnableHebbian: add PlasticityConfig to a non-plasticity node
//! - DisableHebbian: remove PlasticityConfig from a plasticity node
//! - MutateHebbianRule: change the learning rule variant
//! - MutateHebbianRate: perturb the learning rate
//! - ToggleHebbianLamarckian: flip the inheritance flag
//! - EnableRewardModulation: add reward modulation to a pure Hebbian node
//! - DisableRewardModulation: remove reward modulation from a modulated node
//! - MutateRewardSource: change the outcome channel a modulated node listens to
//! - MutateTraceDecay: perturb the trace decay rate on a modulated node

use rand::Rng;

use crate::creature::genome::{
    BackendDef, CreatureGenome, GraphInternalNode, HebbianRule, OutcomeChannel, PlasticityConfig,
    RewardModulationConfig,
};
use crate::mutation::types::MutationSkipReason;

const ALL_RULES: [HebbianRule; 4] = [
    HebbianRule::Classic,
    HebbianRule::Oja,
    HebbianRule::AntiHebb,
    HebbianRule::Covariance,
];

const ALL_CHANNELS: [OutcomeChannel; 4] = [
    OutcomeChannel::EnergyDelta,
    OutcomeChannel::ActionSuccess,
    OutcomeChannel::DamageDelta,
    OutcomeChannel::OffspringSuccess,
];

/// Pick a random internal node matching `predicate`. Returns its index or
/// `NoApplicableTarget` if none match. Uses two-pass count-then-select to
/// avoid allocating a Vec of eligible indices.
fn select_eligible(
    nodes: &[GraphInternalNode],
    predicate: impl Fn(&GraphInternalNode) -> bool,
    rng: &mut impl Rng,
) -> Result<usize, MutationSkipReason> {
    let count = nodes.iter().filter(|n| predicate(n)).count();
    if count == 0 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let target = rng.gen_range(0..count);
    nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| predicate(n))
        .nth(target)
        .map(|(i, _)| i)
        .ok_or(MutationSkipReason::NoApplicableTarget)
}

/// Add PlasticityConfig to a random non-plasticity internal node.
pub fn enable_hebbian(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        let int_idx = select_eligible(
            &g.internal_nodes,
            |n| n.plasticity.is_none() && !n.inputs.is_empty(),
            rng,
        )?;

        let rule = ALL_RULES[rng.gen_range(0..ALL_RULES.len())];
        g.internal_nodes[int_idx].plasticity = Some(PlasticityConfig {
            rule,
            learning_rate: rng.gen_range(0.01f32..0.2),
            weight_clamp: rng.gen_range(1.0f32..5.0),
            lamarckian: rng.gen_bool(0.5),
            modulation: None,
        });
    }
    Ok(())
}

/// Remove PlasticityConfig from a random plasticity internal node.
pub fn disable_hebbian(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        let int_idx = select_eligible(&g.internal_nodes, |n| n.plasticity.is_some(), rng)?;
        g.internal_nodes[int_idx].plasticity = None;
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
        let int_idx = select_eligible(&g.internal_nodes, |n| n.plasticity.is_some(), rng)?;

        if let Some(ref mut cfg) = g.internal_nodes[int_idx].plasticity {
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
        let int_idx = select_eligible(&g.internal_nodes, |n| n.plasticity.is_some(), rng)?;

        if let Some(ref mut cfg) = g.internal_nodes[int_idx].plasticity {
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
        let int_idx = select_eligible(&g.internal_nodes, |n| n.plasticity.is_some(), rng)?;

        if let Some(ref mut cfg) = g.internal_nodes[int_idx].plasticity {
            cfg.lamarckian = !cfg.lamarckian;
        }
    }
    Ok(())
}

/// Add reward modulation to a random plasticity node that has `modulation=None`.
pub fn enable_reward_modulation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        let int_idx = select_eligible(
            &g.internal_nodes,
            |n| {
                n.plasticity
                    .as_ref()
                    .is_some_and(|p| p.modulation.is_none())
            },
            rng,
        )?;

        if let Some(ref mut cfg) = g.internal_nodes[int_idx].plasticity {
            let channel = ALL_CHANNELS[rng.gen_range(0..ALL_CHANNELS.len())];
            cfg.modulation = Some(RewardModulationConfig {
                reward_source: channel,
                trace_decay: rng.gen_range(0.5f32..0.99),
            });
        }
    }
    Ok(())
}

/// Remove reward modulation from a random reward-modulated node.
pub fn disable_reward_modulation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        let int_idx = select_eligible(
            &g.internal_nodes,
            |n| {
                n.plasticity
                    .as_ref()
                    .is_some_and(|p| p.modulation.is_some())
            },
            rng,
        )?;

        if let Some(ref mut cfg) = g.internal_nodes[int_idx].plasticity {
            cfg.modulation = None;
        }
    }
    Ok(())
}

/// Switch the `reward_source` channel on a random reward-modulated node.
pub fn mutate_reward_source(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        let int_idx = select_eligible(
            &g.internal_nodes,
            |n| {
                n.plasticity
                    .as_ref()
                    .is_some_and(|p| p.modulation.is_some())
            },
            rng,
        )?;

        if let Some(ref mut cfg) = g.internal_nodes[int_idx].plasticity {
            if let Some(ref mut modulation) = cfg.modulation {
                let others: Vec<OutcomeChannel> = ALL_CHANNELS
                    .iter()
                    .copied()
                    .filter(|c| *c != modulation.reward_source)
                    .collect();
                if !others.is_empty() {
                    modulation.reward_source = others[rng.gen_range(0..others.len())];
                }
            }
        }
    }
    Ok(())
}

/// Perturb the `trace_decay` on a random reward-modulated node.
///
/// Uses ~30% multiplicative perturbation when decay is above a threshold,
/// otherwise a small additive perturbation. Clamped to [0.0, 1.0].
pub fn mutate_trace_decay(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    if let BackendDef::Graph(ref mut g) = genome.nodes[node_idx].backend_def {
        let int_idx = select_eligible(
            &g.internal_nodes,
            |n| {
                n.plasticity
                    .as_ref()
                    .is_some_and(|p| p.modulation.is_some())
            },
            rng,
        )?;

        if let Some(ref mut cfg) = g.internal_nodes[int_idx].plasticity {
            if let Some(ref mut modulation) = cfg.modulation {
                if modulation.trace_decay.abs() > 0.01 {
                    modulation.trace_decay *= 1.0 + rng.gen_range(-0.3f32..=0.3);
                } else {
                    modulation.trace_decay += rng.gen_range(-0.05f32..=0.05);
                }
                modulation.trace_decay = modulation.trace_decay.clamp(0.0, 1.0);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::NodeId;
    use crate::creature::genome::{
        GraphBackendDef, GraphInput, GraphInternalNode, GraphNodeKind, NodeGenome, OutcomeChannel,
        RewardModulationConfig,
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
            plasticity: None,
        }
    }

    fn node_with_hebbian() -> GraphInternalNode {
        GraphInternalNode {
            kind: GraphNodeKind::Add,
            inputs: vec![GraphInput {
                source_idx: 0,
                weight: 1.0,
            }],
            plasticity: Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: false,
                modulation: None,
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
            assert!(g.internal_nodes[0].plasticity.is_some());
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
            assert!(g.internal_nodes[0].plasticity.is_none());
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
            let new_rule = g.internal_nodes[0].plasticity.as_ref().unwrap().rule;
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
                let rate = g.internal_nodes[0]
                    .plasticity
                    .as_ref()
                    .unwrap()
                    .learning_rate;
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
            assert!(!g.internal_nodes[0].plasticity.as_ref().unwrap().lamarckian);
        }

        toggle_hebbian_lamarckian(&mut genome, 0, &mut r).unwrap();

        // Now true
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            assert!(g.internal_nodes[0].plasticity.as_ref().unwrap().lamarckian);
        }

        toggle_hebbian_lamarckian(&mut genome, 0, &mut r).unwrap();

        // Back to false
        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            assert!(!g.internal_nodes[0].plasticity.as_ref().unwrap().lamarckian);
        }
    }

    #[test]
    fn toggle_lamarckian_skips_when_no_hebbian() {
        let mut genome = genome_with_graph(vec![node_without_hebbian()]);
        let mut r = rng(42);
        let result = toggle_hebbian_lamarckian(&mut genome, 0, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    fn node_with_reward_modulation() -> GraphInternalNode {
        GraphInternalNode {
            kind: GraphNodeKind::Add,
            inputs: vec![GraphInput {
                source_idx: 0,
                weight: 1.0,
            }],
            plasticity: Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: false,
                modulation: Some(RewardModulationConfig {
                    reward_source: OutcomeChannel::EnergyDelta,
                    trace_decay: 0.9,
                }),
            }),
        }
    }

    #[test]
    fn enable_reward_modulation_adds_config() {
        let mut genome = genome_with_graph(vec![node_with_hebbian()]);
        let mut r = rng(42);
        let result = enable_reward_modulation(&mut genome, 0, &mut r);
        assert!(result.is_ok());

        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            let cfg = g.internal_nodes[0].plasticity.as_ref().unwrap();
            assert!(cfg.modulation.is_some());
        }
    }

    #[test]
    fn enable_reward_modulation_skips_already_modulated() {
        let mut genome = genome_with_graph(vec![node_with_reward_modulation()]);
        let mut r = rng(42);
        let result = enable_reward_modulation(&mut genome, 0, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    #[test]
    fn disable_reward_modulation_removes_config() {
        let mut genome = genome_with_graph(vec![node_with_reward_modulation()]);
        let mut r = rng(42);
        let result = disable_reward_modulation(&mut genome, 0, &mut r);
        assert!(result.is_ok());

        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            let cfg = g.internal_nodes[0].plasticity.as_ref().unwrap();
            assert!(cfg.modulation.is_none());
        }
    }

    #[test]
    fn disable_reward_modulation_skips_pure_hebbian() {
        let mut genome = genome_with_graph(vec![node_with_hebbian()]);
        let mut r = rng(42);
        let result = disable_reward_modulation(&mut genome, 0, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }

    #[test]
    fn mutate_reward_source_changes_channel() {
        let mut genome = genome_with_graph(vec![node_with_reward_modulation()]);
        let original = OutcomeChannel::EnergyDelta;
        let mut r = rng(42);
        let result = mutate_reward_source(&mut genome, 0, &mut r);
        assert!(result.is_ok());

        if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
            let cfg = g.internal_nodes[0].plasticity.as_ref().unwrap();
            let new_channel = cfg.modulation.as_ref().unwrap().reward_source;
            assert_ne!(new_channel, original);
        }
    }

    #[test]
    fn mutate_trace_decay_stays_in_bounds() {
        let mut genome = genome_with_graph(vec![node_with_reward_modulation()]);
        for seed in 0u64..100 {
            let mut r = rng(seed);
            mutate_trace_decay(&mut genome, 0, &mut r).unwrap();

            if let BackendDef::Graph(ref g) = genome.nodes[0].backend_def {
                let decay = g.internal_nodes[0]
                    .plasticity
                    .as_ref()
                    .unwrap()
                    .modulation
                    .as_ref()
                    .unwrap()
                    .trace_decay;
                assert!(
                    (0.0..=1.0).contains(&decay),
                    "decay {decay} out of bounds at seed {seed}"
                );
            }
        }
    }

    #[test]
    fn enable_hebbian_skips_nodes_without_inputs() {
        // A node with no inputs should not get Hebbian enabled.
        let node = GraphInternalNode {
            kind: GraphNodeKind::Constant(1.0),
            inputs: vec![],
            plasticity: None,
        };
        let mut genome = genome_with_graph(vec![node]);
        let mut r = rng(42);
        let result = enable_hebbian(&mut genome, 0, &mut r);
        assert_eq!(result, Err(MutationSkipReason::NoApplicableTarget));
    }
}
