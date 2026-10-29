//! Semantic search: rank files by the cosine between the query's embedding and each file's saved
//! embedding (embeddings.json), blended with the plain keyword score from `commands::search`.
//!
//! The query is embedded with the same OpenAI-style `/v1/embeddings` endpoint and model the index was
//! built with (provider_config.json, Lemonade by default). If that is not possible (other provider, server
//! down, no embeddings yet) the answer falls back to keyword search and says so in `mode`.

use crate::commands::{self, EmbeddingsData, SearchResult};
use serde_json::Value;
use std::fs;
use std::path::Path;

const LEMONADE_BASE: &str = "http://127.0.0.1:13305/api";

pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let (mut dot, mut na, mut nb) = (0f32, 0f32, 0f32);
    for (x, y) in a.iter().zip(b) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na.sqrt() * nb.sqrt()) }
}

/// Blend: `w` weights the (non-negative) cosine, `1 - w` the keyword score capped at 1.
pub fn blend(cos: f32, keyword: f32, w: f32) -> f32 {
    let w = w.clamp(0.0, 1.0);
    w * cos.max(0.0) + (1.0 - w) * keyword.clamp(0.0, 1.0)
}

/// (endpoint, model) the index was embedded with, or None if it was not a Lemonade/llama.cpp server.
fn server_of(index_dir: &Path, embeddings_model: &str) -> Option<(String, String)> {
    let cfg: Option<Value> = fs::read_to_string(index_dir.join("provider_config.json")).ok().and_then(|t| serde_json::from_str(&t).ok());
    match cfg {
        Some(c) => {
            if c.get("provider").and_then(|p| p.as_str()) != Some("llama") {
                return None;
            }
            let ep = c.get("local_endpoint").and_then(|v| v.as_str()).filter(|s| !s.trim().is_empty()).unwrap_or(LEMONADE_BASE);
            Some((ep.to_string(), embeddings_model.to_string()))
        }
        None => Some((LEMONADE_BASE.to_string(), embeddings_model.to_string())),
    }
}

async fn embed_query(base: &str, model: &str, query: &str) -> Result<Vec<f32>, String> {
    let base = base.trim().trim_end_matches('/');
    let base = base.strip_suffix("/v1").unwrap_or(base);
    let resp = reqwest::Client::new()
        .post(format!("{}/v1/embeddings", base))
        .json(&serde_json::json!({"model": model, "input": query}))
        .send()
        .await
        .map_err(|e| format!("embedding request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("embedding request returned {}", resp.status()));
    }
    let json: Value = resp.json().await.map_err(|e| format!("embedding reply was not JSON: {e}"))?;
    let arr = json.get("data").and_then(|d| d.get(0)).and_then(|d| d.get("embedding")).ok_or("embedding reply had no vector")?;
    serde_json::from_value::<Vec<f32>>(arr.clone()).map_err(|e| format!("bad vector: {e}"))
}

fn preview_of(path: &str) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    Some(text.chars().take(120).collect::<String>().trim().to_string())
}

/// Hybrid search. Returns `{ "mode": "semantic" | "keyword", "note": .., "results": [..] }`.
pub async fn run(query: &str, index_dir: &str, top_k: usize, semantic_weight: f32) -> Result<Value, String> {
    let dir = Path::new(index_dir);
    let kw_all: Vec<SearchResult> = serde_json::from_value(commands::search(query.to_string(), index_dir.to_string(), usize::MAX, semantic_weight).await?)
        .map_err(|e| format!("keyword search result: {e}"))?;

    let keyword_only = |note: String, kw: Vec<SearchResult>| {
        let mut kw = kw;
        kw.truncate(top_k);
        serde_json::json!({ "mode": "keyword", "note": note, "results": kw })
    };

    let emb: EmbeddingsData = match fs::read_to_string(dir.join("embeddings.json")).ok().and_then(|t| serde_json::from_str(&t).ok()) {
        Some(e) => e,
        None => return Ok(keyword_only("No embeddings yet: keyword search only.".into(), kw_all)),
    };
    let Some((base, model)) = server_of(dir, &emb.model) else {
        return Ok(keyword_only("Embeddings come from a provider this search cannot query: keyword search only.".into(), kw_all));
    };
    let q = match embed_query(&base, &model, query).await {
        Ok(v) => v,
        Err(e) => return Ok(keyword_only(format!("Could not embed the query ({e}): keyword search only."), kw_all)),
    };

    let mut rows: Vec<(f32, f32, String, String, Option<String>)> = Vec::new(); // score, cos, path, name, preview
    for fe in &emb.embeddings {
        let cos = cosine(&q, &fe.embedding);
        let kw = kw_all.iter().find(|r| r.path == fe.path);
        let name = Path::new(&fe.path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| fe.path.clone());
        let preview = kw.and_then(|r| r.preview.clone()).or_else(|| preview_of(&fe.path));
        rows.push((blend(cos, kw.map(|r| r.score).unwrap_or(0.0), semantic_weight), cos, fe.path.clone(), name, preview));
    }
    rows.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    rows.truncate(top_k);
    let results: Vec<Value> = rows
        .into_iter()
        .map(|(score, cos, path, name, preview)| serde_json::json!({ "path": path, "name": name, "score": score, "similarity": cos, "preview": preview }))
        .collect();
    Ok(serde_json::json!({ "mode": "semantic", "note": null, "results": results }))
}

#[tauri::command]
pub async fn semantic_search(query: String, index_dir: String, top_k: usize, semantic_weight: f32) -> Result<Value, String> {
    run(&query, &index_dir, top_k, semantic_weight).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{FileEmbedding, FileEntry, IndexData};
    use crate::mock_http;
    use std::sync::Arc;

    #[test]
    fn cosine_and_blend_basics() {
        assert!((cosine(&[1.0, 0.0], &[1.0, 0.0]) - 1.0).abs() < 1e-6);
        assert_eq!(cosine(&[1.0, 0.0], &[0.0, 1.0]), 0.0);
        assert_eq!(cosine(&[1.0], &[1.0, 2.0]), 0.0);
        assert!((blend(0.5, 2.0, 0.7) - (0.7 * 0.5 + 0.3)).abs() < 1e-6);
        assert_eq!(blend(-0.4, 0.0, 1.0), 0.0);
    }

    fn fixture(tag: &str, endpoint: &str) -> String {
        let dir = std::env::temp_dir().join(format!("wf-sem-{}-{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let files = ["alpha.md", "beta.md"];
        let mut entries = Vec::new();
        let mut embs = Vec::new();
        for (i, f) in files.iter().enumerate() {
            let p = dir.join(f);
            fs::write(&p, format!("note number {i}")).unwrap();
            entries.push(FileEntry { path: p.to_string_lossy().to_string(), name: f.to_string(), size: 1, modified: "x".into(), extension: "md".into() });
            let v = if i == 0 { vec![1.0, 0.0] } else { vec![0.0, 1.0] };
            embs.push(FileEmbedding { path: p.to_string_lossy().to_string(), embedding: v, content_hash: "h".into() });
        }
        fs::write(dir.join("index.json"), serde_json::to_string(&IndexData { files: entries, scan_path: dir.to_string_lossy().to_string(), created_at: "x".into() }).unwrap()).unwrap();
        fs::write(dir.join("embeddings.json"), serde_json::to_string(&EmbeddingsData { embeddings: embs, model: "m".into(), created_at: "x".into() }).unwrap()).unwrap();
        fs::write(dir.join("provider_config.json"), serde_json::json!({"provider": "llama", "local_model": "m", "local_endpoint": endpoint}).to_string()).unwrap();
        dir.to_string_lossy().to_string()
    }

    #[tokio::test]
    async fn ranks_by_query_embedding_and_sends_the_query() {
        // The mock "embeds" every query as [0, 1], which is the second file's vector.
        let (base, log) = mock_http::start(Arc::new(|_m, _p, _b| (200, r#"{"data":[{"embedding":[0.0,1.0]}]}"#.to_string())));
        let dir = fixture("rank", &format!("{base}/api"));
        let out = run("zzz-no-keyword-match", &dir, 5, 0.7).await.unwrap();
        assert_eq!(out["mode"], "semantic");
        assert_eq!(out["results"][0]["name"], "beta.md");
        assert!(out["results"][0]["similarity"].as_f64().unwrap() > 0.99);
        let sent = log.lock().unwrap();
        assert_eq!(sent[0].path, "/api/v1/embeddings");
        assert!(sent[0].body.contains("zzz-no-keyword-match") && sent[0].body.contains("\"m\""));
        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn falls_back_to_keyword_when_the_server_is_down() {
        let dir = fixture("down", "http://127.0.0.1:1/api");
        let out = run("alpha", &dir, 5, 0.7).await.unwrap();
        assert_eq!(out["mode"], "keyword");
        assert!(out["note"].as_str().unwrap().contains("Could not embed"));
        assert_eq!(out["results"][0]["name"], "alpha.md");
        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn no_embeddings_means_keyword_only() {
        let dir = fixture("none", "http://127.0.0.1:1/api");
        fs::remove_file(Path::new(&dir).join("embeddings.json")).unwrap();
        let out = run("beta", &dir, 5, 0.7).await.unwrap();
        assert_eq!(out["mode"], "keyword");
        assert_eq!(out["results"][0]["name"], "beta.md");
        let _ = fs::remove_dir_all(&dir);
    }
}
