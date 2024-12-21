use std::sync::{Arc, Mutex};

use crate::app::config::NewVectorDbConfig;
use crate::app::constants::{VECTOR_DB_HOST, VECTOR_DB_NAME, VECTOR_DB_PORT, VECTOR_DB_USER};

use app::commands::{build_args, Commands};
use app::constants::EMBEDDING_URL;
use hyper::Client as HttpClient;
use log::{error, info, warn};
use postgres::Client;
use vectordb::pg_vector;

mod app;
mod embedding;
mod vectordb;

fn main() {
    info!("Starting");
    let commands = build_args();

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

    let db_config = NewVectorDbConfig(
        VECTOR_DB_HOST,
        VECTOR_DB_PORT,
        VECTOR_DB_USER,
        VECTOR_DB_NAME,
    );

    // Initialize the client outside the thread and wrap it in Arc<Mutex>
    let client: Arc<Mutex<Client>> = Arc::new(Mutex::new(match pg_vector::pg_client(&db_config) {
        Ok(client) => client,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    }));

    // Initialize the http client outside the thread // TODO wrap in Arc<Mutex>
    let http_client = HttpClient::new();

    if commands.is_write() {
        if let Some(Commands::Write {
            input,
            model,
            table,
            dim,
        }) = commands.write()
        {
            let input_list = input;
            let embed_model = model.to_string();
            let vector_table = table.to_string();
            let dimension = dim.to_string();
            info!("Using the Write arguments below:");
            info!(" Input Length: {:?}", input.len());
            info!(" Model: {:?}", model);
            info!(" Table: {:?}", table);
            info!(" Dimension: {:?}", dim);

            let embed_handler = embedding::run_embedding::run_embedding_load(
                &rt,
                url,
                embed_model,
                input_list,
                vector_table,
                dimension,
                client,
                &http_client,
            );

            match embed_handler {
                Ok(_) => {
                    info!("Embedding loaded successfully");
                }
                Err(e) => {
                    error!("Error: {}", e);
                }
            }
        }
        rt.shutdown_timeout(std::time::Duration::from_secs(1));
    } else if commands.is_query() {
        if let Some(Commands::Query {
            input,
            model,
            table,
        }) = commands.query()
        {
            let input_list = input;
            let embed_model = model.to_string();
            let vector_table = table.to_string();
            info!("Query command is run with below arguments:");
            info!(" Query: {:?}", input);
            info!(" Model: {:?}", model);
            info!(" Table: {:?}", table);

            vectordb::query_vector::run_query(
                &rt,
                embed_model,
                input_list,
                vector_table,
                client,
                &http_client,
            );

            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
    } else {
        warn!("No embedding command provided");
    }
}
