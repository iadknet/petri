import { useState } from "react";
import type { CreatureDiagnostics } from "../../types/creature-detail.ts";

interface InspectorDiagnosticsCardsProps {
	diagnostics: CreatureDiagnostics | null;
}

function activeNeighborCount(values: number[]): number {
	return values.reduce((count, value) => (value > 0 ? count + 1 : count), 0);
}

function formatSlotList(slots: number[]): string {
	if (slots.length === 0) {
		return "none";
	}
	return slots.join(", ");
}

function sortedEntries(counts: Record<string, number>): Array<[string, number]> {
	return Object.entries(counts).sort((a, b) => b[1] - a[1]);
}

export function InspectorDiagnosticsCards({ diagnostics }: InspectorDiagnosticsCardsProps) {
	const [expanded, setExpanded] = useState(false);

	const toggle = (
		<button
			type="button"
			onClick={() => setExpanded((prev) => !prev)}
			aria-label="Diagnostics"
			aria-expanded={expanded}
			className="flex w-full items-center gap-1 text-[11px] uppercase tracking-wide text-slate-400 transition-colors hover:text-slate-300"
			data-testid="inspector-diagnostics-toggle"
		>
			<span className="text-[10px]">{expanded ? "\u25BC" : "\u25B6"}</span>
			<span>Diagnostics</span>
		</button>
	);

	if (!diagnostics) {
		return (
			<section
				className="shrink-0 border-b border-slate-800 bg-slate-900/40 px-3 py-2"
				data-testid="inspector-diagnostics-empty"
			>
				{toggle}
				{expanded && (
					<div className="mt-1 text-[11px] text-slate-500">
						Diagnostics unavailable for this creature response.
					</div>
				)}
			</section>
		);
	}

	const barrierNeighbors = activeNeighborCount(diagnostics.current_inputs.neighbor_barrier);
	const occupiedNeighbors = activeNeighborCount(diagnostics.current_inputs.neighbor_occupied);
	const readClassEntries = sortedEntries(diagnostics.live_circuit.reachable_read_class_counts);
	const writeClassEntries = sortedEntries(diagnostics.live_circuit.reachable_write_class_counts);
	const recentActionEntries = sortedEntries(diagnostics.recent_actions.by_action_result).slice(
		0,
		4,
	);

	return (
		<section
			className="shrink-0 border-b border-slate-800 bg-slate-900/40 px-3 py-2"
			data-testid="inspector-diagnostics"
		>
			{toggle}
			{expanded && (
				<>
					<div className="mt-2 grid grid-cols-2 gap-2 text-[11px]">
						<div className="rounded border border-slate-800 bg-slate-950/60 p-2">
							<div className="font-medium text-slate-300">Barrier Perception</div>
							<div className="mt-1 flex justify-between text-slate-400">
								<span>Food Here</span>
								<span className="font-mono text-slate-200">
									{diagnostics.current_inputs.food_here.toFixed(3)}
								</span>
							</div>
							<div className="flex justify-between text-slate-400">
								<span>Barrier Neighbors</span>
								<span className="font-mono text-slate-200">{barrierNeighbors}/8</span>
							</div>
							<div className="flex justify-between text-slate-400">
								<span>Occupied Neighbors</span>
								<span className="font-mono text-slate-200">{occupiedNeighbors}/8</span>
							</div>
							<div className="mt-2 border-t border-slate-800 pt-1 text-slate-400">
								<div className="flex justify-between">
									<span>Blocked Move (recent)</span>
									<span className="font-mono text-slate-200">
										{diagnostics.recent_actions.blocked_move_count}
									</span>
								</div>
								<div className="flex justify-between">
									<span>Invalid Reproduce (recent)</span>
									<span className="font-mono text-slate-200">
										{diagnostics.recent_actions.invalid_target_reproduce_count}
									</span>
								</div>
							</div>
						</div>

						<div className="rounded border border-slate-800 bg-slate-950/60 p-2">
							<div className="font-medium text-slate-300">Live Circuit Structure</div>
							<div className="mt-1 flex justify-between text-slate-400">
								<span>Reachable Nodes</span>
								<span className="font-mono text-slate-200">
									{diagnostics.live_circuit.reachable_node_count}
								</span>
							</div>
							<div className="flex justify-between text-slate-400">
								<span>Stateful Reachable</span>
								<span className="font-mono text-slate-200">
									{diagnostics.live_circuit.stateful_reachable_node_count}
								</span>
							</div>
							<div className="flex justify-between text-slate-400">
								<span>Barrier Readers</span>
								<span className="font-mono text-slate-200">
									{diagnostics.live_circuit.barrier_reader_reachable_node_count}
								</span>
							</div>
							<div className="mt-2 border-t border-slate-800 pt-1 text-slate-400">
								<div className="flex justify-between">
									<span>Upstream Slots</span>
									<span className="font-mono text-slate-200">
										{formatSlotList(diagnostics.live_circuit.distinct_upstream_slots_read)}
									</span>
								</div>
								<div className="flex justify-between">
									<span>Payload Slots</span>
									<span className="font-mono text-slate-200">
										{formatSlotList(diagnostics.live_circuit.distinct_payload_slots_written)}
									</span>
								</div>
								<div className="flex justify-between">
									<span>Custom Outputs</span>
									<span className="font-mono text-slate-200">
										{formatSlotList(diagnostics.live_circuit.distinct_custom_output_slots_written)}
									</span>
								</div>
							</div>
						</div>
					</div>

					<div className="mt-2 grid grid-cols-3 gap-2 text-[11px] text-slate-400">
						<div className="rounded border border-slate-800 bg-slate-950/60 p-2">
							<div className="font-medium text-slate-300">Reachable Read Classes</div>
							{readClassEntries.length === 0 && <div className="mt-1 text-slate-500">none</div>}
							{readClassEntries.map(([kind, count]) => (
								<div key={kind} className="mt-1 flex justify-between">
									<span>{kind}</span>
									<span className="font-mono text-slate-200">{count}</span>
								</div>
							))}
						</div>
						<div className="rounded border border-slate-800 bg-slate-950/60 p-2">
							<div className="font-medium text-slate-300">Reachable Write Classes</div>
							{writeClassEntries.length === 0 && <div className="mt-1 text-slate-500">none</div>}
							{writeClassEntries.map(([kind, count]) => (
								<div key={kind} className="mt-1 flex justify-between">
									<span>{kind}</span>
									<span className="font-mono text-slate-200">{count}</span>
								</div>
							))}
						</div>
						<div className="rounded border border-slate-800 bg-slate-950/60 p-2">
							<div className="font-medium text-slate-300">Recent Outcomes</div>
							{recentActionEntries.length === 0 && <div className="mt-1 text-slate-500">none</div>}
							{recentActionEntries.map(([kind, count]) => (
								<div key={kind} className="mt-1 flex justify-between">
									<span>{kind}</span>
									<span className="font-mono text-slate-200">{count}</span>
								</div>
							))}
						</div>
					</div>
				</>
			)}
		</section>
	);
}
