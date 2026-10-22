//! Taxonomy: map a request to one coarse category label on the device.
//!
//! Classification is purely local and stateless: the request text is never
//! returned or stored, only a coarse [`Category`] label is produced.

/// One coarse category for a user request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    RenameFile,
    FindFile,
    OrganizeFolder,
    TaskBreakdown,
    GitHelp,
    ExplainCode,
    WritingHelp,
    Other,
}

impl Category {
    /// The stable snake_case name for this category.
    pub fn label(&self) -> &'static str {
        match self {
            Category::RenameFile => "rename_file",
            Category::FindFile => "find_file",
            Category::OrganizeFolder => "organize_folder",
            Category::TaskBreakdown => "task_breakdown",
            Category::GitHelp => "git_help",
            Category::ExplainCode => "explain_code",
            Category::WritingHelp => "writing_help",
            Category::Other => "other",
        }
    }
}

/// Keyword rules in priority order; the first matching rule wins.
///
/// Keywords are stored already normalised (lowercase, single spaces, and a
/// leading and trailing space) so they can be matched against a normalised
/// request with a plain substring test that respects word boundaries.
const RULES: &[(&[&str], Category)] = &[
    (
        &[" rename ", " name this ", " file name ", " filename "],
        Category::RenameFile,
    ),
    (
        &[" find ", " search ", " where is ", " locate "],
        Category::FindFile,
    ),
    (
        &[
            " organize ",
            " organise ",
            " clean up ",
            " sort ",
            " folder ",
            " move ",
        ],
        Category::OrganizeFolder,
    ),
    (
        &[
            " break down ",
            " steps ",
            " plan ",
            " todo ",
            " to do ",
            " tasks ",
        ],
        Category::TaskBreakdown,
    ),
    (
        &[
            " git ",
            " commit ",
            " branch ",
            " merge ",
            " push ",
            " pull request ",
        ],
        Category::GitHelp,
    ),
    (
        &[
            " explain ",
            " what does ",
            " error ",
            " bug ",
            " compile ",
            " function ",
        ],
        Category::ExplainCode,
    ),
    (
        &[
            " essay ",
            " paragraph ",
            " summary ",
            " summarize ",
            " summarise ",
            " write ",
        ],
        Category::WritingHelp,
    ),
];

/// Normalise `text` for whole-word matching.
///
/// The result is lowercase, every run of non-alphanumeric characters becomes a
/// single space, and the whole string is padded with a leading and trailing
/// space. This makes `" keyword "` a word-boundary-aware substring test.
fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push(' ');
    let mut pending_space = false;
    for c in text.chars() {
        if c.is_alphanumeric() {
            if pending_space {
                out.push(' ');
                pending_space = false;
            }
            for lower in c.to_lowercase() {
                out.push(lower);
            }
        } else {
            pending_space = true;
        }
    }
    out.push(' ');
    out
}

/// Classify a request into one coarse [`Category`].
///
/// Matching is case-insensitive and on word boundaries; the first rule that
/// matches wins. An empty or whitespace-only request is [`Category::Other`].
/// No part of `request` is returned or stored.
pub fn classify(request: &str) -> Category {
    let normalized = normalize(request);
    for (keywords, category) in RULES {
        for keyword in *keywords {
            if normalized.contains(keyword) {
                return *category;
            }
        }
    }
    Category::Other
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_unique() {
        let all = [
            Category::RenameFile,
            Category::FindFile,
            Category::OrganizeFolder,
            Category::TaskBreakdown,
            Category::GitHelp,
            Category::ExplainCode,
            Category::WritingHelp,
            Category::Other,
        ];
        let mut labels: Vec<&str> = all.iter().map(Category::label).collect();
        labels.sort_unstable();
        labels.dedup();
        assert_eq!(labels.len(), all.len());
    }

    #[test]
    fn labels_are_snake_case() {
        assert_eq!(Category::RenameFile.label(), "rename_file");
        assert_eq!(Category::FindFile.label(), "find_file");
        assert_eq!(Category::OrganizeFolder.label(), "organize_folder");
        assert_eq!(Category::TaskBreakdown.label(), "task_breakdown");
        assert_eq!(Category::GitHelp.label(), "git_help");
        assert_eq!(Category::ExplainCode.label(), "explain_code");
        assert_eq!(Category::WritingHelp.label(), "writing_help");
        assert_eq!(Category::Other.label(), "other");
    }

    #[test]
    fn one_example_per_category() {
        assert_eq!(classify("rename this file"), Category::RenameFile);
        assert_eq!(classify("find my notes"), Category::FindFile);
        assert_eq!(classify("organize my folder"), Category::OrganizeFolder);
        assert_eq!(classify("break down this project"), Category::TaskBreakdown);
        assert_eq!(classify("help me with git"), Category::GitHelp);
        assert_eq!(classify("explain this function"), Category::ExplainCode);
        assert_eq!(classify("write an essay"), Category::WritingHelp);
        assert_eq!(classify("hello there"), Category::Other);
    }

    #[test]
    fn rule_order_first_match_wins() {
        assert_eq!(classify("rename and git"), Category::RenameFile);
        assert_eq!(classify("find and organize"), Category::FindFile);
        assert_eq!(classify("organize the steps"), Category::OrganizeFolder);
        assert_eq!(classify("plan a git commit"), Category::TaskBreakdown);
        assert_eq!(classify("git error"), Category::GitHelp);
        assert_eq!(classify("explain the summary"), Category::ExplainCode);
    }

    #[test]
    fn word_boundaries_are_respected() {
        assert_eq!(classify("pushing buttons"), Category::Other);
        assert_eq!(classify("cleanup time"), Category::Other);
        assert_eq!(classify("searching around"), Category::Other);
        assert_eq!(classify("filename"), Category::RenameFile);
        assert_eq!(classify("filenames"), Category::Other);
    }

    #[test]
    fn punctuation_counts_as_boundary() {
        assert_eq!(classify("rename, please!"), Category::RenameFile);
        assert_eq!(classify("find? where is: it"), Category::FindFile);
        assert_eq!(classify("organize_folder"), Category::OrganizeFolder);
        assert_eq!(classify("git: commit, merge."), Category::GitHelp);
        assert_eq!(classify("(explain)"), Category::ExplainCode);
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(classify("RENAME"), Category::RenameFile);
        assert_eq!(classify("Find"), Category::FindFile);
        assert_eq!(classify("ORGANIZE"), Category::OrganizeFolder);
        assert_eq!(classify("Git"), Category::GitHelp);
        assert_eq!(classify("ExPlAiN"), Category::ExplainCode);
    }

    #[test]
    fn empty_and_whitespace_are_other() {
        assert_eq!(classify(""), Category::Other);
        assert_eq!(classify("   "), Category::Other);
        assert_eq!(classify("\t\n"), Category::Other);
    }

    #[test]
    fn multi_word_keywords() {
        assert_eq!(classify("name this thing"), Category::RenameFile);
        assert_eq!(classify("where is my file"), Category::FindFile);
        assert_eq!(classify("clean up the desk"), Category::OrganizeFolder);
        assert_eq!(classify("break down the work"), Category::TaskBreakdown);
        assert_eq!(classify("open a pull request"), Category::GitHelp);
        assert_eq!(classify("what does this do"), Category::ExplainCode);
    }
}
