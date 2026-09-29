//! `v3-lab`: the capability-assay lab entry point (T22). Logic lives in the
//! library; this shell parses arguments and maps the outcome to an exit code.

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let cli = v3_lab::cli::Cli::parse();
    // Resolved so an invalid value is reported; the lab exports nothing until
    // T21.F06.
    #[cfg(feature = "telemetry")]
    if let Err(message) =
        cli.telemetry_switch(std::env::var(v3_telemetry::SWITCH_ENV).ok().as_deref())
    {
        eprintln!("error: {message}");
        return ExitCode::from(1);
    }
    match v3_lab::cli::execute(cli) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}
