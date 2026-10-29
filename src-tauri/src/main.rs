// Tauri app entry point
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rustls::crypto::ring; // ensure a crypto provider is installed

fn main() {
    // Install rustls crypto provider explicitly to avoid runtime selection errors
    ring::default_provider().install_default().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|_app| {
            // Initialize app state if needed
            Ok(())
        })
        .invoke_handler(wayfinder_tauri::wayfinder_handler!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
