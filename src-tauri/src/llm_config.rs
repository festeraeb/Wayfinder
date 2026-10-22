// LLM configuration: resolves which OpenAI-compatible chat server to use.
//
// Precedence (highest first):
//   1. Environment variables WAYFINDER_LLM_ENDPOINT, WAYFINDER_LLM_MODEL and
//      WAYFINDER_LLM_API_KEY (each independent; ignored when empty after trim)
//   2. The text of lemonade_config.json with optional string fields
//      "chat_endpoint", "chat_model" and "api_key" (an unparseable file is
//      ignored, as are empty strings)
//   3. Built-in defaults: the local Lemonade server with DEFAULT_CHAT_MODEL.
//
// `base` is normalised so callers can always append "/v1/chat/completions".

use serde::Deserialize;

/// Base URL of a local Lemonade server (AMD's OpenAI-compatible local LLM
/// server). Callers append `/v1/chat/completions`.
pub const LEMONADE_LOCAL_BASE: &str = "http://127.0.0.1:13305/api";

/// Chat model used when neither env nor config file specifies one.
pub const DEFAULT_CHAT_MODEL: &str = "Qwen3-4B-Instruct-2507-GGUF";

/// Where to send chat requests. Any OpenAI-compatible server works, including
/// hosted ones that require an API key.
#[derive(Clone, PartialEq, Eq)]
pub struct LlmTarget {
    pub base: String,
    pub model: String,
    pub api_key: Option<String>,
}

// Hand-written so the API key can never leak through Debug output (logs,
// panic messages, etc.).
impl std::fmt::Debug for LlmTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmTarget")
            .field("base", &self.base)
            .field("model", &self.model)
            .field("api_key", &self.api_key.as_ref().map(|_| "***"))
            .finish()
    }
}

/// Optional contents of lemonade_config.json.
#[derive(Deserialize)]
struct LemonadeFileConfig {
    chat_endpoint: Option<String>,
    chat_model: Option<String>,
    api_key: Option<String>,
}

/// Resolve the chat target from env vars and an optional config file text.
pub fn resolve_chat_target(
    env: &dyn Fn(&str) -> Option<String>,
    config_json: Option<&str>,
) -> LlmTarget {
    // Start from the config file (ignored entirely when it does not parse),
    // then let env vars override field by field.
    let (mut base, mut model, mut api_key) = match config_json
        .and_then(|text| serde_json::from_str::<LemonadeFileConfig>(text).ok())
    {
        Some(cfg) => (
            non_empty(cfg.chat_endpoint),
            non_empty(cfg.chat_model),
            non_empty(cfg.api_key),
        ),
        None => (None, None, None),
    };

    env_override(env, "WAYFINDER_LLM_ENDPOINT", &mut base);
    env_override(env, "WAYFINDER_LLM_MODEL", &mut model);
    env_override(env, "WAYFINDER_LLM_API_KEY", &mut api_key);

    LlmTarget {
        base: normalize_base(&base.unwrap_or_else(|| LEMONADE_LOCAL_BASE.to_string())),
        model: model.unwrap_or_else(|| DEFAULT_CHAT_MODEL.to_string()),
        api_key,
    }
}

/// Full chat completions URL for a target.
pub fn chat_url(t: &LlmTarget) -> String {
    format!("{}/v1/chat/completions", t.base)
}

/// Authorization header value for a target, when it has an API key.
pub fn auth_header(t: &LlmTarget) -> Option<String> {
    t.api_key.as_ref().map(|key| format!("Bearer {}", key))
}

/// Apply one env var to `slot`, ignoring values that are empty after trim.
fn env_override(env: &dyn Fn(&str) -> Option<String>, name: &str, slot: &mut Option<String>) {
    if let Some(value) = env(name) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            *slot = Some(trimmed.to_string());
        }
    }
}

/// Trim a config string and drop it when empty.
fn non_empty(value: Option<String>) -> Option<String> {
    let value = value?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Normalise a base URL: trim whitespace, drop trailing '/' characters and
/// drop one trailing "/v1" (callers append "/v1/..." themselves), so
/// "https://x/lemonade/v1/" becomes "https://x/lemonade".
fn normalize_base(raw: &str) -> String {
    let mut base = raw.trim().to_string();
    while base.ends_with('/') {
        base.pop();
    }
    if base.ends_with("/v1") {
        base.truncate(base.len() - "/v1".len());
        // "host//v1" would otherwise keep a dangling slash.
        while base.ends_with('/') {
            base.pop();
        }
    }
    base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_nothing_is_set() {
        let env = |_: &str| -> Option<String> { None };
        let target = resolve_chat_target(&env, None);
        assert_eq!(target.base, LEMONADE_LOCAL_BASE);
        assert_eq!(target.model, DEFAULT_CHAT_MODEL);
        assert_eq!(target.api_key, None);
    }

    #[test]
    fn env_overrides_file() {
        let env = |key: &str| -> Option<String> {
            match key {
                "WAYFINDER_LLM_ENDPOINT" => Some("http://env-host:9/v1/".to_string()),
                "WAYFINDER_LLM_MODEL" => Some("env-model".to_string()),
                "WAYFINDER_LLM_API_KEY" => Some("env-secret".to_string()),
                _ => None,
            }
        };
        let file = Some(
            r#"{"chat_endpoint": "http://file-host:1", "chat_model": "file-model", "api_key": "file-secret"}"#,
        );
        let target = resolve_chat_target(&env, file);
        assert_eq!(target.base, "http://env-host:9");
        assert_eq!(target.model, "env-model");
        assert_eq!(target.api_key, Some("env-secret".to_string()));
    }

    #[test]
    fn file_overrides_defaults() {
        let env = |_: &str| -> Option<String> { None };
        let file = Some(r#"{"chat_endpoint": "http://file-host:1/", "chat_model": "file-model"}"#);
        let target = resolve_chat_target(&env, file);
        assert_eq!(target.base, "http://file-host:1");
        assert_eq!(target.model, "file-model");
        assert_eq!(target.api_key, None);
    }

    #[test]
    fn empty_env_strings_are_ignored() {
        let env = |key: &str| -> Option<String> {
            match key {
                "WAYFINDER_LLM_ENDPOINT" => Some("   ".to_string()),
                "WAYFINDER_LLM_MODEL" => Some(String::new()),
                "WAYFINDER_LLM_API_KEY" => Some(" \t ".to_string()),
                _ => None,
            }
        };
        let file = Some(
            r#"{"chat_endpoint": "http://file-host:1", "chat_model": "file-model", "api_key": "file-secret"}"#,
        );
        let target = resolve_chat_target(&env, file);
        assert_eq!(target.base, "http://file-host:1");
        assert_eq!(target.model, "file-model");
        assert_eq!(target.api_key, Some("file-secret".to_string()));
    }

    #[test]
    fn empty_file_strings_are_ignored() {
        let env = |_: &str| -> Option<String> { None };
        let file = Some(r#"{"chat_endpoint": "", "chat_model": "  ", "api_key": ""}"#);
        let target = resolve_chat_target(&env, file);
        assert_eq!(target.base, LEMONADE_LOCAL_BASE);
        assert_eq!(target.model, DEFAULT_CHAT_MODEL);
        assert_eq!(target.api_key, None);
    }

    #[test]
    fn malformed_json_falls_back_to_defaults() {
        let env = |_: &str| -> Option<String> { None };
        for bad in ["{ not json", "[]", "null", ""] {
            let target = resolve_chat_target(&env, Some(bad));
            assert_eq!(target.base, LEMONADE_LOCAL_BASE);
            assert_eq!(target.model, DEFAULT_CHAT_MODEL);
            assert_eq!(target.api_key, None);
        }
    }

    #[test]
    fn trailing_slash_and_v1_variants_normalise() {
        let env = |_: &str| -> Option<String> { None };

        let file = Some(r#"{"chat_endpoint": "https://x/lemonade/v1/"}"#);
        assert_eq!(resolve_chat_target(&env, file).base, "https://x/lemonade");

        let file = Some(r#"{"chat_endpoint": "https://x/lemonade/"}"#);
        assert_eq!(resolve_chat_target(&env, file).base, "https://x/lemonade");

        // Surrounding whitespace is trimmed as well.
        let file = Some(r#"{"chat_endpoint": "  https://x/lemonade  "}"#);
        assert_eq!(resolve_chat_target(&env, file).base, "https://x/lemonade");

        let env = |key: &str| -> Option<String> {
            match key {
                "WAYFINDER_LLM_ENDPOINT" => Some("https://x/lemonade/v1/".to_string()),
                _ => None,
            }
        };
        assert_eq!(resolve_chat_target(&env, None).base, "https://x/lemonade");
    }

    #[test]
    fn chat_url_appends_v1_chat_completions() {
        let target = LlmTarget {
            base: LEMONADE_LOCAL_BASE.to_string(),
            model: DEFAULT_CHAT_MODEL.to_string(),
            api_key: None,
        };
        assert_eq!(
            chat_url(&target),
            "http://127.0.0.1:13305/api/v1/chat/completions"
        );
    }

    #[test]
    fn api_key_from_env() {
        let env = |key: &str| -> Option<String> {
            match key {
                "WAYFINDER_LLM_API_KEY" => Some("env-secret".to_string()),
                _ => None,
            }
        };
        let target = resolve_chat_target(&env, None);
        assert_eq!(target.api_key, Some("env-secret".to_string()));
    }

    #[test]
    fn api_key_from_file() {
        let env = |_: &str| -> Option<String> { None };
        let file = Some(r#"{"api_key": "file-secret"}"#);
        let target = resolve_chat_target(&env, file);
        assert_eq!(target.api_key, Some("file-secret".to_string()));
    }

    #[test]
    fn auth_header_is_bearer_when_key_set() {
        let target = LlmTarget {
            base: "http://x".to_string(),
            model: "m".to_string(),
            api_key: Some("abc123".to_string()),
        };
        assert_eq!(auth_header(&target), Some("Bearer abc123".to_string()));
    }

    #[test]
    fn auth_header_is_none_without_key() {
        let target = LlmTarget {
            base: "http://x".to_string(),
            model: "m".to_string(),
            api_key: None,
        };
        assert_eq!(auth_header(&target), None);
    }

    #[test]
    fn debug_output_never_contains_api_key() {
        let target = LlmTarget {
            base: "http://x".to_string(),
            model: "m".to_string(),
            api_key: Some("super-secret-value".to_string()),
        };
        let debug = format!("{:?}", target);
        assert!(!debug.contains("super-secret-value"));
        assert!(debug.contains("api_key: Some(\"***\")"));

        let no_key = LlmTarget {
            base: "http://x".to_string(),
            model: "m".to_string(),
            api_key: None,
        };
        assert!(format!("{:?}", no_key).contains("api_key: None"));
    }
}
