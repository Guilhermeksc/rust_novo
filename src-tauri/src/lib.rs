use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Módulos
pub mod types;
pub mod pdf_processor;
pub mod sicaf_processor;
pub mod commands;
pub mod config;

// Re-export types for easy access
pub use types::*;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Arc::new(Mutex::new(HashMap::<String, types::ProcessingStatus>::new())))
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::process_pdf_file,
            commands::process_pdf_directory,
            commands::process_pdf_fixed_directory,
            commands::get_pdf_directory,
            commands::get_output_directory,
            commands::open_folder,
            commands::verify_output_directory,
            commands::get_processing_status,
            commands::list_pdf_files,
            commands::validate_pdf_file,
            commands::clear_processing_state,
            commands::get_current_directory,
            commands::create_default_directories,
            commands::initialize_database_structure,
            commands::list_json_files,
            commands::read_json_file,
            commands::get_json_file_info,
            commands::get_pdf_file_info,
            commands::get_pdf_files_info,
            commands::open_pdf_file,
            commands::load_app_config,
            commands::save_app_config,
            commands::update_config_directories,
            commands::add_config_log,
            commands::clear_config_logs,
            commands::update_config_verbose,
            commands::get_config_directory,
            commands::get_sicaf_directory,
            commands::process_sicaf_pdfs,
            commands::load_sicaf_data,
            commands::verify_cnpj_sicaf,
            commands::get_cnpj_sicaf_data,
            commands::generate_sicaf_comparison_report,
            commands::debug_and_repair_config,
            commands::initialize_application,
            commands::get_app_directories_info,
            commands::get_default_pdf_directory,
            commands::get_default_output_directory,
            commands::ensure_directory_exists,
            commands::get_user_home_directory,
            commands::update_pdf_directory,
            commands::update_output_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
