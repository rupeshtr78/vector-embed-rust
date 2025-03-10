use super::pg_vector;
use crate::app::constants::CHAT_API_URL;
use crate::embedder::config::EmbedRequest;
use anyhow::Context;
use anyhow::Result;
use hyper::client::HttpConnector;
use ::hyper::Client as HttpClient;
use log::{debug, error, info};
use postgres::Client;

/// Run the query to get the nearest embeddings
/// Arguments:
/// - rt: &tokio::runtime::Runtime
/// - embed_model: String
/// - input_list: &Vec<String>
/// - vector_table: String
/// - db_config: VectorDbConfig
/// - http_client: &HttpClient<HttpConnector>
/// Returns:
/// - Result<()>: Result of the query
pub async fn run_pg_vector_query(
    rt: &tokio::runtime::Runtime,
    embed_model: String,
    input_list: &Vec<String>,
    vector_table: String,
    client: &mut Client,
    http_client: &HttpClient<HttpConnector>,
) -> Result<()> {
    // colog::init();

    info!("Starting query");

    // let commands = build_args();
    info!("Length of input list: {}", input_list[0].len());
    // check if list is length one String is length one
    if input_list.len() == 1 && input_list[0].is_empty() {
        error!("Query Input is empty");
        return Err(anyhow::anyhow!("Query Input is empty"));
    }

    let url = format!("{}/{}", CHAT_API_URL, "api/embed");

    let query_request_arc =
        EmbedRequest::NewArcEmbedRequest(&embed_model, input_list, &"".to_string(), None);
    let query_response = rt
        .block_on(crate::embedder::fetch_embedding(
            &url,
            &query_request_arc,
            http_client,
        ))
        .context("Failed to fetch embedding")?;

    // query the embeddings from the vector table TODO - handle the query response
    let query = pg_vector::query_nearest(client, &vector_table, &query_response.embeddings)
        .await
        .context("Failed to query nearest embeddings");

    debug!("Query Response: {:?}", query);
    Ok(())
}
