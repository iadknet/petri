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
    BackendDef, CreatureGenome, HebbianRule, OutcomeChannel, PlasticityConfig,
    RewardModulationConfig,
};
use crate::mutation::types::MutationSkipReason;

#[inline]
fn graph_def_mut(
    genome: &mut CreatureGenome,
    node_idx: usize,
) -> Result<&mut CgpGraphBackendDef, MutationSkipReason> {
    let node = genome
        .nodes
        .get_mut(node_idx)
        .ok_or(MutationSkipReason::NoApplicableTarget)?;
    match &mut node.backend_def {
        BackendDef::Graph(def) => Ok(def),
        _ => Err(MutationSkipReason::NoApplicableTarget),
    }
}

/// Add PlasticityConfig to a random non-plasticity compute node.
pub(super) fn enable_hebbian(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    enable_hebbian_in_def(graph_def_mut(genome, node_idx)?, rng)
}

/// Remove PlasticityConfig from a random plasticity compute node.
pub(super) fn disable_hebbian(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    disable_hebbian_in_def(graph_def_mut(genome, node_idx)?, rng)
}

/// Change the HebbianRule variant on a random Hebbian compute node.
pub(super) fn mutate_hebbian_rule(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    mutate_hebbian_rule_in_def(graph_def_mut(genome, node_idx)?, rng)
}

/// Perturb the learning_rate on a random Hebbian compute node.
pub(super) fn mutate_hebbian_rate(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    mutate_hebbian_rate_in_def(graph_def_mut(genome, node_idx)?, rng)
}

/// Flip the `lamarckian` flag on a random Hebbian compute node.
pub(super) fn toggle_hebbian_lamarckian(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    toggle_hebbian_lamarckian_in_def(graph_def_mut(genome, node_idx)?, rng)
}

/// Add reward modulation to a random plasticity node that has `modulation=None`.
pub(super) fn enable_reward_modulation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    enable_reward_modulation_in_def(graph_def_mut(genome, node_idx)?, rng)
}

/// Remove reward modulation from a random reward-modulated node.
pub(super) fn disable_reward_modulation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    disable_reward_modulation_in_def(graph_def_mut(genome, node_idx)?, rng)
}

/// Switch the `reward_source` channel on a random reward-modulated node.
pub(super) fn mutate_reward_source(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    mutate_reward_source_in_def(graph_def_mut(genome, node_idx)?, rng)
}

/// Perturb the `trace_decay` on a random reward-modulated node.
pub(super) fn mutate_trace_decay(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    mutate_trace_decay_in_def(graph_def_mut(genome, node_idx)?, rng)
}

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

// ─── Applicability predicates ───────────────────────────────────────────────
//
// The per-node predicates the plasticity operators draw their target from.
// `GraphMutator::apply` filters Graph-backend nodes by [`any_node`] over the
// same predicate, so an operator is never selected onto a module whose
// compute nodes it cannot touch.

/// `EnableHebbian`: a node with edges to learn on and no plasticity yet.
pub(super) fn can_enable_hebbian(node: &ComputeNode) -> bool {
    node.plasticity.is_none() && !node.inputs.is_empty()
}

/// `DisableHebbian`, `MutateHebbianRule`, `MutateHebbianRate`,
/// `ToggleHebbianLamarckian`: a node that already carries plasticity.
pub(super) fn is_plastic(node: &ComputeNode) -> bool {
    node.plasticity.is_some()
}

/// `EnableRewardModulation`: a plastic node without modulation.
pub(super) fn can_enable_reward_modulation(node: &ComputeNode) -> bool {
    node.plasticity
        .as_ref()
        .is_some_and(|p| p.modulation.is_none())
}

/// `DisableRewardModulation`, `MutateRewardSource`, `MutateTraceDecay`: a
/// plastic node that carries modulation.
pub(super) fn is_reward_modulated(node: &ComputeNode) -> bool {
    node.plasticity
        .as_ref()
        .is_some_and(|p| p.modulation.is_some())
}

/// Whether any compute node in `def` matches `predicate`: the def-level form
/// of the predicates above, and what [`select_eligible`] needs to succeed.
pub(super) fn any_node(def: &CgpGraphBackendDef, predicate: fn(&ComputeNode) -> bool) -> bool {
    def.compute_nodes.iter().any(predicate)
}

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
pub(crate) fn enable_hebbian_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(&def.compute_nodes, can_enable_hebbian, rng)?;

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
pub(crate) fn disable_hebbian_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(&def.compute_nodes, is_plastic, rng)?;
    def.compute_nodes[idx].plasticity = None;
    Ok(())
}

/// Change the HebbianRule variant on a random Hebbian compute node.
pub(crate) fn mutate_hebbian_rule_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(&def.compute_nodes, is_plastic, rng)?;

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
pub(crate) fn mutate_hebbian_rate_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(&def.compute_nodes, is_plastic, rng)?;

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
pub(crate) fn toggle_hebbian_lamarckian_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(&def.compute_nodes, is_plastic, rng)?;

    if let Some(ref mut cfg) = def.compute_nodes[idx].plasticity {
        cfg.lamarckian = !cfg.lamarckian;
    }
    Ok(())
}

/// Add reward modulation to a random plasticity node that has `modulation=None`.
pub(crate) fn enable_reward_modulation_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(&def.compute_nodes, can_enable_reward_modulation, rng)?;

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
pub(crate) fn disable_reward_modulation_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(&def.compute_nodes, is_reward_modulated, rng)?;

    if let Some(ref mut cfg) = def.compute_nodes[idx].plasticity {
        cfg.modulation = None;
    }
    Ok(())
}

/// Switch the `reward_source` channel on a random reward-modulated node.
pub(crate) fn mutate_reward_source_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(&def.compute_nodes, is_reward_modulated, rng)?;

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
pub(crate) fn mutate_trace_decay_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let idx = select_eligible(&def.compute_nodes, is_reward_modulated, rng)?;

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
            birth_weights: None,
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
        assert!(enable_hebbian_in_def(&mut def, &mut r).is_ok());
        assert!(def.compute_nodes[0].plasticity.is_some());
    }

    #[test]
    fn enable_hebbian_skips_when_all_already_hebbian() {
        let mut def = def_with_nodes(vec![node_with_hebbian()]);
        let mut r = rng(42);
        assert_eq!(
            enable_hebbian_in_def(&mut def, &mut r),
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
            enable_hebbian_in_def(&mut def, &mut r),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }

    #[test]
    fn disable_hebbian_removes_config() {
        let mut def = def_with_nodes(vec![node_with_hebbian()]);
        let mut r = rng(42);
        assert!(disable_hebbian_in_def(&mut def, &mut r).is_ok());
        assert!(def.compute_nodes[0].plasticity.is_none());
    }

    #[test]
    fn disable_hebbian_skips_when_none_hebbian() {
        let mut def = def_with_nodes(vec![node_without_hebbian()]);
        let mut r = rng(42);
        assert_eq!(
            disable_hebbian_in_def(&mut def, &mut r),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }

    #[test]
    fn mutate_rule_changes_variant() {
        let mut def = def_with_nodes(vec![node_with_hebbian()]);
        let mut r = rng(42);
        assert!(mutate_hebbian_rule_in_def(&mut def, &mut r).is_ok());
        let new_rule = def.compute_nodes[0].plasticity.as_ref().unwrap().rule;
        assert_ne!(new_rule, HebbianRule::Classic);
    }

    /// Each seed starts from the same node so the perturbation is measured
    /// against a known baseline: the rate stays in `[0.0, 1.0]`, and the
    /// operator actually moves it rather than reporting success and leaving
    /// the node untouched.
    #[test]
    fn mutate_rate_stays_in_bounds_and_moves_the_rate() {
        let baseline = node_with_hebbian()
            .plasticity
            .as_ref()
            .unwrap()
            .learning_rate;
        let mut changed = 0usize;
        for seed in 0u64..100 {
            let mut def = def_with_nodes(vec![node_with_hebbian()]);
            let mut r = rng(seed);
            mutate_hebbian_rate_in_def(&mut def, &mut r).unwrap();
            let rate = def.compute_nodes[0]
                .plasticity
                .as_ref()
                .unwrap()
                .learning_rate;
            assert!(
                (0.0..=1.0).contains(&rate),
                "rate {rate} out of bounds at seed {seed}"
            );
            if rate != baseline {
                changed += 1;
            }
        }
        assert!(
            changed > 0,
            "no seed changed the learning rate from {baseline}"
        );
    }

    #[test]
    fn mutate_rate_skips_when_no_node_is_plastic() {
        let mut def = def_with_nodes(vec![node_without_hebbian()]);
        let mut r = rng(42);
        assert_eq!(
            mutate_hebbian_rate_in_def(&mut def, &mut r),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }

    #[test]
    fn toggle_lamarckian_flips_flag() {
        let mut def = def_with_nodes(vec![node_with_hebbian()]);
        let mut r = rng(42);

        assert!(!def.compute_nodes[0].plasticity.as_ref().unwrap().lamarckian);
        toggle_hebbian_lamarckian_in_def(&mut def, &mut r).unwrap();
        assert!(def.compute_nodes[0].plasticity.as_ref().unwrap().lamarckian);
        toggle_hebbian_lamarckian_in_def(&mut def, &mut r).unwrap();
        assert!(!def.compute_nodes[0].plasticity.as_ref().unwrap().lamarckian);
    }

    #[test]
    fn enable_reward_modulation_adds_config() {
        let mut def = def_with_nodes(vec![node_with_hebbian()]);
        let mut r = rng(42);
        assert!(enable_reward_modulation_in_def(&mut def, &mut r).is_ok());
        let cfg = def.compute_nodes[0].plasticity.as_ref().unwrap();
        assert!(cfg.modulation.is_some());
    }

    #[test]
    fn enable_reward_modulation_skips_already_modulated() {
        let mut def = def_with_nodes(vec![node_with_reward_modulation()]);
        let mut r = rng(42);
        assert_eq!(
            enable_reward_modulation_in_def(&mut def, &mut r),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }

    #[test]
    fn disable_reward_modulation_removes_config() {
        let mut def = def_with_nodes(vec![node_with_reward_modulation()]);
        let mut r = rng(42);
        assert!(disable_reward_modulation_in_def(&mut def, &mut r).is_ok());
        let cfg = def.compute_nodes[0].plasticity.as_ref().unwrap();
        assert!(cfg.modulation.is_none());
    }

    #[test]
    fn mutate_reward_source_changes_channel() {
        let mut def = def_with_nodes(vec![node_with_reward_modulation()]);
        let mut r = rng(42);
        assert!(mutate_reward_source_in_def(&mut def, &mut r).is_ok());
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

    /// As with the learning rate: a fresh node per seed, bounds held, and the
    /// decay actually moved off its baseline.
    #[test]
    fn mutate_trace_decay_stays_in_bounds_and_moves_the_decay() {
        let baseline = node_with_reward_modulation()
            .plasticity
            .as_ref()
            .unwrap()
            .modulation
            .as_ref()
            .unwrap()
            .trace_decay;
        let mut changed = 0usize;
        for seed in 0u64..100 {
            let mut def = def_with_nodes(vec![node_with_reward_modulation()]);
            let mut r = rng(seed);
            mutate_trace_decay_in_def(&mut def, &mut r).unwrap();
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
            if decay != baseline {
                changed += 1;
            }
        }
        assert!(
            changed > 0,
            "no seed changed the trace decay from {baseline}"
        );
    }

    /// A plastic node with `modulation = None` is not a trace-decay site, so
    /// the operator must skip before the draw rather than report success.
    #[test]
    fn mutate_trace_decay_skips_when_no_node_is_reward_modulated() {
        let mut def = def_with_nodes(vec![node_with_hebbian()]);
        let mut r = rng(42);
        assert_eq!(
            mutate_trace_decay_in_def(&mut def, &mut r),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }
}
