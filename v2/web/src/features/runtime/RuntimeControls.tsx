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
  const disableStart = props.disabled || props.state === "running";
  const disablePause = props.disabled || props.state !== "running";
  const disableStep = props.disabled || props.state !== "paused";

  return (
    <div className="control-stack">
      <p className="status-row">
        <span>State</span>
        <strong>{props.state}</strong>
        <span>Tick</span>
        <strong>{props.tick}</strong>
      </p>
      <div className="runtime-action-row">
        <button type="button" onClick={() => void props.onStart()} disabled={disableStart}>
          Start
        </button>
        <button type="button" onClick={() => void props.onPause()} disabled={disablePause}>
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
          disabled={props.disabled}
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
          disabled={props.disabled}
          onChange={(event) =>
            props.onTicksPerSecondChange(Math.max(1, Number(event.target.value) || 1))
          }
        />
      </label>

      <button
        className="primary"
        type="button"
        onClick={() => void props.onStep()}
        disabled={disableStep}
      >
        Step
      </button>
    </div>
  );
}
