//! Plasticity mutation operators for CGP compute nodes.
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

use crate::creature::genome::cgp::{CgpGraphBackendDef, ComputeNode};
use crate::creature::genome::{
    HebbianRule, OutcomeChannel, PlasticityConfig, RewardModulationConfig,
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

/// Pick a random compute node matching `predicate`. Returns its index or
/// `NoApplicableTarget` if none match. Two-pass count-then-select avoids
/// allocating a Vec of eligible indices.
fn select_eligible(
    nodes: &[ComputeNode],
    predicate: impl Fn(&ComputeNode) -> bool,
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

/// Add PlasticityConfig to a random non-plasticity compute node.
pub(crate) fn enable_hebbian(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(
        &def.compute_nodes,
        |n| n.plasticity.is_none() && !n.inputs.is_empty(),
        rng,
    )?;

    let rule = ALL_RULES[rng.gen_range(0..ALL_RULES.len())];
    def.compute_nodes[idx].plasticity = Some(PlasticityConfig {
        rule,
        learning_rate: rng.gen_range(0.01f32..0.2),
        weight_clamp: rng.gen_range(1.0f32..5.0),
        lamarckian: rng.gen_bool(0.5),
        modulation: None,
    });
    Ok(())
}

/// Remove PlasticityConfig from a random plasticity compute node.
pub(crate) fn disable_hebbian(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(&def.compute_nodes, |n| n.plasticity.is_some(), rng)?;
    def.compute_nodes[idx].plasticity = None;
    Ok(())
}

/// Change the HebbianRule variant on a random Hebbian compute node.
pub(crate) fn mutate_hebbian_rule(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(&def.compute_nodes, |n| n.plasticity.is_some(), rng)?;

    if let Some(ref mut cfg) = def.compute_nodes[idx].plasticity {
        let others: Vec<HebbianRule> = ALL_RULES
            .iter()
            .copied()
            .filter(|r| *r != cfg.rule)
            .collect();
        if !others.is_empty() {
            cfg.rule = others[rng.gen_range(0..others.len())];
        }
    }
    Ok(())
}

/// Perturb the learning_rate on a random Hebbian compute node.
///
/// Uses ~30% multiplicative perturbation when rate is above a threshold,
/// otherwise a small additive perturbation.
pub(crate) fn mutate_hebbian_rate(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(&def.compute_nodes, |n| n.plasticity.is_some(), rng)?;

    if let Some(ref mut cfg) = def.compute_nodes[idx].plasticity {
        if cfg.learning_rate.abs() > 0.01 {
            cfg.learning_rate *= 1.0 + rng.gen_range(-0.3f32..=0.3);
        } else {
            cfg.learning_rate += rng.gen_range(-0.05f32..=0.05);
        }
        cfg.learning_rate = cfg.learning_rate.clamp(0.0, 1.0);
    }
    Ok(())
}

/// Flip the `lamarckian` flag on a random Hebbian compute node.
pub(crate) fn toggle_hebbian_lamarckian(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(&def.compute_nodes, |n| n.plasticity.is_some(), rng)?;

    if let Some(ref mut cfg) = def.compute_nodes[idx].plasticity {
        cfg.lamarckian = !cfg.lamarckian;
    }
    Ok(())
}

/// Add reward modulation to a random plasticity node that has `modulation=None`.
pub(crate) fn enable_reward_modulation(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(
        &def.compute_nodes,
        |n| {
            n.plasticity
                .as_ref()
                .is_some_and(|p| p.modulation.is_none())
        },
        rng,
    )?;

    if let Some(ref mut cfg) = def.compute_nodes[idx].plasticity {
        let channel = ALL_CHANNELS[rng.gen_range(0..ALL_CHANNELS.len())];
        cfg.modulation = Some(RewardModulationConfig {
            reward_source: channel,
            trace_decay: rng.gen_range(0.5f32..0.99),
        });
    }
    Ok(())
}

/// Remove reward modulation from a random reward-modulated node.
pub(crate) fn disable_reward_modulation(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(
        &def.compute_nodes,
        |n| {
            n.plasticity
                .as_ref()
                .is_some_and(|p| p.modulation.is_some())
        },
        rng,
    )?;

    if let Some(ref mut cfg) = def.compute_nodes[idx].plasticity {
        cfg.modulation = None;
    }
    Ok(())
}

/// Switch the `reward_source` channel on a random reward-modulated node.
pub(crate) fn mutate_reward_source(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(
        &def.compute_nodes,
        |n| {
            n.plasticity
                .as_ref()
                .is_some_and(|p| p.modulation.is_some())
        },
        rng,
    )?;

    if let Some(ref mut cfg) = def.compute_nodes[idx].plasticity {
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
    Ok(())
}

/// Perturb the `trace_decay` on a random reward-modulated node.
///
/// Uses ~30% multiplicative perturbation when decay is above a threshold,
/// otherwise a small additive perturbation. Clamped to [0.0, 1.0].
pub(crate) fn mutate_trace_decay(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(
        &def.compute_nodes,
        |n| {
            n.plasticity
                .as_ref()
                .is_some_and(|p| p.modulation.is_some())
        },
        rng,
    )?;

    if let Some(ref mut cfg) = def.compute_nodes[idx].plasticity {
        if let Some(ref mut modulation) = cfg.modulation {
            if modulation.trace_decay.abs() > 0.01 {
                modulation.trace_decay *= 1.0 + rng.gen_range(-0.3f32..=0.3);
            } else {
                modulation.trace_decay += rng.gen_range(-0.05f32..=0.05);
            }
            modulation.trace_decay = modulation.trace_decay.clamp(0.0, 1.0);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::genome::cgp::{ComputeNodeKind, ExecuteGate, GraphEdge, GraphSource};
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn rng(seed: u64) -> SmallRng {
        SmallRng::seed_from_u64(seed)
    }

    fn def_with_nodes(nodes: Vec<ComputeNode>) -> CgpGraphBackendDef {
        CgpGraphBackendDef {
            compute_nodes: nodes,
            output_sinks: Vec::new(),
            action_bank: Vec::new(),
            execute_gate: ExecuteGate { inputs: Vec::new() },
        }
    }

    fn node_without_hebbian() -> ComputeNode {
        ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 1.0,
            }],
            plasticity: None,
        }
    }

    fn node_with_hebbian() -> ComputeNode {
        ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
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

    fn node_with_reward_modulation() -> ComputeNode {
        ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
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
    fn enable_hebbian_adds_config() {
        let mut def = def_with_nodes(vec![node_without_hebbian()]);
        let mut r = rng(42);
        assert!(enable_hebbian(&mut def, &mut r).is_ok());
        assert!(def.compute_nodes[0].plasticity.is_some());
    }

    #[test]
    fn enable_hebbian_skips_when_all_already_hebbian() {
        let mut def = def_with_nodes(vec![node_with_hebbian()]);
        let mut r = rng(42);
        assert_eq!(
            enable_hebbian(&mut def, &mut r),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }

    #[test]
    fn enable_hebbian_skips_nodes_without_inputs() {
        let node = ComputeNode {
            kind: ComputeNodeKind::Constant(1.0),
            inputs: Vec::new(),
            plasticity: None,
        };
        let mut def = def_with_nodes(vec![node]);
        let mut r = rng(42);
        assert_eq!(
            enable_hebbian(&mut def, &mut r),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }

    #[test]
    fn disable_hebbian_removes_config() {
        let mut def = def_with_nodes(vec![node_with_hebbian()]);
        let mut r = rng(42);
        assert!(disable_hebbian(&mut def, &mut r).is_ok());
        assert!(def.compute_nodes[0].plasticity.is_none());
    }

    #[test]
    fn disable_hebbian_skips_when_none_hebbian() {
        let mut def = def_with_nodes(vec![node_without_hebbian()]);
        let mut r = rng(42);
        assert_eq!(
            disable_hebbian(&mut def, &mut r),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }

    #[test]
    fn mutate_rule_changes_variant() {
        let mut def = def_with_nodes(vec![node_with_hebbian()]);
        let mut r = rng(42);
        assert!(mutate_hebbian_rule(&mut def, &mut r).is_ok());
        let new_rule = def.compute_nodes[0].plasticity.as_ref().unwrap().rule;
        assert_ne!(new_rule, HebbianRule::Classic);
    }

    #[test]
    fn mutate_rate_stays_in_bounds() {
        let mut def = def_with_nodes(vec![node_with_hebbian()]);
        for seed in 0u64..100 {
            let mut r = rng(seed);
            mutate_hebbian_rate(&mut def, &mut r).unwrap();
            let rate = def.compute_nodes[0]
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

    #[test]
    fn toggle_lamarckian_flips_flag() {
        let mut def = def_with_nodes(vec![node_with_hebbian()]);
        let mut r = rng(42);

        assert!(!def.compute_nodes[0].plasticity.as_ref().unwrap().lamarckian);
        toggle_hebbian_lamarckian(&mut def, &mut r).unwrap();
        assert!(def.compute_nodes[0].plasticity.as_ref().unwrap().lamarckian);
        toggle_hebbian_lamarckian(&mut def, &mut r).unwrap();
        assert!(!def.compute_nodes[0].plasticity.as_ref().unwrap().lamarckian);
    }

    #[test]
    fn enable_reward_modulation_adds_config() {
        let mut def = def_with_nodes(vec![node_with_hebbian()]);
        let mut r = rng(42);
        assert!(enable_reward_modulation(&mut def, &mut r).is_ok());
        let cfg = def.compute_nodes[0].plasticity.as_ref().unwrap();
        assert!(cfg.modulation.is_some());
    }

    #[test]
    fn enable_reward_modulation_skips_already_modulated() {
        let mut def = def_with_nodes(vec![node_with_reward_modulation()]);
        let mut r = rng(42);
        assert_eq!(
            enable_reward_modulation(&mut def, &mut r),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }

    #[test]
    fn disable_reward_modulation_removes_config() {
        let mut def = def_with_nodes(vec![node_with_reward_modulation()]);
        let mut r = rng(42);
        assert!(disable_reward_modulation(&mut def, &mut r).is_ok());
        let cfg = def.compute_nodes[0].plasticity.as_ref().unwrap();
        assert!(cfg.modulation.is_none());
    }

    #[test]
    fn mutate_reward_source_changes_channel() {
        let mut def = def_with_nodes(vec![node_with_reward_modulation()]);
        let mut r = rng(42);
        assert!(mutate_reward_source(&mut def, &mut r).is_ok());
        let channel = def.compute_nodes[0]
            .plasticity
            .as_ref()
            .unwrap()
            .modulation
            .as_ref()
            .unwrap()
            .reward_source;
        assert_ne!(channel, OutcomeChannel::EnergyDelta);
    }

    #[test]
    fn mutate_trace_decay_stays_in_bounds() {
        let mut def = def_with_nodes(vec![node_with_reward_modulation()]);
        for seed in 0u64..100 {
            let mut r = rng(seed);
            mutate_trace_decay(&mut def, &mut r).unwrap();
            let decay = def.compute_nodes[0]
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
