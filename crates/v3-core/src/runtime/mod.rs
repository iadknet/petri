pub mod action_decode;
pub(crate) mod cgp;
pub mod inputs;
pub mod mesh;
pub(crate) mod plasticity;
pub(crate) mod routing;
pub mod trace;
pub mod traced_mesh;
pub mod traced_vm;
pub mod types;
pub mod vm;
pub use mesh::execute_creature_mesh;
pub use types::{sanitize_f32, MeshOutput, MeshSideOutputs, OUTPUT_SLOT_COUNT};

#[cfg(test)]
mod f15_tests;

#[cfg(test)]
mod cognition_tests;
