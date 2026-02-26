import { Tooltip } from "./Tooltip.tsx";
import type { FieldDef } from "./types.ts";

interface FieldRowProps {
	field: FieldDef;
	id: string;
	value: number;
	serverValue?: number;
	disabled: boolean;
	onChange: (path: string, value: number) => void;
	testId?: string;
}

export function FieldRow({
	field,
	id,
	value,
	serverValue,
	disabled,
	onChange,
	testId,
}: FieldRowProps) {
	const isDirty = serverValue !== undefined ? value !== serverValue : false;
	const inputId = `${id}-input`;
	const sliderAccentClass = id.startsWith("runtime-") ? "accent-sky-500" : "accent-emerald-500";
	const showReset = field.defaultValue !== undefined && value !== field.defaultValue;

	return (
		<div
			className={`flex flex-col gap-1.5 py-1.5 pl-2 border-l-2 ${isDirty ? "border-blue-500" : "border-transparent"}`}
		>
			<div className="flex items-center justify-between gap-2">
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
			</div>
			<div className="flex items-center gap-2">
				<input
					id={inputId}
					data-testid={testId}
					type="number"
					disabled={disabled}
					value={value}
					min={field.min}
					max={field.max}
					step={field.step}
					onChange={(e) => onChange(field.path, Number(e.target.value))}
					className="w-24 px-1.5 py-0.5 text-xs font-mono text-right bg-slate-800 border border-slate-700 rounded text-slate-200 disabled:opacity-40"
				/>
				<input
					aria-label={`${field.label} slider`}
					type="range"
					disabled={disabled}
					value={value}
					min={field.min}
					max={field.max}
					step={field.step}
					onChange={(e) => onChange(field.path, Number(e.target.value))}
					className={`w-full h-1 ${sliderAccentClass} disabled:opacity-40`}
				/>
			</div>
		</div>
	);
}
