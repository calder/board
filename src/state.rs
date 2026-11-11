use std::io::ErrorKind;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use vestaboard::Grid;

/// How long past the ETA to keep showing a countdown.
const TIMEOUT: Duration = Duration::from_mins(10);

/// App state kept across restarts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// Grid the board should show.
    pub grid: Grid,

    /// Arrival time of an active countdown.
    #[serde(default)]
    pub countdown: Option<Timestamp>,
}

impl Default for State {
    fn default() -> State {
        State {
            grid: Grid::EMPTY,
            countdown: None,
        }
    }
}

impl State {
    /// Load state from `path` or start from a blank grid if it doesn't exist.
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

    /// Grid to show at `now` including any countdown.
    pub fn displayed(&self, now: Timestamp) -> Grid {
        self.countdown
            .and_then(|eta| countdown_grid(eta, now))
            .unwrap_or(self.grid)
    }
}

/// Countdown grid at `now` or `None` if the countdown has finished.
fn countdown_grid(eta: Timestamp, now: Timestamp) -> Option<Grid> {
    let minutes = minutes_shown(eta, now)?;
    let unit = if minutes == 1 { "minute" } else { "minutes" };
    let clock = eta
        .in_tz("America/Los_Angeles")
        .ok()?
        .strftime("%H:%M")
        .to_string();

    Grid::from_text(&format!(
        "DAD IN TRANSIT\nETA {clock}\n\nT-{minutes} {unit}"
    ))
    .ok()
}

/// Minutes to show at `now` or `None` if the countdown has finished.
fn minutes_shown(eta: Timestamp, now: Timestamp) -> Option<u32> {
    let timeout = eta.checked_add(TIMEOUT).ok()?;
    if now >= timeout {
        return None;
    }
    if now >= eta {
        return Some(1);
    }
    let secs = eta.duration_since(now).as_secs().max(0) as u64;
    let left = secs.div_ceil(60) as u32;

    Some(if secs <= 5 * 60 {
        left.max(1)
    } else if secs <= 10 * 60 {
        10
    } else if secs <= 30 * 60 {
        30
    } else {
        left.div_ceil(30) * 30
    })
}
