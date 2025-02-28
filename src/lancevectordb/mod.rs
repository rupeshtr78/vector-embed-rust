pub mod load_lancedb;
pub mod query;
use crate::docsplitter::code_loader;
use crate::embedder::fetch_embedding;
use ::anyhow::Context;
use ::anyhow::Result;
use hyper::client::HttpConnector;
use hyper::Client;
use load_lancedb::TableSchema;
use ::log::debug;
use ::log::info;
use ::std::path::PathBuf;
use ::std::sync::Arc;

#[deprecated]
#[allow(dead_code)]
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
/// * `embed_url` - The URL of the embedder
/// * `http_client` - The HTTP client
/// # Returns
/// * `Result<()>` - The result of the operation
pub async fn run(
    path: String,
    chunk_size: usize,
    embed_url: &str,
    http_client: &Client<HttpConnector>,
) -> Result<()> {
    // Load the codebase into chunks
    let chunks = code_loader::load_codebase_into_chunks(&path, chunk_size)
        .await
        .context("Failed to split codebase into chunks")?;

    // Extract the embed requests from the chunks
    let embed_requests = chunks
        .iter()
        .map(|chunk: &code_loader::FileChunk| chunk.embed_request_arc())
        .collect::<Vec<_>>();

    // Print the embed requests for debugging
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

    // create table
    let table_name = format!("{}_{}", &file_name, "table");
    let table_schema = TableSchema::new(&table_name);

    load_lancedb::create_lance_table(&mut db, &table_schema)
        .await
        .context("Failed to create table")?;

    // load embeddings
    let table = db
        .open_table(&table_name)
        .execute()
        .await
        .context("Failed to open table")?;
    for (id, embed_request) in embed_requests.iter().enumerate() {
        // fetch embeddings
        let embed_response = fetch_embedding(embed_url, embed_request, http_client)
            .await
            .context("Failed to fetch embeddings")?;
        info!("Embedding Response: {:?}", embed_response.embeddings.len());

        let request = Arc::clone(embed_request);

        // create record batch
        let record_batch =
            load_lancedb::create_record_batch(id as i32, request, embed_response, &table_schema)
                .await
                .context("Failed to create record batch")?;

        // insert records
        load_lancedb::insert_embeddings(&table_schema, record_batch, table.clone())
            .await
            .context("Failed to insert embeddings")?;

        info!("Embeddings inserted successfully");
    }

    // create index
    let embedding_col = table_schema.vector.name();
    load_lancedb::create_index_on_embedding(
        &mut db,
        table_schema.name.as_str(),
        [embedding_col.as_str()].to_vec(),
    )
    .await
    .context("Failed to create index")?;

    Ok(())
}

fn get_file_name(root_dir: &str) -> String {
    let root_path = PathBuf::from(root_dir);

    let file_name = root_path.file_name().map_or_else(
        || "None".to_string(),
        |s| s.to_str().unwrap_or("None").to_string(),
    );
    println!("File Name: {}", file_name);
    file_name
}

pub async fn run_v2(
    path: String,
    chunk_size: usize,
    embed_url: &str,
    http_client: &Client<HttpConnector>,
) -> Result<()> {
    // Load the codebase into chunks
    let chunks = code_loader::load_codebase_into_chunks(&path, chunk_size)
        .await
        .context("Failed to split codebase into chunks")?;

    // Extract embed requests from the chunks
    let embed_requests: Vec<_> = chunks
        .iter()
        .map(|chunk| chunk.embed_request_arc())
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
        let embed_url = embed_url.to_string();
        let http_client = http_client.clone();
        let table_schema = table_schema.clone();
        let table = table.clone();

        // Spawn a task to fetch and insert embeddings in parallel
        let task = tokio::spawn(async move {
            // Fetch embeddings
            let embed_response = fetch_embedding(&embed_url, &embed_request, &http_client)
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

    Ok(())
}
