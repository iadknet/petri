//! Learning mutation operators (T25.F06, T25.F09, T25.F12).
//!
//! Every operator draws one target uniformly among the node's eligible
//! targets ([`Target`]): edges in `edge_sites` order over both surfaces,
//! then bundle slots in `BundleId` order, `scale_learning` before
//! `member_default` (count-then-select, one draw). A picked member then
//! draws the broadcast coin `gen_bool(0.5)`: true lands the edit on every
//! member of its bundle the same predicate admits, in canonical order;
//! lone edges and slots draw none. A value is drawn once after the coin
//! from the picked target's stored learning and every target applies its
//! law to its own:
//! - EnableHebbian: a fresh learning on a lone edge without one, a member
//!   not `Own` (it becomes `Own`) or an absent slot
//! - DisableHebbian: a lone edge or a slot loses its learning; a member not
//!   `Static` becomes `Static`
//! - ClearLearningOverride: a member not `Inherit` becomes `Inherit`
//! - MutateHebbianRule, MutateHebbianRate, ToggleHebbianLamarckian,
//!   EnableRewardModulation, DisableRewardModulation, MutateRewardSource,
//!   MutateTraceDecay: change a stored learning (lone, `Own`, present slot)
//! - EnableRetention, DisableRetention, MutateRetention (T25.F09): move a
//!   stored learning's retention `ρ` on and off 1 and step its deficit
//!   `1 − ρ`
//! - ToggleCredits: the other of the picked member's `credits`

use rand::{Rng, RngCore};

use crate::contracts::OutcomeChannel;
use crate::genome::cgp::{
    BundleId, CgpGraphBackendDef, EdgeOccurrence, EdgeRole, LearningSlot, Membership,
};
use crate::genome::{
    retention_decays, HebbianRule, Learning, MemberPolicy, RewardModulationConfig,
};
use crate::mutation::event::{Broadcast, MutationSkipReason};

// ─── Targets and applicability predicates ───────────────────────────────────
//
// `GraphMutator::applies_to` filters Graph-backend nodes by [`any_target`]
// over the operator's predicate, so an operator is never selected onto a
// module with no target it can touch. A `ClearSlot` edge never matches
// (`CgpGraphBackendDef::can_learn`).

/// What a learning edit lands on (T25.F12 invariant 9): an edge (lone or a
/// member) or one of a bundle's two slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Target {
    Edge(EdgeOccurrence),
    Slot(BundleId, LearningSlot),
}

/// Every target of `def` in draw order: edges, then slots.
fn targets(def: &CgpGraphBackendDef) -> impl Iterator<Item = Target> + '_ {
    def.edge_sites().map(Target::Edge).chain(
        def.indexed_bundles()
            .flat_map(|(id, _)| LearningSlot::ALL.map(|slot| Target::Slot(id, slot))),
    )
}

/// The learning `target` stores: a lone edge's, an `Own` member's override,
/// a slot's.
fn stored(def: &CgpGraphBackendDef, target: Target) -> Option<Learning> {
    match target {
        Target::Edge(occurrence) => def.edge_at(occurrence).stored_learning(),
        Target::Slot(id, slot) => def.bundles()[id.index()].slot(slot),
    }
}

/// The membership at `target`, `None` for a lone edge or a slot.
fn membership(def: &CgpGraphBackendDef, target: Target) -> Option<Membership> {
    match target {
        Target::Edge(occurrence) => def.edge_at(occurrence).membership(),
        Target::Slot(..) => None,
    }
}

/// The member policy at `target`, `None` for a lone edge or a slot.
fn policy(def: &CgpGraphBackendDef, target: Target) -> Option<MemberPolicy> {
    membership(def, target).map(|membership| membership.policy)
}

/// `EnableHebbian`: a lone edge without learning that may learn, a member
/// not `Own`, an absent slot.
pub(super) fn can_enable_hebbian(def: &CgpGraphBackendDef, target: Target) -> bool {
    match target {
        Target::Edge(occurrence) => match def.edge_at(occurrence).role() {
            EdgeRole::Lone(learning) => learning.is_none() && def.can_learn(occurrence),
            EdgeRole::Member(membership) => !matches!(membership.policy, MemberPolicy::Own(_)),
        },
        Target::Slot(..) => stored(def, target).is_none(),
    }
}

/// `DisableHebbian`: a lone edge with learning, a member not `Static`, a
/// present slot.
pub(super) fn can_disable_hebbian(def: &CgpGraphBackendDef, target: Target) -> bool {
    match policy(def, target) {
        Some(policy) => policy != MemberPolicy::Static,
        None => stored(def, target).is_some(),
    }
}

/// `ClearLearningOverride`: a member not `Inherit`.
pub(super) fn can_clear_override(def: &CgpGraphBackendDef, target: Target) -> bool {
    policy(def, target).is_some_and(|policy| policy != MemberPolicy::Inherit)
}

/// `MutateHebbianRule`, `MutateHebbianRate`, `ToggleHebbianLamarckian`: a
/// stored learning.
pub(super) fn stores_learning(def: &CgpGraphBackendDef, target: Target) -> bool {
    stored(def, target).is_some()
}

/// `EnableRewardModulation`: a stored learning without modulation.
pub(super) fn can_enable_reward_modulation(def: &CgpGraphBackendDef, target: Target) -> bool {
    stored(def, target).is_some_and(|learning| learning.modulation.is_none())
}

/// `DisableRewardModulation`, `MutateRewardSource`, `MutateTraceDecay`: a
/// stored learning with modulation.
pub(super) fn is_reward_modulated(def: &CgpGraphBackendDef, target: Target) -> bool {
    stored(def, target).is_some_and(|learning| learning.modulation.is_some())
}

/// `EnableRetention`: a stored learning whose retention is exactly 1.
pub(super) fn can_enable_retention(def: &CgpGraphBackendDef, target: Target) -> bool {
    stored(def, target).is_some_and(|learning| !retention_decays(learning.retention))
}

/// `DisableRetention`, `MutateRetention`: a stored learning whose retention
/// is below 1.
pub(super) fn is_decaying(def: &CgpGraphBackendDef, target: Target) -> bool {
    stored(def, target).is_some_and(|learning| retention_decays(learning.retention))
}

/// `ToggleCredits`, `DetachGraphEdge`: a bundle member.
pub(super) fn is_member(def: &CgpGraphBackendDef, target: Target) -> bool {
    policy(def, target).is_some()
}

/// A target predicate of this module.
pub(super) type TargetPredicate = fn(&CgpGraphBackendDef, Target) -> bool;

/// Whether any target of `def` matches `predicate`: what an operator's
/// `applies_to` and [`select_eligible`] need to succeed.
pub(super) fn any_target(def: &CgpGraphBackendDef, predicate: TargetPredicate) -> bool {
    targets(def).any(|target| predicate(def, target))
}

/// Pick a random target matching `predicate` in draw order, or
/// `NoApplicableTarget` if none match. Two-pass count-then-select with one
/// draw avoids allocating the eligible list.
pub(super) fn select_eligible(
    def: &CgpGraphBackendDef,
    predicate: TargetPredicate,
    rng: &mut impl Rng,
) -> Result<Target, MutationSkipReason> {
    let matching = || targets(def).filter(|&target| predicate(def, target));
    let count = matching().count();
    if count == 0 {
        return Err(MutationSkipReason::NoApplicableTarget);
    }
    let target = rng.gen_range(0..count);
    matching()
        .nth(target)
        .ok_or(MutationSkipReason::NoApplicableTarget)
}

/// The broadcast coin (T25.F12 invariant 9): the reach of an edit picked at
/// `picked` and its targets. A lone edge or a slot draws no coin and is the
/// only target; a member draws `gen_bool(0.5)`: false, itself alone; true,
/// every member of its bundle that `admits` accepts, in canonical order.
pub(super) fn broadcast_targets(
    def: &CgpGraphBackendDef,
    picked: Target,
    admits: TargetPredicate,
    rng: &mut impl Rng,
) -> (Broadcast, Vec<Target>) {
    match membership(def, picked).map(|membership| membership.bundle) {
        Some(bundle) if rng.gen_bool(0.5) => (
            Broadcast::Members,
            def.members(bundle)
                .map(Target::Edge)
                .filter(|&member| admits(def, member))
                .collect(),
        ),
        _ => (Broadcast::One, vec![picked]),
    }
}

/// Write `learning` as `target`'s stored learning: a lone edge's or a
/// slot's; on a member, `Some` an `Own` override and `None` `Static`.
fn write(def: &mut CgpGraphBackendDef, target: Target, learning: Option<Learning>) {
    match target {
        Target::Edge(occurrence) => {
            let is_member = def.edge_at(occurrence).membership().is_some();
            let mut edge = def.edge_mut(occurrence);
            if is_member {
                edge.set_policy(learning.map_or(MemberPolicy::Static, MemberPolicy::Own));
            } else {
                edge.set_learning(learning);
            }
        }
        Target::Slot(id, slot) => def.bundle_mut(id).set_slot(slot, learning),
    }
}

/// A learning law split into its draws and its per-target apply: the value
/// is drawn once, after the broadcast coin, from the picked target's stored
/// learning, and every target applies it to its own.
struct Law<V> {
    draw: fn(Option<Learning>, &mut dyn RngCore) -> V,
    apply: fn(Option<Learning>, &V) -> Option<Learning>,
}

/// Pick one target matching `predicate`, draw the broadcast coin, draw the
/// law's value once and write it to every target; returns the reach.
fn mutate_broadcast<V, R: Rng>(
    def: &mut CgpGraphBackendDef,
    predicate: TargetPredicate,
    rng: &mut R,
    law: Law<V>,
) -> Result<Broadcast, MutationSkipReason> {
    let picked = select_eligible(def, predicate, rng)?;
    let (broadcast, targets) = broadcast_targets(def, picked, predicate, rng);
    let value = (law.draw)(stored(def, picked), rng);
    for target in targets {
        let learning = (law.apply)(stored(def, target), &value);
        write(def, target, learning);
    }
    Ok(broadcast)
}

/// `EnableHebbian`'s value law: a fresh pure-Hebbian learning, in draw
/// order rule, rate, clamp, inheritance.
pub(super) fn draw_hebbian_config(rng: &mut dyn RngCore) -> Learning {
    let rule = HebbianRule::ALL[rng.gen_range(0..HebbianRule::ALL.len())];
    Learning {
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
    let channel = OutcomeChannel::ALL[rng.gen_range(0..OutcomeChannel::ALL.len())];
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

fn draw_u(_: Option<Learning>, rng: &mut dyn RngCore) -> f32 {
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

/// `EnableHebbian`: one fresh learning, written to every admitted target.
pub(crate) fn enable_hebbian_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Broadcast, MutationSkipReason> {
    mutate_broadcast(
        def,
        can_enable_hebbian,
        rng,
        Law {
            draw: |_, rng| draw_hebbian_config(rng),
            apply: |_, &learning| Some(learning),
        },
    )
}

/// `DisableHebbian`: every target loses its stored learning; a member
/// becomes `Static`.
pub(crate) fn disable_hebbian_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Broadcast, MutationSkipReason> {
    mutate_broadcast(
        def,
        can_disable_hebbian,
        rng,
        Law {
            draw: |_, _| (),
            apply: |_, ()| None,
        },
    )
}

/// `ClearLearningOverride` (T25.F12): every target member becomes
/// `Inherit`; no value draw.
pub(crate) fn clear_learning_override_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Broadcast, MutationSkipReason> {
    let picked = select_eligible(def, can_clear_override, rng)?;
    let (broadcast, targets) = broadcast_targets(def, picked, can_clear_override, rng);
    for target in targets {
        if let Target::Edge(occurrence) = target {
            def.edge_mut(occurrence).set_policy(MemberPolicy::Inherit);
        }
    }
    Ok(broadcast)
}

/// `ToggleCredits` (T25.F12): the other of the picked member's `credits`,
/// written to every target; no value draw.
pub(crate) fn toggle_credits_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Broadcast, MutationSkipReason> {
    let picked = select_eligible(def, is_member, rng)?;
    let credits = !membership(def, picked)
        .ok_or(MutationSkipReason::NoApplicableTarget)?
        .credits;
    let (broadcast, targets) = broadcast_targets(def, picked, is_member, rng);
    for target in targets {
        if let Target::Edge(occurrence) = target {
            def.edge_mut(occurrence).set_credits(credits);
        }
    }
    Ok(broadcast)
}

/// Change the rule of a stored learning: one of the others of the picked
/// target's rule, written to every target.
pub(crate) fn mutate_hebbian_rule_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Broadcast, MutationSkipReason> {
    mutate_broadcast(
        def,
        stores_learning,
        rng,
        Law {
            draw: |picked, rng| {
                draw_other(&HebbianRule::ALL, picked.map(|picked| picked.rule), rng)
            },
            apply: |learning, &rule| learning.map(|learning| Learning { rule, ..learning }),
        },
    )
}

/// Perturb a stored learning's rate: one `u`, each target stepping its own
/// rate by its branch.
pub(crate) fn mutate_hebbian_rate_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Broadcast, MutationSkipReason> {
    mutate_broadcast(
        def,
        stores_learning,
        rng,
        Law {
            draw: draw_u,
            apply: |learning, &u| {
                learning.map(|learning| Learning {
                    learning_rate: perturb(learning.learning_rate, u),
                    ..learning
                })
            },
        },
    )
}

/// Flip a stored learning's `lamarckian` flag: every target takes the
/// picked target's flipped value.
pub(crate) fn toggle_hebbian_lamarckian_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Broadcast, MutationSkipReason> {
    mutate_broadcast(
        def,
        stores_learning,
        rng,
        Law {
            draw: |picked, _| !picked.is_some_and(|picked| picked.lamarckian),
            apply: |learning, &lamarckian| {
                learning.map(|learning| Learning {
                    lamarckian,
                    ..learning
                })
            },
        },
    )
}

/// Add reward modulation to a stored learning without it: one modulation,
/// set on every target, which keeps its rule, rate, clamp and flag.
pub(crate) fn enable_reward_modulation_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Broadcast, MutationSkipReason> {
    mutate_broadcast(
        def,
        can_enable_reward_modulation,
        rng,
        Law {
            draw: |_, rng| draw_modulation(rng),
            apply: |learning, &modulation| {
                learning.map(|learning| Learning {
                    modulation: Some(modulation),
                    ..learning
                })
            },
        },
    )
}

/// Remove reward modulation from a modulated stored learning, keeping the
/// rest of each target's learning.
pub(crate) fn disable_reward_modulation_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Broadcast, MutationSkipReason> {
    mutate_broadcast(
        def,
        is_reward_modulated,
        rng,
        Law {
            draw: |_, _| (),
            apply: |learning, ()| {
                learning.map(|learning| Learning {
                    modulation: None,
                    ..learning
                })
            },
        },
    )
}

/// Switch a modulated stored learning's reward source: one of the others
/// of the picked target's channel, written to every target.
pub(crate) fn mutate_reward_source_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Broadcast, MutationSkipReason> {
    mutate_broadcast(
        def,
        is_reward_modulated,
        rng,
        Law {
            draw: |picked, rng| {
                let current = picked
                    .and_then(|picked| picked.modulation)
                    .map(|m| m.reward_source);
                draw_other(&OutcomeChannel::ALL, current, rng)
            },
            apply: |learning, &reward_source| {
                learning.map(|learning| Learning {
                    modulation: learning
                        .modulation
                        .map(|modulation| RewardModulationConfig {
                            reward_source,
                            ..modulation
                        }),
                    ..learning
                })
            },
        },
    )
}

/// Perturb a modulated stored learning's trace decay: one `u`, each target
/// stepping its own decay by its branch.
pub(crate) fn mutate_trace_decay_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Broadcast, MutationSkipReason> {
    mutate_broadcast(
        def,
        is_reward_modulated,
        rng,
        Law {
            draw: draw_u,
            apply: |learning, &u| {
                learning.map(|learning| Learning {
                    modulation: learning
                        .modulation
                        .map(|modulation| RewardModulationConfig {
                            trace_decay: perturb(modulation.trace_decay, u),
                            ..modulation
                        }),
                    ..learning
                })
            },
        },
    )
}

/// `EnableRetention` (T25.F09): one retention, drawn once after the coin,
/// written to every admitted target.
pub(crate) fn enable_retention_in_def(
    def: &mut CgpGraphBackendDef,
    rng: &mut impl Rng,
) -> Result<Broadcast, MutationSkipReason> {
    mutate_broadcast(
        def,
        can_enable_retention,
        rng,
        Law {
            draw: |_, rng| draw_retention(rng),
            apply: |learning, &retention| {
                learning.map(|learning| Learning {
                    retention,
                    ..learning
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
) -> Result<Broadcast, MutationSkipReason> {
    mutate_broadcast(
        def,
        is_decaying,
        rng,
        Law {
            draw: |_, _| (),
            apply: |learning, ()| {
                learning.map(|learning| Learning {
                    retention: 1.0,
                    ..learning
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
) -> Result<Broadcast, MutationSkipReason> {
    mutate_broadcast(
        def,
        is_decaying,
        rng,
        Law {
            draw: draw_u,
            apply: |learning, &u| {
                learning.map(|learning| Learning {
                    retention: step_retention(learning.retention, u),
                    ..learning
                })
            },
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::votes::VoteSink;
    use crate::genome::cgp::test_support::plastic_node;
    use crate::genome::cgp::{
        ComputeNode, ComputeNodeKind, EdgeSurface, GraphEdge, GraphSource, OutputSink,
        OutputSinkKind,
    };
    use crate::genome::BackendDef;
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

    fn hebbian() -> Learning {
        Learning {
            rule: HebbianRule::Classic,
            learning_rate: 0.1,
            weight_clamp: 5.0,
            lamarckian: false,
            modulation: None,
            retention: 1.0,
        }
    }

    fn modulated() -> Learning {
        Learning {
            modulation: Some(RewardModulationConfig {
                reward_source: OutcomeChannel::EnergyDelta,
                trace_decay: 0.9,
            }),
            ..hebbian()
        }
    }

    fn node(config: Option<Learning>) -> ComputeNode {
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
    fn first(def: &CgpGraphBackendDef) -> Option<Learning> {
        def.compute_nodes()[0].inputs[0].stored_learning()
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
        assert!(!any_target(&def, can_enable_hebbian));
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

    type Operator =
        fn(&mut CgpGraphBackendDef, &mut SmallRng) -> Result<Broadcast, MutationSkipReason>;

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
                    .set_learning(config);
            }
        }
        def
    }

    /// The learning operators over their predicates.
    const OPERATORS: [(&str, Operator, TargetPredicate); 12] = [
        ("enable", enable_hebbian_in_def, can_enable_hebbian),
        ("disable", disable_hebbian_in_def, can_disable_hebbian),
        ("rule", mutate_hebbian_rule_in_def, stores_learning),
        ("rate", mutate_hebbian_rate_in_def, stores_learning),
        (
            "lamarckian",
            toggle_hebbian_lamarckian_in_def,
            stores_learning,
        ),
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
        (
            "enable_retention",
            enable_retention_in_def,
            can_enable_retention,
        ),
        ("disable_retention", disable_retention_in_def, is_decaying),
        ("retention", mutate_retention_in_def, is_decaying),
    ];

    /// Invariant 9: each operator picks one lone edge uniformly among the
    /// edges matching its predicate, in `edge_sites` order over both
    /// surfaces, with one draw, and edits that edge alone; across seeds it
    /// reaches both surfaces.
    #[test]
    fn every_operator_draws_one_matching_edge_over_both_surfaces() {
        let mut original = mixed_def();
        // A decaying edge per surface for the retention steps.
        for target in [
            EdgeOccurrence::new(EdgeSurface::ComputeInput(1), 0),
            EdgeOccurrence::new(
                original
                    .sink_surface(OutputSinkKind::ActionVote(VoteSink::Eat))
                    .unwrap(),
                0,
            ),
        ] {
            let learning = original.edge_at(target).stored_learning().unwrap();
            original.edge_mut(target).set_learning(Some(Learning {
                retention: 0.5,
                ..learning
            }));
        }
        for (name, operator, predicate) in OPERATORS {
            let matching: Vec<Target> = targets(&original)
                .filter(|&target| predicate(&original, target))
                .collect();
            assert!(!matching.is_empty(), "{name}");
            let mut surfaces = [false; 2];
            for seed in 0..64 {
                let expected = matching[rng(seed).gen_range(0..matching.len())];
                let mut def = original.clone();
                let reach = operator(&mut def, &mut rng(seed)).unwrap();
                assert_eq!(reach, Broadcast::One, "{name}: a lone edge reaches one");
                let edited: Vec<Target> = def
                    .edge_sites()
                    .filter(|&occurrence| def.edge_at(occurrence) != original.edge_at(occurrence))
                    .map(Target::Edge)
                    .collect();
                // A rule or source redraw always changes; a perturbation can
                // round to the same value, which edits nothing.
                assert!(
                    edited.is_empty() || edited == vec![expected],
                    "{name} seed {seed}"
                );
                if let Some(&Target::Edge(occurrence)) = edited.first() {
                    surfaces
                        [usize::from(matches!(occurrence.surface, EdgeSurface::SinkInput(_)))] =
                        true;
                }
                assert_eq!(
                    crate::genome::cgp::validate_graph(&crate::genome::NodeGenome {
                        node_id: crate::contracts::NodeId::new(0),
                        input_refs: Vec::new(),
                        backend_def: BackendDef::Graph(def),
                        targets: Vec::new(),
                    }),
                    Ok(()),
                    "{name} seed {seed}"
                );
            }
            assert_eq!(surfaces, [true, true], "{name} reaches both surfaces");
        }
    }

    // ── Members, slots and the broadcast coin (T25.F12 invariant 9) ─────────

    use crate::contracts::votes::DirectedVoteKind;
    use crate::genome::cgp::test_support::store_learning;
    use crate::genome::cgp::BundleId;

    /// A full catalog with one ring bundle of eight members on `Move`, member
    /// `k` an `Own` member of `configs[k]` (`Static` for `None` and past the
    /// list's end), both slots absent.
    fn bundle_def(configs: &[Option<Learning>]) -> (CgpGraphBackendDef, Vec<EdgeOccurrence>) {
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs();
        let id = def.add_ring_bundle(1, DirectedVoteKind::Move, 0);
        let members: Vec<EdgeOccurrence> = def.members(id).collect();
        for (&member, &config) in members.iter().zip(configs) {
            store_learning(&mut def, member, config);
        }
        (def, members)
    }

    /// The selection draws of an operator over `predicate` on `def`: the
    /// picked target and, for a member, the coin. `rng` continues at the
    /// value draws.
    fn replay_pick(
        def: &CgpGraphBackendDef,
        predicate: TargetPredicate,
        rng: &mut SmallRng,
    ) -> (Target, Option<bool>) {
        let matching: Vec<Target> = targets(def)
            .filter(|&target| predicate(def, target))
            .collect();
        let picked = matching[rng.gen_range(0..matching.len())];
        let coin = match picked {
            Target::Edge(at) => def.edge_at(at).membership().map(|_| rng.gen_bool(0.5)),
            Target::Slot(..) => None,
        };
        (picked, coin)
    }

    /// Every target `op` changed: edges, then slots.
    fn edited(before: &CgpGraphBackendDef, after: &CgpGraphBackendDef) -> Vec<Target> {
        let edges = before
            .edge_sites()
            .filter(|&occurrence| before.edge_at(occurrence) != after.edge_at(occurrence))
            .map(Target::Edge);
        let slots = targets(before).filter(|&target| {
            matches!(target, Target::Slot(..)) && stored(before, target) != stored(after, target)
        });
        edges.chain(slots).collect()
    }

    fn with(config: Learning, edit: impl FnOnce(&mut Learning)) -> Learning {
        let mut config = config;
        edit(&mut config);
        config
    }

    fn the_member(target: Target) -> EdgeOccurrence {
        match target {
            Target::Edge(at) => at,
            Target::Slot(..) => panic!("a member"),
        }
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
            assert!(def
                .edges()
                .any(|edge| edge.stored_learning() == Some(config)));
        }
    }

    /// `EnableHebbian` on a bundle with an `Own` member: the coin lands one
    /// config on the picked member or on every member not `Own`, each
    /// becoming `Own`; the rejected member is untouched; a picked slot
    /// draws no coin and the other slot does not change.
    #[test]
    fn enable_hebbian_lands_one_config_on_the_admitted_targets() {
        let (original, members) = bundle_def(&[None, None, None, Some(modulated())]);
        let mut seen = [false; 3];
        for seed in 0..96 {
            let mut expected = rng(seed);
            let (picked, coin) = replay_pick(&original, can_enable_hebbian, &mut expected);
            let config = draw_hebbian_config(&mut expected);
            let mut def = original.clone();
            let mut actual = rng(seed);
            let reach = enable_hebbian_in_def(&mut def, &mut actual).unwrap();
            assert_eq!(actual.gen::<u64>(), expected.gen::<u64>(), "seed {seed}");
            let want: Vec<Target> = match coin {
                Some(true) => members
                    .iter()
                    .copied()
                    .filter(|&m| m != members[3])
                    .map(Target::Edge)
                    .collect(),
                _ => vec![picked],
            };
            assert_eq!(edited(&original, &def), want, "seed {seed}");
            assert!(want
                .iter()
                .all(|&target| stored(&def, target) == Some(config)));
            assert_eq!(def.edge_at(members[3]), original.edge_at(members[3]));
            assert_eq!(
                reach,
                if coin == Some(true) {
                    Broadcast::Members
                } else {
                    Broadcast::One
                }
            );
            seen[match coin {
                None => 2,
                Some(coin) => usize::from(coin),
            }] = true;
        }
        assert_eq!(seen, [true; 3], "one, members and a slot");
    }

    /// T25.F10 invariant 3: a broadcast selected on a direction member of a
    /// fact-bearing `DirectionPair` gene lands on the gene's fact members as
    /// on its direction members.
    #[test]
    fn a_bundle_broadcast_from_a_direction_member_reaches_the_fact_members() {
        use crate::contracts::{InputReference, WorldInputKey};
        let layout = crate::contracts::pair_layout(
            &InputReference::World(WorldInputKey::AreaOccupancySummary),
            0,
        );
        let mut original = CgpGraphBackendDef::new_with_fixed_outputs();
        let id = original.add_pair_bundle(0, 0, layout, DirectedVoteKind::Move, 3);
        let (facts, directions): (Vec<_>, Vec<_>) = original
            .members(id)
            .partition(|&member| original.site_of(member) == Some(crate::genome::cgp::Site::Drive));
        assert_eq!(facts.len(), layout.facts.len());
        assert!(!facts.is_empty() && !directions.is_empty());
        let mut broadcasts = 0;
        for seed in 0..128 {
            let mut expected = rng(seed);
            let (picked, coin) = replay_pick(&original, can_enable_hebbian, &mut expected);
            let Target::Edge(picked) = picked else {
                continue;
            };
            if original.site_of(picked) == Some(crate::genome::cgp::Site::Drive)
                || coin != Some(true)
            {
                continue;
            }
            let config = draw_hebbian_config(&mut expected);
            let mut def = original.clone();
            let reach = enable_hebbian_in_def(&mut def, &mut rng(seed)).unwrap();
            assert_eq!(reach, Broadcast::Members, "seed {seed}");
            for &member in facts.iter().chain(&directions) {
                assert_eq!(
                    def.edge_at(member).stored_learning(),
                    Some(config),
                    "seed {seed} member {member:?}"
                );
            }
            broadcasts += 1;
        }
        assert!(broadcasts > 0, "a direction member's broadcast was drawn");
    }

    /// `EnableRewardModulation` on a heterogeneous bundle sets one modulation
    /// on every admitted member, each keeping its rule, rate, clamp and flag;
    /// `DisableRewardModulation` clears the modulation alone.
    #[test]
    fn bundle_wide_modulation_changes_keep_each_members_config() {
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
        let (original, members) = bundle_def(&configs);
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
                assert_eq!(def.edge_at(members[k]).stored_learning(), Some(want));
            }
            disable_reward_modulation_in_def(&mut def, &mut rng(seed)).unwrap();
            let cleared = members[..configs.len()]
                .iter()
                .filter(|&&m| {
                    def.edge_at(m)
                        .stored_learning()
                        .unwrap()
                        .modulation
                        .is_none()
                })
                .count();
            assert!(cleared == 1 || cleared == configs.len(), "{cleared}");
            for k in 0..configs.len() {
                let config = def.edge_at(members[k]).stored_learning().unwrap();
                if config.modulation.is_none() {
                    assert_eq!(Some(config), configs[k], "the config kept");
                }
            }
        }
        assert!(all, "a bundle-wide tail");
    }

    /// `ToggleHebbianLamarckian` writes the picked member's flipped flag to
    /// every target; the rate and decay steps draw one `u` and apply it on
    /// each member's own branch, on both sides of 0.01.
    #[test]
    fn bundle_wide_toggle_and_perturbations_follow_the_picked_value_and_each_branch() {
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
        let (original, members) = bundle_def(&configs);
        let mut seen = [false; 3];
        for seed in 0..64 {
            let mut expected = rng(seed);
            let (picked, coin) = replay_pick(&original, stores_learning, &mut expected);
            let mut def = original.clone();
            toggle_hebbian_lamarckian_in_def(&mut def, &mut rng(seed)).unwrap();
            let flag = !stored(&original, picked).unwrap().lamarckian;
            if coin == Some(true) {
                seen[0] = true;
                assert!(members[..2].iter().all(|&m| def
                    .edge_at(m)
                    .stored_learning()
                    .unwrap()
                    .lamarckian
                    == flag));
            }

            let mut expected = rng(seed);
            let (_, coin) = replay_pick(&original, stores_learning, &mut expected);
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
                let rate = |m: usize| {
                    def.edge_at(members[m])
                        .stored_learning()
                        .unwrap()
                        .learning_rate
                };
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
                        .stored_learning()
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
    fn bundle_wide_rule_and_source_redraws_write_the_drawn_value() {
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
        let (original, members) = bundle_def(&configs);
        let mut seen = false;
        for seed in 0..64 {
            let mut def = original.clone();
            let mut expected = rng(seed);
            let (picked, coin) = replay_pick(&original, stores_learning, &mut expected);
            let picked = the_member(picked);
            mutate_hebbian_rule_in_def(&mut def, &mut rng(seed)).unwrap();
            let picked_rule = original.edge_at(picked).stored_learning().unwrap().rule;
            let rule = def.edge_at(picked).stored_learning().unwrap().rule;
            assert_ne!(rule, picked_rule);
            if coin == Some(true) {
                seen = true;
                assert!(members[..2].iter().all(|&m| def
                    .edge_at(m)
                    .stored_learning()
                    .unwrap()
                    .rule
                    == rule));
            }
            let mut def = original.clone();
            mutate_reward_source_in_def(&mut def, &mut rng(seed)).unwrap();
            let source = |def: &CgpGraphBackendDef, m| {
                def.edge_at(m)
                    .stored_learning()
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

    /// `ToggleCredits`: one member, the coin, then the other of the picked
    /// member's `credits` on it alone or on every member; no value draw;
    /// nothing else changes.
    #[test]
    fn toggle_credits_flips_one_member_or_every_member() {
        let (mut original, members) = bundle_def(&[Some(hebbian())]);
        original.edge_mut(members[2]).set_credits(true);
        let mut tails = [false; 2];
        for seed in 0..64 {
            let mut expected = rng(seed);
            let (picked, coin) = replay_pick(&original, is_member, &mut expected);
            let (picked, coin) = (the_member(picked), coin.expect("a member draws the coin"));
            let mut def = original.clone();
            let mut actual = rng(seed);
            let reach = toggle_credits_in_def(&mut def, &mut actual).unwrap();
            assert_eq!(actual.gen::<u64>(), expected.gen::<u64>(), "no value draw");
            let other = !original.edge_at(picked).membership().unwrap().credits;
            let credits = |m: EdgeOccurrence| def.edge_at(m).membership().unwrap().credits;
            let policy = |d: &CgpGraphBackendDef, m: EdgeOccurrence| {
                d.edge_at(m).membership().unwrap().policy
            };
            assert!(members
                .iter()
                .all(|&m| policy(&def, m) == policy(&original, m)));
            if coin {
                assert_eq!(reach, Broadcast::Members);
                assert!(members.iter().all(|&m| credits(m) == other));
            } else {
                assert_eq!(reach, Broadcast::One);
                assert_eq!(credits(picked), other);
                assert!(members
                    .iter()
                    .filter(|&&m| m != picked)
                    .all(|&m| def.edge_at(m) == original.edge_at(m)));
            }
            assert_eq!(def.members(BundleId::new(0)).count(), 8);
            tails[usize::from(coin)] = true;
        }
        assert_eq!(tails, [true, true]);
        let mut lone = mixed_def();
        assert_eq!(
            toggle_credits_in_def(&mut lone, &mut rng(0)),
            Err(MutationSkipReason::NoApplicableTarget)
        );
    }

    /// `ClearLearningOverride`: a member not `Inherit` becomes `Inherit`, on
    /// its own or across the bundle; an `Inherit` member, a lone edge and a
    /// slot are never targets; no value draw.
    #[test]
    fn clear_learning_override_returns_members_to_inherit() {
        let (mut original, members) = bundle_def(&[Some(hebbian()), None]);
        original
            .edge_mut(members[2])
            .set_policy(MemberPolicy::Inherit);
        let mut tails = [false; 2];
        for seed in 0..64 {
            let mut expected = rng(seed);
            let (picked, coin) = replay_pick(&original, can_clear_override, &mut expected);
            let (picked, coin) = (the_member(picked), coin.expect("a member"));
            assert_ne!(picked, members[2]);
            let mut def = original.clone();
            let mut actual = rng(seed);
            clear_learning_override_in_def(&mut def, &mut actual).unwrap();
            assert_eq!(actual.gen::<u64>(), expected.gen::<u64>(), "no value draw");
            let policy = |m: EdgeOccurrence| def.edge_at(m).membership().unwrap().policy;
            if coin {
                assert!(members.iter().all(|&m| policy(m) == MemberPolicy::Inherit));
            } else {
                assert_eq!(edited(&original, &def), vec![Target::Edge(picked)]);
                assert_eq!(policy(picked), MemberPolicy::Inherit);
            }
            tails[usize::from(coin)] = true;
        }
        assert_eq!(tails, [true, true]);
        assert!(!any_target(&mixed_def(), can_clear_override));
    }

    /// The slot learning operator `name` writes from `before`, `rng` at its
    /// value draws: only the operator's field changes, every other field is
    /// `before`'s.
    fn expected_slot_learning(
        name: &str,
        before: Option<Learning>,
        rng: &mut SmallRng,
    ) -> Option<Learning> {
        if name == "enable" {
            return Some(draw_hebbian_config(rng));
        }
        if name == "disable" {
            return None;
        }
        let before = before.expect("a present slot");
        let modulation = before.modulation;
        let modulated =
            |edit: &dyn Fn(RewardModulationConfig) -> RewardModulationConfig| Learning {
                modulation: modulation.map(edit),
                ..before
            };
        Some(match name {
            "rule" => Learning {
                rule: draw_other(&HebbianRule::ALL, Some(before.rule), rng),
                ..before
            },
            "rate" => Learning {
                learning_rate: perturb(before.learning_rate, draw_u(None, rng)),
                ..before
            },
            "lamarckian" => Learning {
                lamarckian: !before.lamarckian,
                ..before
            },
            "enable_modulation" => Learning {
                modulation: Some(draw_modulation(rng)),
                ..before
            },
            "disable_modulation" => Learning {
                modulation: None,
                ..before
            },
            "source" => {
                let current = modulation.map(|m| m.reward_source);
                let reward_source = draw_other(&OutcomeChannel::ALL, current, rng);
                modulated(&|m| RewardModulationConfig { reward_source, ..m })
            }
            "decay" => {
                let u = draw_u(None, rng);
                modulated(&|m| RewardModulationConfig {
                    trace_decay: perturb(m.trace_decay, u),
                    ..m
                })
            }
            "enable_retention" => Learning {
                retention: draw_retention(rng),
                ..before
            },
            "disable_retention" => Learning {
                retention: 1.0,
                ..before
            },
            "retention" => Learning {
                retention: step_retention(before.retention, draw_u(None, rng)),
                ..before
            },
            other => panic!("no slot law for {other}"),
        })
    }

    /// The track guard (T25.F12): every learning-edit operator reaches its
    /// lone-edge, member and both slot targets; an edit picked on a slot
    /// writes the operator's law to that slot alone (its other fields, the
    /// other slot, the record's scale and every member unchanged) and draws
    /// no coin; an edit on a member changes no slot.
    #[test]
    fn every_learning_operator_reaches_lone_edges_members_and_both_slots() {
        let decaying = with(modulated(), |c| c.retention = 0.5);
        let (mut original, members) = bundle_def(&[Some(hebbian()), Some(decaying), None]);
        let id = BundleId::new(0);
        // A lone sink edge beside the bundle, and the slots: one present
        // per operator family, or absent for the enables.
        let surface = original
            .sink_surface(OutputSinkKind::ActionVote(VoteSink::Eat))
            .unwrap();
        original.push_edge(surface, GraphEdge::new(GraphSource::ComputeNode(0), 1.0));
        let lone = EdgeOccurrence::new(surface, 0);
        let slotted = |scale: Option<Learning>, default: Option<Learning>| {
            let mut def = original.clone();
            def.bundle_mut(id).set_slot(LearningSlot::Scale, scale);
            def.bundle_mut(id)
                .set_slot(LearningSlot::MemberDefault, default);
            def
        };
        for (name, operator, predicate) in OPERATORS {
            for (scale, default) in [
                (None, None),
                (Some(hebbian()), Some(decaying)),
                (Some(decaying), Some(hebbian())),
            ] {
                let mut base = slotted(scale, default);
                if name.starts_with("enable") {
                    store_learning(&mut base, lone, None);
                } else {
                    store_learning(&mut base, lone, Some(decaying));
                }
                let mut reached = [false; 4];
                for seed in 0..256 {
                    let mut expected = rng(seed);
                    let Ok(_) = select_eligible(&base, predicate, &mut expected) else {
                        break;
                    };
                    // On a slot, `expected` now sits at the value draws.
                    let (picked, coin) = replay_pick(&base, predicate, &mut rng(seed));
                    let mut def = base.clone();
                    let reach = operator(&mut def, &mut rng(seed)).unwrap();
                    let changed = edited(&base, &def);
                    match picked {
                        Target::Slot(_, slot) => {
                            reached[2 + usize::from(slot == LearningSlot::MemberDefault)] = true;
                            assert_eq!(coin, None);
                            assert_eq!(reach, Broadcast::One, "{name}");
                            assert!(changed.iter().all(|&t| t == picked), "{name} {changed:?}");
                            assert_eq!(
                                stored(&def, picked),
                                expected_slot_learning(name, stored(&base, picked), &mut expected),
                                "{name}"
                            );
                            assert_eq!(def.bundles()[0].scale(), base.bundles()[0].scale());
                        }
                        Target::Edge(at) if at == lone => {
                            reached[0] = true;
                            assert!(changed.iter().all(|&t| t == picked), "{name}");
                        }
                        Target::Edge(_) => {
                            reached[1] = true;
                            assert!(
                                changed
                                    .iter()
                                    .all(|t| matches!(t, Target::Edge(m) if members.contains(m))),
                                "{name} {changed:?}"
                            );
                        }
                    }
                }
                let wanted = [
                    targets(&base).any(|t| t == Target::Edge(lone) && predicate(&base, t)),
                    members.iter().any(|&m| predicate(&base, Target::Edge(m))),
                    predicate(&base, Target::Slot(id, LearningSlot::Scale)),
                    predicate(&base, Target::Slot(id, LearningSlot::MemberDefault)),
                ];
                assert_eq!(reached, wanted, "{name} {scale:?} {default:?}");
            }
        }
    }
}
