import { RowActions, RowLabel, dirtyTint } from "./FieldRow.tsx";
import { useAppliedValue } from "./baseline.ts";
import { useRowFilter } from "./filter.ts";
import type { BooleanFieldDef } from "./types.ts";

interface ToggleRowProps {
	field: BooleanFieldDef;
	id: string;
	value: boolean;
	serverValue?: boolean;
	disabled: boolean;
	onChange: (path: string, value: boolean) => void;
	testId?: string;
}

const onOff = (value: boolean) => (value ? "on" : "off");

export function ToggleRow({
	field,
	id,
	value,
	serverValue: explicitServerValue,
	disabled,
	onChange,
	testId,
}: ToggleRowProps) {
	const serverValue = useAppliedValue(field.path, explicitServerValue);
	const isDirty = serverValue !== undefined && value !== serverValue;
	const { visible, showHelp } = useRowFilter(field, isDirty);
	if (!visible) return null;

	const inputId = `${id}-input`;
	const runtime = id.startsWith("runtime-");
	const showReset = field.defaultValue !== undefined && value !== field.defaultValue;

	return (
		<div
			data-config-row
			className={`-mx-2 flex flex-col gap-1 rounded px-2 py-1.5 ${isDirty ? dirtyTint(id.startsWith("runtime-")) : ""}`}
		>
			<div className="flex min-h-5 items-center justify-between gap-2">
				<RowLabel inputId={inputId} label={field.label} tooltip={field.tooltip} isDirty={isDirty} />
				<div className="flex shrink-0 items-center gap-1.5">
					<RowActions
						path={field.path}
						isDirty={isDirty}
						appliedText={serverValue === undefined ? undefined : onOff(serverValue)}
						onUndo={() => serverValue !== undefined && onChange(field.path, serverValue)}
						defaultText={showReset ? onOff(field.defaultValue!) : undefined}
						onReset={() => onChange(field.path, field.defaultValue!)}
						disabled={disabled}
					/>
					<input
						id={inputId}
						data-testid={testId}
						type="checkbox"
						role="switch"
						aria-checked={value}
						checked={value}
						disabled={disabled}
						onChange={(e) => onChange(field.path, e.target.checked)}
						className={`config-switch ${runtime ? "config-switch-live" : "config-switch-startup"}`}
					/>
				</div>
			</div>
			{showHelp && field.tooltip && (
				<p className="text-[11px] leading-snug text-slate-400">{field.tooltip}</p>
			)}
		</div>
	);
}
