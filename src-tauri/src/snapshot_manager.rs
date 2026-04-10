// Snapshot manager for backing up and restoring Wayfinder index state
// Supports local snapshots with Azure Blob Storage integration via MCP

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    pub storage_provider: String, // "azure" or "local"
    pub account_name: Option<String>,
    pub account_key: Option<String>,
    pub container_name: Option<String>,
    pub retention_days: i32,
    pub max_snapshots: i32,
    pub auto_snapshot_interval_minutes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub id: String,
    pub timestamp: String,
    pub size_bytes: u64,
    pub files_count: usize,
    pub description: Option<String>,
    pub storage_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotIndex {
    pub snapshots: Vec<SnapshotMetadata>,
    pub last_updated: String,
}

impl Default for SnapshotIndex {
    fn default() -> Self {
        SnapshotIndex {
            snapshots: Vec::new(),
            last_updated: Utc::now().to_rfc3339(),
        }
    }
}

pub struct SnapshotManager {
    config: SnapshotConfig,
    index_path: PathBuf,
    snapshots_dir: PathBuf,
}

impl SnapshotManager {
    pub fn new(index_path: &Path, config: SnapshotConfig) -> Result<Self, String> {
        let snapshots_dir = index_path.join(".wayfinder_snapshots");
        fs::create_dir_all(&snapshots_dir)
            .map_err(|e| format!("Failed to create snapshots directory: {}", e))?;

        Ok(SnapshotManager {
            config,
            index_path: index_path.to_path_buf(),
            snapshots_dir,
        })
    }

    pub async fn create_snapshot(
        &self,
        description: Option<String>,
    ) -> Result<SnapshotMetadata, String> {
        let snapshot_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().to_rfc3339();

        // Files to backup: index.json, classifications.json, move_history.json, pattern_database.db
        let files_to_backup = vec![
            "index.json",
            "classifications.json",
            "move_history.json",
            "pattern_database.db",
        ];

        // Create snapshot directory
        let snapshot_dir = self.snapshots_dir.join(&snapshot_id);
        fs::create_dir_all(&snapshot_dir)
            .map_err(|e| format!("Failed to create snapshot directory: {}", e))?;

        let mut total_files = 0;
        let mut total_size: u64 = 0;

        // Copy files to snapshot directory
        for filename in &files_to_backup {
            let source = self.index_path.join(filename);
            if source.exists() {
                let dest = snapshot_dir.join(filename);
                fs::copy(&source, &dest)
                    .map_err(|e| format!("Failed to copy {}: {}", filename, e))?;

                if let Ok(metadata) = fs::metadata(&dest) {
                    total_size += metadata.len();
                }
                total_files += 1;
            }
        }

        // Create snapshot metadata file
        let snapshot_metadata = SnapshotMetadata {
            id: snapshot_id.clone(),
            timestamp: timestamp.clone(),
            size_bytes: total_size,
            files_count: total_files,
            description: description.clone(),
            storage_url: if self.config.storage_provider == "azure" {
                Some(format!(
                    "https://{}.blob.core.windows.net/{}/{}",
                    self.config.account_name.as_ref().unwrap_or(&"unknown".to_string()),
                    self.config.container_name.as_ref().unwrap_or(&"snapshots".to_string()),
                    snapshot_id
                ))
            } else {
                None
            },
        };

        // Write metadata
        let metadata_file = snapshot_dir.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&snapshot_metadata)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
        fs::write(&metadata_file, metadata_json)
            .map_err(|e| format!("Failed to write metadata: {}", e))?;

        // Update snapshot index
        self.update_snapshot_index(&snapshot_metadata).await?;

        println!("[SNAPSHOT] Created snapshot {} ({}  files, {} bytes)", snapshot_id, total_files, total_size);

        Ok(snapshot_metadata)
    }

    pub async fn restore_snapshot(
        &self,
        snapshot_id: &str,
    ) -> Result<usize, String> {
        let snapshot_dir = self.snapshots_dir.join(snapshot_id);

        if !snapshot_dir.exists() {
            return Err(format!("Snapshot {} not found", snapshot_id));
        }

        let mut restored_count = 0;

        // Restore files from snapshot
        for entry in fs::read_dir(&snapshot_dir)
            .map_err(|e| format!("Failed to read snapshot directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            // Skip metadata file
            if path.file_name().map_or(false, |n| n == "metadata.json") {
                continue;
            }

            if let Some(filename) = path.file_name() {
                let dest = self.index_path.join(filename);
                fs::copy(&path, &dest)
                    .map_err(|e| format!("Failed to restore {}: {}", filename.to_string_lossy(), e))?;
                restored_count += 1;
            }
        }

        println!("[SNAPSHOT] Restored snapshot {} ({} files)", snapshot_id, restored_count);

        Ok(restored_count)
    }

    pub async fn list_snapshots(&self) -> Result<Vec<SnapshotMetadata>, String> {
        let index_file = self.index_path.join("snapshot_index.json");

        if !index_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&index_file)
            .map_err(|e| format!("Failed to read snapshot index: {}", e))?;

        let index: SnapshotIndex = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse snapshot index: {}", e))?;

        Ok(index.snapshots)
    }

    pub async fn get_snapshot_status(&self) -> Result<serde_json::Value, String> {
        let snapshots = self.list_snapshots().await?;
        let last_snapshot = snapshots.first().cloned();
        let total_size: u64 = snapshots.iter().map(|s| s.size_bytes).sum();

        Ok(serde_json::json!({
            "configured": self.config.storage_provider == "azure" || self.config.storage_provider == "local",
            "has_key": self.config.account_key.is_some(),
            "last_snapshot": last_snapshot,
            "snapshots_available": snapshots.len(),
            "total_size_bytes": total_size,
        }))
    }

    async fn update_snapshot_index(
        &self,
        metadata: &SnapshotMetadata,
    ) -> Result<(), String> {
        let index_file = self.index_path.join("snapshot_index.json");

        let mut index = if index_file.exists() {
            let content = fs::read_to_string(&index_file)
                .map_err(|e| format!("Failed to read snapshot index: {}", e))?;
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse snapshot index: {}", e))?
        } else {
            SnapshotIndex::default()
        };

        // Add new snapshot (newest first)
        index.snapshots.insert(0, metadata.clone());
        index.last_updated = Utc::now().to_rfc3339();

        // Apply retention policy
        if index.snapshots.len() > self.config.max_snapshots as usize {
            index.snapshots.truncate(self.config.max_snapshots as usize);

            // Delete old snapshot directories
            for old_snapshot in &index.snapshots[self.config.max_snapshots as usize..] {
                let old_dir = self.snapshots_dir.join(&old_snapshot.id);
                let _ = fs::remove_dir_all(&old_dir);
            }
        }

        let json = serde_json::to_string_pretty(&index)
            .map_err(|e| format!("Failed to serialize snapshot index: {}", e))?;

        fs::write(&index_file, json)
            .map_err(|e| format!("Failed to write snapshot index: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_config_creation() {
        let config = SnapshotConfig {
            storage_provider: "azure".to_string(),
            account_name: Some("test".to_string()),
            account_key: Some("key".to_string()),
            container_name: Some("snapshots".to_string()),
            retention_days: 7,
            max_snapshots: 30,
            auto_snapshot_interval_minutes: 5,
        };

        assert_eq!(config.storage_provider, "azure");
        assert_eq!(config.max_snapshots, 30);
    }
}
