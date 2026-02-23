import { type ReactNode, createContext, use, useCallback, useState } from "react";

export interface PanelLayout {
	configOpen: boolean;
	statsOpen: boolean;
	toggleConfig: () => void;
	toggleStats: () => void;
}

const PanelLayoutContext = createContext<PanelLayout | null>(null);

export function PanelLayoutProvider({ children }: { children: ReactNode }) {
	const [configOpen, setConfigOpen] = useState(true);
	const [statsOpen, setStatsOpen] = useState(true);

	const toggleConfig = useCallback(() => setConfigOpen((v) => !v), []);
	const toggleStats = useCallback(() => setStatsOpen((v) => !v), []);

	return (
		<PanelLayoutContext value={{ configOpen, statsOpen, toggleConfig, toggleStats }}>
			{children}
		</PanelLayoutContext>
	);
}

export function usePanelLayout(): PanelLayout {
	const ctx = use(PanelLayoutContext);
	if (!ctx) throw new Error("usePanelLayout must be inside PanelLayoutProvider");
	return ctx;
}
