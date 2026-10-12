/** Readable names for the server's raw stat keys. */

const REJECTION_REASONS: Record<string, { label: string; hint: string }> = {
	RejectedInvalidTarget: {
		label: "Target cell blocked",
		hint: "The cell for the child was a barrier, occupied or off the map.",
	},
	RejectedAgeConstraints: {
		label: "Too young",
		hint: "The parent was below Min Reproduce Age.",
	},
	RejectedEnergyConstraints: {
		label: "Not enough energy",
		hint: "The parent was below Min Reproduce Energy or could not pay the charge.",
	},
	RejectedPopulationCap: {
		label: "Population cap",
		hint: "The world was at Max Creatures.",
	},
};

/** A reproduction rejection reason key, as words; unknown keys pass through. */
export function rejectionReason(key: string): { label: string; hint: string } {
	return REJECTION_REASONS[key] ?? { label: key, hint: "" };
}

/** Split `CamelCase` or `snake_case` into lower-case words. */
function words(text: string): string {
	return text
		.replace(/_/g, " ")
		.replace(/([a-z0-9])([A-Z])/g, "$1 $2")
		.replace(/([A-Z]+)([A-Z][a-z])/g, "$1 $2")
		.toLowerCase();
}

const DOMAIN_NAMES: Record<string, string> = {
	Topology: "Mesh",
	Vm: "VM",
	Graph: "Graph",
	InputRef: "Inputs",
};

/** A mutation operator key such as `Vm.VmConstantMutation`, as
 * `VM · constant mutation`; the domain prefix repeated in the name is dropped. */
export function mutationOperatorLabel(key: string): string {
	const dot = key.indexOf(".");
	if (dot < 0) return words(key);
	const domain = key.slice(0, dot);
	const name = key.slice(dot + 1);
	const bare = name.startsWith(domain) ? name.slice(domain.length) : name;
	return `${DOMAIN_NAMES[domain] ?? domain} · ${words(bare)}`;
}
