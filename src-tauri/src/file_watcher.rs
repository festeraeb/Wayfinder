// File Watcher Service
// Watches directories for new/modified files and triggers suggestions

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::file_intelligence::{DocumentType, DiscoveredDocument};

// ============================================================================
// TYPES
// ============================================================================

/// Event when a file is created or modified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvent {
    pub path: String,
    pub file_name: String,
    pub event_type: FileEventType,
    pub doc_type: DocumentType,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileEventType {
    Created,
    Modified,
    Renamed { from: String },
}

/// Configuration for watching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    pub paths: Vec<String>,
    pub debounce_ms: u64,           // Wait this long before firing event
    pub ignore_patterns: Vec<String>,
    pub watch_only_organizable: bool,
}

impl Default for WatchConfig {
    fn default() -> Self {
        WatchConfig {
            paths: get_default_watch_paths(),
            debounce_ms: 2000,  // 2 second debounce
            ignore_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "__pycache__".to_string(),
                ".vscode".to_string(),
                "target".to_string(),
                ".tmp".to_string(),
                "~$".to_string(),  // Office temp files
            ],
            watch_only_organizable: true,
        }
    }
}

/// The file watcher state
pub struct FileWatcherState {
    pub is_running: bool,
    pub watched_paths: Vec<String>,
    pub pending_events: Vec<FileEvent>,
    pub event_count: usize,
}

impl Default for FileWatcherState {
    fn default() -> Self {
        FileWatcherState {
            is_running: false,
            watched_paths: Vec::new(),
            pending_events: Vec::new(),
            event_count: 0,
        }
    }
}

// ============================================================================
// FILE WATCHER
// ============================================================================

pub struct FileWatcher {
    config: WatchConfig,
    state: Arc<Mutex<FileWatcherState>>,
    debounce_map: Arc<Mutex<HashMap<PathBuf, Instant>>>,
    event_sender: Option<Sender<FileEvent>>,
}

impl FileWatcher {
    pub fn new(config: WatchConfig) -> Self {
        FileWatcher {
            config,
            state: Arc::new(Mutex::new(FileWatcherState::default())),
            debounce_map: Arc::new(Mutex::new(HashMap::new())),
            event_sender: None,
        }
    }
    
    /// Start watching the configured paths
    pub fn start(&mut self) -> Result<Receiver<FileEvent>, String> {
        // Create channel for events
        let (tx, rx) = channel::<FileEvent>();
        self.event_sender = Some(tx.clone());
        
        // Update state
        {
            let mut state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
            state.is_running = true;
            state.watched_paths = self.config.paths.clone();
        }
        
        // Clone what we need for the thread
        let config = self.config.clone();
        let state = Arc::clone(&self.state);
        let debounce_map = Arc::clone(&self.debounce_map);
        
        // Spawn watcher thread
        thread::spawn(move || {
            if let Err(e) = run_watcher(config, tx, state, debounce_map) {
                eprintln!("[FILE_WATCHER] Error: {}", e);
            }
        });
        
        println!("[FILE_WATCHER] Started watching {} paths", self.config.paths.len());
        Ok(rx)
    }
    
    /// Stop watching
    pub fn stop(&mut self) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
        state.is_running = false;
        println!("[FILE_WATCHER] Stopped");
        Ok(())
    }
    
    /// Get current state
    pub fn get_state(&self) -> Result<FileWatcherState, String> {
        let state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
        Ok(FileWatcherState {
            is_running: state.is_running,
            watched_paths: state.watched_paths.clone(),
            pending_events: state.pending_events.clone(),
            event_count: state.event_count,
        })
    }
    
    /// Clear pending events
    pub fn clear_events(&self) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| format!("Lock error: {}", e))?;
        state.pending_events.clear();
        Ok(())
    }
}

/// The actual watcher loop running in a thread
fn run_watcher(
    config: WatchConfig,
    event_tx: Sender<FileEvent>,
    state: Arc<Mutex<FileWatcherState>>,
    debounce_map: Arc<Mutex<HashMap<PathBuf, Instant>>>,
) -> Result<(), String> {
    let (tx, rx) = channel::<Result<Event, notify::Error>>();
    
    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(|e| format!("Failed to create watcher: {}", e))?;
    
    // Add paths to watch
    for path_str in &config.paths {
        let path = Path::new(path_str);
        if path.exists() {
            watcher.watch(path, RecursiveMode::Recursive)
                .map_err(|e| format!("Failed to watch {}: {}", path_str, e))?;
            println!("[FILE_WATCHER] Watching: {}", path_str);
        } else {
            println!("[FILE_WATCHER] Path does not exist, skipping: {}", path_str);
        }
    }
    
    let debounce_duration = Duration::from_millis(config.debounce_ms);
    
    // Event loop
    loop {
        // Check if we should stop
        {
            if let Ok(s) = state.lock() {
                if !s.is_running {
                    break;
                }
            }
        }
        
        // Non-blocking receive with timeout
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => {
                if let Some(file_event) = process_event(&event, &config, &debounce_map, debounce_duration) {
                    // Send event
                    if event_tx.send(file_event.clone()).is_err() {
                        // Channel closed, stop watching
                        break;
                    }
                    
                    // Store in state for polling
                    if let Ok(mut s) = state.lock() {
                        s.pending_events.push(file_event);
                        s.event_count += 1;
                        
                        // Keep only last 50 events
                        if s.pending_events.len() > 50 {
                            s.pending_events.remove(0);
                        }
                    }
                }
            }
            Ok(Err(e)) => {
                eprintln!("[FILE_WATCHER] Watch error: {}", e);
            }
            Err(_) => {
                // Timeout, continue loop
            }
        }
    }
    
    println!("[FILE_WATCHER] Watcher thread exiting");
    Ok(())
}

/// Process a raw notify event into our FileEvent
fn process_event(
    event: &Event,
    config: &WatchConfig,
    debounce_map: &Arc<Mutex<HashMap<PathBuf, Instant>>>,
    debounce_duration: Duration,
) -> Option<FileEvent> {
    let path = event.paths.first()?;
    
    // Skip directories
    if path.is_dir() {
        return None;
    }
    
    // Check ignore patterns
    let path_str = path.to_string_lossy().to_lowercase();
    for pattern in &config.ignore_patterns {
        if path_str.contains(&pattern.to_lowercase()) {
            return None;
        }
    }
    
    // Get file extension and type
    let ext = path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let doc_type = DocumentType::from_extension(&ext);
    
    // Skip non-organizable if configured
    if config.watch_only_organizable && !doc_type.is_organizable() {
        return None;
    }
    
    // Debouncing: skip if we just saw this file
    {
        let mut map = debounce_map.lock().ok()?;
        let now = Instant::now();
        
        if let Some(last_time) = map.get(path) {
            if now.duration_since(*last_time) < debounce_duration {
                return None; // Too soon, skip
            }
        }
        
        map.insert(path.clone(), now);
    }
    
    // Determine event type (include renames/deletes so the UI sees removals)
    let event_type = match &event.kind {
        EventKind::Create(_) => FileEventType::Created,
        EventKind::Modify(notify::event::ModifyKind::Name(_)) => {
            FileEventType::Renamed { from: path_str.clone() }
        }
        EventKind::Modify(_) => FileEventType::Modified,
        EventKind::Remove(_) => FileEventType::Modified,
        EventKind::Any => FileEventType::Modified,
        _ => return None, // Ignore accesses, metadata-only, etc.
    };
    
    let file_name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    Some(FileEvent {
        path: path.to_string_lossy().to_string(),
        file_name,
        event_type,
        doc_type,
        timestamp,
    })
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Get default paths to watch (Downloads, Desktop, Documents)
fn get_default_watch_paths() -> Vec<String> {
    let mut paths = Vec::new();
    
    #[cfg(windows)]
    {
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            paths.push(format!("{}\\Downloads", userprofile));
            paths.push(format!("{}\\Desktop", userprofile));
            paths.push(format!("{}\\Documents", userprofile));
        }
    }
    
    #[cfg(not(windows))]
    {
        if let Ok(home) = std::env::var("HOME") {
            paths.push(format!("{}/Downloads", home));
            paths.push(format!("{}/Desktop", home));
            paths.push(format!("{}/Documents", home));
        }
    }
    
    paths
}

/// Create a quick scanner DiscoveredDocument from a FileEvent
pub fn event_to_document(event: &FileEvent) -> DiscoveredDocument {
    let path = Path::new(&event.path);
    let parent = path.parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    
    let ext = path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    
    let metadata = std::fs::metadata(&event.path).ok();
    let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
    
    DiscoveredDocument {
        path: event.path.clone(),
        name: event.file_name.clone(),
        extension: ext,
        doc_type: event.doc_type.clone(),
        size_bytes: size,
        modified: event.timestamp.clone(),
        parent_dir: parent,
        depth: 0,
        siblings_count: 0,
        similar_siblings: 0,
    }
}

// ============================================================================
// SAVE PROMPTER INTEGRATION
// ============================================================================

/// Configuration for the Save Prompter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePrompterConfig {
    pub enabled: bool,
    pub prompt_on_new_files: bool,
    pub prompt_on_modified: bool,
    pub cooldown_hours: f64,        // Only prompt once per X hours for same file
    pub confidence_threshold: f64,  // Only prompt if suggestion confidence > this
}

impl Default for SavePrompterConfig {
    fn default() -> Self {
        SavePrompterConfig {
            enabled: true,
            prompt_on_new_files: true,
            prompt_on_modified: false,  // Less annoying
            cooldown_hours: 24.0,
            confidence_threshold: 0.7,
        }
    }
}

/// Check if we should prompt for this file event
pub fn should_prompt_for_event(
    event: &FileEvent,
    config: &SavePrompterConfig,
    last_prompts: &HashMap<String, Instant>,
) -> bool {
    if !config.enabled {
        return false;
    }
    
    // Check event type
    match &event.event_type {
        FileEventType::Created if !config.prompt_on_new_files => return false,
        FileEventType::Modified if !config.prompt_on_modified => return false,
        FileEventType::Renamed { .. } => return true, // Always offer to improve renames
        _ => {}
    }
    
    // Check cooldown
    let cooldown = Duration::from_secs_f64(config.cooldown_hours * 3600.0);
    if let Some(last_time) = last_prompts.get(&event.path) {
        if last_time.elapsed() < cooldown {
            return false;
        }
    }
    
    true
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_watch_paths() {
        let paths = get_default_watch_paths();
        // Should have at least some paths on any system
        // May be empty in some containerized environments
        println!("Default watch paths: {:?}", paths);
    }
    
    #[test]
    fn test_watch_config_default() {
        let config = WatchConfig::default();
        assert_eq!(config.debounce_ms, 2000);
        assert!(config.watch_only_organizable);
    }
}
