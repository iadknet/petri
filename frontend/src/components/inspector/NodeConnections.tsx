import type { InputReference, RouteTarget } from "../../types/genome.ts";
import { describeWholeInputRef } from "./inputRefUtils.ts";

interface NodeConnectionsProps {
	inputRefs: InputReference[];
	targets: RouteTarget[];
}

export function NodeConnections({ inputRefs, targets }: NodeConnectionsProps) {
	return (
		<div className="px-3 py-2 text-[10px] font-mono">
			<div className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-0.5">
				<span className="text-slate-400">in</span>
				<span className="text-slate-300">
					{inputRefs.length > 0
						? inputRefs.map((ref, i) => {
								const { label, description } = describeWholeInputRef(ref);
								return (
									// biome-ignore lint/suspicious/noArrayIndexKey: genome order is the identity
									<span key={i} title={description || undefined}>
										{i > 0 ? ", " : ""}
										{label}
									</span>
								);
							})
						: "—"}
				</span>
				<span className="text-slate-400">out</span>
				<span className="text-slate-300">
					{targets.length > 0 ? targets.map((t) => `#${t.target_id}`).join(", ") : "—"}
				</span>
			</div>
		</div>
	);
}
