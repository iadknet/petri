import {
	type ComponentProps,
	type FocusEvent,
	type KeyboardEvent,
	type MouseEvent,
	type ReactNode,
	useId,
	useLayoutEffect,
	useRef,
	useState,
} from "react";
import { createPortal } from "react-dom";
import { Icon } from "./Icon.tsx";

const GAP = 6;
const EDGE = 8;

/**
 * A tooltip bubble beside `anchor` (above it, or below when there is no
 * room). It is portalled and fixed-positioned so a scrolling sidebar or an
 * SVG canvas never clips it.
 */
export function TipBubble({ anchor, children }: { anchor: DOMRect; children: ReactNode }) {
	const bubble = useRef<HTMLDivElement>(null);
	const [position, setPosition] = useState<{ left: number; top: number } | null>(null);

	useLayoutEffect(() => {
		if (!bubble.current) return;
		const box = bubble.current.getBoundingClientRect();
		const left = Math.min(
			Math.max(EDGE, anchor.left + anchor.width / 2 - box.width / 2),
			window.innerWidth - box.width - EDGE,
		);
		const above = anchor.top - box.height - GAP;
		const top = above >= EDGE ? above : anchor.bottom + GAP;
		setPosition({ left, top });
	}, [anchor]);

	return createPortal(
		<div
			ref={bubble}
			aria-hidden="true"
			data-testid="tooltip"
			style={{
				left: position?.left ?? 0,
				top: position?.top ?? 0,
				visibility: position ? "visible" : "hidden",
			}}
			className="pointer-events-none fixed z-50 max-w-72 whitespace-normal rounded-md border border-slate-700 bg-slate-800 px-2.5 py-1.5 font-sans text-[11px] leading-snug text-slate-200 shadow-[0_8px_24px_rgba(2,6,23,0.6)]"
		>
			{children}
		</div>,
		document.body,
	);
}

/** Open/close state for a bubble anchored at an element or a pointer. */
export function useTipAnchor() {
	const [anchor, setAnchor] = useState<DOMRect | null>(null);
	return { anchor, show: setAnchor, hide: () => setAnchor(null) };
}

/**
 * Handlers that open a tooltip on hover, keyboard focus or a tap and close it
 * on leave, blur or Escape, and `tip(content)`: the screen-reader text the
 * trigger is described by plus the bubble while open.
 */
export function useTipTrigger() {
	const id = useId();
	const { anchor, show, hide } = useTipAnchor();
	const open = (event: MouseEvent<Element> | FocusEvent<Element>) =>
		show(event.currentTarget.getBoundingClientRect());
	return {
		triggerProps: {
			"aria-describedby": id,
			onMouseEnter: open,
			onMouseLeave: hide,
			onFocus: open,
			onBlur: hide,
			onKeyDown: (event: KeyboardEvent<Element>) => {
				if (event.key === "Escape") hide();
			},
			// A click only opens: hover already did, and toggling would close it under the pointer.
			onClick: open,
		},
		tip: (content: ReactNode) => (
			<>
				<span id={id} className="sr-only">
					{content}
				</span>
				{anchor && <TipBubble anchor={anchor}>{content}</TipBubble>}
			</>
		),
	};
}

/**
 * A short on-screen label whose detail opens in a tooltip, dotted-underlined
 * unless `plain` (a chip that already reads as one).
 */
export function Tip({
	content,
	children,
	className = "",
	plain = false,
	...rest
}: Omit<ComponentProps<"span">, "content"> & {
	content: ReactNode;
	plain?: boolean;
}) {
	const { triggerProps, tip } = useTipTrigger();
	return (
		<>
			<span
				{...rest}
				// biome-ignore lint/a11y/noNoninteractiveTabindex: focus opens the tooltip for keyboard users
				tabIndex={0}
				{...triggerProps}
				className={`cursor-help rounded-sm focus-visible:outline-1 focus-visible:outline-sky-400 ${
					plain ? "" : "underline decoration-slate-500 decoration-dotted underline-offset-2"
				} ${className}`}
			>
				{children}
			</span>
			{tip(content)}
		</>
	);
}

/** An info mark that explains a setting on hover or keyboard focus. */
export function Tooltip({ text }: { text: string }) {
	const { triggerProps, tip } = useTipTrigger();
	return (
		<>
			<button
				type="button"
				aria-label="About this setting"
				{...triggerProps}
				onClick={(event) => {
					// Inside a <label>, a click would otherwise focus the field.
					event.preventDefault();
					triggerProps.onClick(event);
				}}
				className="relative inline-flex size-4 items-center justify-center rounded-sm text-slate-400 before:absolute before:-inset-1 hover:text-slate-300 focus-visible:text-slate-200 focus-visible:outline-2 focus-visible:outline-sky-400"
			>
				<Icon name="info" className="size-3" />
			</button>
			{tip(text)}
		</>
	);
}
