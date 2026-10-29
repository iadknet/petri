//! The body chromosome (T29.F02 invariant 1): its genes, the colour gene's
//! validation, the deterministic gene operations against the arithmetic they
//! replace, and the genome's serialization with the body.

use proptest::prelude::*;

use super::*;
use crate::creature::founder::{default_founder_body, v3alpha1_founder_genome};
use crate::genome::CreatureGenome;

fn colour() -> impl Strategy<Value = Phenotype> {
    (any::<[u8; 6]>(), 0usize..6, any::<[bool; 6]>()).prop_map(|(channels, active, polarity)| {
        Phenotype::new(channels, active, polarity).expect("an in-range active channel")
    })
}

fn body() -> impl Strategy<Value = BodyGenome> {
    (
        colour(),
        any::<u32>(),
        50.0f32..=800.0,
        0.01f32..=1.0,
        0.01f32..=1.0,
    )
        .prop_map(|(colour, kin_tag, capacity, steal, give)| {
            BodyGenome::new(colour, kin_tag, capacity, steal, give)
        })
}

const RANGES: BodyRanges = BodyRanges {
    capacity: (50.0, 800.0),
    share: (0.01, 1.0),
};

#[test]
fn the_body_is_five_genes_in_chromosome_order() {
    assert_eq!(BodyGenome::UNITS, 5);
    assert_eq!(
        BodyGene::ALL,
        [
            BodyGene::Colour,
            BodyGene::KinTag,
            BodyGene::Capacity,
            BodyGene::StealShare,
            BodyGene::GiveShare,
        ]
    );
}

#[test]
fn an_active_channel_outside_the_six_is_rejected_at_construction_and_decode() {
    assert_eq!(
        Phenotype::new([0; 6], 6, [true; 6]),
        Err(ActiveChannelOutOfRange(6))
    );
    let mut json = serde_json::to_value(default_founder_body()).unwrap();
    json["colour"]["active_channel"] = serde_json::json!(6);
    let error = serde_json::from_value::<BodyGenome>(json).unwrap_err();
    assert!(error.to_string().contains("active channel 6"), "{error}");
    let mut genome = serde_json::to_value(v3alpha1_founder_genome()).unwrap();
    genome["body"]["colour"]["active_channel"] = serde_json::json!(9);
    assert!(serde_json::from_value::<CreatureGenome>(genome).is_err());
}

#[test]
fn a_genome_without_a_body_does_not_decode() {
    let mut genome = serde_json::to_value(v3alpha1_founder_genome()).unwrap();
    genome.as_object_mut().unwrap().remove("body");
    assert!(serde_json::from_value::<CreatureGenome>(genome).is_err());
}

#[test]
fn genome_equality_and_debug_include_the_body() {
    let genome = v3alpha1_founder_genome();
    let other = genome
        .clone()
        .with_body(default_founder_body().with_capacity(51.0));
    assert_ne!(genome, other);
    assert_ne!(format!("{genome:?}"), format!("{other:?}"));
    assert_eq!(other.body().capacity(), 51.0);
    assert_eq!(genome.brain_units(), other.brain_units());
}

proptest! {
    /// A genome round-trips through JSON with its body.
    #[test]
    fn a_genome_round_trips_with_its_body(body in body()) {
        let genome = v3alpha1_founder_genome().with_body(body);
        let json = serde_json::to_string(&genome).unwrap();
        prop_assert_eq!(serde_json::from_str::<CreatureGenome>(&json).unwrap(), genome);
    }

    /// The colour edit is the colour step; the kin-tag edit flips exactly
    /// the drawn bit; neither touches another gene.
    #[test]
    fn the_colour_and_kin_tag_edits_change_only_their_gene(
        body in body(),
        switch in proptest::option::of(0usize..5),
        flip in any::<bool>(),
        step in 1u8..=255,
        bit in 0u32..32,
    ) {
        let draw = PhenotypeDraw { switch_to_other: switch, flip_polarity: flip, step };
        let mut coloured = body;
        coloured.apply(BodyEdit::Colour(draw), &RANGES);
        prop_assert_eq!(coloured, BodyGenome { colour: body.colour.mutated(&draw), ..body });
        prop_assert!(coloured.colour().active_channel() < 6);
        let mut flipped = body;
        flipped.apply(BodyEdit::KinTag(bit), &RANGES);
        prop_assert_eq!(flipped.kin_tag() ^ body.kin_tag(), 1 << bit);
        prop_assert_eq!(BodyGenome { kin_tag: body.kin_tag, ..flipped }, body);
    }

    /// A scalar edit is today's `inherit_scalar` arithmetic bit for bit:
    /// `(value × e^δ).clamp(min, max)` over its gene's range.
    #[test]
    fn a_scalar_edit_is_the_clamped_log_step(body in body(), delta in -4.0f32..=4.0) {
        let step = |value: f32, (min, max): (f32, f32)| (value * delta.exp()).clamp(min, max);
        for (edit, read, range) in [
            (BodyEdit::Capacity(delta), BodyGenome::capacity as fn(&BodyGenome) -> f32, RANGES.capacity),
            (BodyEdit::StealShare(delta), BodyGenome::steal_share, RANGES.share),
            (BodyEdit::GiveShare(delta), BodyGenome::give_share, RANGES.share),
        ] {
            let mut edited = body;
            edited.apply(edit, &RANGES);
            prop_assert_eq!(read(&edited).to_bits(), step(read(&body), range).to_bits());
            prop_assert_eq!(edited.colour(), body.colour());
            prop_assert_eq!(edited.kin_tag(), body.kin_tag());
        }
    }

    /// The clamp holds each scalar in its range, is idempotent, and leaves
    /// an in-range body unchanged.
    #[test]
    fn the_clamp_is_idempotent_and_holds_the_ranges(
        body in body(),
        capacity in proptest::num::f32::POSITIVE | proptest::num::f32::ZERO,
        share in proptest::num::f32::POSITIVE | proptest::num::f32::ZERO,
    ) {
        let mut in_range = body;
        in_range.clamp(&RANGES);
        prop_assert_eq!(in_range, body);
        let mut wild = BodyGenome::new(*body.colour(), body.kin_tag(), capacity, share, share);
        wild.clamp(&RANGES);
        prop_assert!((50.0..=800.0).contains(&wild.capacity()));
        prop_assert!((0.01..=1.0).contains(&wild.steal_share()));
        let once = wild;
        wild.clamp(&RANGES);
        prop_assert_eq!(wild, once);
    }

    /// Repeated scalar events at the extreme normalized `share_min` (a
    /// subnormal) and the widest normalized `log_step` stay finite and in
    /// range: a step never multiplies a zero into a `NaN`.
    #[test]
    fn repeated_scalar_events_at_extreme_bounds_stay_finite_and_in_range(
        deltas in proptest::collection::vec(-1.0f32..=1.0, 1..200),
    ) {
        let share_min = 1.0e-40f32;
        let log_step = share_min.ln().abs();
        let ranges = BodyRanges { capacity: (1.0, f32::MAX), share: (share_min, 1.0) };
        let mut body = BodyGenome::new(Phenotype::FOUNDER, 0, 200.0, 0.2, 0.2);
        for delta in deltas {
            let delta = delta * log_step;
            body.apply(BodyEdit::StealShare(delta), &ranges);
            body.apply(BodyEdit::GiveShare(-delta), &ranges);
            body.apply(BodyEdit::Capacity(delta), &ranges);
            for value in [body.steal_share(), body.give_share()] {
                prop_assert!(value.is_finite() && (share_min..=1.0).contains(&value), "{value}");
            }
            prop_assert!(body.capacity().is_finite() && body.capacity() >= 1.0);
        }
    }
}
