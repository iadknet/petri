//! The completed T20.F09 assay is reproducible at its measured source revision.
//! Its unqualified production operators are absent from current builds.

pub const WALL_CAP_SECS: u64 = 7200;
pub const BYTE_CAP: u64 = 512 * 1024 * 1024;
pub const RETIRED_REASON: &str = "T20.F09 input-discovery is retired: no candidate qualified. Historical assay source: cb254ca9f89d158a1da6a9a3e62b3ac4994dba67. Current builds cannot run the retired arms.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retired_assay_caps_remain_fixed() {
        assert_eq!(WALL_CAP_SECS, 7_200);
        assert_eq!(BYTE_CAP, 536_870_912);
    }
}
