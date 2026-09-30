//! The OTLP/HTTP exporter and the rejections it reports.
//!
//! The pinned `opentelemetry-otlp` exporter reads a partial-success response
//! only to log it and reports the request as exported. The HTTP client here
//! decodes that response itself and adds its `rejected_log_records` to a
//! counter the queue worker reads after each export, so rejected records are
//! counted as failed.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use opentelemetry_http::{Bytes, HttpClient, HttpError, Request, Response};
use opentelemetry_otlp::{
    ExporterBuildError, LogExporter, Protocol, RetryPolicy, WithExportConfig, WithHttpConfig,
};
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceResponse;
use prost::Message;

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
            let rejected = ExportLogsServiceResponse::decode(response.body().as_ref())
                .ok()
                .and_then(|decoded| decoded.partial_success)
                .map_or(0, |partial| {
                    u64::try_from(partial.rejected_log_records).unwrap_or(0)
                });
            self.rejections.add(rejected);
        }
        Ok(response)
    }
}

/// Builds the OTLP/HTTP protobuf log exporter for `url`, with no retry.
pub(crate) fn build(
    url: String,
    timeout: Duration,
    rejections: Arc<Rejections>,
) -> Result<LogExporter, ExporterBuildError> {
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
    LogExporter::builder()
        .with_http()
        .with_http_client(CountingClient { inner, rejections })
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(url)
        .with_timeout(timeout)
        .with_retry_policy(RetryPolicy::disabled())
        .build()
}
