//! Outcome accumulation for reward-modulated plasticity.
//!
//! Tracks what happened to each creature during a tick (energy changes, action
//! results, damage, offspring) and converts them into an [`OutcomeSignalBank`]
//! for the Phase 2.5 reward learning pass.
//!
//! All types and methods are wired into `tick.rs` in Stage 2.

use crate::contracts::CreatureId;
use crate::creature::genome::{OutcomeChannel, OUTCOME_CHANNEL_COUNT};
use crate::runtime::plasticity::OutcomeSignalBank;
use slotmap::SecondaryMap;

/// Per-creature accumulator for tick-level outcomes.
///
/// Populated during Phase 1 (energy snapshot) and Phase 2 (action execution).
/// Consumed in Phase 2.5 to compute [`OutcomeSignalBank`].
#[derive(Debug, Clone, Default)]
pub(crate) struct OutcomeRecord {
    /// Energy at the start of the tick (before cognition).
    pub energy_before: f32,
    /// Number of actions attempted this tick.
    pub actions_attempted: u32,
    /// Number of actions that succeeded this tick.
    pub actions_succeeded: u32,
    /// Cumulative damage received (energy stolen by predators) this tick.
    pub damage_received: f32,
    /// Number of offspring successfully spawned this tick.
    pub offspring_spawned: u32,
}

/// Tick-level outcome accumulator for all creatures.
///
/// Currently created fresh per tick. If moved into `Simulation` for
/// cross-tick reuse, call [`clear`](OutcomeAccumulator::clear) to avoid
/// reallocation (`mem-reuse-collections`).
#[derive(Debug, Clone, Default)]
pub(crate) struct OutcomeAccumulator {
    records: SecondaryMap<CreatureId, OutcomeRecord>,
}

impl OutcomeAccumulator {
    /// Reset for a new tick without deallocating.
    #[allow(dead_code)] // Used when accumulator is stored in Simulation for reuse.
    pub(crate) fn clear(&mut self) {
        self.records.clear();
    }

    /// Snapshot a creature's energy at the start of the tick.
    pub(crate) fn snapshot_energy(&mut self, id: CreatureId, energy: f32) {
        self.records.insert(
            id,
            OutcomeRecord {
                energy_before: energy,
                ..Default::default()
            },
        );
    }

    /// Record the result of an action attempt.
    pub(crate) fn record_action_result(&mut self, id: CreatureId, succeeded: bool) {
        if let Some(rec) = self.records.get_mut(id) {
            rec.actions_attempted += 1;
            if succeeded {
                rec.actions_succeeded += 1;
            }
        }
    }

    /// Record damage received by a creature (e.g. from predation).
    pub(crate) fn record_damage(&mut self, id: CreatureId, amount: f32) {
        if let Some(rec) = self.records.get_mut(id) {
            rec.damage_received += amount;
        }
    }

    /// Record a successful offspring spawn.
    pub(crate) fn record_offspring(&mut self, id: CreatureId) {
        if let Some(rec) = self.records.get_mut(id) {
            rec.offspring_spawned += 1;
        }
    }

    /// Compute the outcome signal bank for a creature given its current energy.
    ///
    /// Returns `None` if no record exists for the creature (e.g. newborns
    /// spawned during this tick — they have no outcome accumulator entry).
    pub(crate) fn compute_signal_bank(
        &self,
        id: CreatureId,
        energy_after: f32,
    ) -> Option<OutcomeSignalBank> {
        let rec = self.records.get(id)?;
        let mut signals = [0.0f32; OUTCOME_CHANNEL_COUNT];

        // EnergyDelta: signed change in energy over the tick.
        signals[OutcomeChannel::EnergyDelta as usize] = energy_after - rec.energy_before;

        // ActionSuccess: fraction of actions that succeeded (0.0 if no actions attempted).
        signals[OutcomeChannel::ActionSuccess as usize] = if rec.actions_attempted > 0 {
            rec.actions_succeeded as f32 / rec.actions_attempted as f32
        } else {
            0.0
        };

        // DamageDelta: negated damage received (negative = bad).
        signals[OutcomeChannel::DamageDelta as usize] = -rec.damage_received;

        // OffspringSuccess: count of offspring spawned (0 or positive).
        signals[OutcomeChannel::OffspringSuccess as usize] = rec.offspring_spawned as f32;

        Some(OutcomeSignalBank { signals })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::SlotMap;

    fn make_id() -> CreatureId {
        let mut sm: SlotMap<CreatureId, ()> = SlotMap::with_key();
        sm.insert(())
    }

    #[test]
    fn empty_accumulator_returns_none() {
        let acc = OutcomeAccumulator::default();
        let id = make_id();
        assert!(acc.compute_signal_bank(id, 50.0).is_none());
    }

    #[test]
    fn energy_delta_computed_correctly() {
        let mut acc = OutcomeAccumulator::default();
        let id = make_id();
        acc.snapshot_energy(id, 100.0);
        let bank = acc.compute_signal_bank(id, 120.0).unwrap();
        assert!((bank.signals[OutcomeChannel::EnergyDelta as usize] - 20.0).abs() < 1e-6);
    }

    #[test]
    fn action_success_fraction() {
        let mut acc = OutcomeAccumulator::default();
        let id = make_id();
        acc.snapshot_energy(id, 50.0);
        acc.record_action_result(id, true);
        acc.record_action_result(id, false);
        acc.record_action_result(id, true);
        let bank = acc.compute_signal_bank(id, 50.0).unwrap();
        // 2/3 succeeded
        let expected = 2.0 / 3.0;
        assert!((bank.signals[OutcomeChannel::ActionSuccess as usize] - expected).abs() < 1e-6);
    }

    #[test]
    fn no_actions_gives_zero_success() {
        let mut acc = OutcomeAccumulator::default();
        let id = make_id();
        acc.snapshot_energy(id, 50.0);
        let bank = acc.compute_signal_bank(id, 50.0).unwrap();
        assert!((bank.signals[OutcomeChannel::ActionSuccess as usize] - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn damage_delta_is_negated() {
        let mut acc = OutcomeAccumulator::default();
        let id = make_id();
        acc.snapshot_energy(id, 50.0);
        acc.record_damage(id, 10.0);
        acc.record_damage(id, 5.0);
        let bank = acc.compute_signal_bank(id, 35.0).unwrap();
        assert!((bank.signals[OutcomeChannel::DamageDelta as usize] - (-15.0)).abs() < 1e-6);
    }

    #[test]
    fn offspring_success_counts() {
        let mut acc = OutcomeAccumulator::default();
        let id = make_id();
        acc.snapshot_energy(id, 50.0);
        acc.record_offspring(id);
        acc.record_offspring(id);
        let bank = acc.compute_signal_bank(id, 50.0).unwrap();
        assert!(
            (bank.signals[OutcomeChannel::OffspringSuccess as usize] - 2.0).abs() < f32::EPSILON
        );
    }

    #[test]
    fn clear_resets_accumulator() {
        let mut acc = OutcomeAccumulator::default();
        let id = make_id();
        acc.snapshot_energy(id, 100.0);
        acc.clear();
        assert!(acc.compute_signal_bank(id, 100.0).is_none());
    }

    #[test]
    fn outcome_signal_bank_default_is_zero() {
        let bank = OutcomeSignalBank::default();
        assert_eq!(bank.signals, [0.0; OUTCOME_CHANNEL_COUNT]);
    }

    #[test]
    fn record_on_missing_id_is_noop() {
        let mut acc = OutcomeAccumulator::default();
        let id = make_id();
        // No snapshot — these should be no-ops, not panics.
        acc.record_action_result(id, true);
        acc.record_damage(id, 5.0);
        acc.record_offspring(id);
        assert!(acc.compute_signal_bank(id, 50.0).is_none());
    }
}
