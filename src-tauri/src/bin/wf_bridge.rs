//! Test bridge for Playwright on machines without a webview driver (build with `--features bridge`).
//!
//! Serves the built front end (WF_DIST, default ../dist) and forwards `POST /invoke/<command>`
//! (JSON body = the invoke arguments) to Wayfinder's real command handlers through Tauri's mock
//! runtime. The Rust side under test is the real code; only the webview shell is absent.
//!
//!   wf-bridge [port]        # default 14321, listens on 127.0.0.1 only

use std::path::PathBuf;
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::webview::InvokeRequest;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

type Webview = tauri::WebviewWindow<tauri::test::MockRuntime>;

fn invoke(webview: &Webview, cmd: &str, args: serde_json::Value) -> (u16, String) {
    let req = InvokeRequest {
        cmd: cmd.to_string(),
        callback: CallbackFn(0),
        error: CallbackFn(1),
        url: "http://tauri.localhost".parse().unwrap(),
        body: InvokeBody::Json(args),
        headers: Default::default(),
        invoke_key: tauri::test::INVOKE_KEY.to_string(),
    };
    match tauri::test::get_ipc_response(webview, req) {
        Ok(body) => match body.deserialize::<serde_json::Value>() {
            Ok(v) => (200, v.to_string()),
            Err(e) => (500, serde_json::json!(format!("bad response: {e}")).to_string()),
        },
        Err(v) => (500, v.to_string()),
    }
}

fn mime(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript",
        "css" => "text/css",
        "png" => "image/png",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

async fn handle(mut sock: TcpStream, webview: Webview, dist: PathBuf) {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 8192];
    let (head_end, content_len) = loop {
        let n = match sock.read(&mut tmp).await { Ok(0) | Err(_) => return, Ok(n) => n };
        buf.extend_from_slice(&tmp[..n]);
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&buf[..i]).to_lowercase();
            let cl = head.lines().find_map(|l| l.strip_prefix("content-length:")).and_then(|v| v.trim().parse::<usize>().ok()).unwrap_or(0);
            break (i + 4, cl);
        }
    };
    while buf.len() < head_end + content_len {
        let n = match sock.read(&mut tmp).await { Ok(0) | Err(_) => return, Ok(n) => n };
        buf.extend_from_slice(&tmp[..n]);
    }
    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
    let mut parts = head.lines().next().unwrap_or("").split_whitespace();
    let (method, path) = (parts.next().unwrap_or(""), parts.next().unwrap_or("/").split('?').next().unwrap_or("/").to_string());
    let body = &buf[head_end..head_end + content_len];

    let (status, ctype, payload): (u16, &str, Vec<u8>) = if method == "POST" && path.starts_with("/invoke/") {
        let cmd = path["/invoke/".len()..].to_string();
        let args: serde_json::Value = serde_json::from_slice(body).unwrap_or(serde_json::json!({}));
        let wv = webview.clone();
        let (s, b) = tokio::task::spawn_blocking(move || invoke(&wv, &cmd, args)).await.unwrap_or((500, "\"join error\"".into()));
        (s, "application/json", b.into_bytes())
    } else if method == "GET" {
        let rel = if path == "/" { "index.html".to_string() } else { path.trim_start_matches('/').to_string() };
        if rel.contains("..") {
            (400, "text/plain", b"bad path".to_vec())
        } else {
            match std::fs::read(dist.join(&rel)) {
                Ok(b) => (200, mime(&rel), b),
                Err(_) => (404, "text/plain", b"not found".to_vec()),
            }
        }
    } else {
        (405, "text/plain", b"method not allowed".to_vec())
    };
    let resp = format!("HTTP/1.1 {} X\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", status, ctype, payload.len());
    let _ = sock.write_all(resp.as_bytes()).await;
    let _ = sock.write_all(&payload).await;
    let _ = sock.shutdown().await;
}

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider().install_default().ok();
    let port: u16 = std::env::args().nth(1).and_then(|p| p.parse().ok()).unwrap_or(14321);
    let dist = PathBuf::from(std::env::var("WF_DIST").unwrap_or_else(|_| "../dist".to_string()));
    let app = tauri::test::mock_builder()
        .invoke_handler(wayfinder_tauri::wayfinder_handler!())
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("mock app");
    let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().expect("mock window");
    let listener = TcpListener::bind(("127.0.0.1", port)).await.expect("bind");
    println!("wf-bridge listening on http://127.0.0.1:{port} (dist {})", dist.display());
    loop {
        let Ok((sock, _)) = listener.accept().await else { continue };
        tokio::spawn(handle(sock, webview.clone(), dist.clone()));
    }
}
