import { ConfigPatch, StartupDraft } from "../../../protocol";
import { RuntimeConfig } from "../api/simulationApiClient";
import { STARTUP_LIMITS } from "../store/simulationStore";

type AdvancedConfigPanelProps = {
  startupDraft: StartupDraft | null;
  runtimeConfig: RuntimeConfig | null;
  phase: string;
  onUpdateStartupField: <K extends keyof StartupDraft>(key: K, value: StartupDraft[K]) => void;
  onUpdateRuntimeField: (patch: ConfigPatch, optimistic: RuntimeConfig) => void;
};

export function AdvancedConfigPanel({
  startupDraft,
  runtimeConfig,
  phase,
  onUpdateStartupField,
  onUpdateRuntimeField
}: AdvancedConfigPanelProps) {
  const runtimeDisabled = phase === "idle";

  return (
    <section className="section">
      <h2>Advanced Config</h2>
      {startupDraft && runtimeConfig ? (
        <>
          <label className="slider-label" htmlFor="startup-max-creatures-slider">
            Max creatures: {startupDraft.max_creatures}
          </label>
          <input
            id="startup-max-creatures-slider"
            type="range"
            min={STARTUP_LIMITS.max_creatures.min}
            max={STARTUP_LIMITS.max_creatures.max}
            step={STARTUP_LIMITS.max_creatures.step}
            value={startupDraft.max_creatures}
            onChange={(event) => onUpdateStartupField("max_creatures", Number(event.target.value))}
          />

          <label className="slider-label" htmlFor="startup-energy-initial-slider">
            Founder energy: {startupDraft.energy_initial.toFixed(2)}
          </label>
          <input
            id="startup-energy-initial-slider"
            type="range"
            min={STARTUP_LIMITS.energy_initial.min}
            max={STARTUP_LIMITS.energy_initial.max}
            step={STARTUP_LIMITS.energy_initial.step}
            value={startupDraft.energy_initial}
            onChange={(event) => onUpdateStartupField("energy_initial", Number(event.target.value))}
          />

          <label className="slider-label" htmlFor="runtime-food-max-density-slider">
            Food max density: {runtimeConfig.food_max_density.toFixed(2)}
          </label>
          <input
            id="runtime-food-max-density-slider"
            type="range"
            min={STARTUP_LIMITS.food_max_density.min}
            max={STARTUP_LIMITS.food_max_density.max}
            step={STARTUP_LIMITS.food_max_density.step}
            value={runtimeConfig.food_max_density}
            disabled={runtimeDisabled}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { food_max_density: value },
                { ...runtimeConfig, food_max_density: value }
              );
            }}
          />

          <label className="slider-label" htmlFor="runtime-food-energy-value-slider">
            Food energy value: {runtimeConfig.food_energy_value.toFixed(2)}
          </label>
          <input
            id="runtime-food-energy-value-slider"
            type="range"
            min={STARTUP_LIMITS.food_energy_value.min}
            max={STARTUP_LIMITS.food_energy_value.max}
            step={STARTUP_LIMITS.food_energy_value.step}
            value={runtimeConfig.food_energy_value}
            disabled={runtimeDisabled}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { food_energy_value: value },
                { ...runtimeConfig, food_energy_value: value }
              );
            }}
          />

          <label className="slider-label" htmlFor="runtime-energy-tick-decay-slider">
            Runtime tick decay: {runtimeConfig.energy_per_tick_decay.toFixed(3)}
          </label>
          <input
            id="runtime-energy-tick-decay-slider"
            type="range"
            min={STARTUP_LIMITS.energy_per_tick_decay.min}
            max={STARTUP_LIMITS.energy_per_tick_decay.max}
            step={STARTUP_LIMITS.energy_per_tick_decay.step}
            value={runtimeConfig.energy_per_tick_decay}
            disabled={runtimeDisabled}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { energy_per_tick_decay: value },
                { ...runtimeConfig, energy_per_tick_decay: value }
              );
            }}
          />

          <label className="slider-label" htmlFor="runtime-energy-move-cost-slider">
            Runtime move cost: {runtimeConfig.energy_per_move.toFixed(3)}
          </label>
          <input
            id="runtime-energy-move-cost-slider"
            type="range"
            min={STARTUP_LIMITS.energy_per_move.min}
            max={STARTUP_LIMITS.energy_per_move.max}
            step={STARTUP_LIMITS.energy_per_move.step}
            value={runtimeConfig.energy_per_move}
            disabled={runtimeDisabled}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { energy_per_move: value },
                { ...runtimeConfig, energy_per_move: value }
              );
            }}
          />

          <label className="slider-label" htmlFor="runtime-energy-compute-cost-slider">
            Compute node cost: {runtimeConfig.energy_per_compute_node.toFixed(3)}
          </label>
          <input
            id="runtime-energy-compute-cost-slider"
            type="range"
            min={STARTUP_LIMITS.energy_per_compute_node.min}
            max={STARTUP_LIMITS.energy_per_compute_node.max}
            step={STARTUP_LIMITS.energy_per_compute_node.step}
            value={runtimeConfig.energy_per_compute_node}
            disabled={runtimeDisabled}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { energy_per_compute_node: value },
                { ...runtimeConfig, energy_per_compute_node: value }
              );
            }}
          />

          <label className="slider-label" htmlFor="runtime-energy-reproduce-cost-slider">
            Reproduction cost: {runtimeConfig.energy_per_reproduce.toFixed(3)}
          </label>
          <input
            id="runtime-energy-reproduce-cost-slider"
            type="range"
            min={STARTUP_LIMITS.energy_per_reproduce.min}
            max={STARTUP_LIMITS.energy_per_reproduce.max}
            step={STARTUP_LIMITS.energy_per_reproduce.step}
            value={runtimeConfig.energy_per_reproduce}
            disabled={runtimeDisabled}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { energy_per_reproduce: value },
                { ...runtimeConfig, energy_per_reproduce: value }
              );
            }}
          />

          <label className="slider-label" htmlFor="runtime-energy-max-slider">
            Energy max: {runtimeConfig.energy_max.toFixed(2)}
          </label>
          <input
            id="runtime-energy-max-slider"
            type="range"
            min={STARTUP_LIMITS.energy_max.min}
            max={STARTUP_LIMITS.energy_max.max}
            step={STARTUP_LIMITS.energy_max.step}
            value={runtimeConfig.energy_max}
            disabled={runtimeDisabled}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField({ energy_max: value }, { ...runtimeConfig, energy_max: value });
            }}
          />

          <label className="slider-label" htmlFor="runtime-min-reproduce-energy-slider">
            Min reproduce energy: {runtimeConfig.min_reproduce_energy.toFixed(2)}
          </label>
          <input
            id="runtime-min-reproduce-energy-slider"
            type="range"
            min={STARTUP_LIMITS.min_reproduce_energy.min}
            max={STARTUP_LIMITS.min_reproduce_energy.max}
            step={STARTUP_LIMITS.min_reproduce_energy.step}
            value={runtimeConfig.min_reproduce_energy}
            disabled={runtimeDisabled}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { min_reproduce_energy: value },
                { ...runtimeConfig, min_reproduce_energy: value }
              );
            }}
          />

          <label className="slider-label" htmlFor="runtime-offspring-energy-fraction-slider">
            Offspring energy fraction: {runtimeConfig.offspring_energy_fraction.toFixed(2)}
          </label>
          <input
            id="runtime-offspring-energy-fraction-slider"
            type="range"
            min={STARTUP_LIMITS.offspring_energy_fraction.min}
            max={STARTUP_LIMITS.offspring_energy_fraction.max}
            step={STARTUP_LIMITS.offspring_energy_fraction.step}
            value={runtimeConfig.offspring_energy_fraction}
            disabled={runtimeDisabled}
            onChange={(event) => {
              const value = Number(event.target.value);
              onUpdateRuntimeField(
                { offspring_energy_fraction: value },
                { ...runtimeConfig, offspring_energy_fraction: value }
              );
            }}
          />
        </>
      ) : (
        <p className="muted">Loading advanced config...</p>
      )}
    </section>
  );
}
