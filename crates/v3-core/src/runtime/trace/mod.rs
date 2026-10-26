//! Runtime's brain-execution trace:
//! - `domain`: the passive sampler records of one tick's execution
//! - `size`: those records' in-memory sizes, for recording budgets

pub mod domain;
#[cfg(feature = "telemetry-seams")]
pub mod size;
