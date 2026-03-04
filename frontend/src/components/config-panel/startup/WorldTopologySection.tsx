import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { Tooltip } from "../shared/Tooltip.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, StartupSectionProps } from "../shared/types.ts";

const FIELDS: FieldDef[] = [
	{
		path: "world.width",
		label: "Width",
		min: 10,
		max: 2000,
		step: 10,
		testId: "startup-field-world-width",
		defaultValue: 400,
		tooltip: "World grid width in cells",
	},
	{
		path: "world.height",
		label: "Height",
		min: 10,
		max: 2000,
		step: 10,
		testId: "startup-field-world-height",
		defaultValue: 400,
		tooltip: "World grid height in cells",
	},
];

const EDGE_MODES = ["Wrap", "Bounded"] as const;

export function WorldTopologySection({ startupPreset, updateStartupPreset }: StartupSectionProps) {
	return (
		<FieldGroup title="World Topology">
			<div className="flex flex-col gap-1.5 py-1.5 pl-2 border-l-2 border-transparent">
				<label
					htmlFor="startup-world-edge-mode"
					className="flex items-center gap-1 text-xs text-slate-300"
				>
					Edge Mode
					<Tooltip text="How creatures interact with world boundaries. Wrap: opposite edges connect. Bounded: edges are walls.">
						<span className="text-slate-500 cursor-help text-[10px]">&#x24D8;</span>
					</Tooltip>
				</label>
				<select
					id="startup-world-edge-mode"
					data-testid="startup-field-world-edge-mode"
					value={startupPreset.world.edge_mode}
					onChange={(e) => updateStartupPreset("world.edge_mode", e.target.value)}
					className="w-32 px-1.5 py-0.5 text-xs bg-slate-800 border border-slate-700 rounded text-slate-200"
				>
					{EDGE_MODES.map((mode) => (
						<option key={mode} value={mode}>
							{mode}
						</option>
					))}
				</select>
			</div>
			{FIELDS.map((field) => (
				<FieldRow
					key={`startup-${field.path}`}
					field={field}
					id={`startup-${field.path.replaceAll(".", "-")}`}
					value={getByPath(startupPreset, field.path) as number}
					disabled={false}
					onChange={updateStartupPreset}
					testId={field.testId}
				/>
			))}
		</FieldGroup>
	);
}
