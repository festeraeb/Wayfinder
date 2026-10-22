//! Mr. Clippy's lines: sarcastic about the file, never about the person, and softer as the folder's
//! back-off level rises (the user has been saving well there).

use crate::hints::HintReason;

/// Tone band from the back-off level: 0 = full sarcasm, 1 = dry, 2 and up = gentle.
pub fn tone(level: u8) -> usize {
    (level as usize).min(2)
}

const DUMPING_GROUND: [[&str; 3]; 3] = [
    [
        "Ah, Downloads. Where files go to be lost forever. Want to give this one a real home?",
        "The Desktop called. It's full. Shall we move this somewhere it can be found again?",
        "Saving straight into Documents? Bold. A subfolder would make future-you very happy.",
    ],
    [
        "This landed in a catch-all folder. Want to file it now while you remember what it is?",
        "Quick one: this folder is a junk drawer. Move it somewhere that matches what it is?",
        "Future-you will search for this. Want to put it where future-you will look?",
    ],
    [
        "You've been filing things nicely. This one's in a catch-all folder; want to move it?",
        "Small thing: this ended up in a general folder. Want a better spot for it?",
        "Nearly perfect streak. Want to give this file a proper folder too?",
    ],
];

const PLACEHOLDER: [[&str; 3]; 3] = [
    [
        "'Untitled'. A classic. Truly says everything about what's inside.",
        "Another 'New Document'. Your files are starting a support group.",
        "This name tells me nothing, and I'm a paperclip. Try the suggestion?",
    ],
    [
        "This still has its default name. Want the suggested one instead?",
        "Default name spotted. A real name now saves a search later.",
        "Placeholder name. Two seconds now, five minutes saved later.",
    ],
    [
        "Just the default name left to fix. Want to use the suggestion?",
        "Almost there: give this one a real name and you're set.",
        "One quick rename and this file is as tidy as the rest.",
    ],
];

const COPY_OR_VERSION: [[&str; 3]; 3] = [
    [
        "'final (2)'. So it wasn't final. It never is.",
        "A copy of a copy. We're one step from a photocopier art project.",
        "'_old' and still here? Bold of it. Keep, rename or archive?",
    ],
    [
        "This looks like a copy or an old version. Keep it, rename it, or archive the old one?",
        "Version clutter detected. Want one clear name instead of 'final', 'final (2)'?",
        "Copies pile up fast. Want to keep just the one you need?",
    ],
    [
        "Looks like a duplicate version. Want to tidy it into one file?",
        "A small cleanup: this seems to be an extra copy. Keep or archive?",
        "You're keeping things neat; this copy is the last loose end.",
    ],
];

const SPACES: [[&str; 3]; 3] = [
    [
        "That name has more spaces than a parking lot. Want the cleaned-up version?",
        "Double spaces in a file name: technically allowed, emotionally confusing.",
        "This name needs a tidy. The suggestion has fewer mysterious gaps.",
    ],
    [
        "The name has extra spaces. Want the cleaned-up suggestion?",
        "Small fix: collapse the extra spaces in this name?",
        "Tidier name available, same meaning, fewer gaps.",
    ],
    [
        "Just some extra spaces in the name. Want the tidy version?",
        "Tiny tweak: the suggested name drops the extra spaces.",
        "Nearly there: clean up the spaces and it's perfect.",
    ],
];

/// One line for a save prompt: the first reason picks the theme, the level picks the tone, and `seed`
/// (for example the save number) picks one of three lines so Clippy doesn't repeat itself every time.
pub fn clippy_line(reasons: &[HintReason], level: u8, seed: u64) -> &'static str {
    let table = match reasons.first() {
        Some(HintReason::DumpingGround) => &DUMPING_GROUND,
        Some(HintReason::PlaceholderName) => &PLACEHOLDER,
        Some(HintReason::CopyOrVersionSuffix) => &COPY_OR_VERSION,
        Some(HintReason::SpacesAndCaps) => &SPACES,
        None => return "",
    };
    table[tone(level)][(seed % 3) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_reasons_means_no_line() {
        assert_eq!(clippy_line(&[], 0, 0), "");
    }

    #[test]
    fn first_reason_picks_the_theme_and_level_softens_the_tone() {
        let r = [HintReason::PlaceholderName, HintReason::DumpingGround];
        assert_eq!(clippy_line(&r, 0, 0), PLACEHOLDER[0][0]);
        assert_eq!(clippy_line(&r, 1, 0), PLACEHOLDER[1][0]);
        assert_eq!(clippy_line(&r, 2, 0), PLACEHOLDER[2][0]);
        assert_eq!(clippy_line(&r, 9, 0), PLACEHOLDER[2][0], "levels above 2 stay gentle");
    }

    #[test]
    fn seed_rotates_through_three_lines() {
        let r = [HintReason::CopyOrVersionSuffix];
        let lines: Vec<&str> = (0..6).map(|s| clippy_line(&r, 0, s)).collect();
        assert_eq!(lines[0], lines[3]);
        assert_ne!(lines[0], lines[1]);
        assert_ne!(lines[1], lines[2]);
    }

    #[test]
    fn lines_never_insult_the_person() {
        let banned = ["stupid", "idiot", "lazy", "dumb", "you always", "you never", "loser"];
        for table in [&DUMPING_GROUND, &PLACEHOLDER, &COPY_OR_VERSION, &SPACES] {
            for band in table.iter() {
                for line in band.iter() {
                    let l = line.to_lowercase();
                    assert!(!line.is_empty());
                    assert!(banned.iter().all(|b| !l.contains(b)), "{line}");
                }
            }
        }
    }
}
