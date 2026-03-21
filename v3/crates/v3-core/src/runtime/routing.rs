use crate::contracts::{NodeId, RouteTarget, MAX_GATE_SLOTS};
use crate::runtime::types::sanitize_f32;

/// Internal routing decision produced by node execution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RouteDecision {
    /// VM semantics: floor to i64 with NaN/Inf handling, then wrap with rem_euclid.
    VmWrap { raw_value: f32 },
    /// CGP semantics: normalized binning in [0, target_count - 1].
    CgpNormalized { raw_value: f32 },
}

/// Resolve an internal route decision into a concrete target index.
#[inline]
#[must_use]
pub(crate) fn resolve_route_index(target_count: usize, route: RouteDecision) -> usize {
    if target_count == 0 {
        return 0;
    }

    match route {
        RouteDecision::VmWrap { raw_value } => {
            let route_idx_i64: i64 = if raw_value.is_nan() {
                -1
            } else if raw_value == f32::INFINITY {
                i64::MAX
            } else if raw_value == f32::NEG_INFINITY {
                i64::MIN
            } else {
                raw_value.clamp(i64::MIN as f32, i64::MAX as f32).floor() as i64
            };
            route_idx_i64.rem_euclid(target_count as i64) as usize
        }
        RouteDecision::CgpNormalized { raw_value } => {
            let clamped = sanitize_f32(raw_value).clamp(0.0, 1.0);
            let idx = (clamped * target_count as f32).floor() as usize;
            idx.min(target_count - 1)
        }
    }
}

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
    if targets.is_empty() {
        return None;
    }
    if targets.len() == 1 {
        return Some((0, targets[0].target_id));
    }

    let mut best_idx = 0;
    let mut best_score = f32::NEG_INFINITY;
    for (i, target) in targets.iter().enumerate() {
        let runtime = if (target.slot as usize) < MAX_GATE_SLOTS {
            gates.scores[target.slot as usize]
        } else {
            0.0
        };
        let effective = target.gate_bias + runtime;
        if effective > best_score {
            best_score = effective;
            best_idx = i;
        }
    }
    Some((best_idx, targets[best_idx].target_id))
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
    fn vm_wrap_handles_negative_and_wraps() {
        let idx = resolve_route_index(4, RouteDecision::VmWrap { raw_value: -1.0 });
        assert_eq!(idx, 3);
    }

    #[test]
    fn vm_wrap_handles_infinities_and_nan() {
        assert_eq!(
            resolve_route_index(
                5,
                RouteDecision::VmWrap {
                    raw_value: f32::INFINITY
                }
            ),
            (i64::MAX.rem_euclid(5)) as usize
        );
        assert_eq!(
            resolve_route_index(
                5,
                RouteDecision::VmWrap {
                    raw_value: f32::NEG_INFINITY
                }
            ),
            (i64::MIN.rem_euclid(5)) as usize
        );
        assert_eq!(
            resolve_route_index(
                5,
                RouteDecision::VmWrap {
                    raw_value: f32::NAN
                }
            ),
            4
        );
    }

    #[test]
    fn cgp_normalized_bins_and_clamps() {
        assert_eq!(
            resolve_route_index(8, RouteDecision::CgpNormalized { raw_value: -1.0 }),
            0
        );
        assert_eq!(
            resolve_route_index(8, RouteDecision::CgpNormalized { raw_value: 0.0 }),
            0
        );
        assert_eq!(
            resolve_route_index(8, RouteDecision::CgpNormalized { raw_value: 0.499 }),
            3
        );
        assert_eq!(
            resolve_route_index(8, RouteDecision::CgpNormalized { raw_value: 1.0 }),
            7
        );
        assert_eq!(
            resolve_route_index(
                8,
                RouteDecision::CgpNormalized {
                    raw_value: f32::INFINITY
                }
            ),
            7
        );
    }

    #[test]
    fn route_gate_map_default_is_all_zeros() {
        let map = RouteGateMap::default();
        assert!(map.scores.iter().all(|&s| s == 0.0));
    }
}
