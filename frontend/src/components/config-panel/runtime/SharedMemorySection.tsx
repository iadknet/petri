import type { FieldDef } from "../shared/types.ts";

export const SHARED_MEMORY_FIELDS: FieldDef[] = [
	{
		path: "shared_memory.decay_rate",
		label: "Decay Rate",
		min: 0,
		max: 1,
		step: 0.01,
		testId: "config-field-shared-memory-decay-rate",
		defaultValue: 0.0,
		tooltip: "Rate at which shared memory values decay each tick (0 = no decay)",
	},
];
