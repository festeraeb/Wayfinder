//! Bridge from Wayfinder's file watcher to the save-prompter crate: decide whether and how to prompt on a save.

pub use wayfinder_save_prompter as sp;

use sp::backoff::Backoff;
use sp::{hints, naming};

/// What the UI needs to show one save prompt.
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct SavePrompt {
    pub path: String,
    /// Hint names, e.g. "PlaceholderName".
    pub reasons: Vec<String>,
    pub suggested_name: String,
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
        level,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
