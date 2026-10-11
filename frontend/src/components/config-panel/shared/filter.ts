import { createContext, useContext } from "react";
import type { AnyFieldDef } from "./types.ts";

export interface ConfigFilter {
	/** Lowercased search words; empty means no search. */
	terms: string[];
	/** Show only rows whose draft differs from the applied config. */
	changedOnly: boolean;
}

export const NO_FILTER: ConfigFilter = { terms: [], changedOnly: false };

export const ConfigFilterContext = createContext<ConfigFilter>(NO_FILTER);

export function useConfigFilter(): ConfigFilter {
	return useContext(ConfigFilterContext);
}

export function searchTerms(query: string): string[] {
	return query.toLowerCase().split(/\s+/).filter(Boolean);
}

/** "world.food.shared.growth_rate" also matches "growth rate" and "food shared". */
function pathWords(path: string): string {
	return `${path} ${path.replace(/[._]/g, " ")}`;
}

function haystack(parts: (string | undefined)[]): string {
	return parts.filter(Boolean).join(" ").toLowerCase();
}

export function matchesTerms(terms: string[], parts: (string | undefined)[]): boolean {
	if (terms.length === 0) return true;
	const text = haystack(parts);
	return terms.every((term) => text.includes(term));
}

export type FieldMatch = "name" | "help" | null;

/**
 * Whether a field matches the search, and where: on its name (label or config
 * path) or only in its help text, which the row then shows inline.
 */
export function matchField(field: AnyFieldDef, terms: string[]): FieldMatch {
	if (terms.length === 0) return "name";
	if (matchesTerms(terms, [field.label, pathWords(field.path)])) return "name";
	if (matchesTerms(terms, [field.label, pathWords(field.path), field.tooltip])) return "help";
	return null;
}

/** Row visibility under the active filter. */
export function useRowFilter(field: AnyFieldDef, isDirty: boolean) {
	const filter = useConfigFilter();
	const match = matchField(field, filter.terms);
	const visible = match !== null && (!filter.changedOnly || isDirty);
	return { visible, showHelp: match === "help" };
}
