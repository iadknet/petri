//! Seed derivation. Every stream in the lab is `hash(parent, tags…)`: a
//! SHA-256 over a length-prefixed encoding of the parts, first eight bytes
//! little-endian. Fixed by construction, so seeds never depend on thread
//! count or evaluation order.

use rand::rngs::SmallRng;
use rand::SeedableRng;
use sha2::{Digest, Sha256};

/// One hashed part: a number or a text tag.
#[derive(Debug, Clone, Copy)]
pub enum Part<'a> {
    U(u64),
    S(&'a str),
}

impl From<u64> for Part<'_> {
    fn from(value: u64) -> Self {
        Self::U(value)
    }
}

impl<'a> From<&'a str> for Part<'a> {
    fn from(value: &'a str) -> Self {
        Self::S(value)
    }
}

/// `hash(parts…)`: deterministic across processes and platforms.
#[must_use]
pub fn hash(parts: &[Part<'_>]) -> u64 {
    let mut hasher = Sha256::new();
    for part in parts {
        match part {
            Part::U(value) => {
                hasher.update([0u8]);
                hasher.update(value.to_le_bytes());
            }
            Part::S(text) => {
                hasher.update([1u8]);
                hasher.update((text.len() as u64).to_le_bytes());
                hasher.update(text.as_bytes());
            }
        }
    }
    let digest = hasher.finalize();
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    u64::from_le_bytes(bytes)
}

/// A `SmallRng` seeded from `hash(parts…)`.
#[must_use]
pub fn stream(parts: &[Part<'_>]) -> SmallRng {
    SmallRng::seed_from_u64(hash(parts))
}

/// Replicate seed `r_i = hash(seed, i)`.
#[must_use]
pub fn replicate_seed(seed: u64, replicate: u32) -> u64 {
    hash(&[Part::U(seed), Part::U(u64::from(replicate))])
}

/// Named per-replicate or per-run stream seed `hash(parent, tag)`.
#[must_use]
pub fn tagged(parent: u64, tag: &str) -> u64 {
    hash(&[Part::U(parent), Part::S(tag)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_and_text_parts_do_not_collide() {
        assert_ne!(hash(&["1".into()]), hash(&[1u64.into()]));
        assert_ne!(
            hash(&["ab".into(), "c".into()]),
            hash(&["a".into(), "bc".into()])
        );
    }

    #[test]
    fn replicate_seeds_differ_and_are_stable() {
        assert_eq!(replicate_seed(1, 0), replicate_seed(1, 0));
        assert_ne!(replicate_seed(1, 0), replicate_seed(1, 1));
        assert_ne!(tagged(7, "scenes"), tagged(7, "selection"));
    }
}
