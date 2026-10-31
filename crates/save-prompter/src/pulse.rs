//! Commit nudges: how loudly to nag, from the gap since the last commit and the work piling up.

/// A snapshot of repository activity, handed in by the caller.
///
/// Pure data: this module never touches the clock, disk or network, so
/// callers supply `now_ms` and the counts themselves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoPulse {
    pub now_ms: u64,
    pub last_commit_ms: Option<u64>,
    pub saves_since_commit: u32,
    pub uncommitted_files: u32,
}

/// How loudly to nag the user about committing their work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Nudge {
    None,
    Gentle,
    Firm,
    Loud,
}

impl Nudge {
    /// Raise the nudge by `steps` levels, never past `Loud`.
    fn raised_by(self, steps: u32) -> Nudge {
        let mut nudge = self;
        for _ in 0..steps {
            nudge = match nudge {
                Nudge::None => Nudge::Gentle,
                Nudge::Gentle => Nudge::Firm,
                Nudge::Firm | Nudge::Loud => Nudge::Loud,
            };
        }
        nudge
    }
}

/// Minutes-since-commit thresholds `(gentle, firm, loud)` for a strictness
/// that has already been clamped to `0..=3`.
fn thresholds(strictness: u8) -> (u64, u64, u64) {
    match strictness {
        0 => (240, 480, 1440),
        1 => (120, 240, 720),
        2 => (60, 120, 360),
        _ => (30, 60, 180),
    }
}

/// How loudly to nag, given the repo's pulse and how strict we feel.
///
/// `strictness` is clamped to `0..=3`.
pub fn commit_nudge(p: &RepoPulse, strictness: u8) -> Nudge {
    let strictness = strictness.min(3);

    // No work at all, so there is no missed commit to nag about.
    if p.saves_since_commit == 0 && p.uncommitted_files == 0 {
        return Nudge::None;
    }

    // A missing commit timestamp behaves like a day-old commit.
    let minutes = match p.last_commit_ms {
        Some(last) => p.now_ms.saturating_sub(last) / 60_000,
        None => 24 * 60,
    };

    // A fresh commit resets the nag, however much work piled up since.
    if minutes < 10 {
        return Nudge::None;
    }

    let (gentle, firm, loud) = thresholds(strictness);
    let base = if minutes < gentle {
        Nudge::None
    } else if minutes < firm {
        Nudge::Gentle
    } else if minutes < loud {
        Nudge::Firm
    } else {
        Nudge::Loud
    };

    // Piled-up files turn the volume up: one step from 20 files, two from
    // 50, capped at `Loud`.
    let steps = if p.uncommitted_files >= 50 {
        2
    } else if p.uncommitted_files >= 20 {
        1
    } else {
        0
    };
    base.raised_by(steps)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A pulse whose last commit was `minutes_ago` minutes before `now_ms`.
    fn pulse(minutes_ago: u64, saves: u32, files: u32) -> RepoPulse {
        RepoPulse {
            now_ms: minutes_ago * 60_000,
            last_commit_ms: Some(0),
            saves_since_commit: saves,
            uncommitted_files: files,
        }
    }

    #[test]
    fn no_work_is_none_at_any_gap() {
        for &minutes in &[0u64, 5, 10, 59, 720, 10_000] {
            for strictness in 0..=3 {
                let p = pulse(minutes, 0, 0);
                assert_eq!(
                    commit_nudge(&p, strictness),
                    Nudge::None,
                    "minutes={minutes} strictness={strictness}"
                );
            }
        }
    }

    #[test]
    fn fresh_commit_beats_even_mountains_of_files() {
        for strictness in 0..=3 {
            // Five minutes old, with piles of saves and files: too fresh.
            assert_eq!(commit_nudge(&pulse(5, 100, 100), strictness), Nudge::None);
            // Nine minutes is the last minute under the wire.
            assert_eq!(commit_nudge(&pulse(9, 100, 100), strictness), Nudge::None);
        }
    }

    #[test]
    fn ten_minutes_is_old_enough_to_nag_again() {
        // Strictness 0 has a None base at ten minutes, but 100 files bump it
        // two steps up to Firm.
        assert_eq!(commit_nudge(&pulse(10, 1, 100), 0), Nudge::Firm);
    }

    #[test]
    fn strictness_0_and_3_disagree_at_90_minutes() {
        let p = pulse(90, 1, 0);
        assert_eq!(commit_nudge(&p, 0), Nudge::None);
        assert_eq!(commit_nudge(&p, 3), Nudge::Firm);
    }

    #[test]
    fn threshold_boundaries_at_strictness_2() {
        // (gentle, firm, loud) = (60, 120, 360) minutes at strictness 2.
        let cases = [
            (59, Nudge::None),
            (60, Nudge::Gentle),
            (119, Nudge::Gentle),
            (120, Nudge::Firm),
            (359, Nudge::Firm),
            (360, Nudge::Loud),
        ];
        for &(minutes, expected) in &cases {
            let p = pulse(minutes, 1, 0);
            assert_eq!(commit_nudge(&p, 2), expected, "minutes={minutes}");
        }
    }

    #[test]
    fn uncommitted_files_turn_the_volume_up() {
        // Base Gentle at strictness 2: 90 minutes sits in [60, 120).
        let cases = [
            (19, Nudge::Gentle),  // below the first bump
            (20, Nudge::Firm),    // one step up
            (49, Nudge::Firm),
            (50, Nudge::Loud),    // two steps up
            (5_000, Nudge::Loud), // and Loud is the ceiling
        ];
        for &(files, expected) in &cases {
            let p = pulse(90, 1, files);
            assert_eq!(commit_nudge(&p, 2), expected, "files={files}");
        }
    }

    #[test]
    fn loud_is_the_ceiling() {
        // Base Loud at strictness 2 (400 >= 360); bumps cannot go higher.
        for &files in &[0u32, 20, 50, 1_000] {
            let p = pulse(400, 1, files);
            assert_eq!(commit_nudge(&p, 2), Nudge::Loud, "files={files}");
        }
    }

    #[test]
    fn quiet_gaps_still_respond_to_piled_up_files() {
        // 30 minutes at strictness 2 is below the gentle threshold (base
        // None), but a real pile of files still turns the volume up.
        assert_eq!(commit_nudge(&pulse(30, 1, 19), 2), Nudge::None);
        assert_eq!(commit_nudge(&pulse(30, 1, 20), 2), Nudge::Gentle);
        assert_eq!(commit_nudge(&pulse(30, 1, 50), 2), Nudge::Firm);
    }

    #[test]
    fn missing_commit_timestamp_behaves_as_24_hours() {
        let no_ts = RepoPulse {
            now_ms: 24 * 60 * 60_000,
            last_commit_ms: None,
            saves_since_commit: 1,
            uncommitted_files: 0,
        };
        let day_old = pulse(24 * 60, 1, 0);
        for strictness in 0..=3 {
            assert_eq!(
                commit_nudge(&no_ts, strictness),
                commit_nudge(&day_old, strictness),
                "strictness={strictness}"
            );
        }
        // A full day sits exactly on the loud threshold at strictness 0.
        assert_eq!(commit_nudge(&no_ts, 0), Nudge::Loud);
    }

    #[test]
    fn strictness_above_three_behaves_as_three() {
        let p = pulse(90, 1, 0);
        assert_eq!(commit_nudge(&p, 9), commit_nudge(&p, 3));
        assert_eq!(commit_nudge(&p, 9), Nudge::Firm);
        assert_eq!(commit_nudge(&p, u8::MAX), commit_nudge(&p, 3));
    }

    #[test]
    fn more_uncommitted_files_never_lower_the_nudge() {
        for &minutes in &[30u64, 90, 400] {
            for strictness in 0..=3 {
                let mut previous = commit_nudge(&pulse(minutes, 1, 0), strictness);
                for files in 1..80 {
                    let current = commit_nudge(&pulse(minutes, 1, files), strictness);
                    assert!(
                        current >= previous,
                        "minutes={minutes} strictness={strictness} files={files}: \
                         {current:?} dipped below {previous:?}"
                    );
                    previous = current;
                }
            }
        }
    }
}
