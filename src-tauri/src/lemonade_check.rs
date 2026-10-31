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
        self.models_ok
            && self.chat_model_present
            && self.chat_model_downloaded
            && self.embed_model_present
            && self.embed_model_downloaded
            && self.embed_dim.map_or(false, |d| d > 0)
            && self.chat_reply.as_deref().map_or(false, |r| !r.trim().is_empty())
            && self.problems.is_empty()
    }
}

/// "http://host/api/v1/" -> "http://host/api".
fn normalise_base(base: &str) -> String {
    let b = base.trim().trim_end_matches('/');
    let b = b.strip_suffix("/v1").unwrap_or(b);
    b.trim_end_matches('/').to_string()
}

/// Look up `name` in the /v1/models "data" array: (present, downloaded, checkpoint).
/// A missing "downloaded" field counts as downloaded (non-Lemonade servers omit it).
fn find_model(data: &[serde_json::Value], name: &str) -> (bool, bool, Option<String>) {
    match data.iter().find(|m| m.get("id").and_then(|v| v.as_str()) == Some(name)) {
        Some(m) => (
            true,
            m.get("downloaded").and_then(|v| v.as_bool()).unwrap_or(true),
            m.get("checkpoint").and_then(|v| v.as_str()).map(str::to_string),
        ),
        None => (false, false, None),
    }
}

/// Run the check. Never panics and never returns Err: failures go into
/// `problems`.
pub async fn check_lemonade(base: &str, chat_model: &str, embed_model: &str) -> LemonadeReport {
    check_lemonade_with_auth(base, chat_model, embed_model, None).await
}

/// [`check_lemonade`] against a key-gated Lemonade: `auth` is the Authorization header value sent on
/// every request (models, embeddings, chat).
pub async fn check_lemonade_with_auth(base: &str, chat_model: &str, embed_model: &str, auth: Option<&str>) -> LemonadeReport {
    let base = normalise_base(base);
    let with_auth = |r: reqwest::RequestBuilder| match auth {
        Some(h) => r.header("Authorization", h),
        None => r,
    };
    let client = reqwest::Client::new();
    let mut report = LemonadeReport {
        models_ok: false,
        chat_model_present: false,
        chat_model_downloaded: false,
        embed_model_present: false,
        embed_model_downloaded: false,
        embed_checkpoint: None,
        embed_dim: None,
        chat_reply: None,
        problems: Vec::new(),
    };

    let resp = match with_auth(client.get(format!("{}/v1/models", base))).send().await {
        Ok(r) => r,
        Err(e) => {
            report.problems.push(format!("Lemonade not reachable at {}: {}", base, e));
            return report;
        }
    };
    if !resp.status().is_success() {
        report.problems.push(format!("GET /v1/models returned {}", resp.status()));
        return report;
    }
    let data = match resp.json::<serde_json::Value>().await {
        Ok(json) => match json.get("data").and_then(|d| d.as_array()) {
            Some(d) => d.clone(),
            None => {
                report.problems.push("GET /v1/models answered without a \"data\" list".to_string());
                return report;
            }
        },
        Err(e) => {
            report.problems.push(format!("GET /v1/models did not return JSON: {}", e));
            return report;
        }
    };
    report.models_ok = true;

    let (present, downloaded, _) = find_model(&data, chat_model);
    report.chat_model_present = present;
    report.chat_model_downloaded = downloaded;
    if !present {
        report.problems.push(format!("chat model {} is not listed", chat_model));
    } else if !downloaded {
        report.problems.push(format!("chat model {} is not downloaded", chat_model));
    }

    let (present, downloaded, checkpoint) = find_model(&data, embed_model);
    report.embed_model_present = present;
    report.embed_model_downloaded = downloaded;
    report.embed_checkpoint = checkpoint;
    if !present {
        report.problems.push(format!("embedding model {} is not listed", embed_model));
    } else if !downloaded {
        report.problems.push(format!("embedding model {} is not downloaded", embed_model));
    }

    if report.embed_model_present && report.embed_model_downloaded {
        let body = serde_json::json!({"model": embed_model, "input": "Wayfinder setup check"});
        match with_auth(client.post(format!("{}/v1/embeddings", base))).json(&body).send().await {
            Ok(r) if r.status().is_success() => {
                let dim = r.json::<serde_json::Value>().await.ok().and_then(|j| {
                    j.pointer("/data/0/embedding").and_then(|e| e.as_array()).map(|e| e.len())
                });
                match dim {
                    Some(d) if d > 0 => report.embed_dim = Some(d),
                    _ => report.problems.push("POST /v1/embeddings returned no embedding".to_string()),
                }
            }
            Ok(r) => report.problems.push(format!("POST /v1/embeddings returned {}", r.status())),
            Err(e) => report.problems.push(format!("POST /v1/embeddings failed: {}", e)),
        }
    }

    // Never chat with a model that is not downloaded: Lemonade would start a multi-GB download.
    if report.chat_model_present && report.chat_model_downloaded {
        let body = serde_json::json!({
            "model": chat_model,
            "messages": [{"role": "user", "content": "Reply with the single word: ok"}],
            "max_tokens": 8,
            "temperature": 0,
        });
        match with_auth(client.post(format!("{}/v1/chat/completions", base))).json(&body).send().await {
            Ok(r) if r.status().is_success() => {
                let reply = r.json::<serde_json::Value>().await.ok().and_then(|j| {
                    j.pointer("/choices/0/message/content").and_then(|c| c.as_str()).map(str::to_string)
                });
                match reply {
                    Some(text) if !text.trim().is_empty() => report.chat_reply = Some(text),
                    _ => report.problems.push("POST /v1/chat/completions returned no reply".to_string()),
                }
            }
            Ok(r) => report.problems.push(format!("POST /v1/chat/completions returned {}", r.status())),
            Err(e) => report.problems.push(format!("POST /v1/chat/completions failed: {}", e)),
        }
    }

    report
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

    #[tokio::test]
    async fn lemonade_check_sends_the_key_on_every_request() {
        let (base, log) = server(models_body(Some(true), true));
        let report = check_lemonade_with_auth(&format!("{}/api", base), CHAT, EMBED, Some("Bearer k-123")).await;
        assert!(report.all_ok(), "{:?}", report);
        let reqs = mock_http::requests(&log);
        assert_eq!(reqs.len(), 3, "{:?}", reqs);
        assert!(reqs.iter().all(|r| r.authorization.as_deref() == Some("Bearer k-123")), "{:?}", reqs);
    }

    #[tokio::test]
    async fn lemonade_check_without_a_key_sends_none() {
        let (base, log) = server(models_body(Some(true), true));
        check_lemonade(&format!("{}/api", base), CHAT, EMBED).await;
        assert!(mock_http::requests(&log).iter().all(|r| r.authorization.is_none()));
    }
}
