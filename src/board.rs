use anyhow::{Context, Result, bail};

use vestaboard::Grid;

/// Vestaboard.
pub enum Board {
    /// Test board (terminal).
    Test,

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
            return Ok(Board::Test);
        }

        let host = std::env::var("VESTABOARD_HOST").unwrap_or_else(|_| "vestaboard.local".into());
        let key = std::env::var("VESTABOARD_KEY").context("Missing VESTABOARD_KEY")?;

        Ok(Board::Local {
            client: reqwest::Client::new(),
            host,
            key,
        })
    }

    /// Show text centered on board.
    pub async fn show(&self, text: &str) -> Result<()> {
        self.show_grid(&Grid::from_text(text)?).await
    }

    /// Show full text grid on board.
    pub async fn show_grid(&self, grid: &Grid) -> Result<()> {
        match self {
            Board::Test => {
                println!("{grid}");

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
}
