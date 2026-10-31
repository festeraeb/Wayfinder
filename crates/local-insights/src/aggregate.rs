//! Aggregate: turn local labels into shareable daily counts with small counts suppressed.

use std::collections::BTreeMap;

/// Label under which the total of suppressed small-count labels is reported.
const OTHER_SUPPRESSED: &str = "other_suppressed";

/// Daily per-label counts for one day, ready to share.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyCounts {
    pub day: String,
    pub counts: BTreeMap<String, u32>,
}

/// Count `labels` for `day`, then drop every label whose count is below
/// `min_count` (small counts can identify a person). If any label was dropped,
/// their combined total is reported under `"other_suppressed"`, but only when
/// that total itself reaches `min_count`; otherwise it is dropped too.
pub fn aggregate(day: &str, labels: &[&str], min_count: u32) -> DailyCounts {
    let mut counted: BTreeMap<String, u32> = BTreeMap::new();
    for label in labels {
        *counted.entry((*label).to_string()).or_insert(0) += 1;
    }

    // Split into kept labels and the running total of suppressed counts.
    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut suppressed_total: u32 = 0;
    let mut any_suppressed = false;
    for (label, count) in counted {
        if count < min_count {
            any_suppressed = true;
            suppressed_total = suppressed_total.saturating_add(count);
        } else {
            counts.insert(label, count);
        }
    }

    if any_suppressed && suppressed_total >= min_count {
        counts.insert(OTHER_SUPPRESSED.to_string(), suppressed_total);
    }

    DailyCounts {
        day: day.to_string(),
        counts,
    }
}

/// Escape the two characters this JSON needs escaped: backslash and double quote.
fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(ch),
        }
    }
    out
}

/// Compact JSON `{"day":"...","counts":{...}}`; `BTreeMap` yields sorted keys.
pub fn to_json(c: &DailyCounts) -> String {
    let mut out = String::new();
    out.push_str("{\"day\":\"");
    out.push_str(&escape_json(&c.day));
    out.push_str("\",\"counts\":{");
    for (i, (label, count)) in c.counts.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&escape_json(label));
        out.push_str("\":");
        out.push_str(&count.to_string());
    }
    out.push_str("}}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a counts map from `(label, count)` pairs.
    fn counts(pairs: &[(&str, u32)]) -> BTreeMap<String, u32> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn counts_each_label() {
        let got = aggregate("2026-10-22", &["code", "chat", "code", "code", "chat"], 1);
        assert_eq!(got.day, "2026-10-22");
        assert_eq!(got.counts, counts(&[("chat", 2), ("code", 3)]));
    }

    #[test]
    fn suppresses_labels_below_min_count() {
        let got = aggregate("d", &["code", "chat", "chat"], 2);
        assert_eq!(got.counts, counts(&[("chat", 2)]));
    }

    #[test]
    fn other_suppressed_added_when_total_reaches_min_count() {
        // "a" and "b" each count 1 (< 2); their combined total 2 reaches min_count.
        let got = aggregate("d", &["code", "code", "a", "b"], 2);
        assert_eq!(got.counts, counts(&[("code", 2), ("other_suppressed", 2)]));
    }

    #[test]
    fn other_suppressed_dropped_when_total_below_min_count() {
        // Only "a" is suppressed, with count 1; the total 1 is below min_count.
        let got = aggregate("d", &["code", "code", "a"], 2);
        assert_eq!(got.counts, counts(&[("code", 2)]));
    }

    #[test]
    fn min_count_zero_keeps_everything() {
        let got = aggregate("d", &["a", "b", "a"], 0);
        assert_eq!(got.counts, counts(&[("a", 2), ("b", 1)]));
    }

    #[test]
    fn min_count_one_keeps_everything() {
        let got = aggregate("d", &["a", "b", "b"], 1);
        assert_eq!(got.counts, counts(&[("a", 1), ("b", 2)]));
    }

    #[test]
    fn empty_input_gives_empty_counts() {
        let got = aggregate("d", &[], 3);
        assert_eq!(got.day, "d");
        assert!(got.counts.is_empty());
        assert_eq!(to_json(&got), "{\"day\":\"d\",\"counts\":{}}");
    }

    #[test]
    fn to_json_exact_output_with_escaped_quote() {
        let c = DailyCounts {
            day: "2026-01-02".to_string(),
            counts: counts(&[("code", 3), ("say \"hi\"", 2)]),
        };
        assert_eq!(
            to_json(&c),
            "{\"day\":\"2026-01-02\",\"counts\":{\"code\":3,\"say \\\"hi\\\"\":2}}"
        );
    }

    #[test]
    fn to_json_escapes_backslashes() {
        let c = DailyCounts {
            day: "d".to_string(),
            counts: counts(&[("path\\label", 2)]),
        };
        assert_eq!(to_json(&c), "{\"day\":\"d\",\"counts\":{\"path\\\\label\":2}}");
    }

    #[test]
    fn json_never_contains_a_suppressed_label() {
        let got = aggregate("d", &["secret_label", "other_thing", "code", "code"], 2);
        let json = to_json(&got);
        assert!(!json.contains("secret_label"));
        assert!(!json.contains("other_thing"));
        // Their combined total (2) reached min_count, so the bucket is present.
        assert!(json.contains("other_suppressed"));
    }
}
