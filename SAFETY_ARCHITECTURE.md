# Wayfinder Safety & Backup Architecture

## Philosophy
**Never lose work. Always have a rollback point.**

Every major decision gets a snapshot. Every file modification gets tracked. Every agent action is reversible.

---

## 1. Backup Strategy (Anti-Loss)

### Tier 1: Live Index Snapshots
- **Frequency**: Every 5 minutes (background, non-blocking)
- **Location**: `.wayfinder_index/snapshots/snapshot_YYYYMMDD_HHMMSS.json`
- **Content**: Complete state (index.json + embeddings.json + clusters.json)
- **Size**: ~5MB per snapshot (manageable)
- **Retention**: Keep last 30 snapshots (~2.5 hours of history)

```sql
-- Snapshots table for restoration
CREATE TABLE snapshots (
    id INTEGER PRIMARY KEY,
    timestamp TEXT,
    snapshot_path TEXT,
    scan_path TEXT,
    file_count INTEGER,
    size_bytes INTEGER,
    created_at TEXT DEFAULT datetime('now')
);
```

### Tier 2: File Change History
- **Per-file tracking**: Path → hash → timestamp → action (create/modify/delete)
- **Enables**: "What changed in this file?" + "Recover deleted files"
- **Storage**: `.wayfinder_index/file_history.json` (rolling append log)

### Tier 3: Decision Audit Log
- **What**: Every suggestion accepted/rejected gets logged with context
- **Why**: Understand your own patterns, debug unexpected results
- **Schema**: `{timestamp, file_path, action_type, confidence, accepted, model_version}`

---

## 2. File Watcher Implementation (OS-Specific)

### Windows: ReadDirectoryChangesW (Native)
```rust
// Replaces: notify crate (unreliable on Windows for rapid changes)
// Uses: Windows API directly via `winapi` crate
// Advantages:
//   - Real kernel events (not polling)
//   - Handles rapid saves, rename chains
//   - Respects OneDrive/network drives
// 
// Key: Use USN Journal for efficient change tracking
```

**Fallback**: `notify` crate for quick MVP

### macOS: FSEvents (Native)
```rust
// Uses: FSEvents API (FSEventStreamCreate)
// Advantages:
//   - Coarse-grained but reliable
//   - Efficient (not per-file overhead)
//   - No polling needed
//
// Implementation: fsevents2 Rust crate
```

### Chromebook: Service Worker + IndexedDB (Web)
```typescript
// No native file system access
// Alternative strategy:
// 1. User selects folder via File System Access API
// 2. ServiceWorker monitors IndexedDB + localStorage
// 3. Background sync to server (if available)
// 4. Local-first: all analysis happens in browser
```

---

## 3. Live File Watcher UI Integration

### Lifecycle Events
```
App Start
  → Check for existing watcher
  → Restore from last snapshot (if corrupted)
  → Start new watcher in background thread
  
File Event (Modify/Create)
  → Debounce 1s (avoid noise from rapid saves)
  → Run lightweight analysis (hash, size, name)
  → Send to UI (non-blocking)
  
UI Decision (Accept/Reject Suggestion)
  → Record in decision_history.db
  → Update pattern_database confidence
  → Create lightweight checkpoint
  
App Close
  → Finalize watcher
  → Save final snapshot
```

### UI Toggles (Tauri App)
```
Settings → File Watcher
├── [ ] Enable Live Watcher
├── [ ] Auto-accept high-confidence suggestions (>90%)
├── [ ] Daily snapshot (auto-save at midnight)
├── Index Directory: [_______________]
├── Watch Paths: [____ ] [add] [remove]
└── Sarcasm Level: [Sarcastic 100%] ← 🎯 for you
                   [Kid-Friendly 0%] ← for Chromebook sharing
```

---

## 4. Safety Net: Rollback Points

### Snapshot Types
1. **Auto-Snapshots** (every 5 min, keeps 30)
2. **Checkpoint** (user-triggered: "Save checkpoint")
3. **Pre-Action** (before any bulk operation)
4. **Daily** (midnight, keeps 7)

### Restore Process
1. Pick snapshot from timeline
2. **Preview**: "Restoring to 2:30pm - shows N files changed, X deleted"
3. **Confirm**: "Undo to this point? (reversible)"
4. **Execute**: Restore index + embeddings + clusters
5. **Verify**: Rebuild hash index, validate file paths

---

## 5. Agent Safety Guardrails

### Confidence Scoring
- Suggestions < 70% → **require user approval**
- Suggestions 70-90% → **batch review before applying**
- Suggestions > 90% → **auto-apply if enabled** (create pre-action snapshot)

### Reversibility
- **Every AI action must be logged** with enough context to undo
- **Format**: `{action_type, file_path, old_value, new_value, model, confidence, timestamp}`

### Agent Handover
When switching agents (local 0.5 → premium Opus):
1. **Export context**: Current state + decision history + learned patterns
2. **Summary**: "Here's what I did, here's the state"
3. **Rollback available**: "If you don't like my work, undo to X"

---

## 6. Implementation Roadmap

### Phase 1 (Week 1) - Backup Infrastructure
- [ ] Add snapshots table to pattern_database.db
- [ ] Implement `create_snapshot()` command (5min background task)
- [ ] Implement `list_snapshots()` + `restore_snapshot()`
- [ ] Add file change history logging
- [ ] UI "Snapshots" tab with timeline + rollback button

### Phase 2 (Week 2) - File Watcher
- [ ] **Windows**: Integrate `notify` crate properly (debounce tuning)
- [ ] **macOS**: Add `fsevents2` crate, test with real saves
- [ ] Wire up start/stop in app lifecycle
- [ ] Settings UI with path selection

### Phase 3 (Week 3) - Agent Safety
- [ ] Decision audit log command
- [ ] "Last suggestion you rejected" quick undo
- [ ] Sarcasm toggle (boolean flag + prompt injection)
- [ ] Pre-action snapshots before bulk operations

### Phase 4+ - Chromebook
- [ ] Design web-based file watcher (PWA)
- [ ] File System Access API integration
- [ ] Kid-friendly UI variant

---

## 7. Configuration (Example in `.env`)

```ini
# Snapshot settings
SNAPSHOT_INTERVAL_MINUTES=5
SNAPSHOT_RETENTION_COUNT=30
SNAPSHOT_RETENTION_DAYS=7

# File watcher
WATCHER_DEBOUNCE_MS=1000
WATCHER_IGNORE_PATTERNS=.git,node_modules,__pycache__,target

# Agent safety
AUTO_ACCEPT_THRESHOLD=0.90
LOG_ALL_DECISIONS=true
SARCASM_LEVEL=100  # 0-100, where 0 = kid-friendly
```

---

## 8. Success Metrics

✅ **Never lose an index** - Rollback within 5 min always available
✅ **Understand agent decisions** - Every suggestion logged + reversible
✅ **Peace of mind** - Hit "Undo" and it works
✅ **Works offline** - Watcher + backup doesn't need internet
✅ **Multiple platforms** - Windows, Mac, eventually Chromebook

---

## Questions for You

1. **Snapshot location preference**: Local disk (fast) or git commits? (auditable)
2. **Sarcasm toggle**: Simple boolean, or context-aware? (e.g., detect user tone)
3. **Priority**: Get Phase 1 (backups) working first, or start with Phase 2 (watcher)?
4. **Chromebook timeline**: Nice-to-have for beta, or critical?

