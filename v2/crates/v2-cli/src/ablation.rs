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
