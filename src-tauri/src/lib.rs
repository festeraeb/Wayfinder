// Tauri library exports
pub mod commands;
pub mod git_assistant;
pub mod file_intelligence;
pub mod pattern_database;
pub mod file_watcher;
pub mod llm_config;
pub mod lcon;
pub mod save_prompter_bridge;
pub mod lemonade_check;
pub mod embedder_id;

#[cfg(test)]
mod windows_deployment_tests;
#[cfg(test)]
mod mock_http;
#[cfg(test)]
mod embed_llama_tests;
#[cfg(test)]
mod lemonade_live_tests;
// pub mod handlers; // Not needed - using pure Rust
// pub mod state;    // Not needed yet
