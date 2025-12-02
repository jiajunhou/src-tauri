use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FocusSession {
    pub id: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFocusSession {
    pub start_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DiaryEntry {
    pub id: i64,
    pub date: NaiveDate,
    pub title: Option<String>,
    pub content: String,
    pub mood: Option<i32>,
    pub images: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDiaryEntry {
    pub date: NaiveDate,
    pub title: Option<String>,
    pub content: String,
    pub mood: Option<i32>,
    pub images: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDiaryEntry {
    pub id: i64,
    pub title: Option<String>,
    pub content: Option<String>,
    pub mood: Option<i32>,
    pub images: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub priority: i32,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTodo {
    pub title: String,
    pub description: Option<String>,
    pub priority: i32,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTodo {
    pub id: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
    pub priority: Option<i32>,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Alarm {
    pub id: i64,
    pub time: String,
    pub days: Option<String>,
    pub enabled: bool,
    pub label: Option<String>,
    pub sound_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAlarm {
    pub time: String,
    pub days: Option<Vec<String>>,
    pub label: Option<String>,
    pub sound_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAlarm {
    pub id: i64,
    pub time: Option<String>,
    pub days: Option<Vec<String>>,
    pub enabled: Option<bool>,
    pub label: Option<String>,
    pub sound_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub mode: String, // "light" or "dark"
}