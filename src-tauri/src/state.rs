// Application state management
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppState {
    pub current_index_dir: String,
    pub is_scanning: bool,
    pub is_embedding: bool,
    pub is_clustering: bool,
    pub last_error: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_index_dir: String::new(),
            is_scanning: false,
            is_embedding: false,
            is_clustering: false,
            last_error: None,
        }
    }
}
