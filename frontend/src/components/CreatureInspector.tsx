import { useCreatureDetail } from "../hooks/useCreatureDetail.ts";
import { useCreatureInspectorStore } from "../stores/creatureInspector.ts";
import { InspectorHeader } from "./inspector/InspectorHeader.tsx";
import { MemoryHexView } from "./inspector/MemoryHexView.tsx";
import { NodeGraph } from "./inspector/NodeGraph.tsx";
import { PhenotypeDetail } from "./inspector/PhenotypeDetail.tsx";
import { StatsSection } from "./inspector/StatsSection.tsx";

function CreatureInspector() {
	useCreatureDetail();

	const stats = useCreatureInspectorStore((s) => s.creatureStats);
	const genome = useCreatureInspectorStore((s) => s.creatureGenome);
	const memory = useCreatureInspectorStore((s) => s.creatureMemory);
	const isLoading = useCreatureInspectorStore((s) => s.isLoading);
	const isDead = useCreatureInspectorStore((s) => s.isDead);
	const error = useCreatureInspectorStore((s) => s.error);
	const clearSelection = useCreatureInspectorStore((s) => s.clearSelection);

	return (
		<div
			role="complementary"
			aria-label="Creature Inspector"
			className="h-full bg-slate-900 border-l border-slate-800 flex flex-col overflow-hidden"
		>
			{isLoading && !stats && <LoadingSkeleton />}

			{error && (
				<div className="px-4 py-6 text-center">
					<p className="text-sm text-red-400">{error}</p>
					<button
						type="button"
						onClick={clearSelection}
						className="mt-2 text-xs text-slate-400 hover:text-slate-200"
					>
						Dismiss
					</button>
				</div>
			)}

			{stats && (
				<>
					{isDead && (
						<div className="px-4 py-2 bg-red-950/50 border-b border-red-900/50">
							<p className="text-xs text-red-400 text-center">This creature has died</p>
						</div>
					)}

					<InspectorHeader
						id={stats.id}
						rgb={stats.phenotype.rgb}
						generation={stats.generation}
						age={stats.age}
						complexity={stats.complexity}
						onClose={clearSelection}
					/>

					<div className="flex-1 overflow-y-auto min-h-0">
						<div className="border-t border-slate-800">
							<StatsSection
								energy={stats.energy}
								maxEnergy={stats.maxEnergy}
								position={stats.position}
								age={stats.age}
								generation={stats.generation}
							/>
						</div>

						{/* Phenotype channels */}
						<div className="border-t border-slate-800">
							<PhenotypeDetail phenotype={stats.phenotype} />
						</div>

						{/* Genome mesh graph */}
						{genome && (
							<div className="border-t border-slate-800">
								<NodeGraph genome={genome} />
							</div>
						)}

						{/* Memory hex view */}
						{memory && (
							<div className="border-t border-slate-800">
								<MemoryHexView memory={memory} />
							</div>
						)}
					</div>
				</>
			)}
		</div>
	);
}

function LoadingSkeleton() {
	return (
		<div className="px-4 py-4 space-y-3 animate-pulse">
			<div className="flex gap-3">
				<div className="w-10 h-10 rounded-md bg-slate-800" />
				<div className="flex-1 space-y-2">
					<div className="h-4 w-20 rounded bg-slate-800" />
					<div className="h-3 w-32 rounded bg-slate-800" />
				</div>
			</div>
			<div className="h-2 rounded-full bg-slate-800" />
			<div className="space-y-2">
				<div className="h-3 w-full rounded bg-slate-800" />
				<div className="h-3 w-3/4 rounded bg-slate-800" />
			</div>
		</div>
	);
}

export default CreatureInspector;
