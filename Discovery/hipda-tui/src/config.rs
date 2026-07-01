use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_forums")]
    pub forums: Vec<i32>,
    #[serde(default)]
    pub show_sticky: bool,
    #[serde(default)]
    pub tail_text: String,
    #[serde(default = "default_true")]
    pub add_tail: bool,
    #[serde(default)]
    pub uid: String,
    #[serde(default)]
    pub blacklist: Vec<String>,
    #[serde(default)]
    pub favorites: Vec<String>,
    #[serde(default)]
    pub attention: Vec<String>,
}

fn default_forums() -> Vec<i32> { vec![2, 6, 7] }
fn default_true() -> bool { true }

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = config_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&content).unwrap_or_default())
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn clear_auth(&mut self) { self.uid.clear(); }
    pub fn is_login_info_valid(&self) -> bool { !self.username.is_empty() && !self.password.is_empty() }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            username: String::new(), password: String::new(),
            forums: default_forums(), show_sticky: true,
            tail_text: String::new(), add_tail: true,
            uid: String::new(), blacklist: vec![],
            favorites: vec![], attention: vec![],
        }
    }
}

fn config_path() -> anyhow::Result<PathBuf> {
    let base = std::env::var("XDG_CONFIG_HOME").ok()
        .map(PathBuf::from)
        .or_else(|| std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    Ok(base.join("hipda-tui").join("config.toml"))
}
