import { type ReactNode, createContext, useContext, useId, useState } from "react";
import { Icon } from "./Icon.tsx";
import { ConfigFilterContext, NO_FILTER } from "./filter.ts";

const COLLAPSED_KEY = "petri.config-panel.collapsed";

function readCollapsed(): Set<string> {
	try {
		const raw = window.localStorage.getItem(COLLAPSED_KEY);
		return new Set(raw ? (JSON.parse(raw) as string[]) : []);
	} catch {
		return new Set();
	}
}

function writeCollapsed(ids: Set<string>) {
	try {
		window.localStorage.setItem(COLLAPSED_KEY, JSON.stringify([...ids]));
	} catch {
		// Storage can be blocked; collapse state is a convenience only.
	}
}

/** Remembers, per viewer, which categories were folded away. */
function useCollapsed(id: string): [boolean, (collapsed: boolean) => void] {
	const [collapsed, setCollapsed] = useState(() => readCollapsed().has(id));
	return [
		collapsed,
		(next) => {
			const ids = readCollapsed();
			if (next) ids.add(id);
			else ids.delete(id);
			writeCollapsed(ids);
			setCollapsed(next);
		},
	];
}

/** Whether the surrounding panel is the live one, for accents. */
export const LiveScopeContext = createContext(false);

interface CategoryProps {
	id: string;
	title: string;
	summary: string;
	/** Settings in the category, shown muted beside the title. */
	settingCount?: number;
	/** Live settings changed and not applied yet. */
	changedCount?: number;
	/** Open while a search or filter is showing matches inside. */
	forceOpen?: boolean;
	children: ReactNode;
}

export function Category({
	id,
	title,
	summary,
	settingCount,
	changedCount = 0,
	forceOpen = false,
	children,
}: CategoryProps) {
	const live = useContext(LiveScopeContext);
	const [collapsed, setCollapsed] = useCollapsed(id);
	const regionId = useId();
	const expanded = forceOpen || !collapsed;

	return (
		<section data-config-category={id} className="border-b border-slate-800/80">
			<h3 className="sticky top-0 z-10 bg-petri-panel">
				<button
					type="button"
					aria-expanded={expanded}
					aria-controls={regionId}
					onClick={() => setCollapsed(expanded)}
					className="group flex w-full items-start gap-2 px-3 py-2.5 text-left hover:bg-slate-800/40 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-sky-400"
				>
					<Icon
						name="chevron"
						className={`mt-0.5 size-3.5 text-slate-400 transition-transform duration-150 group-hover:text-slate-300 motion-reduce:transition-none ${expanded ? "rotate-90" : ""}`}
					/>
					<span className="flex min-w-0 flex-1 flex-col">
						<span className="flex items-baseline gap-2">
							<span className="text-[13px] font-semibold text-slate-100">{title}</span>
							{settingCount !== undefined && (
								<span className="font-mono text-[10px] tabular-nums text-slate-400">
									{settingCount}
								</span>
							)}
						</span>
						<span className="text-[11px] leading-snug text-slate-400">{summary}</span>
					</span>
					{changedCount > 0 && (
						<span
							className={`mt-0.5 shrink-0 rounded px-1.5 py-px text-[10px] font-medium tabular-nums ${live ? "bg-sky-400/15 text-sky-300" : "bg-emerald-400/15 text-emerald-300"}`}
						>
							{changedCount} changed
						</span>
					)}
				</button>
			</h3>
			<div id={regionId} hidden={!expanded} className="pb-2">
				{children}
			</div>
		</section>
	);
}

/**
 * Shows an editor whole when its category matched the search: list editors
 * (food types, layers) make no sense with half their fields filtered away.
 */
export function Unfiltered({ children }: { children: ReactNode }) {
	return (
		<ConfigFilterContext value={NO_FILTER}>
			<div data-config-row>{children}</div>
		</ConfigFilterContext>
	);
}
