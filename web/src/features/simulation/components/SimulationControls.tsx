import { RuntimeConfig } from "../api/simulationApiClient";

type SimulationControlsProps = {
  phase: string;
  runId: number | null | undefined;
  busyAction: "start" | "restart" | null;
  startDisabled: boolean;
  runtimeConfig: RuntimeConfig | null;
  startupViable: boolean | undefined;
  startupViabilityMessage: string | null | undefined;
  onStart: () => void;
  onRestart: () => void;
  onTogglePause: () => void;
};

export function SimulationControls({
  phase,
  runId,
  busyAction,
  startDisabled,
  runtimeConfig,
  startupViable,
  startupViabilityMessage,
  onStart,
  onRestart,
  onTogglePause
}: SimulationControlsProps) {
  return (
    <section className="section">
      <h2>Simulation Controls</h2>
      <div className="button-row">
        <button className="primary-btn" onClick={onStart} disabled={startDisabled}>
          {busyAction === "start" ? "Starting..." : "Start Simulation"}
        </button>
        <button
          className="primary-btn"
          onClick={onRestart}
          disabled={runId == null || busyAction !== null}
        >
          {busyAction === "restart" ? "Restarting..." : "Restart Simulation"}
        </button>
        <button
          className="primary-btn"
          onClick={onTogglePause}
          disabled={phase === "idle" || !runtimeConfig}
        >
          {runtimeConfig?.paused ? "Resume" : "Pause"}
        </button>
      </div>
      {phase === "idle" && startupViable === false ? (
        <p className="error">
          Startup draft is non-viable. {startupViabilityMessage ?? "Adjust controls."}
        </p>
      ) : null}
    </section>
  );
}
