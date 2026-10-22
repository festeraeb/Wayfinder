// Tauri library exports
pub mod commands;
pub mod git_assistant;
pub mod file_intelligence;
pub mod pattern_database;
pub mod file_watcher;
pub mod llm_config;
pub mod save_prompter_bridge;

#[cfg(test)]
mod windows_deployment_tests;
// pub mod handlers; // Not needed - using pure Rust
// pub mod state;    // Not needed yet
