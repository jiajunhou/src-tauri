use sqlx::{SqlitePool, migrate::MigrateDatabase};
use std::path::PathBuf;
use directories::ProjectDirs;
use anyhow::Result;

pub struct Database {
    pool: SqlitePool,
    db_path: PathBuf,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        let db_url = format!("sqlite://{}", db_path.display());
        
        // Create database if it doesn't exist
        if !sqlx::Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            sqlx::Sqlite::create_database(&db_url).await?;
        }
        
        let pool = SqlitePool::connect(&db_url).await?;
        
        // Run migrations
        Self::run_migrations(&pool).await?;
        Self::upgrade_diary_schema_if_needed(&pool).await?;
        Self::enforce_diary_unique_by_date(&pool).await?;
        
        Ok(Self { pool, db_path: db_path })
    }
    
    fn get_db_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "productivityapp", "app")
            .ok_or_else(|| anyhow::anyhow!("Failed to get project directories"))?;
        
        let data_dir = proj_dirs.data_dir();
        std::fs::create_dir_all(data_dir)?;
        
        Ok(data_dir.join("productivity.db"))
    }
    
    async fn run_migrations(pool: &SqlitePool) -> Result<()> {
        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS focus_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time DATETIME NOT NULL,
                end_time DATETIME,
                duration INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(pool)
        .await?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS diary_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date DATE NOT NULL,
                title TEXT,
                content TEXT NOT NULL,
                mood INTEGER,
                images TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(pool)
        .await?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS todos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT,
                completed BOOLEAN DEFAULT FALSE,
                priority INTEGER DEFAULT 0,
                due_date DATETIME,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(pool)
        .await?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS alarms (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                time TIME NOT NULL,
                days TEXT,
                enabled BOOLEAN DEFAULT TRUE,
                label TEXT,
                sound_path TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    async fn upgrade_diary_schema_if_needed(pool: &SqlitePool) -> Result<()> {
        let row: Option<(String,)> = sqlx::query_as("SELECT sql FROM sqlite_master WHERE type='table' AND name='diary_entries'")
            .fetch_optional(pool)
            .await?;
        if let Some((sql,)) = row {
            if sql.contains("date DATE NOT NULL UNIQUE") {
                sqlx::query(
                    r#"
                    CREATE TABLE IF NOT EXISTS diary_entries_new (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        date DATE NOT NULL,
                        title TEXT,
                        content TEXT NOT NULL,
                        mood INTEGER,
                        images TEXT,
                        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
                    )
                    "#
                )
                .execute(pool)
                .await?;
                sqlx::query(
                    "INSERT INTO diary_entries_new (id, date, title, content, mood, images, created_at, updated_at)
                     SELECT id, date, title, content, mood, images, created_at, updated_at FROM diary_entries"
                )
                .execute(pool)
                .await?;
                sqlx::query("DROP TABLE diary_entries")
                    .execute(pool)
                    .await?;
                sqlx::query("ALTER TABLE diary_entries_new RENAME TO diary_entries")
                    .execute(pool)
                    .await?;
            }
        }
        Ok(())
    }

    async fn enforce_diary_unique_by_date(pool: &SqlitePool) -> Result<()> {
        // Deduplicate keeping latest updated_at per date
        sqlx::query(
            r#"DELETE FROM diary_entries 
               WHERE (date, updated_at) NOT IN (
                 SELECT date, MAX(updated_at) FROM diary_entries GROUP BY date
               )"#
        )
        .execute(pool)
        .await?;
        // Ensure unique index on date
        sqlx::query(
            r#"CREATE UNIQUE INDEX IF NOT EXISTS idx_diary_entries_date ON diary_entries(date)"#
        )
        .execute(pool)
        .await?;
        Ok(())
    }
    
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn path(&self) -> &PathBuf {
        &self.db_path
    }
}