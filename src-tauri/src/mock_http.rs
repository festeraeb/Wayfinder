//! Tiny blocking HTTP/1.1 mock server for tests (std only, no extra crates).
//!
//! Every request is answered by `handler(method, path, body) -> (status, body)`
//! and recorded so tests can check exactly what Wayfinder sent. Responses are
//! JSON with `Connection: close`, one request per connection.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Recorded {
    pub method: String,
    pub path: String,
    pub body: String,
    /// The Authorization header, when the client sent one.
    pub authorization: Option<String>,
}

pub type Handler = Arc<dyn Fn(&str, &str, &str) -> (u16, String) + Send + Sync>;
pub type Log = Arc<Mutex<Vec<Recorded>>>;

/// Start a mock server on 127.0.0.1 with a free port.
/// Returns ("http://127.0.0.1:<port>", request log).
pub fn start(handler: Handler) -> (String, Log) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
    let addr = listener.local_addr().expect("mock server address");
    let log: Log = Arc::new(Mutex::new(Vec::new()));
    let log_in_thread = log.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = match stream {
                Ok(s) => s,
                Err(_) => continue,
            };
            let mut reader = match stream.try_clone() {
                Ok(s) => BufReader::new(s),
                Err(_) => continue,
            };
            let mut request_line = String::new();
            if reader.read_line(&mut request_line).is_err() {
                continue;
            }
            let mut parts = request_line.split_whitespace();
            let method = parts.next().unwrap_or("").to_string();
            let path = parts.next().unwrap_or("").to_string();
            let mut content_length = 0usize;
            let mut authorization = None;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
                let trimmed = line.trim_end();
                if trimmed.is_empty() {
                    break;
                }
                if let Some((name, value)) = trimmed.split_once(':') {
                    if name.trim().eq_ignore_ascii_case("content-length") {
                        content_length = value.trim().parse().unwrap_or(0);
                    } else if name.trim().eq_ignore_ascii_case("authorization") {
                        authorization = Some(value.trim().to_string());
                    }
                }
            }
            let mut body = vec![0u8; content_length];
            if reader.read_exact(&mut body).is_err() {
                continue;
            }
            let body = String::from_utf8_lossy(&body).to_string();
            let (status, reply) = handler(&method, &path, &body);
            log_in_thread
                .lock()
                .unwrap()
                .push(Recorded { method, path, body, authorization });
            let response = format!(
                "HTTP/1.1 {} MOCK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                status,
                reply.len(),
                reply
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });
    (format!("http://{}", addr), log)
}

/// A base URL on a port that refuses connections (bound, then released).
pub fn closed_base() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("address");
    drop(listener);
    format!("http://{}", addr)
}

/// Snapshot of the request log.
pub fn requests(log: &Log) -> Vec<Recorded> {
    log.lock().unwrap().clone()
}
