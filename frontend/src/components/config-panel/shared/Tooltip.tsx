import type { ReactNode } from "react";

interface TooltipProps {
	text: string;
	children: ReactNode;
}

export function Tooltip({ text, children }: TooltipProps) {
	return (
		<span className="relative group/tooltip inline-flex">
			{children}
			<span className="absolute left-1/2 -translate-x-1/2 bottom-full mb-1.5 px-2 py-1 text-[10px] leading-tight text-slate-200 bg-slate-700 rounded shadow-lg whitespace-normal max-w-48 pointer-events-none opacity-0 group-hover/tooltip:opacity-100 transition-opacity z-50">
				{text}
			</span>
		</span>
	);
}
