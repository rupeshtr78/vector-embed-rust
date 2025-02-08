use crate::app::config::EmbedRequest;
use crate::app::constants::EMBEDDING_URL;
use crate::embedder;
use ::hyper::Client as HttpClient;
use anyhow::{anyhow, Context, Result};
use futures::StreamExt;
use hyper::client::HttpConnector;
use lancedb::query::ExecutableQuery;
use lancedb::query::IntoQueryVector;
use lancedb::query::QueryBase;
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
    vector_table: &str,
    http_client: &HttpClient<HttpConnector>,
) -> Result<()> {
    // colog::init();

    info!("Starting query");

    // let commands = build_args();
    info!("Length of input list: {}", input_list[0].len());
    // check if list is length one String is length one
    if input_list.len() == 1 && input_list[0].len() == 0 {
        error!("Query Input is empty");
        return Err(anyhow!("Query Input is empty"));
    }

    let url = EMBEDDING_URL;

    let query_request_arc =
        EmbedRequest::NewArcEmbedRequest(&embed_model, &input_list, &"".to_string());
    let query_response =
        embedder::run_embedding::fetch_embedding(&url, &query_request_arc, http_client).await;

    let query_vector = query_response.embeddings[0].clone();

    query_table(db, vector_table, query_vector)
        .await
        .context("Failed to query table")?;

    debug!("Finishes running query");

    Ok(())
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

    // let lower_bound = Some(0.5);
    // let upper_bound = Some(1.5);

    let stream = table
        .query()
        .nearest_to(query_vector) // Find the nearest vectors to the query vector
        .context("Failed to select nearest vector")?
        // .distance_range(lower_bound, upper_bound) // bug in DataFusion library
        .distance_type(lancedb::DistanceType::Cosine)
        .refine_factor(10)
        .nprobes(5)
        .postfilter()
        .only_if("_distance > 0.3 AND _distance < 1")
        .select(lancedb::query::Select::Columns(vec![
            "_distance".to_string(),
            "id".to_string(),
            "content".to_string(),
        ]))
        .execute()
        .await
        .context("Failed to execute query and fetch records")?;

    let batches = stream.collect::<Vec<_>>().await;

    for batch in batches {
        let batch: arrow_array::RecordBatch = batch.unwrap();
        // println!("Batch: {:?}", batch);

        let schema = batch.schema(); // Bind schema to a variable
        for i in 0..batch.num_columns() {
            let column = batch.column(i);
            let column_name = schema.field(i).name(); // Now this is safe

            println!("Column {:?}: {:?}", column_name, column);
        }
    }

    Ok(())
}
