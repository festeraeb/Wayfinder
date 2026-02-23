// Git Clippy Assistant - Helpful git companion for ADHD developers

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::commands::FileEntry;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitStatus {
    pub is_repo: bool,
    pub branch: String,
    pub uncommitted_files: usize,
    pub staged_files: usize,
    pub untracked_files: usize,
    pub days_since_commit: i64,
    pub last_commit_message: Option<String>,
    pub last_commit_date: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DuplicateFile {
    pub original: String,
    pub duplicates: Vec<String>,
    pub content_hash: String,
    pub size_bytes: u64,
    pub kept_mtime: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileSuggestion {
    pub file_path: String,
    pub suggestion: String,
    pub action: String,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommitSuggestion {
    pub files: Vec<String>,
    pub suggested_message: String,
    pub category: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitClippyReport {
    pub status: GitStatus,
    pub urgency_level: String,
    pub message: String,
    pub suggestions: Vec<ClippySuggestion>,
    pub duplicates: Vec<DuplicateFile>,
    pub commit_suggestions: Vec<CommitSuggestion>,
    pub copy_pattern_files: Vec<FileSuggestion>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClippySuggestion {
    pub id: String,
    pub icon: String,
    pub title: String,
    pub description: String,
    pub actions: Vec<ClippyAction>,
    pub priority: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClippyAction {
    pub label: String,
    pub action_type: String,
    pub data: Option<serde_json::Value>,
}

/// Run a git command and return output (with hidden window on Windows)
fn run_git_command(repo_path: &str, args: &[&str]) -> Result<String, String> {
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;
    
    let mut cmd = Command::new("git");
    cmd.args(args).current_dir(repo_path);
    
    // Hide the console window on Windows
    #[cfg(windows)]
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    
    let output = cmd.output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Check if a path is a git repository
pub fn is_git_repo(path: &str) -> bool {
    Path::new(path).join(".git").exists()
}

/// Get comprehensive git status
pub fn get_git_status(repo_path: &str) -> Result<GitStatus, String> {
    if !is_git_repo(repo_path) {
        return Ok(GitStatus {
            is_repo: false,
            branch: String::new(),
            uncommitted_files: 0,
            staged_files: 0,
            untracked_files: 0,
            days_since_commit: 0,
            last_commit_message: None,
            last_commit_date: None,
        });
    }

    // Get current branch
    let branch = run_git_command(repo_path, &["branch", "--show-current"])
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

    // Get status --porcelain for file counts
    let status_output = run_git_command(repo_path, &["status", "--porcelain"]).unwrap_or_default();

    let mut staged = 0;
    let mut uncommitted = 0;
    let mut untracked = 0;

    for line in status_output.lines() {
        if line.len() < 2 {
            continue;
        }
        let status_chars: Vec<char> = line.chars().take(2).collect();

        match (status_chars.get(0), status_chars.get(1)) {
            (Some('?'), Some('?')) => untracked += 1,
            (Some(' '), Some(_)) => uncommitted += 1,
            (Some(_), Some(' ')) => staged += 1,
            (Some(_), Some(_)) => {
                staged += 1;
                uncommitted += 1;
            }
            _ => {}
        }
    }

    // Get last commit info
    let last_commit_message = run_git_command(repo_path, &["log", "-1", "--format=%s"])
        .ok()
        .map(|s| s.trim().to_string());

    let last_commit_date = run_git_command(repo_path, &["log", "-1", "--format=%ci"])
        .ok()
        .map(|s| s.trim().to_string());

    // Calculate days since last commit
    let days_since_commit = if let Some(ref date_str) = last_commit_date {
        if let Ok(commit_date) = DateTime::parse_from_str(
            &format!("{} +0000", &date_str.get(..19).unwrap_or(date_str)),
            "%Y-%m-%d %H:%M:%S %z",
        ) {
            let now = Utc::now();
            let duration = now.signed_duration_since(commit_date.with_timezone(&Utc));
            duration.num_days()
        } else {
            0
        }
    } else {
        0
    };

    Ok(GitStatus {
        is_repo: true,
        branch,
        uncommitted_files: uncommitted + untracked,
        staged_files: staged,
        untracked_files: untracked,
        days_since_commit,
        last_commit_message,
        last_commit_date,
    })
}

/// Find duplicate files in the repository
pub fn find_duplicates(files: &[FileEntry]) -> Vec<DuplicateFile> {
    use std::collections::hash_map::DefaultHasher;
    use std::fs;
    use std::hash::{Hash, Hasher};

    #[derive(Clone)]
    struct FileMeta {
        path: String,
        size: u64,
        mtime: Option<std::time::SystemTime>,
    }

    let mut content_map: HashMap<(u64, u64), Vec<FileMeta>> = HashMap::new();

    for file in files {
        if let Ok(metadata) = fs::metadata(&file.path) {
            let size = metadata.len();
            // Skip zero-size files for duplicate detection
            if size == 0 {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&file.path) {
                let mut hasher = DefaultHasher::new();
                content.hash(&mut hasher);
                let hash = hasher.finish();

                content_map
                    .entry((size, hash))
                    .or_insert_with(Vec::new)
                    .push(FileMeta {
                        path: file.path.clone(),
                        size,
                        mtime: metadata.modified().ok(),
                    });
            }
        }
    }

    let mut duplicates: Vec<DuplicateFile> = Vec::new();

    for ((size, hash), metas) in content_map {
        if metas.len() > 1 {
            let mut metas_sorted = metas.clone();
            metas_sorted.sort_by(|a, b| b.mtime.cmp(&a.mtime)); // newest first

            let keep = metas_sorted.remove(0);
            let kept_mtime = keep
                .mtime
                .and_then(|t: std::time::SystemTime| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
                .map(|d: std::time::Duration| d.as_secs().to_string());

            duplicates.push(DuplicateFile {
                original: keep.path,
                duplicates: metas_sorted.into_iter().map(|m| m.path).collect(),
                content_hash: format!("{:x}", hash),
                size_bytes: size,
                kept_mtime,
            });
        }
    }

    duplicates
}

/// Detect copy/backup naming patterns
pub fn detect_copy_patterns(files: &[FileEntry]) -> Vec<FileSuggestion> {
    let copy_patterns = [
        "_copy", "_backup", "_old", "_new", "_working", "_temp", "_COPY", "_BACKUP", "_OLD",
        "_NEW", "_WORKING", "_TEMP", " copy", " Copy", " (copy)", " - Copy", "_v1", "_v2", "_v3",
        "_v4", "_v5", "_final", "_FINAL", "_final_v2", "_ACTUAL", "_ACTUAL_working", "_no_really",
        "_this_one_works", "_latest", "_LATEST",
    ];

    let mut suggestions: Vec<FileSuggestion> = Vec::new();
    let mut pattern_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for file in files {
        let name = &file.name;
        let stem = Path::new(name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        for pattern in &copy_patterns {
            if stem.to_lowercase().contains(&pattern.to_lowercase()) {
                *pattern_counts.entry((*pattern).to_string()).or_insert(0) += 1;
                
                let potential_original = stem.replace(pattern, "");
                let ext = Path::new(name)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");

                let snarky_reason = match pattern_counts.get(*pattern).unwrap_or(&1) {
                    1..=3 => format!("Contains '{}' pattern", pattern),
                    4..=10 => format!("Another '{}' file? There's {} of them now...", pattern, pattern_counts.get(*pattern).unwrap_or(&1)),
                    11..=20 => format!("This is the {}th file with '{}'. Maybe we should talk about your branching strategy?", pattern_counts.get(*pattern).unwrap_or(&1), pattern),
                    _ => format!("{}th file with '{}' in the name. At this point it's a collection. 📋📋📋", pattern_counts.get(*pattern).unwrap_or(&1), pattern),
                };

                suggestions.push(FileSuggestion {
                    file_path: file.path.clone(),
                    suggestion: format!("Looks like a copy of '{}.{}'", potential_original, ext),
                    action: "review".to_string(),
                    reason: snarky_reason,
                });
                break;
            }
        }
    }

    suggestions
}

/// Generate smart commit suggestions based on file patterns
pub fn suggest_commits(repo_path: &str) -> Result<Vec<CommitSuggestion>, String> {
    let status_output = run_git_command(repo_path, &["status", "--porcelain"])?;

    let mut files_by_dir: HashMap<String, Vec<String>> = HashMap::new();
    let mut files_by_ext: HashMap<String, Vec<String>> = HashMap::new();

    for line in status_output.lines() {
        if line.len() < 3 {
            continue;
        }
        let file_path = line[3..].trim();

        let dir = Path::new(file_path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("root")
            .to_string();

        files_by_dir
            .entry(dir)
            .or_insert_with(Vec::new)
            .push(file_path.to_string());

        let ext = Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("none")
            .to_string();

        files_by_ext
            .entry(ext)
            .or_insert_with(Vec::new)
            .push(file_path.to_string());
    }

    let mut suggestions: Vec<CommitSuggestion> = Vec::new();

    for (dir, files) in &files_by_dir {
        if files.len() >= 2 {
            let dir_name = Path::new(dir)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(dir);

            suggestions.push(CommitSuggestion {
                files: files.clone(),
                suggested_message: format!("Update {} files in {}", files.len(), dir_name),
                category: "feature".to_string(),
            });
        }
    }

    let config_exts = ["json", "yaml", "yml", "toml", "ini", "cfg"];
    let config_files: Vec<String> = files_by_ext
        .iter()
        .filter(|(ext, _)| config_exts.contains(&ext.as_str()))
        .flat_map(|(_, files)| files.clone())
        .collect();

    if config_files.len() >= 2 {
        suggestions.push(CommitSuggestion {
            files: config_files,
            suggested_message: "Update configuration files".to_string(),
            category: "config".to_string(),
        });
    }

    let doc_exts = ["md", "txt", "rst", "doc"];
    let doc_files: Vec<String> = files_by_ext
        .iter()
        .filter(|(ext, _)| doc_exts.contains(&ext.as_str()))
        .flat_map(|(_, files)| files.clone())
        .collect();

    if !doc_files.is_empty() {
        suggestions.push(CommitSuggestion {
            files: doc_files,
            suggested_message: "Update documentation".to_string(),
            category: "docs".to_string(),
        });
    }

    Ok(suggestions)
}

/// Generate the full Clippy report
pub fn generate_clippy_report(
    repo_path: &str,
    index_files: Option<&[FileEntry]>,
) -> Result<GitClippyReport, String> {
    let status = get_git_status(repo_path)?;

    let mut suggestions: Vec<ClippySuggestion> = Vec::new();
    let mut duplicates: Vec<DuplicateFile> = Vec::new();
    let mut copy_pattern_files: Vec<FileSuggestion> = Vec::new();

    // Check uncommitted files
    if status.uncommitted_files > 200 {
        suggestions.push(ClippySuggestion {
            id: "extreme_uncommitted".to_string(),
            icon: "🚨".to_string(),
            title: format!("{} modified files in working directory", status.uncommitted_files),
            description: format!(
                "📎 \"I notice you haven't committed in {} days.\n    Your working directory has {} modified files.\n    Should I...\"",
                status.days_since_commit, status.uncommitted_files
            ),
            actions: vec![
                ClippyAction {
                    label: "Commit everything".to_string(),
                    action_type: "wip_commit".to_string(),
                    data: None,
                },
                ClippyAction {
                    label: "Create panic backup".to_string(),
                    action_type: "panic_backup".to_string(),
                    data: None,
                },
                ClippyAction {
                    label: "Cry".to_string(),
                    action_type: "cry".to_string(),
                    data: None,
                },
            ],
            priority: 10,
        });
    } else if status.uncommitted_files > 50 {
        suggestions.push(ClippySuggestion {
            id: "many_uncommitted".to_string(),
            icon: "⚠️".to_string(),
            title: format!("{} uncommitted files", status.uncommitted_files),
            description:
                "That's a lot of changes. Your future self might thank you for committing. Or at least making a backup before Claude starts making copies..."
                    .to_string(),
            actions: vec![
                ClippyAction {
                    label: "Smart commit".to_string(),
                    action_type: "commit".to_string(),
                    data: None,
                },
                ClippyAction {
                    label: "Review changes".to_string(),
                    action_type: "review".to_string(),
                    data: None,
                },
                ClippyAction {
                    label: "Panic mode (commit all as WIP)".to_string(),
                    action_type: "wip_commit".to_string(),
                    data: None,
                },
            ],
            priority: 4,
        });
    } else if status.uncommitted_files > 10 {
        suggestions.push(ClippySuggestion {
            id: "some_uncommitted".to_string(),
            icon: "📝".to_string(),
            title: format!("{} uncommitted files", status.uncommitted_files),
            description: "Good progress! Consider committing related changes together.".to_string(),
            actions: vec![
                ClippyAction {
                    label: "Smart commit".to_string(),
                    action_type: "commit".to_string(),
                    data: None,
                },
                ClippyAction {
                    label: "Later".to_string(),
                    action_type: "dismiss".to_string(),
                    data: None,
                },
            ],
            priority: 2,
        });
    }

    // Check days since commit
    if status.days_since_commit > 7 {
        suggestions.push(ClippySuggestion {
            id: "long_since_commit".to_string(),
            icon: "⏰".to_string(),
            title: format!("{} days since last commit", status.days_since_commit),
            description: "It's been a while! Even a WIP commit is better than losing work."
                .to_string(),
            actions: vec![
                ClippyAction {
                    label: "Quick WIP save".to_string(),
                    action_type: "wip_commit".to_string(),
                    data: None,
                },
                ClippyAction {
                    label: "I know what I'm doing".to_string(),
                    action_type: "dismiss".to_string(),
                    data: None,
                },
            ],
            priority: 5,
        });
    } else if status.days_since_commit > 3 {
        suggestions.push(ClippySuggestion {
            id: "few_days_since_commit".to_string(),
            icon: "💭".to_string(),
            title: format!("{} days since last commit", status.days_since_commit),
            description: "Just a friendly reminder to save your progress.".to_string(),
            actions: vec![
                ClippyAction {
                    label: "Commit now".to_string(),
                    action_type: "commit".to_string(),
                    data: None,
                },
                ClippyAction {
                    label: "Remind me tomorrow".to_string(),
                    action_type: "snooze".to_string(),
                    data: Some(serde_json::json!({"hours": 24})),
                },
            ],
            priority: 2,
        });
    }

    // Find duplicates if we have index data
    if let Some(files) = index_files {
        duplicates = find_duplicates(files);

        if !duplicates.is_empty() {
            let total_dupes: usize = duplicates.iter().map(|d| d.duplicates.len()).sum();

            suggestions.push(ClippySuggestion {
                id: "duplicates_found".to_string(),
                icon: "🗑️".to_string(),
                title: format!("{} duplicate files found", total_dupes),
                description: "Same content, different names. Want to clean up?".to_string(),
                actions: vec![
                    ClippyAction {
                        label: "Show me".to_string(),
                        action_type: "show_duplicates".to_string(),
                        data: None,
                    },
                    ClippyAction {
                        label: "Clean up safely".to_string(),
                        action_type: "cleanup".to_string(),
                        data: None,
                    },
                    ClippyAction {
                        label: "I'm a hoarder, leave them".to_string(),
                        action_type: "dismiss".to_string(),
                        data: None,
                    },
                ],
                priority: 3,
            });
        }

        let copy_suggestions = detect_copy_patterns(files);
        copy_pattern_files = copy_suggestions.clone();
        if copy_suggestions.len() > 10 {
            suggestions.push(ClippySuggestion {
                id: "copy_patterns_extreme".to_string(),
                icon: "📋".to_string(),
                title: format!("You just created your {}th file with 'copy' in the name", copy_suggestions.len()),
                description: format!(
                    "📎 \"This is the {}th file with 'copy' in the name.\n    Maybe we should talk about your branching strategy?\"\n\nFound: _copy, _backup, _final, _v2, _ACTUAL_working...",
                    copy_suggestions.len()
                ),
                actions: vec![
                    ClippyAction {
                        label: "Review them".to_string(),
                        action_type: "show_copies".to_string(),
                        data: None,
                    },
                    ClippyAction {
                        label: "Learn about git branches".to_string(),
                        action_type: "learn_branches".to_string(),
                        data: None,
                    },
                    ClippyAction {
                        label: "I'm a hoarder, leave them".to_string(),
                        action_type: "dismiss".to_string(),
                        data: None,
                    },
                ],
                priority: 4,
            });
        } else if copy_suggestions.len() > 3 {
            suggestions.push(ClippySuggestion {
                id: "copy_patterns".to_string(),
                icon: "📋".to_string(),
                title: format!("{} files with copy/backup patterns", copy_suggestions.len()),
                description: "Found files like '_copy', '_old', '_backup'. Need help organizing?"
                    .to_string(),
                actions: vec![
                    ClippyAction {
                        label: "Review them".to_string(),
                        action_type: "show_copies".to_string(),
                        data: None,
                    },
                    ClippyAction {
                        label: "Ignore".to_string(),
                        action_type: "dismiss".to_string(),
                        data: None,
                    },
                ],
                priority: 2,
            });
        }
    }

    // Check if on main branch with uncommitted changes
    if (status.branch == "main" || status.branch == "master") && status.uncommitted_files > 5 {
        suggestions.push(ClippySuggestion {
            id: "experimenting_on_main".to_string(),
            icon: "🌿".to_string(),
            title: "Experimenting on main branch".to_string(),
            description: "You're making changes directly on main. Want to create a feature branch?".to_string(),
            actions: vec![
                ClippyAction {
                    label: "Create branch".to_string(),
                    action_type: "create_branch".to_string(),
                    data: Some(serde_json::json!({
                        "suggested_name": format!("feature-{}", chrono::Local::now().format("%Y%m%d"))
                    })),
                },
                ClippyAction {
                    label: "YOLO on main".to_string(),
                    action_type: "dismiss".to_string(),
                    data: None,
                },
            ],
            priority: 3,
        });
    }

    // Check for merge conflicts or merge in progress
    let merge_head = Path::new(repo_path).join(".git").join("MERGE_HEAD");
    if merge_head.exists() {
        suggestions.push(ClippySuggestion {
            id: "merge_in_progress".to_string(),
            icon: "🏃".to_string(),
            title: "It looks like you're trying to merge!".to_string(),
            description: "📎 \"It looks like you're trying to merge!\n    Just kidding, I'm backing away slowly.\n    You're on your own with this one. Good luck! 🏃\"\n\n(But seriously, finish the merge or abort it)".to_string(),
            actions: vec![
                ClippyAction {
                    label: "I know what I'm doing".to_string(),
                    action_type: "dismiss".to_string(),
                    data: None,
                },
                ClippyAction {
                    label: "Abort merge".to_string(),
                    action_type: "abort_merge".to_string(),
                    data: None,
                },
                ClippyAction {
                    label: "Pray".to_string(),
                    action_type: "pray".to_string(),
                    data: None,
                },
            ],
            priority: 8,
        });
    }

    // Check for rebase in progress
    let rebase_merge = Path::new(repo_path).join(".git").join("rebase-merge");
    let rebase_apply = Path::new(repo_path).join(".git").join("rebase-apply");
    if rebase_merge.exists() || rebase_apply.exists() {
        suggestions.push(ClippySuggestion {
            id: "rebase_in_progress".to_string(),
            icon: "🎢".to_string(),
            title: "Rebase in progress".to_string(),
            description: "You started a rebase. No pressure, but you should probably finish it before doing anything else.".to_string(),
            actions: vec![
                ClippyAction {
                    label: "Continue rebase".to_string(),
                    action_type: "rebase_continue".to_string(),
                    data: None,
                },
                ClippyAction {
                    label: "Abort rebase".to_string(),
                    action_type: "rebase_abort".to_string(),
                    data: None,
                },
            ],
            priority: 9,
        });
    }

    let commit_suggestions = suggest_commits(repo_path).unwrap_or_default();

    suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));

    // Enhanced urgency levels
    let urgency_level = if status.days_since_commit > 7 && status.uncommitted_files > 200 {
        "existential_crisis"
    } else if status.days_since_commit > 7 && status.uncommitted_files > 50 {
        "panic"
    } else if status.days_since_commit > 3 || status.uncommitted_files > 20 {
        "warning"
    } else if status.uncommitted_files > 5 {
        "nudge"
    } else {
        "chill"
    };

    let message = match urgency_level {
        "existential_crisis" => format!(
            "📎 Oh no. {} days and {} files.\n    At what point do we just backup and start fresh?\n    I'm not judging. I'm just... concerned. 😰",
            status.days_since_commit, status.uncommitted_files
        ),
        "panic" => format!(
            "📎 \"I notice you haven't committed in {} days.\n    Your working directory has {} modified files.\n    Should I [Commit everything] [Create panic backup] [Cry]?\"",
            status.days_since_commit, status.uncommitted_files
        ),
        "warning" => "📎 Hey! Just checking in. Things are getting a bit messy.\n    Your future self will thank you for committing.".to_string(),
        "nudge" => "📎 Looking good! A few suggestions when you have a moment.".to_string(),
        _ => "📎 All clear! You're doing great. ✨".to_string(),
    };

    Ok(GitClippyReport {
        status,
        urgency_level: urgency_level.to_string(),
        message,
        suggestions,
        duplicates,
        commit_suggestions,
        copy_pattern_files,
    })
}

/// Execute a git action
pub fn execute_git_action(
    repo_path: &str,
    action: &str,
    data: Option<&serde_json::Value>,
) -> Result<String, String> {
    match action {
        "wip_commit" => {
            run_git_command(repo_path, &["add", "-A"])?;
            run_git_command(repo_path, &["commit", "-m", "WIP: Work in progress save"])
        }
        "panic_backup" => {
            // Create a timestamped backup branch
            let backup_name = format!("panic-backup-{}", chrono::Local::now().format("%Y%m%d-%H%M%S"));
            run_git_command(repo_path, &["add", "-A"])?;
            run_git_command(repo_path, &["stash", "push", "-m", &format!("Panic backup {}", backup_name)])?;
            Ok(format!("📎 Created panic backup stash. Use 'git stash list' to see it. Breathe. It's going to be okay. 🫂"))
        }
        "create_branch" => {
            let branch_name = data
                .and_then(|d| d.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("feature-branch");
            run_git_command(repo_path, &["checkout", "-b", branch_name])
        }
        "stage_all" => run_git_command(repo_path, &["add", "-A"]),
        "commit" => {
            let message = data
                .and_then(|d| d.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Update files");
            run_git_command(repo_path, &["commit", "-m", message])
        }
        "git_init" => run_git_command(repo_path, &["init"]),
        "abort_merge" => {
            run_git_command(repo_path, &["merge", "--abort"])?;
            Ok("📎 Merge aborted. We don't talk about what just happened. 🤫".to_string())
        }
        "rebase_continue" => run_git_command(repo_path, &["rebase", "--continue"]),
        "rebase_abort" => {
            run_git_command(repo_path, &["rebase", "--abort"])?;
            Ok("📎 Rebase aborted. Sometimes the bravest thing is to walk away. 🚶".to_string())
        }
        "cry" => {
            Ok("📎 *hands you a tissue* 🤧\n    It's okay. We've all been there.\n    When you're ready, try [Create panic backup].\n    No judgment here.".to_string())
        }
        "pray" => {
            Ok("📎 🙏 Merge gods have been notified.\n    May your conflicts resolve themselves.\n    (They won't, but we can hope)".to_string())
        }
        "learn_branches" => {
            Ok("📎 Quick branching tip:\n\n    git checkout -b feature/my-new-thing\n    <do your experiments>\n    git add -A && git commit -m 'Experiment: my new thing'\n\n    Now you have version control instead of:\n    bag_scanner_copy_final_v2_ACTUAL_working.py 📋".to_string())
        }
        _ => Err(format!("Unknown action: {}", action)),
    }
}
