use super::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;

mod copy;
mod extensions;
mod operators;

fn rng(seed: u64) -> SmallRng {
    SmallRng::seed_from_u64(seed)
}

#[test]
fn complexity_effect_consistent_with_types() {
    use crate::mutation::types::ComplexityEffect;
    for &op in &GraphOperator::ALL {
        let effect = op.complexity_effect();
        assert!(
            matches!(
                effect,
                ComplexityEffect::Increasing
                    | ComplexityEffect::Decreasing
                    | ComplexityEffect::Neutral
            ),
            "complexity_effect must return valid effect for {:?}",
            op
        );
    }
}

#[test]
fn random_non_increasing_never_returns_increasing() {
    use crate::mutation::types::ComplexityEffect;
    for seed in 0u64..200 {
        let mut r = rng(seed);
        if let Some(op) = GraphOperator::random_non_increasing(&mut r) {
            assert_ne!(
                op.complexity_effect(),
                ComplexityEffect::Increasing,
                "random_non_increasing returned Increasing operator {:?} at seed {}",
                op,
                seed
            );
        }
    }
}

#[test]
fn random_non_increasing_covers_neutral_and_decreasing() {
    use crate::mutation::types::ComplexityEffect;
    let mut saw_neutral = false;
    let mut saw_decreasing = false;
    for seed in 0u64..1000 {
        let mut r = rng(seed);
        if let Some(op) = GraphOperator::random_non_increasing(&mut r) {
            match op.complexity_effect() {
                ComplexityEffect::Neutral => saw_neutral = true,
                ComplexityEffect::Decreasing => saw_decreasing = true,
                ComplexityEffect::Increasing => unreachable!(),
            }
        }
        if saw_neutral && saw_decreasing {
            break;
        }
    }
    assert!(saw_neutral, "must produce at least one neutral operator");
    assert!(
        saw_decreasing,
        "must produce at least one decreasing operator"
    );
}

#[test]
fn random_decreasing_never_returns_non_decreasing() {
    use crate::mutation::types::ComplexityEffect;
    for seed in 0u64..200 {
        let mut r = rng(seed);
        if let Some(op) = GraphOperator::random_decreasing(&mut r) {
            assert_eq!(
                op.complexity_effect(),
                ComplexityEffect::Decreasing,
                "random_decreasing returned non-Decreasing operator {:?} at seed {}",
                op,
                seed
            );
        }
    }
}

#[test]
fn random_decreasing_covers_all_decreasing_operators() {
    use std::collections::HashSet;
    let expected: HashSet<GraphOperator> = GraphOperator::ALL
        .iter()
        .copied()
        .filter(|op| op.complexity_effect().is_decreasing())
        .collect();
    let mut seen = HashSet::new();
    for seed in 0u64..2000 {
        let mut r = rng(seed);
        if let Some(op) = GraphOperator::random_decreasing(&mut r) {
            seen.insert(op);
        }
        if seen == expected {
            break;
        }
    }
    assert_eq!(
        seen, expected,
        "random_decreasing must cover all Decreasing operators"
    );
}
