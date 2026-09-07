use crate::contracts::{NodeId, RouteTarget, MAX_GATE_SLOTS};

/// Per-slot gate scores produced by node execution.
#[derive(Debug, Clone, Copy, PartialEq)]
#[must_use]
pub(crate) struct RouteGateMap {
    pub scores: [f32; MAX_GATE_SLOTS],
}

impl Default for RouteGateMap {
    fn default() -> Self {
        Self {
            scores: [0.0; MAX_GATE_SLOTS],
        }
    }
}

impl RouteGateMap {
    /// Get the runtime gate score for a slot, returning 0.0 for out-of-range slots.
    #[inline]
    pub fn score_for_slot(&self, slot: u8) -> f32 {
        let s = slot as usize;
        if s < MAX_GATE_SLOTS {
            self.scores[s]
        } else {
            0.0
        }
    }
}

// Hot-path type size assertion — prevent accidental regressions.
const _: () = assert!(std::mem::size_of::<RouteGateMap>() == 32);

/// Resolve which target to route to based on gate scores.
///
/// effective(target) = gate_bias + runtime_gate[target.slot]
/// Winner = argmax. Ties broken by position (first wins via strict `>`).
/// Returns `(winning_index, winning_target_id)` or `None` if empty.
#[inline]
#[must_use]
pub(crate) fn resolve_gated_route(
    targets: &[RouteTarget],
    gates: &RouteGateMap,
) -> Option<(usize, NodeId)> {
    resolve_gated_route_where(targets, gates, |_| true)
}

/// Resolve among eligible destinations, preserving the original float and tie policy.
#[inline]
pub(crate) fn resolve_gated_route_where(
    targets: &[RouteTarget],
    gates: &RouteGateMap,
    eligible: impl Fn(NodeId) -> bool,
) -> Option<(usize, NodeId)> {
    let mut best = None;
    let mut best_score = f32::NEG_INFINITY;
    for (index, target) in targets
        .iter()
        .enumerate()
        .filter(|(_, target)| eligible(target.target_id))
    {
        let effective = target.gate_bias + gates.score_for_slot(target.slot);
        // Even all-NaN/-infinity candidates retain the earliest eligible entry.
        if best.is_none() {
            best = Some((index, target.target_id));
        }
        if effective > best_score {
            best_score = effective;
            best = Some((index, target.target_id));
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::{NodeId, RouteTarget};

    #[test]
    fn gated_route_empty_targets_returns_none() {
        let gates = RouteGateMap::default();
        assert_eq!(resolve_gated_route(&[], &gates), None);
    }

    #[test]
    fn gated_route_single_target_skips_scoring() {
        let targets = [RouteTarget {
            target_id: NodeId::new(5),
            slot: 0,
            gate_bias: -99.0,
        }];
        let gates = RouteGateMap::default();
        assert_eq!(
            resolve_gated_route(&targets, &gates),
            Some((0, NodeId::new(5)))
        );
    }

    #[test]
    fn gated_route_picks_highest_effective_score() {
        let targets = [
            RouteTarget {
                target_id: NodeId::new(1),
                slot: 0,
                gate_bias: 0.0,
            },
            RouteTarget {
                target_id: NodeId::new(2),
                slot: 1,
                gate_bias: 0.0,
            },
        ];
        let mut gates = RouteGateMap::default();
        gates.scores[1] = 1.0;
        assert_eq!(
            resolve_gated_route(&targets, &gates),
            Some((1, NodeId::new(2)))
        );
    }

    #[test]
    fn gated_route_bias_plus_runtime() {
        let targets = [
            RouteTarget {
                target_id: NodeId::new(1),
                slot: 0,
                gate_bias: 2.0,
            },
            RouteTarget {
                target_id: NodeId::new(2),
                slot: 1,
                gate_bias: -1.0,
            },
        ];
        let mut gates = RouteGateMap::default();
        gates.scores[1] = 4.0; // effective: -1.0 + 4.0 = 3.0 > 2.0
        assert_eq!(
            resolve_gated_route(&targets, &gates),
            Some((1, NodeId::new(2)))
        );
    }

    #[test]
    fn gated_route_first_target_wins_ties() {
        let targets = [
            RouteTarget {
                target_id: NodeId::new(1),
                slot: 0,
                gate_bias: 0.0,
            },
            RouteTarget {
                target_id: NodeId::new(2),
                slot: 1,
                gate_bias: 0.0,
            },
        ];
        let gates = RouteGateMap::default();
        assert_eq!(
            resolve_gated_route(&targets, &gates),
            Some((0, NodeId::new(1)))
        );
    }

    #[test]
    fn gated_route_out_of_range_slot_gets_zero_runtime() {
        let targets = [
            RouteTarget {
                target_id: NodeId::new(1),
                slot: 0,
                gate_bias: 0.0,
            },
            RouteTarget {
                target_id: NodeId::new(2),
                slot: 200,
                gate_bias: 1.0,
            },
        ];
        let gates = RouteGateMap::default();
        assert_eq!(
            resolve_gated_route(&targets, &gates),
            Some((1, NodeId::new(2)))
        );
    }

    #[test]
    fn route_gate_map_default_is_all_zeros() {
        let map = RouteGateMap::default();
        assert!(map.scores.iter().all(|&s| s == 0.0));
    }
}
