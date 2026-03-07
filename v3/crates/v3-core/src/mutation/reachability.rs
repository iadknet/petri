//! Reachability-aware mutation target selection.
//!
//! Provides biased node selection that preferentially targets mesh nodes reachable
//! from the entry node while preserving drift on unreachable structure.
//!
//! The parent's cached reachable set is used to bias mutations on the offspring.
//! After mutations, the offspring's actual reachable set may differ. This is
//! intentional — mutations are biased toward what was functional in the parent.
//! The offspring gets a fresh BFS via `CreatureState::new()`.

use rand::Rng;

use super::types::TargetReachability;

/// Select a node index from `eligible` with probabilistic bias toward reachable nodes.
///
/// Both `eligible` and `reachable` must be sorted ascending.
///
/// - `bias = 0.0`: uniform selection from `eligible` (no reachability preference).
/// - `bias = 1.0`: always select from reachable nodes when any are eligible.
///
/// Returns `(selected_index, classification)` or `None` if `eligible` is empty.
#[must_use]
pub fn biased_select_from(
    eligible: &[usize],
    reachable: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Option<(usize, TargetReachability)> {
    if eligible.is_empty() {
        return None;
    }

    let bias_clamped = if bias.is_finite() {
        bias.clamp(0.0, 1.0)
    } else {
        0.0
    };

    // Attempt biased selection toward reachable nodes
    if bias_clamped > 0.0 && rng.gen_bool(bias_clamped) {
        let count = intersection_count(eligible, reachable);
        if count > 0 {
            let pick = rng.gen_range(0..count);
            let idx = nth_intersection(eligible, reachable, pick);
            return Some((idx, TargetReachability::Reachable));
        }
        // No reachable nodes in eligible set — fall through to uniform
    }

    // Uniform selection from all eligible nodes
    let picked = eligible[rng.gen_range(0..eligible.len())];
    let classification = classify_target(picked, reachable);
    Some((picked, classification))
}

/// Classify whether a node index is reachable.
#[must_use]
pub fn classify_target(node_idx: usize, reachable: &[usize]) -> TargetReachability {
    if reachable.binary_search(&node_idx).is_ok() {
        TargetReachability::Reachable
    } else {
        TargetReachability::Unreachable
    }
}

/// Count elements in the intersection of two sorted slices (two-pointer merge).
fn intersection_count(a: &[usize], b: &[usize]) -> usize {
    let (mut i, mut j, mut count) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                count += 1;
                i += 1;
                j += 1;
            }
        }
    }
    count
}

/// Find the k-th element (0-based) in the intersection of two sorted slices.
///
/// Panics if `k >= intersection_count(a, b)`.
fn nth_intersection(a: &[usize], b: &[usize], k: usize) -> usize {
    let (mut i, mut j, mut found) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                if found == k {
                    return a[i];
                }
                found += 1;
                i += 1;
                j += 1;
            }
        }
    }
    panic!("k={k} out of range for intersection");
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn seeded_rng(seed: u64) -> SmallRng {
        SmallRng::seed_from_u64(seed)
    }

    #[test]
    fn empty_eligible_returns_none() {
        let mut rng = seeded_rng(0);
        assert!(biased_select_from(&[], &[0, 1], 0.5, &mut rng).is_none());
    }

    #[test]
    fn bias_zero_selects_uniformly() {
        let eligible = vec![0, 1, 2, 3, 4];
        let reachable = vec![0, 1]; // only 0, 1 reachable
        let mut rng = seeded_rng(42);
        let mut selected_unreachable = false;
        for _ in 0..200 {
            let (idx, _) = biased_select_from(&eligible, &reachable, 0.0, &mut rng).unwrap();
            if idx >= 2 {
                selected_unreachable = true;
                break;
            }
        }
        assert!(
            selected_unreachable,
            "bias=0.0 should eventually select unreachable nodes"
        );
    }

    #[test]
    fn bias_one_always_selects_reachable_when_available() {
        let eligible = vec![0, 1, 2, 3, 4];
        let reachable = vec![1, 3]; // only 1, 3 reachable
        let mut rng = seeded_rng(99);
        for _ in 0..500 {
            let (idx, class) =
                biased_select_from(&eligible, &reachable, 1.0, &mut rng).unwrap();
            assert!(
                idx == 1 || idx == 3,
                "bias=1.0 must select reachable, got {idx}"
            );
            assert_eq!(class, TargetReachability::Reachable);
        }
    }

    #[test]
    fn bias_one_falls_back_when_no_reachable_eligible() {
        let eligible = vec![2, 4, 6];
        let reachable = vec![1, 3, 5]; // no overlap
        let mut rng = seeded_rng(7);
        let (idx, class) = biased_select_from(&eligible, &reachable, 1.0, &mut rng).unwrap();
        assert!(eligible.contains(&idx));
        assert_eq!(class, TargetReachability::Unreachable);
    }

    #[test]
    fn classify_reachable_node() {
        assert_eq!(classify_target(3, &[1, 3, 5]), TargetReachability::Reachable);
    }

    #[test]
    fn classify_unreachable_node() {
        assert_eq!(
            classify_target(4, &[1, 3, 5]),
            TargetReachability::Unreachable
        );
    }

    #[test]
    fn classify_empty_reachable() {
        assert_eq!(classify_target(0, &[]), TargetReachability::Unreachable);
    }

    #[test]
    fn statistical_bias_0_7() {
        let eligible: Vec<usize> = (0..10).collect();
        let reachable = vec![0, 1, 2, 3, 4]; // 5 of 10 reachable
        let mut rng = seeded_rng(12345);
        let n = 5000;
        let mut reachable_count = 0u32;
        for _ in 0..n {
            let (_, class) =
                biased_select_from(&eligible, &reachable, 0.7, &mut rng).unwrap();
            if class == TargetReachability::Reachable {
                reachable_count += 1;
            }
        }
        let rate = f64::from(reachable_count) / n as f64;
        // Expected: 0.7 * 1.0 + 0.3 * 0.5 = 0.85 (70% biased always-reachable + 30% uniform 50/50)
        assert!(
            (0.80..=0.90).contains(&rate),
            "expected reachable rate in [0.80, 0.90], got {rate:.3}"
        );
    }

    #[test]
    fn intersection_count_basic() {
        assert_eq!(intersection_count(&[1, 3, 5], &[2, 3, 5, 7]), 2);
        assert_eq!(intersection_count(&[], &[1, 2]), 0);
        assert_eq!(intersection_count(&[1, 2], &[]), 0);
        assert_eq!(intersection_count(&[1, 2, 3], &[1, 2, 3]), 3);
    }

    #[test]
    fn nth_intersection_basic() {
        assert_eq!(nth_intersection(&[1, 3, 5, 7], &[3, 5, 9], 0), 3);
        assert_eq!(nth_intersection(&[1, 3, 5, 7], &[3, 5, 9], 1), 5);
    }
}
