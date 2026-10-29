//! Tauri command wrapping the Lemonade setup check for the UI.

use crate::lemonade_check::check_lemonade;

/// Embedding model the app asks Lemonade for when nothing else is chosen (same as the embeddings default).
const DEFAULT_EMBED_MODEL: &str = "nomic-embed-text-v1-GGUF";

fn non_blank(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Run the Lemonade setup check. Blank arguments fall back to the chat target resolved from the
/// WAYFINDER_LLM_* environment (Lemonade on 127.0.0.1:13305 and the default chat model if unset).
#[tauri::command]
pub async fn check_lemonade_setup(
    base: Option<String>,
    chat_model: Option<String>,
    embed_model: Option<String>,
) -> Result<serde_json::Value, String> {
    let target = crate::llm_config::resolve_chat_target(&|k| std::env::var(k).ok(), None);
    let base = non_blank(base).unwrap_or(target.base);
    let chat_model = non_blank(chat_model).unwrap_or(target.model);
    let embed_model = non_blank(embed_model)
        .or_else(|| non_blank(std::env::var("WAYFINDER_EMBED_MODEL").ok()))
        .unwrap_or_else(|| DEFAULT_EMBED_MODEL.to_string());
    let r = check_lemonade(&base, &chat_model, &embed_model).await;
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
