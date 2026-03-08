import { useCallback, useMemo, useState } from "react";

export type MeshViewportCommand =
	| {
			kind: "fit" | "reset";
			token: number;
	  }
	| {
			kind: "center-node";
			nodeId: number;
			token: number;
	  };

export interface MeshWorkspaceController {
	searchQuery: string;
	selectedSearchNodeId: number | null;
	viewportCommand: MeshViewportCommand | null;
	setSearchQuery: (query: string) => void;
	setSelectedSearchNodeId: (nodeId: number | null) => void;
	requestFitView: () => void;
	requestResetView: () => void;
	requestCenterNode: (nodeId: number) => void;
}

export function useMeshWorkspaceController(): MeshWorkspaceController {
	const [searchQuery, setSearchQueryState] = useState("");
	const [selectedSearchNodeId, setSelectedSearchNodeId] = useState<
		number | null
	>(null);
	const [viewportCommand, setViewportCommand] =
		useState<MeshViewportCommand | null>(null);

	const requestFitView = useCallback(() => {
		setViewportCommand((current) => ({
			kind: "fit",
			token: (current?.token ?? 0) + 1,
		}));
	}, []);

	const setSearchQuery = useCallback((query: string) => {
		setSearchQueryState(query);
		setSelectedSearchNodeId(null);
	}, []);

	const requestResetView = useCallback(() => {
		setViewportCommand((current) => ({
			kind: "reset",
			token: (current?.token ?? 0) + 1,
		}));
	}, []);

	const requestCenterNode = useCallback((nodeId: number) => {
		setViewportCommand((current) => ({
			kind: "center-node",
			nodeId,
			token: (current?.token ?? 0) + 1,
		}));
	}, []);

	return useMemo(
		() => ({
			searchQuery,
			selectedSearchNodeId,
			viewportCommand,
			setSearchQuery,
			setSelectedSearchNodeId,
			requestFitView,
			requestResetView,
			requestCenterNode,
		}),
		[
			searchQuery,
			selectedSearchNodeId,
			viewportCommand,
			setSearchQuery,
			requestFitView,
			requestResetView,
			requestCenterNode,
		],
	);
}
