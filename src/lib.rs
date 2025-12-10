mod database;
mod encryption;
mod backup;
mod commands;
mod models;

use tauri::Manager;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    db: Arc<Mutex<database::Database>>,
    encryption: Arc<encryption::Encryption>,
    backup_manager: Arc<backup::BackupManager>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let context = tauri::generate_context!();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        
        .invoke_handler(tauri::generate_handler![
            commands::get_focus_sessions,
            commands::start_focus_session,
            commands::end_focus_session,
            commands::get_diary_entry,
            commands::save_diary_entry,
            commands::get_diary_entries_by_month,
            commands::get_all_diary_entries,
            commands::delete_diary_entry,
            commands::get_diary_entries_by_date,
            commands::get_diary_entry_by_id,
            commands::update_diary_entry,
            commands::delete_diary_entry_by_id,
            commands::load_file_base64,
            commands::load_resource_file_base64,
            commands::resolve_resource_path,
            commands::get_todos,
            commands::create_todo,
            commands::update_todo,
            commands::delete_todo,
            commands::get_alarms,
            commands::create_alarm,
            commands::update_alarm,
            commands::delete_alarm,
            commands::get_theme,
            commands::set_theme,
            commands::set_do_not_disturb,
        ])
        .setup(|app| {
            let rt = tokio::runtime::Runtime::new()?;
            
            let state = rt.block_on(async {
                let db = database::Database::new().await?;
                let encryption_key = encryption::Encryption::load_or_init_key()?;
                let encryption = encryption::Encryption::new(&encryption_key)?;
                let backup_manager = backup::BackupManager::new()?;
                
                Ok::<AppState, anyhow::Error>(AppState {
                    db: Arc::new(Mutex::new(db)),
                    encryption: Arc::new(encryption),
                    backup_manager: Arc::new(backup_manager),
                })
            })?;
            
            app.manage(state);
            
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }
            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}
