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
