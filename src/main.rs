use crate::config::config::{EmbedRequest, EmbedResponse};
use crate::config::config::{
    EMBEDDING_MODEL, EMBEDDING_URL, VECTOR_DB_DIM, VECTOR_DB_HOST, VECTOR_DB_NAME, VECTOR_DB_PORT,
    VECTOR_DB_TABLE, VECTOR_DB_USER,
};

use log::{debug, error, info};
use postgres::Client;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::thread::{self};

mod config;
mod embedding;
mod vectordb;

fn main() {
    colog::init();
    info!("Starting");

    let url = EMBEDDING_URL;
    let model = EMBEDDING_MODEL;

    // let input = vec!["hello".to_string()];
    let input = vec![
        "The dog is barking",
        "The cat is purring",
        "The bear is growling",
    ];

    // Arc (Atomic Reference Counted) pointer. It is a thread-safe reference-counting pointer.
    let embed_request_arc = config::config::ArcEmbedRequest(model, input);

    let embed_request_arc_clone = Arc::clone(&embed_request_arc);

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

    // Run embedding request
    let embed_response = rt.block_on(run_embedding(&url, &embed_request_arc));

    // query the embeddings
    let query_input = vec!["some animal is purring"];
    let query_request_arc = config::config::ArcEmbedRequest(model, query_input);
    let query_response = rt.block_on(run_embedding(&url, &query_request_arc));

    let db_config = config::config::NewVectorDbConfig(
        VECTOR_DB_HOST,
        VECTOR_DB_PORT,
        VECTOR_DB_USER,
        VECTOR_DB_NAME,
    );

    let table = VECTOR_DB_TABLE;
    let dim = VECTOR_DB_DIM;

    let db1 = db_config.clone();

    let embed_thread = thread::spawn(move || {
        let mut client = match vectordb::pg_vector::pg_client(&db1) {
            Ok(client) => client,
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        };
        let embed_data = embed_request_arc_clone.read().unwrap();

        match run_load_data(
            &mut client,
            table,
            dim,
            &embed_data,
            &embed_response.embeddings,
        ) {
            Ok(_) => {}
            Err(e) => {
                error!("Error: {}", e);
            }
        }

        if let Err(e) = client.close() {
            error!("Error: {}", e);
            return;
        }
    });

    // Wait for the embed thread to finish
    if let Err(e) = embed_thread.join() {
        error!("Error: {:?}", e);
    }

    // query the embeddings in a separate thread
    let query_thread = thread::spawn(move || {
        let mut client = match vectordb::pg_vector::pg_client(&db_config) {
            Ok(client) => client,
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        };

        // query the embeddings
        let query =
            vectordb::pg_vector::query_nearest(&mut client, &table, &query_response.embeddings);
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

    if let Err(e) = query_thread.join() {
        error!("Error: {:?}", e);
    }

    debug!("Done with main");
}

async fn run_embedding(url: &str, embed_data: &Arc<RwLock<EmbedRequest>>) -> EmbedResponse {
    let embed_data = match embed_data.read() {
        Ok(data) => data,
        Err(e) => {
            error!("Error: {}", e);
            return EmbedResponse {
                model: "".to_string(),
                embeddings: vec![],
            };
        }
    };

    match embedding::vector_embedding::create_embed_request(url, &embed_data).await {
        Ok(embed_response) => embed_response,
        Err(e) => {
            error!("Error: {}", e);
            return EmbedResponse {
                model: "".to_string(),
                embeddings: vec![],
            };
        }
    }
}

fn run_load_data(
    pg_client: &mut Client,
    table: &str,
    dimension: i32,
    embed_request: &EmbedRequest,
    embeddings: &Vec<Vec<f32>>,
) -> Result<(), Box<dyn Error>> {
    match vectordb::pg_vector::create_table(pg_client, &table, dimension) {
        Ok(_) => {
            debug!("Create table successful");
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }

    match vectordb::pg_vector::load_vector_data(pg_client, &table, &embed_request, embeddings) {
        Ok(_) => {
            debug!("Load vector data successful");
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }

    Ok(())
}
