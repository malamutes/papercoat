use anyhow::Result;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub template: String,
    pub format: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub ocr: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            template: "default".into(),
            format: "tex".into(),
            title: None,
            author: None,
            ocr: false,
        }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        let base = BaseDirs::new()
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        base.join("papercoat").join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    pub fn reset() -> Result<()> {
        let path = Self::path();
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    pub fn merge_with_cli(
        &self,
        template: Option<String>,
        title: Option<String>,
        author: Option<String>,
    ) -> Self {
        let mut c = self.clone();
        if let Some(t) = template {
            c.template = t;
        }
        if let Some(t) = title {
            c.title = Some(t);
        }
        if let Some(a) = author {
            c.author = Some(a);
        }
        c
    }
}
