import { StartupDraft } from "../../../protocol";
import { STARTUP_LIMITS } from "../store/simulationStore";

type StartupDraftPanelProps = {
  startupDraft: StartupDraft | null;
  phase: string;
  onUpdate: <K extends keyof StartupDraft>(key: K, value: StartupDraft[K]) => void;
};

export function StartupDraftPanel({ startupDraft, phase, onUpdate }: StartupDraftPanelProps) {
  return (
    <section className="section">
      <h2>Startup Draft</h2>
      {startupDraft ? (
        <>
          <label className="slider-label" htmlFor="initial-creatures-slider">
            Initial population: {startupDraft.initial_creatures}
          </label>
          <input
            id="initial-creatures-slider"
            type="range"
            min={STARTUP_LIMITS.initial_creatures.min}
            max={STARTUP_LIMITS.initial_creatures.max}
            step={STARTUP_LIMITS.initial_creatures.step}
            value={startupDraft.initial_creatures}
            onChange={(event) => onUpdate("initial_creatures", Number(event.target.value))}
          />

          <label className="slider-label" htmlFor="world-width-slider">
            World width: {startupDraft.width}
          </label>
          <input
            id="world-width-slider"
            type="range"
            min={STARTUP_LIMITS.width.min}
            max={STARTUP_LIMITS.width.max}
            step={STARTUP_LIMITS.width.step}
            value={startupDraft.width}
            onChange={(event) => onUpdate("width", Number(event.target.value))}
          />

          <label className="slider-label" htmlFor="world-height-slider">
            World height: {startupDraft.height}
          </label>
          <input
            id="world-height-slider"
            type="range"
            min={STARTUP_LIMITS.height.min}
            max={STARTUP_LIMITS.height.max}
            step={STARTUP_LIMITS.height.step}
            value={startupDraft.height}
            onChange={(event) => onUpdate("height", Number(event.target.value))}
          />

          <label className="slider-label" htmlFor="initial-food-density-slider">
            Initial food density: {startupDraft.initial_food_density.toFixed(2)}
          </label>
          <input
            id="initial-food-density-slider"
            type="range"
            min={STARTUP_LIMITS.initial_food_density.min}
            max={STARTUP_LIMITS.initial_food_density.max}
            step={STARTUP_LIMITS.initial_food_density.step}
            value={startupDraft.initial_food_density}
            onChange={(event) => onUpdate("initial_food_density", Number(event.target.value))}
          />

          <label className="slider-label" htmlFor="startup-food-spawn-rate-slider">
            Food spawn rate: {startupDraft.food_spawn_rate.toFixed(2)}
          </label>
          <input
            id="startup-food-spawn-rate-slider"
            type="range"
            min={STARTUP_LIMITS.food_spawn_rate.min}
            max={STARTUP_LIMITS.food_spawn_rate.max}
            step={STARTUP_LIMITS.food_spawn_rate.step}
            value={startupDraft.food_spawn_rate}
            onChange={(event) => onUpdate("food_spawn_rate", Number(event.target.value))}
          />

          <label className="slider-label" htmlFor="startup-food-growth-rate-slider">
            Food growth rate: {startupDraft.food_growth_rate.toFixed(2)}
          </label>
          <input
            id="startup-food-growth-rate-slider"
            type="range"
            min={STARTUP_LIMITS.food_growth_rate.min}
            max={STARTUP_LIMITS.food_growth_rate.max}
            step={STARTUP_LIMITS.food_growth_rate.step}
            value={startupDraft.food_growth_rate}
            onChange={(event) => onUpdate("food_growth_rate", Number(event.target.value))}
          />

          <label className="slider-label" htmlFor="tick-decay-slider">
            Tick decay: {startupDraft.energy_per_tick_decay.toFixed(3)}
          </label>
          <input
            id="tick-decay-slider"
            type="range"
            min={STARTUP_LIMITS.energy_per_tick_decay.min}
            max={STARTUP_LIMITS.energy_per_tick_decay.max}
            step={STARTUP_LIMITS.energy_per_tick_decay.step}
            value={startupDraft.energy_per_tick_decay}
            onChange={(event) => onUpdate("energy_per_tick_decay", Number(event.target.value))}
          />

          <label className="slider-label" htmlFor="move-cost-slider">
            Move cost: {startupDraft.energy_per_move.toFixed(3)}
          </label>
          <input
            id="move-cost-slider"
            type="range"
            min={STARTUP_LIMITS.energy_per_move.min}
            max={STARTUP_LIMITS.energy_per_move.max}
            step={STARTUP_LIMITS.energy_per_move.step}
            value={startupDraft.energy_per_move}
            onChange={(event) => onUpdate("energy_per_move", Number(event.target.value))}
          />

          <label className="slider-label" htmlFor="world-wrap-toggle">
            World wrap
          </label>
          <input
            id="world-wrap-toggle"
            type="checkbox"
            checked={startupDraft.world_wrap}
            onChange={(event) => onUpdate("world_wrap", event.target.checked)}
          />

          {phase !== "idle" ? (
            <p className="muted">Startup draft updates apply on Restart Simulation.</p>
          ) : null}
        </>
      ) : (
        <p className="muted">Loading startup draft...</p>
      )}
    </section>
  );
}
