import type { FieldDef } from "../shared/types.ts";

export const ACTION_LOG_FIELDS: FieldDef[] = [
	{
		path: "action_log.capacity",
		label: "Log Capacity",
		min: 0,
		max: 5000,
		step: 50,
		testId: "config-field-action-log-capacity",
		defaultValue: 500,
		tooltip: "Maximum number of entries retained in the per-creature action log",
	},
];
