//! Plasticity mutation operators on graph edges (T25.F06).
//!
//! Nine operators that evolve per-edge learning attributes on either
//! surface. Each picks one edge uniformly among the edges matching its
//! predicate, in `edge_sites` order over both surfaces (count-then-select,
//! one draw); a picked member then draws the scope coin, which lands the
//! change on it alone or on every member of its group the predicate admits
//! (T25.F07); the value is drawn once and each target applies its law:
//! - EnableHebbian: add learning attributes to a static, eligible edge
//! - DisableHebbian: remove them from a plastic edge
//! - MutateHebbianRule: change the learning rule variant
//! - MutateHebbianRate: perturb the learning rate
//! - ToggleHebbianLamarckian: flip the inheritance flag
//! - EnableRewardModulation: add reward modulation to a pure Hebbian edge
//! - DisableRewardModulation: remove reward modulation from a modulated edge
//! - MutateRewardSource: change the outcome channel a modulated edge listens to
//! - MutateTraceDecay: perturb the trace decay rate on a modulated edge
//!
//!
//! Three retention operators (T25.F09) move an edge's learned-weight
//! retention `ρ` on and off 1 and step its deficit `1 − ρ`, through the same
//! pick and coin:
//! - EnableRetention: a plastic edge with `ρ = 1` decays, `1 − ρ` drawn
//!   log-uniform over `[10⁻⁴, 1]`
//! - DisableRetention: a decaying edge's `ρ` returns to 1
//! - MutateRetention: a decaying edge's deficit steps multiplicatively
//!
//! `SetScope` flips a member's scope through the same coin.

use rand::{Rng, RngCore};

use crate::creature::genome::cgp::{CgpGraphBackendDef, EdgeOccurrence, Scope};
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

// ─── Applicability predicates ───────────────────────────────────────────────
//
// The per-edge predicates the plasticity operators draw their target from.
// `GraphMutator::apply` filters Graph-backend nodes by [`any_edge`] over the
// same predicate, so an operator is never selected onto a module with no
// edge it can touch. A `ClearSlot` edge never matches
// (`CgpGraphBackendDef::can_learn`); a member does (T25.F07).

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

/// `EnableRetention`: a plastic edge whose retention is exactly 1.
pub(super) fn can_enable_retention(def: &CgpGraphBackendDef, occurrence: EdgeOccurrence) -> bool {
    plasticity_of(def, occurrence).is_some_and(|p| !decays(p.retention))
}

/// `DisableRetention`, `MutateRetention`: a plastic edge whose retention is
/// below 1.
pub(super) fn is_decaying(def: &CgpGraphBackendDef, occurrence: EdgeOccurrence) -> bool {
    plasticity_of(def, occurrence).is_some_and(|p| decays(p.retention))
}

/// Whether a retention decays: below 1, compared exactly (T25.F09).
pub(super) fn decays(retention: f32) -> bool {
    retention < 1.0
}

/// `SetScope`: a group member.
pub(super) fn is_member(def: &CgpGraphBackendDef, occurrence: EdgeOccurrence) -> bool {
    def.edge_at(occurrence).membership().is_some()
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
pub(super) fn select_eligible(
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

/// The scope coin (T25.F07 invariant 7): the scope a change picked at
/// `picked` lands on and its targets (T25.F08 invariant 8). A non-member
/// draws no coin and is the only target, at `Edge`; a member draws
/// `gen_bool(0.5)`: false, itself at `Edge`; true, every member of its group
/// that `admits` accepts, in canonical order, at `Group`.
pub(super) fn scope_targets(
    def: &CgpGraphBackendDef,
    picked: EdgeOccurrence,
    admits: EdgePredicate,
    rng: &mut impl Rng,
) -> (Scope, Vec<EdgeOccurrence>) {
    match def.edge_at(picked).membership() {
        Some(membership) if rng.gen_bool(0.5) => (
            Scope::Group,
            def.members(membership.group)
                .filter(|&member| admits(def, member))
                .collect(),
        ),
        _ => (Scope::Edge, vec![picked]),
    }
}

/// A plasticity law split into its draws and its per-config apply (R3):
/// the value is drawn once, after the scope coin, from the picked edge's
/// config, and every target applies it to its own config.
struct Law<V> {
    draw: fn(Option<PlasticityConfig>, &mut dyn RngCore) -> V,
    apply: fn(Option<PlasticityConfig>, &V) -> Option<PlasticityConfig>,
}

/// Pick one edge matching `predicate`, draw the scope coin, draw the law's
/// value once and apply it to every target's config; returns the scope the
/// change landed on.
fn mutate_scoped<V, R: Rng>(
    def: &mut CgpGraphBackendDef,
    predicate: EdgePredicate,
    rng: &mut R,
    law: Law<V>,
) -> Result<Scope, MutationSkipReason> {
    let picked = select_eligible(def, predicate, rng)?;
    let (scope, targets) = scope_targets(def, picked, predicate, rng);
    let value = (law.draw)(plasticity_of(def, picked), rng);
    for target in targets {
        let config = (law.apply)(plasticity_of(def, target), &value);
        def.edge_mut(target).set_plasticity(config);
    }
    Ok(scope)
}

/// `EnableHebbian`'s value law: a fresh pure-Hebbian config, in draw order
/// rule, rate, clamp, inheritance.
pub(super) fn draw_hebbian_config(rng: &mut dyn RngCore) -> PlasticityConfig {
    let rule = ALL_RULES[rng.gen_range(0..ALL_RULES.len())];
    PlasticityConfig {
        rule,
        learning_rate: rng.gen_range(0.01f32..0.2),
        weight_clamp: rng.gen_range(1.0f32..5.0),
        lamarckian: rng.gen_bool(0.5),
        modulation: None,
        retention: 1.0,
    }
}

/// `EnableRewardModulation`'s value law: a fresh modulation, in draw order
/// channel, decay.
pub(super) fn draw_modulation(rng: &mut dyn RngCore) -> RewardModulationConfig {
    let channel = ALL_CHANNELS[rng.gen_range(0..ALL_CHANNELS.len())];
    RewardModulationConfig {
        reward_source: channel,
        trace_decay: rng.gen_range(0.5f32..0.99),
    }
}

/// One perturbation draw `u ∈ [−1, 1]`, applied per value by its branch:
/// `× (1 + 0.3u)` above 0.01, else `+ 0.05u`, then clamped to `[0, 1]`
/// (I1).
fn perturb(value: f32, u: f32) -> f32 {
    let stepped = if value > 0.01 {
        value * (1.0 + 0.3 * u)
    } else {
        value + 0.05 * u
    };
    stepped.clamp(0.0, 1.0)
}

/// One of `all` other than `current`, uniformly: one draw over the others
/// in `all` order.
fn draw_other<T: Copy + PartialEq>(all: &[T], current: Option<T>, rng: &mut dyn RngCore) -> T {
    let others: Vec<T> = all
        .iter()
        .copied()
        .filter(|&value| Some(value) != current)
        .collect();
    others[rng.gen_range(0..others.len())]
}

fn draw_u(_: Option<PlasticityConfig>, rng: &mut dyn RngCore) -> f32 {
    rng.gen_range(-1.0f32..=1.0)
}

/// The smallest deficit `1 − ρ` the enable draw and the step reach
/// (T25.F09 invariant 6).
pub(super) const RETENTION_DEFICIT_MIN: f32 = 1e-4;

/// `EnableRetention`'s value law: one `u = gen_range(−4..=0)`, `ρ = 1 −
/// 10ᵘ`, so the deficit is log-uniform over `[10⁻⁴, 1]`.
pub(super) fn draw_retention(rng: &mut dyn RngCore) -> f32 {
    let u = rng.gen_range(-4.0f32..=0.0);
    1.0 - 10f32.powf(u)
}

/// `MutateRetention`'s per-target law: the deficit `d = 1 − ρ` steps by
/// one `u ∈ [−1, 1]`, `d ← (d · (1 + 0.3u)).clamp(10⁻⁴, 1)` (`perturb`'s
/// multiplicative branch, a log-space step), and `ρ = 1 − d`.
pub(super) fn step_retention(retention: f32, u: f32) -> f32 {
    let deficit = 1.0 - retention;
    1.0 - (deficit * (1.0 + 0.3 * u)).clamp(RETENTION_DEFICIT_MIN, 1.0)
}

/// Add learning attributes to a random static, eligible edge (one config,
/// written to every admitted static member).
pub(crate) fn enable_hebbian_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Scope, MutationSkipReason> {
    mutate_scoped(
        def,
        can_enable_hebbian,
        rng,
        Law {
            draw: |_, rng| draw_hebbian_config(rng),
            apply: |_, &config| Some(config),
        },
    )
}

/// Remove the learning attributes of a random plastic edge.
pub(crate) fn disable_hebbian_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Scope, MutationSkipReason> {
    mutate_scoped(
        def,
        is_plastic,
        rng,
        Law {
            draw: |_, _| (),
            apply: |_, ()| None,
        },
    )
}

/// Change the HebbianRule variant on a random plastic edge: one of the
/// others of the picked edge's rule, written to every target.
pub(crate) fn mutate_hebbian_rule_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Scope, MutationSkipReason> {
    mutate_scoped(
        def,
        is_plastic,
        rng,
        Law {
            draw: |picked, rng| draw_other(&ALL_RULES, picked.map(|picked| picked.rule), rng),
            apply: |config, &rule| config.map(|config| PlasticityConfig { rule, ..config }),
        },
    )
}

/// Perturb the learning_rate on a random plastic edge: one `u`, each target
/// stepping its own rate by its branch.
pub(crate) fn mutate_hebbian_rate_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Scope, MutationSkipReason> {
    mutate_scoped(
        def,
        is_plastic,
        rng,
        Law {
            draw: draw_u,
            apply: |config, &u| {
                config.map(|config| PlasticityConfig {
                    learning_rate: perturb(config.learning_rate, u),
                    ..config
                })
            },
        },
    )
}

/// Flip the `lamarckian` flag on a random plastic edge: every target takes
/// the picked edge's flipped value.
pub(crate) fn toggle_hebbian_lamarckian_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Scope, MutationSkipReason> {
    mutate_scoped(
        def,
        is_plastic,
        rng,
        Law {
            draw: |picked, _| !picked.is_some_and(|picked| picked.lamarckian),
            apply: |config, &lamarckian| {
                config.map(|config| PlasticityConfig {
                    lamarckian,
                    ..config
                })
            },
        },
    )
}

/// Add reward modulation to a random plastic edge that has `modulation=None`:
/// one modulation, set on every target, which keeps its rule, rate, clamp
/// and flag.
pub(crate) fn enable_reward_modulation_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Scope, MutationSkipReason> {
    mutate_scoped(
        def,
        can_enable_reward_modulation,
        rng,
        Law {
            draw: |_, rng| draw_modulation(rng),
            apply: |config, &modulation| {
                config.map(|config| PlasticityConfig {
                    modulation: Some(modulation),
                    ..config
                })
            },
        },
    )
}

/// Remove reward modulation from a random reward-modulated edge, keeping
/// each target's config.
pub(crate) fn disable_reward_modulation_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Scope, MutationSkipReason> {
    mutate_scoped(
        def,
        is_reward_modulated,
        rng,
        Law {
            draw: |_, _| (),
            apply: |config, ()| {
                config.map(|config| PlasticityConfig {
                    modulation: None,
                    ..config
                })
            },
        },
    )
}

/// Switch the `reward_source` channel on a random reward-modulated edge:
/// one of the others of the picked edge's channel, written to every target.
pub(crate) fn mutate_reward_source_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Scope, MutationSkipReason> {
    mutate_scoped(
        def,
        is_reward_modulated,
        rng,
        Law {
            draw: |picked, rng| {
                let current = picked
                    .and_then(|picked| picked.modulation)
                    .map(|m| m.reward_source);
                draw_other(&ALL_CHANNELS, current, rng)
            },
            apply: |config, &reward_source| {
                config.map(|config| PlasticityConfig {
                    modulation: config.modulation.map(|modulation| RewardModulationConfig {
                        reward_source,
                        ..modulation
                    }),
                    ..config
                })
            },
        },
    )
}

/// Perturb the `trace_decay` on a random reward-modulated edge: one `u`,
/// each target stepping its own decay by its branch.
pub(crate) fn mutate_trace_decay_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Scope, MutationSkipReason> {
    mutate_scoped(
        def,
        is_reward_modulated,
        rng,
        Law {
            draw: draw_u,
            apply: |config, &u| {
                config.map(|config| PlasticityConfig {
                    modulation: config.modulation.map(|modulation| RewardModulationConfig {
                        trace_decay: perturb(modulation.trace_decay, u),
                        ..modulation
                    }),
                    ..config
                })
            },
        },
    )
}

/// `EnableRetention` (T25.F09): one retention, drawn once after the scope
/// coin, written to every admitted target.
pub(crate) fn enable_retention_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Scope, MutationSkipReason> {
    mutate_scoped(
        def,
        can_enable_retention,
        rng,
        Law {
            draw: |_, rng| draw_retention(rng),
            apply: |config, &retention| {
                config.map(|config| PlasticityConfig {
                    retention,
                    ..config
                })
            },
        },
    )
}

/// `DisableRetention` (T25.F09): every target's retention returns to 1; no
/// value draw.
pub(crate) fn disable_retention_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Scope, MutationSkipReason> {
    mutate_scoped(
        def,
        is_decaying,
        rng,
        Law {
            draw: |_, _| (),
            apply: |config, ()| {
                config.map(|config| PlasticityConfig {
                    retention: 1.0,
                    ..config
                })
            },
        },
    )
}

/// `MutateRetention` (T25.F09): one `u`, each target stepping its own
/// deficit.
pub(crate) fn mutate_retention_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Scope, MutationSkipReason> {
    mutate_scoped(
        def,
        is_decaying,
        rng,
        Law {
            draw: draw_u,
            apply: |config, &u| {
                config.map(|config| PlasticityConfig {
                    retention: step_retention(config.retention, u),
                    ..config
                })
            },
        },
    )
}

/// `SetScope` (T25.F07): one member uniformly in canonical order, the scope
/// coin, then the other of the picked member's scope written to it alone or
/// to every member of its group; no value draw. Returns the scope the
/// change landed on (the coin's), not the scope it wrote.
pub(crate) fn set_scope_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Scope, MutationSkipReason> {
    let picked = select_eligible(def, is_member, rng)?;
    let scope = def
        .edge_at(picked)
        .membership()
        .ok_or(MutationSkipReason::NoApplicableTarget)?
        .scope
        .other();
    let (landed, targets) = scope_targets(def, picked, is_member, rng);
    for target in targets {
        def.edge_mut(target).set_scope(scope);
    }
    Ok(landed)
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
    use crate::creature::genome::BackendDef;
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
            retention: 1.0,
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

    /// The multiplicative branch is strictly above 0.01: a value of exactly
    /// 0.01 takes the additive step, a value just above it the
    /// multiplicative one.
    #[test]
    fn perturb_steps_a_value_of_exactly_one_hundredth_additively() {
        let above = f32::from_bits(0.01f32.to_bits() + 1);

        assert_eq!(perturb(0.01, 1.0), 0.01 + 0.05);
        assert_eq!(perturb(0.01, -1.0), 0.0);
        assert_eq!(perturb(above, 1.0), above * 1.3);
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

    type Operator = fn(&mut CgpGraphBackendDef, &mut SmallRng) -> Result<Scope, MutationSkipReason>;

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

    // ── The scope coin (T25.F07 invariant 7) ────────────────────────────────

    use crate::creature::genome::cgp::{GroupId, Scope, Shape};
    use crate::creature::genome::vote::DirectedVoteKind;

    /// A full catalog with one ring group of eight members on `Move`, member
    /// `k` carrying `configs[k]` (static past the list's end).
    fn group_def(
        configs: &[Option<PlasticityConfig>],
    ) -> (CgpGraphBackendDef, Vec<EdgeOccurrence>) {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        let id = def.add_group(Shape::Ring { ring_ref: 1 }, DirectedVoteKind::Move, 0);
        let members: Vec<EdgeOccurrence> = def.members(id).collect();
        for (&member, &config) in members.iter().zip(configs) {
            def.edge_mut(member).set_plasticity(config);
        }
        (def, members)
    }

    /// The selection draws of an operator over `predicate` on `def`: the
    /// picked edge and, for a member, the coin. `rng` continues at the value
    /// draws.
    fn replay_pick(
        def: &CgpGraphBackendDef,
        predicate: EdgePredicate,
        rng: &mut SmallRng,
    ) -> (EdgeOccurrence, Option<bool>) {
        let matching: Vec<EdgeOccurrence> = def
            .edge_sites()
            .filter(|&occurrence| predicate(def, occurrence))
            .collect();
        let picked = matching[rng.gen_range(0..matching.len())];
        let coin = def.edge_at(picked).membership().map(|_| rng.gen_bool(0.5));
        (picked, coin)
    }

    /// The edges whose learning attributes or scope `op` changed.
    fn edited(before: &CgpGraphBackendDef, after: &CgpGraphBackendDef) -> Vec<EdgeOccurrence> {
        before
            .edge_sites()
            .filter(|&occurrence| before.edge_at(occurrence) != after.edge_at(occurrence))
            .collect()
    }

    fn with(
        config: PlasticityConfig,
        edit: impl FnOnce(&mut PlasticityConfig),
    ) -> PlasticityConfig {
        let mut config = config;
        edit(&mut config);
        config
    }

    /// A non-member draws no coin: the operator's draws are the pick and the
    /// value alone.
    #[test]
    fn a_non_member_draws_no_coin() {
        for seed in 0..16 {
            let mut def = mixed_def();
            let mut expected = rng(seed);
            let (_, coin) = replay_pick(&def, can_enable_hebbian, &mut expected);
            assert_eq!(coin, None);
            let config = draw_hebbian_config(&mut expected);
            let mut actual = rng(seed);
            enable_hebbian_in_def(&mut def, &mut actual).unwrap();
            assert_eq!(actual.gen::<u64>(), expected.gen::<u64>(), "seed {seed}");
            assert!(def.edges().any(|edge| edge.plasticity() == Some(config)));
        }
    }

    /// `EnableHebbian` on a group with a modulated member: the coin lands
    /// one config on the picked member or on every static member; the
    /// member the predicate rejects is untouched and consumes no draw.
    #[test]
    fn enable_hebbian_lands_one_config_on_the_admitted_members() {
        let (original, members) = group_def(&[None, None, None, Some(modulated())]);
        let mut tails = [false; 2];
        for seed in 0..64 {
            let mut expected = rng(seed);
            let (picked, coin) = replay_pick(&original, can_enable_hebbian, &mut expected);
            let coin = coin.expect("a member draws the coin");
            let config = draw_hebbian_config(&mut expected);
            let mut def = original.clone();
            let mut actual = rng(seed);
            enable_hebbian_in_def(&mut def, &mut actual).unwrap();
            assert_eq!(actual.gen::<u64>(), expected.gen::<u64>(), "seed {seed}");
            let want: Vec<EdgeOccurrence> = if coin {
                members
                    .iter()
                    .copied()
                    .filter(|&m| m != members[3])
                    .collect()
            } else {
                vec![picked]
            };
            assert_eq!(edited(&original, &def), want, "seed {seed}");
            assert!(want
                .iter()
                .all(|&m| def.edge_at(m).plasticity() == Some(config)));
            assert_eq!(def.edge_at(members[3]), original.edge_at(members[3]));
            tails[usize::from(coin)] = true;
        }
        assert_eq!(tails, [true, true]);
    }

    /// `EnableRewardModulation` on a heterogeneous group sets one modulation
    /// on every admitted member, each keeping its rule, rate, clamp and flag;
    /// `DisableRewardModulation` clears the modulation alone.
    #[test]
    fn group_wide_modulation_changes_keep_each_members_config() {
        let base = hebbian();
        let configs = [
            Some(base),
            Some(with(base, |c| {
                c.rule = HebbianRule::Oja;
                c.learning_rate = 0.3;
            })),
            Some(with(base, |c| {
                c.weight_clamp = 1.5;
                c.lamarckian = true;
            })),
        ];
        let (original, members) = group_def(&configs);
        let mut all = false;
        for seed in 0..32 {
            let mut expected = rng(seed);
            let (_, coin) = replay_pick(&original, can_enable_reward_modulation, &mut expected);
            let modulation = draw_modulation(&mut expected);
            let mut def = original.clone();
            enable_reward_modulation_in_def(&mut def, &mut rng(seed)).unwrap();
            if coin != Some(true) {
                continue;
            }
            all = true;
            for (k, config) in configs.iter().enumerate() {
                let want = with(config.unwrap(), |c| c.modulation = Some(modulation));
                assert_eq!(def.edge_at(members[k]).plasticity(), Some(want));
            }
            disable_reward_modulation_in_def(&mut def, &mut rng(seed)).unwrap();
            let cleared = members[..configs.len()]
                .iter()
                .filter(|&&m| def.edge_at(m).plasticity().unwrap().modulation.is_none())
                .count();
            assert!(cleared == 1 || cleared == configs.len(), "{cleared}");
            for k in 0..configs.len() {
                let config = def.edge_at(members[k]).plasticity().unwrap();
                if config.modulation.is_none() {
                    assert_eq!(Some(config), configs[k], "the config kept");
                }
            }
        }
        assert!(all, "a group-wide tail");
    }

    /// `ToggleHebbianLamarckian` writes the picked member's flipped flag to
    /// every target; the rate and decay steps draw one `u` and apply it on
    /// each member's own branch, on both sides of 0.01.
    #[test]
    fn group_wide_toggle_and_perturbations_follow_the_picked_value_and_each_branch() {
        let low = with(modulated(), |c| {
            c.learning_rate = 0.005;
            c.lamarckian = true;
            c.modulation = Some(RewardModulationConfig {
                reward_source: OutcomeChannel::DamageDelta,
                trace_decay: 0.004,
            });
        });
        let high = with(modulated(), |c| c.learning_rate = 0.5);
        let configs = [Some(low), Some(high)];
        let (original, members) = group_def(&configs);
        let mut seen = [false; 3];
        for seed in 0..64 {
            let mut expected = rng(seed);
            let (picked, coin) = replay_pick(&original, is_plastic, &mut expected);
            let mut def = original.clone();
            toggle_hebbian_lamarckian_in_def(&mut def, &mut rng(seed)).unwrap();
            let flag = !original.edge_at(picked).plasticity().unwrap().lamarckian;
            if coin == Some(true) {
                seen[0] = true;
                assert!(members[..2].iter().all(|&m| def
                    .edge_at(m)
                    .plasticity()
                    .unwrap()
                    .lamarckian
                    == flag));
            }

            let mut expected = rng(seed);
            let (_, coin) = replay_pick(&original, is_plastic, &mut expected);
            let u: f32 = expected.gen_range(-1.0f32..=1.0);
            let mut def = original.clone();
            let mut actual = rng(seed);
            mutate_hebbian_rate_in_def(&mut def, &mut actual).unwrap();
            assert_eq!(
                actual.gen::<u64>(),
                expected.gen::<u64>(),
                "one u, seed {seed}"
            );
            if coin == Some(true) {
                seen[1] = true;
                let rate = |m: usize| def.edge_at(members[m]).plasticity().unwrap().learning_rate;
                assert_eq!(rate(0), (0.005 + 0.05 * u).clamp(0.0, 1.0));
                assert_eq!(rate(1), (0.5 * (1.0 + 0.3 * u)).clamp(0.0, 1.0));
            }

            let mut expected = rng(seed);
            let (_, coin) = replay_pick(&original, is_reward_modulated, &mut expected);
            let u: f32 = expected.gen_range(-1.0f32..=1.0);
            let mut def = original.clone();
            mutate_trace_decay_in_def(&mut def, &mut rng(seed)).unwrap();
            if coin == Some(true) {
                seen[2] = true;
                let decay = |m: usize| {
                    def.edge_at(members[m])
                        .plasticity()
                        .unwrap()
                        .modulation
                        .unwrap()
                        .trace_decay
                };
                assert_eq!(decay(0), (0.004 + 0.05 * u).clamp(0.0, 1.0));
                assert_eq!(decay(1), (0.9 * (1.0 + 0.3 * u)).clamp(0.0, 1.0));
            }
        }
        assert_eq!(seen, [true; 3]);
    }

    /// `MutateHebbianRule` and `MutateRewardSource` draw among the others of
    /// the picked member's value and write it to every target.
    #[test]
    fn group_wide_rule_and_source_redraws_write_the_drawn_value() {
        let configs = [
            Some(modulated()),
            Some(with(modulated(), |c| {
                c.rule = HebbianRule::Covariance;
                c.modulation = Some(RewardModulationConfig {
                    reward_source: OutcomeChannel::ActionSuccess,
                    trace_decay: 0.7,
                });
            })),
        ];
        let (original, members) = group_def(&configs);
        let mut seen = false;
        for seed in 0..64 {
            let mut def = original.clone();
            let mut expected = rng(seed);
            let (picked, coin) = replay_pick(&original, is_plastic, &mut expected);
            mutate_hebbian_rule_in_def(&mut def, &mut rng(seed)).unwrap();
            let picked_rule = original.edge_at(picked).plasticity().unwrap().rule;
            let rule = def.edge_at(picked).plasticity().unwrap().rule;
            assert_ne!(rule, picked_rule);
            if coin == Some(true) {
                seen = true;
                assert!(members[..2]
                    .iter()
                    .all(|&m| def.edge_at(m).plasticity().unwrap().rule == rule));
            }
            let mut def = original.clone();
            mutate_reward_source_in_def(&mut def, &mut rng(seed)).unwrap();
            let source = |def: &CgpGraphBackendDef, m| {
                def.edge_at(m)
                    .plasticity()
                    .unwrap()
                    .modulation
                    .unwrap()
                    .reward_source
            };
            assert_ne!(source(&def, picked), source(&original, picked));
            if coin == Some(true) {
                assert_eq!(source(&def, members[0]), source(&def, members[1]));
            }
        }
        assert!(seen);
    }

    /// `SetScope`: one member, the coin, then the other of the picked
    /// member's scope on it alone or on every member; no value draw.
    #[test]
    fn set_scope_flips_one_member_or_every_member() {
        let (mut original, members) = group_def(&[Some(hebbian())]);
        original.edge_mut(members[2]).set_scope(Scope::Group);
        let mut tails = [false; 2];
        for seed in 0..64 {
            let mut expected = rng(seed);
            let (picked, coin) = replay_pick(&original, is_member, &mut expected);
            let coin = coin.expect("a member draws the coin");
            let mut def = original.clone();
            let mut actual = rng(seed);
            set_scope_in_def(&mut def, &mut actual).unwrap();
            assert_eq!(actual.gen::<u64>(), expected.gen::<u64>(), "no value draw");
            let other = original.edge_at(picked).membership().unwrap().scope.other();
            let scope = |m: EdgeOccurrence| def.edge_at(m).membership().unwrap().scope;
            if coin {
                assert!(members.iter().all(|&m| scope(m) == other));
            } else {
                assert_eq!(scope(picked), other);
                assert!(members
                    .iter()
                    .filter(|&&m| m != picked)
                    .all(|&m| def.edge_at(m) == original.edge_at(m)));
            }
            assert_eq!(def.members(GroupId::new(0)).count(), 8);
            tails[usize::from(coin)] = true;
        }
        assert_eq!(tails, [true, true]);
        let mut ungrouped = mixed_def();
        assert_eq!(
            set_scope_in_def(&mut ungrouped, &mut rng(0)),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }
}
