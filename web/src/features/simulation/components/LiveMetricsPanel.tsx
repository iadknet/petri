type LiveMetricsPanelProps = {
  tick: number;
  population: number;
  averageEnergy: number;
};

function formatEnergy(value: number): string {
  return Number.isFinite(value) ? value.toFixed(4) : "0.0000";
}

export function LiveMetricsPanel({ tick, population, averageEnergy }: LiveMetricsPanelProps) {
  return (
    <section className="section">
      <h2>Live Metrics</h2>
      <div className="metric-row">
        <span>Tick</span>
        <strong>{tick}</strong>
      </div>
      <div className="metric-row">
        <span>Population</span>
        <strong>{population}</strong>
      </div>
      <div className="metric-row">
        <span>Avg Energy</span>
        <strong>{formatEnergy(averageEnergy)}</strong>
      </div>
    </section>
  );
}
