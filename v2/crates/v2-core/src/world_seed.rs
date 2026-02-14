#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldSeedConfig {
    pub initial_food_density: f32,
    pub food_growth_rate: f32,
    pub food_spawn_rate: f32,
    pub food_spread_threshold: f32,
    pub food_spawn_floor_density: f32,
}

impl Default for WorldSeedConfig {
    fn default() -> Self {
        Self {
            initial_food_density: 0.15,
            food_growth_rate: 0.10,
            food_spawn_rate: 0.05,
            food_spread_threshold: 0.75,
            food_spawn_floor_density: 0.03,
        }
    }
}

#[must_use]
pub fn seed_food_cells(width: u16, height: u16, density: f32, seed: u64) -> Vec<(u16, u16)> {
    let total_cells = u32::from(width) * u32::from(height);
    if total_cells == 0 {
        return Vec::new();
    }

    let clamped_density = density.clamp(0.0, 1.0);
    let target = ((clamped_density * total_cells as f32).round() as u32).min(total_cells);
    sample_unique_cells(width, height, target as usize, seed ^ 0xF00D_CAFE_DEAD_BEEF)
}

#[must_use]
pub fn seed_creature_cells(width: u16, height: u16, count: usize, seed: u64) -> Vec<(u16, u16)> {
    sample_unique_cells(width, height, count, seed ^ 0xC001_CAFE_5EED_BAAD)
}

#[must_use]
fn sample_unique_cells(width: u16, height: u16, count: usize, seed: u64) -> Vec<(u16, u16)> {
    let total_cells = u32::from(width) * u32::from(height);
    if total_cells == 0 || count == 0 {
        return Vec::new();
    }

    let capped = count.min(total_cells as usize);
    let start = (mix(seed) % u64::from(total_cells)) as u32;

    let mut step = ((mix(seed ^ 0x9E37_79B9_7F4A_7C15) % u64::from(total_cells)) as u32).max(1);
    while gcd_u32(step, total_cells) != 1 {
        step = step.wrapping_add(1);
        if step >= total_cells {
            step = 1;
        }
    }

    let mut cells = Vec::with_capacity(capped);
    let mut index = start;
    for _ in 0..capped {
        let x = (index % u32::from(width)) as u16;
        let y = (index / u32::from(width)) as u16;
        cells.push((x, y));
        index = (index + step) % total_cells;
    }

    cells
}

#[must_use]
fn mix(mut value: u64) -> u64 {
    value ^= value >> 33;
    value = value.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    value ^= value >> 33;
    value = value.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    value ^ (value >> 33)
}

#[must_use]
fn gcd_u32(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a.max(1)
}
