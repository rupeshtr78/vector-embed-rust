use app::cli;
use app::commands::build_args;
use app::constants::EMBEDDING_URL;
use log::{error, info};

mod app;
mod docsplitter;
mod embedder;
mod lancevectordb;
mod pgvectordb;

fn main() {
    info!("Starting");

    let commands = build_args();
    // dbg_cmd(); // Debugging command
    let url = EMBEDDING_URL;

    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            error!("Failed to build runtime: {}", e);
            return;
        }
    };

    cli::cliV2(commands, rt, url);

    info!("Finished");
}
