use anyhow::{anyhow, Context, Result};
use app::cli;
use app::commands::build_args;
use app::constants::EMBEDDING_URL;
use log::{error, info};

mod app;
mod chat;
mod docsplitter;
mod embedder;
mod lancevectordb;
mod pgvectordb;

fn main() -> Result<()> {
    info!("Starting");

    // app::commands::dbg_cmd(); // Debugging

    let commands = build_args();
    let url = EMBEDDING_URL;

    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            error!("Failed to build runtime: {}", e);
            return Err(anyhow!("Failed to build runtime: {}", e));
        }
    };

    cli::cli(commands, rt, url).context("Failed to run Command")?;

    info!("Exiting Chatbot");

    Ok(())
}
