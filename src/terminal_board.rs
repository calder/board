use std::sync::Mutex;

use anyhow::Result;
use async_trait::async_trait;
use vestaboard::Grid;

use crate::board::Board;

/// Dry-run board that prints to the terminal.
#[derive(Default)]
pub struct TerminalBoard {
    shown: Mutex<Option<Grid>>,
}

#[async_trait]
impl Board for TerminalBoard {
    async fn set_grid(&self, grid: &Grid) -> Result<()> {
        let mut shown = self.shown.lock().expect("lock isn't poisoned");
        if *shown != Some(*grid) {
            log::info!("{grid}");
            *shown = Some(*grid);
        }

        Ok(())
    }
}
