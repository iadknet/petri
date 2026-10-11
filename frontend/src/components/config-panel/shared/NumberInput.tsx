import { type InputHTMLAttributes, useEffect, useRef, useState } from "react";
import { cleanConfigNumber, formatConfigNumber, sameConfigNumber } from "./numberFormat.ts";

type NativeProps = Omit<
	InputHTMLAttributes<HTMLInputElement>,
	"type" | "value" | "onChange" | "min" | "max" | "step"
>;

interface NumberInputProps extends NativeProps {
	value: number;
	/**
	 * Called with each parseable keystroke (unclamped, so multi-digit values can
	 * be typed) and again with the clamped value when the edit is committed.
	 */
	onChange: (value: number) => void;
	min?: number;
	max?: number;
	/** Arrow-key increment; Shift multiplies it by ten. */
	step?: number;
	/** Round committed and stepped values to whole numbers. */
	integer?: boolean;
	/** Keep committed values inside the bounds. Defaults to true. */
	clamp?: boolean;
}

function parse(text: string): number | null {
	if (text.trim() === "") return null;
	const value = Number(text);
	return Number.isFinite(value) ? value : null;
}

/**
 * A number field that holds the typed text while it is being edited, so a
 * cleared field or a half-typed "0." is never written back as 0. Enter or blur
 * commits (clamped and, for integers, rounded); Escape restores the value the
 * field had when it was focused.
 */
export function NumberInput({
	value,
	onChange,
	min,
	max,
	step = 1,
	integer = false,
	clamp = true,
	onBlur,
	onFocus,
	onKeyDown,
	...rest
}: NumberInputProps) {
	const [draft, setDraft] = useState<string | null>(null);
	const focusValue = useRef(value);

	// A value changed from outside (Discard, reset to default, Apply) replaces
	// whatever text was left over from an earlier edit.
	useEffect(() => {
		setDraft((current) => {
			if (current === null) return null;
			const parsed = parse(current);
			return parsed !== null && sameConfigNumber(parsed, value) ? current : null;
		});
	}, [value]);

	const normalize = (next: number) => {
		let result = integer ? Math.round(next) : cleanConfigNumber(next);
		if (clamp) {
			if (min !== undefined) result = Math.max(min, result);
			if (max !== undefined) result = Math.min(max, result);
		}
		return result;
	};

	const commit = () => {
		if (draft === null) return;
		const parsed = parse(draft);
		setDraft(null);
		if (parsed === null) return;
		const next = normalize(parsed);
		if (next !== value) onChange(next);
	};

	const nudge = (direction: 1 | -1, large: boolean) => {
		const base = (draft !== null ? parse(draft) : null) ?? value;
		const next = normalize(base + direction * step * (large ? 10 : 1));
		setDraft(null);
		if (next !== value) onChange(next);
	};

	return (
		<input
			{...rest}
			type="number"
			inputMode="decimal"
			step="any"
			min={min}
			max={max}
			value={draft ?? formatConfigNumber(value)}
			onFocus={(event) => {
				focusValue.current = value;
				onFocus?.(event);
			}}
			onChange={(event) => {
				const text = event.target.value;
				setDraft(text);
				const parsed = parse(text);
				if (parsed !== null && parsed !== value) onChange(parsed);
			}}
			onBlur={(event) => {
				commit();
				onBlur?.(event);
			}}
			onKeyDown={(event) => {
				onKeyDown?.(event);
				if (event.defaultPrevented) return;
				if (event.key === "Enter") {
					commit();
				} else if (event.key === "Escape") {
					setDraft(null);
					if (focusValue.current !== value) onChange(focusValue.current);
				} else if (event.key === "ArrowUp" || event.key === "ArrowDown") {
					event.preventDefault();
					nudge(event.key === "ArrowUp" ? 1 : -1, event.shiftKey);
				}
			}}
			// A focused number field otherwise changes under a scroll wheel that
			// was only meant to scroll the panel.
			onWheel={(event) => event.currentTarget.blur()}
		/>
	);
}
