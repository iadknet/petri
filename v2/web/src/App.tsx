import { AppShell } from "./app/AppShell";
import { RunHealthPanel } from "./features/health/RunHealthPanel";
import { CreatureInspector } from "./features/inspector/CreatureInspector";
import { PaintToolbar } from "./features/paint/PaintToolbar";
import { StartupPanel } from "./features/startup/StartupPanel";
import { RuntimeControls } from "./features/runtime/RuntimeControls";
import { ViewportCanvas } from "./features/viewport/ViewportCanvas";
import { useSimulationStore } from "./features/simulation/store/simulationStore";

export default function App() {
  const simulation = useSimulationStore();

  async function handleClearAll(): Promise<void> {
    const confirmed = window.confirm("Clear all food and barriers from the world?");
    if (!confirmed) {
      return;
    }
    await simulation.clearPaint();
  }

  return (
    <AppShell
      statusLine={
        <span>
          State <strong>{simulation.currentState}</strong> | Tick <strong>{simulation.tickLabel}</strong> |
          Connection <strong>{simulation.connectionState}</strong>
        </span>
      }
      protocolBanner={
        <span>
          {simulation.protocolVersion}
          {simulation.protocolMismatch ? " (version mismatch)" : " (compatible)"}
          {simulation.errorMessage ? ` | ${simulation.errorMessage}` : ""}
        </span>
      }
      startupPanel={
        <StartupPanel
          draft={simulation.startupDraft}
          onChange={simulation.setStartupDraft}
          onApply={simulation.applyStartup}
          disabled={simulation.busy}
        />
      }
      runtimeControls={
        <RuntimeControls
          state={simulation.currentState}
          tick={simulation.tickLabel}
          ticksPerSecond={simulation.startupDraft.runtime.ticks_per_second}
          onTicksPerSecondChange={simulation.setTicksPerSecond}
          stepCount={simulation.stepCount}
          onStepCountChange={simulation.setStepCount}
          onStart={simulation.start}
          onPause={simulation.pause}
          onStep={simulation.step}
          onRefresh={simulation.refresh}
          disabled={simulation.busy}
        />
      }
      paintToolbar={
        <PaintToolbar
          tool={simulation.paintTool}
          brushHalfExtent={simulation.brushHalfExtent}
          editable={simulation.editable}
          busy={simulation.busy}
          paintLastTouchedCells={simulation.paintLastTouchedCells}
          errorMessage={simulation.paintError}
          onToolChange={simulation.setPaintTool}
          onBrushHalfExtentChange={simulation.setBrushHalfExtent}
          onClearAll={handleClearAll}
        />
      }
      viewport={
        <ViewportCanvas
          frame={simulation.frame}
          selectedCreatureId={simulation.selectedCreatureId}
          paintTool={simulation.paintTool}
          brushHalfExtent={simulation.brushHalfExtent}
          paintEnabled={simulation.editable}
          busy={simulation.busy}
          onSelectCreature={simulation.setSelectedCreatureId}
          onStrokeCommit={simulation.commitPaint}
        />
      }
      inspector={<CreatureInspector creature={simulation.selectedCreature} />}
      runHealth={<RunHealthPanel status={simulation.status} />}
    />
  );
}
