import type {
	CostsConfig,
	FailedActionPenaltyRampConfig,
	FoodConfig,
	LifecycleEnergyConfig,
	MutationConfig,
	PhenotypeConfig,
	PopulationConfig,
	RuntimeConfig,
	SimulationConfig,
	TopologyNewNodeBirthConfig,
	VmConfig,
	WorldConfig,
} from "./config.ts";
import type {
	HealthPayload,
	SimState,
	SnapshotView,
	StatusPayload,
	WorldStaticPayload,
} from "./protocol.ts";

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

export interface SnapshotResponse {
	protocol_version: string;
	projection_revision: number;
	world_static_revision: number;
	tick: number;
	status: StatusPayload;
	health: HealthPayload;
	world_static: WorldStaticPayload;
	view: SnapshotView;
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
	mutation?: Partial<Omit<MutationConfig, "phenotype" | "topology_new_node_birth">> & {
		phenotype?: Partial<PhenotypeConfig>;
		topology_new_node_birth?: Partial<TopologyNewNodeBirthConfig>;
	};
	startup?: {
		ramps?: {
			failed_action_penalty?: Partial<FailedActionPenaltyRampConfig>;
		};
	};
}

export interface StepRequest {
	steps?: number;
}
