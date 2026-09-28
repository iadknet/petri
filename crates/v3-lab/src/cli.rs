//! Command-line interface (stub; filled in below).

use crate::LabError;

#[derive(clap::Parser, Debug)]
pub struct Cli {}

/// Run the parsed command and return the process exit code.
///
/// # Errors
///
/// Any [`LabError`].
pub fn execute(_cli: Cli) -> Result<u8, LabError> {
    Ok(0)
}
