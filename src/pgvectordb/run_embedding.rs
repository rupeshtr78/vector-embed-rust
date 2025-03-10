use crate::embedder::config::EmbedRequest;
use crate::embedder::fetch_embedding;
use crate::pgvectordb;
use anyhow::Context;
use anyhow::Result;
use hyper::client::HttpConnector;
use hyper::Client as HttpClient;
use log::{debug, error};
use postgres::Client;
use tokio::sync::Mutex;

/// Run the embedding request and load the embeddings into the database
/// Arguments:
/// - rt: &tokio::runtime::Runtime
/// - embed_model: String
/// - input_list: &Vec<String>
/// - vector_table: String
/// - dimension: String
/// - db_config: VectorDbConfig
/// Returns:
/// - Result<JoinHandle<()>, Box<dyn Error>>
pub async fn run_embedding_load(
    url: &str,
    embed_model: String,
    input_list: &Vec<String>,
    vector_table: String,
    dimension: String,
    client: Mutex<Client>,
    http_client: &HttpClient<HttpConnector>,
) -> Result<()> {
    debug!("Starting Loading Embeddings");

    // Arc (Atomic Reference Counted) pointer. It is a thread-safe reference-counting pointer.
    let embed_request_arc =
        EmbedRequest::NewArcEmbedRequest(&embed_model, input_list, &"".to_string(), None);
    // let embed_request_arc_clone = Arc::clone(&embed_request_arc);

    // Run embedding request in a separate thread
    let embed_response = fetch_embedding(url, &embed_request_arc, http_client)
        .await
        .context("Failed to fetch embedding")?;

    let dim = dimension.parse::<i32>().unwrap_or_else(|_| {
        error!("Failed to parse dimension");
        0
    });

    let mut client = client.lock().await;
    let embed_data = embed_request_arc.read().await;

    pg_persist_embedding_data(
        &mut client,
        &vector_table,
        dim,
        &embed_data,
        &embed_response.embeddings,
    )
    .context("Failed to persist embedding data")?;

    debug!("Finished Loading Embeddings");
    Ok(())
}

/// Persist the embedding data into the postgres database
/// Arguments:
/// - pg_client: &mut Client
/// - table: &String
/// - dimension: i32
/// - embed_request: &EmbedRequest
/// - embeddings: &Vec<Vec<f32>>
/// Returns:
/// - Result<(), Box<dyn Error>>
pub fn pg_persist_embedding_data(
    pg_client: &mut Client,
    table: &String,
    dimension: i32,
    embed_request: &EmbedRequest,
    embeddings: &Vec<Vec<f32>>,
) -> Result<()> {
    debug!("Loading data into table");
    match pgvectordb::pg_vector::create_table(pg_client, table, dimension) {
        Ok(_) => {
            debug!("Create table successful");
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }

    match pgvectordb::pg_vector::load_vector_data(pg_client, table, embed_request, embeddings) {
        Ok(_) => {
            debug!("Load vector data successful");
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }

    Ok(())
}
