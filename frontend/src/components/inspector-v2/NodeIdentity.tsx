import { meshBackendTones } from "../inspector/mesh/meshPresentation.ts";

interface NodeIdentityProps {
	nodeId: number;
	backendKind: "vm" | "graph";
	backendSummary: { label: string; detail: string };
	isEntry: boolean;
	reachable: boolean;
	semanticLabel: string | null;
	badges: string[];
}

export function NodeIdentity({
	nodeId,
	backendKind,
	backendSummary,
	isEntry,
	reachable,
	semanticLabel,
	badges,
}: NodeIdentityProps) {
	const tone = meshBackendTones[backendKind];

	return (
		<div className="px-3 py-2">
			<div className="flex items-baseline gap-2">
				<span className="font-mono text-xs font-medium text-slate-50">#{nodeId}</span>
				<span
					className="text-[10px] font-mono uppercase"
					style={{ color: tone.muted }}
				>
					{backendSummary.label}
				</span>
				{semanticLabel ? (
					<span className="text-xs text-cyan-200">{semanticLabel}</span>
				) : null}
			</div>
			<div className="mt-0.5 flex items-center gap-2 text-[10px] font-mono text-slate-500">
				<span>{backendSummary.detail}</span>
				{isEntry ? <span className="text-emerald-400">entry</span> : null}
				{!reachable ? <span className="text-slate-500">unreachable</span> : null}
				{badges.map((badge) => (
					<span key={badge} className="text-cyan-400/70">{badge}</span>
				))}
			</div>
		</div>
	);
}
