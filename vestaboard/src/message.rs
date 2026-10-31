use anyhow::Result;

use super::background::Background;
use super::grid::Grid;

/// Text with an optional margin and background.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "cli", derive(clap::Args))]
pub struct Message {
    /// Text to show.
    pub text: String,

    /// Leave a blank margin around the text.
    #[cfg_attr(feature = "cli", arg(long))]
    #[serde(default)]
    pub pad: bool,

    /// Fill around the text.
    #[cfg_attr(feature = "cli", arg(long))]
    pub background: Option<Background>,
}

impl Message {
    /// Lay out the message on the board.
    pub fn to_grid(&self) -> Result<Grid> {
        let mut grid = Grid::from_text(&self.text)?;
        if self.pad {
            grid.pad();
        }
        if let Some(background) = self.background {
            grid.fill(background);
        }

        Ok(grid)
    }
}
