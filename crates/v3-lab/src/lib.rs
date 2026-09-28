//! Capability assay lab (T22). Evolves production founders under the
//! production mutation engine and lab truncation selection on fixed arenas,
//! and reports whether a capability reached a calibrated threshold.
//!
//! Lab selection, lab tasks and authored solutions never enter production:
//! nothing here is reachable from `v3-core`, `v3-cli` or `v3-server`.

pub mod arena;
pub mod calibration;
pub mod campaign;
pub mod cli;
pub mod eval;
pub mod output;
pub mod rng;
pub mod run;
pub mod scene;
pub mod stats;
pub mod summary;

use std::fmt;
use std::path::Path;

use sha2::{Digest, Sha256};
use v3_core::creature::genome::CreatureGenome;

/// The lab's genome file (`--genome`, `--comparator`, `--arm …:genome.json`
/// and `elites/*.json`). `v3_core_version` is the workspace version the
/// genome was written under (every crate shares it).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenomeFile {
    pub genome_format: u32,
    pub v3_core_version: String,
    pub genome: CreatureGenome,
}

impl GenomeFile {
    #[must_use]
    pub fn new(genome: CreatureGenome) -> Self {
        Self {
            genome_format: summary::GENOME_FORMAT,
            v3_core_version: env!("CARGO_PKG_VERSION").to_owned(),
            genome,
        }
    }

    /// Read a genome file.
    ///
    /// # Errors
    ///
    /// I/O failures, malformed JSON, or an unknown `genome_format`.
    pub fn load(path: &Path) -> Result<CreatureGenome, LabError> {
        let file: Self = read_json(path)?;
        if file.genome_format != summary::GENOME_FORMAT {
            return Err(LabError::Config(format!(
                "{}: genome_format {} is not {}",
                path.display(),
                file.genome_format,
                summary::GENOME_FORMAT
            )));
        }
        Ok(file.genome)
    }
}

/// Read and parse a JSON file, naming `path` in the error.
///
/// # Errors
///
/// [`LabError::Io`] when unreadable, [`LabError::Config`] when malformed.
pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, LabError> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| LabError::Io(format!("{}: {error}", path.display())))?;
    serde_json::from_str(&text)
        .map_err(|error| LabError::Config(format!("{}: {error}", path.display())))
}

/// Lower-case hex SHA-256 of `bytes`.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut hex, byte| {
            use std::fmt::Write as _;
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

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
