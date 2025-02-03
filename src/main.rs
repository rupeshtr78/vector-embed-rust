use std::sync::{Arc, Mutex};

use crate::app::config::VectorDbConfig;
use crate::app::constants::{VECTOR_DB_HOST, VECTOR_DB_NAME, VECTOR_DB_PORT, VECTOR_DB_USER};

use app::cli;
use app::commands::{build_args, Commands};
use app::constants::EMBEDDING_URL;
use hyper::Client as HttpClient;
use log::{error, info, warn};
use pgvectordb::pg_vector;
use postgres::Client;
use crate::app::commands::dbg_cmd;

mod app;
mod embedder;
mod pgvectordb;
mod loader;

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
