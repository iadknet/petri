use crate::contracts::WorldAction;

/// Result returned by a single node evaluation.
/// The mesh executor uses this to decide routing and final WorldAction.
#[must_use]
#[derive(Debug, Clone, PartialEq)]
pub struct NodeResult {
    /// Output slots for downstream nodes. Initialized from incoming upstream_slots;
    /// only slots written by the node are overwritten.
    pub output_slots: [f32; 12],
    /// Routing target index (f32). Mesh executor applies rem_euclid over targets.len().
    pub route_target_idx: f32,
    /// World action emitted by this node, if any.
    pub world_action: Option<WorldAction>,
    /// True when the node was halted due to energy exhaustion.
    /// When true, the mesh executor MUST return WorldAction::NoOp immediately.
    pub energy_exhausted: bool,
}

impl NodeResult {
    /// Create a no-action result (halt or step cap reached).
    pub fn halted(output_slots: [f32; 12], route_target_idx: f32) -> Self {
        Self {
            output_slots,
            route_target_idx,
            world_action: None,
            energy_exhausted: false,
        }
    }

    /// Create a result indicating energy exhaustion.
    pub fn exhausted() -> Self {
        Self {
            output_slots: [0.0; 12],
            route_target_idx: 0.0,
            world_action: None,
            energy_exhausted: true,
        }
    }

    /// Create a result with an emitted world action.
    pub fn action(output_slots: [f32; 12], route_target_idx: f32, action: WorldAction) -> Self {
        Self {
            output_slots,
            route_target_idx,
            world_action: Some(action),
            energy_exhausted: false,
        }
    }
}

/// Energy cost attributed to VM and graph node execution during one mesh evaluation.
///
/// Returned alongside the [`WorldAction`] by [`execute_creature_mesh`].
/// Does not include lifecycle decay, move, eat, noop, or reproduce costs.
#[derive(Debug, Clone, Default)]
pub struct ComputeCostReport {
    /// Total energy deducted from executing VM nodes this tick.
    pub vm_cost: f32,
    /// Total energy deducted from executing Graph nodes this tick.
    pub graph_cost: f32,
}

/// Sanitize an f32 value per v3-vm-isa-spec.md Section 5:
/// - NaN → 0.0
/// - +Inf → +1_000_000_000.0
/// - -Inf → -1_000_000_000.0
/// - Finite values clamped to [-1e9, 1e9]
#[inline]
#[must_use]
pub fn sanitize_f32(v: f32) -> f32 {
    const CLAMP: f32 = 1_000_000_000.0;
    if v.is_nan() {
        0.0
    } else {
        v.clamp(-CLAMP, CLAMP)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_nan_becomes_zero() {
        assert_eq!(sanitize_f32(f32::NAN), 0.0);
    }

    #[test]
    fn sanitize_pos_inf_becomes_clamp() {
        assert_eq!(sanitize_f32(f32::INFINITY), 1_000_000_000.0);
    }

    #[test]
    fn sanitize_neg_inf_becomes_neg_clamp() {
        assert_eq!(sanitize_f32(f32::NEG_INFINITY), -1_000_000_000.0);
    }

    #[test]
    fn sanitize_large_finite_clamps() {
        assert_eq!(sanitize_f32(2e9), 1_000_000_000.0);
        assert_eq!(sanitize_f32(-2e9), -1_000_000_000.0);
    }

    #[test]
    fn sanitize_normal_value_unchanged() {
        assert!((sanitize_f32(1.5) - 1.5).abs() < 1e-6);
        assert!((sanitize_f32(-0.5) - (-0.5)).abs() < 1e-6);
        assert_eq!(sanitize_f32(0.0), 0.0);
    }

    #[test]
    fn sanitize_exactly_at_boundary_unchanged() {
        assert_eq!(sanitize_f32(1_000_000_000.0), 1_000_000_000.0);
        assert_eq!(sanitize_f32(-1_000_000_000.0), -1_000_000_000.0);
    }

    #[test]
    fn node_result_halted_has_no_action() {
        let r = NodeResult::halted([0.0; 12], 0.0);
        assert!(r.world_action.is_none());
        assert!(!r.energy_exhausted);
    }

    #[test]
    fn node_result_exhausted_has_flag() {
        let r = NodeResult::exhausted();
        assert!(r.energy_exhausted);
        assert!(r.world_action.is_none());
    }

    #[test]
    fn node_result_action_carries_action() {
        let r = NodeResult::action([0.0; 12], 0.0, WorldAction::Eat);
        assert_eq!(r.world_action, Some(WorldAction::Eat));
        assert!(!r.energy_exhausted);
    }
}
