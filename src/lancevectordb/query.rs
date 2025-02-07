use crate::app::config::EmbedRequest;
use crate::app::constants::EMBEDDING_URL;
use crate::embedder;
use ::hyper::Client as HttpClient;
use anyhow::{Context, Result};
use futures::StreamExt;
use hyper::client::HttpConnector;
use lancedb::query::ExecutableQuery;
use lancedb::query::IntoQueryVector;
use lancedb::Connection;
use log::{debug, error, info};

/// Run the query to get the nearest embeddings
/// Arguments:
/// - rt: &tokio::runtime::Runtime
/// - embed_model: String
/// - input_list: &Vec<String>
/// - vector_table: String
/// - db_config: VectorDbConfig
/// Returns: ()
pub async fn run_query(
    db: &mut Connection,
    embed_model: String,
    input_list: &Vec<String>,
    vector_table: String,
    http_client: &HttpClient<HttpConnector>,
) {
    // colog::init();

    info!("Starting query");

    // let commands = build_args();
    info!("Length of input list: {}", input_list[0].len());
    // check if list is length one String is length one
    if input_list.len() == 1 && input_list[0].len() == 0 {
        error!("Query Input is empty");
        return;
    }

    let url = EMBEDDING_URL;

    let query_request_arc =
        EmbedRequest::NewArcEmbedRequest(&embed_model, &input_list, &"".to_string());
    let query_response =
        embedder::run_embedding::fetch_embedding(&url, &query_request_arc, http_client).await;

    let query_vector = query_response.embeddings[0].clone();

    query_table(db, vector_table.as_str(), query_vector)
        .await
        .unwrap();

    debug!("Finishes running query");
}

pub async fn query_table(
    db: &mut Connection,
    table_name: &str,
    query_vector: impl IntoQueryVector,
) -> Result<()> {
    let table = db
        .open_table(table_name)
        .execute()
        .await
        .context("Failed to open a table")?;

    let stream = table
        .query()
        .nearest_to(query_vector)
        .unwrap()
        .refine_factor(5)
        .nprobes(10)
        .execute()
        .await
        .unwrap();

    let batches = stream.collect::<Vec<_>>().await;

    for result in batches {
        let batch = result.unwrap();
        println!("Batch: {:?}", batch);
    }

    Ok(())
}
