//! License Key Generator - Tauri Backend
//!
//! Simplified backend for the Supabase-based license generator.
//! License generation is now done via Supabase API calls from the frontend.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
