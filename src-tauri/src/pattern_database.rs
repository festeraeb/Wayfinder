// Pattern Learning Database
// Stores user preferences and learns from decisions

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::file_intelligence::{
    PatternType, FolderStructure, SuggestionFrequency, UserPreferences
};

const DB_NAME: &str = "wayfinder_patterns.db";

/// A record of a user decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub id: i64,
    pub file_path: String,
    pub file_name: String,
    pub suggestion_type: String,
    pub accepted: bool,
    pub timestamp: String,
    pub context: String,  // JSON with additional context
}

/// Naming pattern learned from user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub id: i64,
    pub pattern_type: String,
    pub example: String,
    pub occurrences: i32,
    pub confidence: f64,
    pub file_type: String,  // "Word", "PDF", etc.
}

/// Get the database path
pub fn get_db_path(data_dir: &str) -> String {
    Path::new(data_dir).join(DB_NAME).to_string_lossy().to_string()
}

/// Initialize the database (create tables if needed)
pub fn init_database(data_dir: &str) -> Result<Connection, String> {
    let db_path = get_db_path(data_dir);
    
    // Create data directory if it doesn't exist
    if let Some(parent) = Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;
    }
    
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    // Create tables
    conn.execute_batch(r#"
        -- User decisions (accept/reject suggestions)
        CREATE TABLE IF NOT EXISTS decisions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path TEXT NOT NULL,
            file_name TEXT NOT NULL,
            suggestion_type TEXT NOT NULL,
            accepted INTEGER NOT NULL,
            timestamp TEXT DEFAULT (datetime('now')),
            context TEXT DEFAULT '{}'
        );
        
        -- Learned naming patterns
        CREATE TABLE IF NOT EXISTS naming_patterns (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pattern_type TEXT NOT NULL,
            example TEXT NOT NULL,
            occurrences INTEGER DEFAULT 1,
            confidence REAL DEFAULT 0.5,
            file_type TEXT DEFAULT 'any',
            updated_at TEXT DEFAULT (datetime('now'))
        );
        
        -- User preferences
        CREATE TABLE IF NOT EXISTS preferences (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT DEFAULT (datetime('now'))
        );
        
        -- Dismissed files (never suggest again)
        CREATE TABLE IF NOT EXISTS dismissed_files (
            file_path TEXT PRIMARY KEY,
            dismissed_at TEXT DEFAULT (datetime('now')),
            reason TEXT
        );
        
        -- Custom category mappings
        CREATE TABLE IF NOT EXISTS custom_categories (
            category TEXT PRIMARY KEY,
            target_path TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now'))
        );
        
        -- Create indices for common queries
        CREATE INDEX IF NOT EXISTS idx_decisions_file ON decisions(file_path);
        CREATE INDEX IF NOT EXISTS idx_decisions_type ON decisions(suggestion_type);
        CREATE INDEX IF NOT EXISTS idx_patterns_type ON naming_patterns(pattern_type);
    "#).map_err(|e| format!("Failed to create tables: {}", e))?;
    
    println!("[PATTERN_DB] Database initialized at {}", db_path);
    Ok(conn)
}

// ============================================================================
// DECISION TRACKING
// ============================================================================

/// Record a user decision (accept or reject)
pub fn record_decision(
    conn: &Connection,
    file_path: &str,
    file_name: &str,
    suggestion_type: &str,
    accepted: bool,
    context: &str,
) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO decisions (file_path, file_name, suggestion_type, accepted, context) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![file_path, file_name, suggestion_type, accepted as i32, context],
    ).map_err(|e| format!("Failed to record decision: {}", e))?;
    
    let id = conn.last_insert_rowid();
    
    // Update pattern confidence based on decision
    update_pattern_from_decision(conn, suggestion_type, accepted)?;
    
    println!("[PATTERN_DB] Recorded decision: {} -> {} ({})", file_name, suggestion_type, if accepted { "accept" } else { "reject" });
    
    Ok(id)
}

/// Update pattern confidence based on user decision
fn update_pattern_from_decision(conn: &Connection, suggestion_type: &str, _accepted: bool) -> Result<(), String> {
    // Get current stats for this suggestion type
    let (accept_count, reject_count): (i32, i32) = conn.query_row(
        "SELECT 
            COALESCE(SUM(CASE WHEN accepted = 1 THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN accepted = 0 THEN 1 ELSE 0 END), 0)
         FROM decisions WHERE suggestion_type = ?1",
        params![suggestion_type],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).unwrap_or((0, 0));
    
    let total = accept_count + reject_count;
    if total > 0 {
        let confidence = accept_count as f64 / total as f64;
        
        // Update or insert the pattern
        conn.execute(
            "INSERT INTO naming_patterns (pattern_type, example, occurrences, confidence)
             VALUES (?1, ?2, 1, ?3)
             ON CONFLICT(pattern_type) DO UPDATE SET
                occurrences = occurrences + 1,
                confidence = ?3,
                updated_at = datetime('now')",
            params![suggestion_type, suggestion_type, confidence],
        ).map_err(|e| format!("Failed to update pattern: {}", e))?;
    }
    
    Ok(())
}

/// Get decision history for analysis
pub fn get_decisions(conn: &Connection, limit: Option<usize>) -> Result<Vec<DecisionRecord>, String> {
    let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
    
    let mut stmt = conn.prepare(&format!(
        "SELECT id, file_path, file_name, suggestion_type, accepted, timestamp, context 
         FROM decisions ORDER BY timestamp DESC{}", 
        limit_clause
    )).map_err(|e| format!("Query error: {}", e))?;
    
    let decisions = stmt.query_map([], |row| {
        Ok(DecisionRecord {
            id: row.get(0)?,
            file_path: row.get(1)?,
            file_name: row.get(2)?,
            suggestion_type: row.get(3)?,
            accepted: row.get::<_, i32>(4)? != 0,
            timestamp: row.get(5)?,
            context: row.get(6)?,
        })
    }).map_err(|e| format!("Query error: {}", e))?;
    
    decisions.map(|r| r.map_err(|e| format!("Row error: {}", e)))
        .collect()
}

// ============================================================================
// PATTERN LEARNING
// ============================================================================

/// Get learned patterns
pub fn get_learned_patterns(conn: &Connection) -> Result<Vec<LearnedPattern>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, pattern_type, example, occurrences, confidence, file_type 
         FROM naming_patterns ORDER BY confidence DESC, occurrences DESC"
    ).map_err(|e| format!("Query error: {}", e))?;
    
    let patterns = stmt.query_map([], |row| {
        Ok(LearnedPattern {
            id: row.get(0)?,
            pattern_type: row.get(1)?,
            example: row.get(2)?,
            occurrences: row.get(3)?,
            confidence: row.get(4)?,
            file_type: row.get(5)?,
        })
    }).map_err(|e| format!("Query error: {}", e))?;
    
    patterns.map(|r| r.map_err(|e| format!("Row error: {}", e)))
        .collect()
}

/// Check if we should show suggestions based on learned confidence
pub fn should_show_suggestions(conn: &Connection, suggestion_type: &str) -> Result<bool, String> {
    // If user has rejected this type more than 80% of the time, stop showing
    let opt_confidence: Option<f64> = conn.query_row(
        "SELECT confidence FROM naming_patterns WHERE pattern_type = ?1",
        params![suggestion_type],
        |row| row.get(0),
    ).ok();
    
    match opt_confidence {
        Some(confidence) => Ok(confidence > 0.2),  // Only show if acceptance rate > 20%
        None => Ok(true),  // No data yet, show suggestions
    }
}

// ============================================================================
// DISMISSED FILES
// ============================================================================

/// Dismiss a file (never suggest again)
pub fn dismiss_file(conn: &Connection, file_path: &str, reason: Option<&str>) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO dismissed_files (file_path, reason) VALUES (?1, ?2)",
        params![file_path, reason.unwrap_or("")],
    ).map_err(|e| format!("Failed to dismiss file: {}", e))?;
    
    println!("[PATTERN_DB] Dismissed file: {}", file_path);
    Ok(())
}

/// Check if a file is dismissed
pub fn is_file_dismissed(conn: &Connection, file_path: &str) -> Result<bool, String> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM dismissed_files WHERE file_path = ?1",
        params![file_path],
        |row| row.get(0),
    ).unwrap_or(0);
    
    Ok(count > 0)
}

/// Get all dismissed files
pub fn get_dismissed_files(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn.prepare("SELECT file_path FROM dismissed_files")
        .map_err(|e| format!("Query error: {}", e))?;
    
    let paths = stmt.query_map([], |row| row.get(0))
        .map_err(|e| format!("Query error: {}", e))?;
    
    paths.map(|r| r.map_err(|e| format!("Row error: {}", e)))
        .collect()
}

// ============================================================================
// USER PREFERENCES
// ============================================================================

/// Save a preference
pub fn save_preference(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO preferences (key, value) VALUES (?1, ?2)",
        params![key, value],
    ).map_err(|e| format!("Failed to save preference: {}", e))?;
    
    Ok(())
}

/// Get a preference
pub fn get_preference(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    let result: Option<String> = conn.query_row(
        "SELECT value FROM preferences WHERE key = ?1",
        params![key],
        |row| row.get(0),
    ).ok();
    
    Ok(result)
}

/// Load all preferences into UserPreferences struct
pub fn load_preferences(conn: &Connection) -> Result<UserPreferences, String> {
    let mut prefs = UserPreferences::default();
    
    // Load naming preference
    if let Some(naming) = get_preference(conn, "preferred_naming")? {
        prefs.preferred_naming = Some(match naming.as_str() {
            "DatePrefix" => PatternType::DatePrefix,
            "DateSuffix" => PatternType::DateSuffix,
            "CategoryFirst" => PatternType::CategoryFirst,
            "ProjectBased" => PatternType::ProjectBased,
            "VersionNumbered" => PatternType::VersionNumbered,
            "Semantic" => PatternType::Semantic,
            _ => PatternType::Unstructured,
        });
    }
    
    // Load folder structure preference
    if let Some(structure) = get_preference(conn, "preferred_structure")? {
        prefs.preferred_structure = match structure.as_str() {
            "ByType" => FolderStructure::ByType,
            "ByProject" => FolderStructure::ByProject,
            "ByDate" => FolderStructure::ByDate,
            "ByCategory" => FolderStructure::ByCategory,
            "GTD" => FolderStructure::GTD,
            _ => FolderStructure::default(),
        };
    }
    
    // Load suggestion frequency
    if let Some(freq) = get_preference(conn, "suggestion_frequency")? {
        prefs.suggestion_frequency = match freq.as_str() {
            "Always" => SuggestionFrequency::Always,
            "Smart" => SuggestionFrequency::Smart,
            "Daily" => SuggestionFrequency::Daily,
            "Weekly" => SuggestionFrequency::Weekly,
            "Never" => SuggestionFrequency::Never,
            _ => SuggestionFrequency::default(),
        };
    }
    
    // Load dismissed files
    prefs.dismissed_suggestions = get_dismissed_files(conn)?;
    
    Ok(prefs)
}

/// Save custom category mapping
pub fn save_custom_category(conn: &Connection, category: &str, path: &str) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO custom_categories (category, target_path) VALUES (?1, ?2)",
        params![category, path],
    ).map_err(|e| format!("Failed to save category: {}", e))?;
    
    println!("[PATTERN_DB] Saved category: {} -> {}", category, path);
    Ok(())
}

/// Get custom category path
pub fn get_category_path(conn: &Connection, category: &str) -> Result<Option<String>, String> {
    let result: Option<String> = conn.query_row(
        "SELECT target_path FROM custom_categories WHERE category = ?1",
        params![category],
        |row| row.get(0),
    ).ok();
    
    Ok(result)
}

// ============================================================================
// ANALYTICS
// ============================================================================

/// Get acceptance rate for a suggestion type
pub fn get_acceptance_rate(conn: &Connection, suggestion_type: &str) -> Result<f64, String> {
    let (accepts, total): (i32, i32) = conn.query_row(
        "SELECT 
            COALESCE(SUM(CASE WHEN accepted = 1 THEN 1 ELSE 0 END), 0),
            COUNT(*)
         FROM decisions WHERE suggestion_type = ?1",
        params![suggestion_type],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).unwrap_or((0, 0));
    
    if total == 0 {
        Ok(0.5)  // No data, assume neutral
    } else {
        Ok(accepts as f64 / total as f64)
    }
}

/// Get overall statistics
pub fn get_learning_stats(conn: &Connection) -> Result<serde_json::Value, String> {
    let total_decisions: i32 = conn.query_row(
        "SELECT COUNT(*) FROM decisions",
        [],
        |row| row.get(0),
    ).unwrap_or(0);
    
    let accepted_decisions: i32 = conn.query_row(
        "SELECT COUNT(*) FROM decisions WHERE accepted = 1",
        [],
        |row| row.get(0),
    ).unwrap_or(0);
    
    let dismissed_count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM dismissed_files",
        [],
        |row| row.get(0),
    ).unwrap_or(0);
    
    let patterns = get_learned_patterns(conn)?;
    
    Ok(serde_json::json!({
        "total_decisions": total_decisions,
        "accepted_decisions": accepted_decisions,
        "rejected_decisions": total_decisions - accepted_decisions,
        "acceptance_rate": if total_decisions > 0 { 
            accepted_decisions as f64 / total_decisions as f64 
        } else { 
            0.5 
        },
        "dismissed_files": dismissed_count,
        "learned_patterns": patterns.len()
    }))
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    
    fn test_db() -> Connection {
        let temp = temp_dir().join("wayfinder_test");
        std::fs::create_dir_all(&temp).ok();
        init_database(&temp.to_string_lossy()).unwrap()
    }
    
    #[test]
    fn test_record_decision() {
        let conn = test_db();
        
        let id = record_decision(
            &conn,
            "/path/to/file.docx",
            "file.docx",
            "move_to_documents",
            true,
            "{}",
        ).unwrap();
        
        assert!(id > 0);
    }
    
    #[test]
    fn test_dismiss_file() {
        let conn = test_db();
        
        dismiss_file(&conn, "/path/to/file.pdf", Some("user prefers here")).unwrap();
        assert!(is_file_dismissed(&conn, "/path/to/file.pdf").unwrap());
        assert!(!is_file_dismissed(&conn, "/other/file.pdf").unwrap());
    }
    
    #[test]
    fn test_preferences() {
        let conn = test_db();
        
        save_preference(&conn, "test_key", "test_value").unwrap();
        let value = get_preference(&conn, "test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));
    }
}
