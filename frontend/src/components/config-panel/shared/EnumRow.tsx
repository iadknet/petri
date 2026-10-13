import { RowActions, RowLabel, dirtyTint } from "./FieldRow.tsx";
import { useAppliedValue } from "./baseline.ts";
import { useRowFilter } from "./filter.ts";
import type { EnumFieldDef } from "./types.ts";

interface EnumRowProps {
	field: EnumFieldDef;
	id: string;
	value: string;
	serverValue?: string;
	disabled: boolean;
	onChange: (path: string, value: string) => void;
	testId?: string;
	error?: string;
}

export const SELECT_CLASS =
	"min-w-0 rounded border border-slate-700 bg-slate-950/60 px-1.5 py-0.5 text-xs text-slate-100 transition-colors hover:border-slate-600 focus:border-sky-500/70 focus:outline-none disabled:cursor-not-allowed disabled:opacity-45";

export function EnumRow({
	field,
	id,
	value,
	serverValue: explicitServerValue,
	disabled,
	onChange,
	testId,
	error,
}: EnumRowProps) {
	const serverValue = useAppliedValue(field.path, explicitServerValue);
	const isDirty = serverValue !== undefined && value !== serverValue;
	const { visible, showHelp } = useRowFilter(field, isDirty);
	if (!visible) return null;

	const inputId = `${id}-input`;
	const label = (option: string | undefined) =>
		field.options.find((candidate) => candidate.value === option)?.label ?? option;
	const showReset = field.defaultValue !== undefined && value !== field.defaultValue;

	return (
		<div
			data-config-row
			className={`-mx-2 flex flex-col gap-1 rounded px-2 py-1.5 ${isDirty ? dirtyTint(id.startsWith("runtime-")) : ""}`}
		>
			<div className="flex min-h-5 items-center justify-between gap-2">
				<RowLabel inputId={inputId} label={field.label} tooltip={field.tooltip} isDirty={isDirty} />
				<RowActions
					path={field.path}
					isDirty={isDirty}
					appliedText={serverValue === undefined ? undefined : label(serverValue)}
					onUndo={() => serverValue !== undefined && onChange(field.path, serverValue)}
					defaultText={showReset ? label(field.defaultValue) : undefined}
					onReset={() => onChange(field.path, field.defaultValue!)}
					disabled={disabled}
				/>
			</div>
			<select
				id={inputId}
				data-testid={testId}
				value={value}
				disabled={disabled}
				onChange={(e) => onChange(field.path, e.target.value)}
				className={`${SELECT_CLASS} w-full`}
			>
				{field.options.map((option) => (
					<option key={option.value} value={option.value}>
						{option.label}
					</option>
				))}
			</select>
			{showHelp && field.tooltip && (
				<p className="text-[11px] leading-snug text-slate-400">{field.tooltip}</p>
			)}
			{error && (
				<p
					data-testid={`field-error-${field.path.replaceAll(".", "-")}`}
					className="text-[11px] text-red-400"
				>
					{error}
				</p>
			)}
		</div>
	);
}
