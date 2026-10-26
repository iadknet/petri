/// Eight-directional compass rose, canonical per v3-world-grid-spec Section 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Direction {
    /// All 8 directions in canonical spec order: N, NE, E, SE, S, SW, W, NW.
    pub const ALL: [Direction; 8] = [
        Direction::N,
        Direction::NE,
        Direction::E,
        Direction::SE,
        Direction::S,
        Direction::SW,
        Direction::W,
        Direction::NW,
    ];

    /// Returns the (dx, dy) delta for this direction.
    /// x increases rightward, y increases downward (screen coordinates).
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::N => (0, -1),
            Direction::NE => (1, -1),
            Direction::E => (1, 0),
            Direction::SE => (1, 1),
            Direction::S => (0, 1),
            Direction::SW => (-1, 1),
            Direction::W => (-1, 0),
            Direction::NW => (-1, -1),
        }
    }

    /// Canonical direction index (0=N, 1=NE, 2=E, 3=SE, 4=S, 5=SW, 6=W, 7=NW).
    /// Matches the position in `Direction::ALL`.
    pub fn to_index(self) -> usize {
        match self {
            Direction::N => 0,
            Direction::NE => 1,
            Direction::E => 2,
            Direction::SE => 3,
            Direction::S => 4,
            Direction::SW => 5,
            Direction::W => 6,
            Direction::NW => 7,
        }
    }
}

/// The unit compass's `x` component at direction index `d` (mod 8):
/// `Direction::ALL[d].delta().0` normalized, exactly `0.0`, `±1.0` or
/// `±FRAC_1_SQRT_2` (T25.F04).
#[must_use]
pub fn compass_x(d: u8) -> f32 {
    let (dx, dy) = Direction::ALL[usize::from(d % 8)].delta();
    let scale = if dx != 0 && dy != 0 {
        std::f32::consts::FRAC_1_SQRT_2
    } else {
        1.0
    };
    dx as f32 * scale
}

/// The unit compass's `y` component at direction index `d`:
/// `compass_x((d + 6) mod 8)`, the normalized `dy` (T25.F04).
#[must_use]
pub fn compass_y(d: u8) -> f32 {
    compass_x((d % 8 + 6) % 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn py_equals_normalized_delta_dy() {
        for (d, direction) in (0u8..).zip(Direction::ALL) {
            let (dx, dy) = direction.delta();
            let norm = ((dx * dx + dy * dy) as f32).sqrt();
            assert!((compass_x(d) - dx as f32 / norm).abs() <= f32::EPSILON);
            assert!((compass_y(d) - dy as f32 / norm).abs() <= f32::EPSILON);
            let len = compass_x(d).hypot(compass_y(d));
            assert!((len - 1.0).abs() <= f32::EPSILON, "row {d} norm {len}");
            for value in [compass_x(d), compass_y(d)] {
                assert!([
                    0.0,
                    1.0,
                    -1.0,
                    std::f32::consts::FRAC_1_SQRT_2,
                    -std::f32::consts::FRAC_1_SQRT_2
                ]
                .contains(&value));
            }
        }
    }

    #[test]
    fn compass_components_reduce_every_index_mod_8() {
        for d in 0..=u8::MAX {
            assert_eq!(compass_x(d), compass_x(d % 8), "x at {d}");
            assert_eq!(compass_y(d), compass_y(d % 8), "y at {d}");
        }
    }

    #[test]
    fn direction_all_has_eight_entries() {
        assert_eq!(Direction::ALL.len(), 8);
    }

    #[test]
    fn direction_deltas_canonical() {
        assert_eq!(Direction::N.delta(), (0, -1));
        assert_eq!(Direction::NE.delta(), (1, -1));
        assert_eq!(Direction::E.delta(), (1, 0));
        assert_eq!(Direction::SE.delta(), (1, 1));
        assert_eq!(Direction::S.delta(), (0, 1));
        assert_eq!(Direction::SW.delta(), (-1, 1));
        assert_eq!(Direction::W.delta(), (-1, 0));
        assert_eq!(Direction::NW.delta(), (-1, -1));
    }

    #[test]
    fn direction_order_matches_spec() {
        let expected = [
            Direction::N,
            Direction::NE,
            Direction::E,
            Direction::SE,
            Direction::S,
            Direction::SW,
            Direction::W,
            Direction::NW,
        ];
        assert_eq!(Direction::ALL, expected);
    }

    #[test]
    fn direction_serde_roundtrip() {
        let d = Direction::SE;
        let json = serde_json::to_string(&d).unwrap();
        let d2: Direction = serde_json::from_str(&json).unwrap();
        assert_eq!(d, d2);
    }

    #[test]
    fn to_index_matches_all_order() {
        for (i, dir) in Direction::ALL.iter().enumerate() {
            assert_eq!(dir.to_index(), i);
        }
    }
}
