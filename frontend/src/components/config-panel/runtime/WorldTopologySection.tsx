import { CollapsibleGroup } from "../shared/CollapsibleGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, RuntimePanelProps } from "../shared/types.ts";

export const WORLD_TOPOLOGY_FIELDS: FieldDef[] = [
	{
		path: "world.width",
		label: "Width",
		min: 10,
		max: 2000,
		step: 10,
		topologyField: true,
		testId: "config-field-world-width",
	},
	{
		path: "world.height",
		label: "Height",
		min: 10,
		max: 2000,
		step: 10,
		topologyField: true,
	},
];

export function WorldTopologySection({
	localDraft,
	serverConfig,
	simState,
	updateDraft,
}: RuntimePanelProps) {
	return (
		<CollapsibleGroup title="World Topology">
			{WORLD_TOPOLOGY_FIELDS.map((field) => (
				<FieldRow
					key={`runtime-${field.path}`}
					field={field}
					id={`runtime-${field.path.replaceAll(".", "-")}`}
					value={getByPath(localDraft, field.path) as number}
					serverValue={getByPath(serverConfig, field.path) as number}
					disabled={simState !== "idle"}
					onChange={updateDraft}
					testId={field.testId}
				/>
			))}
		</CollapsibleGroup>
	);
}
