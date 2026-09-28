//! `v3-lab`: the capability-assay lab entry point (T22). Logic lives in the
//! library; this shell parses arguments and maps the outcome to an exit code.

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let cli = v3_lab::cli::Cli::parse();
    match v3_lab::cli::execute(cli) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}
