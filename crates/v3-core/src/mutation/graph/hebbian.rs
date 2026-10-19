//! Plasticity mutation operators on graph edges (T25.F06).
//!
//! Nine operators that evolve per-edge learning attributes on either
//! surface. Each picks one edge uniformly among the edges matching its
//! predicate, in `edge_sites` order over both surfaces (count-then-select,
//! one draw), and applies its value law:
//! - EnableHebbian: add learning attributes to a static, eligible edge
//! - DisableHebbian: remove them from a plastic edge
//! - MutateHebbianRule: change the learning rule variant
//! - MutateHebbianRate: perturb the learning rate
//! - ToggleHebbianLamarckian: flip the inheritance flag
//! - EnableRewardModulation: add reward modulation to a pure Hebbian edge
//! - DisableRewardModulation: remove reward modulation from a modulated edge
//! - MutateRewardSource: change the outcome channel a modulated edge listens to
//! - MutateTraceDecay: perturb the trace decay rate on a modulated edge

use rand::Rng;

use crate::creature::genome::cgp::{CgpGraphBackendDef, EdgeOccurrence};
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

/// Add learning attributes to a random static, eligible edge.
pub(super) fn enable_hebbian(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    enable_hebbian_in_def(graph_def_mut(genome, node_idx)?, rng)
}

/// Remove the learning attributes of a random plastic edge.
pub(super) fn disable_hebbian(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    disable_hebbian_in_def(graph_def_mut(genome, node_idx)?, rng)
}

/// Change the HebbianRule variant on a random plastic edge.
pub(super) fn mutate_hebbian_rule(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    mutate_hebbian_rule_in_def(graph_def_mut(genome, node_idx)?, rng)
}

/// Perturb the learning_rate on a random plastic edge.
pub(super) fn mutate_hebbian_rate(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    mutate_hebbian_rate_in_def(graph_def_mut(genome, node_idx)?, rng)
}

/// Flip the `lamarckian` flag on a random plastic edge.
pub(super) fn toggle_hebbian_lamarckian(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    toggle_hebbian_lamarckian_in_def(graph_def_mut(genome, node_idx)?, rng)
}

/// Add reward modulation to a random plastic edge that has `modulation=None`.
pub(super) fn enable_reward_modulation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    enable_reward_modulation_in_def(graph_def_mut(genome, node_idx)?, rng)
}

/// Remove reward modulation from a random reward-modulated edge.
pub(super) fn disable_reward_modulation(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    disable_reward_modulation_in_def(graph_def_mut(genome, node_idx)?, rng)
}

/// Switch the `reward_source` channel on a random reward-modulated edge.
pub(super) fn mutate_reward_source(
    genome: &mut CreatureGenome,
    node_idx: usize,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    mutate_reward_source_in_def(graph_def_mut(genome, node_idx)?, rng)
}

/// Perturb the `trace_decay` on a random reward-modulated edge.
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
// The per-edge predicates the plasticity operators draw their target from.
// `GraphMutator::apply` filters Graph-backend nodes by [`any_edge`] over the
// same predicate, so an operator is never selected onto a module with no
// edge it can touch. A `ClearSlot` edge or a group member never matches
// (`CgpGraphBackendDef::can_learn`).

/// The edge's learning attributes.
fn plasticity_of(def: &CgpGraphBackendDef, occurrence: EdgeOccurrence) -> Option<PlasticityConfig> {
    def.edge_at(occurrence).plasticity()
}

/// `EnableHebbian`: a static edge that may learn.
pub(super) fn can_enable_hebbian(def: &CgpGraphBackendDef, occurrence: EdgeOccurrence) -> bool {
    plasticity_of(def, occurrence).is_none() && def.can_learn(occurrence)
}

/// `DisableHebbian`, `MutateHebbianRule`, `MutateHebbianRate`,
/// `ToggleHebbianLamarckian`: an edge that already carries plasticity.
pub(super) fn is_plastic(def: &CgpGraphBackendDef, occurrence: EdgeOccurrence) -> bool {
    plasticity_of(def, occurrence).is_some()
}

/// `EnableRewardModulation`: a plastic edge without modulation.
pub(super) fn can_enable_reward_modulation(
    def: &CgpGraphBackendDef,
    occurrence: EdgeOccurrence,
) -> bool {
    plasticity_of(def, occurrence).is_some_and(|p| p.modulation.is_none())
}

/// `DisableRewardModulation`, `MutateRewardSource`, `MutateTraceDecay`: a
/// plastic edge that carries modulation.
pub(super) fn is_reward_modulated(def: &CgpGraphBackendDef, occurrence: EdgeOccurrence) -> bool {
    plasticity_of(def, occurrence).is_some_and(|p| p.modulation.is_some())
}

/// An edge predicate of this module.
pub(super) type EdgePredicate = fn(&CgpGraphBackendDef, EdgeOccurrence) -> bool;

/// Whether any edge in `def` matches `predicate`: what an operator's
/// `applies_to` and [`select_eligible`] need to succeed.
pub(super) fn any_edge(def: &CgpGraphBackendDef, predicate: EdgePredicate) -> bool {
    def.edge_sites()
        .any(|occurrence| predicate(def, occurrence))
}

/// Pick a random edge matching `predicate`, in `edge_sites` order over both
/// surfaces, or `NoApplicableTarget` if none match. Two-pass
/// count-then-select with one draw avoids allocating the eligible list.
fn select_eligible(
    def: &CgpGraphBackendDef,
    predicate: EdgePredicate,
    rng: &mut impl Rng,
) -> Result<EdgeOccurrence, MutationSkipReason> {
    let matching = || {
        def.edge_sites()
            .filter(|&occurrence| predicate(def, occurrence))
    };
    let count = matching().count();
    if count == 0 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let target = rng.gen_range(0..count);
    matching()
        .nth(target)
        .ok_or(MutationSkipReason::NoApplicableTarget)
}

/// Rewrite the learning attributes of a random edge matching `predicate`
/// with `law`, which receives the current config.
fn mutate_selected<R: Rng>(
    def: &mut CgpGraphBackendDef,
    predicate: EdgePredicate,
    rng: &mut R,
    law: impl FnOnce(&mut PlasticityConfig, &mut R),
) -> Result<(), MutationSkipReason> {
    let occurrence = select_eligible(def, predicate, rng)?;
    let mut config =
        plasticity_of(def, occurrence).ok_or(MutationSkipReason::NoApplicableTarget)?;
    law(&mut config, rng);
    def.edge_mut(occurrence).set_plasticity(Some(config));
    Ok(())
}

/// Add learning attributes to a random static, eligible edge.
pub(crate) fn enable_hebbian_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let occurrence = select_eligible(def, can_enable_hebbian, rng)?;

    let rule = ALL_RULES[rng.gen_range(0..ALL_RULES.len())];
    def.edge_mut(occurrence)
        .set_plasticity(Some(PlasticityConfig {
            rule,
            learning_rate: rng.gen_range(0.01f32..0.2),
            weight_clamp: rng.gen_range(1.0f32..5.0),
            lamarckian: rng.gen_bool(0.5),
            modulation: None,
        }));
    Ok(())
}

/// Remove the learning attributes of a random plastic edge.
pub(crate) fn disable_hebbian_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    let occurrence = select_eligible(def, is_plastic, rng)?;
    def.edge_mut(occurrence).set_plasticity(None);
    Ok(())
}

/// Change the HebbianRule variant on a random plastic edge.
pub(crate) fn mutate_hebbian_rule_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    mutate_selected(def, is_plastic, rng, |cfg, rng| {
        let others: Vec<HebbianRule> = ALL_RULES
            .iter()
            .copied()
            .filter(|r| *r != cfg.rule)
            .collect();
        cfg.rule = others[rng.gen_range(0..others.len())];
    })
}

/// Perturb the learning_rate on a random plastic edge.
///
/// Uses ~30% multiplicative perturbation when rate is above a threshold,
/// otherwise a small additive perturbation; clamped to [0.0, 1.0] (I1).
pub(crate) fn mutate_hebbian_rate_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    mutate_selected(def, is_plastic, rng, |cfg, rng| {
        if cfg.learning_rate.abs() > 0.01 {
            cfg.learning_rate *= 1.0 + rng.gen_range(-0.3f32..=0.3);
        } else {
            cfg.learning_rate += rng.gen_range(-0.05f32..=0.05);
        }
        cfg.learning_rate = cfg.learning_rate.clamp(0.0, 1.0);
    })
}

/// Flip the `lamarckian` flag on a random plastic edge.
pub(crate) fn toggle_hebbian_lamarckian_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    mutate_selected(def, is_plastic, rng, |cfg, _| {
        cfg.lamarckian = !cfg.lamarckian;
    })
}

/// Add reward modulation to a random plastic edge that has `modulation=None`.
pub(crate) fn enable_reward_modulation_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    mutate_selected(def, can_enable_reward_modulation, rng, |cfg, rng| {
        let channel = ALL_CHANNELS[rng.gen_range(0..ALL_CHANNELS.len())];
        cfg.modulation = Some(RewardModulationConfig {
            reward_source: channel,
            trace_decay: rng.gen_range(0.5f32..0.99),
        });
    })
}

/// Remove reward modulation from a random reward-modulated edge.
pub(crate) fn disable_reward_modulation_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    mutate_selected(def, is_reward_modulated, rng, |cfg, _| {
        cfg.modulation = None;
    })
}

/// Switch the `reward_source` channel on a random reward-modulated edge.
pub(crate) fn mutate_reward_source_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    mutate_selected(def, is_reward_modulated, rng, |cfg, rng| {
        if let Some(modulation) = &mut cfg.modulation {
            let others: Vec<OutcomeChannel> = ALL_CHANNELS
                .iter()
                .copied()
                .filter(|c| *c != modulation.reward_source)
                .collect();
            modulation.reward_source = others[rng.gen_range(0..others.len())];
        }
    })
}

/// Perturb the `trace_decay` on a random reward-modulated edge.
///
/// Uses ~30% multiplicative perturbation when decay is above a threshold,
/// otherwise a small additive perturbation. Clamped to [0.0, 1.0] (I1).
pub(crate) fn mutate_trace_decay_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<(), MutationSkipReason> {
    mutate_selected(def, is_reward_modulated, rng, |cfg, rng| {
        if let Some(modulation) = &mut cfg.modulation {
            if modulation.trace_decay.abs() > 0.01 {
                modulation.trace_decay *= 1.0 + rng.gen_range(-0.3f32..=0.3);
            } else {
                modulation.trace_decay += rng.gen_range(-0.05f32..=0.05);
            }
            modulation.trace_decay = modulation.trace_decay.clamp(0.0, 1.0);
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::genome::cgp::test_support::plastic_node;
    use crate::creature::genome::cgp::{
        ComputeNode, ComputeNodeKind, EdgeSurface, GraphEdge, GraphSource, OutputSink,
        OutputSinkKind,
    };
    use crate::creature::genome::vote::VoteSink;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn rng(seed: u64) -> SmallRng {
        SmallRng::seed_from_u64(seed)
    }

    fn def_with_nodes(nodes: Vec<ComputeNode>) -> CgpGraphBackendDef {
        CgpGraphBackendDef::new_with_fixed_outputs()
            .with_compute_nodes(nodes)
            .with_output_sinks(Vec::new())
    }

    fn hebbian() -> PlasticityConfig {
        PlasticityConfig {
            rule: HebbianRule::Classic,
            learning_rate: 0.1,
            weight_clamp: 5.0,
            lamarckian: false,
            modulation: None,
        }
    }

    fn modulated() -> PlasticityConfig {
        PlasticityConfig {
            modulation: Some(RewardModulationConfig {
                reward_source: OutcomeChannel::EnergyDelta,
                trace_decay: 0.9,
            }),
            ..hebbian()
        }
    }

    fn node(config: Option<PlasticityConfig>) -> ComputeNode {
        plastic_node(
            ComputeNodeKind::Add,
            vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.0)],
            config,
        )
    }

    fn node_without_hebbian() -> ComputeNode {
        node(None)
    }

    fn node_with_hebbian() -> ComputeNode {
        node(Some(hebbian()))
    }

    fn node_with_reward_modulation() -> ComputeNode {
        node(Some(modulated()))
    }

    /// The learning attributes of the first edge of compute node 0.
    fn first(def: &CgpGraphBackendDef) -> Option<PlasticityConfig> {
        def.compute_nodes()[0].inputs[0].plasticity()
    }

    #[test]
    fn enable_hebbian_adds_config() {
        let mut def = def_with_nodes(vec![node_without_hebbian()]);
        let mut r = rng(42);
        assert!(enable_hebbian_in_def(&mut def, &mut r).is_ok());
        let config = first(&def).expect("the edge learns");
        assert!(config.in_range());
        assert!(config.modulation.is_none());
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
    fn enable_hebbian_skips_a_graph_without_edges() {
        let node = ComputeNode::new(ComputeNodeKind::Constant(1.0), Vec::new());
        let mut def = def_with_nodes(vec![node]);
        let mut r = rng(42);
        assert_eq!(
            enable_hebbian_in_def(&mut def, &mut r),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }

    /// A `ClearSlot` edge never matches (T25.F06 invariant 2): its sum is
    /// ignored, so it has nothing to learn.
    #[test]
    fn enable_hebbian_never_targets_a_clear_slot_edge() {
        let mut def = def_with_nodes(Vec::new()).with_output_sinks(vec![OutputSink::new(
            OutputSinkKind::ClearSlot(3),
            vec![GraphEdge::new(GraphSource::ComputeNode(0), 1.0)],
        )]);
        assert!(!any_edge(&def, can_enable_hebbian));
        assert_eq!(
            enable_hebbian_in_def(&mut def, &mut rng(42)),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }

    #[test]
    fn disable_hebbian_removes_config() {
        let mut def = def_with_nodes(vec![node_with_hebbian()]);
        let mut r = rng(42);
        assert!(disable_hebbian_in_def(&mut def, &mut r).is_ok());
        assert!(first(&def).is_none());
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
        assert_ne!(first(&def).unwrap().rule, HebbianRule::Classic);
    }

    /// Each seed starts from the same edge so the perturbation is measured
    /// against a known baseline: the rate stays in `[0.0, 1.0]`, and the
    /// operator actually moves it rather than reporting success and leaving
    /// the edge untouched.
    #[test]
    fn mutate_rate_stays_in_bounds_and_moves_the_rate() {
        let baseline = hebbian().learning_rate;
        let mut changed = 0usize;
        for seed in 0u64..100 {
            let mut def = def_with_nodes(vec![node_with_hebbian()]);
            let mut r = rng(seed);
            mutate_hebbian_rate_in_def(&mut def, &mut r).unwrap();
            let rate = first(&def).unwrap().learning_rate;
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
    fn mutate_rate_skips_when_no_edge_is_plastic() {
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
        assert!(!first(&def).unwrap().lamarckian);
        toggle_hebbian_lamarckian_in_def(&mut def, &mut r).unwrap();
        assert!(first(&def).unwrap().lamarckian);
        toggle_hebbian_lamarckian_in_def(&mut def, &mut r).unwrap();
        assert!(!first(&def).unwrap().lamarckian);
    }

    #[test]
    fn enable_reward_modulation_adds_config() {
        let mut def = def_with_nodes(vec![node_with_hebbian()]);
        let mut r = rng(42);
        assert!(enable_reward_modulation_in_def(&mut def, &mut r).is_ok());
        let config = first(&def).unwrap();
        assert!(config.modulation.is_some());
        assert!(config.in_range());
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
        assert!(first(&def).unwrap().modulation.is_none());
    }

    #[test]
    fn mutate_reward_source_changes_channel() {
        let mut def = def_with_nodes(vec![node_with_reward_modulation()]);
        let mut r = rng(42);
        assert!(mutate_reward_source_in_def(&mut def, &mut r).is_ok());
        let channel = first(&def).unwrap().modulation.unwrap().reward_source;
        assert_ne!(channel, OutcomeChannel::EnergyDelta);
    }

    /// As with the learning rate: a fresh edge per seed, bounds held, and the
    /// decay actually moved off its baseline.
    #[test]
    fn mutate_trace_decay_stays_in_bounds_and_moves_the_decay() {
        let baseline = modulated().modulation.unwrap().trace_decay;
        let mut changed = 0usize;
        for seed in 0u64..100 {
            let mut def = def_with_nodes(vec![node_with_reward_modulation()]);
            let mut r = rng(seed);
            mutate_trace_decay_in_def(&mut def, &mut r).unwrap();
            let decay = first(&def).unwrap().modulation.unwrap().trace_decay;
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

    /// A plastic edge with `modulation = None` is not a trace-decay site, so
    /// the operator must skip before the draw rather than report success.
    #[test]
    fn mutate_trace_decay_skips_when_no_edge_is_reward_modulated() {
        let mut def = def_with_nodes(vec![node_with_hebbian()]);
        let mut r = rng(42);
        assert_eq!(
            mutate_trace_decay_in_def(&mut def, &mut r),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }

    type Operator = fn(&mut CgpGraphBackendDef, &mut SmallRng) -> Result<(), MutationSkipReason>;

    /// A graph with static, pure-Hebbian and reward-modulated edges on both
    /// surfaces, and a static `ClearSlot` edge that never matches.
    fn mixed_def() -> CgpGraphBackendDef {
        let edge = |source: u16| GraphEdge::new(GraphSource::ComputeNode(source), 1.0);
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs().with_compute_nodes(vec![
            node_without_hebbian(),
            node_with_hebbian(),
            node_with_reward_modulation(),
        ]);
        for (kind, config) in [
            (OutputSinkKind::CustomOutput(0), None),
            (OutputSinkKind::ActionVote(VoteSink::Eat), Some(hebbian())),
            (
                OutputSinkKind::ActionVote(VoteSink::Terminate),
                Some(modulated()),
            ),
            (OutputSinkKind::ClearSlot(2), None),
            (OutputSinkKind::WriteSlot(4), None),
        ] {
            let surface = def.sink_surface(kind).unwrap();
            def.push_edge(surface, edge(2));
            if config.is_some() {
                let index = def.surface_edges(surface).len() - 1;
                def.edge_mut(EdgeOccurrence::new(surface, index))
                    .set_plasticity(config);
            }
        }
        def
    }

    /// Invariant 9: each operator picks one edge uniformly among the edges
    /// matching its predicate, in `edge_sites` order over both surfaces, with
    /// one draw, and edits that edge alone; across seeds it reaches both
    /// surfaces.
    #[test]
    fn every_operator_draws_one_matching_edge_over_both_surfaces() {
        let operators: [(&str, Operator, EdgePredicate); 9] = [
            ("enable", enable_hebbian_in_def, can_enable_hebbian),
            ("disable", disable_hebbian_in_def, is_plastic),
            ("rule", mutate_hebbian_rule_in_def, is_plastic),
            ("rate", mutate_hebbian_rate_in_def, is_plastic),
            ("lamarckian", toggle_hebbian_lamarckian_in_def, is_plastic),
            (
                "enable_modulation",
                enable_reward_modulation_in_def,
                can_enable_reward_modulation,
            ),
            (
                "disable_modulation",
                disable_reward_modulation_in_def,
                is_reward_modulated,
            ),
            ("source", mutate_reward_source_in_def, is_reward_modulated),
            ("decay", mutate_trace_decay_in_def, is_reward_modulated),
        ];
        let original = mixed_def();
        for (name, operator, predicate) in operators {
            let matching: Vec<EdgeOccurrence> = original
                .edge_sites()
                .filter(|&occurrence| predicate(&original, occurrence))
                .collect();
            assert!(!matching.is_empty(), "{name}");
            let mut surfaces = [false; 2];
            for seed in 0..64 {
                let expected = matching[rng(seed).gen_range(0..matching.len())];
                let mut def = original.clone();
                operator(&mut def, &mut rng(seed)).unwrap();
                let edited: Vec<EdgeOccurrence> = def
                    .edge_sites()
                    .filter(|&occurrence| def.edge_at(occurrence) != original.edge_at(occurrence))
                    .collect();
                // A rule or source redraw always changes; a perturbation can
                // round to the same value, which edits nothing.
                assert!(
                    edited.is_empty() || edited == vec![expected],
                    "{name} seed {seed}"
                );
                if let Some(&occurrence) = edited.first() {
                    surfaces
                        [usize::from(matches!(occurrence.surface, EdgeSurface::SinkInput(_)))] =
                        true;
                }
                assert_eq!(
                    crate::creature::genome::cgp::validate_graph(
                        &crate::creature::genome::NodeGenome {
                            node_id: crate::contracts::NodeId::new(0),
                            input_refs: Vec::new(),
                            backend_def: BackendDef::Graph(def),
                            targets: Vec::new(),
                        }
                    ),
                    Ok(()),
                    "{name} seed {seed}"
                );
            }
            assert_eq!(surfaces, [true, true], "{name} reaches both surfaces");
        }
    }
}
