use crate::embedder::config::EmbedRequest;
use crate::app::constants::EMBEDDING_URL;
use crate::embedder;
use ::hyper::Client as HttpClient;
use anyhow::{anyhow, Context, Result};
use arrow_array::{Array, StringArray};
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
) -> Result<Vec<String>> {
    // colog::init();

    info!("Starting query");

    // let commands = build_args();
    info!("Length of input list: {}", input_list[0].len());
    // check if list is length one String is length one
    if input_list.len() == 1 && input_list[0].is_empty() {
        error!("Query Input is empty");
        return Err(anyhow!("Query Input is empty"));
    }

    let url = EMBEDDING_URL;

    let query_request_arc =
        EmbedRequest::NewArcEmbedRequest(&embed_model, input_list, &"".to_string());
    let query_response =
        embedder::fetch_embedding(url, &query_request_arc, http_client).await.context("Failed to fetch embedding")?;

    let query_vector = query_response.embeddings[0].clone();

    let content = query_table(db, vector_table, query_vector)
        .await
        .context("Failed to query table")?;

    debug!("Finishes running query");

    Ok(content)
}

pub async fn query_table(
    db: &mut Connection,
    table_name: &str,
    query_vector: impl IntoQueryVector,
) -> Result<Vec<String>> {
    let table = db
        .open_table(table_name)
        .execute()
        .await
        .context("Failed to open a table")?;

    // let lower_bound = Some(0.5);
    // let upper_bound = Some(1.5);

    let stream2 = table
        .query()
        .nearest_to(query_vector) // Find the nearest vectors to the query vector
        .context("Failed to select nearest vector")?
        // .distance_range(lower_bound, upper_bound) // bug in DataFusion library
        .distance_type(lancedb::DistanceType::L2)
        .refine_factor(20)
        .nprobes(10)
        .postfilter()
        .only_if("_distance > 0.3 AND _distance < 1")
        .select(lancedb::query::Select::Columns(vec![
            "_distance".to_string(),
            "id".to_string(),
            "metadata".to_string(),
            "content".to_string(),
        ]))
        .execute()
        .await
        .context("Failed to execute query and fetch records")?;

    let stream = table
        .query()
        .only_if("metadata IS NOT NULL")
        .select(lancedb::query::Select::Columns(vec![
            "id".to_string(),
            "metadata".to_string(),
            "content".to_string(),
        ]))
        .limit(1000)
        .execute()
        .await
        .context("Failed to execute query and fetch records")?;

    let batches = stream.collect::<Vec<_>>().await;

    for batch in batches {
        let batch: arrow_array::RecordBatch = batch.context("Failed to get RecordBatch")?;
        // println!("Batch: {:?}", batch);

        let schema = batch.schema(); // Bind schema to a variable
        for i in 0..batch.num_columns() {
            let column = batch.column(i);
            let column_name = schema.field(i).name(); // Now this is safe

            debug!("Column {:?}: {:?}", column_name, column);

            if column_name == "content" {
                let content_array = column.as_any().downcast_ref::<StringArray>().context("Failed to downcast to StringArray")?;
                let content = get_content(content_array).context("Failed to get content from lancedb")?;
                return Ok(content);
            }
        }
    }


    Ok(Vec::new())
}


pub fn get_content(batch: &StringArray) -> Result<Vec<String>> {
    let mut content_vec = Vec::new();
    for i in 0..batch.len() {
        let content = batch.value(i);
        content_vec.push(content.to_string());
    }
    Ok(content_vec)
}