import type { StartupRequest } from "../protocol/models";

interface StartupPanelProps {
  draft: StartupRequest;
  onChange: (next: StartupRequest) => void;
  onApply: () => Promise<void>;
  disabled: boolean;
}

export function StartupPanel(props: StartupPanelProps) {
  const draft = props.draft;

  return (
    <div className="control-stack">
      <div className="grid-form">
        <label>
          Seed
          <input
            aria-label="Seed"
            type="number"
            value={draft.seed}
            onChange={(event) =>
              props.onChange({ ...draft, seed: Number(event.target.value) })
            }
          />
        </label>
        <label>
          Sensor Radius
          <input
            aria-label="Sensor Radius"
            type="number"
            min={1}
            value={draft.world.sensor_radius}
            onChange={(event) =>
              props.onChange({
                ...draft,
                world: { ...draft.world, sensor_radius: Number(event.target.value) },
              })
            }
          />
        </label>
        <label>
          World Width
          <input
            aria-label="World Width"
            type="number"
            min={1}
            value={draft.world.width}
            onChange={(event) =>
              props.onChange({
                ...draft,
                world: { ...draft.world, width: Number(event.target.value) },
              })
            }
          />
        </label>
        <label>
          World Height
          <input
            aria-label="World Height"
            type="number"
            min={1}
            value={draft.world.height}
            onChange={(event) =>
              props.onChange({
                ...draft,
                world: { ...draft.world, height: Number(event.target.value) },
              })
            }
          />
        </label>
        <label>
          Initial Creatures
          <input
            aria-label="Initial Creatures"
            type="number"
            min={0}
            value={draft.population.initial_creatures}
            onChange={(event) =>
              props.onChange({
                ...draft,
                population: {
                  ...draft.population,
                  initial_creatures: Number(event.target.value),
                },
              })
            }
          />
        </label>
        <label>
          Max Creatures
          <input
            aria-label="Max Creatures"
            type="number"
            min={1}
            value={draft.population.max_creatures}
            onChange={(event) =>
              props.onChange({
                ...draft,
                population: {
                  ...draft.population,
                  max_creatures: Number(event.target.value),
                },
              })
            }
          />
        </label>
        <label>
          Ticks Per Second
          <input
            aria-label="Ticks Per Second"
            type="number"
            min={1}
            value={draft.runtime.ticks_per_second}
            onChange={(event) =>
              props.onChange({
                ...draft,
                runtime: {
                  ...draft.runtime,
                  ticks_per_second: Math.max(1, Number(event.target.value)),
                },
              })
            }
          />
        </label>
        <label>
          Max Tick Budget (ms)
          <input
            aria-label="Max Tick Budget (ms)"
            type="number"
            min={1}
            value={draft.runtime.max_tick_budget_ms}
            onChange={(event) =>
              props.onChange({
                ...draft,
                runtime: {
                  ...draft.runtime,
                  max_tick_budget_ms: Math.max(1, Number(event.target.value)),
                },
              })
            }
          />
        </label>

        <label>
          Initial Food Density
          <input
            aria-label="Initial Food Density"
            type="number"
            min={0}
            max={1}
            step={0.01}
            value={draft.tuning.food.initial_food_density}
            onChange={(event) =>
              props.onChange({
                ...draft,
                tuning: {
                  ...draft.tuning,
                  food: {
                    ...draft.tuning.food,
                    initial_food_density: Number(event.target.value),
                  },
                },
              })
            }
          />
        </label>
        <label>
          Food Growth Rate
          <input
            aria-label="Food Growth Rate"
            type="number"
            min={0}
            max={1}
            step={0.01}
            value={draft.tuning.food.food_growth_rate}
            onChange={(event) =>
              props.onChange({
                ...draft,
                tuning: {
                  ...draft.tuning,
                  food: {
                    ...draft.tuning.food,
                    food_growth_rate: Number(event.target.value),
                  },
                },
              })
            }
          />
        </label>
        <label>
          Food Spawn Rate
          <input
            aria-label="Food Spawn Rate"
            type="number"
            min={0}
            max={1}
            step={0.01}
            value={draft.tuning.food.food_spawn_rate}
            onChange={(event) =>
              props.onChange({
                ...draft,
                tuning: {
                  ...draft.tuning,
                  food: {
                    ...draft.tuning.food,
                    food_spawn_rate: Number(event.target.value),
                  },
                },
              })
            }
          />
        </label>
        <label>
          Food Spread Threshold
          <input
            aria-label="Food Spread Threshold"
            type="number"
            min={0}
            max={1}
            step={0.01}
            value={draft.tuning.food.food_spread_threshold}
            onChange={(event) =>
              props.onChange({
                ...draft,
                tuning: {
                  ...draft.tuning,
                  food: {
                    ...draft.tuning.food,
                    food_spread_threshold: Number(event.target.value),
                  },
                },
              })
            }
          />
        </label>
        <label>
          Food Spawn Floor Density
          <input
            aria-label="Food Spawn Floor Density"
            type="number"
            min={0}
            max={1}
            step={0.01}
            value={draft.tuning.food.food_spawn_floor_density}
            onChange={(event) =>
              props.onChange({
                ...draft,
                tuning: {
                  ...draft.tuning,
                  food: {
                    ...draft.tuning.food,
                    food_spawn_floor_density: Number(event.target.value),
                  },
                },
              })
            }
          />
        </label>
        <label>
          Initial Energy
          <input
            aria-label="Initial Energy"
            type="number"
            min={0}
            step={0.1}
            value={draft.tuning.tick.initial_energy}
            onChange={(event) =>
              props.onChange({
                ...draft,
                tuning: {
                  ...draft.tuning,
                  tick: {
                    ...draft.tuning.tick,
                    initial_energy: Number(event.target.value),
                  },
                },
              })
            }
          />
        </label>
        <label>
          Tick Decay Energy
          <input
            aria-label="Tick Decay Energy"
            type="number"
            min={0}
            step={0.01}
            value={draft.tuning.tick.energy_decay_per_tick}
            onChange={(event) =>
              props.onChange({
                ...draft,
                tuning: {
                  ...draft.tuning,
                  tick: {
                    ...draft.tuning.tick,
                    energy_decay_per_tick: Number(event.target.value),
                  },
                },
              })
            }
          />
        </label>
        <label>
          Move Cost
          <input
            aria-label="Move Cost"
            type="number"
            min={0}
            step={0.01}
            value={draft.tuning.tick.move_cost}
            onChange={(event) =>
              props.onChange({
                ...draft,
                tuning: {
                  ...draft.tuning,
                  tick: {
                    ...draft.tuning.tick,
                    move_cost: Number(event.target.value),
                  },
                },
              })
            }
          />
        </label>
        <label>
          Food Energy Gain
          <input
            aria-label="Food Energy Gain"
            type="number"
            min={0}
            step={0.01}
            value={draft.tuning.tick.food_energy_gain}
            onChange={(event) =>
              props.onChange({
                ...draft,
                tuning: {
                  ...draft.tuning,
                  tick: {
                    ...draft.tuning.tick,
                    food_energy_gain: Number(event.target.value),
                  },
                },
              })
            }
          />
        </label>
        <label>
          Reproduce Cost
          <input
            aria-label="Reproduce Cost"
            type="number"
            min={0}
            step={0.01}
            value={draft.tuning.tick.reproduce_cost}
            onChange={(event) =>
              props.onChange({
                ...draft,
                tuning: {
                  ...draft.tuning,
                  tick: {
                    ...draft.tuning.tick,
                    reproduce_cost: Number(event.target.value),
                  },
                },
              })
            }
          />
        </label>
        <label>
          Min Reproduce Energy
          <input
            aria-label="Min Reproduce Energy"
            type="number"
            min={0}
            step={0.1}
            value={draft.tuning.tick.min_reproduce_energy}
            onChange={(event) =>
              props.onChange({
                ...draft,
                tuning: {
                  ...draft.tuning,
                  tick: {
                    ...draft.tuning.tick,
                    min_reproduce_energy: Number(event.target.value),
                  },
                },
              })
            }
          />
        </label>
        <label>
          Offspring Energy Fraction
          <input
            aria-label="Offspring Energy Fraction"
            type="number"
            min={0}
            max={1}
            step={0.01}
            value={draft.tuning.tick.offspring_energy_fraction}
            onChange={(event) =>
              props.onChange({
                ...draft,
                tuning: {
                  ...draft.tuning,
                  tick: {
                    ...draft.tuning.tick,
                    offspring_energy_fraction: Number(event.target.value),
                  },
                },
              })
            }
          />
        </label>
        <label>
          Energy Max
          <input
            aria-label="Energy Max"
            type="number"
            min={0.1}
            step={0.1}
            value={draft.tuning.tick.energy_max}
            onChange={(event) =>
              props.onChange({
                ...draft,
                tuning: {
                  ...draft.tuning,
                  tick: {
                    ...draft.tuning.tick,
                    energy_max: Math.max(0.1, Number(event.target.value)),
                  },
                },
              })
            }
          />
        </label>
      </div>

      <label>
        World Wrap
        <select
          aria-label="World Wrap"
          value={draft.world.wrap ? "wrap" : "bounded"}
          onChange={(event) =>
            props.onChange({
              ...draft,
              world: { ...draft.world, wrap: event.target.value === "wrap" },
            })
          }
        >
          <option value="wrap">Wrap</option>
          <option value="bounded">Bounded</option>
        </select>
      </label>

      <button
        className="primary"
        type="button"
        onClick={() => void props.onApply()}
        disabled={props.disabled}
      >
        Apply Startup
      </button>
    </div>
  );
}
