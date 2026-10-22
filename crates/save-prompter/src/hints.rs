//! Save hints: spot dumping-ground folders, placeholder names and copy or version clutter.

/// Why a save looks like it is about to become clutter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HintReason {
    DumpingGround,
    PlaceholderName,
    CopyOrVersionSuffix,
    SpacesAndCaps,
}

/// Split a path into non-empty components, accepting both `/` and `\`.
fn components(path: &str) -> Vec<&str> {
    path.split(|c| c == '/' || c == '\\')
        .filter(|part| !part.is_empty())
        .collect()
}

/// The file stem: the name without its final extension.
fn file_stem(name: &str) -> &str {
    match name.rfind('.') {
        Some(0) | None => name,
        Some(i) => &name[..i],
    }
}

/// True when `stem` ends with `(n)` or ` (n)` for n in 1..=99.
fn has_paren_number(stem: &str) -> bool {
    (1..=99u32)
        .any(|n| stem.ends_with(&format!("({})", n)) || stem.ends_with(&format!(" ({})", n)))
}

/// True when `stem` ends with `v{n}` (n in 2..=99) as a separate token.
fn has_version_token(stem: &str) -> bool {
    let bytes = stem.as_bytes();
    (2..=99u32).any(|n| {
        let token = format!("v{}", n);
        stem.ends_with(&token) && {
            let idx = stem.len() - token.len();
            idx == 0 || !bytes[idx - 1].is_ascii_alphanumeric()
        }
    })
}

/// Sarcastic save hints for a path, in enum order and without duplicates.
pub fn save_hints(path: &str) -> Vec<HintReason> {
    let parts = components(path);
    let mut hints = Vec::new();

    // DumpingGround: the parent folder itself is a known dumping ground.
    if parts.len() >= 2 {
        let parent = parts[parts.len() - 2].to_lowercase();
        if matches!(parent.as_str(), "downloads" | "desktop" | "documents") {
            hints.push(HintReason::DumpingGround);
        }
    }

    let stem = parts.last().map(|name| file_stem(name)).unwrap_or("");
    let lower = stem.to_lowercase();

    // PlaceholderName: the stem still looks like a default name.
    const PLACEHOLDERS: [&str; 6] = [
        "untitled",
        "new document",
        "new text document",
        "document1",
        "book1",
        "image",
    ];
    if PLACEHOLDERS.iter().any(|p| lower.starts_with(p)) {
        hints.push(HintReason::PlaceholderName);
    }

    // CopyOrVersionSuffix: the stem carries copy/version clutter.
    const COPY_SUFFIXES: [&str; 5] = [" - copy", "_copy", "final", "_old", "_backup"];
    if COPY_SUFFIXES.iter().any(|s| lower.ends_with(s))
        || has_paren_number(&lower)
        || has_version_token(&lower)
    {
        hints.push(HintReason::CopyOrVersionSuffix);
    }

    // SpacesAndCaps: two or more consecutive spaces.
    if stem.contains("  ") {
        hints.push(HintReason::SpacesAndCaps);
    }

    hints
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_path_has_no_hints() {
        assert_eq!(
            save_hints("C:\\Users\\sam\\school\\biology\\cell-notes.md"),
            Vec::new()
        );
    }

    #[test]
    fn dumping_ground() {
        assert_eq!(
            save_hints("/home/sam/Downloads/report.md"),
            vec![HintReason::DumpingGround]
        );
    }

    #[test]
    fn downloads_subfolder_is_not_dumping_ground() {
        assert_eq!(save_hints("/home/sam/Downloads/projects/x.md"), Vec::new());
    }

    #[test]
    fn placeholder_name() {
        assert_eq!(
            save_hints("/home/sam/school/untitled report.md"),
            vec![HintReason::PlaceholderName]
        );
    }

    #[test]
    fn copy_or_version_suffix() {
        assert_eq!(
            save_hints("/home/sam/school/notes - copy.md"),
            vec![HintReason::CopyOrVersionSuffix]
        );
        assert_eq!(
            save_hints("/home/sam/school/notes (12).md"),
            vec![HintReason::CopyOrVersionSuffix]
        );
        assert_eq!(
            save_hints("/home/sam/school/notes v3.md"),
            vec![HintReason::CopyOrVersionSuffix]
        );
    }

    #[test]
    fn spaces_and_caps() {
        assert_eq!(
            save_hints("/home/sam/school/cell  notes.md"),
            vec![HintReason::SpacesAndCaps]
        );
    }

    #[test]
    fn three_reasons_at_once() {
        assert_eq!(
            save_hints("/home/sam/Downloads/untitled  report.md"),
            vec![
                HintReason::DumpingGround,
                HintReason::PlaceholderName,
                HintReason::SpacesAndCaps,
            ]
        );
    }
}
