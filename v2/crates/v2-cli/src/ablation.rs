use v2_core::ecology::EcologyConfig;

#[must_use]
pub fn deterministic_score(seed: u64, ticks: u64, preset: &str) -> f32 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in preset.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash ^= seed;
    hash = hash.wrapping_mul(0x1000_0000_01b3);
    hash ^= ticks;
    let scaled = (hash % 10_000) as f32;
    scaled / 100.0
}

#[must_use]
pub fn best_preset(results: &[(String, f32)]) -> String {
    results
        .iter()
        .max_by(|(name_a, score_a), (name_b, score_b)| {
            score_a
                .partial_cmp(score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| name_b.cmp(name_a))
        })
        .map(|(name, _)| name.clone())
        .unwrap_or_else(|| "default".to_string())
}

#[must_use]
pub fn config_for_preset(preset: &str) -> EcologyConfig {
    let mut config = EcologyConfig::default();
    let hash = preset_hash(preset) as f32;

    // Keep preset perturbations bounded and deterministic.
    let spawn_offset = ((hash % 11.0) - 5.0) * 0.0008;
    config.base_food_spawn_rate = (config.base_food_spawn_rate + spawn_offset).clamp(0.003, 0.03);

    let crowding_offset = ((hash % 7.0) - 3.0) * 0.001;
    config.crowding_penalty_per_neighbor =
        (config.crowding_penalty_per_neighbor + crowding_offset).clamp(0.001, 0.02);

    config
}

fn preset_hash(preset: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in preset.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}
