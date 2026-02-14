import type { PaintTool } from "../protocol/models";

interface PaintToolbarProps {
  tool: PaintTool;
  brushHalfExtent: number;
  editable: boolean;
  busy: boolean;
  paintLastTouchedCells: number | null;
  errorMessage: string | null;
  onToolChange: (tool: PaintTool) => void;
  onBrushHalfExtentChange: (halfExtent: number) => void;
  onClearAll: () => Promise<void>;
}

interface ToolOption {
  label: string;
  value: PaintTool;
}

const TOOL_OPTIONS: ToolOption[] = [
  { label: "Food Tool", value: "food" },
  { label: "Barrier Tool", value: "barrier" },
  { label: "Erase Food Tool", value: "erase_food" },
  { label: "Erase Barrier Tool", value: "erase_barrier" },
];

const BRUSH_OPTIONS = [
  { label: "1x1", value: 0 },
  { label: "3x3", value: 1 },
  { label: "5x5", value: 2 },
];

export function PaintToolbar(props: PaintToolbarProps) {
  return (
    <div className="paint-controls control-stack">
      <p>
        Editability: <strong>{props.editable ? "editable" : "running (locked)"}</strong>
      </p>

      <div className="paint-tool-grid">
        {TOOL_OPTIONS.map((tool) => (
          <button
            key={tool.value}
            type="button"
            className={props.tool === tool.value ? "active" : undefined}
            onClick={() => props.onToolChange(tool.value)}
            disabled={props.busy}
          >
            {tool.label}
          </button>
        ))}
      </div>

      <div className="paint-brush-row">
        {BRUSH_OPTIONS.map((brush) => (
          <button
            key={brush.value}
            type="button"
            className={props.brushHalfExtent === brush.value ? "active" : undefined}
            onClick={() => props.onBrushHalfExtentChange(brush.value)}
            disabled={props.busy}
          >
            {brush.label}
          </button>
        ))}
      </div>

      <button type="button" onClick={() => void props.onClearAll()} disabled={!props.editable || props.busy}>
        Clear All
      </button>

      <p>Paint last touched cells: {props.paintLastTouchedCells ?? 0}</p>
      {!props.editable ? <p className="muted">Paint disabled while simulation is running.</p> : null}
      {props.errorMessage ? <p className="error-text">{props.errorMessage}</p> : null}
    </div>
  );
}
