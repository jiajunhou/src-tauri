use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Local};
use anyhow::Result;
use directories::ProjectDirs;

pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    pub fn new() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("com", "productivityapp", "app")
            .ok_or_else(|| anyhow::anyhow!("Failed to get project directories"))?;
        
        let backup_dir = proj_dirs.data_dir().join("backups");
        fs::create_dir_all(&backup_dir)?;
        
        Ok(Self { backup_dir })
    }
    
    pub fn create_backup(&self, db_path: &PathBuf) -> Result<PathBuf> {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("backup_{}.db", timestamp);
        let backup_path = self.backup_dir.join(backup_name);
        
        fs::copy(db_path, &backup_path)?;
        
        // Clean up old backups (keep last 10)
        self.cleanup_old_backups()?;
        
        Ok(backup_path)
    }
    
    fn cleanup_old_backups(&self) -> Result<()> {
        let mut backups: Vec<(PathBuf, DateTime<Local>)> = Vec::new();
        
        for entry in fs::read_dir(&self.backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("db") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(created) = metadata.created() {
                        let datetime: DateTime<Local> = created.into();
                        backups.push((path, datetime));
                    }
                }
            }
        }
        
        // Sort by creation time, newest first
        backups.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Remove backups older than the 10 most recent
        for (path, _) in backups.iter().skip(10) {
            let _ = fs::remove_file(path);
        }
        
        Ok(())
    }
    
    pub fn get_backup_dir(&self) -> &PathBuf {
        &self.backup_dir
    }
}