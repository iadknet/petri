# Complex Barrier Painting Tools — Algorithm Specifications

**Parent plan:** `master_plan.md`

**Goal:** Detailed specifications for each of the 5 pattern generation algorithms.

**Goal IDs:** GP-02, GP-04

**Scope:** Algorithm specifications only — all algorithms live within `v3-core::patterns`.

**Docs Impact:** None beyond parent plan.

**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-02`: All algorithms are pure functions (`(bounds, params, rng) -> Vec<PaintPoint>`)
  with no side effects and no dependencies beyond `kernel` types. Clean boundary:
  generation logic is fully separated from world state mutation.

## Boundary Impact

All algorithms live within the `v3-core::patterns` module. No external boundary
impact — algorithms depend only on `kernel::paint::PaintPoint` for output type
and standard library / `rand` crate for RNG.

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `v3-core::patterns` | new module | Pure algorithm functions, no world state coupling |
| `v3-core::kernel` | not touched | Algorithms use kernel types but don't modify kernel |

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| (none remaining) | — | — | — |

## Common Contract

All pattern algorithms share:

```rust
fn generate_PATTERN(
    bounds: PatternBounds,   // by value — Copy type, 8 bytes
    params: &PatternParams,  // matched to the specific variant
    rng: &mut impl Rng,
) -> Vec<PaintPoint>
```

**Guarantees:**
- All returned positions are within `bounds` (x..x+width, y..y+height)
- No duplicate positions in output
- Empty bounds (width=0 or height=0) returns empty vec
- Deterministic for same RNG state

## 1. Maze

**Algorithm:** Recursive backtracker (depth-first) on a grid of cells.

**Steps:**
1. Divide the bounding rectangle into a grid of cells. Each cell is
   `corridor_width` wide/tall, separated by walls of `wall_thickness`.
2. Compute grid dimensions: `cols = (width - wall_thickness) / (corridor_width + wall_thickness)`,
   similarly for rows. Minimum 2x2 grid.
3. Initialize all walls as present (barrier cells).
4. Run recursive backtracker from a random starting cell:
   - Mark current cell as visited
   - Shuffle unvisited neighbors
   - For each unvisited neighbor: remove wall between current and neighbor, recurse
5. If `open_center_radius > 0`: clear all barrier cells within that radius of the
   bounds center.
6. Ensure at least one opening on each of the four edges of the bounding rectangle
   (clear a corridor-width gap on each edge). Openings are always placed regardless
   of surrounding context — the maze doesn't know about world state outside its
   bounds, and openings allow creatures to enter/exit from any direction.
7. Collect all wall positions as barrier cells. Return them.

**Test invariants:**
- Output contains no cells in corridor spaces (corridors are connected)
- Every non-wall cell is reachable from every other non-wall cell (connectivity)
- Wall cells are within bounds

**Parameter defaults:** `corridor_width: 2, wall_thickness: 1, open_center_radius: 0`

**Parameter constraints:** `corridor_width >= 1, wall_thickness >= 1`

## 2. Spiral

**Algorithm:** Archimedean spiral with configurable arms.

**Steps:**
1. Compute center of bounds: `(cx, cy) = (x + width/2, y + height/2)`.
2. Compute max radius: `max_r = min(width, height) / 2`.
3. For each arm `i` in `0..arm_count`:
   - Starting angle offset: `theta_0 = i * (2*PI / arm_count)`
   - Direction multiplier: `dir = 1.0` if clockwise, `-1.0` if CCW
   - Trace the spiral: for `t` from 0.0 stepping adaptively
     (step = `min(0.5, 0.5 / max(1.0, r))` to avoid gaps near center):
     - `theta = theta_0 + dir * t * (2*PI / (gap_width + arm_thickness))`
     - `r = t`
     - If `r > max_r`: stop
     - If `r < open_center_radius`: skip (don't place barriers near center)
     - `(px, py) = (cx + r*cos(theta), cy + r*sin(theta))`
     - For thickness: place barrier cells in a disk of radius `arm_thickness/2`
       centered at `(px, py)`, clipped to bounds
4. Deduplicate positions. Return them.

**Test invariants:**
- Number of distinct angular sectors with barriers >= arm_count
- No barriers within open_center_radius of center (if > 0)
- All positions within bounds

**Parameter defaults:** `arm_count: 3, arm_thickness: 2, gap_width: 4, clockwise: true, open_center_radius: 0`

**Parameter constraints:** `arm_count >= 1, arm_thickness >= 1, gap_width >= 1`

## 3. Random Noise

**Algorithm:** Clustered random noise using seed points with radial falloff.

**Steps:**
1. Compute total cells in bounds: `total = width * height`.
2. Target barrier count: `target = (total as f32 * density).round()`.
3. If `cluster_size <= 1` (point noise):
   - Randomly select `target` unique positions within bounds.
4. If `cluster_size > 1` (clustered noise):
   - Compute seed count: `seeds = target / (cluster_size^2 * PI/4)` (approximate
     area of each cluster circle).
   - For each seed: pick random position within bounds.
   - For each seed: place barrier cells in a disk of radius `cluster_size`,
     each with probability proportional to `1 - (dist/cluster_size)^2`
     (quadratic falloff). Use rng for stochastic placement.
   - If total barriers < target: add random individual cells to fill.
   - If total barriers > target: randomly remove excess.
5. Deduplicate and clip to bounds. Return positions.

**Test invariants:**
- Actual barrier count is within 10% of `density * area` (stochastic tolerance)
- All positions within bounds
- For cluster_size > 1: barriers are not uniformly distributed (clustering test:
  variance of local density > uniform variance)

**Parameter defaults:** `density: 0.15, cluster_size: 3`

**Parameter constraints:** `density in 0.0..=1.0, cluster_size >= 1`

## 4. Parallel Lines

**Algorithm:** Generate parallel lines with jagged/noisy displacement.

**Steps:**
1. Convert `angle_degrees` to radians: `angle = angle_degrees * PI / 180`.
2. Compute line direction vector: `(dx, dy) = (cos(angle), sin(angle))`.
3. Compute perpendicular vector: `(nx, ny) = (-sin(angle), cos(angle))`.
4. Compute line count from spacing: `count = max_extent_along_normal / spacing`.
5. For each line `i` in `0..count`:
   - Line base offset along normal: `offset = i * spacing + spacing/2`.
   - For each pixel position along the line direction (stepping by 1):
     - Compute base position: `(bx, by) = bounds_origin + offset*(nx,ny) + t*(dx,dy)`
     - Apply jaggedness: displacement along normal = `jaggedness * noise(t)` where
       `noise` is a simple 1D noise function (e.g., smoothed random walk with
       `rng`). Amplitude = `jaggedness * spacing * 0.3`.
     - For thickness: place barrier cells in a perpendicular band of `thickness`
       pixels centered at the displaced position.
     - Clip to bounds.
6. Deduplicate positions. Return them.

**Noise function for jaggedness:** Simple random walk: maintain a displacement
value that changes by `rng.gen_range(-step..=step)` each pixel, clamped to
`[-amplitude, amplitude]`. Step size = `amplitude * 0.15`.

**Test invariants:**
- Lines are approximately parallel (average perpendicular distance between
  adjacent lines is approximately `spacing`)
- Line count matches expected count for bounds size and spacing
- All positions within bounds

**Parameter defaults:** `spacing: 8, thickness: 1, jaggedness: 0.5, angle_degrees: 0.0`

**Parameter constraints:** `spacing >= 2, thickness >= 1, jaggedness in 0.0..=1.0, angle_degrees in 0.0..360.0`

## 5. Star

**Algorithm:** Radiating line segments from center points.

**Steps:**
1. Distribute `point_count` center points within bounds:
   - If `point_count == 1`: use bounds center.
   - If `point_count > 1`: use Poisson disk sampling with minimum distance
     `min(width, height) / (point_count + 1)` for even distribution. Fallback to
     random placement if Poisson fails within iteration limit.
2. For each center point:
   - For each ray `j` in `0..ray_count`:
     - Ray angle: `theta = j * (2*PI / ray_count) + small_random_jitter` (jitter
       from rng, ~5% of angular spacing, to avoid mechanical look).
     - Trace ray from center outward for `ray_length` pixels:
       - `(px, py) = (cx + t*cos(theta), cy + t*sin(theta))` for t in 0..ray_length
       - Place barrier cells in a perpendicular band of `ray_thickness` centered
         at (px, py).
       - Stop if position exits bounds.
3. Deduplicate positions. Return them.

**Test invariants:**
- Number of distinct angular clusters from each center >= ray_count
- Total star centers placed == point_count (or close, if bounds are small)
- All positions within bounds

**Parameter defaults:** `point_count: 1, ray_count: 8, ray_length: 20, ray_thickness: 1`

**Parameter constraints:** `point_count >= 1, ray_count >= 2, ray_length >= 1, ray_thickness >= 1`

## Implementation Notes

- All algorithms use `impl Rng` for testability (can pass seeded `SmallRng`).
- Position deduplication: use `HashSet<(u16, u16)>` during generation, convert to
  `Vec<PaintPoint>` at the end. (`PaintPoint` does not derive `Hash`, so
  deduplication must use `(u16, u16)` tuples internally.)
- Bounds clipping is done at cell placement time, not as a post-processing step,
  to avoid unnecessary allocations.
- For floating-point to grid conversion: use `floor()` for pixel-to-cell mapping
  within the bounds coordinate system, then add bounds origin offset.
- Each algorithm is a separate function within the `patterns` module, dispatched
  by `generate_pattern()` match on `PatternParams` variant.

**Review cycles:** 6 (reviewed as part of parent plan review process)
