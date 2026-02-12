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
            max={360}
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

          <label className="slider-label" htmlFor="runtime-sensor-radius-slider">
            Runtime sensor radius: {runtimeConfig.sensor_radius}
          </label>
          <input
            id="runtime-sensor-radius-slider"
            type="range"
            min={STARTUP_LIMITS.sensor_radius.min}
            max={STARTUP_LIMITS.sensor_radius.max}
            step={STARTUP_LIMITS.sensor_radius.step}
            value={runtimeConfig.sensor_radius}
            disabled={phase === "idle"}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { sensor_radius: value },
                { ...runtimeConfig, sensor_radius: value }
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

          <label className="slider-label" htmlFor="runtime-food-spread-threshold-slider">
            Runtime food spread threshold: {runtimeConfig.food_spread_threshold.toFixed(2)}
          </label>
          <input
            id="runtime-food-spread-threshold-slider"
            type="range"
            min={STARTUP_LIMITS.food_spread_threshold.min}
            max={STARTUP_LIMITS.food_spread_threshold.max}
            step={STARTUP_LIMITS.food_spread_threshold.step}
            value={runtimeConfig.food_spread_threshold}
            disabled={phase === "idle"}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { food_spread_threshold: value },
                { ...runtimeConfig, food_spread_threshold: value }
              );
            }}
          />

          <label className="slider-label" htmlFor="runtime-food-spawn-floor-density-slider">
            Runtime food spawn floor: {runtimeConfig.food_spawn_floor_density.toFixed(2)}
          </label>
          <input
            id="runtime-food-spawn-floor-density-slider"
            type="range"
            min={STARTUP_LIMITS.food_spawn_floor_density.min}
            max={STARTUP_LIMITS.food_spawn_floor_density.max}
            step={STARTUP_LIMITS.food_spawn_floor_density.step}
            value={runtimeConfig.food_spawn_floor_density}
            disabled={phase === "idle"}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { food_spawn_floor_density: value },
                { ...runtimeConfig, food_spawn_floor_density: value }
              );
            }}
          />

          <label className="slider-label" htmlFor="runtime-weight-mutation-rate-slider">
            Weight mutation rate: {runtimeConfig.weight_mutation_rate.toFixed(2)}
          </label>
          <input
            id="runtime-weight-mutation-rate-slider"
            type="range"
            min={STARTUP_LIMITS.weight_mutation_rate.min}
            max={STARTUP_LIMITS.weight_mutation_rate.max}
            step={STARTUP_LIMITS.weight_mutation_rate.step}
            value={runtimeConfig.weight_mutation_rate}
            disabled={phase === "idle"}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { weight_mutation_rate: value },
                { ...runtimeConfig, weight_mutation_rate: value }
              );
            }}
          />

          <label className="slider-label" htmlFor="runtime-weight-mutation-magnitude-slider">
            Weight mutation magnitude: {runtimeConfig.weight_mutation_magnitude.toFixed(2)}
          </label>
          <input
            id="runtime-weight-mutation-magnitude-slider"
            type="range"
            min={STARTUP_LIMITS.weight_mutation_magnitude.min}
            max={STARTUP_LIMITS.weight_mutation_magnitude.max}
            step={STARTUP_LIMITS.weight_mutation_magnitude.step}
            value={runtimeConfig.weight_mutation_magnitude}
            disabled={phase === "idle"}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { weight_mutation_magnitude: value },
                { ...runtimeConfig, weight_mutation_magnitude: value }
              );
            }}
          />

          <label className="slider-label" htmlFor="runtime-logic-node-mutation-rate-slider">
            Logic node mutation rate: {runtimeConfig.logic_node_mutation_rate.toFixed(2)}
          </label>
          <input
            id="runtime-logic-node-mutation-rate-slider"
            type="range"
            min={STARTUP_LIMITS.logic_node_mutation_rate.min}
            max={STARTUP_LIMITS.logic_node_mutation_rate.max}
            step={STARTUP_LIMITS.logic_node_mutation_rate.step}
            value={runtimeConfig.logic_node_mutation_rate}
            disabled={phase === "idle"}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { logic_node_mutation_rate: value },
                { ...runtimeConfig, logic_node_mutation_rate: value }
              );
            }}
          />

          <label className="slider-label" htmlFor="runtime-structural-mutation-rate-slider">
            Structural mutation rate: {runtimeConfig.structural_mutation_rate.toFixed(2)}
          </label>
          <input
            id="runtime-structural-mutation-rate-slider"
            type="range"
            min={STARTUP_LIMITS.structural_mutation_rate.min}
            max={STARTUP_LIMITS.structural_mutation_rate.max}
            step={STARTUP_LIMITS.structural_mutation_rate.step}
            value={runtimeConfig.structural_mutation_rate}
            disabled={phase === "idle"}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { structural_mutation_rate: value },
                { ...runtimeConfig, structural_mutation_rate: value }
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
