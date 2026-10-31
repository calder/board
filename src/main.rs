use anyhow::Result;
use clap::{Parser, Subcommand};

use board::{Board, serve};
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
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    yansi::whenever(yansi::Condition::TTY_AND_COLOR);
    dotenvy::dotenv()?;
    let app = App::parse();
    let board = Board::from_env()?;

    match app.command {
        Command::Show(message) => board.show_grid(&message.to_grid()?).await,
        Command::Serve { host, port } => serve(board, &host, port).await,
    }
}
