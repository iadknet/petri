import type { SimulationConfig } from "../../../types/generated/SimulationConfig.ts";
import { Category } from "../shared/Category.tsx";
import { matchField, useConfigFilter } from "../shared/filter.ts";
import { sameConfigNumber } from "../shared/numberFormat.ts";
import { getByPath } from "../shared/pathUtils.ts";
import type { AnyFieldDef, RuntimePanelProps } from "../shared/types.ts";
import { RuntimeFieldGroup } from "./RuntimeFieldGroup.tsx";
import { RUNTIME_CATEGORIES, RUNTIME_PATCH_FIELDS } from "./groups.ts";

export { RUNTIME_PATCH_FIELDS };

/** Whether a field's draft differs from what the server applied. */
export function isFieldDirty(
	field: AnyFieldDef,
	draft: SimulationConfig,
	server: SimulationConfig,
): boolean {
	const a = getByPath(draft, field.path);
	const b = getByPath(server, field.path);
	return typeof a === "number" && typeof b === "number" ? !sameConfigNumber(a, b) : a !== b;
}

/** Live fields the current search and filter leave visible. */
export function visibleRuntimeFields(
	terms: string[],
	changedOnly: boolean,
	draft: SimulationConfig,
	server: SimulationConfig,
): AnyFieldDef[] {
	return RUNTIME_PATCH_FIELDS.filter(
		(field) =>
			matchField(field, terms) !== null && (!changedOnly || isFieldDirty(field, draft, server)),
	);
}

interface RuntimeConfigPanelProps extends Omit<RuntimePanelProps, "localDraft" | "serverConfig"> {
	localDraft: RuntimePanelProps["localDraft"] | null;
	serverConfig: RuntimePanelProps["serverConfig"] | null;
}

export function RuntimeConfigPanel({ localDraft, serverConfig, ...rest }: RuntimeConfigPanelProps) {
	const filter = useConfigFilter();
	if (!localDraft || !serverConfig) {
		return (
			<p className="px-3 py-6 text-center text-xs text-slate-400">
				Runtime config unavailable. Restart to initialize the simulation.
			</p>
		);
	}

	const filtering = filter.terms.length > 0 || filter.changedOnly;
	const visible = new Set(
		visibleRuntimeFields(filter.terms, filter.changedOnly, localDraft, serverConfig),
	);

	return (
		<>
			{RUNTIME_CATEGORIES.map((category) => {
				const fields = category.subgroups.flatMap((subgroup) => subgroup.fields);
				const shown = fields.filter((field) => visible.has(field));
				if (shown.length === 0) return null;
				return (
					<Category
						key={category.id}
						id={`live-${category.id}`}
						title={category.title}
						summary={category.summary}
						settingCount={fields.length}
						changedCount={
							fields.filter((field) => isFieldDirty(field, localDraft, serverConfig)).length
						}
						forceOpen={filtering}
					>
						{category.subgroups
							.filter((subgroup) => subgroup.fields.some((field) => visible.has(field)))
							.map((subgroup) => (
								<RuntimeFieldGroup
									key={subgroup.title}
									title={subgroup.title}
									fields={subgroup.fields}
									{...rest}
									localDraft={localDraft}
									serverConfig={serverConfig}
								/>
							))}
					</Category>
				);
			})}
		</>
	);
}
