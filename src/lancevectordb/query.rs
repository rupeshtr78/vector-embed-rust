use crate::app::constants::CHAT_API_URL;
use crate::embedder;
use crate::embedder::config::EmbedRequest;
use anyhow::{anyhow, Context, Result};
use arrow_array::{Array, StringArray};
use arrow_array::{Int32Array, RecordBatch, StringArrayType};
use arrow_schema::DataType::{Int32, Utf8};
use arrow_schema::SchemaRef;
use futures::StreamExt;
use hyper::client::HttpConnector;
use ::hyper::Client as HttpClient;
use lancedb::arrow::SendableRecordBatchStream;
use lancedb::query::ExecutableQuery;
use lancedb::query::IntoQueryVector;
use lancedb::query::QueryBase;
use lancedb::{Connection, Table};
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
    whole_query: bool,
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

    let url = format!("{}/{}", CHAT_API_URL, "api/embed");

    // create embedder request for query
    let query_request_arc =
        EmbedRequest::NewArcEmbedRequest(&embed_model, input_list, &"".to_string(), None);
    let query_response = embedder::fetch_embedding(&url, &query_request_arc, http_client)
        .await
        .context("Failed to fetch embedding")?;

    let query_vector = query_response.embeddings[0].clone();

    // query the vector table
    let content = query_vector_table(db, vector_table, query_vector, whole_query)
        .await
        .context("Failed to query table")?;

    debug!("Finishes running query");

    Ok(content)
}

pub async fn query_vector_table(
    db: &mut Connection,
    table_name: &str,
    query_vector: impl IntoQueryVector,
    whole_query: bool,
) -> Result<Vec<String>> {
    let table = db
        .open_table(table_name)
        .execute()
        .await
        .context("Failed to open a table")?;

    // let lower_bound = Some(0.5);
    // let upper_bound = Some(1.5);

    let batches;

    if !whole_query {
        let stream = query_nearest_vector(query_vector, &table).await?;
        batches = stream.collect::<Vec<_>>().await;
    } else {
        let stream = query_all_content(table).await?;
        batches = stream.collect::<Vec<_>>().await;
    }

    debug!("Number of batches retrieved from query: {}", &batches.len());
    let content = get_data_from_batch(&batches, "content")
        .context("Failed to get content from record batch")?;
    //
    let _ = get_data_from_batch(&batches, "chunk_number")
        .context("Failed to get chunk number from record batch")?;

    Ok(content)
}

fn get_data_from_batch(
    batches: &Vec<lancedb::error::Result<RecordBatch>>,
    table_column: &str,
) -> Result<Vec<String>> {
    for batch in batches {
        // to avoid moving the elements and instead borrow them,
        // iterate over references to the elements:
        let batch_ref = batch
            .as_ref()
            .map_err(|e| anyhow!(format!("Failed to get RecordBatch: {}", e)))?;
        let schema = batch_ref.schema(); // Bind schema to a variable

        let content = get_column_data_from_batch(table_column, batch_ref, schema);
        if content.is_ok() {
            return content;
        }
    }

    Ok(Vec::new())
}

fn get_column_data_from_batch(
    table_column: &str,
    batch_ref: &RecordBatch,
    schema: SchemaRef,
) -> Result<Vec<String>> {
    for i in 0..batch_ref.num_columns() {
        let column = batch_ref.column(i);
        let column_name = schema.field(i).name();
        let data_type = schema.field(i).data_type();

        if column_name == table_column {
            let content = match data_type {
                Utf8 => {
                    let content_array = column
                        .as_any()
                        .downcast_ref::<StringArray>()
                        .context("Failed to downcast to StringArray")?;

                    let content = content_array
                        .iter()
                        .map(|opt| opt.map_or_else(|| "NULL".to_string(), |s| s.to_string()))
                        .collect::<Vec<String>>();
                    debug!("Fetched Content: {:?} {:?}", column_name, content);
                    content
                }
                Int32 => {
                    let content_array = column
                        .as_any()
                        .downcast_ref::<Int32Array>()
                        .context("Failed to downcast to Int32Array")?;

                    let content = content_array
                        .iter()
                        .map(|opt| opt.map_or_else(|| "NULL".to_string(), |s| s.to_string()))
                        .collect::<Vec<_>>();

                    debug!("Fetched Content: {:?} {:?}", column_name, content);
                    content
                }
                _ => {
                    return Err(anyhow!("Unsupported column type: {:?}", data_type));
                }
            };
            return Ok(content);
        }
    }

    error!("No content found in the batch");
    Ok(Vec::new())
}

async fn query_all_content(table: Table) -> Result<SendableRecordBatchStream> {
    let stream = table
        .query()
        .only_if("content IS NOT NULL".to_string())
        .select(lancedb::query::Select::Columns(vec![
            "id".to_string(),
            "metadata".to_string(),
            "content".to_string(),
        ]))
        .limit(1000)
        .execute()
        .await
        .context("Failed to execute whole query and fetch records")?;
    Ok(stream)
}

async fn query_nearest_vector(
    query_vector: impl IntoQueryVector + Sized,
    table: &Table,
) -> Result<SendableRecordBatchStream> {
    let stream: SendableRecordBatchStream = table
        .query()
        .nearest_to(query_vector) // Find the nearest vectors to the query vector
        .context("Failed to select nearest vector")?
        // .distance_range(lower_bound, upper_bound) // bug in DataFusion library
        .distance_type(lancedb::DistanceType::Cosine)
        .refine_factor(10)
        .limit(20)
        .nprobes(40) // default is 20
        .postfilter()
        // .only_if("_distance > 0.3 AND _distance < 1")
        .select(lancedb::query::Select::Columns(vec![
            "_distance".to_string(),
            "chunk_number".to_string(),
            "metadata".to_string(),
            "content".to_string(),
        ]))
        .execute()
        .await
        .context("Failed to execute query and fetch records")?;
    Ok(stream)
}
