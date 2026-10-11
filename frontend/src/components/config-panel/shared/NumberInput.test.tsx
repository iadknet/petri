import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import { NumberInput } from "./NumberInput.tsx";
import { formatConfigNumber, sameConfigNumber } from "./numberFormat.ts";

function Harness({
	initial,
	onValue,
	min = 0,
	max = 10,
	step = 0.1,
}: {
	initial: number;
	onValue?: (value: number) => void;
	min?: number;
	max?: number;
	step?: number;
}) {
	const [value, setValue] = useState(initial);
	return (
		<NumberInput
			aria-label="field"
			value={value}
			min={min}
			max={max}
			step={step}
			onChange={(next) => {
				setValue(next);
				onValue?.(next);
			}}
		/>
	);
}

describe("formatConfigNumber", () => {
	it("hides f32 widening noise", () => {
		expect(formatConfigNumber(0.699999988079071)).toBe("0.7");
		expect(formatConfigNumber(9.999999974752427e-7)).toBe("0.000001");
		expect(formatConfigNumber(9.999999747378752e-5)).toBe("0.0001");
		expect(formatConfigNumber(10000)).toBe("10000");
	});

	it("treats an f32 round trip as the same number", () => {
		expect(sameConfigNumber(0.699999988079071, 0.7)).toBe(true);
		expect(sameConfigNumber(0.7, 0.71)).toBe(false);
	});
});

describe("NumberInput", () => {
	it("shows the clean value", () => {
		render(<Harness initial={0.699999988079071} />);
		expect(screen.getByLabelText("field")).toHaveValue(0.7);
	});

	it("does not turn a cleared field into zero", () => {
		const onValue = vi.fn();
		render(<Harness initial={4} onValue={onValue} />);
		const input = screen.getByLabelText("field");
		fireEvent.focus(input);
		fireEvent.change(input, { target: { value: "" } });
		expect(onValue).not.toHaveBeenCalled();
		fireEvent.blur(input);
		expect(onValue).not.toHaveBeenCalled();
		expect(input).toHaveValue(4);
	});

	it("passes typed values through and clamps on commit", () => {
		const onValue = vi.fn();
		render(<Harness initial={4} onValue={onValue} />);
		const input = screen.getByLabelText("field");
		fireEvent.focus(input);
		fireEvent.change(input, { target: { value: "25" } });
		expect(onValue).toHaveBeenLastCalledWith(25);
		fireEvent.keyDown(input, { key: "Enter" });
		expect(onValue).toHaveBeenLastCalledWith(10);
		expect(input).toHaveValue(10);
	});

	it("reverts to the value it had on focus with Escape", () => {
		const onValue = vi.fn();
		render(<Harness initial={4} onValue={onValue} />);
		const input = screen.getByLabelText("field");
		fireEvent.focus(input);
		fireEvent.change(input, { target: { value: "7" } });
		fireEvent.keyDown(input, { key: "Escape" });
		expect(onValue).toHaveBeenLastCalledWith(4);
		expect(input).toHaveValue(4);
	});

	it("steps with the arrow keys, ten times with Shift, inside the bounds", () => {
		render(<Harness initial={4} step={0.1} max={5} />);
		const input = screen.getByLabelText("field");
		fireEvent.keyDown(input, { key: "ArrowUp" });
		expect(input).toHaveValue(4.1);
		fireEvent.keyDown(input, { key: "ArrowDown", shiftKey: true });
		expect(input).toHaveValue(3.1);
		fireEvent.keyDown(input, { key: "ArrowUp", shiftKey: true });
		fireEvent.keyDown(input, { key: "ArrowUp", shiftKey: true });
		expect(input).toHaveValue(5);
	});

	it("never reports the native step as invalid", () => {
		render(<Harness initial={0.0002} step={0.001} />);
		expect(screen.getByLabelText("field")).toHaveAttribute("step", "any");
	});
});
