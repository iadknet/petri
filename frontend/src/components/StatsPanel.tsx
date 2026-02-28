import { useCallback, useState } from "react";
import { ActionsTab } from "./stats/ActionsTab.tsx";
import { ComputationTab } from "./stats/ComputationTab.tsx";
import { EvolutionTab } from "./stats/EvolutionTab.tsx";
import { OverviewTab } from "./stats/OverviewTab.tsx";
import { PredationTab } from "./stats/PredationTab.tsx";

type Tab = "overview" | "actions" | "evolution" | "computation" | "predation";

export function StatsPanel() {
	const [tab, setTab] = useState<Tab>("overview");

	const tabButton = useCallback(
		(t: Tab, label: string, testId: string) => (
			<button
				type="button"
				data-testid={testId}
				onClick={() => setTab(t)}
				className={`px-3 py-1 text-xs font-medium rounded-t ${
					tab === t ? "bg-petri-panel text-slate-200" : "text-slate-500 hover:text-slate-300"
				}`}
			>
				{label}
			</button>
		),
		[tab],
	);

	return (
		<div
			data-testid="stats-panel"
			className="bg-petri-panel border-t border-petri-border flex flex-col"
			style={{ height: "280px" }}
		>
			<div className="flex gap-1 px-3 pt-1 bg-slate-950">
				{tabButton("overview", "Overview", "stats-tab-overview")}
				{tabButton("actions", "Actions", "stats-tab-actions")}
				{tabButton("evolution", "Evolution", "stats-tab-evolution")}
				{tabButton("computation", "Computation", "stats-tab-computation")}
				{tabButton("predation", "Predation", "stats-tab-predation")}
			</div>
			<div className="flex-1 overflow-y-auto">
				{tab === "overview" && <OverviewTab />}
				{tab === "actions" && <ActionsTab />}
				{tab === "evolution" && <EvolutionTab />}
				{tab === "computation" && <ComputationTab />}
				{tab === "predation" && <PredationTab />}
			</div>
		</div>
	);
}
