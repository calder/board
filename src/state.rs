use std::io::ErrorKind;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use vestaboard::Grid;

/// App state kept across restarts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// Grid the board should show.
    pub grid: Option<Grid>,
}

impl State {
    /// Load state from `path` or start empty if it doesn't exist.
    pub fn load(path: &Path) -> Result<State> {
        match std::fs::read_to_string(path) {
            Ok(json) => serde_json::from_str(&json)
                .with_context(|| format!("Can't parse {}", path.display())),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(State::default()),
            Err(err) => Err(err).with_context(|| format!("Can't read {}", path.display())),
        }
    }

    /// Save state to `path` atomically.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("Can't create {}", dir.display()))?;
        }
        let temp = path.with_extension("tmp");
        std::fs::write(&temp, serde_json::to_string(self)?)
            .with_context(|| format!("Can't write {}", temp.display()))?;
        std::fs::rename(&temp, path).with_context(|| format!("Can't write {}", path.display()))?;

        Ok(())
    }
}
