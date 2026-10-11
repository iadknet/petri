import { EnumRow } from "../shared/EnumRow.tsx";
import { FieldGroup } from "../shared/FieldGroup.tsx";
import { FieldRow } from "../shared/FieldRow.tsx";
import { ToggleRow } from "../shared/ToggleRow.tsx";
import { getByPath } from "../shared/pathUtils.ts";
import {
	type AnyFieldDef,
	type RuntimePanelProps,
	isEnumField,
	isNumberField,
} from "../shared/types.ts";
import { resolveRuntimeBounds } from "./bounds.ts";

interface RuntimeFieldGroupProps extends RuntimePanelProps {
	title: string;
	fields: AnyFieldDef[];
}

export function RuntimeFieldGroup({
	title,
	fields,
	localDraft,
	serverConfig,
	simState,
	updateDraft,
	fieldErrors,
}: RuntimeFieldGroupProps) {
	const disabled = simState === "running";
	return (
		<FieldGroup title={title}>
			{fields.map((field) => {
				const id = `runtime-${field.path.replaceAll(".", "-")}`;
				const common = {
					id,
					disabled,
					onChange: updateDraft,
					testId: field.testId,
				};
				if (isNumberField(field)) {
					return (
						<FieldRow
							key={id}
							{...common}
							field={field}
							value={getByPath(localDraft, field.path) as number}
							serverValue={getByPath(serverConfig, field.path) as number}
							bounds={resolveRuntimeBounds(field, localDraft)}
							error={fieldErrors[field.path]}
							disabledReason="Pause the world to edit live settings"
						/>
					);
				}
				if (isEnumField(field)) {
					return (
						<EnumRow
							key={id}
							{...common}
							field={field}
							value={getByPath(localDraft, field.path) as string}
							serverValue={getByPath(serverConfig, field.path) as string}
							error={fieldErrors[field.path]}
						/>
					);
				}
				return (
					<ToggleRow
						key={id}
						{...common}
						field={field}
						value={getByPath(localDraft, field.path) as boolean}
						serverValue={getByPath(serverConfig, field.path) as boolean}
					/>
				);
			})}
		</FieldGroup>
	);
}
