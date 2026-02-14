import type { StatusPayload } from "../protocol/models";

interface RunHealthPanelProps {
  status: StatusPayload | null;
}

export function RunHealthPanel(props: RunHealthPanelProps) {
  if (!props.status) {
    return <p className="muted">No run-health snapshot yet.</p>;
  }

  const status = props.status;
  return (
    <div className="control-stack metric-list">
      <p className="metric-row">
        <span>Population</span>
        <strong>{status.population}</strong>
      </p>
      <p className="metric-row">
        <span>Mean Energy</span>
        <strong>{status.mean_energy.toFixed(2)}</strong>
      </p>
      <p className="metric-row">
        <span>Births/Deaths Window</span>
        <strong>
          {status.births_last_window}/{status.deaths_last_window}
        </strong>
      </p>
      <p className="metric-row metric-row-stack">
        <span>Action Counts</span>
        <strong>
          m{status.last_action_counts.move} e{status.last_action_counts.eat} r
          {status.last_action_counts.reproduce} ip{status.last_action_counts.inventory_pickup} iu
          {status.last_action_counts.inventory_put} n{status.last_action_counts.noop}
        </strong>
      </p>
    </div>
  );
}
