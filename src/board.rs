use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use vestaboard::Grid;

/// Vestaboard.
pub enum Board {
    /// Test board (terminal).
    Test {
        shown: Mutex<Grid>,
        writes: AtomicUsize,
        failing: AtomicBool,
    },

    /// Local Vestaboard.
    ///
    /// https://docs.vestaboard.com/docs/local-api/endpoints
    Local {
        client: reqwest::Client,
        host: String,
        key: String,
    },
}

impl Board {
    /// Construct board from environment variables.
    pub fn from_env() -> Result<Board> {
        if std::env::var("VESTABOARD_TEST").is_ok_and(|v| !v.is_empty() && v != "0") {
            return Ok(Board::test(Grid::EMPTY));
        }

        let host = std::env::var("VESTABOARD_HOST").unwrap_or_else(|_| "vestaboard.local".into());
        let key = std::env::var("VESTABOARD_KEY").context("Missing VESTABOARD_KEY")?;

        Ok(Board::Local {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()?,
            host,
            key,
        })
    }

    /// Construct a test board showing `grid`.
    pub fn test(grid: Grid) -> Board {
        Board::Test {
            shown: Mutex::new(grid),
            writes: AtomicUsize::new(0),
            failing: AtomicBool::new(false),
        }
    }

    /// Show text centered on board.
    pub async fn show(&self, text: &str) -> Result<()> {
        self.set_grid(&Grid::from_text(text)?).await
    }

    /// Set grid.
    pub async fn set_grid(&self, grid: &Grid) -> Result<()> {
        match self {
            Board::Test {
                shown,
                writes,
                failing,
            } => {
                if failing.load(Ordering::SeqCst) {
                    bail!("test failure");
                }
                println!("{grid}");
                *shown.lock().unwrap() = *grid;
                writes.fetch_add(1, Ordering::SeqCst);

                Ok(())
            }
            Board::Local { client, host, key } => {
                let res = client
                    .post(format!("http://{host}:7000/local-api/message"))
                    .header("X-Vestaboard-Local-Api-Key", key)
                    .json(&grid.codes())
                    .send()
                    .await?;
                if !res.status().is_success() {
                    bail!(
                        "Vestaboard Local API {}: {}",
                        res.status(),
                        res.text().await?
                    );
                }

                Ok(())
            }
        }
    }

    /// Get grid.
    pub async fn get_grid(&self) -> Result<Grid> {
        match self {
            Board::Test { shown, failing, .. } => {
                if failing.load(Ordering::SeqCst) {
                    bail!("test failure");
                }

                Ok(*shown.lock().unwrap())
            }
            Board::Local { client, host, key } => {
                #[derive(Deserialize)]
                struct Shown {
                    message: Grid,
                }

                let res = client
                    .get(format!("http://{host}:7000/local-api/message"))
                    .header("X-Vestaboard-Local-Api-Key", key)
                    .send()
                    .await?;
                if !res.status().is_success() {
                    bail!(
                        "Vestaboard Local API {}: {}",
                        res.status(),
                        res.text().await?
                    );
                }
                let shown: Shown = res.json().await?;

                Ok(shown.message)
            }
        }
    }
}
