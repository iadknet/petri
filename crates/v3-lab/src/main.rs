//! `v3-lab`: the capability-assay lab entry point (T22). Logic lives in the
//! library; this shell parses arguments, maps the outcome to an exit code,
//! and holds the lab's only telemetry calls (T21.F06): `run` and `why-not`
//! with telemetry on record one measurement, exported after the run ends.

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let cli = v3_lab::cli::Cli::parse();
    #[cfg(feature = "telemetry")]
    let measurement = match record::begin(&cli) {
        Ok(measurement) => measurement,
        Err(message) => {
            eprintln!("error: {message}");
            return ExitCode::from(1);
        }
    };
    let result = v3_lab::cli::execute(cli);
    let code = match &result {
        Ok(executed) => executed.exit_code,
        Err(error) => {
            eprintln!("error: {error}");
            1
        }
    };
    #[cfg(feature = "telemetry")]
    record::finish(
        measurement,
        code,
        result
            .ok()
            .and_then(|executed| executed.measurement)
            .as_ref(),
    );
    ExitCode::from(code)
}

/// The lab's measurement records: the library hands back plain data, and
/// only this module touches `v3_telemetry`.
#[cfg(feature = "telemetry")]
mod record {
    use v3_lab::cli::{Cli, Measurement};
    use v3_telemetry::{MeasurementEnd, MeasurementStart, Service, Telemetry};

    /// Held telemetry and `measurement.started` for `run` and `why-not`;
    /// `report` records nothing. An invalid setting refuses to start.
    pub(super) fn begin(
        cli: &Cli,
    ) -> Result<Option<(Telemetry, v3_telemetry::MeasurementHandle)>, String> {
        let switch = v3_telemetry::switch_from_env(cli.telemetry)?;
        let Some((command, args)) = cli.command.measured() else {
            return Ok(None);
        };
        let telemetry = Telemetry::start_held(Service::Lab, switch)?;
        let handle = telemetry.begin_measurement(MeasurementStart {
            command,
            assay: Some(args.assay.name()),
            seed: Some(args.seed),
            threads: args.threads.map(|threads| threads as u64),
            ..MeasurementStart::default()
        });
        Ok(handle.map(|handle| (telemetry, handle)))
    }

    /// `measurement.ended` (with the summary's readings when the run wrote
    /// one), then release and the bounded flush.
    pub(super) fn finish(
        measurement: Option<(Telemetry, v3_telemetry::MeasurementHandle)>,
        code: u8,
        written: Option<&Measurement>,
    ) {
        let Some((telemetry, handle)) = measurement else {
            return;
        };
        let path = written.map(|written| written.summary_path.display().to_string());
        telemetry.end_measurement(
            handle,
            MeasurementEnd {
                exit_code: code,
                incomplete: written.map(|written| written.incomplete.is_some()),
                summary_path: path.as_deref(),
                body: written.map(Measurement::body),
                ..MeasurementEnd::default()
            },
        );
        telemetry.release();
        telemetry.shutdown();
    }
}
