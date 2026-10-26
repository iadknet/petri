//! Deterministic hashing shared by every domain.

/// The SplitMix64 finalizer: a deterministic 64-bit mix used for seed
/// derivation (founder kin tags), the food stream's per-cell hash (T24.F02)
/// and telemetry window selection.
#[must_use]
pub fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix64_is_deterministic() {
        let a = splitmix64(42);
        let b = splitmix64(42);
        assert_eq!(a, b);
        // Different inputs produce different outputs
        assert_ne!(splitmix64(42), splitmix64(43));
    }
}
