mod app;
mod config;
mod event;
mod factorio;
mod handler;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fml", about = "Factorio Mod Manager for headless servers")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize FML configuration
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            config::FmlConfig::init()?;
        }
        None => {
            println!("Hello, FML!");
        }
    }

    Ok(())
}
