import type { ReactNode } from "react";

interface FieldGroupProps {
	title: string;
	children: ReactNode;
}

/** A named run of rows inside a category. Hidden when the filter leaves it empty. */
export function FieldGroup({ title, children }: FieldGroupProps) {
	return (
		<div className="config-subgroup pt-2 pb-1 first:pt-1">
			<h4 className="px-3 pb-0.5 text-[11px] font-medium text-slate-400">{title}</h4>
			<div className="flex flex-col px-3">{children}</div>
		</div>
	);
}
