import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { FieldDef, RuntimePanelProps } from "../shared/types.ts";

interface RuntimeFieldGroupProps extends RuntimePanelProps {
	title: string;
	fields: FieldDef[];
}

export function RuntimeFieldGroup({
	title,
	fields,
	localDraft,
	serverConfig,
	simState,
	updateDraft,
}: RuntimeFieldGroupProps) {
	return (
		<FieldGroup title={title}>
			{fields.map((field) => (
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
		</FieldGroup>
	);
}
