//! Tauri command wrapping the Lemonade setup check for the UI.

use crate::lemonade_check::check_lemonade_with_auth;

/// Embedding model the app asks Lemonade for when nothing else is chosen (same as the embeddings default).
const DEFAULT_EMBED_MODEL: &str = "nomic-embed-text-v1-GGUF";

fn non_blank(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Run the Lemonade setup check. Blank arguments fall back to the chat target resolved from the
/// WAYFINDER_LLM_* environment and lemonade_config.json (Lemonade on 127.0.0.1:13305 and the default
/// chat model if unset). The configured API key is sent when the check targets the configured server.
#[tauri::command]
pub async fn check_lemonade_setup(
    base: Option<String>,
    chat_model: Option<String>,
    embed_model: Option<String>,
) -> Result<serde_json::Value, String> {
    let target = crate::commands::resolved_chat_target();
    let base = non_blank(base).unwrap_or_else(|| target.base.clone());
    let chat_model = non_blank(chat_model).unwrap_or_else(|| target.model.clone());
    let embed_model = non_blank(embed_model)
        .or_else(|| non_blank(std::env::var("WAYFINDER_EMBED_MODEL").ok()))
        .unwrap_or_else(|| DEFAULT_EMBED_MODEL.to_string());
    let auth = crate::llm_config::auth_for_base(&base, &target);
    let r = check_lemonade_with_auth(&base, &chat_model, &embed_model, auth.as_deref()).await;
    Ok(serde_json::json!({
        "all_ok": r.all_ok(),
        "base": base,
        "chat_model": chat_model,
        "embed_model": embed_model,
        "models_ok": r.models_ok,
        "chat_model_present": r.chat_model_present,
        "chat_model_downloaded": r.chat_model_downloaded,
        "embed_model_present": r.embed_model_present,
        "embed_model_downloaded": r.embed_model_downloaded,
        "embed_checkpoint": r.embed_checkpoint,
        "embed_dim": r.embed_dim,
        "chat_reply": r.chat_reply,
        "problems": r.problems,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unreachable_server_reports_a_problem_not_an_error() {
        let v = check_lemonade_setup(Some("http://127.0.0.1:1/api".into()), Some("c".into()), Some("e".into()))
            .await
            .unwrap();
        assert_eq!(v["all_ok"], false);
        assert_eq!(v["models_ok"], false);
        assert_eq!(v["chat_model"], "c");
        assert!(v["problems"][0].as_str().unwrap().contains("not reachable"));
    }
}
