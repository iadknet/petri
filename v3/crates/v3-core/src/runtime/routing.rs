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

#[cfg(test)]
mod tests {
    use super::*;

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
}
