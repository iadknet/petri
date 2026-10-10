import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { ToggleRow } from "../shared/ToggleRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import type { BooleanFieldDef, FieldDef, RuntimePanelProps } from "../shared/types.ts";
import { resolveRuntimeBounds } from "./bounds.ts";

interface RuntimeFieldGroupProps extends RuntimePanelProps {
	title: string;
	fields: FieldDef[];
	toggles?: BooleanFieldDef[];
}

export function RuntimeFieldGroup({
	title,
	fields,
	toggles,
	localDraft,
	serverConfig,
	simState,
	updateDraft,
	fieldErrors,
}: RuntimeFieldGroupProps) {
	return (
		<FieldGroup title={title}>
			{toggles?.map((field) => (
				<ToggleRow
					key={`runtime-${field.path}`}
					field={field}
					id={`runtime-${field.path.replaceAll(".", "-")}`}
					value={getByPath(localDraft, field.path) as boolean}
					serverValue={getByPath(serverConfig, field.path) as boolean}
					disabled={simState === "running"}
					onChange={updateDraft}
					testId={field.testId}
				/>
			))}
			{fields.map((field) => (
				<FieldRow
					key={`runtime-${field.path}`}
					field={field}
					id={`runtime-${field.path.replaceAll(".", "-")}`}
					value={getByPath(localDraft, field.path) as number}
					serverValue={getByPath(serverConfig, field.path) as number}
					bounds={resolveRuntimeBounds(field, localDraft)}
					error={fieldErrors[field.path]}
					disabled={simState === "running"}
					disabledReason="Locked in current state"
					onChange={updateDraft}
					testId={field.testId}
				/>
			))}
		</FieldGroup>
	);
}
