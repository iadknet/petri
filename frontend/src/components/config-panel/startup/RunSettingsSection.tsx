import { FieldGroup } from "../shared/FieldGroup.tsx";
import { NUMBER_INPUT_CLASS, RowLabel } from "../shared/FieldRow.tsx";
import { NumberInput } from "../shared/NumberInput.tsx";
import { RandomSeedButton } from "../shared/RandomSeedButton.tsx";
import { useRowFilter } from "../shared/filter.ts";
import type { FieldDef } from "../shared/types.ts";

interface RunSettingsSectionProps {
	seed: number;
	updateSeed: (value: number) => void;
	randomizeSeed: () => void;
}

const SEED_FIELD: FieldDef = {
	path: "seed",
	label: "Run Seed",
	min: 0,
	max: Number.MAX_SAFE_INTEGER,
	step: 1,
	tooltip:
		"Controls food placement, founders and runtime draws; also the map when Map Seed is blank. Same seed, same world.",
};

export function RunSettingsSection({ seed, updateSeed, randomizeSeed }: RunSettingsSectionProps) {
	const { visible } = useRowFilter(SEED_FIELD, false);
	return (
		<FieldGroup title="Run">
			{visible && (
				<div data-config-row className="-mx-2 flex flex-col gap-1 rounded px-2 py-1.5">
					<RowLabel
						inputId="startup-seed-input"
						label={SEED_FIELD.label}
						tooltip={SEED_FIELD.tooltip}
						isDirty={false}
					/>
					<div className="flex items-center gap-1.5">
						<NumberInput
							id="startup-seed-input"
							data-testid="startup-field-seed"
							value={seed}
							min={0}
							max={Number.MAX_SAFE_INTEGER}
							integer
							onChange={(value) => {
								if (Number.isSafeInteger(value) && value >= 0) updateSeed(value);
							}}
							className={`${NUMBER_INPUT_CLASS} w-auto min-w-0 flex-1 text-left`}
						/>
						<RandomSeedButton
							label="run seed"
							testId="startup-seed-randomize"
							onClick={randomizeSeed}
						/>
					</div>
				</div>
			)}
		</FieldGroup>
	);
}
