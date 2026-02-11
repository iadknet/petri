type ControlHeaderProps = {
  serverReachable: boolean;
  wsConnected: boolean;
  phase: string;
  initializationStage?: string | null;
  viabilityProbeEnabled?: boolean;
  runId: number | null | undefined;
  pendingRestart: boolean | undefined;
};

export function ControlHeader({
  serverReachable,
  wsConnected,
  phase,
  initializationStage,
  viabilityProbeEnabled,
  runId,
  pendingRestart
}: ControlHeaderProps) {
  return (
    <>
      <h1>Petri Control</h1>
      <div className="status-row">
        <span>Server</span>
        <strong>{serverReachable ? "Reachable" : "Offline"}</strong>
      </div>
      <div className="status-row">
        <span>WebSocket</span>
        <strong>{wsConnected ? "Connected" : "Disconnected"}</strong>
      </div>
      <div className="status-row">
        <span>State</span>
        <strong>{phase}</strong>
      </div>
      <div className="status-row">
        <span>Init Stage</span>
        <strong>{initializationStage ?? "----"}</strong>
      </div>
      <div className="status-row">
        <span>Viability Probe</span>
        <strong>
          {viabilityProbeEnabled === undefined ? "----" : viabilityProbeEnabled ? "On" : "Off"}
        </strong>
      </div>
      <div className="status-row">
        <span>Run ID</span>
        <strong>{runId ?? "----"}</strong>
      </div>
      <div className="status-row">
        <span>Pending Restart</span>
        <strong>{pendingRestart ? "Yes" : "No"}</strong>
      </div>
    </>
  );
}
