pub mod load_lancedb;
pub mod query;
use crate::docsplitter::code_loader;
use crate::docsplitter::code_loader::chunk_embed_request_arc;
use crate::embedder::fetch_embedding;
use ::anyhow::Context;
use ::anyhow::Result;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client as LegacyClient;
use load_lancedb::TableSchema;
// use hyper::client::HttpConnector;
// use hyper::Client;
use ::log::debug;
use ::log::info;
use ::std::path::PathBuf;
pub type HttpsClient = LegacyClient<HttpsConnector<HttpConnector>, Full<Bytes>>;

fn get_file_name(root_dir: &str) -> String {
    let root_path = PathBuf::from(root_dir);

    let file_name = root_path.file_name().map_or_else(
        || "None".to_string(),
        |s| s.to_str().unwrap_or("None").to_string(),
    );
    debug!("File Name: {}", file_name);
    file_name
}

/// Run the LanceVectorDB pipeline
/// 1. Load the codebase into chunks
/// 2. Extract the embed requests from the chunks
/// 3. Initialize the database
/// 4. Create a table
/// 5. Load embeddings
/// 6. Create an index
/// # Arguments
/// * `path` - The path to the codebase
/// * `chunk_size` - The size of the chunks
/// * `embed_url` - The URL of the embedding API
/// * `http_client` - The HTTP client
/// # Returns
/// * `Result<()>` - The result of the operation
pub async fn run_embedding_pipeline(
    path: &String,
    chunk_size: usize,
    provider: &str,
    embed_url: &str,
    api_key: &str,
    model: &str,
    https_client: &HttpsClient,
) -> Result<()> {
    // Load the codebase into chunks
    let chunks = code_loader::load_codebase_into_chunks(&path, chunk_size)
        .await
        .context("Failed to split codebase into chunks")?;

    // Extract embed requests from the chunks
    let embed_requests: Vec<_> = chunks
        .iter()
        .map(|chunk| chunk_embed_request_arc(chunk, provider, embed_url, api_key, model))
        .collect();

    // Log embed requests for debugging
    for embed_request in &embed_requests {
        let embed_request = embed_request.read().await;
        debug!("Embed Request Metadata: {:?}", embed_request.metadata);
    }

    // Initialize the database
    let file_name = get_file_name(&path);
    let db_uri = format!("{}_{}", &file_name, "db");
    let mut db = lancedb::connect(&db_uri)
        .execute()
        .await
        .context("Failed to connect to the database")?;

    // Create table
    let table_name = format!("{}_{}", &file_name, "table");
    let table_schema = TableSchema::new(&table_name);

    load_lancedb::create_lance_table(&mut db, &table_schema)
        .await
        .context("Failed to create table")?;

    // Load embeddings in parallel to improve performance
    let mut tasks = Vec::new();
    let table = db
        .open_table(&table_name)
        .execute()
        .await
        .context("Failed to open table")?;
    for (id, embed_request) in embed_requests.into_iter().enumerate() {
        // let embed_url = embed_url.to_string();
        let https_client = https_client.clone();
        let table_schema = table_schema.clone();
        let table = table.clone();

        // Spawn a task to fetch and insert embeddings in parallel
        let task = tokio::spawn(async move {
            // Fetch embeddings
            let embed_response = fetch_embedding(&embed_request, &https_client)
                .await
                .context("Failed to fetch embeddings")?;
            info!("Embedding Response: {:?}", embed_response.embeddings.len());

            // Create record batch
            let record_batch = load_lancedb::create_record_batch(
                id as i32,
                embed_request,
                embed_response,
                &table_schema,
            )
            .await
            .context("Failed to create record batch")?;

            // Insert embeddings into the table
            load_lancedb::insert_embeddings(&table_schema, record_batch, table)
                .await
                .context("Failed to insert embeddings")?;

            info!("Embeddings inserted successfully");
            Ok::<(), anyhow::Error>(())
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    for task in tasks {
        // task.await??;
        task.await
            .context("Failed to run task")?
            .context("Insert Task failed")?;
    }

    // Create an index on the embedding column
    let embedding_col = table_schema.vector.name();
    load_lancedb::create_index_on_embedding(
        &mut db,
        table_schema.name.as_str(),
        vec![embedding_col.as_str()],
    )
    .await
    .context("Failed to create index")?;

    // Create an inverted index on the metadata column
    let metadata_col = table_schema.metadata.name();
    // let content_col = table_schema.content.name();
    // create inverted index
    load_lancedb::create_inverted_index(
        &mut db,
        table_schema.name.as_str(),
        vec![metadata_col.as_str()],
    )
    .await
    .context("Failed to create inverted index")?;

    // Create a text index on the content column
    let content_col = table_schema.content.name();
    // create inverted index
    load_lancedb::create_inverted_index(
        &mut db,
        table_schema.name.as_str(),
        vec![content_col.as_str()],
    )
    .await
    .context("Failed to create inverted index")?;

    Ok(())
}
