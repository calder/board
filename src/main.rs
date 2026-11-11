use std::path::PathBuf;

use anyhow::{Context, Result};
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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();
    let app = App::parse();
    let board = board()?;

    match app.command {
        Command::Show(message) => board.set_grid(&message.to_grid()?).await,
        Command::Serve { host, port, state } => serve(board, state, &host, port).await,
    }
}

/// Construct a board from environment variables.
fn board() -> Result<Box<dyn Board>> {
    let host = std::env::var("VESTABOARD_HOST").unwrap_or_default();
    if host.is_empty() {
        log::info!("VESTABOARD_HOST is unset; running in dry-run mode");

        return Ok(Box::new(TerminalBoard::default()));
    }
    let key = std::env::var("VESTABOARD_KEY").context("Missing VESTABOARD_KEY")?;

    Ok(Box::new(LocalBoard::new(host, key)?))
}
