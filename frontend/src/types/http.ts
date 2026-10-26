import type { EnergyCostsConfig } from "./generated/EnergyCostsConfig.ts";
import type { EnergyLifecycleConfig } from "./generated/EnergyLifecycleConfig.ts";
import type { FoodConfig } from "./generated/FoodConfig.ts";
import type { MutationConfig } from "./generated/MutationConfig.ts";
import type { PhenotypeConfig } from "./generated/PhenotypeConfig.ts";
import type { PopulationConfig } from "./generated/PopulationConfig.ts";
import type { RuntimeConfig } from "./generated/RuntimeConfig.ts";
import type { VmRuntimeConfig } from "./generated/VmRuntimeConfig.ts";
import type { WorldConfig } from "./generated/WorldConfig.ts";

/** The `POST /v3/simulation/startup` body: a seed and a partial config the
 * server merges over its baseline. */
export interface StartupRequest {
	seed: number;
	population?: Partial<PopulationConfig>;
	world?: Partial<Omit<WorldConfig, "food">> & { food?: Partial<FoodConfig> };
	energy?: { lifecycle?: Partial<EnergyLifecycleConfig>; costs?: Partial<EnergyCostsConfig> };
	runtime?: Partial<Omit<RuntimeConfig, "vm">> & {
		vm?: Partial<VmRuntimeConfig>;
	};
	mutation?: Partial<Omit<MutationConfig, "phenotype">> & {
		phenotype?: Partial<PhenotypeConfig>;
	};
}
