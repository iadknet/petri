//! Bounded observations of applied energy arithmetic. These never drive execution.

use crate::contracts::{ActionType, ACTION_TYPE_COUNT};

/// The sink that first exhausted a creature since its last positive recovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCause {
    LifecycleDecay,
    GenomeCarrying,
    /// The Phase 0 capacity upkeep of a body above the reference (T26.F02).
    CapacityUpkeep,
    VmCompute,
    GraphCompute,
    MeshRamp,
    PriorityBid,
    ActionNoop,
    ActionEat,
    ActionMove,
    ActionReproduce,
    ActionStealEnergy,
    ActionGiveEnergy,
    ParentalTransfer,
    Predation,
    /// A giver exhausted by its own gift (T27.F03).
    Gift,
    ExternalRemoval,
    Unattributed,
}

impl DeathCause {
    pub const ALL: [Self; 18] = [
        Self::LifecycleDecay,
        Self::GenomeCarrying,
        Self::CapacityUpkeep,
        Self::VmCompute,
        Self::GraphCompute,
        Self::MeshRamp,
        Self::PriorityBid,
        Self::ActionNoop,
        Self::ActionEat,
        Self::ActionMove,
        Self::ActionReproduce,
        Self::ActionStealEnergy,
        Self::ActionGiveEnergy,
        Self::ParentalTransfer,
        Self::Predation,
        Self::Gift,
        Self::ExternalRemoval,
        Self::Unattributed,
    ];

    #[must_use]
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::LifecycleDecay => "lifecycle_decay",
            Self::GenomeCarrying => "genome_carrying",
            Self::CapacityUpkeep => "capacity_upkeep",
            Self::VmCompute => "vm_compute",
            Self::GraphCompute => "graph_compute",
            Self::MeshRamp => "mesh_ramp",
            Self::PriorityBid => "priority_bid",
            Self::ActionNoop => "action_noop",
            Self::ActionEat => "action_eat",
            Self::ActionMove => "action_move",
            Self::ActionReproduce => "action_reproduce",
            Self::ActionStealEnergy => "action_steal_energy",
            Self::ActionGiveEnergy => "action_give_energy",
            Self::ParentalTransfer => "parental_transfer",
            Self::Predation => "predation",
            Self::Gift => "gift",
            Self::ExternalRemoval => "external_removal",
            Self::Unattributed => "unattributed",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MortalityTotals {
    pub deaths_total: u64,
    pub by_cause: [u64; DeathCause::ALL.len()],
}

impl MortalityTotals {
    pub fn record(&mut self, cause: DeathCause) {
        self.deaths_total += 1;
        self.by_cause[cause as usize] += 1;
    }

    #[must_use]
    pub fn count(&self, cause: DeathCause) -> u64 {
        self.by_cause[cause as usize]
    }
}

/// Observe a crossing or a credit that restores positive energy. A zero floor
/// is not recovery, and later costs cannot replace the first exhausting sink.
pub(crate) fn observe_energy_change(
    pending: &mut Option<DeathCause>,
    before: f64,
    after: f64,
    sink: DeathCause,
) {
    if after > 0.0 && after > before {
        *pending = None;
    } else if before > 0.0 && after <= 0.0 && pending.is_none() {
        *pending = Some(sink);
    }
}

#[inline]
pub(crate) fn applied_debit(before: f32, after: f32) -> f64 {
    f64::from(before) - f64::from(after)
}

/// Allocate the actual combined Phase 0 debit by the decay-first convention.
pub(crate) fn decay_partition(before: f32, after: f32, decay: f32) -> (f64, f64) {
    let total = applied_debit(before, after).max(0.0);
    let base = applied_debit(before, before - decay).max(0.0).min(total);
    (base, total - base)
}

/// Dispatch-local observations, accumulated in mesh visit and backend debit order.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CognitionEnergyObservation {
    pub vm_compute: f64,
    pub graph_compute: f64,
    /// Per-tick hop ramp charged in the mesh loop before each dispatch (T19.F01).
    pub mesh_ramp: f64,
    pub priority_bid: f64,
    pub pending_cause: Option<DeathCause>,
}

/// Energy charged per action type, indexed by [`ActionType`] (T16.F02,
/// T27.F03).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ActionCharges([f64; ACTION_TYPE_COUNT as usize]);

/// The share of [`ActionCharges`] paid by attempts that failed (T16.F02): a
/// blocked move, an eat that found no food, a rejected reproduce and a steal
/// with no victim. A subset, not a sink (`failed_action_effort[k] <=
/// action_charges[k]`); `NoOp` cannot fail, so its slot stays 0.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FailedActionEffort([f64; ACTION_TYPE_COUNT as usize]);

macro_rules! action_type_indexed {
    ($ledger:ty) => {
        impl std::ops::Index<ActionType> for $ledger {
            type Output = f64;
            fn index(&self, action: ActionType) -> &f64 {
                &self.0[action.index()]
            }
        }

        impl std::ops::IndexMut<ActionType> for $ledger {
            fn index_mut(&mut self, action: ActionType) -> &mut f64 {
                &mut self.0[action.index()]
            }
        }
    };
}
action_type_indexed!(ActionCharges);
action_type_indexed!(FailedActionEffort);

impl ActionCharges {
    /// The sum over every action type, in [`ActionType::ALL`] order.
    #[must_use]
    pub fn total(&self) -> f64 {
        ActionType::ALL.iter().map(|&action| self[action]).sum()
    }
}

/// Cumulative signed f64 changes of the stored f32 energy at each applied site.
/// Food slots are dense in configured type order; initial founder stock is excluded.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EnergyFlows {
    pub food_intake_by_type: Vec<f64>,
    pub action_charges: ActionCharges,
    pub failed_action_effort: FailedActionEffort,
    pub vm_compute: f64,
    pub priority_bid: f64,
    pub graph_compute: f64,
    pub mesh_ramp: f64,
    pub lifecycle_decay: f64,
    pub genome_carrying: f64,
    /// The signed Phase 0 capacity upkeep (T26.F02): the applied charge less
    /// its decay and carrying shares, negative for a body below the
    /// reference; the zero floor's shortfall lands here.
    pub capacity_upkeep: f64,
    pub genome_size_creature_ticks: u64,
    pub parental_transfer_debit: f64,
    pub offspring_energy_credit: f64,
    /// The share of a delivered litter above the child's capacity, removed by
    /// the birth cap step (T26.F02); `offspring_energy_credit` books the rest.
    pub offspring_capacity_overflow: f64,
    pub predation_victim_debit: f64,
    pub predation_attacker_credit: f64,
    pub predation_kill_bonus_credit: f64,
    /// A giver's applied debit for its gifts (T27.F03).
    pub gift_debit: f64,
    /// A recipient's applied credit for gifts, before its cap.
    pub gift_credit: f64,
    pub maximum_energy_clamp_loss: f64,
    pub zero_floor_credit: f64,
    pub external_removal_loss: f64,
    /// Litter energy of every conceived birth that never became a child
    /// (T03.F12): the parent died during gestation, or delivery found no
    /// valid cell or the population at its cap.
    pub gestation_lost: f64,
}

/// The share of a dispatch-floor `overshoot` that an action's own `charged`
/// effort put below zero: at most the overshoot, at most the charge, never
/// negative. The rest of the overshoot is a deficit the action inherited.
#[must_use]
pub(crate) fn overdrawn_share(overshoot: f64, charged: f64) -> f64 {
    overshoot.min(charged).max(0.0)
}

impl EnergyFlows {
    /// Net the action's overdraft out of its charge, given the charge and
    /// failed-effort totals from before the action and the dispatch floor's
    /// `overshoot`, so a charge records only energy the creature had or
    /// gained. Returns the inherited remainder, a floor credit.
    pub(crate) fn net_action_overdraft(
        &mut self,
        before: (ActionCharges, FailedActionEffort),
        overshoot: f64,
    ) -> f64 {
        let (charges, failed) = before;
        let mut remainder = overshoot;
        for action in ActionType::ALL {
            let charge = &mut self.action_charges[action];
            let share = overdrawn_share(remainder, *charge - charges[action]);
            *charge -= share;
            // A failed attempt's effort is also its failed effort; keep the
            // subset `failed_action_effort[k] <= action_charges[k]`.
            let failed_now = &mut self.failed_action_effort[action];
            if *failed_now > failed[action] {
                *failed_now -= share;
            }
            remainder -= share;
        }
        remainder
    }

    pub(crate) fn record_cognition(&mut self, observation: CognitionEnergyObservation) {
        self.vm_compute += observation.vm_compute;
        self.priority_bid += observation.priority_bid;
        self.graph_compute += observation.graph_compute;
        self.mesh_ramp += observation.mesh_ramp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn cause_keys_follow_the_stable_report_schema(index in 0usize..DeathCause::ALL.len()) {
            let expected = [
                "lifecycle_decay", "genome_carrying", "capacity_upkeep", "vm_compute", "graph_compute",
                "mesh_ramp", "priority_bid", "action_noop",
                "action_eat", "action_move", "action_reproduce", "action_steal_energy",
                "action_give_energy", "parental_transfer", "predation", "gift",
                "external_removal", "unattributed",
            ];
            prop_assert_eq!(DeathCause::ALL[index].as_key(), expected[index]);
        }

        #[test]
        fn unchanged_energy_preserves_pending_attribution(
            positive in 0.001f64..1.0e6,
            cause in 0usize..DeathCause::ALL.len(),
        ) {
            for energy in [-positive, 0.0, positive] {
                for initial in [None, Some(DeathCause::ALL[cause])] {
                    let mut pending = initial;
                    observe_energy_change(&mut pending, energy, energy, DeathCause::VmCompute);
                    prop_assert_eq!(pending, initial);
                }
            }
        }

        #[test]
        fn mortality_counts_partition_every_removal(causes in prop::collection::vec(0usize..DeathCause::ALL.len(), 0..200)) {
            let mut totals = MortalityTotals::default();
            for &index in &causes {
                totals.record(DeathCause::ALL[index]);
            }
            prop_assert_eq!(totals.deaths_total, causes.len() as u64);
            prop_assert_eq!(totals.by_cause.iter().sum::<u64>(), totals.deaths_total);
            for (index, &count) in totals.by_cause.iter().enumerate() {
                prop_assert_eq!(count, causes.iter().filter(|&&c| c == index).count() as u64);
            }
        }

        #[test]
        fn overdrawn_share_is_bounded_by_overshoot_and_charge(
            overshoot in -1.0e9f64..1.0e9,
            charged in -1.0e9f64..1.0e9,
        ) {
            let share = overdrawn_share(overshoot, charged);
            prop_assert!(share >= 0.0);
            prop_assert!(share <= overshoot.max(0.0));
            prop_assert!(share <= charged.max(0.0));
        }

        #[test]
        fn netting_an_overdraft_conserves_it_and_keeps_failed_within_charge(
            row in 0usize..ActionType::ALL.len(),
            failed_attempt in any::<bool>(),
            charged in 0.0f64..1.0e8,
            overshoot in 0.0f64..1.0e8,
        ) {
            let action = ActionType::ALL[row];
            let mut flows = EnergyFlows::default();
            let before = (flows.action_charges, flows.failed_action_effort);
            flows.action_charges[action] += charged;
            // NoOp cannot fail: its failed slot is never written.
            if failed_attempt && action != ActionType::NoOp {
                flows.failed_action_effort[action] += charged;
            }
            let remainder = flows.net_action_overdraft(before, overshoot);
            let c = flows.action_charges;
            let f = flows.failed_action_effort;
            let total = c.total();
            prop_assert!(total >= 0.0);
            prop_assert!(remainder >= 0.0);
            prop_assert!(((charged - total) + remainder - overshoot).abs() <= 1e-6 * overshoot.max(1.0));
            for action in ActionType::ALL {
                prop_assert!(f[action] <= c[action]);
            }
            prop_assert_eq!(f[ActionType::NoOp], 0.0);
        }

        #[test]
        fn pending_cause_retains_first_crossing_until_positive_recovery(
            positive in 0.001f64..1.0e6,
            overshoot in -1.0e6f64..=0.0,
            first in 0usize..DeathCause::ALL.len(),
            second in 0usize..DeathCause::ALL.len(),
        ) {
            let mut pending = None;
            observe_energy_change(&mut pending, positive, overshoot, DeathCause::ALL[first]);
            prop_assert_eq!(pending, Some(DeathCause::ALL[first]));
            observe_energy_change(&mut pending, overshoot, overshoot - 1.0, DeathCause::ALL[second]);
            observe_energy_change(&mut pending, overshoot - 1.0, 0.0, DeathCause::ALL[second]);
            prop_assert_eq!(pending, Some(DeathCause::ALL[first]));
            observe_energy_change(&mut pending, 0.0, positive, DeathCause::ALL[second]);
            prop_assert_eq!(pending, None);
            observe_energy_change(&mut pending, positive, 0.0, DeathCause::ALL[second]);
            prop_assert_eq!(pending, Some(DeathCause::ALL[second]));
        }

        #[test]
        fn phase_zero_partition_preserves_the_one_applied_debit(
            before in -10000.0f32..10000.0,
            decay in 0.0f32..1000.0,
            rate in 0.0f32..10.0,
            size in 0u32..100000,
        ) {
            let after = before - (decay + rate * size as f32);
            let (base, carrying) = decay_partition(before, after, decay);
            let applied = f64::from(before) - f64::from(after);
            prop_assert!(base >= 0.0 && carrying >= 0.0);
            // Exact equality is the accounting invariant for stored f32 endpoints.
            prop_assert_eq!(base + carrying, applied);
            let (base_only, zero_carrying) = decay_partition(before, before - decay, decay);
            prop_assert_eq!(zero_carrying, 0.0);
            prop_assert_eq!(base_only, f64::from(before) - f64::from(before - decay));
        }
    }

    #[test]
    fn rounded_away_debits_are_zero_and_overshoots_are_not_clipped() {
        assert_eq!(applied_debit(100.0, 100.0 - f32::EPSILON), 0.0);
        assert_eq!(applied_debit(1.0, -2.0), 3.0);
    }
}
