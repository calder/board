use anyhow::Result;
use async_trait::async_trait;
use vestaboard::Grid;

use crate::board::Board;

/// Dry-run board that prints to the terminal.
pub struct TerminalBoard;

#[async_trait]
impl Board for TerminalBoard {
    async fn set_grid(&self, grid: &Grid) -> Result<()> {
        println!("{grid}");

        Ok(())
    }
}
