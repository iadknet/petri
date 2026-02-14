import type { LifecycleState } from "../protocol/models";

interface RuntimeControlsProps {
  state: LifecycleState;
  tick: number;
  ticksPerSecond: number;
  onTicksPerSecondChange: (next: number) => void;
  stepCount: number;
  onStepCountChange: (next: number) => void;
  onStart: () => Promise<void>;
  onPause: () => Promise<void>;
  onStep: () => Promise<void>;
  onRefresh: () => Promise<void>;
  disabled: boolean;
}

export function RuntimeControls(props: RuntimeControlsProps) {
  return (
    <div className="control-stack">
      <p>
        state <strong>{props.state}</strong> | tick <strong>{props.tick}</strong>
      </p>
      <div style={{ display: "flex", gap: "0.4rem", flexWrap: "wrap" }}>
        <button type="button" onClick={() => void props.onStart()} disabled={props.disabled}>
          Start
        </button>
        <button type="button" onClick={() => void props.onPause()} disabled={props.disabled}>
          Pause
        </button>
        <button type="button" onClick={() => void props.onRefresh()} disabled={props.disabled}>
          Refresh
        </button>
      </div>

      <label>
        Step Count
        <input
          aria-label="Step Count"
          type="number"
          min={1}
          max={1000}
          value={props.stepCount}
          onChange={(event) =>
            props.onStepCountChange(Math.max(1, Number(event.target.value) || 1))
          }
        />
      </label>

      <label>
        Ticks Per Second
        <input
          aria-label="Ticks Per Second Runtime"
          type="number"
          min={1}
          value={props.ticksPerSecond}
          onChange={(event) =>
            props.onTicksPerSecondChange(Math.max(1, Number(event.target.value) || 1))
          }
        />
      </label>

      <button
        className="primary"
        type="button"
        onClick={() => void props.onStep()}
        disabled={props.disabled || props.state !== "paused"}
      >
        Step
      </button>
    </div>
  );
}
