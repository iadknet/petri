use v2_core::phenotype::{
    FOUNDER_PHENOTYPE_RGB, PHENOTYPE_RGB_CHANNEL_STEP, Phenotype, inherit_phenotype,
};

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

#[test]
fn phenotype_mutation_changes_one_channel_by_configured_step() {
    let parent = Phenotype::founder();
    let child = inherit_phenotype(parent, true, 7);

    let changed = parent
        .rgb
        .iter()
        .zip(child.rgb.iter())
        .filter(|(before, after)| before != after)
        .count();
    assert_eq!(changed, 1);

    let delta = parent
        .rgb
        .iter()
        .zip(child.rgb.iter())
        .map(|(before, after)| i16::from(*after) - i16::from(*before))
        .find(|difference| *difference != 0)
        .expect("one channel should change");
    assert_eq!(delta.abs(), i16::from(PHENOTYPE_RGB_CHANNEL_STEP));
}
