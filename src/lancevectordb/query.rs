use crate::embedder;
use crate::embedder::config::EmbedRequest;
// use hyper::client::HttpConnector;
// use ::hyper::Client as HttpClient;
use crate::lancevectordb::HttpsClient;
use anyhow::{anyhow, Context, Result};
use arrow_array::{Array, StringArray};
use arrow_array::{Int32Array, RecordBatch};
use arrow_schema::DataType::{Int32, Utf8};
use arrow_schema::SchemaRef;
use futures::StreamExt;
use lancedb::arrow::SendableRecordBatchStream;
use lancedb::query::ExecutableQuery;
use lancedb::query::IntoQueryVector;
use lancedb::query::QueryBase;
use lancedb::{Connection, Table};
use log::{debug, error};

/// Run the query to get the nearest embeddings
/// Arguments:
/// - rt: &tokio::runtime::Runtime
/// - embed_model: String
/// - input_list: &Vec<String>
/// - vector_table: String
/// - db_config: VectorDbConfig
/// - http_client: &HttpClient<HttpConnector>
/// - whole_query: bool
/// Returns:
/// - Result<Vec<String>>
pub async fn run_query(
    db: &mut Connection,
    provider: &str,
    api_url: &str,
    api_key: &str,
    embed_model: &str,
    input_list: &Vec<String>,
    vector_table: &str,
    http_client: &HttpsClient,
    whole_query: bool,
    file_context: bool,
) -> Result<Vec<String>> {
    // colog::init();

    debug!("Starting query");

    // let commands = build_args();
    debug!("Length of input list: {}", input_list[0].len());
    // check if list is length one String is length one
    if input_list.len() == 1 && input_list[0].is_empty() {
        error!("Query Input is empty");
        return Err(anyhow!("Query Input is empty"));
    }

    // let url = format!("{}/{}", CHAT_API_URL, "api/embed");

    // create embedder request for query
    let query_request_arc = EmbedRequest::NewArcEmbedRequest(
        provider,
        api_url,
        api_key,
        embed_model,
        input_list,
        &"".to_string(),
        None,
    );

    let embed_url = query_request_arc.read().await.get_embed_url();

    let query_response = embedder::fetch_embedding(&query_request_arc, http_client)
        .await
        .with_context(|| format!("Failed to fetch embedding response from {}", &embed_url))?;

    let query_vector = query_response.embeddings[0].clone();

    // query the vector table
    let content = query_vector_table(db, vector_table, query_vector, whole_query, file_context)
        .await
        .context("Failed to query table")?;

    debug!("Finishes running query");

    Ok(content)
}

/// Queries a vector table in the database, either fetching all content or querying the nearest vectors.
///
/// # Arguments
/// * `db` - A mutable reference to the database connection.
/// * `table_name` - The name of the table to query.
/// * `query_vector` - The vector to query against the table.
/// * `whole_query` - If true, fetches all content from the table. If false, queries the nearest vectors.
/// * `file_context` - If true, fetches the entire file context for the nearest vectors.
///
/// # Returns
/// A `Result` containing a vector of strings representing the queried content, or an error if the operation fails.
pub async fn query_vector_table(
    db: &mut Connection,
    table_name: &str,
    query_vector: impl IntoQueryVector,
    whole_query: bool,
    file_context: bool,
) -> Result<Vec<String>> {
    let table = db
        .open_table(table_name)
        .execute()
        .await
        .context("Failed to open a table")?;

    let batches;

    if whole_query {
        let stream = query_all_content(&table).await?;
        batches = stream.collect::<Vec<_>>().await;
        let content = get_content_from_stream(&batches, "content")
            .context("Failed to get content from record batch")?;

        Ok(content)
    } else {
        let stream = query_nearest_vector(query_vector, &table).await?;
        batches = stream.collect::<Vec<_>>().await;
        match file_context {
            true => {
                // Fetch the whole file context
                let files = get_content_from_stream(&batches, "metadata")
                    .context("Failed to get file names from metadata")?;

                // files remove duplicates
                let files_unique: Vec<String> = files
                    .into_iter()
                    .filter(|x| x != "NULL")
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();

                debug!("Unique file names after deduplication: {:?}", &files_unique);

                // query the content based on file names
                let file_content = query_content_based_on_metadata(&table, files_unique)
                    .await
                    .context("Failed to query content based on file name metadata")?;

                // convert the stream to a vector
                let file_batch = file_content.collect::<Vec<_>>().await;
                // debug!("File Batch: {:?}", &file_batch); // add if required
                // get the content from the record batch
                let file_data = get_content_from_stream(&file_batch, "content")
                    .context("Failed to get content from record batch")?;
                // debug!("Chunk Data: {:?}", &file_data); // add if required
                Ok(file_data)
            }
            false => {
                debug!("Number of batches retrieved from query: {}", &batches.len());
                let content = get_content_from_stream(&batches, "content")
                    .context("Failed to get content from record batch")?;

                Ok(content)
            }
        }
    }
}

/// Get content from the record stream based on the column name for example "metadata" has the file names
/// Arguments:
/// - batches: &Vec<lancedb::error::Result<RecordBatch>>
/// - table_column: &str
/// Returns:
/// - Result<Vec<String>>
pub fn get_content_from_stream(
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

/// Helper function to Get list of metadata from the record batch based on the column name returns a list of chunks or file names
/// Arguments:
/// - table_column: &str
/// - batch_ref: &RecordBatch
/// - schema: SchemaRef
/// Returns:
/// - Result<Vec<String>>
pub fn get_column_data_from_batch(
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

/// Queries all content from the table, selecting the id, metadata, and content columns.
/// Returns a stream of record batches containing the queried data.
async fn query_all_content(table: &Table) -> Result<SendableRecordBatchStream> {
    let stream = table
        .query()
        .only_if("content IS NOT NULL")
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

/// Queries the nearest vector to the given query vector.
/// Returns a stream of record batches containing the queried data.
/// Arguments:
/// - query_vector: impl IntoQueryVector + Sized
/// - table: &Table
/// Returns:
/// - Result<SendableRecordBatchStream>
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
        .limit(30)
        .nprobes(40) // default is 20
        .postfilter()
        // .only_if("_distance > 0.3 AND _distance < 1")
        .select(lancedb::query::Select::Columns(vec![
            "_distance".to_string(),
            "chunk_number".to_string(),
            "metadata".to_string(),
            "content".to_string(),
        ]))
        .only_if("content IS NOT NULL")
        .execute()
        .await
        .context("Failed to execute query and fetch records")?;
    Ok(stream)
}

#[allow(dead_code)]
async fn query_content_based_on_chunks(
    table: &Table,
    chunks: Vec<String>,
) -> Result<SendableRecordBatchStream> {
    // chunk_number in  ["5", "3", "8", "14", "1", "10", "7", "6", "4"]

    let stream = table
        .query()
        .only_if(format!(
            "chunk_number in ({})",
            chunks
                .iter()
                .map(|s| s.parse::<i32>().unwrap_or(0).to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))
        .select(lancedb::query::Select::Columns(vec![
            "id".to_string(),
            "metadata".to_string(),
            "content".to_string(),
        ]))
        .limit(1000)
        .execute()
        .await
        .context("Failed to execute chunk based query and fetch records")?;
    Ok(stream)
}

/// Query content based on metadata selects all the records with the given metadata
/// Arguments:
/// - table: &Table
/// - metadata: Vec<String>
/// Returns:
/// - Result<SendableRecordBatchStream>
#[allow(dead_code)]
pub async fn query_content_based_on_metadata(
    table: &Table,
    metadata: Vec<String>,
) -> Result<SendableRecordBatchStream> {
    // metadata in  ["mod.rs", "cli.rs", "commands.rs", "constants.rs", "main.rs", "lib.rs", "chat_config.rs"]

    let stream = table
        .query()
        .only_if(format!(
            "metadata IN ({})",
            metadata
                .iter()
                .map(|m| format!("'{}'", m))
                .collect::<Vec<_>>()
                .join(", ")
        ))
        .select(lancedb::query::Select::Columns(vec![
            "id".to_string(),
            "metadata".to_string(),
            "content".to_string(),
        ]))
        .limit(1000)
        .execute()
        .await
        .context("Failed to execute chunk based query and fetch records")?;
    Ok(stream)
}
