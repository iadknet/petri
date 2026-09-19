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
			className="shrink-0 px-1.5 py-0.5 text-xs bg-slate-700 text-slate-200 rounded hover:bg-slate-600 transition-colors"
		>
			&#x21bb;
		</button>
	);
}
