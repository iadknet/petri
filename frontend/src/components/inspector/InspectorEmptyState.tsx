interface InspectorEmptyStateProps {
	title: string;
	description: string;
}

export function InspectorEmptyState({ title, description }: InspectorEmptyStateProps) {
	return (
		<div className="px-4 py-6">
			<div className="rounded-2xl border border-dashed border-slate-800 bg-slate-900/60 px-4 py-5">
				<p className="text-xs font-mono uppercase tracking-[0.2em] text-slate-500">{title}</p>
				<p className="mt-2 text-sm leading-6 text-slate-400">{description}</p>
			</div>
		</div>
	);
}
