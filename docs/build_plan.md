# Wayfinder Build Plan

## Scope
- Cleaner org without breaking paths (junction-based "Wayfinder View").
- Smarter embeddings for similarity/versioning and move plans.
- Git/Nauti Clippy UX: reminders, IDE launch, context recovery, heartbeat.
- Future: shell hooks for OS-wide prompts.

## Phase 1: Safe views and detection
- Add dry-run + apply + rollback for junction/symlink view; move messy root under Wayfinder/Projects/Active/<name>, leave junction at old path.
- Generate curated "View" tree using links grouped by workflow/topic; expose a report before applying.
- Duplicate/version detection: use embeddings to flag >0.9 similarity; pick newest by mtime unless pinned; soft-archive older to .tmp_hidden/Archive.
- Keep move-plan report (no destructive ops by default).

## Phase 2: Clippy + reminders + IDE launch
- IDE launcher detection: .code-workspace/.vscode/.idea/.sln → launch appropriate tool; fallback to folder; remember last successful launcher per repo.
- Reminder store (JSON): snooze presets, due/overdue surfacing on Home (Important/Later); links open via launcher.
- Git Clippy polish: stale-clean repo suggestion, dup/stub counts, no nag when clean; integrate reminders.
- Context Recovery: summarize last N commits/file saves on returning to a repo.

## Phase 3: Embeddings and organization
- Full-pass embedding with resume and batch progress; deep scan, no max cap by default.
- Version picking and grouping: prefer newest; cluster by topic/workflow using path + text hints (e.g., Garmin RSD, firmware parsers, sailing notes).
- Suggested folderization: propose moves (Workflows/, Notes/, Firmware/, Logs/, Backups/); dry-run + apply + undo.

## Phase 4: Nauti overlay and activity cues
- Nauti Clippy always-on-top mini window (non-blocking first); filename nudge heuristics on watcher events.
- Heartbeat: last activity per project; auto-hide to Deep Storage at 30 days with toggle to unhide.
- "Shiny Object" trapdoor: snapshot current workspace state (repos, notes, pending files) for quick context switch/restore.

## Future: Shell hooks
- Explore lightweight shell extensions/handlers to intercept saves or Explorer opens for prompts; keep optional and off by default.

## Safety/UX
- Every destructive-looking op is opt-in with dry-run, move-plan, and rollback.
- Logs + undo for junction creation, move plans, archives.
- Respect GPU/endpoint settings for embedding; fallback paths logged.
