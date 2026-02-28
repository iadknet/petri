import { Tooltip } from "./Tooltip.tsx";
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

export function ToggleRow({
	field,
	id,
	value,
	serverValue,
	disabled,
	onChange,
	testId,
}: ToggleRowProps) {
	const isDirty = serverValue !== undefined ? value !== serverValue : false;
	const inputId = `${id}-input`;
	const showReset = field.defaultValue !== undefined && value !== field.defaultValue;

	return (
		<div
			className={`flex items-center justify-between gap-2 py-1.5 pl-2 border-l-2 ${isDirty ? "border-blue-500" : "border-transparent"}`}
		>
			<label htmlFor={inputId} className="flex items-center gap-1 text-xs text-slate-300">
				{disabled && (
					<span className="text-slate-500" title="Locked in current state">
						&#x1f512;
					</span>
				)}
				{field.label}
				{field.tooltip && (
					<Tooltip text={field.tooltip}>
						<span className="text-slate-500 cursor-help text-[10px]">&#x24D8;</span>
					</Tooltip>
				)}
			</label>
			<div className="flex items-center gap-1.5">
				{showReset && (
					<button
						type="button"
						data-testid={`reset-${field.path.replaceAll(".", "-")}`}
						onClick={() => onChange(field.path, field.defaultValue!)}
						disabled={disabled}
						className="text-[10px] text-slate-500 hover:text-slate-300 disabled:opacity-40"
						title={`Reset to default (${field.defaultValue})`}
					>
						&#x21ba;
					</button>
				)}
				<input
					id={inputId}
					data-testid={testId}
					type="checkbox"
					checked={value}
					disabled={disabled}
					onChange={(e) => onChange(field.path, e.target.checked)}
					className="accent-sky-500 disabled:opacity-40"
				/>
			</div>
		</div>
	);
}
