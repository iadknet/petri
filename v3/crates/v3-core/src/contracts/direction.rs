/// Eight-directional compass rose, canonical per v3-world-grid-spec Section 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

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
