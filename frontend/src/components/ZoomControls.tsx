interface ZoomControlsProps {
	onZoomIn: () => void;
	onZoomOut: () => void;
	onFitToWorld: () => void;
}

export function ZoomControls({ onZoomIn, onZoomOut, onFitToWorld }: ZoomControlsProps) {
	const btnClass =
		"w-8 h-8 flex items-center justify-center text-sm text-slate-300 bg-slate-800/60 backdrop-blur-sm rounded hover:bg-slate-700/80 transition-colors";

	return (
		<div className="absolute top-3 right-3 flex flex-col gap-1 z-10">
			<button type="button" onClick={onZoomIn} className={btnClass} title="Zoom in">
				+
			</button>
			<button type="button" onClick={onZoomOut} className={btnClass} title="Zoom out">
				&minus;
			</button>
			<button type="button" onClick={onFitToWorld} className={btnClass} title="Fit to world (Home)">
				&#x2B1C;
			</button>
		</div>
	);
}
