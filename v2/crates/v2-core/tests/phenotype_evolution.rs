use v2_core::phenotype::{FOUNDER_PHENOTYPE_RGB, Phenotype, inherit_phenotype};

#[test]
fn founders_share_single_baseline_phenotype() {
    let founders = (0..16).map(|_| Phenotype::founder()).collect::<Vec<_>>();
    assert!(
        founders
            .iter()
            .all(|founder| founder.rgb == FOUNDER_PHENOTYPE_RGB)
    );
    assert!(
        founders
            .windows(2)
            .all(|pair| pair[0].channel_weights == pair[1].channel_weights)
    );
    assert!(
        founders
            .windows(2)
            .all(|pair| pair[0].channel_positive_increment == pair[1].channel_positive_increment)
    );
}

#[test]
fn phenotype_mutation_is_deterministic_for_fixed_seed() {
    let parent = Phenotype::founder();
    let first = inherit_phenotype(parent, true, 42);
    let second = inherit_phenotype(parent, true, 42);
    assert_eq!(first, second);
}
