import { useContext } from "react";
import { type FieldBounds, clampToBounds } from "../runtime/bounds.ts";
import { LiveScopeContext } from "./Category.tsx";
import { Icon } from "./Icon.tsx";
import { NumberInput } from "./NumberInput.tsx";
import { Tooltip } from "./Tooltip.tsx";
import { useAppliedValue } from "./baseline.ts";
import { useRowFilter } from "./filter.ts";
import { formatConfigNumber, sameConfigNumber } from "./numberFormat.ts";
import type { FieldDef } from "./types.ts";

export const NUMBER_INPUT_CLASS =
	"w-24 shrink-0 rounded border border-slate-700 bg-slate-950/60 px-1.5 py-0.5 text-right font-mono text-xs text-slate-100 tabular-nums transition-colors hover:border-slate-600 focus:border-sky-500/70 focus:outline-none disabled:cursor-not-allowed disabled:opacity-45 disabled:hover:border-slate-700";

interface FieldRowProps {
	field: FieldDef;
	id: string;
	value: number;
	/** The applied value; when given, a differing draft is marked and can be undone. */
	serverValue?: number;
	disabled: boolean;
	disabledReason?: string;
	onChange: (path: string, value: number) => void;
	testId?: string;
	/** Draft-derived bounds that tighten the field's own hard bounds. */
	bounds?: FieldBounds;
	/** Reason the server refused this field on the last Apply. */
	error?: string;
}

export const dirtyTint = (live: boolean) => (live ? "bg-sky-400/[0.07]" : "bg-emerald-400/[0.07]");

export function RowActions({
	path,
	isDirty,
	appliedText,
	onUndo,
	defaultText,
	onReset,
	disabled,
}: {
	path: string;
	isDirty: boolean;
	appliedText?: string;
	onUndo: () => void;
	defaultText?: string;
	onReset?: () => void;
	disabled: boolean;
}) {
	const key = path.replaceAll(".", "-");
	return (
		<div className="flex shrink-0 items-center gap-0.5">
			{isDirty && appliedText !== undefined && (
				<button
					type="button"
					data-testid={`revert-${key}`}
					onClick={onUndo}
					disabled={disabled}
					title={`Undo: back to the applied value (${appliedText})`}
					className="flex h-5 items-center gap-1 rounded px-1 font-mono text-[10px] text-slate-400 hover:bg-slate-800 hover:text-slate-100 disabled:opacity-40"
				>
					<span className="sr-only">Undo change, applied value</span>
					was {appliedText}
					<Icon name="undo" className="size-3" />
				</button>
			)}
			{onReset && defaultText !== undefined && (
				<button
					type="button"
					data-testid={`reset-${key}`}
					onClick={onReset}
					disabled={disabled}
					title={`Reset to default (${defaultText})`}
					aria-label={`Reset to default (${defaultText})`}
					className="flex size-5 items-center justify-center rounded text-slate-500 opacity-60 hover:bg-slate-800 hover:text-slate-200 hover:opacity-100 focus-visible:opacity-100 disabled:opacity-30"
				>
					<Icon name="reset" className="size-3" />
				</button>
			)}
		</div>
	);
}

export function RowLabel({
	inputId,
	label,
	tooltip,
	isDirty,
}: {
	inputId: string;
	label: string;
	tooltip?: string;
	isDirty: boolean;
}) {
	const live = useContext(LiveScopeContext);
	return (
		<div className="flex min-w-0 items-center gap-1.5">
			{isDirty && (
				<span
					className={`size-1.5 shrink-0 rounded-full ${live ? "bg-sky-400" : "bg-emerald-400"}`}
					title={live ? "Changed, not applied" : "Changes at the next Restart"}
				/>
			)}
			<label
				htmlFor={inputId}
				className={`truncate text-xs ${isDirty ? "text-slate-100" : "text-slate-300"}`}
			>
				{label}
			</label>
			{tooltip && <Tooltip text={tooltip} />}
		</div>
	);
}

export function FieldRow({
	field,
	id,
	value,
	disabled,
	disabledReason,
	onChange,
	testId,
	bounds,
	error,
	serverValue: explicitServerValue,
}: FieldRowProps) {
	const serverValue = useAppliedValue(field.path, explicitServerValue);
	const isDirty = serverValue !== undefined && !sameConfigNumber(value, serverValue);
	const { visible, showHelp } = useRowFilter(field, isDirty);
	if (!visible) return null;

	const inputId = `${id}-input`;
	const runtime = id.startsWith("runtime-");
	const hard = bounds ?? { min: field.min, max: field.max };
	const isDefault = field.defaultValue === undefined || sameConfigNumber(value, field.defaultValue);
	const integer = Number.isInteger(field.step) && Number.isInteger(hard.min);
	// The slider covers a comfortable span, widened to include any real value
	// outside it so the thumb never lies about where the value sits.
	// Values only pass the hard bounds mid-typing, so the span ignores those.
	const inBounds = clampToBounds(value, hard);
	const sliderMin = Math.min(Math.max(field.sliderMin ?? hard.min, hard.min), inBounds);
	const sliderMax = Math.max(Math.min(field.sliderMax ?? hard.max, hard.max), inBounds);

	return (
		<div
			data-config-row
			title={disabled ? disabledReason : undefined}
			className={`-mx-2 flex flex-col gap-1 rounded px-2 py-1.5 ${isDirty ? dirtyTint(runtime) : ""}`}
		>
			<div className="flex min-h-5 items-center justify-between gap-2">
				<RowLabel inputId={inputId} label={field.label} tooltip={field.tooltip} isDirty={isDirty} />
				<RowActions
					path={field.path}
					isDirty={isDirty}
					appliedText={serverValue === undefined ? undefined : formatConfigNumber(serverValue)}
					onUndo={() => serverValue !== undefined && onChange(field.path, serverValue)}
					defaultText={
						isDefault || field.defaultValue === undefined
							? undefined
							: formatConfigNumber(field.defaultValue)
					}
					onReset={() =>
						field.defaultValue !== undefined && onChange(field.path, field.defaultValue)
					}
					disabled={disabled}
				/>
			</div>
			<div className="flex items-center gap-2.5">
				<NumberInput
					id={inputId}
					data-testid={testId}
					disabled={disabled}
					value={value}
					min={hard.min}
					max={hard.max}
					step={field.step}
					integer={integer}
					aria-invalid={error ? true : undefined}
					onChange={(next) => onChange(field.path, next)}
					className={`${NUMBER_INPUT_CLASS} ${error ? "border-red-500/70" : ""}`}
				/>
				{field.slider !== false && (
					<input
						aria-label={`${field.label} slider`}
						type="range"
						tabIndex={-1}
						disabled={disabled}
						value={inBounds}
						min={sliderMin}
						max={sliderMax}
						step={field.step}
						onChange={(e) => onChange(field.path, clampToBounds(Number(e.target.value), hard))}
						className={`config-slider h-1 w-full min-w-0 ${runtime ? "accent-sky-500" : "accent-emerald-500"} disabled:cursor-not-allowed disabled:opacity-40`}
					/>
				)}
			</div>
			{showHelp && field.tooltip && (
				<p className="text-[11px] leading-snug text-slate-500">{field.tooltip}</p>
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
