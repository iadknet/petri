import { CollapsibleGroup } from "../shared/CollapsibleGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, RuntimePanelProps } from "../shared/types.ts";

export const RUNTIME_FIELDS: FieldDef[] = [
	{
		path: "runtime.max_mesh_hops",
		label: "Max Mesh Hops",
		min: 1,
		max: 1024,
		step: 1,
	},
	{
		path: "runtime.max_vm_steps",
		label: "Max VM Steps",
		min: 1,
		max: 10000,
		step: 1,
	},
	{
		path: "runtime.max_graph_relax_iters",
		label: "Graph Relax Iters",
		min: 1,
		max: 100,
		step: 1,
	},
	{
		path: "runtime.graph_convergence_epsilon",
		label: "Convergence Epsilon",
		min: 0.0001,
		max: 1,
		step: 0.0001,
	},
	{
		path: "runtime.graph_convergence_stable_passes",
		label: "Stable Passes",
		min: 1,
		max: 10,
		step: 1,
	},
	{
		path: "runtime.graph_node_base_cost",
		label: "Node Base Cost",
		min: 0,
		max: 100,
		step: 0.1,
	},
	{
		path: "runtime.vm.opcode_cost_multiplier",
		label: "Opcode Cost Mult.",
		min: 0,
		max: 10,
		step: 0.1,
	},
];

export function RuntimeSection({
	localDraft,
	serverConfig,
	simState,
	updateDraft,
}: RuntimePanelProps) {
	return (
		<CollapsibleGroup title="Runtime">
			{RUNTIME_FIELDS.map((field) => (
				<FieldRow
					key={`runtime-${field.path}`}
					field={field}
					id={`runtime-${field.path.replaceAll(".", "-")}`}
					value={getByPath(localDraft, field.path) as number}
					serverValue={getByPath(serverConfig, field.path) as number}
					disabled={simState === "running"}
					onChange={updateDraft}
					testId={field.testId}
				/>
			))}
		</CollapsibleGroup>
	);
}
