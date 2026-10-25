//! One-shot setup check against a Lemonade (or any OpenAI-compatible) server:
//! is it reachable, are the chat and embedding models listed and downloaded,
//! does one embedding come back, does one short chat reply come back.
//!
//! `base` uses the same convention as `llm_config::LEMONADE_LOCAL_BASE`
//! ("http://127.0.0.1:13305/api"); requests go to `{base}/v1/models`,
//! `{base}/v1/embeddings` and `{base}/v1/chat/completions`.

/// Result of `check_lemonade`. Every field reflects something that was
/// actually observed; nothing is assumed.
#[derive(Debug, Clone, PartialEq)]
pub struct LemonadeReport {
    /// GET {base}/v1/models answered 2xx with a JSON "data" array.
    pub models_ok: bool,
    /// An entry in "data" has "id" == chat_model.
    pub chat_model_present: bool,
    /// That entry has "downloaded": true, or has no "downloaded" field at all
    /// (non-Lemonade servers do not send it).
    pub chat_model_downloaded: bool,
    pub embed_model_present: bool,
    pub embed_model_downloaded: bool,
    /// The "checkpoint" string of the embed model entry, e.g.
    /// "nomic-ai/nomic-embed-text-v1-GGUF:Q4_K_S".
    pub embed_checkpoint: Option<String>,
    /// Length of data[0].embedding from POST {base}/v1/embeddings.
    pub embed_dim: Option<usize>,
    /// choices[0].message.content from POST {base}/v1/chat/completions.
    pub chat_reply: Option<String>,
    /// One plain-English line per failed step.
    pub problems: Vec<String>,
}

impl LemonadeReport {
    /// True only when every step was checked and passed.
    pub fn all_ok(&self) -> bool {
        todo!("WF13")
    }
}

/// Run the check. Never panics and never returns Err: failures go into
/// `problems`.
pub async fn check_lemonade(base: &str, chat_model: &str, embed_model: &str) -> LemonadeReport {
    let _ = (base, chat_model, embed_model);
    todo!("WF13")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_http;
    use std::sync::Arc;

    const CHAT: &str = "Qwen3-4B-Instruct-2507-GGUF";
    const EMBED: &str = "nomic-embed-text-v1-GGUF";

    /// GET /api/v1/models as served by Lemonade 2026.41.1 (fields trimmed to
    /// the ones Wayfinder reads; the real entries carry more).
    fn models_body(chat_downloaded: Option<bool>, embed_listed: bool) -> String {
        let mut data = vec![serde_json::json!({
            "id": "Qwen3-0.6B-GGUF",
            "checkpoint": "unsloth/Qwen3-0.6B-GGUF:Q4_0",
            "downloaded": true,
            "labels": ["chat", "reasoning", "tool-calling"],
            "object": "model",
            "owned_by": "lemonade",
            "recipe": "llamacpp"
        })];
        if let Some(downloaded) = chat_downloaded {
            data.push(serde_json::json!({
                "id": CHAT,
                "checkpoint": "unsloth/Qwen3-4B-Instruct-2507-GGUF:Q4_K_M",
                "downloaded": downloaded,
                "labels": ["chat", "tool-calling"],
                "object": "model",
                "owned_by": "lemonade",
                "recipe": "llamacpp"
            }));
        }
        if embed_listed {
            data.push(serde_json::json!({
                "id": EMBED,
                "checkpoint": "nomic-ai/nomic-embed-text-v1-GGUF:Q4_K_S",
                "downloaded": true,
                "labels": ["embeddings"],
                "max_context_window": 2048,
                "object": "model",
                "owned_by": "lemonade",
                "recipe": "llamacpp"
            }));
        }
        serde_json::json!({"data": data, "object": "list"}).to_string()
    }

    fn embedding_body() -> String {
        let v: Vec<f32> = (0..768).map(|i| (i as f32) / 1000.0).collect();
        serde_json::json!({
            "data": [{"embedding": v, "index": 0, "object": "embedding"}],
            "model": EMBED,
            "object": "list",
            "usage": {"prompt_tokens": 4, "total_tokens": 4}
        })
        .to_string()
    }

    /// POST /api/v1/chat/completions reply observed from Lemonade 2026.41.1.
    fn chat_body() -> String {
        serde_json::json!({
            "choices": [{"finish_reason": "stop", "index": 0,
                         "message": {"content": "ok", "role": "assistant"}}],
            "created": 1792900887,
            "id": "chatcmpl-DBHNKELUY9ueDQgTI18C3ahn44jtLPZJ",
            "model": CHAT,
            "object": "chat.completion",
            "usage": {"completion_tokens": 2, "prompt_tokens": 15, "total_tokens": 17}
        })
        .to_string()
    }

    fn server(models: String) -> (String, mock_http::Log) {
        mock_http::start(Arc::new(move |method: &str, path: &str, _body: &str| {
            match (method, path) {
                ("GET", "/api/v1/models") => (200, models.clone()),
                ("POST", "/api/v1/embeddings") => (200, embedding_body()),
                ("POST", "/api/v1/chat/completions") => (200, chat_body()),
                _ => (404, r#"{"error":{"message":"mock: no route"}}"#.to_string()),
            }
        }))
    }

    fn paths(log: &mock_http::Log) -> Vec<String> {
        mock_http::requests(log).into_iter().map(|r| r.path).collect()
    }

    #[tokio::test]
    async fn lemonade_check_happy_path() {
        let (base, log) = server(models_body(Some(true), true));
        let report = check_lemonade(&format!("{}/api", base), CHAT, EMBED).await;

        assert!(report.models_ok, "{:?}", report);
        assert!(report.chat_model_present && report.chat_model_downloaded, "{:?}", report);
        assert!(report.embed_model_present && report.embed_model_downloaded, "{:?}", report);
        assert_eq!(report.embed_checkpoint.as_deref(), Some("nomic-ai/nomic-embed-text-v1-GGUF:Q4_K_S"));
        assert_eq!(report.embed_dim, Some(768));
        assert_eq!(report.chat_reply.as_deref(), Some("ok"));
        assert!(report.problems.is_empty(), "{:?}", report.problems);
        assert!(report.all_ok());

        let reqs = mock_http::requests(&log);
        let embed_req = reqs.iter().find(|r| r.path == "/api/v1/embeddings").expect("embeddings request");
        let body: serde_json::Value = serde_json::from_str(&embed_req.body).unwrap();
        assert_eq!(body["model"], EMBED);
        let chat_req = reqs.iter().find(|r| r.path == "/api/v1/chat/completions").expect("chat request");
        let body: serde_json::Value = serde_json::from_str(&chat_req.body).unwrap();
        assert_eq!(body["model"], CHAT);
        assert!(body["max_tokens"].as_u64().unwrap_or(9999) <= 16, "keep the check cheap");
    }

    #[tokio::test]
    async fn lemonade_check_trailing_slash_and_v1_are_accepted() {
        for suffix in ["/api/", "/api/v1", "/api/v1/"] {
            let (base, _log) = server(models_body(Some(true), true));
            let report = check_lemonade(&format!("{}{}", base, suffix), CHAT, EMBED).await;
            assert!(report.all_ok(), "suffix {}: {:?}", suffix, report);
        }
    }

    #[tokio::test]
    async fn lemonade_check_does_not_chat_with_a_model_that_is_not_downloaded() {
        // Lemonade would start a multi-GB download if asked to chat with it.
        let (base, log) = server(models_body(Some(false), true));
        let report = check_lemonade(&format!("{}/api", base), CHAT, EMBED).await;

        assert!(report.chat_model_present);
        assert!(!report.chat_model_downloaded);
        assert_eq!(report.chat_reply, None);
        assert!(!report.all_ok());
        assert!(report.problems.iter().any(|p| p.contains(CHAT)), "{:?}", report.problems);
        assert!(!paths(&log).iter().any(|p| p.ends_with("/chat/completions")), "{:?}", paths(&log));
        // The embedding half is still checked.
        assert_eq!(report.embed_dim, Some(768));
    }

    #[tokio::test]
    async fn lemonade_check_missing_embed_model_is_reported_and_not_called() {
        let (base, log) = server(models_body(Some(true), false));
        let report = check_lemonade(&format!("{}/api", base), CHAT, EMBED).await;

        assert!(!report.embed_model_present);
        assert!(!report.embed_model_downloaded);
        assert_eq!(report.embed_checkpoint, None);
        assert_eq!(report.embed_dim, None);
        assert!(!report.all_ok());
        assert!(report.problems.iter().any(|p| p.contains(EMBED)), "{:?}", report.problems);
        assert!(!paths(&log).iter().any(|p| p.ends_with("/embeddings")), "{:?}", paths(&log));
    }

    #[tokio::test]
    async fn lemonade_check_server_down_is_a_problem_not_a_panic() {
        let report = check_lemonade(&format!("{}/api", mock_http::closed_base()), CHAT, EMBED).await;

        assert!(!report.models_ok);
        assert!(!report.all_ok());
        assert!(!report.problems.is_empty());
        assert_eq!(report.embed_dim, None);
        assert_eq!(report.chat_reply, None);
    }

    #[tokio::test]
    async fn lemonade_check_error_status_on_embeddings_is_a_problem() {
        let models = models_body(Some(true), true);
        let (base, _log) = mock_http::start(Arc::new(move |method: &str, path: &str, _b: &str| {
            match (method, path) {
                ("GET", "/api/v1/models") => (200, models.clone()),
                ("POST", "/api/v1/embeddings") => (500, crate::embed_llama_tests::LEMONADE_TOO_LARGE_BODY.to_string()),
                ("POST", "/api/v1/chat/completions") => (200, chat_body()),
                _ => (404, "{}".to_string()),
            }
        }));
        let report = check_lemonade(&format!("{}/api", base), CHAT, EMBED).await;

        assert_eq!(report.embed_dim, None);
        assert_eq!(report.chat_reply.as_deref(), Some("ok"));
        assert!(!report.all_ok());
        assert!(report.problems.iter().any(|p| p.contains("500")), "{:?}", report.problems);
    }
}
