use crate::app::constants::CHAT_API_URL;
use anyhow::{Context, Result};
use app::cli;
use app::commands::build_args;
use log::info;

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
    // let url = format!("{}/{}", CHAT_API_URL, "api/embed");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("chatbot")
        .enable_all()
        .build()
        .context("Failed to build runtime")?;

    cli::cli(commands, rt, CHAT_API_URL).context("Failed to run Command")?;

    info!("Exiting Chatbot");

    Ok(())
}
