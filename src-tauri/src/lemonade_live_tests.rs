//! LIVE end-to-end check of Wayfinder's own Lemonade code paths.
//!
//! Skipped (passes with a note) unless LEMONADE_URL is set, so lanes and CI
//! never need a server. The operator runs it on a machine with Lemonade:
//!
//!   LEMONADE_URL=http://127.0.0.1:13305/api \
//!   LEMONADE_CHAT_MODEL=Qwen3-4B-Instruct-2507-GGUF \
//!   cargo test --manifest-path src-tauri/Cargo.toml --lib lemonade_live -- --nocapture --test-threads=1
//!
//! LEMONADE_URL may also end in "/v1". LEMONADE_CHAT_MODEL defaults to
//! llm_config::DEFAULT_CHAT_MODEL and LEMONADE_EMBED_MODEL to
//! "nomic-embed-text-v1-GGUF" (the app defaults).
//!
//! Path driven: commands::scan_directory -> commands::generate_embeddings_llama
//! (the provider the app picks when no provider_config.json exists) ->
//! embeddings.json on disk, and commands::chat_llama for chat.

use crate::commands::{chat_llama, generate_embeddings_llama, scan_directory};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn live_base() -> Option<String> {
    let raw = std::env::var("LEMONADE_URL").ok()?;
    let mut base = raw.trim().trim_end_matches('/').to_string();
    if base.is_empty() {
        return None;
    }
    if base.ends_with("/v1") {
        base.truncate(base.len() - "/v1".len());
    }
    Some(base)
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn temp_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("wayfinder_live_{}_{}", tag, nanos));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[tokio::test]
async fn lemonade_live_scan_and_embed_real_student_notes() {
    let Some(base) = live_base() else {
        eprintln!("SKIPPED lemonade_live_scan_and_embed_real_student_notes: LEMONADE_URL is not set");
        return;
    };
    let embed_model = env_or("LEMONADE_EMBED_MODEL", "nomic-embed-text-v1-GGUF");

    let root = temp_dir("embed");
    let notes = root.join("notes");
    let index = root.join("index");
    fs::create_dir_all(&notes).unwrap();
    // Three realistic files: a short note, a long note (~6 KB, like a real
    // essay draft) and a non-English note whose multi-byte characters sit on
    // the 8000-byte truncation point.
    fs::write(notes.join("biology.md"), "# Cells\nMitochondria make ATP. Ribosomes build proteins.\n").unwrap();
    let essay = "The causes of the First World War include militarism, alliances, imperialism and nationalism. ".repeat(65);
    fs::write(notes.join("history.txt"), &essay).unwrap();
    let french = format!("{}é suite des notes de vocabulaire", "a".repeat(7999));
    fs::write(notes.join("french_vocab.md"), &french).unwrap();

    let scan = scan_directory(
        notes.to_string_lossy().to_string(),
        index.to_string_lossy().to_string(),
        None,
        None,
    )
    .await
    .expect("scan_directory failed");
    eprintln!("scan: {}", scan);
    assert_eq!(scan["files_scanned"], 3, "scan: {}", scan);

    let result = generate_embeddings_llama(
        index.to_string_lossy().to_string(),
        None,
        None,
        Some(base.clone()),
        Some(embed_model.clone()),
    )
    .await
    .expect("generate_embeddings_llama returned Err");
    eprintln!("embed: {}", result);
    let error_log = fs::read_to_string(index.join("error_log.json")).unwrap_or_default();
    if !error_log.is_empty() {
        eprintln!("error_log.json: {}", error_log);
    }

    // Truth: every file must really have been embedded; Ok(..) alone proves nothing.
    assert_eq!(result["error_count"], 0, "embed result: {} / error log: {}", result, error_log);
    assert_eq!(result["embeddings_generated"], 3, "embed result: {}", result);

    let saved: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(index.join("embeddings.json")).unwrap()).unwrap();
    assert_eq!(saved["model"], embed_model.as_str());
    let vectors = saved["embeddings"].as_array().unwrap();
    assert_eq!(vectors.len(), 3);
    let dims: Vec<usize> = vectors
        .iter()
        .map(|e| e["embedding"].as_array().map(|a| a.len()).unwrap_or(0))
        .collect();
    eprintln!("dims: {:?}", dims);
    assert!(dims.iter().all(|&d| d > 0 && d == dims[0]), "dims: {:?}", dims);
    // Distinct texts must give distinct vectors.
    assert_ne!(vectors[0]["embedding"], vectors[1]["embedding"]);

    let _ = fs::remove_dir_all(&root);
}

#[tokio::test]
async fn lemonade_live_chat_reply_is_not_empty() {
    let Some(base) = live_base() else {
        eprintln!("SKIPPED lemonade_live_chat_reply_is_not_empty: LEMONADE_URL is not set");
        return;
    };
    let chat_model = env_or("LEMONADE_CHAT_MODEL", crate::llm_config::DEFAULT_CHAT_MODEL);

    let reply = chat_llama(
        "Reply with the single word: ok".to_string(),
        Some(chat_model.clone()),
        Some(base),
    )
    .await
    .unwrap_or_else(|e| panic!("chat_llama with model {} failed: {}", chat_model, e));
    eprintln!("chat ({}): {}", chat_model, reply);
    let text = reply["text"].as_str().unwrap_or("");
    assert!(!text.trim().is_empty(), "empty chat reply: {}", reply);
}

#[tokio::test]
async fn lemonade_live_setup_check_reports_all_ok() {
    let Some(base) = live_base() else {
        eprintln!("SKIPPED lemonade_live_setup_check_reports_all_ok: LEMONADE_URL is not set");
        return;
    };
    let chat_model = env_or("LEMONADE_CHAT_MODEL", crate::llm_config::DEFAULT_CHAT_MODEL);
    let embed_model = env_or("LEMONADE_EMBED_MODEL", "nomic-embed-text-v1-GGUF");

    let report = crate::lemonade_check::check_lemonade(&base, &chat_model, &embed_model).await;
    eprintln!("setup check: {:#?}", report);
    assert!(report.all_ok(), "problems: {:?}", report.problems);
}
