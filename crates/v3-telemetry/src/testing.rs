//! An in-process OTLP/HTTP receiver for tests, on an ephemeral port.
//!
//! It decodes each `POST /v1/logs` protobuf body and keeps the records, and
//! each `POST /v1/metrics` body and keeps the snapshot; in
//! [`Receiver::rejecting`] mode it answers with a partial success that rejects
//! records or data points, and in [`Receiver::hanging`] mode it accepts the
//! connection and never answers.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use opentelemetry_proto::tonic::collector::logs::v1::{
    ExportLogsPartialSuccess, ExportLogsServiceRequest, ExportLogsServiceResponse,
};
use opentelemetry_proto::tonic::collector::metrics::v1::{
    ExportMetricsPartialSuccess, ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use opentelemetry_proto::tonic::common::v1::{any_value, AnyValue, KeyValue};
use opentelemetry_proto::tonic::metrics::v1::{metric, number_data_point, NumberDataPoint};
use prost::Message;

/// One decoded log record with its resource, values rendered as strings.
#[derive(Debug, Clone, PartialEq)]
pub struct ReceivedRecord {
    pub resource: BTreeMap<String, String>,
    pub event_name: String,
    pub attributes: BTreeMap<String, String>,
    pub body: Option<String>,
}

impl ReceivedRecord {
    pub fn attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(String::as_str)
    }
}

/// How a received metric family is exported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricKind {
    Gauge,
    MonotonicSum,
    /// A non-monotonic cumulative sum.
    Sum,
}

/// A data point's value as sent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointValue {
    Int(i64),
    Double(f64),
}

impl PointValue {
    pub fn as_f64(self) -> f64 {
        match self {
            Self::Int(count) => count as f64,
            Self::Double(amount) => amount,
        }
    }
}

/// One decoded data point with its family's name, kind and metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct ReceivedPoint {
    pub name: String,
    pub description: String,
    pub unit: String,
    pub kind: MetricKind,
    pub attributes: BTreeMap<String, String>,
    pub value: PointValue,
    pub time_unix_nano: u64,
    pub start_time_unix_nano: u64,
}

impl ReceivedPoint {
    pub fn attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(String::as_str)
    }
}

/// One decoded `POST /v1/metrics` request: a run snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct ReceivedSnapshot {
    pub resource: BTreeMap<String, String>,
    pub points: Vec<ReceivedPoint>,
}

impl ReceivedSnapshot {
    /// The run the snapshot's data points name.
    pub fn run_id(&self) -> Option<&str> {
        self.points.first()?.attribute("petri.run_id")
    }

    /// The points of family `name`.
    pub fn family<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a ReceivedPoint> + 'a {
        self.points.iter().filter(move |point| point.name == name)
    }

    /// The value of the one point of family `name` whose attributes, other
    /// than `petri.run_id`, are exactly `labels`.
    pub fn value(&self, name: &str, labels: &[(&str, &str)]) -> Option<PointValue> {
        self.family(name)
            .find(|point| {
                point.attributes.len() == labels.len() + 1
                    && labels
                        .iter()
                        .all(|(key, value)| point.attribute(key) == Some(value))
            })
            .map(|point| point.value)
    }

    /// The `petri.run.tick` gauge.
    pub fn tick(&self) -> Option<u64> {
        match self.value("petri.run.tick", &[])? {
            PointValue::Int(tick) => u64::try_from(tick).ok(),
            PointValue::Double(_) => None,
        }
    }

    /// The stamp every point carries.
    pub fn time_unix_nano(&self) -> Option<u64> {
        Some(self.points.first()?.time_unix_nano)
    }
}

/// A test OTLP receiver. Its threads live until the test process exits.
#[derive(Debug)]
pub struct Receiver {
    endpoint: String,
    records: Arc<Mutex<Vec<ReceivedRecord>>>,
    snapshots: Arc<Mutex<Vec<ReceivedSnapshot>>>,
}

fn render(value: Option<&AnyValue>) -> Option<String> {
    Some(match value?.value.as_ref()? {
        any_value::Value::StringValue(text) => text.clone(),
        any_value::Value::BoolValue(flag) => flag.to_string(),
        any_value::Value::IntValue(number) => number.to_string(),
        any_value::Value::DoubleValue(number) => number.to_string(),
        other => format!("{other:?}"),
    })
}

fn to_map(pairs: &[KeyValue]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .filter_map(|pair| Some((pair.key.clone(), render(pair.value.as_ref())?)))
        .collect()
}

fn decode(body: &[u8]) -> Vec<ReceivedRecord> {
    let Ok(request) = ExportLogsServiceRequest::decode(body) else {
        return Vec::new();
    };
    let mut records = Vec::new();
    for resource_logs in request.resource_logs {
        let resource = resource_logs
            .resource
            .map(|resource| to_map(&resource.attributes))
            .unwrap_or_default();
        for scope_logs in resource_logs.scope_logs {
            for record in scope_logs.log_records {
                records.push(ReceivedRecord {
                    resource: resource.clone(),
                    event_name: record.event_name.clone(),
                    attributes: to_map(&record.attributes),
                    body: render(record.body.as_ref()),
                });
            }
        }
    }
    records
}

fn point(
    name: &str,
    description: &str,
    unit: &str,
    kind: MetricKind,
    data_point: NumberDataPoint,
) -> ReceivedPoint {
    ReceivedPoint {
        name: name.to_owned(),
        description: description.to_owned(),
        unit: unit.to_owned(),
        kind,
        attributes: to_map(&data_point.attributes),
        value: match data_point.value {
            Some(number_data_point::Value::AsInt(count)) => PointValue::Int(count),
            Some(number_data_point::Value::AsDouble(amount)) => PointValue::Double(amount),
            None => PointValue::Double(f64::NAN),
        },
        time_unix_nano: data_point.time_unix_nano,
        start_time_unix_nano: data_point.start_time_unix_nano,
    }
}

/// Decodes an OTLP metrics request body into its snapshots.
pub fn decode_metrics(body: &[u8]) -> Vec<ReceivedSnapshot> {
    let Ok(request) = ExportMetricsServiceRequest::decode(body) else {
        return Vec::new();
    };
    let mut snapshots = Vec::new();
    for resource_metrics in request.resource_metrics {
        let resource = resource_metrics
            .resource
            .map(|resource| to_map(&resource.attributes))
            .unwrap_or_default();
        let mut points = Vec::new();
        for scope_metrics in resource_metrics.scope_metrics {
            for family in scope_metrics.metrics {
                let (kind, data_points) = match family.data {
                    Some(metric::Data::Gauge(gauge)) => (MetricKind::Gauge, gauge.data_points),
                    Some(metric::Data::Sum(sum)) if sum.is_monotonic => {
                        (MetricKind::MonotonicSum, sum.data_points)
                    }
                    Some(metric::Data::Sum(sum)) => (MetricKind::Sum, sum.data_points),
                    _ => continue,
                };
                points.extend(data_points.into_iter().map(|data_point| {
                    point(
                        &family.name,
                        &family.description,
                        &family.unit,
                        kind,
                        data_point,
                    )
                }));
            }
        }
        snapshots.push(ReceivedSnapshot { resource, points });
    }
    snapshots
}

/// Reads one HTTP/1.1 request's path and body; `None` at end of stream.
fn read_request(reader: &mut BufReader<TcpStream>) -> Option<(String, Vec<u8>)> {
    let mut length = 0_usize;
    let mut saw_line = false;
    let mut path = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            if saw_line {
                break;
            }
            continue;
        }
        if !saw_line {
            path = line.split(' ').nth(1).unwrap_or_default().to_owned();
        }
        saw_line = true;
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                length = value.trim().parse().ok()?;
            }
        }
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).ok()?;
    Some((path, body))
}

/// How a receiver answers each request.
#[derive(Debug, Clone, Copy)]
enum Answer {
    /// `200 OK` with an empty body: every record accepted.
    Accept,
    /// `200 OK` with a partial success rejecting up to this many records, or
    /// data points of a snapshot, which is then not kept.
    Reject(u64),
    /// Never answer.
    Hang,
}

/// Keeps a metrics request's snapshots unless points are rejected; returns
/// the reply body.
fn receive_metrics(
    body: &[u8],
    snapshots: &Mutex<Vec<ReceivedSnapshot>>,
    answer: Answer,
) -> Vec<u8> {
    let decoded = decode_metrics(body);
    let points: usize = decoded.iter().map(|snapshot| snapshot.points.len()).sum();
    let rejected = match answer {
        Answer::Reject(limit) => limit.min(points as u64),
        Answer::Accept | Answer::Hang => 0,
    };
    if rejected == 0 {
        snapshots
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .extend(decoded);
        return Vec::new();
    }
    ExportMetricsServiceResponse {
        partial_success: Some(ExportMetricsPartialSuccess {
            rejected_data_points: rejected as i64,
            error_message: "rejected by the test receiver".to_owned(),
        }),
    }
    .encode_to_vec()
}

fn serve(
    stream: TcpStream,
    records: &Mutex<Vec<ReceivedRecord>>,
    snapshots: &Mutex<Vec<ReceivedSnapshot>>,
    answer: Answer,
) {
    let Ok(mut writer) = stream.try_clone() else {
        return;
    };
    let mut reader = BufReader::new(stream);
    while let Some((path, body)) = read_request(&mut reader) {
        if let Answer::Hang = answer {
            loop {
                // Hold the connection open and never reply.
                std::thread::park();
            }
        }
        let reply = if path.ends_with("/v1/metrics") {
            receive_metrics(&body, snapshots, answer)
        } else {
            receive_logs(&body, records, answer)
        };
        let head = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/x-protobuf\r\ncontent-length: {}\r\n\r\n",
            reply.len()
        );
        if writer.write_all(head.as_bytes()).is_err() || writer.write_all(&reply).is_err() {
            return;
        }
    }
}

/// Keeps a logs request's records but the rejected ones; returns the reply
/// body.
fn receive_logs(body: &[u8], records: &Mutex<Vec<ReceivedRecord>>, answer: Answer) -> Vec<u8> {
    let decoded = decode(body);
    let rejected = match answer {
        Answer::Reject(limit) => limit.min(decoded.len() as u64),
        Answer::Accept | Answer::Hang => 0,
    };
    let accepted = decoded.len() - rejected as usize;
    records
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .extend(decoded.into_iter().take(accepted));
    if rejected == 0 {
        return Vec::new();
    }
    ExportLogsServiceResponse {
        partial_success: Some(ExportLogsPartialSuccess {
            rejected_log_records: rejected as i64,
            error_message: "rejected by the test receiver".to_owned(),
        }),
    }
    .encode_to_vec()
}

impl Receiver {
    /// A receiver that answers every request with `200 OK`.
    pub fn start() -> Self {
        Self::spawn(Answer::Accept)
    }

    /// A receiver that keeps all but the last `per_request` records of each
    /// logs request and reports those as rejected in a partial success; a
    /// metrics request has up to `per_request` data points rejected and its
    /// snapshot is not kept.
    pub fn rejecting(per_request: u64) -> Self {
        Self::spawn(Answer::Reject(per_request))
    }

    /// A receiver that accepts connections and never answers.
    pub fn hanging() -> Self {
        Self::spawn(Answer::Hang)
    }

    fn spawn(answer: Answer) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
        let endpoint = format!(
            "http://{}",
            listener.local_addr().expect("listener has an address")
        );
        let records: Arc<Mutex<Vec<ReceivedRecord>>> = Arc::default();
        let snapshots: Arc<Mutex<Vec<ReceivedSnapshot>>> = Arc::default();
        let (shared_records, shared_snapshots) = (Arc::clone(&records), Arc::clone(&snapshots));
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let records = Arc::clone(&shared_records);
                let snapshots = Arc::clone(&shared_snapshots);
                std::thread::spawn(move || serve(stream, &records, &snapshots, answer));
            }
        });
        Self {
            endpoint,
            records,
            snapshots,
        }
    }

    /// The OTLP base endpoint (no `/v1/logs` or `/v1/metrics`).
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn records(&self) -> Vec<ReceivedRecord> {
        self.records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// The snapshots received and kept, in arrival order.
    pub fn snapshots(&self) -> Vec<ReceivedSnapshot> {
        self.snapshots
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Polls until `done` holds for the received records or `timeout` passes.
    pub fn wait_until(
        &self,
        timeout: Duration,
        done: impl Fn(&[ReceivedRecord]) -> bool,
    ) -> Vec<ReceivedRecord> {
        let deadline = Instant::now() + timeout;
        loop {
            let records = self.records();
            if done(&records) || Instant::now() >= deadline {
                return records;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

/// An endpoint on a port nothing listens on: bound, then released.
pub fn closed_endpoint() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
    let address = listener.local_addr().expect("listener has an address");
    drop(listener);
    format!("http://{address}")
}

/// The fields of a `telemetry: run=…` self-report line, by name.
pub fn parse_run_line(line: &str) -> Option<BTreeMap<String, String>> {
    let rest = line.strip_prefix("telemetry: run=")?;
    let mut fields = BTreeMap::new();
    let mut parts = rest.split(' ');
    fields.insert("run".to_owned(), parts.next()?.to_owned());
    for part in parts {
        let (key, value) = part.split_once('=')?;
        fields.insert(key.to_owned(), value.to_owned());
    }
    Some(fields)
}
