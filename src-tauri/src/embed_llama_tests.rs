//! Offline tests for `commands::generate_embeddings_llama` (the Lemonade /
//! OpenAI-style embeddings client) against a std-only mock server.
//!
//! The error body below is copied verbatim from a live Lemonade 2026.41.1
//! server (nomic-embed-text-v1-GGUF, default physical batch 512) when a
//! ~2000-byte input was sent.

use crate::commands::generate_embeddings_llama;
use crate::mock_http;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const LEMONADE_TOO_LARGE_BODY: &str = r#"{"error":{"code":500,"details":{"backend":"llama-server","response":{"error":{"code":500,"message":"input (521 tokens) is too large to process. increase the physical batch size (current batch size: 512)","type":"server_error"}}},"message":"input (521 tokens) is too large to process. increase the physical batch size (current batch size: 512)","status_code":500,"type":"server_error"}}"#;

const MODEL: &str = "nomic-embed-text-v1-GGUF";

fn ok_embedding_body() -> String {
    serde_json::json!({
        "data": [{"embedding": [0.25, -0.5, 0.125], "index": 0, "object": "embedding"}],
        "model": MODEL,
        "object": "list",
        "usage": {"prompt_tokens": 4, "total_tokens": 4}
    })
    .to_string()
}

fn temp_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("wayfinder_embed_test_{}_{}", tag, nanos));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Create `<dir>/notes/biology.md` with `content` and `<dir>/index/index.json`
/// listing it. Returns the index directory.
fn setup_one_file(tag: &str, content: &str) -> PathBuf {
    let root = temp_dir(tag);
    let notes = root.join("notes");
    let index = root.join("index");
    fs::create_dir_all(&notes).unwrap();
    fs::create_dir_all(&index).unwrap();
    let file = notes.join("biology.md");
    fs::write(&file, content).unwrap();
    let index_json = serde_json::json!({
        "files": [{
            "path": file.to_string_lossy(),
            "name": "biology.md",
            "size": content.len(),
            "modified": "2026-10-25 12:00:00",
            "extension": "md"
        }],
        "scan_path": notes.to_string_lossy(),
        "created_at": "2026-10-25 12:00:00"
    });
    fs::write(index.join("index.json"), index_json.to_string()).unwrap();
    index
}

fn input_of(body: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(body).expect("request body is JSON");
    v["input"].as_str().expect("input is a string").to_string()
}

async fn run(index: &Path, base: &str) -> serde_json::Value {
    generate_embeddings_llama(
        index.to_string_lossy().to_string(),
        None,
        None,
        Some(format!("{}/api", base)),
        Some(MODEL.to_string()),
    )
    .await
    .expect("generate_embeddings_llama returned Err")
}

// ---------------------------------------------------------------- WF11 ----

#[tokio::test]
async fn utf8_boundary_at_8000_bytes_does_not_panic() {
    // 7999 ASCII bytes, then a 2-byte 'é' spanning bytes 7999..8001: slicing
    // at byte 8000 lands inside the character.
    let content = format!("{}é end of note", "a".repeat(7999));
    let index = setup_one_file("utf8", &content);
    let (base, log) = mock_http::start(Arc::new(|_m: &str, _p: &str, body: &str| {
        if input_of(body).len() <= 8000 {
            (200, ok_embedding_body())
        } else {
            (400, r#"{"error":{"message":"mock: input over 8000 bytes"}}"#.to_string())
        }
    }));

    let result = run(&index, &base).await;

    assert_eq!(result["embeddings_generated"], 1, "result: {}", result);
    assert_eq!(result["error_count"], 0, "result: {}", result);
    let reqs = mock_http::requests(&log);
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].method, "POST");
    assert_eq!(reqs[0].path, "/api/v1/embeddings");
    // Cut at the last char boundary at or below 8000 bytes.
    assert_eq!(input_of(&reqs[0].body), "a".repeat(7999));
}

#[tokio::test]
async fn utf8_short_multibyte_file_is_sent_whole() {
    let content = "Photosynthèse: la lumière → glucose. 光合作用";
    let index = setup_one_file("utf8_short", content);
    let (base, log) = mock_http::start(Arc::new(|_m: &str, _p: &str, _b: &str| (200, ok_embedding_body())));

    let result = run(&index, &base).await;

    assert_eq!(result["embeddings_generated"], 1, "result: {}", result);
    let reqs = mock_http::requests(&log);
    assert_eq!(reqs.len(), 1);
    assert_eq!(input_of(&reqs[0].body), content);
}

// ---------------------------------------------------------------- WF12 ----

#[tokio::test]
async fn too_large_input_is_retried_with_halved_text() {
    // 8000 ASCII bytes; the mock behaves like stock Lemonade and rejects
    // anything over 1800 bytes with the real "too large to process" body.
    let content = "word ".repeat(1600);
    let index = setup_one_file("too_large", &content);
    let (base, log) = mock_http::start(Arc::new(|_m: &str, _p: &str, body: &str| {
        if input_of(body).len() > 1800 {
            (500, LEMONADE_TOO_LARGE_BODY.to_string())
        } else {
            (200, ok_embedding_body())
        }
    }));

    let result = run(&index, &base).await;

    assert_eq!(result["embeddings_generated"], 1, "result: {}", result);
    assert_eq!(result["error_count"], 0, "result: {}", result);
    let reqs = mock_http::requests(&log);
    let lens: Vec<usize> = reqs.iter().map(|r| input_of(&r.body).len()).collect();
    assert!(reqs.len() >= 2 && reqs.len() <= 5, "request lengths: {:?}", lens);
    for pair in lens.windows(2) {
        assert!(pair[1] * 2 <= pair[0] + 1, "each retry must halve the text: {:?}", lens);
    }
    assert!(*lens.last().unwrap() <= 1800, "request lengths: {:?}", lens);
    for r in &reqs {
        assert!(content.starts_with(&input_of(&r.body)), "every input must be a prefix of the file");
    }
}

#[tokio::test]
async fn too_large_gives_up_after_four_retries() {
    let content = "word ".repeat(1600);
    let index = setup_one_file("too_large_forever", &content);
    let (base, log) = mock_http::start(Arc::new(|_m: &str, _p: &str, _b: &str| {
        (500, LEMONADE_TOO_LARGE_BODY.to_string())
    }));

    let result = run(&index, &base).await;

    assert_eq!(result["embeddings_generated"], 0, "result: {}", result);
    assert_eq!(result["error_count"], 1, "one failed file counts once: {}", result);
    assert_eq!(mock_http::requests(&log).len(), 5, "first try + 4 halvings");
}

#[tokio::test]
async fn other_server_errors_are_not_retried() {
    let content = "word ".repeat(1600);
    let index = setup_one_file("other_500", &content);
    let (base, log) = mock_http::start(Arc::new(|_m: &str, _p: &str, _b: &str| {
        (500, r#"{"error":{"code":500,"message":"boom","type":"server_error"}}"#.to_string())
    }));

    let result = run(&index, &base).await;

    assert_eq!(result["embeddings_generated"], 0, "result: {}", result);
    assert_eq!(result["error_count"], 1, "result: {}", result);
    assert_eq!(mock_http::requests(&log).len(), 1);
}
