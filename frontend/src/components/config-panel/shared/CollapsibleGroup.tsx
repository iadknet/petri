import type { ReactNode } from "react";
import { useState } from "react";

interface CollapsibleGroupProps {
	title: string;
	children: ReactNode;
	defaultOpen?: boolean;
}

export function CollapsibleGroup({ title, children, defaultOpen = true }: CollapsibleGroupProps) {
	const [open, setOpen] = useState(defaultOpen);

	return (
		<div className="border-b border-petri-border">
			<button
				type="button"
				onClick={() => setOpen(!open)}
				className="w-full flex items-center justify-between px-3 py-2 text-xs font-medium text-slate-300 hover:bg-slate-800/50"
			>
				{title}
				<span className={`transition-transform ${open ? "rotate-180" : ""}`}>&#x25B4;</span>
			</button>
			{open && <div className="px-3 pb-2 flex flex-col gap-0.5">{children}</div>}
		</div>
	);
}
