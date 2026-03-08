import { create } from "zustand";

export type SampleSessionStatus = "idle" | "recording";

interface SampleSessionState {
	status: SampleSessionStatus;
	creatureId: number | null;
	error: string | null;
	setRecording: (creatureId: number) => void;
	clear: () => void;
	setError: (error: string) => void;
}

export const useSampleSessionStore = create<SampleSessionState>()((set) => ({
	status: "idle",
	creatureId: null,
	error: null,

	setRecording: (creatureId) =>
		set({
			status: "recording",
			creatureId,
			error: null,
		}),

	clear: () =>
		set({
			status: "idle",
			creatureId: null,
			error: null,
		}),

	setError: (error) =>
		set({
			status: "idle",
			creatureId: null,
			error,
		}),
}));
