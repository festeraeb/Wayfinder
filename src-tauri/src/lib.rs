// Tauri library exports

/// The invoke handler with every command the app exposes; shared by the app (main.rs) and the
/// test bridge (bin/wf_bridge.rs) so the two cannot drift apart.
#[macro_export]
macro_rules! wayfinder_handler {
    () => {
        tauri::generate_handler![
            $crate::commands::scan_directory,
            $crate::commands::generate_embeddings,
            $crate::commands::generate_embeddings_multi,
            $crate::commands::create_clusters,
            $crate::commands::create_scoped_clusters,
            $crate::commands::get_scoped_clusters,
            $crate::commands::search,
            $crate::semantic_search::semantic_search,
            $crate::commands::get_clusters_summary,
            $crate::commands::get_timeline,
            $crate::commands::get_stats,
            $crate::commands::validate_index,
            $crate::commands::get_system_info,
            $crate::commands::save_azure_config,
            $crate::commands::save_gcp_config,
            $crate::commands::save_provider_config,
            $crate::commands::load_azure_config,
            $crate::commands::load_gcp_config,
            $crate::commands::load_provider_config,
            $crate::commands::get_clusters_data,
            $crate::commands::classify_files,
            // Progress and error tracking
            $crate::commands::get_embedding_progress,
            $crate::commands::cancel_embedding,
            $crate::commands::get_error_log,
            $crate::commands::clear_error_log,
            // Git Clippy Assistant commands
            $crate::commands::get_git_clippy_report,
            $crate::commands::execute_clippy_action,
            $crate::commands::is_git_repo,
            $crate::commands::delete_duplicate_files,
            $crate::commands::chat_llama,
            $crate::commands::get_nauticlippy_report,
            // File Intelligence commands
            $crate::commands::scan_for_documents,
            $crate::commands::get_organization_suggestions,
            $crate::commands::get_scan_statistics,
            $crate::commands::dismiss_suggestion,
            // File Watcher commands
            $crate::commands::start_file_watcher,
            $crate::commands::stop_file_watcher,
            $crate::commands::get_file_events,
            $crate::commands::get_watcher_status,
            $crate::commands::get_file_suggestions,
            $crate::save_prompter_bridge::get_save_prompts,
            $crate::save_prompter_bridge::set_save_strictness,
            $crate::setup_commands::check_lemonade_setup,
            $crate::lcon::set_interaction_settings,
            $crate::lcon::get_chat_contract,
            $crate::commands::validate_azure_config,
            $crate::commands::validate_gcp_config,
            $crate::commands::validate_all_azure_configs,
            $crate::commands::cache_index_locally,
            $crate::commands::export_index,
            $crate::commands::import_index,
            $crate::commands::build_ref_index,
            $crate::commands::analyze_cluster,
            $crate::commands::plan_moves,
            $crate::commands::apply_move_plan,
            $crate::commands::list_move_history,
            $crate::commands::undo_move_plan,
        ]
    };
}

pub mod commands;
pub mod git_assistant;
pub mod file_intelligence;
pub mod pattern_database;
pub mod file_watcher;
pub mod llm_config;
pub mod lcon;
pub mod save_prompter_bridge;
pub mod lemonade_check;
pub mod setup_commands;
pub mod semantic_search;
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
