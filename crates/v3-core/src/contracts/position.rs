/// A 2D grid position.
/// Origin (0,0) is top-left. x increases rightward, y increases downward.
/// Valid range: x in [0, width-1], y in [0, height-1].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    /// Compute neighbor using toroidal wrapping.
    /// Returns None only if width or height is 0.
    pub fn neighbor_wrap(self, dx: i32, dy: i32, width: u16, height: u16) -> Option<Position> {
        if width == 0 || height == 0 {
            return None;
        }
        let nx = (self.x as i32 + dx).rem_euclid(width as i32) as u16;
        let ny = (self.y as i32 + dy).rem_euclid(height as i32) as u16;
        Some(Position::new(nx, ny))
    }

    /// Compute neighbor in bounded mode.
    /// Returns None if the neighbor falls outside [0, width-1] x [0, height-1].
    pub fn neighbor_bounded(self, dx: i32, dy: i32, width: u16, height: u16) -> Option<Position> {
        let nx = self.x as i32 + dx;
        let ny = self.y as i32 + dy;
        if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
            return None;
        }
        Some(Position::new(nx as u16, ny as u16))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::Direction;

    #[test]
    fn neighbor_wrap_north_from_top_edge() {
        let pos = Position::new(5, 0);
        let (dx, dy) = Direction::N.delta();
        let neighbor = pos.neighbor_wrap(dx, dy, 10, 10).unwrap();
        assert_eq!(neighbor, Position::new(5, 9));
    }

    #[test]
    fn neighbor_wrap_east_from_right_edge() {
        let pos = Position::new(9, 5);
        let (dx, dy) = Direction::E.delta();
        let neighbor = pos.neighbor_wrap(dx, dy, 10, 10).unwrap();
        assert_eq!(neighbor, Position::new(0, 5));
    }

    #[test]
    fn neighbor_bounded_off_edge_returns_none() {
        let pos = Position::new(0, 0);
        let (dx, dy) = Direction::N.delta();
        assert!(pos.neighbor_bounded(dx, dy, 10, 10).is_none());
        let (dx, dy) = Direction::W.delta();
        assert!(pos.neighbor_bounded(dx, dy, 10, 10).is_none());
    }

    #[test]
    fn neighbor_bounded_valid_returns_position() {
        let pos = Position::new(3, 3);
        let (dx, dy) = Direction::SE.delta();
        let neighbor = pos.neighbor_bounded(dx, dy, 10, 10).unwrap();
        assert_eq!(neighbor, Position::new(4, 4));
    }

    #[test]
    fn neighbor_wrap_all_directions_stay_in_bounds() {
        let pos = Position::new(5, 5);
        for dir in Direction::ALL {
            let (dx, dy) = dir.delta();
            let n = pos.neighbor_wrap(dx, dy, 10, 10).unwrap();
            assert!(n.x < 10 && n.y < 10);
        }
    }

    #[test]
    fn neighbor_wrap_corner_nw() {
        let pos = Position::new(0, 0);
        let (dx, dy) = Direction::NW.delta();
        let n = pos.neighbor_wrap(dx, dy, 10, 10).unwrap();
        assert_eq!(n, Position::new(9, 9));
    }
}
