//! An in-process OTLP/HTTP log receiver for tests, on an ephemeral port.
//!
//! It decodes each `POST /v1/logs` protobuf body and keeps the records, or, in
//! [`Receiver::hanging`] mode, accepts the connection and never answers.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::common::v1::{any_value, AnyValue, KeyValue};
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

/// A test OTLP receiver. Its threads live until the test process exits.
#[derive(Debug)]
pub struct Receiver {
    endpoint: String,
    records: Arc<Mutex<Vec<ReceivedRecord>>>,
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

/// Reads one HTTP/1.1 request body; `None` at end of stream.
fn read_request(reader: &mut BufReader<TcpStream>) -> Option<Vec<u8>> {
    let mut length = 0_usize;
    let mut saw_line = false;
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
        saw_line = true;
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                length = value.trim().parse().ok()?;
            }
        }
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).ok()?;
    Some(body)
}

fn serve(stream: TcpStream, records: &Mutex<Vec<ReceivedRecord>>, answer: bool) {
    let Ok(mut writer) = stream.try_clone() else {
        return;
    };
    let mut reader = BufReader::new(stream);
    while let Some(body) = read_request(&mut reader) {
        if !answer {
            // Hold the connection open and never reply.
            loop {
                std::thread::park();
            }
        }
        records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .extend(decode(&body));
        let reply =
            "HTTP/1.1 200 OK\r\ncontent-type: application/x-protobuf\r\ncontent-length: 0\r\n\r\n";
        if writer.write_all(reply.as_bytes()).is_err() {
            return;
        }
    }
}

impl Receiver {
    /// A receiver that answers every request with `200 OK`.
    pub fn start() -> Self {
        Self::spawn(true)
    }

    /// A receiver that accepts connections and never answers.
    pub fn hanging() -> Self {
        Self::spawn(false)
    }

    fn spawn(answer: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
        let endpoint = format!(
            "http://{}",
            listener.local_addr().expect("listener has an address")
        );
        let records: Arc<Mutex<Vec<ReceivedRecord>>> = Arc::default();
        let shared = Arc::clone(&records);
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let shared = Arc::clone(&shared);
                std::thread::spawn(move || serve(stream, &shared, answer));
            }
        });
        Self { endpoint, records }
    }

    /// The OTLP base endpoint (no `/v1/logs`).
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn records(&self) -> Vec<ReceivedRecord> {
        self.records
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
