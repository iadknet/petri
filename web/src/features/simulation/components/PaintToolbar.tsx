import { IdlePreviewMode, PaintStats, PaintTool } from "../../../protocol";

type PaintToolbarProps = {
  phase: string;
  paintModeEnabled: boolean;
  paintTool: PaintTool;
  brushHalfExtent: 0 | 1 | 2;
  idlePreviewMode: IdlePreviewMode;
  lastPaintStats: PaintStats | null;
  onSetPaintTool: (tool: PaintTool) => void;
  onSetBrushHalfExtent: (extent: 0 | 1 | 2) => void;
  onSetIdlePreviewMode: (mode: IdlePreviewMode) => void;
  onClearPaint: () => void;
};

export function PaintToolbar({
  phase,
  paintModeEnabled,
  paintTool,
  brushHalfExtent,
  idlePreviewMode,
  lastPaintStats,
  onSetPaintTool,
  onSetBrushHalfExtent,
  onSetIdlePreviewMode,
  onClearPaint
}: PaintToolbarProps) {
  if (!paintModeEnabled) {
    return null;
  }

  return (
    <div className="paint-toolbar" role="group" aria-label="Paint tools">
      <div className="paint-row">
        <button
          className={`pill-btn ${paintTool === "food" ? "active" : ""}`}
          onClick={() => onSetPaintTool("food")}
        >
          Food
        </button>
        <button
          className={`pill-btn ${paintTool === "barrier" ? "active" : ""}`}
          onClick={() => onSetPaintTool("barrier")}
        >
          Barrier
        </button>
        <button
          className={`pill-btn ${paintTool === "erase_food" ? "active" : ""}`}
          onClick={() => onSetPaintTool("erase_food")}
        >
          Erase Food
        </button>
        <button
          className={`pill-btn ${paintTool === "erase_barrier" ? "active" : ""}`}
          onClick={() => onSetPaintTool("erase_barrier")}
        >
          Erase Barrier
        </button>
      </div>

      <div className="paint-row">
        <span className="paint-label">Brush</span>
        <button
          className={`pill-btn ${brushHalfExtent === 0 ? "active" : ""}`}
          onClick={() => onSetBrushHalfExtent(0)}
        >
          1x1
        </button>
        <button
          className={`pill-btn ${brushHalfExtent === 1 ? "active" : ""}`}
          onClick={() => onSetBrushHalfExtent(1)}
        >
          3x3
        </button>
        <button
          className={`pill-btn ${brushHalfExtent === 2 ? "active" : ""}`}
          onClick={() => onSetBrushHalfExtent(2)}
        >
          5x5
        </button>
      </div>

      {phase === "idle" ? (
        <div className="paint-row">
          <span className="paint-label">Idle Preview</span>
          <button
            className={`pill-btn ${idlePreviewMode === "paint_layer" ? "active" : ""}`}
            onClick={() => onSetIdlePreviewMode("paint_layer")}
          >
            Paint Layer
          </button>
          <button
            className={`pill-btn ${idlePreviewMode === "full_startup" ? "active" : ""}`}
            onClick={() => onSetIdlePreviewMode("full_startup")}
          >
            Full Startup
          </button>
        </div>
      ) : null}

      <div className="paint-row">
        <button className="pill-btn danger" onClick={onClearPaint}>
          Clear Paint
        </button>
        {lastPaintStats ? (
          <span className="paint-stats">Affected: {lastPaintStats.affected_cells}</span>
        ) : null}
      </div>
    </div>
  );
}
