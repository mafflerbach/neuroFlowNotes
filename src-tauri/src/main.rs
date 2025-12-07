//! NeuroFlow Notes - Tauri Application Entry Point

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use state::AppState;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("neuroflow=debug".parse().unwrap()))
        .init();

    info!("Starting NeuroFlow Notes");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Vault
            commands::open_vault,
            commands::close_vault,
            commands::get_vault_info,
            // Notes
            commands::list_notes,
            commands::get_note,
            commands::get_note_content,
            commands::save_note,
            commands::rename_note,
            commands::delete_note,
            // Folders
            commands::create_folder,
            commands::rename_folder,
            commands::delete_folder,
            // Todos
            commands::get_todos_for_note,
            commands::toggle_todo,
            commands::get_incomplete_todos,
            // Tags & Backlinks
            commands::list_tags,
            commands::get_backlinks,
            // Search
            commands::search_notes,
            // Folder Tree
            commands::get_folder_tree,
            // Properties
            commands::get_properties,
            commands::set_property,
            commands::delete_property,
            // Schedule Blocks
            commands::create_schedule_block,
            commands::get_schedule_blocks,
            commands::get_schedule_blocks_for_date,
            commands::get_schedule_blocks_for_note,
            commands::update_schedule_block,
            commands::delete_schedule_block,
            // Notes by Date
            commands::get_notes_for_date,
            commands::get_notes_for_date_range,
        ])
        .setup(|app| {
            info!("Tauri app setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
