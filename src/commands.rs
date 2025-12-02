use crate::{AppState, models::*};
use tauri::State;
use sqlx::QueryBuilder;
use std::fs;
use std::path::Path;
use base64::Engine;
use chrono::{Utc, NaiveDate};
use anyhow::Result;

#[tauri::command]
pub async fn get_focus_sessions(
    state: State<'_, AppState>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<FocusSession>, String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    
    let mut query_builder = QueryBuilder::new("SELECT * FROM focus_sessions WHERE 1=1");
    
    if let Some(start) = start_date {
        query_builder.push(" AND start_time >= ").push_bind(start);
    }
    
    if let Some(end) = end_date {
        query_builder.push(" AND start_time <= ").push_bind(end);
    }
    
    query_builder.push(" ORDER BY start_time DESC");
    
    let query = query_builder.build_query_as::<FocusSession>();
    let sessions = query.fetch_all(pool).await.map_err(|e| e.to_string())?;
    
    Ok(sessions)
}

#[tauri::command]
pub async fn start_focus_session(
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    
    let result = sqlx::query(
        "INSERT INTO focus_sessions (start_time) VALUES (?)")
        .bind(Utc::now())
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn end_focus_session(
    state: State<'_, AppState>,
    session_id: i64,
) -> Result<(), String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    
    let end_time = Utc::now();
    
    // Get start time to calculate duration
    let session: FocusSession = sqlx::query_as(
        "SELECT * FROM focus_sessions WHERE id = ?")
        .bind(session_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    
    let duration = end_time.timestamp() - session.start_time.timestamp();
    
    sqlx::query(
        "UPDATE focus_sessions SET end_time = ?, duration = ? WHERE id = ?")
        .bind(end_time)
        .bind(duration)
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn get_diary_entry(
    state: State<'_, AppState>,
    date: String,
) -> Result<Option<DiaryEntry>, String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    
    let entry = sqlx::query_as::<_, DiaryEntry>(
        "SELECT * FROM diary_entries WHERE date = ?")
        .bind(date)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(entry)
}

#[tauri::command]
pub async fn save_diary_entry(
    state: State<'_, AppState>,
    entry: NewDiaryEntry,
) -> Result<i64, String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    
    let images_json = entry.images.map(|imgs| serde_json::to_string(&imgs).unwrap());
    
    let result = sqlx::query(
        "INSERT OR REPLACE INTO diary_entries (date, title, content, mood, images, updated_at) 
         VALUES (?, ?, ?, ?, ?, ?)")
        .bind(entry.date)
        .bind(entry.title)
        .bind(entry.content)
        .bind(entry.mood)
        .bind(images_json)
        .bind(Utc::now())
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn get_diary_entries_by_month(
    state: State<'_, AppState>,
    year: i32,
    month: u32,
) -> Result<Vec<DiaryEntry>, String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    
    let start_date = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or("Invalid date")?;
    let end_date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).ok_or("Invalid date")?
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).ok_or("Invalid date")?
    };
    
    let entries = sqlx::query_as::<_, DiaryEntry>(
        "SELECT * FROM diary_entries WHERE date >= ? AND date < ? ORDER BY date DESC")
        .bind(start_date)
        .bind(end_date)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(entries)
}

#[tauri::command]
pub async fn get_all_diary_entries(
    state: State<'_, AppState>,
) -> Result<Vec<DiaryEntry>, String> {
    let db = state.db.lock().await;
    let pool = db.pool();

    let entries = sqlx::query_as::<_, DiaryEntry>(
        "SELECT * FROM diary_entries ORDER BY date DESC, created_at DESC")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

#[tauri::command]
pub async fn get_todos(
    state: State<'_, AppState>,
    completed: Option<bool>,
) -> Result<Vec<Todo>, String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    
    let mut query_builder = QueryBuilder::new("SELECT * FROM todos WHERE 1=1");
    
    if let Some(completed_filter) = completed {
        query_builder.push(" AND completed = ").push_bind(completed_filter);
    }
    
    query_builder.push(" ORDER BY priority DESC, created_at ASC");
    
    let query = query_builder.build_query_as::<Todo>();
    let todos = query.fetch_all(pool).await.map_err(|e| e.to_string())?;
    
    Ok(todos)
}

#[tauri::command]
pub async fn create_todo(
    state: State<'_, AppState>,
    todo: NewTodo,
) -> Result<i64, String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    
    let result = sqlx::query(
        "INSERT INTO todos (title, description, priority, due_date) VALUES (?, ?, ?, ?)")
        .bind(todo.title)
        .bind(todo.description)
        .bind(todo.priority)
        .bind(todo.due_date)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn update_todo(
    state: State<'_, AppState>,
    todo: UpdateTodo,
) -> Result<(), String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    
    let mut query_builder = QueryBuilder::new("UPDATE todos SET updated_at = ?");
    query_builder.push_bind(Utc::now());
    
    if let Some(title) = &todo.title {
        query_builder.push(", title = ").push_bind(title);
    }
    
    if let Some(description) = &todo.description {
        query_builder.push(", description = ").push_bind(description);
    }
    
    if let Some(completed) = todo.completed {
        query_builder.push(", completed = ").push_bind(completed);
    }
    
    if let Some(priority) = todo.priority {
        query_builder.push(", priority = ").push_bind(priority);
    }
    
    if let Some(due_date) = &todo.due_date {
        query_builder.push(", due_date = ").push_bind(due_date);
    }
    
    query_builder.push(" WHERE id = ?").push_bind(todo.id);
    
    query_builder.build().execute(pool).await.map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn delete_todo(
    state: State<'_, AppState>,
    todo_id: i64,
) -> Result<(), String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    
    sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(todo_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn get_alarms(
    state: State<'_, AppState>,
) -> Result<Vec<Alarm>, String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    
    let alarms = sqlx::query_as::<_, Alarm>(
        "SELECT * FROM alarms ORDER BY time ASC")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(alarms)
}

#[tauri::command]
pub async fn delete_diary_entry(
    state: State<'_, AppState>,
    date: String,
) -> Result<(), String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    sqlx::query("DELETE FROM diary_entries WHERE date = ?")
        .bind(date)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_diary_entries_by_date(
    state: State<'_, AppState>,
    date: String,
) -> Result<Vec<DiaryEntry>, String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    let entries = sqlx::query_as::<_, DiaryEntry>(
        "SELECT * FROM diary_entries WHERE date = ? ORDER BY created_at DESC")
        .bind(date)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(entries)
}

#[tauri::command]
pub async fn get_diary_entry_by_id(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Option<DiaryEntry>, String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    let entry = sqlx::query_as::<_, DiaryEntry>(
        "SELECT * FROM diary_entries WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(entry)
}

#[tauri::command]
pub async fn update_diary_entry(
    state: State<'_, AppState>,
    entry: UpdateDiaryEntry,
) -> Result<(), String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    let mut qb = QueryBuilder::new("UPDATE diary_entries SET updated_at = ");
    qb.push_bind(Utc::now());
    if let Some(title) = &entry.title { qb.push(", title = ").push_bind(title); }
    if let Some(content) = &entry.content { qb.push(", content = ").push_bind(content); }
    if let Some(mood) = entry.mood { qb.push(", mood = ").push_bind(mood); }
    if let Some(images) = &entry.images {
        let images_json = serde_json::to_string(images).unwrap_or_default();
        qb.push(", images = ").push_bind(images_json);
    }
    qb.push(" WHERE id = ").push_bind(entry.id);
    qb.build().execute(pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_diary_entry_by_id(
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    sqlx::query("DELETE FROM diary_entries WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn load_file_base64(path: String) -> Result<String, String> {
    let p = Path::new(&path);
    let data = fs::read(p).map_err(|e| e.to_string())?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(data);
    let mime = match p.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    };
    Ok(format!("data:{};base64,{}", mime, encoded))
}

#[tauri::command]
pub async fn create_alarm(
    state: State<'_, AppState>,
    alarm: NewAlarm,
) -> Result<i64, String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    
    let days_json = alarm.days.map(|days| serde_json::to_string(&days).unwrap());
    
    let result = sqlx::query(
        "INSERT INTO alarms (time, days, label, sound_path) VALUES (?, ?, ?, ?)")
        .bind(alarm.time)
        .bind(days_json)
        .bind(alarm.label)
        .bind(alarm.sound_path)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn update_alarm(
    state: State<'_, AppState>,
    alarm: UpdateAlarm,
) -> Result<(), String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    
    let mut query_builder = QueryBuilder::new("UPDATE alarms SET");
    let mut first = true;
    
    if let Some(time) = &alarm.time {
        if !first { query_builder.push(", "); }
        query_builder.push(" time = ").push_bind(time);
        first = false;
    }
    
    if let Some(days) = &alarm.days {
        if !first { query_builder.push(", "); }
        let days_json = serde_json::to_string(days).unwrap();
        query_builder.push(" days = ").push_bind(days_json);
        first = false;
    }
    
    if let Some(enabled) = alarm.enabled {
        if !first { query_builder.push(", "); }
        query_builder.push(" enabled = ").push_bind(enabled);
        first = false;
    }
    
    if let Some(label) = &alarm.label {
        if !first { query_builder.push(", "); }
        query_builder.push(" label = ").push_bind(label);
        first = false;
    }
    
    if let Some(sound_path) = &alarm.sound_path {
        if !first { query_builder.push(", "); }
        query_builder.push(" sound_path = ").push_bind(sound_path);
    }
    
    query_builder.push(" WHERE id = ?").push_bind(alarm.id);
    
    query_builder.build().execute(pool).await.map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn delete_alarm(
    state: State<'_, AppState>,
    alarm_id: i64,
) -> Result<(), String> {
    let db = state.db.lock().await;
    let pool = db.pool();
    
    sqlx::query("DELETE FROM alarms WHERE id = ?")
        .bind(alarm_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn get_theme(
    _state: State<'_, AppState>,
) -> Result<Theme, String> {
    // For now, return default theme
    // In a real implementation, this would be stored in the database or config file
    Ok(Theme {
        mode: "dark".to_string(),
    })
}

#[tauri::command]
pub async fn set_theme(
    _state: State<'_, AppState>,
    _theme: Theme,
) -> Result<(), String> {
    // For now, just return OK
    // In a real implementation, this would be stored in the database or config file
    Ok(())
}

fn resolve_resource_file(name: &str) -> Option<String> {
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|x| x.to_path_buf()));
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Some(dir) = exe_dir.clone() {
        candidates.push(dir.join("recourse").join(name));
        if let Some(parent) = dir.parent() { candidates.push(parent.join("recourse").join(name)); }
        if let Some(grand) = dir.parent().and_then(|p| p.parent()) { candidates.push(grand.join("recourse").join(name)); }
    }
    if let Some(dir) = exe_dir {
        let mut up = dir.clone();
        for _ in 0..6 {
            if let Some(parent) = up.parent() {
                up = parent.to_path_buf();
                let st = up.join("src-tauri").join("recourse").join(name);
                candidates.push(st);
            }
        }
    }
    for p in candidates {
        if p.exists() { return Some(p.to_string_lossy().to_string()); }
    }
    None
}

#[tauri::command]
pub fn load_resource_file_base64(name: String) -> Result<String, String> {
    let resolved = resolve_resource_file(&name).ok_or_else(|| "resource not found".to_string())?;
    load_file_base64(resolved)
}

#[tauri::command]
pub fn resolve_resource_path(name: String) -> Result<String, String> {
    resolve_resource_file(&name).ok_or_else(|| "resource not found".to_string())
}

#[tauri::command]
pub fn set_do_not_disturb(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Notifications\\Settings";
        let (key, _disp) = hkcu.create_subkey(path).map_err(|e| e.to_string())?;
        // Best-effort keys observed in recent Windows versions
        let _ = key.set_value("NOC_GLOBAL_SETTING_DO_NOT_DISTURB", &if enabled { 1u32 } else { 0u32 });
        let _ = key.set_value("NOC_GLOBAL_SETTING_TOASTS_ENABLED", &if enabled { 0u32 } else { 1u32 });
        let _ = key.set_value("FocusAssist", &if enabled { 2u32 } else { 0u32 });
    }
    Ok(())
}
