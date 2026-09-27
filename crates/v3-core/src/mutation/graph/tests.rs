use super::*;

mod copy;
mod extensions;
mod f08;
mod operators;

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
fn graph_operator_weight_total_matches_the_frozen_catalog() {
    assert_eq!(GraphOperator::TOTAL_WEIGHT, 44);
}
