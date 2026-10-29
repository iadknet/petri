//! The engine's draw stream under the default `MutationConfig` (T25.F11):
//! the pin the deleted recruitment-path test held, now on the production
//! path itself.
use super::*;
use crate::creature::founder::v3alpha1_founder_genome;
use rand::{rngs::SmallRng, SeedableRng};

/// Over seeds `0..256`, the founder mutated once under the default
/// `MutationConfig` and the sensor context of two food types: the FNV-1a
/// hash of the mutated genome's JSON, the event records (operator, target,
/// outcome) and the RNG's next draw after the birth. Any change to an
/// operator's draws, the catalog weights, the event order or the genome
/// text moves it.
#[test]
fn the_default_draw_stream_is_pinned() {
    let config = MutationConfig::default();
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut applied = 0;
    for seed in 0..256 {
        let mut genome = v3alpha1_founder_genome();
        let mut rng = SmallRng::seed_from_u64(seed);
        let summary = MutationEngine::apply_mutations_in(
            &mut genome,
            &config,
            &[0, 1, 2],
            ParentExecuted::NONE,
            &mut rng,
            &crate::config::test_sensor_context(2),
        );
        applied += summary.applied_events;
        let bytes = format!(
            "{}{:?}{}",
            serde_json::to_string(&genome).unwrap(),
            summary.events,
            rng.gen::<u64>()
        );
        for byte in bytes.bytes() {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
        }
    }
    assert!(
        applied > 0,
        "the default rate applies events on the founder"
    );
    // Pinned by T25.F11 commit 3 after two agreeing runs; re-pinned by
    // T25.F12 commit 2 (the learning catalog, the target draw over edges and
    // slots, the bundle birth draws and the genome JSON key `learning`).
    assert_eq!(hash, 13_590_202_118_071_790_475, "the default draw stream");
}
