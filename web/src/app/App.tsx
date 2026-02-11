import { useState } from "react";

import { ControlHeader } from "../features/simulation/components/ControlHeader";
import { LiveMetricsPanel } from "../features/simulation/components/LiveMetricsPanel";
import { RuntimeTuningPanel } from "../features/simulation/components/RuntimeTuningPanel";
import { SimulationControls } from "../features/simulation/components/SimulationControls";
import { StartupDraftPanel } from "../features/simulation/components/StartupDraftPanel";
import { ViewportCanvas, ViewportControls } from "../features/simulation/components/ViewportCanvas";
import { useSimulationStore } from "../features/simulation/store/simulationStore";

export default function App() {
  const [zoom, setZoom] = useState(3);
  const simulation = useSimulationStore();

  return (
    <div className="app-root">
      <aside className="panel">
        <ControlHeader
          serverReachable={simulation.serverReachable}
          wsConnected={simulation.wsConnected}
          phase={simulation.phase}
          runId={simulation.status?.run_id}
          pendingRestart={simulation.status?.pending_restart}
        />

        <SimulationControls
          phase={simulation.phase}
          runId={simulation.status?.run_id}
          busyAction={simulation.busyAction}
          startDisabled={simulation.startDisabled}
          runtimeConfig={simulation.runtimeConfig}
          startupViable={simulation.status?.startup_viable}
          startupViabilityMessage={simulation.status?.startup_viability_message}
          onStart={() => void simulation.startSimulation()}
          onRestart={() => void simulation.restartSimulation()}
          onTogglePause={() => void simulation.togglePause()}
        />

        <StartupDraftPanel
          startupDraft={simulation.startupDraft}
          phase={simulation.phase}
          onUpdate={(key, value) => void simulation.updateStartupField(key, value)}
        />

        <RuntimeTuningPanel
          runtimeConfig={simulation.runtimeConfig}
          phase={simulation.phase}
          onUpdateRuntimeField={(patch, optimistic) =>
            void simulation.updateRuntimeField(patch, optimistic)
          }
        />

        <LiveMetricsPanel
          tick={simulation.tick}
          population={simulation.population}
          averageEnergy={simulation.averageEnergy}
        />

        <ViewportControls zoom={zoom} onZoomChange={setZoom} />

        {simulation.error ? <p className="error">{simulation.error}</p> : null}
      </aside>

      <ViewportCanvas phase={simulation.phase} frame={simulation.frame} zoom={zoom} />
    </div>
  );
}
