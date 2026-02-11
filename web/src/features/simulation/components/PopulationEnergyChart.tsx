import { useEffect, useMemo, useState } from "react";

const MAX_SAMPLES = 120;
const CHART_WIDTH = 280;
const CHART_HEIGHT = 120;

type MetricSample = {
  tick: number;
  population: number;
  averageEnergy: number;
};

type PopulationEnergyChartProps = {
  tick: number;
  population: number;
  averageEnergy: number;
};

function toPolylinePoints(values: number[], maxValue: number): string {
  if (values.length === 0) {
    return "";
  }

  const denominator = Math.max(1, values.length - 1);
  return values
    .map((value, index) => {
      const x = (index / denominator) * CHART_WIDTH;
      const y = CHART_HEIGHT - (value / maxValue) * CHART_HEIGHT;
      return `${x.toFixed(2)},${y.toFixed(2)}`;
    })
    .join(" ");
}

export function PopulationEnergyChart({ tick, population, averageEnergy }: PopulationEnergyChartProps) {
  const [samples, setSamples] = useState<MetricSample[]>([]);

  useEffect(() => {
    if (tick <= 0) {
      return;
    }

    setSamples((prev) => {
      if (prev.length > 0 && prev[prev.length - 1].tick === tick) {
        return prev;
      }

      const next = [...prev, { tick, population, averageEnergy }];
      return next.slice(-MAX_SAMPLES);
    });
  }, [tick, population, averageEnergy]);

  const populationSeries = useMemo(() => samples.map((sample) => sample.population), [samples]);
  const energySeries = useMemo(() => samples.map((sample) => sample.averageEnergy), [samples]);
  const populationMax = Math.max(1, ...populationSeries);
  const energyMax = Math.max(1, ...energySeries);

  return (
    <section className="section">
      <h2>Population / Energy</h2>
      <p className="muted">Samples: {samples.length}</p>
      <svg viewBox={`0 0 ${CHART_WIDTH} ${CHART_HEIGHT}`} width="100%" height="120" aria-label="Population and energy chart">
        <polyline
          data-testid="population-series"
          fill="none"
          stroke="#2fd07f"
          strokeWidth="2"
          points={toPolylinePoints(populationSeries, populationMax)}
        />
        <polyline
          data-testid="energy-series"
          fill="none"
          stroke="#4cb3ff"
          strokeWidth="2"
          points={toPolylinePoints(energySeries, energyMax)}
        />
      </svg>
    </section>
  );
}
