import type { FieldDef } from "../shared/types.ts";

export const ACTION_LOG_FIELDS: FieldDef[] = [
	{
		path: "action_log.capacity",
		label: "Log Capacity",
		min: 1,
		max: 100000,
		step: 50,
		sliderMax: 5000,
		testId: "config-field-action-log-capacity",
		defaultValue: 500,
		tooltip: "Maximum number of entries retained in the per-creature action log",
	},
];
