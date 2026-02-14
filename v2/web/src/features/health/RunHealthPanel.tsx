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
    <div className="control-stack">
      <p>Population: {status.population}</p>
      <p>Mean Energy: {status.mean_energy.toFixed(2)}</p>
      <p>
        Births/Deaths Window: {status.births_last_window}/{status.deaths_last_window}
      </p>
      <p>
        Action Counts: m{status.last_action_counts.move} e{status.last_action_counts.eat} r
        {status.last_action_counts.reproduce} ip{status.last_action_counts.inventory_pickup} iu
        {status.last_action_counts.inventory_put} n{status.last_action_counts.noop}
      </p>
    </div>
  );
}
