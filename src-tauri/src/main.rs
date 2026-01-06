//! NeuroFlow Notes - Tauri Application Entry Point

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;
mod stream;

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
        .plugin(tauri_plugin_shell::init())
        .register_uri_scheme_protocol("stream", stream::handle_stream_protocol)
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
            commands::query_tasks,
            commands::get_task_contexts,
            // Tags & Backlinks
            commands::list_tags,
            commands::get_backlinks,
            // Search
            commands::search_notes,
            commands::hybrid_search_notes,
            commands::get_embedding_status,
            commands::test_embedding_connection,
            commands::generate_note_embedding,
            commands::get_notes_needing_embeddings,
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
            // Embeds
            commands::resolve_embed,
            commands::get_note_headings,
            // Assets
            commands::save_pasted_image,
            // Query Builder
            commands::get_property_keys,
            commands::get_property_values,
            commands::get_list_property_values,
            commands::run_query,
            // Query Embeds
            commands::execute_query_embed,
            // Property Management
            commands::rename_property_key,
            commands::rename_property_value,
            commands::merge_property_keys,
            commands::delete_property_key,
            commands::get_property_values_with_counts,
            commands::get_notes_with_property,
            commands::get_notes_with_property_value,
            // Folder Properties
            commands::get_folder_properties,
            commands::set_folder_property,
            commands::delete_folder_property,
            commands::get_properties_with_inheritance,
            commands::get_folders_with_properties,
            // Frontmatter Conversion
            commands::convert_frontmatter_to_db,
            // Import
            commands::import_obsidian_vault,
            // Plugins
            commands::read_plugin_config,
            commands::write_plugin_config,
            commands::list_plugin_configs,
            commands::plugin_http_request,
            // Habits
            commands::create_habit,
            commands::list_habits,
            commands::get_habit,
            commands::update_habit,
            commands::delete_habit,
            commands::archive_habit,
            commands::log_habit_entry,
            commands::get_habit_entries,
            commands::update_habit_entry,
            commands::delete_habit_entry,
            commands::toggle_habit,
            commands::execute_habit_tracker_embed,
            // Templates
            commands::get_template_settings,
            commands::save_template_settings,
            commands::list_templates,
            commands::create_daily_note,
            commands::create_note_from_template,
            commands::preview_daily_note_path,
            // Summarizers
            commands::run_link_summarizer,
            commands::run_transcript_summarizer,
            commands::count_pending_transcripts,
        ])
        .setup(|_app| {
            info!("Tauri app setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
