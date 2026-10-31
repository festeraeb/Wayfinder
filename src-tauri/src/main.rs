// Tauri app entry point
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rustls::crypto::ring; // ensure a crypto provider is installed

fn main() {
    // Install rustls crypto provider explicitly to avoid runtime selection errors
    ring::default_provider().install_default().ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Update check on start and on a timer, and the authenticated push endpoint.
            wayfinder_tauri::updater::start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(wayfinder_tauri::wayfinder_handler!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
