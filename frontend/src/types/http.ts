import type {
	CostsConfig,
	FoodConfig,
	LifecycleEnergyConfig,
	MutationConfig,
	PhenotypeConfig,
	PopulationConfig,
	RuntimeConfig,
	SimulationConfig,
	VmConfig,
	WorldConfig,
} from "./config.ts";
import type { Frame, SimState, StatusPayload } from "./protocol.ts";

export interface LifecycleResponse {
	protocol_version: string;
	state: SimState;
	tick: number;
}

export interface StartupResponse extends LifecycleResponse {
	config_digest: string;
	seeded_creatures: number;
}

export interface StepResponse extends LifecycleResponse {
	steps_applied: number;
}

export interface StatusResponse extends StatusPayload {
	protocol_version: string;
	tick: number;
}

export interface FrameResponse extends Frame {
	protocol_version: string;
	tick: number;
}

export interface ConfigResponse {
	protocol_version: string;
	state: SimState;
	config: SimulationConfig;
}

export interface StartupRequest {
	seed: number;
	population?: Partial<PopulationConfig>;
	world?: Partial<Omit<WorldConfig, "food">> & { food?: Partial<FoodConfig> };
	energy?: { lifecycle?: Partial<LifecycleEnergyConfig>; costs?: Partial<CostsConfig> };
	runtime?: Partial<Omit<RuntimeConfig, "vm">> & {
		vm?: Partial<VmConfig>;
	};
	mutation?: Partial<Omit<MutationConfig, "phenotype">> & {
		phenotype?: Partial<PhenotypeConfig>;
	};
}

export interface StepRequest {
	steps?: number;
}
