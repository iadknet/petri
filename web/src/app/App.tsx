import { useEffect, useMemo, useState } from "react";

import { ControlHeader } from "../features/simulation/components/ControlHeader";
import { CreatureInspectorPanel } from "../features/simulation/components/CreatureInspectorPanel";
import { LiveMetricsPanel } from "../features/simulation/components/LiveMetricsPanel";
import { AdvancedConfigPanel } from "../features/simulation/components/AdvancedConfigPanel";
import { PopulationEnergyChart } from "../features/simulation/components/PopulationEnergyChart";
import { RuntimeTuningPanel } from "../features/simulation/components/RuntimeTuningPanel";
import { SimulationControls } from "../features/simulation/components/SimulationControls";
import { SnapshotPanel } from "../features/simulation/components/SnapshotPanel";
import { StartupDraftPanel } from "../features/simulation/components/StartupDraftPanel";
import { ViewportCanvas, ViewportControls } from "../features/simulation/components/ViewportCanvas";
import { useSimulationStore } from "../features/simulation/store/simulationStore";

export default function App() {
  const [zoom, setZoom] = useState(3);
  const [selectedCreatureId, setSelectedCreatureId] = useState<number | null>(null);
  const simulation = useSimulationStore();
  const selectedCreature = useMemo(() => {
    if (!simulation.frame || selectedCreatureId === null) {
      return null;
    }
    return simulation.frame.creatures.find((creature) => creature.id === selectedCreatureId) ?? null;
  }, [simulation.frame, selectedCreatureId]);

  useEffect(() => {
    if (!simulation.frame) {
      setSelectedCreatureId(null);
      return;
    }
    if (
      selectedCreatureId !== null &&
      !simulation.frame.creatures.some((creature) => creature.id === selectedCreatureId)
    ) {
      setSelectedCreatureId(null);
    }
  }, [simulation.frame, selectedCreatureId]);

  return (
    <div className="app-root">
      <aside className="panel">
        <ControlHeader
          serverReachable={simulation.serverReachable}
          wsConnected={simulation.wsConnected}
          phase={simulation.phase}
          initializationStage={simulation.status?.initialization_stage}
          viabilityProbeEnabled={simulation.status?.viability_probe_enabled}
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

        <AdvancedConfigPanel
          startupDraft={simulation.startupDraft}
          runtimeConfig={simulation.runtimeConfig}
          phase={simulation.phase}
          onUpdateStartupField={(key, value) => void simulation.updateStartupField(key, value)}
          onUpdateRuntimeField={(patch, optimistic) =>
            void simulation.updateRuntimeField(patch, optimistic)
          }
        />

        <LiveMetricsPanel
          tick={simulation.tick}
          population={simulation.population}
          averageEnergy={simulation.averageEnergy}
        />

        <PopulationEnergyChart
          tick={simulation.tick}
          population={simulation.population}
          averageEnergy={simulation.averageEnergy}
        />

        <CreatureInspectorPanel creature={selectedCreature} />

        <SnapshotPanel
          onExportSnapshot={() => simulation.exportSnapshot()}
          onImportSnapshot={(snapshot) => simulation.importSnapshot(snapshot)}
        />

        <ViewportControls zoom={zoom} onZoomChange={setZoom} />

        {simulation.error ? <p className="error">{simulation.error}</p> : null}
      </aside>

      <ViewportCanvas
        phase={simulation.phase}
        frame={simulation.frame}
        zoom={zoom}
        onSelectCreature={(creature) => setSelectedCreatureId(creature?.id ?? null)}
      />
    </div>
  );
}
