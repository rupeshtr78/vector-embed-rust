use crate::app::constants::CHAT_API_URL;
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
    // print the commands
    // println!("{:?}", commands);
    // commands.print_command();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("chatbot")
        .enable_all()
        .build()
        .context("Failed to build runtime")?;

    // start a spinner
    // let pb = cli::cli_spinner().context("Failed to create spinner")?;
    // pb.set_message("Generating...");

    cli::cli(commands, rt, CHAT_API_URL).context("Failed to run Command")?;

    // pb.finish_with_message("Finished!");

    Ok(())
}
