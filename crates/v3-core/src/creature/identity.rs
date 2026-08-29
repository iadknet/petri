/// Per-creature identity state for kin recognition and lineage tracking.
///
/// See `docs/reference/v3-creature-identity-spec.md` for canonical rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CreatureIdentityState {
    /// Stable founder/clade identity. Inherited unchanged from parent.
    pub lineage_id: u32,
    /// Inherited family signature that drifts slowly via single-bit flips.
    pub kin_tag: u32,
}

impl CreatureIdentityState {
    /// Initialize founder identity per v3-creature-identity-spec.md Section 3.
    ///
    /// - `lineage_id = founder_index as u32`
    /// - `kin_tag = splitmix64(startup_seed ^ lineage_id as u64) as u32`
    #[must_use]
    pub fn founder(founder_index: usize, startup_seed: u64) -> Self {
        let lineage_id = founder_index as u32;
        let mixed = splitmix64(startup_seed ^ lineage_id as u64);
        Self {
            lineage_id,
            kin_tag: mixed as u32,
        }
    }

    /// Inherit identity from parent to child per v3-creature-identity-spec.md Section 4.
    ///
    /// - `lineage_id` is inherited unchanged.
    /// - `kin_tag` is inherited unchanged (caller must apply `mutate_kin_tag` separately
    ///   when `MutationSummary.applied_events > 0`).
    #[must_use]
    pub fn inherit(parent: &Self) -> Self {
        *parent
    }

    /// Mutate kin_tag by flipping exactly one uniformly random bit.
    ///
    /// Per v3-creature-identity-spec.md Section 5:
    /// - flip exactly one uniformly random bit in the 32-bit kin_tag
    /// - called only when `MutationSummary.applied_events > 0`
    pub fn mutate_kin_tag(&mut self, rng: &mut impl rand::Rng) {
        let bit = rng.gen_range(0..32u32);
        self.kin_tag ^= 1 << bit;
    }
}

/// splitmix64 finalizer — deterministic hash for seed derivation.
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    #[test]
    fn founder_identity_deterministic_for_same_seed() {
        let id1 = CreatureIdentityState::founder(0, 42);
        let id2 = CreatureIdentityState::founder(0, 42);
        assert_eq!(id1, id2);
    }

    #[test]
    fn founder_identity_unique_per_index() {
        let id0 = CreatureIdentityState::founder(0, 42);
        let id1 = CreatureIdentityState::founder(1, 42);
        let id2 = CreatureIdentityState::founder(2, 42);
        assert_eq!(id0.lineage_id, 0);
        assert_eq!(id1.lineage_id, 1);
        assert_eq!(id2.lineage_id, 2);
        // kin_tags should differ
        assert_ne!(id0.kin_tag, id1.kin_tag);
        assert_ne!(id1.kin_tag, id2.kin_tag);
    }

    #[test]
    fn founder_kin_tag_differs_by_seed() {
        let id_a = CreatureIdentityState::founder(0, 42);
        let id_b = CreatureIdentityState::founder(0, 99);
        assert_ne!(id_a.kin_tag, id_b.kin_tag);
    }

    #[test]
    fn child_inherits_lineage_unchanged() {
        let parent = CreatureIdentityState::founder(5, 100);
        let child = CreatureIdentityState::inherit(&parent);
        assert_eq!(child.lineage_id, parent.lineage_id);
        assert_eq!(child.kin_tag, parent.kin_tag);
    }

    #[test]
    fn child_keeps_kin_tag_when_no_mutation() {
        let parent = CreatureIdentityState::founder(3, 77);
        let child = CreatureIdentityState::inherit(&parent);
        // Without calling mutate_kin_tag, child keeps parent's kin_tag
        assert_eq!(child.kin_tag, parent.kin_tag);
    }

    #[test]
    fn mutate_kin_tag_flips_exactly_one_bit() {
        let mut rng = SmallRng::seed_from_u64(42);
        let original = CreatureIdentityState::founder(0, 42);
        let mut mutated = original;
        mutated.mutate_kin_tag(&mut rng);
        // Exactly one bit should differ
        let diff = original.kin_tag ^ mutated.kin_tag;
        assert_eq!(diff.count_ones(), 1);
    }

    #[test]
    fn mutate_kin_tag_preserves_lineage_id() {
        let mut rng = SmallRng::seed_from_u64(42);
        let mut identity = CreatureIdentityState::founder(7, 42);
        let original_lineage = identity.lineage_id;
        identity.mutate_kin_tag(&mut rng);
        assert_eq!(identity.lineage_id, original_lineage);
    }

    #[test]
    fn splitmix64_is_deterministic() {
        let a = splitmix64(42);
        let b = splitmix64(42);
        assert_eq!(a, b);
        // Different inputs produce different outputs
        assert_ne!(splitmix64(42), splitmix64(43));
    }
}
