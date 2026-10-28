use super::*;

mod birth;
mod bundle;
mod copy;
mod extensions;
mod f08;
mod operators;
mod retention;

#[test]
fn complexity_effect_consistent_with_types() {
    use crate::mutation::event::ComplexityEffect;
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
