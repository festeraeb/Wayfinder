//! Back-off: prompt less in folders where the user keeps saving well.

use std::collections::BTreeMap;

/// Per-folder save history used to throttle prompts.
///
/// Keys are lowercased folder paths so lookups are case-insensitive.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Backoff {
    streaks: BTreeMap<String, u32>,
    misses: BTreeMap<String, u32>,
}

impl Backoff {
    /// A well-saved file: lengthen the streak, forgive past misses.
    pub fn record_good(&mut self, folder: &str) {
        let key = folder.to_lowercase();
        *self.streaks.entry(key.clone()).or_insert(0) += 1;
        self.misses.insert(key, 0);
    }

    /// A poorly-saved file: reset the streak, count the miss.
    pub fn record_bad(&mut self, folder: &str) {
        let key = folder.to_lowercase();
        self.streaks.insert(key.clone(), 0);
        *self.misses.entry(key).or_insert(0) += 1;
    }

    /// Whether to prompt on this save.
    ///
    /// `save_index` is the caller's monotonically increasing save counter; it
    /// is only used to thin out prompts as a folder's streak grows.
    pub fn should_prompt(&self, folder: &str, save_index: u64) -> bool {
        let key = folder.to_lowercase();
        let streak = *self.streaks.get(&key).unwrap_or(&0);
        let misses = *self.misses.get(&key).unwrap_or(&0);

        // Repeated misses always earn a prompt, whatever the streak.
        if misses >= 2 {
            return true;
        }

        match streak {
            0..=2 => true,
            3..=5 => save_index % 2 == 0,
            6..=9 => save_index % 4 == 0,
            _ => save_index % 8 == 0,
        }
    }

    /// How much trust the user has earned, used to soften Clippy's tone.
    pub fn level(&self, folder: &str) -> u8 {
        let key = folder.to_lowercase();
        let streak = *self.streaks.get(&key).unwrap_or(&0);
        match streak {
            0..=2 => 0,
            3..=5 => 1,
            6..=9 => 2,
            _ => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a `Backoff` with `n` consecutive good saves in `folder`.
    fn with_streak(folder: &str, n: u32) -> Backoff {
        let mut backoff = Backoff::default();
        for _ in 0..n {
            backoff.record_good(folder);
        }
        backoff
    }

    #[test]
    fn level_boundary_2_3() {
        assert_eq!(with_streak("dl", 2).level("dl"), 0);
        assert_eq!(with_streak("dl", 3).level("dl"), 1);
    }

    #[test]
    fn level_boundary_5_6() {
        assert_eq!(with_streak("dl", 5).level("dl"), 1);
        assert_eq!(with_streak("dl", 6).level("dl"), 2);
    }

    #[test]
    fn level_boundary_9_10() {
        assert_eq!(with_streak("dl", 9).level("dl"), 2);
        assert_eq!(with_streak("dl", 10).level("dl"), 3);
    }

    #[test]
    fn prompt_bands_follow_streak() {
        // Streak 2: prompt on every save, even odd indices.
        let backoff = with_streak("dl", 2);
        assert!(backoff.should_prompt("dl", 1));
        assert!(backoff.should_prompt("dl", 3));

        // Streak 3: every second save.
        let backoff = with_streak("dl", 3);
        assert!(!backoff.should_prompt("dl", 1));
        assert!(backoff.should_prompt("dl", 2));

        // Streak 5: still every second save.
        let backoff = with_streak("dl", 5);
        assert!(!backoff.should_prompt("dl", 1));
        assert!(backoff.should_prompt("dl", 2));

        // Streak 6: every fourth save.
        let backoff = with_streak("dl", 6);
        assert!(!backoff.should_prompt("dl", 2));
        assert!(backoff.should_prompt("dl", 4));

        // Streak 9: still every fourth save.
        let backoff = with_streak("dl", 9);
        assert!(!backoff.should_prompt("dl", 6));
        assert!(backoff.should_prompt("dl", 8));

        // Streak 10: every eighth save.
        let backoff = with_streak("dl", 10);
        assert!(!backoff.should_prompt("dl", 4));
        assert!(backoff.should_prompt("dl", 8));
    }

    #[test]
    fn two_misses_always_prompt() {
        let mut backoff = Backoff::default();
        backoff.record_bad("dl");
        backoff.record_bad("dl");
        // Prompt even on indices no band would normally prompt on.
        for index in [1u64, 3, 5, 7, 9] {
            assert!(backoff.should_prompt("dl", index));
        }
    }

    #[test]
    fn bad_save_resets_streak() {
        let mut backoff = with_streak("dl", 10);
        assert_eq!(backoff.level("dl"), 3);
        backoff.record_bad("dl");
        assert_eq!(backoff.level("dl"), 0);
        // Back in the always-prompt band.
        assert!(backoff.should_prompt("dl", 1));
    }

    #[test]
    fn good_save_resets_misses() {
        let mut backoff = Backoff::default();
        backoff.record_bad("dl");
        backoff.record_bad("dl");
        backoff.record_good("dl");
        backoff.record_good("dl");
        backoff.record_good("dl"); // streak 3, misses forgiven
        assert_eq!(backoff.level("dl"), 1);
        // The misses override no longer applies: odd index stays quiet.
        assert!(!backoff.should_prompt("dl", 1));
    }

    #[test]
    fn folders_are_tracked_independently() {
        let mut backoff = Backoff::default();
        for _ in 0..10 {
            backoff.record_good("downloads");
        }
        backoff.record_bad("desktop");

        assert_eq!(backoff.level("downloads"), 3);
        assert_eq!(backoff.level("desktop"), 0);
        assert!(!backoff.should_prompt("downloads", 1));
        assert!(backoff.should_prompt("desktop", 1));
    }

    #[test]
    fn folder_keys_are_case_insensitive() {
        let mut backoff = Backoff::default();
        backoff.record_good("Downloads");
        backoff.record_good("DOWNLOADS");
        backoff.record_good("downloads");
        assert_eq!(backoff.level("DoWnLoAdS"), 1);

        backoff.record_bad("DOWNLOADS");
        assert_eq!(backoff.level("downloads"), 0);
        // All three goods and the bad hit a single lowercased key.
        assert_eq!(backoff.streaks.len(), 1);
        assert_eq!(backoff.misses.len(), 1);
    }
}
