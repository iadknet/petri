import { type KeyboardEvent, useCallback, useState } from "react";
import { ActionsTab } from "./stats/ActionsTab.tsx";
import { ComputationTab } from "./stats/ComputationTab.tsx";
import { EvolutionTab } from "./stats/EvolutionTab.tsx";
import { OverviewTab } from "./stats/OverviewTab.tsx";
import { PredationTab } from "./stats/PredationTab.tsx";

type Tab = "overview" | "actions" | "evolution" | "computation" | "predation";

const TABS: { id: Tab; label: string }[] = [
	{ id: "overview", label: "Overview" },
	{ id: "actions", label: "Actions" },
	{ id: "evolution", label: "Evolution" },
	{ id: "computation", label: "Computation" },
	{ id: "predation", label: "Predation" },
];

export function StatsPanel() {
	const [tab, setTab] = useState<Tab>("overview");

	const selectAndFocus = useCallback((next: Tab) => {
		setTab(next);
		document.getElementById(`stats-tab-${next}`)?.focus();
	}, []);

	const handleKeyDown = useCallback(
		(event: KeyboardEvent<HTMLButtonElement>, index: number) => {
			const last = TABS.length - 1;
			let nextIndex: number;
			if (event.key === "ArrowRight") nextIndex = index === last ? 0 : index + 1;
			else if (event.key === "ArrowLeft") nextIndex = index === 0 ? last : index - 1;
			else if (event.key === "Home") nextIndex = 0;
			else if (event.key === "End") nextIndex = last;
			else return;
			const next = TABS[nextIndex];
			if (!next) return;
			event.preventDefault();
			selectAndFocus(next.id);
		},
		[selectAndFocus],
	);

	return (
		<section
			data-testid="stats-panel"
			aria-labelledby="stats-panel-heading"
			className="bg-petri-panel border-t border-petri-border flex flex-col"
			style={{ height: "280px" }}
		>
			<h2 id="stats-panel-heading" className="sr-only">
				Statistics
			</h2>
			<div
				role="tablist"
				aria-label="Statistics views"
				className="flex gap-1 px-3 pt-1 bg-slate-950"
			>
				{TABS.map(({ id, label }, index) => {
					const selected = tab === id;
					return (
						<button
							key={id}
							type="button"
							role="tab"
							id={`stats-tab-${id}`}
							data-testid={`stats-tab-${id}`}
							aria-selected={selected}
							aria-controls={`stats-tabpanel-${id}`}
							tabIndex={selected ? 0 : -1}
							onClick={() => setTab(id)}
							onKeyDown={(event) => handleKeyDown(event, index)}
							className={`px-3 py-1 text-xs font-medium rounded-t border-b-2 focus-visible:outline-2 focus-visible:outline-sky-400 ${
								selected
									? "bg-petri-panel text-slate-200 border-sky-400"
									: "text-slate-500 hover:text-slate-300 border-transparent"
							}`}
						>
							{label}
						</button>
					);
				})}
			</div>
			{TABS.map(({ id }) => (
				<div
					key={id}
					role="tabpanel"
					id={`stats-tabpanel-${id}`}
					aria-labelledby={`stats-tab-${id}`}
					hidden={tab !== id}
					className="flex-1 overflow-y-auto"
				>
					{tab === id && id === "overview" && <OverviewTab />}
					{tab === id && id === "actions" && <ActionsTab />}
					{tab === id && id === "evolution" && <EvolutionTab />}
					{tab === id && id === "computation" && <ComputationTab />}
					{tab === id && id === "predation" && <PredationTab />}
				</div>
			))}
		</section>
	);
}
