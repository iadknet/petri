import { useId } from "react";
import { NumberInput } from "../config-panel/shared/NumberInput.tsx";

interface ParamFieldProps {
	label: string;
	value: number;
	min: number;
	max: number;
	step: number;
	onChange: (value: number) => void;
	testId?: string;
}

export function ParamField({ label, value, min, max, step, onChange, testId }: ParamFieldProps) {
	const id = useId();
	const clamp = (v: number) => Math.min(max, Math.max(min, v));
	return (
		<div className="flex flex-col gap-1">
			<label htmlFor={id} className="text-[10px] text-slate-400">
				{label}
			</label>
			<div className="flex items-center gap-1.5">
				<NumberInput
					id={id}
					data-testid={testId}
					value={value}
					min={min}
					max={max}
					step={step}
					integer={Number.isInteger(step) && Number.isInteger(min)}
					onChange={onChange}
					className="w-16 px-1 py-0.5 text-[11px] font-mono text-right bg-slate-800 border border-slate-700 rounded text-slate-200"
				/>
				<input
					aria-label={`${label} slider`}
					type="range"
					value={value}
					min={min}
					max={max}
					step={step}
					onChange={(e) => onChange(clamp(Number(e.target.value)))}
					className="flex-1 h-1 accent-emerald-500"
				/>
			</div>
		</div>
	);
}
