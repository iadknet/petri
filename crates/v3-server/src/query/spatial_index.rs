//! Query-side spatial index.

use std::collections::HashMap;

use crate::state::CreatureSnapshot;

pub const DEFAULT_TILE_SIZE: u16 = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl ViewRect {
    #[must_use]
    pub fn contains(self, x: u16, y: u16) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x.saturating_add(self.width)
            && y < self.y.saturating_add(self.height)
    }
}

#[derive(Clone, Debug, Default)]
pub struct CreatureTileIndex {
    tile_size: u16,
    buckets: HashMap<(u16, u16), Vec<usize>>,
}

impl CreatureTileIndex {
    #[must_use]
    pub fn build(tile_size: u16, creatures: &[CreatureSnapshot]) -> Self {
        let mut buckets: HashMap<(u16, u16), Vec<usize>> = HashMap::new();
        for (index, creature) in creatures.iter().enumerate() {
            let key = (creature.x / tile_size, creature.y / tile_size);
            buckets.entry(key).or_default().push(index);
        }

        Self { tile_size, buckets }
    }

    #[must_use]
    pub fn query_indices(&self, rect: ViewRect) -> Vec<usize> {
        if rect.width == 0 || rect.height == 0 {
            return Vec::new();
        }

        let max_x = rect.x.saturating_add(rect.width).saturating_sub(1);
        let max_y = rect.y.saturating_add(rect.height).saturating_sub(1);
        let min_tile_x = rect.x / self.tile_size;
        let max_tile_x = max_x / self.tile_size;
        let min_tile_y = rect.y / self.tile_size;
        let max_tile_y = max_y / self.tile_size;
        let tile_count =
            (max_tile_x - min_tile_x + 1) as usize * (max_tile_y - min_tile_y + 1) as usize;

        let mut indices = Vec::with_capacity(tile_count * 4);
        for tile_y in min_tile_y..=max_tile_y {
            for tile_x in min_tile_x..=max_tile_x {
                if let Some(bucket) = self.buckets.get(&(tile_x, tile_y)) {
                    indices.extend(bucket.iter().copied());
                }
            }
        }
        indices.sort_unstable();
        indices
    }

    #[must_use]
    pub fn query_ids(&self, rect: ViewRect, creatures: &[CreatureSnapshot]) -> Vec<u64> {
        self.query_indices(rect)
            .into_iter()
            .filter_map(|index| creatures.get(index))
            .filter(|creature| rect.contains(creature.x, creature.y))
            .map(|creature| creature.id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::query::spatial_index::{CreatureTileIndex, ViewRect, DEFAULT_TILE_SIZE};
    use crate::state::CreatureSnapshot;

    #[test]
    fn creature_tile_index_queries_only_intersecting_creatures() {
        let creatures = vec![
            CreatureSnapshot {
                id: 1,
                x: 2,
                y: 2,
                energy: 1.0,
                generation: 0,
                phenotype_rgb: [1, 2, 3],
            },
            CreatureSnapshot {
                id: 2,
                x: DEFAULT_TILE_SIZE + 1,
                y: DEFAULT_TILE_SIZE + 1,
                energy: 2.0,
                generation: 0,
                phenotype_rgb: [4, 5, 6],
            },
            CreatureSnapshot {
                id: 3,
                x: 90,
                y: 90,
                energy: 3.0,
                generation: 0,
                phenotype_rgb: [7, 8, 9],
            },
        ];
        let index = CreatureTileIndex::build(DEFAULT_TILE_SIZE, &creatures);

        let ids = index.query_ids(
            ViewRect {
                x: 0,
                y: 0,
                width: DEFAULT_TILE_SIZE * 2,
                height: DEFAULT_TILE_SIZE * 2,
            },
            &creatures,
        );

        assert_eq!(ids, vec![1, 2]);
    }
}
