use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use board::{Board, LocalBoard, TerminalBoard, serve};
use vestaboard::Message;

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
    Show(Message),

    /// Run the HTTP API for phones and automations.
    Serve {
        /// Address to listen on.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Port to listen on.
        #[arg(long, default_value_t = 8001)]
        port: u16,

        /// File to keep app state in across restarts.
        #[arg(long, default_value = "state/state.json")]
        state: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    yansi::whenever(yansi::Condition::TTY_AND_COLOR);
    dotenvy::dotenv()?;
    let app = App::parse();
    let board = board()?;

    match app.command {
        Command::Show(message) => board.set_grid(&message.to_grid()?).await,
        Command::Serve { host, port, state } => serve(board, state, &host, port).await,
    }
}

/// Construct a board from environment variables.
fn board() -> Result<Box<dyn Board>> {
    if std::env::var("VESTABOARD_TEST").is_ok_and(|v| !v.is_empty() && v != "0") {
        return Ok(Box::new(TerminalBoard));
    }

    Ok(Box::new(LocalBoard::from_env()?))
}
