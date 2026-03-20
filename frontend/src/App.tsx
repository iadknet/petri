import { Component, type ErrorInfo, type ReactNode, Suspense, lazy, useEffect } from "react";
import { api } from "./api/rest.ts";
import { ConfigPanel } from "./components/ConfigPanel.tsx";
import { ControlBar } from "./components/ControlBar.tsx";
import { StatsPanel } from "./components/StatsPanel.tsx";
import { WorldViewport } from "./components/WorldViewport.tsx";
import { useViewSubscription } from "./hooks/useViewSubscription.ts";
import { useConfigStore } from "./stores/config.ts";
import {
	creatureInspectorSelectors,
	useCreatureInspectorStore,
} from "./stores/creatureInspector.ts";
import { useInspectorWorkspaceStore } from "./stores/inspectorWorkspace.ts";
import { PanelLayoutProvider, usePanelLayout } from "./stores/layout.tsx";
import { useStartupConfigStore } from "./stores/startupConfig.ts";

const LazyCreatureInspector = lazy(() => import("./components/CreatureInspector.tsx"));

function isChunkLoadError(error: unknown): boolean {
	if (!(error instanceof Error)) return false;
	const message = error.message.toLowerCase();
	return (
		message.includes("failed to fetch dynamically imported module") ||
		message.includes("importing a module script failed") ||
		message.includes("loading chunk")
	);
}

function Dashboard() {
	const { configOpen, statsOpen } = usePanelLayout();
	const inspectorOpen = useCreatureInspectorStore(
		(s) => creatureInspectorSelectors.selectedCreatureId(s) !== null,
	);
	const inspectorWidth = useInspectorWorkspaceStore((state) => state.inspectorWidth);

	useViewSubscription();

	// Fetch initial config
	useEffect(() => {
		api
			.getConfig()
			.then((res) => {
				useConfigStore.getState().setServerConfig(res.config, res.state);
				useStartupConfigStore.getState().hydrateFromServerConfig(res.config);
			})
			.catch(() => {
				// Server may not be running yet -- config loads on first successful connection
			});
	}, []);

	const columns = `${configOpen ? "320px " : ""}1fr${inspectorOpen ? ` ${inspectorWidth}px` : ""}`;

	return (
		<div
			className="h-screen w-screen grid overflow-hidden"
			style={{
				gridTemplateRows: "48px 1fr auto",
				gridTemplateColumns: columns,
			}}
		>
			{/* Control bar spans full width */}
			<div style={{ gridColumn: "1 / -1" }}>
				<ControlBar />
			</div>

			{/* Config panel */}
			{configOpen && <ConfigPanel />}

			{/* Main area: viewport + stats */}
			<div className="flex flex-col overflow-hidden min-h-0">
				<div className="flex-1 min-h-0">
					<WorldViewport />
				</div>
				{statsOpen && <StatsPanel />}
			</div>

			{/* Creature inspector */}
			{inspectorOpen && (
				<InspectorErrorBoundary>
					<Suspense fallback={<InspectorFallback />}>
						<LazyCreatureInspector />
					</Suspense>
				</InspectorErrorBoundary>
			)}
		</div>
	);
}

function InspectorFallback() {
	return (
		<div className="h-full bg-slate-900 border-l border-slate-800 animate-pulse">
			<div className="px-4 py-4 space-y-3">
				<div className="flex gap-3">
					<div className="w-10 h-10 rounded-md bg-slate-800" />
					<div className="flex-1 space-y-2">
						<div className="h-4 w-20 rounded bg-slate-800" />
						<div className="h-3 w-32 rounded bg-slate-800" />
					</div>
				</div>
			</div>
		</div>
	);
}

class InspectorErrorBoundary extends Component<
	{ children: ReactNode },
	{ hasError: boolean; isChunkLoadError: boolean }
> {
	state = { hasError: false, isChunkLoadError: false };

	static getDerivedStateFromError(): { hasError: boolean } {
		return { hasError: true };
	}

	componentDidCatch(error: Error, info: ErrorInfo): void {
		this.setState({ isChunkLoadError: isChunkLoadError(error) });
		console.error("[InspectorErrorBoundary]", error, info);
	}

	render() {
		if (this.state.hasError) {
			const retry = () => {
				if (this.state.isChunkLoadError) {
					window.location.reload();
					return;
				}
				this.setState({ hasError: false, isChunkLoadError: false });
			};
			return (
				<div className="h-full bg-slate-900 border-l border-slate-800 flex items-center justify-center">
					<div className="text-center px-4">
						<p className="text-sm text-slate-400">
							{this.state.isChunkLoadError
								? "Inspector is out of date after a frontend rebuild"
								: "Failed to load inspector"}
						</p>
						<button
							type="button"
							onClick={retry}
							className="mt-2 text-xs text-emerald-400 hover:text-emerald-300"
						>
							{this.state.isChunkLoadError ? "Refresh app" : "Retry"}
						</button>
					</div>
				</div>
			);
		}
		return this.props.children;
	}
}

export function App() {
	return (
		<PanelLayoutProvider>
			<Dashboard />
		</PanelLayoutProvider>
	);
}
