import type { ReactNode } from "react";
import { useState } from "react";

interface SectionProps {
	title: string;
	description: string;
	children: ReactNode;
	collapsible?: boolean;
	defaultOpen?: boolean;
	headerClassName?: string;
	sectionClassName?: string;
}

export function Section({
	title,
	description,
	children,
	collapsible = false,
	defaultOpen = true,
	headerClassName,
	sectionClassName,
}: SectionProps) {
	const [open, setOpen] = useState(defaultOpen);

	return (
		<section className={`border-b border-petri-border ${sectionClassName ?? ""}`}>
			{collapsible ? (
				<button
					type="button"
					onClick={() => setOpen((value) => !value)}
					className={`w-full text-left px-3 pt-3 pb-2 hover:bg-slate-800/30 ${headerClassName ?? ""}`}
				>
					<div className="flex items-start justify-between gap-2">
						<div>
							<h3 className="text-xs font-semibold uppercase tracking-wide text-slate-200">
								{title}
							</h3>
							<p className="mt-1 text-[11px] text-slate-500">{description}</p>
						</div>
						<span
							aria-hidden="true"
							className={`text-slate-400 transition-transform ${open ? "rotate-180" : ""}`}
						>
							&#x25B4;
						</span>
					</div>
				</button>
			) : (
				<div className={`px-3 pt-3 pb-2 ${headerClassName ?? ""}`}>
					<h3 className="text-xs font-semibold uppercase tracking-wide text-slate-200">{title}</h3>
					<p className="mt-1 text-[11px] text-slate-500">{description}</p>
				</div>
			)}
			{(!collapsible || open) && children}
		</section>
	);
}
