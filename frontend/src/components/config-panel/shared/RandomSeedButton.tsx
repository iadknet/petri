import { Icon } from "./Icon.tsx";

interface RandomSeedButtonProps {
	label: string;
	testId: string;
	onClick: () => void;
}

export function RandomSeedButton({ label, testId, onClick }: RandomSeedButtonProps) {
	return (
		<button
			type="button"
			data-testid={testId}
			onClick={onClick}
			title={`Randomize ${label}`}
			aria-label={`Randomize ${label}`}
			className="flex size-6 shrink-0 items-center justify-center rounded border border-slate-700 text-slate-400 transition-colors hover:border-slate-600 hover:bg-slate-800 hover:text-slate-100"
		>
			<Icon name="shuffle" className="size-3.5" />
		</button>
	);
}
