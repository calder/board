use anyhow::Result;
use clap::{Parser, Subcommand};

use board::vestaboard::{Background, Board, Grid};

/// Vestaboard control.
#[derive(Parser)]
struct App {
    /// Command.
    #[command(subcommand)]
    command: Command,
}

/// Command.
#[derive(Subcommand)]
enum Command {
    /// Show text and exit.
    Show {
        /// Text to show.
        text: String,

        /// Leave a blank margin around the text.
        #[arg(long)]
        pad: bool,

        /// Fill around the text.
        #[arg(long)]
        background: Option<Background>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    yansi::whenever(yansi::Condition::TTY_AND_COLOR);
    dotenvy::dotenv()?;
    let app = App::parse();
    let board = Board::from_env()?;

    match app.command {
        Command::Show {
            text,
            pad,
            background,
        } => {
            let mut grid = Grid::from_text(&text)?;
            if pad {
                grid.pad();
            }
            if let Some(background) = background {
                grid.fill(background);
            }

            board.show_grid(&grid).await
        }
    }
}
