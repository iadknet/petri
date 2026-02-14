pub mod backends;
pub mod ecology;
pub mod energy;
pub mod evolution;
pub mod mesh;
pub mod phenotype;
pub mod runtime;
pub mod telemetry;
pub mod viability;
pub mod world_seed;

pub const V2_RUNTIME_NAME: &str = "petri-v2";

pub fn runtime_name() -> &'static str {
    V2_RUNTIME_NAME
}
