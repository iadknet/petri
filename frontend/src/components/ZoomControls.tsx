interface ZoomControlsProps {
	onZoomIn: () => void;
	onZoomOut: () => void;
	onFitToWorld: () => void;
}

export const mapControlSurface =
	"rounded-lg border border-slate-700/60 bg-slate-900/80 backdrop-blur-sm";

const buttonClass =
	"flex h-8 w-8 items-center justify-center text-slate-300 transition-[color,background-color,transform] duration-150 ease-out hover:bg-slate-700/60 hover:text-slate-50 active:scale-90 focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-sky-400";

function Icon({ children }: { children: React.ReactNode }) {
	return (
		<svg
			aria-hidden="true"
			viewBox="0 0 16 16"
			className="h-4 w-4"
			fill="none"
			stroke="currentColor"
			strokeWidth={1.5}
			strokeLinecap="round"
			strokeLinejoin="round"
		>
			{children}
		</svg>
	);
}

export function ZoomControls({ onZoomIn, onZoomOut, onFitToWorld }: ZoomControlsProps) {
	return (
		<fieldset
			aria-label="Map zoom"
			className={`absolute top-3 right-3 z-10 m-0 flex min-w-0 flex-col overflow-hidden p-0 ${mapControlSurface}`}
		>
			<button
				type="button"
				data-testid="zoom-in"
				onClick={onZoomIn}
				className={buttonClass}
				title="Zoom in"
				aria-label="Zoom in"
			>
				<Icon>
					<path d="M8 3.5v9M3.5 8h9" />
				</Icon>
			</button>
			<button
				type="button"
				data-testid="zoom-out"
				onClick={onZoomOut}
				className={buttonClass}
				title="Zoom out"
				aria-label="Zoom out"
			>
				<Icon>
					<path d="M3.5 8h9" />
				</Icon>
			</button>
			<div aria-hidden="true" className="mx-2 h-px bg-slate-700/60" />
			<button
				type="button"
				data-testid="zoom-fit"
				onClick={onFitToWorld}
				className={buttonClass}
				title="Fit to world (Home)"
				aria-label="Fit to world"
			>
				<Icon>
					<path d="M3 6V3.5a.5.5 0 0 1 .5-.5H6M10 3h2.5a.5.5 0 0 1 .5.5V6M13 10v2.5a.5.5 0 0 1-.5.5H10M6 13H3.5a.5.5 0 0 1-.5-.5V10" />
				</Icon>
			</button>
		</fieldset>
	);
}
