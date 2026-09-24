import type { SimulationConfig } from "../../../types/api.ts";
import type { FieldDef } from "../shared/types.ts";

export interface FieldBounds {
	min: number;
	max: number;
}

/**
 * Cross-field constraints that `SimulationConfig::normalize` enforces on the
 * server. Static `min`/`max` cannot express them, so the panel derives the
 * offending bound from the current draft and clamps against it; without this
 * the sliders offer values every PATCH refuses (2026-09-07 apply audit, item 5).
 */
const DRAFT_BOUNDS: Record<
	string,
	(draft: SimulationConfig, field: FieldDef) => Partial<FieldBounds>
> = {
	"population.max_creatures": (draft) => ({ min: draft.population.initial_creatures }),
	// The static minimum keeps the resolver total when the draft has no food types.
	"world.food.shared.max_density": (draft, field) => ({
		min: Math.max(field.min, ...draft.world.food.types.map((type) => type.initial_density)),
	}),
	"action_log.capacity": () => ({ min: 1 }),
};

/** Resolves the bounds a runtime control may offer for the current draft. */
export function resolveRuntimeBounds(field: FieldDef, draft: SimulationConfig): FieldBounds {
	const derived = DRAFT_BOUNDS[field.path]?.(draft, field) ?? {};
	const min = derived.min ?? field.min;
	// The derived minimum is the hard server constraint, so it wins whenever the
	// two disagree and the pair would otherwise be inverted.
	return { min, max: Math.max(min, derived.max ?? field.max) };
}

export function clampToBounds(value: number, bounds: FieldBounds): number {
	return Math.min(Math.max(value, bounds.min), bounds.max);
}
