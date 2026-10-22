//! Bridge from Wayfinder's file watcher to the save-prompter crate: decide whether and how to prompt on a save.

pub use wayfinder_save_prompter as sp;

use sp::backoff::Backoff;
use sp::{clippy, hints, naming};

/// What the UI needs to show one save prompt.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct SavePrompt {
    pub path: String,
    /// Hint names, e.g. "PlaceholderName".
    pub reasons: Vec<String>,
    pub suggested_name: String,
    /// What Mr. Clippy says (sarcastic about the file, softer as the folder's level rises).
    pub line: String,
    /// Back-off level of the folder before this save (0 = no streak yet), for Clippy's tone.
    pub level: u8,
}

/// Split a path into (folder, file name), accepting both `/` and `\`.
fn split_path(path: &str) -> (&str, &str) {
    match path.rfind(|c| c == '/' || c == '\\') {
        Some(i) => (&path[..i], &path[i + 1..]),
        None => ("", path),
    }
}

/// Decide whether a save deserves a prompt.
///
/// A clean save extends the folder's streak and never prompts. A messy save is checked against the
/// back-off *before* it is recorded, so a streak the user earned can still keep Clippy quiet; the miss
/// is recorded afterwards, so a second miss in a row always prompts.
pub fn evaluate_save(
    path: &str,
    strictness: u8,
    backoff: &mut Backoff,
    save_index: u64,
    date: Option<&str>,
) -> Option<SavePrompt> {
    let reasons = hints::save_hints(path);
    let (folder, name) = split_path(path);
    if reasons.is_empty() {
        backoff.record_good(folder);
        return None;
    }
    let prompt = backoff.should_prompt(folder, save_index);
    let level = backoff.level(folder);
    backoff.record_bad(folder);
    if !prompt {
        return None;
    }
    Some(SavePrompt {
        path: path.to_string(),
        reasons: reasons.iter().map(|r| format!("{:?}", r)).collect(),
        suggested_name: naming::suggest_name(name, strictness, date),
        line: clippy::clippy_line(&reasons, level, save_index).to_string(),
        level,
    })
}

/// Live save-prompter state for the running app: one back-off ledger, the running save count,
/// the user's strictness and the prompts waiting for the UI.
#[derive(Debug)]
pub struct SavePrompterState {
    pub backoff: Backoff,
    pub save_index: u64,
    pub strictness: u8,
    pub prompts: Vec<SavePrompt>,
}

impl Default for SavePrompterState {
    fn default() -> Self {
        Self { backoff: Backoff::default(), save_index: 0, strictness: 2, prompts: Vec::new() }
    }
}

/// Most prompts kept for the UI; older ones drop off the front.
const MAX_PENDING_PROMPTS: usize = 50;

pub static SAVE_STATE: once_cell::sync::Lazy<std::sync::Mutex<SavePrompterState>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(SavePrompterState::default()));

/// Feed one save from the file watcher through the prompter, queueing a prompt if it earns one.
pub fn on_save_event(state: &mut SavePrompterState, path: &str, date: Option<&str>) {
    let index = state.save_index;
    state.save_index += 1;
    let strictness = state.strictness;
    if let Some(prompt) = evaluate_save(path, strictness, &mut state.backoff, index, date) {
        state.prompts.push(prompt);
        if state.prompts.len() > MAX_PENDING_PROMPTS {
            state.prompts.remove(0);
        }
    }
}

/// Pending save prompts for the UI (optionally cleared once read).
#[tauri::command]
pub async fn get_save_prompts(clear: Option<bool>) -> Result<serde_json::Value, String> {
    let mut state = SAVE_STATE.lock().map_err(|e| format!("Lock error: {}", e))?;
    let prompts = state.prompts.clone();
    if clear.unwrap_or(false) {
        state.prompts.clear();
    }
    Ok(serde_json::json!({ "success": true, "count": prompts.len(), "prompts": prompts, "strictness": state.strictness }))
}

/// Set the naming strictness (0-3, higher values are clamped by the naming rules).
#[tauri::command]
pub async fn set_save_strictness(level: u8) -> Result<serde_json::Value, String> {
    let mut state = SAVE_STATE.lock().map_err(|e| format!("Lock error: {}", e))?;
    state.strictness = level.min(3);
    Ok(serde_json::json!({ "success": true, "strictness": state.strictness }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_save_event_counts_saves_and_queues_only_prompts() {
        let mut st = SavePrompterState::default();
        on_save_event(&mut st, "/home/sam/school/cell-notes.md", Some("2026-10-22"));
        assert!(st.prompts.is_empty());
        on_save_event(&mut st, "/home/sam/Downloads/Untitled.docx", Some("2026-10-22"));
        assert_eq!(st.save_index, 2);
        assert_eq!(st.prompts.len(), 1);
        assert_eq!(st.prompts[0].suggested_name, "untitled.docx");
        st.strictness = 3;
        on_save_event(&mut st, "/home/sam/Desktop/Lab Report (2).docx", Some("2026-10-22"));
        assert_eq!(st.prompts[1].suggested_name, "2026-10-22-lab-report.docx");
    }

    /// End to end through the real watcher: a new placeholder file in a Downloads folder becomes a prompt.
    #[tokio::test(flavor = "multi_thread")]
    async fn real_watcher_turns_a_save_into_a_prompt() {
        let root = std::env::temp_dir().join(format!("wf-watch-{}", std::process::id()));
        let downloads = root.join("Downloads");
        std::fs::create_dir_all(&downloads).unwrap();
        crate::commands::start_file_watcher(Some(vec![downloads.to_string_lossy().to_string()]))
            .await
            .expect("watcher starts");
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let file = downloads.join("Untitled (3).docx");
        std::fs::write(&file, b"enzyme lab notes").unwrap();

        let want = file.to_string_lossy().to_string();
        let mut found = None;
        for _ in 0..40 {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            let st = SAVE_STATE.lock().unwrap();
            if let Some(p) = st.prompts.iter().find(|p| p.path == want) {
                found = Some(p.clone());
                break;
            }
        }
        let _ = crate::commands::stop_file_watcher().await;
        let _ = std::fs::remove_dir_all(&root);
        let p = found.expect("the watcher queued a prompt for the new file");
        assert!(p.reasons.contains(&"DumpingGround".to_string()));
        assert!(p.reasons.contains(&"PlaceholderName".to_string()));
        assert!(p.suggested_name.ends_with("untitled.docx"), "{}", p.suggested_name);
    }

    #[test]
    fn pending_prompts_are_capped() {
        let mut st = SavePrompterState::default();
        for i in 0..(MAX_PENDING_PROMPTS + 7) {
            on_save_event(&mut st, &format!("/tmp/f{i}/Untitled.txt"), None);
        }
        assert_eq!(st.prompts.len(), MAX_PENDING_PROMPTS);
        assert_eq!(st.prompts[0].path, "/tmp/f7/Untitled.txt");
    }

    const DOWNLOADS_UNTITLED: &str = "C:\\Users\\sam\\Downloads\\Untitled (2).docx";

    #[test]
    fn clean_save_is_quiet_and_counts_as_good() {
        let mut b = Backoff::default();
        let path = "/home/sam/school/biology/cell-notes.md";
        assert_eq!(evaluate_save(path, 2, &mut b, 0, None), None);
        assert_eq!(b.level("/home/sam/school/biology"), 0);
        for i in 1..3 {
            assert_eq!(evaluate_save(path, 2, &mut b, i, None), None);
        }
        assert_eq!(b.level("/home/sam/school/biology"), 1, "three clean saves start a streak");
    }

    #[test]
    fn messy_save_prompts_with_reasons_and_a_name() {
        let mut b = Backoff::default();
        let p = evaluate_save(DOWNLOADS_UNTITLED, 2, &mut b, 0, None).expect("prompt");
        assert!(p.reasons.contains(&"DumpingGround".to_string()));
        assert!(p.reasons.contains(&"PlaceholderName".to_string()));
        assert_eq!(p.suggested_name, "untitled.docx");
        assert_eq!(p.path, DOWNLOADS_UNTITLED);
        assert_eq!(p.level, 0);
        assert!(!p.line.is_empty());
    }

    #[test]
    fn an_earned_streak_quiets_odd_saves_and_a_second_miss_always_prompts() {
        let folder = "/home/sam/school";
        let clean = "/home/sam/school/lab-report.md";
        let messy = "/home/sam/school/Untitled.md";
        let mut b = Backoff::default();
        for i in 0..3 {
            assert_eq!(evaluate_save(clean, 2, &mut b, i, None), None);
        }
        let mut quiet = b.clone();
        assert_eq!(evaluate_save(messy, 2, &mut quiet, 3, None), None, "odd save after a streak stays quiet");
        assert!(evaluate_save(messy, 2, &mut quiet, 5, None).is_some(), "the next miss in a row prompts");

        let mut loud = b.clone();
        let p = evaluate_save(messy, 2, &mut loud, 4, None).expect("even save after a streak prompts");
        assert_eq!(p.level, 1, "level reports the streak the user had earned");
        assert_eq!(loud.level(folder), 0, "the miss resets the streak");
    }

    #[test]
    fn split_path_handles_both_separators_and_bare_names() {
        assert_eq!(split_path("C:\\a\\b.txt"), ("C:\\a", "b.txt"));
        assert_eq!(split_path("/a/b/c.md"), ("/a/b", "c.md"));
        assert_eq!(split_path("notes.txt"), ("", "notes.txt"));
    }
}
