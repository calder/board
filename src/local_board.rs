use std::time::Duration;

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use vestaboard::Grid;

use crate::board::Board;

/// Vestaboard on the local network.
///
/// https://docs.vestaboard.com/docs/local-api/endpoints
pub struct LocalBoard {
    client: reqwest::Client,
    host: String,
    key: String,
}

impl LocalBoard {
    /// Construct board from environment variables.
    pub fn from_env() -> Result<LocalBoard> {
        let host = std::env::var("VESTABOARD_HOST").unwrap_or_else(|_| "vestaboard.local".into());
        let key = std::env::var("VESTABOARD_KEY").context("Missing VESTABOARD_KEY")?;

        Ok(LocalBoard {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()?,
            host,
            key,
        })
    }
}

#[async_trait]
impl Board for LocalBoard {
    async fn set_grid(&self, grid: &Grid) -> Result<()> {
        let res = self
            .client
            .post(format!("http://{}:7000/local-api/message", self.host))
            .header("X-Vestaboard-Local-Api-Key", &self.key)
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
