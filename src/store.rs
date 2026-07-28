use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCurve {
    #[serde(default)]
    pub name: String,
    pub expr: String,
    pub color: [u8; 4],
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredState {
    pub curves: Vec<StoredCurve>,
    pub x_min: f64,
    pub x_max: f64,
    pub samples: usize,
    pub dark_mode: bool,
}

pub fn load_session() -> Result<Option<StoredState>> {
    let path = session_file()?;
    if !path.exists() {
        return Ok(None);
    }

    let data = fs::read_to_string(&path)
        .with_context(|| format!("failed to read session file at {}", path.display()))?;
    let state = toml::from_str(&data)
        .with_context(|| format!("failed to parse session file at {}", path.display()))?;
    Ok(Some(state))
}

pub fn save_session(state: &StoredState) -> Result<()> {
    let path = session_file()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create session directory {}", parent.display()))?;
    }

    let data = toml::to_string_pretty(state)?;
    fs::write(&path, data)
        .with_context(|| format!("failed to write session file at {}", path.display()))
}

fn session_file() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "RustNative", "GraphCalc")
        .ok_or_else(|| anyhow::anyhow!("unable to determine data directory"))?;
    Ok(dirs.data_dir().join("session.toml"))
}
