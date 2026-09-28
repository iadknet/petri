//! Capability assay lab (T22). Evolves production founders under the
//! production mutation engine and lab truncation selection on fixed arenas,
//! and reports whether a capability reached a calibrated threshold.
//!
//! Lab selection, lab tasks and authored solutions never enter production:
//! nothing here is reachable from `v3-core`, `v3-cli` or `v3-server`.

pub mod arena;
pub mod cli;
pub mod eval;
pub mod rng;
pub mod scene;

use std::fmt;

/// Every recoverable lab failure. The binary maps it to exit code 1.
#[derive(Debug)]
pub enum LabError {
    /// Invalid arguments or overlay/config resolution failure.
    Config(String),
    /// Filesystem or subprocess failure.
    Io(String),
    /// Output would break the telemetry rule (path or size).
    Output(String),
}

impl fmt::Display for LabError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(message) => write!(f, "config: {message}"),
            Self::Io(message) => write!(f, "io: {message}"),
            Self::Output(message) => write!(f, "output: {message}"),
        }
    }
}

impl std::error::Error for LabError {}

impl From<std::io::Error> for LabError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}
