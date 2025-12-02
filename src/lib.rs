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
            // Background alarm checker
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                use chrono::Local;
                loop {
                    let state = handle.state::<AppState>();
                    let db = state.db.lock().await;
                    let pool = db.pool();
                    let now = Local::now();
                    let current_hhmm = now.format("%H:%M").to_string();
                    let weekday = now.format("%a").to_string();

                    let alarms: Vec<models::Alarm> = sqlx::query_as(
                        "SELECT * FROM alarms WHERE enabled = 1"
                    )
                    .fetch_all(pool)
                    .await
                    .unwrap_or_default();

                    for alarm in alarms {
                        let day_ok = match alarm.days.as_deref() {
                            Some(days_json) => {
                                let d: Vec<String> = serde_json::from_str(days_json).unwrap_or_default();
                                d.is_empty() || d.iter().any(|x| x.eq_ignore_ascii_case(&weekday))
                            }
                            None => true,
                        };
                        if day_ok && alarm.time == current_hhmm {
                            println!("Alarm: {}", alarm.label.clone().unwrap_or_else(|| "Alarm".to_string()));
                        }
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                }
            });
            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}

fn main() {
    run();
}
