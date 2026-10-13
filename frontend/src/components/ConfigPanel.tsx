import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import type { DeepPartial } from "../api/rest.ts";
import { ApiRequestError, api } from "../api/rest.ts";
import { useConfigStore } from "../stores/config.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import { useStartupConfigStore } from "../stores/startupConfig.ts";
import type { SimulationConfig } from "../types/api.ts";
import { type FieldError, describeApiFailure } from "../types/errors.ts";
import {
	RUNTIME_PATCH_FIELDS,
	RuntimeConfigPanel,
	isFieldDirty,
	visibleRuntimeFields,
} from "./config-panel/runtime/RuntimeConfigPanel.tsx";
import { LiveScopeContext } from "./config-panel/shared/Category.tsx";
import { Icon } from "./config-panel/shared/Icon.tsx";
import { StartupBaselineContext } from "./config-panel/shared/baseline.ts";
import {
	type ConfigFilter,
	ConfigFilterContext,
	searchTerms,
} from "./config-panel/shared/filter.ts";
import { buildPatch, getByPath, mergePatch } from "./config-panel/shared/pathUtils.ts";
import {
	StartupConfigPanel,
	startupChangedCount,
	startupMatchCount,
} from "./config-panel/startup/StartupConfigPanel.tsx";

/** Paths that have a runtime row of their own, so a field error can be shown there. */
const RUNTIME_ROW_PATHS = new Set(RUNTIME_PATCH_FIELDS.map((field) => field.path));

type Scope = "live" | "restart";
const SCOPE_KEY = "petri.config-panel.scope";

function readScope(): Scope {
	try {
		return window.localStorage.getItem(SCOPE_KEY) === "restart" ? "restart" : "live";
	} catch {
		return "live";
	}
}

function plural(count: number, word: string): string {
	return `${count} ${word}${count === 1 ? "" : "s"}`;
}

/** Is the keyboard event aimed at something the user is typing into? */
function isTyping(target: EventTarget | null): boolean {
	return (
		target instanceof HTMLElement &&
		(target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName))
	);
}

export function ConfigPanel() {
	const localDraft = useConfigStore((s) => s.localDraft);
	const serverConfig = useConfigStore((s) => s.serverConfig);
	const simState = useSimulationStore((s) => s.simState);
	const updateDraft = useConfigStore((s) => s.updateDraft);
	const resetDraft = useConfigStore((s) => s.resetDraft);
	const commitServerConfig = useConfigStore((s) => s.commitServerConfig);

	const startupPreset = useStartupConfigStore((s) => s.preset);
	const updateStartupPreset = useStartupConfigStore((s) => s.updatePreset);
	const addFoodType = useStartupConfigStore((s) => s.addFoodType);
	const removeFoodType = useStartupConfigStore((s) => s.removeFoodType);
	const updateFoodType = useStartupConfigStore((s) => s.updateFoodType);
	const addFertilityLayer = useStartupConfigStore((s) => s.addFertilityLayer);
	const updateFertilityLayerTarget = useStartupConfigStore((s) => s.updateFertilityLayerTarget);
	const randomizeSeed = useStartupConfigStore((s) => s.randomizeSeed);

	const [scope, setScopeState] = useState<Scope>(readScope);
	const [query, setQuery] = useState("");
	const [changedOnly, setChangedOnly] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});
	const [applying, setApplying] = useState(false);
	const [pausing, setPausing] = useState(false);
	const searchInput = useRef<HTMLInputElement>(null);
	const scroller = useRef<HTMLDivElement>(null);
	// Each tab keeps its own place in the shared scroll area.
	const scrollTops = useRef<Record<Scope, number>>({ live: 0, restart: 0 });

	useLayoutEffect(() => {
		if (scroller.current) scroller.current.scrollTop = scrollTops.current[scope];
	}, [scope]);

	const setScope = (next: Scope) => {
		if (scroller.current) scrollTops.current[scope] = scroller.current.scrollTop;
		setScopeState(next);
		try {
			window.localStorage.setItem(SCOPE_KEY, next);
		} catch {
			// The remembered tab is a convenience only.
		}
	};

	const terms = useMemo(() => searchTerms(query), [query]);
	const liveFilter: ConfigFilter = useMemo(() => ({ terms, changedOnly }), [terms, changedOnly]);
	const restartFilter: ConfigFilter = useMemo(() => ({ terms, changedOnly: false }), [terms]);

	const changedCount =
		localDraft && serverConfig
			? RUNTIME_PATCH_FIELDS.filter((field) => isFieldDirty(field, localDraft, serverConfig)).length
			: 0;
	const liveMatches =
		localDraft && serverConfig
			? visibleRuntimeFields(terms, changedOnly, localDraft, serverConfig).length
			: 0;
	const restartMatches = startupMatchCount(terms);
	const restartChanges = startupChangedCount(startupPreset, serverConfig);
	const searching = terms.length > 0;
	const locked = simState === "running";

	// "/" jumps to search from anywhere outside a text field, like most tools.
	useEffect(() => {
		const onKey = (event: KeyboardEvent) => {
			if (event.key !== "/" || event.metaKey || event.ctrlKey || isTyping(event.target)) return;
			event.preventDefault();
			searchInput.current?.focus();
			searchInput.current?.select();
		};
		window.addEventListener("keydown", onKey);
		return () => window.removeEventListener("keydown", onKey);
	}, []);

	// Nothing changed is left to show once the last change is applied or undone.
	useEffect(() => {
		if (changedCount === 0) setChangedOnly(false);
	}, [changedCount]);

	const handleApply = useCallback(async () => {
		// Commit any in-progress number edit, which clamps it to the resolved
		// bounds, before reading the draft this Apply will send.
		if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
		const { localDraft: draft, serverConfig: server } = useConfigStore.getState();
		if (!draft || !server) return;
		setApplying(true);
		setError(null);
		setFieldErrors({});

		try {
			const patch: Record<string, unknown> = {};
			for (const field of RUNTIME_PATCH_FIELDS) {
				if (isFieldDirty(field, draft, server)) {
					mergePatch(
						patch,
						buildPatch(field.path, getByPath(draft, field.path) as number | boolean | string),
					);
				}
			}

			const res = await api.patchConfig(patch as DeepPartial<SimulationConfig>);
			commitServerConfig(res.config, res.state);
		} catch (e) {
			if (e instanceof ApiRequestError) {
				const rowErrors: Record<string, string> = {};
				const unmatched: FieldError[] = [];
				for (const fieldError of e.fieldErrors) {
					if (RUNTIME_ROW_PATHS.has(fieldError.field)) {
						rowErrors[fieldError.field] = fieldError.reason;
					} else {
						unmatched.push(fieldError);
					}
				}
				setFieldErrors(rowErrors);
				setError(describeApiFailure(e.message, unmatched));
			} else {
				setError(e instanceof Error ? e.message : "Config update failed");
			}
		} finally {
			setApplying(false);
		}
	}, [commitServerConfig]);

	const handleReset = useCallback(() => {
		setError(null);
		setFieldErrors({});
		resetDraft();
	}, [resetDraft]);

	const handlePause = useCallback(async () => {
		setPausing(true);
		try {
			const res = await api.pause();
			useSimulationStore.getState().setSimState(res.state);
			useSimulationStore.getState().setTick(res.tick);
		} catch (e) {
			setError(e instanceof Error ? `Pause failed: ${e.message}` : "Pause failed");
		} finally {
			setPausing(false);
		}
	}, []);

	const tab = (id: Scope, label: string, count: number, countLabel: string) => {
		const selected = scope === id;
		const live = id === "live";
		return (
			<button
				type="button"
				role="tab"
				id={`config-tab-${id}`}
				data-testid={`config-tab-${id}`}
				aria-selected={selected}
				aria-controls={`config-tabpanel-${id}`}
				tabIndex={selected ? 0 : -1}
				onClick={() => setScope(id)}
				onKeyDown={(event) => {
					if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
						event.preventDefault();
						const next = live ? "restart" : "live";
						setScope(next);
						document.getElementById(`config-tab-${next}`)?.focus();
					}
				}}
				className={`flex h-7 items-center justify-center gap-1.5 rounded text-xs font-medium transition-colors focus-visible:outline-2 focus-visible:outline-sky-400 ${
					selected
						? "bg-slate-800 text-slate-100 shadow-[inset_0_0_0_1px_rgba(148,163,184,0.18)]"
						: "text-slate-400 hover:text-slate-200"
				}`}
			>
				<span
					aria-hidden="true"
					className={`size-1.5 rounded-full ${live ? "bg-sky-400" : "bg-emerald-400"} ${selected ? "" : "opacity-50"}`}
				/>
				{label}
				{count > 0 && (
					<span
						className={`rounded px-1 font-mono text-[10px] tabular-nums ${
							live ? "bg-sky-400/15 text-sky-300" : "bg-emerald-400/15 text-emerald-300"
						}`}
					>
						<span className="sr-only">{countLabel}: </span>
						{count}
					</span>
				)}
			</button>
		);
	};

	const otherScope: Scope = scope === "live" ? "restart" : "live";
	const otherMatches = scope === "live" ? restartMatches : liveMatches;
	const currentMatches = scope === "live" ? liveMatches : restartMatches;

	return (
		<aside
			data-testid="config-panel"
			aria-label="Simulation settings"
			className="flex min-h-0 w-80 flex-col overflow-hidden border-r border-petri-border bg-petri-panel"
		>
			<div className="flex shrink-0 flex-col gap-2 border-b border-petri-border px-3 pt-3 pb-2">
				<div
					role="tablist"
					aria-label="When settings take effect"
					className="grid grid-cols-2 gap-0.5 rounded-md border border-slate-800 bg-slate-950/50 p-0.5"
				>
					{tab(
						"live",
						"Live",
						searching ? liveMatches : changedCount,
						searching ? "matches" : "unapplied changes",
					)}
					{tab(
						"restart",
						"On Restart",
						searching ? restartMatches : restartChanges,
						searching ? "matches" : "changes at the next Restart",
					)}
				</div>

				<div className="relative">
					<Icon
						name="search"
						className="pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2 text-slate-400"
					/>
					<input
						ref={searchInput}
						type="search"
						data-testid="config-search"
						aria-label="Search settings"
						placeholder="Search settings"
						value={query}
						onChange={(event) => setQuery(event.target.value)}
						onKeyDown={(event) => {
							if (event.key === "Escape") {
								if (query) event.preventDefault();
								setQuery("");
							}
						}}
						className="config-search h-7 w-full rounded border border-slate-700 bg-slate-950/60 pr-7 pl-7 text-xs text-slate-100 placeholder:text-slate-400 hover:border-slate-600 focus:border-sky-500/70 focus:outline-none"
					/>
					{query ? (
						<button
							type="button"
							aria-label="Clear search"
							onClick={() => {
								setQuery("");
								searchInput.current?.focus();
							}}
							className="absolute top-1/2 right-1 flex size-5 -translate-y-1/2 items-center justify-center rounded text-slate-400 before:absolute before:-inset-0.5 hover:bg-slate-800 hover:text-slate-200"
						>
							<Icon name="close" className="size-3" />
						</button>
					) : (
						<kbd className="pointer-events-none absolute top-1/2 right-1.5 -translate-y-1/2 rounded border border-slate-700 px-1 font-mono text-[10px] leading-4 text-slate-400">
							/
						</kbd>
					)}
				</div>

				<div className="flex min-h-5 items-center justify-between gap-2 text-[11px] text-slate-400">
					<span aria-live="polite" className="truncate">
						{searching
							? `${plural(currentMatches, "match")} here`
							: scope === "live"
								? "Applies when you press Apply"
								: "Takes effect at the next Restart"}
					</span>
					{scope === "live" && changedCount > 0 && (
						<label className="flex shrink-0 cursor-pointer items-center gap-1.5 whitespace-nowrap text-slate-400 hover:text-slate-200">
							<input
								type="checkbox"
								role="switch"
								aria-checked={changedOnly}
								data-testid="config-changed-only"
								checked={changedOnly}
								onChange={(event) => setChangedOnly(event.target.checked)}
								className="config-switch config-switch-live config-switch-sm"
							/>
							Changed only
						</label>
					)}
				</div>
			</div>

			{scope === "live" && locked && localDraft && (
				<div
					data-testid="config-locked"
					className="flex shrink-0 items-center gap-2 border-b border-petri-border bg-slate-950/40 px-3 py-2"
				>
					<Icon name="lock" className="size-3.5 text-slate-400" />
					<p className="flex-1 text-[11px] leading-snug text-slate-400">
						Live settings are locked while the world runs.
					</p>
					<button
						type="button"
						data-testid="config-locked-pause"
						onClick={handlePause}
						disabled={pausing}
						className="flex h-6 items-center gap-1 rounded border border-slate-700 px-2 text-[11px] text-slate-200 hover:border-slate-600 hover:bg-slate-800 disabled:opacity-50"
					>
						<Icon name="pause" className="size-3" />
						{pausing ? "Pausing…" : "Pause"}
					</button>
				</div>
			)}

			<div ref={scroller} className="config-scroll min-h-0 flex-1 overflow-y-auto">
				<div
					role="tabpanel"
					id="config-tabpanel-live"
					aria-labelledby="config-tab-live"
					hidden={scope !== "live"}
				>
					<LiveScopeContext value={true}>
						<ConfigFilterContext value={liveFilter}>
							<RuntimeConfigPanel
								localDraft={localDraft}
								serverConfig={serverConfig}
								simState={simState}
								updateDraft={updateDraft}
								fieldErrors={fieldErrors}
							/>
						</ConfigFilterContext>
					</LiveScopeContext>
				</div>
				<div
					role="tabpanel"
					id="config-tabpanel-restart"
					aria-labelledby="config-tab-restart"
					hidden={scope !== "restart"}
				>
					<ConfigFilterContext value={restartFilter}>
						<StartupBaselineContext value={serverConfig}>
							<StartupConfigPanel
								startupPreset={startupPreset}
								updateStartupPreset={updateStartupPreset}
								addFoodType={addFoodType}
								removeFoodType={removeFoodType}
								updateFoodType={updateFoodType}
								addFertilityLayer={addFertilityLayer}
								updateFertilityLayerTarget={updateFertilityLayerTarget}
								randomizeSeed={randomizeSeed}
							/>
						</StartupBaselineContext>
					</ConfigFilterContext>
				</div>

				{searching && currentMatches === 0 && (
					<div className="flex flex-col items-center gap-2 px-6 py-8 text-center">
						<p className="text-xs text-slate-400">
							No {scope === "live" ? "live" : "restart"} settings match “{query.trim()}”.
						</p>
						{otherMatches > 0 && (
							<button
								type="button"
								onClick={() => setScope(otherScope)}
								className="rounded border border-slate-700 px-2 py-1 text-[11px] text-slate-200 hover:border-slate-600 hover:bg-slate-800"
							>
								{plural(otherMatches, "match")} in {otherScope === "live" ? "Live" : "On Restart"}
							</button>
						)}
					</div>
				)}
			</div>

			{scope === "live" && localDraft && serverConfig && (
				<div className="flex shrink-0 flex-col gap-2 border-t border-petri-border px-3 py-2.5">
					{error && (
						<p data-testid="config-error" role="alert" className="text-[11px] text-red-400">
							{error}
						</p>
					)}
					<div className="flex items-center gap-2">
						<span className="flex-1 truncate text-[11px] text-slate-400" aria-live="polite">
							{changedCount > 0 ? `${changedCount} unapplied` : "All applied"}
						</span>
						<button
							type="button"
							data-testid="config-reset"
							disabled={changedCount === 0}
							onClick={handleReset}
							title="Discard every unapplied change"
							className="h-7 rounded px-2.5 text-xs text-slate-300 hover:bg-slate-800 hover:text-slate-100 disabled:opacity-40 disabled:hover:bg-transparent"
						>
							Discard
						</button>
						<button
							type="button"
							data-testid="config-apply"
							disabled={changedCount === 0 || applying}
							onClick={handleApply}
							className="h-7 rounded bg-sky-600 px-3 text-xs font-medium text-white transition-colors hover:bg-sky-500 disabled:cursor-not-allowed disabled:bg-slate-800 disabled:text-slate-500"
						>
							{applying ? "Applying..." : "Apply Changes"}
						</button>
					</div>
				</div>
			)}
		</aside>
	);
}
