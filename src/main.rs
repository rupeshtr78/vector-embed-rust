use anyhow::{Context, Result};
use app::cli;
use app::commands::build_args;

mod app;
mod chat;
mod docsplitter;
mod embedder;
mod lancevectordb;
mod pgvectordb;

fn main() -> Result<()> {
    println!("Starting Application");

    // app::commands::dbg_cmd(); // Debugging

    let commands = build_args();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("chatbot")
        .enable_all()
        .build()
        .context("Failed to build runtime")?;

    cli::cli(commands, rt).context("Failed to run Command")?;

    println!("Exiting Chatbot");

    Ok(())
}
