use super::pg_vector;
use crate::app::config::VectorDbConfig;
use crate::app::config::{NewArcEmbedRequest, NewVectorDbConfig};
use crate::app::constants::EMBEDDING_URL;
use crate::embedding;
use log::LevelFilter;
use log::{debug, error, info};
use postgres::Client;
use std::sync::RwLock;
use std::thread;

pub fn run_query(
    rt: &tokio::runtime::Runtime,
    embed_model: String,
    input_list: &Vec<String>,
    vector_table: String,
    db_config: VectorDbConfig,
) {
    // colog::init();

    info!("Starting query");

    // let commands = build_args();

    let url = EMBEDDING_URL;

    let query_request_arc = NewArcEmbedRequest(&embed_model, &input_list);
    let query_response = rt.block_on(embedding::run_embedding::run_embedding(
        &url,
        &query_request_arc,
    ));

    let query_thread = thread::Builder::new().name("query_thread".to_owned());

    let run_query_thread = query_thread.spawn(move || {
        let mut client = match pg_vector::pg_client(&db_config) {
            Ok(client) => client,
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        };

        // query the embeddings
        let query =
            pg_vector::query_nearest(&mut client, &vector_table, &query_response.embeddings);
        match query {
            Ok(_) => {}
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        }

        if let Err(e) = client.close() {
            error!("Error: {}", e);
            return;
        }
    });

    match run_query_thread {
        Ok(handle) => match handle.join() {
            Ok(_) => info!("Finished running query thread!"),
            Err(_) => eprintln!("Thread panicked!"),
        },
        Err(_) => error!("Failed to spawn thread!"),
    }

    debug!("Done with main");
}
