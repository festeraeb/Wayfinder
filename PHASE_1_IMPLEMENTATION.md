# Wayfinder Phase 1: Snapshot & Safety (0.5 Agent Roadmap)

## Goal
Build a **bulletproof backup system** so you never lose an index to a bad decision or errant agent.

Nothing fancy. Just solid, reversible backups.

---

## Tasks for 0.5 Agent (Cheaper Model)

### Task 1: Add Snapshot Support to Pattern Database
**Why**: Centralize all state management in SQLite (easier to query/restore)

**Files to modify**: `src-tauri/src/pattern_database.rs`

```rust
// Add these constants
const SNAPSHOTS_TABLE: &str = r#"
    CREATE TABLE snapshots (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        snapshot_id TEXT UNIQUE NOT NULL,
        snapshot_path TEXT NOT NULL,
        scan_path TEXT NOT NULL,
        file_count INTEGER,
        total_size_bytes INTEGER,
        index_hash TEXT,  -- Hash of index.json for integrity check
        embeddings_hash TEXT,  -- Hash of embeddings.json
        clusters_hash TEXT,  -- Hash of clusters.json
        created_at TEXT DEFAULT (datetime('now')),
        description TEXT,  -- User can add notes
        is_manual INTEGER DEFAULT 0  -- Manual vs auto snapshot
    );
";

// Add these functions (stubs are fine, logic comes later):
pub fn create_snapshot(conn: &Connection, index_dir: &str, scan_path: &str, description: Option<&str>) -> Result<String, String>
pub fn list_snapshots(conn: &Connection, limit: Option<usize>) -> Result<Vec<SnapshotRecord>, String>
pub fn get_snapshot(conn: &Connection, snapshot_id: &str) -> Result<SnapshotRecord, String>
pub fn delete_snapshot(conn: &Connection, snapshot_id: &str) -> Result<(), String>
pub fn cleanup_old_snapshots(conn: &Connection, keep_count: usize) -> Result<usize, String>
```

**Definition of Done**:
- [ ] Table created + tested
- [ ] Functions compile
- [ ] All methods have basic error handling

---

### Task 2: Implement Snapshot Creation (Rust)
**File**: `src-tauri/src/commands.rs`

Add this new atomic command:

```rust
/// Create a snapshot of current index state (blocking operation)
#[tauri::command]
pub async fn create_snapshot(
    index_dir: String,
    description: Option<String>
) -> Result<serde_json::Value, String> {
    // Logic:
    // 1. Read current index.json, embeddings.json, clusters.json
    // 2. Hash each file for integrity checking
    // 3. Create snapshot directory: .wayfinder_index/snapshots/snapshot_TIMESTAMP/
    // 4. Copy files into snapshot directory
    // 5. Write manifest.json with metadata + hashes
    // 6. Record in snapshots table with snapshot_id
    // 7. Return snapshot_id + metadata
    //
    // Error handling:
    //  - If index doesn't exist → error
    //  - If snapshot disk space low → warning but proceed (user can delete old snapshots)
    //  - If copy fails → cleanup partial snapshot, return error
}
```

**Definition of Done**:
- [ ] Can create a snapshot
- [ ] Snapshot files are readable
- [ ] Metadata includes file count, timestamps, hashes
- [ ] Returns snapshot_id for restoration

---

### Task 3: Implement Snapshot Restoration (Rust)
**File**: `src-tauri/src/commands.rs`

```rust
/// Restore index from a previous snapshot
#[tauri::command]
pub async fn restore_snapshot(
    index_dir: String,
    snapshot_id: String,
    create_backup: Option<bool>  // Always true: before restoring, create backup of current state
) -> Result<serde_json::Value, String> {
    // Logic:
    // 1. Find snapshot by snapshot_id
    // 2. Verify snapshot integrity (hash check)
    // 3. If create_backup=true:
    //    a. Create backup of current state before overwriting
    //    b. This becomes "pre-restore" snapshot
    // 4. Copy snapshot files back to index.json, embeddings.json, clusters.json
    // 5. Validate restored files (parse JSON)
    // 6. Log restoration event
    // 7. Return success + what changed (file count, sizes, timestamps)
    //
    // Safety:
    //  - Never delete original if restore fails
    //  - Always have escape route
}
```

**Definition of Done**:
- [ ] Can restore from any snapshot
- [ ] Always backs up current state first
- [ ] Validates restored files before returning
- [ ] Undo for the undo (rollback safety)

---

### Task 4: List Snapshots for UI
**File**: `src-tauri/src/commands.rs`

```rust
/// Get list of snapshots with metadata
#[tauri::command]
pub async fn list_snapshots(
    index_dir: String,
    limit: Option<usize>
) -> Result<serde_json::Value, String> {
    // Returns:
    // {
    //   "snapshots": [
    //     {
    //       "id": "snapshot_20260301_143022",
    //       "timestamp": "2026-03-01 14:30:22",
    //       "file_count": 427,
    //       "size_mb": 5.2,
    //       "description": "Before running agent X",
    //       "is_manual": true,
    //       "restorable": true
    //     },
    //     ...
    //   ],
    //   "total_snapshots": 15,
    //   "disk_used_mb": 78.4
    // }
}
```

**Definition of Done**:
- [ ] Returns list in reverse chronological order (newest first)
- [ ] Includes size estimates
- [ ] Marks snapshots as "manual" vs "auto"

---

### Task 5: Auto-Delete Old Snapshots
**File**: `src-tauri/src/commands.rs`

```rust
/// Clean up old snapshots to save disk space
/// Keeps: last N snapshots, or within last M days, whichever is more
#[tauri::command]
pub async fn cleanup_snapshots(
    index_dir: String,
    keep_count: Option<usize>,      // default: 30
    keep_days: Option<i64>           // default: 7
) -> Result<serde_json::Value, String> {
    // Logic:
    // 1. List all snapshots
    // 2. Delete ones older than keep_days
    // 3. But always keep at least keep_count most recent
    // 4. Return how many deleted + space freed
    //
    // This runs in background every hour (not user-facing)
}
```

**Definition of Done**:
- [ ] Doesn't delete manually-marked snapshots (unless forced)
- [ ] Reports space freed
- [ ] Safe to call repeatedly (idempotent)

---

### Task 6: File Change History Logging
**File**: `src-tauri/src/pattern_database.rs`

Add this table:

```rust
// In init_database():
r#"
CREATE TABLE file_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL,
    event_type TEXT NOT NULL,  -- 'created', 'modified', 'deleted', 'renamed'
    old_hash TEXT,             -- For detection of actual changes
    new_hash TEXT,
    file_size INTEGER,
    timestamp TEXT DEFAULT (datetime('now')),
    
    UNIQUE(file_path, timestamp)  -- Avoid duplicates
);
"#

// Helper function:
pub fn log_file_change(
    conn: &Connection,
    file_path: &str,
    event_type: &str,
    old_hash: Option<&str>,
    new_hash: Option<&str>,
    file_size: Option<u64>
) -> Result<(), String>
```

**Definition of Done**:
- [ ] Can log file changes
- [ ] Hashes are computed efficiently (don't hang on large files)
- [ ] Query shows "what changed in this file over time"

---

### Task 7: UI: Snapshots Panel (React)
**File**: `src/components/SnapshotsPanel.tsx`

Basic UI (can be ugly, just functional):

```typescript
export const SnapshotsPanel = () => {
  const [snapshots, setSnapshots] = useState([]);
  const [loading, setLoading] = useState(false);

  const handleLoadSnapshots = async () => {
    // Call list_snapshots Tauri command
    // Parse response, show in table/list
  };

  const handleRestore = async (snapshotId: string) => {
    // Confirm: "Restore to [timestamp]? Current state will be backed up."
    // Call restore_snapshot Tauri command
    // Show result
  };

  const handleCreateSnapshot = async () => {
    // Prompt for optional description
    // Call create_snapshot Tauri command
    // Refresh list
  };

  return (
    <div className="snapshots-panel">
      <h2>Index Snapshots</h2>
      <button onClick={handleCreateSnapshot}>💾 Create Snapshot</button>
      <button onClick={handleLoadSnapshots}>🔄 Refresh</button>
      
      {loading && <p>Loading...</p>}
      
      <table>
        <thead>
          <tr>
            <th>When</th>
            <th>Files</th>
            <th>Size</th>
            <th>Notes</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {snapshots.map(s => (
            <tr key={s.id}>
              <td>{s.timestamp}</td>
              <td>{s.file_count}</td>
              <td>{s.size_mb.toFixed(1)} MB</td>
              <td>{s.description || '-'}</td>
              <td>
                <button onClick={() => handleRestore(s.id)}>Restore</button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};
```

**Definition of Done**:
- [ ] Panel shows up in app
- [ ] Can create, list, restore snapshots from UI
- [ ] No crashes on empty list

---

### Task 8: Wire Up to App Lifecycle
**File**: `src-tauri/src/main.rs`

Add background snapshot task at startup:

```rust
// In app setup:
let index_dir_clone = index_dir.clone();
std::thread::spawn(move || {
    loop {
        // Every 5 minutes, create auto-snapshot (if index exists)
        std::thread::sleep(std::time::Duration::from_secs(300));
        
        if Path::new(&index_dir_clone).join("index.json").exists() {
            let _ = create_auto_snapshot(&index_dir_clone);
        }
        
        // Every hour, cleanup old snapshots
        // (keep 30, or within 7 days)
    }
});
```

**Definition of Done**:
- [ ] App doesn't crash on startup
- [ ] Snapshots are created automatically
- [ ] Old snapshots are cleaned up

---

## Phase 1 Success Criteria

✅ Can create snapshots manually  
✅ Can restore from any snapshot  
✅ Snapshots auto-created every 5 minutes (background)  
✅ Old snapshots auto-deleted (keep 30)  
✅ UI shows snapshots with timeline  
✅ Restoration always creates backup first (rollback-safe)  

---

## Notes for 0.5 Agent

- These are **straightforward, isolated tasks**—no complex reasoning needed
- **Test each function independently** before wiring to UI
- File hashing: Use Blake3 (fast, cryptographically sound) or MD5 (simpler)
- **Error messages must be user-readable** ("Snapshot corrupted" not "serde_json error")
- If you get stuck on a task, **focus on making it work**, not making it perfect
- Tauri command handlers should always return `Result<Value, String>` (consistent)

---

## What This Enables

Once Phase 1 is solid:

1. **Agent Safety**: Before any model runs bulk operations → create snapshot
2. **Rollback Confidence**: "If I screw up, I undo to X" (you'll use this constantly)
3. **Peace of Mind**: You can experiment without fear
4. **Audit Trail**: "What was I thinking?" → check decision history + snapshots
5. **Handoff Ready**: When switching agents → snapshot = state transfer point

**This is your insurance policy. Make it solid.**

