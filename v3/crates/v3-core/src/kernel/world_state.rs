use super::types::{CreatureId, Direction, Position};

pub struct WorldState {
    pub width: u16,
    pub height: u16,
    pub wrap: bool,

    // Flat arrays indexed by (y * width + x) for cache-friendly O(1) access.
    // A 400x400 world is only 160K entries — flat Vec beats HashMap here.
    /// Food density per cell (0-255 quantized).
    food: Vec<u8>,

    /// Barrier locations.
    barriers: Vec<bool>,

    /// Spatial index: which creature (if any) occupies each cell.
    /// Provides O(1) position->creature lookup and occupancy checking.
    creature_at: Vec<Option<CreatureId>>,
}

impl WorldState {
    pub fn new(width: u16, height: u16, wrap: bool) -> Self {
        let len = (width as usize) * (height as usize);
        Self {
            width,
            height,
            wrap,
            food: vec![0u8; len],
            barriers: vec![false; len],
            creature_at: vec![None; len],
        }
    }

    #[inline]
    fn cell_index(&self, pos: Position) -> usize {
        (pos.y as usize) * (self.width as usize) + (pos.x as usize)
    }

    pub fn get_food_density(&self, pos: Position) -> u8 {
        self.food[self.cell_index(pos)]
    }

    pub fn set_food_density(&mut self, pos: Position, density: u8) {
        let idx = self.cell_index(pos);
        self.food[idx] = density;
    }

    pub fn consume_food(&mut self, pos: Position) -> u8 {
        let idx = self.cell_index(pos);
        let amount = self.food[idx];
        self.food[idx] = 0;
        amount
    }

    pub fn is_occupied(&self, pos: Position) -> bool {
        self.creature_at[self.cell_index(pos)].is_some()
    }

    pub fn creature_at(&self, pos: Position) -> Option<CreatureId> {
        self.creature_at[self.cell_index(pos)]
    }

    pub fn is_barrier(&self, pos: Position) -> bool {
        self.barriers[self.cell_index(pos)]
    }

    pub fn place_creature(&mut self, pos: Position, id: CreatureId) {
        let idx = self.cell_index(pos);
        debug_assert!(self.creature_at[idx].is_none(), "cell already occupied");
        self.creature_at[idx] = Some(id);
    }

    pub fn remove_creature(&mut self, pos: Position) {
        let idx = self.cell_index(pos);
        debug_assert!(self.creature_at[idx].is_some(), "cell not occupied");
        self.creature_at[idx] = None;
    }

    /// Resolve a neighbor position, respecting wrap/clamp. Returns None if
    /// the target is out-of-bounds in a non-wrapping world.
    pub fn resolve_neighbor(&self, pos: Position, dir: Direction) -> Option<Position> {
        let (dx, dy) = dir.delta();
        let nx = pos.x as i32 + dx;
        let ny = pos.y as i32 + dy;

        if self.wrap {
            Some(Position {
                x: ((nx % self.width as i32 + self.width as i32) % self.width as i32) as u16,
                y: ((ny % self.height as i32 + self.height as i32) % self.height as i32) as u16,
            })
        } else if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
            Some(Position {
                x: nx as u16,
                y: ny as u16,
            })
        } else {
            None
        }
    }
}
