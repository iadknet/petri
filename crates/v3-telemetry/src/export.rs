//! The OTLP/HTTP exporter and the rejections it reports.
//!
//! The pinned `opentelemetry-otlp` exporter reads a partial-success response
//! only to log it and reports the request as exported. The HTTP client here
//! decodes that response itself and adds its `rejected_log_records` to a
//! counter the queue worker reads after each export, so rejected records are
//! counted as failed. Run snapshots (T21.F02) are posted to `/v1/metrics` and
//! tick traces (T21.F03) to `/v1/traces` by [`EncodedClient`] on the same HTTP
//! client, which reads each response's rejected data points or spans itself. A success response whose body does
//! not decode confirms nothing, so its request counts as failed; an empty body
//! decodes to full acceptance.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use opentelemetry_http::{Bytes, HttpClient, HttpError, Request, Response};
use opentelemetry_otlp::{
    ExporterBuildError, LogExporter, Protocol, RetryPolicy, WithExportConfig, WithHttpConfig,
};
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceResponse;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceResponse;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceResponse;
use prost::Message;

use crate::queue::{Outcome, PostEncoded, Signal};

/// Records the collector rejected since the worker last took the count.
#[derive(Debug, Default)]
pub(crate) struct Rejections(AtomicU64);

impl Rejections {
    fn add(&self, count: u64) {
        self.0.fetch_add(count, Ordering::Relaxed);
    }

    /// The count since the last take; one worker exports, so it belongs to
    /// the batch that just returned.
    pub(crate) fn take(&self) -> u64 {
        self.0.swap(0, Ordering::Relaxed)
    }
}

/// A partial success's rejected count; a negative count is zero.
fn count(rejected: i64) -> u64 {
    u64::try_from(rejected).unwrap_or(0)
}

/// The blocking reqwest client, plus the rejected count of each successful
/// response.
#[derive(Debug)]
struct CountingClient {
    inner: reqwest::blocking::Client,
    rejections: Arc<Rejections>,
}

#[async_trait::async_trait]
impl HttpClient for CountingClient {
    async fn send_bytes(&self, request: Request<Bytes>) -> Result<Response<Bytes>, HttpError> {
        let response = self.inner.send_bytes(request).await?;
        if response.status().is_success() {
            let decoded = ExportLogsServiceResponse::decode(response.body().as_ref())?;
            self.rejections.add(
                decoded
                    .partial_success
                    .map_or(0, |partial| count(partial.rejected_log_records)),
            );
        }
        Ok(response)
    }
}

/// Posts `body` to `url` with no retry; a success whose body decodes as `R`
/// is exported with the count `rejected` reads from it.
fn post_proto<R: Message + Default>(
    client: &reqwest::blocking::Client,
    url: &str,
    body: Vec<u8>,
    rejected: impl FnOnce(R) -> u64,
) -> Outcome {
    let response = client
        .post(url)
        .header("content-type", "application/x-protobuf")
        .body(body)
        .send();
    let Ok(response) = response else {
        return Outcome::Failed;
    };
    if !response.status().is_success() {
        return Outcome::Failed;
    }
    let Ok(bytes) = response.bytes() else {
        return Outcome::Failed;
    };
    match R::decode(bytes.as_ref()) {
        Ok(decoded) => Outcome::Exported {
            rejected: rejected(decoded),
        },
        Err(_) => Outcome::Failed,
    }
}

/// Posts the requests the worker encodes itself, with no retry: snapshots to
/// the collector's `/v1/metrics`, and tick traces and creature windows to
/// its `/v1/traces`.
#[derive(Debug)]
pub(crate) struct EncodedClient {
    client: reqwest::blocking::Client,
    metrics_url: String,
    traces_url: String,
}

impl PostEncoded for EncodedClient {
    fn post(&self, signal: Signal, body: Vec<u8>) -> Outcome {
        match signal {
            Signal::Metrics => post_proto(
                &self.client,
                &self.metrics_url,
                body,
                |decoded: ExportMetricsServiceResponse| {
                    decoded
                        .partial_success
                        .map_or(0, |partial| count(partial.rejected_data_points))
                },
            ),
            Signal::Traces | Signal::Windows => post_proto(
                &self.client,
                &self.traces_url,
                body,
                |decoded: ExportTraceServiceResponse| {
                    decoded
                        .partial_success
                        .map_or(0, |partial| count(partial.rejected_spans))
                },
            ),
        }
    }
}

/// Builds the OTLP/HTTP protobuf log exporter for `<endpoint>/v1/logs` and the
/// clients for `<endpoint>/v1/metrics` and `<endpoint>/v1/traces` on one HTTP
/// client, with no retry.
pub(crate) fn build(
    endpoint: &str,
    timeout: Duration,
    rejections: Arc<Rejections>,
) -> Result<(LogExporter, EncodedClient), ExporterBuildError> {
    let base = endpoint.trim_end_matches('/');
    // reqwest's blocking client starts and stops its own runtime, which
    // panics on a thread that is already inside one (the server's main).
    let inner = std::thread::spawn(move || {
        reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()
    })
    .join()
    .map_err(|_| ExporterBuildError::InternalFailure("HTTP client build panicked".to_owned()))?
    .map_err(|error| ExporterBuildError::InternalFailure(error.to_string()))?;
    let encoded = EncodedClient {
        client: inner.clone(),
        metrics_url: format!("{base}/v1/metrics"),
        traces_url: format!("{base}/v1/traces"),
    };
    let url = format!("{base}/v1/logs");
    let logs = LogExporter::builder()
        .with_http()
        .with_http_client(CountingClient { inner, rejections })
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(url)
        .with_timeout(timeout)
        .with_retry_policy(RetryPolicy::disabled())
        .build()?;
    Ok((logs, encoded))
}
