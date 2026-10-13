import { useId, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { Icon } from "./Icon.tsx";

interface TooltipProps {
	text: string;
}

const GAP = 6;
const EDGE = 8;

/**
 * An info mark that explains a setting on hover or keyboard focus. The bubble
 * is portalled and fixed-positioned so the scrolling sidebar never clips it.
 */
export function Tooltip({ text }: TooltipProps) {
	const id = useId();
	const trigger = useRef<HTMLButtonElement>(null);
	const bubble = useRef<HTMLDivElement>(null);
	const [open, setOpen] = useState(false);
	const [position, setPosition] = useState<{ left: number; top: number } | null>(null);

	useLayoutEffect(() => {
		if (!open || !trigger.current || !bubble.current) return;
		const anchor = trigger.current.getBoundingClientRect();
		const box = bubble.current.getBoundingClientRect();
		const left = Math.min(
			Math.max(EDGE, anchor.left + anchor.width / 2 - box.width / 2),
			window.innerWidth - box.width - EDGE,
		);
		const above = anchor.top - box.height - GAP;
		const top = above >= EDGE ? above : anchor.bottom + GAP;
		setPosition({ left, top });
	}, [open]);

	const show = () => setOpen(true);
	const hide = () => {
		setOpen(false);
		setPosition(null);
	};

	return (
		<>
			<button
				ref={trigger}
				type="button"
				aria-label="About this setting"
				aria-describedby={id}
				onMouseEnter={show}
				onMouseLeave={hide}
				onFocus={show}
				onBlur={hide}
				onKeyDown={(event) => {
					if (event.key === "Escape") hide();
				}}
				onClick={(event) => {
					// Inside a <label>, a click would otherwise focus the field. A click only
					// opens: hover already did, and toggling would close it under the pointer.
					event.preventDefault();
					setOpen(true);
				}}
				className="inline-flex size-4 items-center justify-center rounded-sm text-slate-400 hover:text-slate-300 focus-visible:text-slate-200 focus-visible:outline-2 focus-visible:outline-sky-400"
			>
				<Icon name="info" className="size-3" />
			</button>
			<span id={id} className="sr-only">
				{text}
			</span>
			{open &&
				createPortal(
					<div
						ref={bubble}
						aria-hidden="true"
						data-testid="config-tooltip"
						style={{
							left: position?.left ?? 0,
							top: position?.top ?? 0,
							visibility: position ? "visible" : "hidden",
						}}
						className="pointer-events-none fixed z-50 max-w-64 rounded-md border border-slate-700 bg-slate-800 px-2.5 py-1.5 text-[11px] leading-snug text-slate-200 shadow-[0_8px_24px_rgba(2,6,23,0.6)]"
					>
						{text}
					</div>,
					document.body,
				)}
		</>
	);
}
