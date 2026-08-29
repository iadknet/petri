/// A flat row-major 2D grid of values.
/// Width and height are fixed at construction time.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Grid<T> {
    width: u16,
    height: u16,
    cells: Vec<T>,
}

impl<T: Clone> Grid<T> {
    /// Create a new grid filled with `fill`. Panics if width * height overflows usize.
    pub fn new(width: u16, height: u16, fill: T) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            cells: vec![fill; size],
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    #[inline]
    fn idx(&self, x: u16, y: u16) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    pub fn get(&self, x: u16, y: u16) -> &T {
        &self.cells[self.idx(x, y)]
    }

    pub fn get_mut(&mut self, x: u16, y: u16) -> &mut T {
        let idx = self.idx(x, y);
        &mut self.cells[idx]
    }

    pub fn set(&mut self, x: u16, y: u16, value: T) {
        let idx = self.idx(x, y);
        self.cells[idx] = value;
    }

    /// Borrow the underlying flat storage as a slice.
    pub fn as_slice(&self) -> &[T] {
        &self.cells
    }

    /// Iterate over all cells as (x, y, value) tuples.
    pub fn iter(&self) -> impl Iterator<Item = (u16, u16, &T)> {
        self.cells.iter().enumerate().map(move |(i, v)| {
            let x = (i % self.width as usize) as u16;
            let y = (i / self.width as usize) as u16;
            (x, y, v)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_set_roundtrip() {
        let mut g: Grid<u8> = Grid::new(4, 3, 0);
        g.set(2, 1, 42);
        assert_eq!(*g.get(2, 1), 42);
        assert_eq!(*g.get(0, 0), 0);
    }

    #[test]
    fn dimensions_correct() {
        let g: Grid<bool> = Grid::new(10, 20, false);
        assert_eq!(g.width(), 10);
        assert_eq!(g.height(), 20);
    }

    #[test]
    fn iter_visits_all_cells() {
        let g: Grid<u8> = Grid::new(5, 3, 0);
        assert_eq!(g.iter().count(), 15);
    }

    #[test]
    fn iter_positions_in_bounds() {
        let w = 3u16;
        let h = 2u16;
        let g: Grid<u8> = Grid::new(w, h, 0);
        for (x, y, _) in g.iter() {
            assert!(x < w && y < h);
        }
    }

    #[test]
    fn fill_initializes_all_cells() {
        let g: Grid<i32> = Grid::new(3, 3, 99);
        for (_, _, v) in g.iter() {
            assert_eq!(*v, 99);
        }
    }

    #[test]
    fn get_mut_modifies_cell() {
        let mut g: Grid<u32> = Grid::new(2, 2, 0);
        *g.get_mut(1, 1) = 77;
        assert_eq!(*g.get(1, 1), 77);
    }
}
