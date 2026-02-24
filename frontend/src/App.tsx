import { useEffect } from "react";
import { api } from "./api/rest.ts";
import { wsClient } from "./api/websocket.ts";
import { ConfigPanel } from "./components/ConfigPanel.tsx";
import { ControlBar } from "./components/ControlBar.tsx";
import { StatsPanel } from "./components/StatsPanel.tsx";
import { WorldViewport } from "./components/WorldViewport.tsx";
import { useConfigStore } from "./stores/config.ts";
import { PanelLayoutProvider, usePanelLayout } from "./stores/layout.tsx";
import { useStartupConfigStore } from "./stores/startupConfig.ts";

function Dashboard() {
	const { configOpen, statsOpen } = usePanelLayout();

	// Connect WebSocket on mount
	useEffect(() => {
		wsClient.connect();
		return () => wsClient.disconnect();
	}, []);

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

	return (
		<div
			className="h-screen w-screen grid overflow-hidden"
			style={{
				gridTemplateRows: "48px 1fr auto",
				gridTemplateColumns: configOpen ? "320px 1fr" : "1fr",
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
		</div>
	);
}

export function App() {
	return (
		<PanelLayoutProvider>
			<Dashboard />
		</PanelLayoutProvider>
	);
}
