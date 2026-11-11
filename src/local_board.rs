use std::time::Duration;

use anyhow::{Result, bail};
use async_trait::async_trait;
use tokio::sync::Mutex;
use vestaboard::Grid;

use crate::board::Board;

/// Vestaboard on the local network.
///
/// https://docs.vestaboard.com/docs/local-api/endpoints
pub struct LocalBoard {
    client: reqwest::Client,
    host: String,
    key: String,
    shown: Mutex<Option<Grid>>,
}

impl LocalBoard {
    /// Construct a board at `host` using local API `key`.
    pub fn new(host: String, key: String) -> Result<LocalBoard> {
        Ok(LocalBoard {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()?,
            host,
            key,
            shown: Mutex::new(None),
        })
    }
}

#[async_trait]
impl Board for LocalBoard {
    async fn set_grid(&self, grid: &Grid) -> Result<()> {
        let mut shown = self.shown.lock().await;
        if *shown == Some(*grid) {
            return Ok(());
        }
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
        *shown = Some(*grid);

        Ok(())
    }
}
