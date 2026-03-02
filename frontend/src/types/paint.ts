export type PaintTool = "food" | "barrier" | "erase_food" | "erase_barrier";

export interface PaintPoint {
	x: number;
	y: number;
}

export interface PaintRequest {
	tool: PaintTool;
	brush_half_extent: 0 | 1 | 2;
	points: PaintPoint[];
}

export interface PaintStats {
	affected_cells: number;
	food_set_cells: number;
	food_cleared_cells: number;
	barrier_set_cells: number;
	barrier_cleared_cells: number;
	creatures_removed: number;
}

export interface DirtyRect {
	x: number;
	y: number;
	width: number;
	height: number;
}

export interface PaintResponse {
	protocol_version: string;
	stats: PaintStats;
	dirty_rect: DirtyRect;
	world_static_changed: boolean;
}
