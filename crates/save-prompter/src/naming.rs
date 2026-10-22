//! File-name suggestions on a 0-3 strictness scale.

/// Suggest a cleaned-up file name.
///
/// `file_name` includes the extension, which is preserved (lowercased).
/// `strictness` is clamped to `0..=3`. `date` is an optional `YYYY-MM-DD`
/// string supplied by the caller; the clock is never read here.
pub fn suggest_name(file_name: &str, strictness: u8, date: Option<&str>) -> String {
    let level = strictness.min(3);
    let (stem, ext) = split_name(file_name);

    // Level 0: trim surrounding whitespace.
    let mut cleaned = stem.trim().to_string();

    // Level 1: strip copy/version clutter.
    if level >= 1 {
        cleaned = strip_clutter(&cleaned);
    }

    // Level 2: normalise the stem into a slug.
    if level >= 2 {
        cleaned = normalize(&cleaned);
    }

    // Cleaning may have removed everything.
    if cleaned.is_empty() {
        cleaned = "untitled".to_string();
    }

    // Level 3: prefix the caller-supplied date.
    if level >= 3 {
        cleaned = prefix_date(&cleaned, date);
    }

    match ext {
        Some(ext) => format!("{}.{}", cleaned, ext.to_lowercase()),
        None => cleaned,
    }
}

/// Split a file name into its stem and (optional) extension.
///
/// The extension is everything after the final `.`; a leading dot (hidden
/// file) or a trailing dot is treated as having no extension.
fn split_name(file_name: &str) -> (&str, Option<&str>) {
    match file_name.rfind('.') {
        Some(i) if i > 0 && i + 1 < file_name.len() => {
            (&file_name[..i], Some(&file_name[i + 1..]))
        }
        _ => (file_name, None),
    }
}

/// Repeatedly strip clutter suffixes until nothing changes.
fn strip_clutter(stem: &str) -> String {
    let mut current = stem.to_string();
    while let Some(next) = strip_one(&current) {
        current = next;
    }
    current
}

/// Try to strip a single clutter suffix, returning the shortened stem.
fn strip_one(stem: &str) -> Option<String> {
    for suffix in [" - copy", "_copy", " copy", "_old", "_backup"] {
        if let Some(rest) = strip_suffix_ci(stem, suffix) {
            return Some(rest);
        }
    }
    if let Some(rest) = strip_paren_number(stem) {
        return Some(rest);
    }
    strip_final(stem)
}

/// Case-insensitive suffix removal.
fn strip_suffix_ci(s: &str, suffix: &str) -> Option<String> {
    if s.len() < suffix.len() {
        return None;
    }
    let idx = s.len() - suffix.len();
    if !s.is_char_boundary(idx) {
        return None;
    }
    if s[idx..].eq_ignore_ascii_case(suffix) {
        Some(s[..idx].to_string())
    } else {
        None
    }
}

/// Strip a trailing `(n)` or ` (n)` for n in 1..=99.
fn strip_paren_number(s: &str) -> Option<String> {
    for n in 1..=99u32 {
        let spaced = format!(" ({})", n);
        if let Some(rest) = strip_suffix_ci(s, &spaced) {
            return Some(rest);
        }
        let tight = format!("({})", n);
        if let Some(rest) = strip_suffix_ci(s, &tight) {
            return Some(rest);
        }
    }
    None
}

/// Strip a trailing `final` token (case-insensitive).
fn strip_final(s: &str) -> Option<String> {
    const TOKEN: &str = "final";
    if s.len() < TOKEN.len() {
        return None;
    }
    let idx = s.len() - TOKEN.len();
    if !s.is_char_boundary(idx) || !s[idx..].eq_ignore_ascii_case(TOKEN) {
        return None;
    }
    if idx == 0 {
        return Some(String::new());
    }
    let prev = s[..idx].chars().next_back()?;
    if prev.is_whitespace() {
        Some(s[..idx].trim_end().to_string())
    } else {
        None
    }
}

/// Lowercase and slugify a stem: runs of space/underscore/dot become `-`,
/// other characters are dropped, repeated `-` collapse and edges are trimmed.
fn normalize(stem: &str) -> String {
    let mut out = String::with_capacity(stem.len());
    let mut last_dash = false;
    for ch in stem.to_lowercase().chars() {
        if ch == ' ' || ch == '_' || ch == '.' || ch == '-' {
            if !last_dash {
                out.push('-');
                last_dash = true;
            }
        } else if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            out.push(ch);
            last_dash = false;
        }
    }
    out.trim_matches('-').to_string()
}

/// Prefix `stem` with `date` unless it already starts with a `YYYY-MM-DD` date.
fn prefix_date(stem: &str, date: Option<&str>) -> String {
    match date {
        Some(date) if !starts_with_date(stem) => format!("{}-{}", date, stem),
        _ => stem.to_string(),
    }
}

/// True when `s` begins with a `YYYY-MM-DD` shaped date.
fn starts_with_date(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() >= 10
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(u8::is_ascii_digit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level0_only_trims() {
        assert_eq!(
            suggest_name("Biology Notes FINAL (2).docx", 0, None),
            "Biology Notes FINAL (2).docx"
        );
    }

    #[test]
    fn level1_strips_clutter() {
        assert_eq!(
            suggest_name("Biology Notes FINAL (2).docx", 1, None),
            "Biology Notes.docx"
        );
    }

    #[test]
    fn level2_normalises() {
        assert_eq!(
            suggest_name("Biology Notes FINAL (2).docx", 2, None),
            "biology-notes.docx"
        );
    }

    #[test]
    fn level3_prefixes_date() {
        assert_eq!(
            suggest_name("Biology Notes FINAL (2).docx", 3, Some("2026-10-22")),
            "2026-10-22-biology-notes.docx"
        );
    }

    #[test]
    fn level2_keeps_version_token() {
        assert_eq!(suggest_name("report_v2_old.PDF", 2, None), "report-v2.pdf");
    }

    #[test]
    fn level3_keeps_existing_date() {
        assert_eq!(
            suggest_name("2026-01-05-lab.md", 3, Some("2026-01-05")),
            "2026-01-05-lab.md"
        );
    }

    #[test]
    fn empty_stem_becomes_untitled() {
        assert_eq!(suggest_name("   .txt", 2, None), "untitled.txt");
    }

    #[test]
    fn strictness_is_clamped() {
        assert_eq!(
            suggest_name("Biology Notes FINAL (2).docx", 9, Some("2026-10-22")),
            "2026-10-22-biology-notes.docx"
        );
    }

    #[test]
    fn name_without_extension() {
        assert_eq!(suggest_name("My Notes", 0, None), "My Notes");
        assert_eq!(suggest_name("My Notes", 2, None), "my-notes");
    }
}
