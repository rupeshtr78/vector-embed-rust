use crate::config::config::{EmbedRequest, EmbedResponse};
use crate::config::config::{
    EMBEDDING_MODEL, EMBEDDING_URL, VECTOR_DB_DIM, VECTOR_DB_HOST, VECTOR_DB_NAME, VECTOR_DB_PORT,
    VECTOR_DB_TABLE, VECTOR_DB_USER,
};
use log::{debug, error, info};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;

mod config;
mod embedding;
mod vectordb;

#[tokio::main]
async fn main() {
    colog::init();
    info!("Starting");

    let url = EMBEDDING_URL;
    let model = EMBEDDING_MODEL;

    // let input = vec!["hello".to_string()];
    let input = vec![
        "The dog is barking",
        "The cat is purring",
        "The bear is growling",
    ]
    .iter()
    .map(|&s| s.to_string())
    .collect();

    let data = config::config::NewEmbedRequest(model.to_string(), input);

    // Wrap data with Arc and Mutex for thread-safe sharing
    let data_shared = Arc::new(Mutex::new(data));
    // let input_clone = data.input.clone();

    let embed_shared_data = Arc::clone(&data_shared);

    let embedding = task::spawn(async move {
        // Lock the shared data and perform operations
        let data = &*embed_shared_data.lock().await;

        match embedding::vector_embedding::create_embed_request(url, data).await {
            Ok(response) => response,
            Err(e) => {
                error!("Error: {}", e);
                return EmbedResponse {
                    model: "".to_string(),
                    embeddings: vec![],
                };
            }
        }
    });

    let response = embedding.await.unwrap_or_else(|e| {
        error!("Error: {:?}", e);
        EmbedResponse {
            model: "".to_string(),
            embeddings: vec![],
        }
    });

    // query the embeddings
    let query_input = vec!["some animal is purring".to_string()];
    let query_data = EmbedRequest {
        model: model.to_string(),
        input: query_input,
    };

    let query_embedding = task::spawn(async move {
        match embedding::vector_embedding::create_embed_request(url, &query_data).await {
            Ok(response) => response,
            Err(e) => {
                error!("Error: {}", e);
                return EmbedResponse {
                    model: "".to_string(),
                    embeddings: vec![],
                };
            }
        }
    });

    let query_response = query_embedding.await.unwrap_or_else(|e| {
        error!("Error: {:?}", e);
        EmbedResponse {
            model: "".to_string(),
            embeddings: vec![],
        }
    });

    let db_config = config::config::NewVectorDbConfig(
        VECTOR_DB_HOST,
        VECTOR_DB_PORT,
        VECTOR_DB_USER,
        VECTOR_DB_NAME,
    );

    let vector_shared_data = Arc::clone(&data_shared);

    let embed_thread = task::spawn(async move {
        let data = vector_shared_data.lock().await;
        let mut client = match vectordb::pg_vector::pg_client(&db_config) {
            Ok(client) => client,
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        };

        let table = VECTOR_DB_TABLE;
        let dim = VECTOR_DB_DIM;
        match vectordb::pg_vector::create_table(&mut client, table, dim) {
            Ok(_) => {
                info!("Create table successful");
            }
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        }

        match vectordb::pg_vector::load_vector_data(&mut client, table, &data, &response.embeddings)
        {
            Ok(_) => {
                info!("Load vector data successful");
            }
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        }

        // match vectordb::pg_vector::select_embeddings(&mut client, &table) {
        //     Ok(_) => {
        //         info!("Select main successful");
        //     }
        //     Err(e) => {
        //         error!("Error: {}", e);
        //         return;
        //     }
        // };

        // query the embeddings
        let query =
            vectordb::pg_vector::query_nearest(&mut client, table, &query_response.embeddings);
        match query {
            Ok(_) => {
                debug!("Query nearest vector successful");
            }
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

    embed_thread.await.unwrap_or_else(|e| {
        error!("Embed thread panicked: {:?}", e);
    });

    info!("Done with main");
}

fn run_embedding() {
    // Your embedding logic here
    println!("Running embedding...");
}
