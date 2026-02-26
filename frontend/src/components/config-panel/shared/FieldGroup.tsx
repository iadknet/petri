import type { ReactNode } from "react";

interface FieldGroupProps {
	title: string;
	children: ReactNode;
}

export function FieldGroup({ title, children }: FieldGroupProps) {
	return (
		<div className="border-b border-petri-border">
			<div className="w-full px-3 py-2 text-xs font-medium text-slate-300 bg-slate-900/20">
				{title}
			</div>
			<div className="px-3 pb-2 flex flex-col gap-0.5">{children}</div>
		</div>
	);
}
