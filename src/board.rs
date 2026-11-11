use anyhow::Result;
use async_trait::async_trait;
use vestaboard::Grid;

/// Write-only Vestaboard.
#[async_trait]
pub trait Board: Send + Sync {
    /// Show `grid` unless it's already showing.
    async fn set_grid(&self, grid: &Grid) -> Result<()>;
}
