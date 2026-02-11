import { ConfigPatch } from "../../../protocol";
import { RuntimeConfig } from "../api/simulationApiClient";
import { STARTUP_LIMITS } from "../store/simulationStore";

type RuntimeTuningPanelProps = {
  runtimeConfig: RuntimeConfig | null;
  phase: string;
  onUpdateRuntimeField: (patch: ConfigPatch, optimistic: RuntimeConfig) => void;
};

export function RuntimeTuningPanel({
  runtimeConfig,
  phase,
  onUpdateRuntimeField
}: RuntimeTuningPanelProps) {
  return (
    <section className="section">
      <h2>Runtime Tuning</h2>
      {runtimeConfig ? (
        <>
          <label className="slider-label" htmlFor="tps-slider">
            Ticks / second: {runtimeConfig.ticks_per_second}
          </label>
          <input
            id="tps-slider"
            type="range"
            min={1}
            max={120}
            step={1}
            value={runtimeConfig.ticks_per_second}
            disabled={phase === "idle"}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { ticks_per_second: value },
                { ...runtimeConfig, ticks_per_second: value }
              );
            }}
          />

          <label className="slider-label" htmlFor="runtime-food-spawn-rate-slider">
            Runtime food spawn: {runtimeConfig.food_spawn_rate.toFixed(2)}
          </label>
          <input
            id="runtime-food-spawn-rate-slider"
            type="range"
            min={STARTUP_LIMITS.food_spawn_rate.min}
            max={STARTUP_LIMITS.food_spawn_rate.max}
            step={STARTUP_LIMITS.food_spawn_rate.step}
            value={runtimeConfig.food_spawn_rate}
            disabled={phase === "idle"}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { food_spawn_rate: value },
                { ...runtimeConfig, food_spawn_rate: value }
              );
            }}
          />

          <label className="slider-label" htmlFor="runtime-food-growth-rate-slider">
            Runtime food growth: {runtimeConfig.food_growth_rate.toFixed(2)}
          </label>
          <input
            id="runtime-food-growth-rate-slider"
            type="range"
            min={STARTUP_LIMITS.food_growth_rate.min}
            max={STARTUP_LIMITS.food_growth_rate.max}
            step={STARTUP_LIMITS.food_growth_rate.step}
            value={runtimeConfig.food_growth_rate}
            disabled={phase === "idle"}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { food_growth_rate: value },
                { ...runtimeConfig, food_growth_rate: value }
              );
            }}
          />
        </>
      ) : (
        <p className="muted">Loading runtime config...</p>
      )}
    </section>
  );
}
