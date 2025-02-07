use crate::app::commands::Commands;
use crate::app::config::EmbedRequest;
use crate::app::config::EmbedResponse;
use crate::app::config::VectorDbConfig;
use crate::app::constants;
use crate::app::constants::{VECTOR_DB_HOST, VECTOR_DB_NAME, VECTOR_DB_PORT, VECTOR_DB_USER};
use crate::docsplitter::code_loader;
use crate::embedder::run_embedding::{fetch_embedding, run_embedding_load};
use crate::lancevectordb;
use crate::lancevectordb::load_lancedb;
use crate::lancevectordb::load_lancedb::TableSchema;
use crate::pgvectordb::{pg_vector, query_vector};
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use hyper::Client as HttpClient;
use log::{debug, error, info, warn};
use postgres::Client;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub fn cli(commands: Commands, rt: tokio::runtime::Runtime, url: &str) -> Result<()> {
    match commands {
        Commands::Write {
            input,
            model,
            table,
            dim,
        } => {
            let input_list = input;
            let embed_model = model.to_string();
            let vector_table = table.to_string();
            let dimension = dim.to_string();
            info!("Using the Write arguments below:");
            info!(" Input Length: {:?}", input_list.len());
            info!(" Model: {:?}", model);
            info!(" Table: {:?}", table);
            info!(" Dimension: {:?}", dim);

            let db_config = VectorDbConfig::NewVectorDbConfig(
                VECTOR_DB_HOST,
                VECTOR_DB_PORT,
                VECTOR_DB_USER,
                VECTOR_DB_NAME,
            );

            // Initialize the client outside the thread and wrap it in Arc<Mutex>
            let client: Arc<Mutex<Client>> =
                Arc::new(Mutex::new(match pg_vector::pg_client(&db_config) {
                    Ok(client) => client,
                    Err(e) => {
                        error!("Error: {}", e);
                        return Err(anyhow!("Error: {}", e));
                    }
                }));

            // Initialize the http client outside the thread // TODO wrap in Arc<Mutex>
            let http_client = HttpClient::new();

            let embed_handler = run_embedding_load(
                &rt,
                url,
                embed_model,
                &input_list,
                vector_table,
                dimension,
                client,
                &http_client,
            );

            match embed_handler {
                Ok(_) => info!("Embedding loaded successfully"),
                Err(e) => error!("Error: {}", e),
            }

            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        Commands::Query {
            input,
            model,
            table,
        } => {
            let input_list = input;
            let embed_model = model.to_string();
            let vector_table = table.to_string();

            info!("Query command is run with below arguments:");
            info!(" Query: {:?}", input_list);
            info!(" Model: {:?}", model);
            info!(" Table: {:?}", table);

            let db_config = VectorDbConfig::NewVectorDbConfig(
                VECTOR_DB_HOST,
                VECTOR_DB_PORT,
                VECTOR_DB_USER,
                VECTOR_DB_NAME,
            );

            // Initialize the client outside the thread and wrap it in Arc<Mutex>
            let client: Arc<Mutex<Client>> =
                Arc::new(Mutex::new(match pg_vector::pg_client(&db_config) {
                    Ok(client) => client,
                    Err(e) => {
                        error!("Error: {}", e);
                        return Err(anyhow!("Error: {}", e));
                    }
                }));

            // Initialize the http client outside the thread // TODO wrap in Arc<Mutex>
            let http_client = HttpClient::new();

            query_vector::run_query(
                &rt,
                embed_model,
                &input_list,
                vector_table,
                client,
                &http_client,
            );

            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        Commands::Load { path, chunk_size } => {
            info!("Using the Load arguments below:");
            info!(" Path: {:?}", path);
            info!(" Chunk Size: {:?}", chunk_size);

            // Load the codebase into chunks
            let chunks = rt.block_on(code_loader::load_codebase_into_chunks(&path, chunk_size));
            let chunks = match chunks {
                Ok(chunks) => chunks,
                Err(e) => {
                    error!("Error: {}", e);
                    return Err(anyhow!("Error: {}", e));
                }
            };

            // Extract the embed requests from the chunks
            let embed_requests: Vec<Arc<std::sync::RwLock<EmbedRequest>>> = chunks
                .iter()
                .map(|chunk: &code_loader::FileChunk| chunk.embed_request_arc())
                .collect::<Vec<_>>();

            // Print the embed requests
            for embed_request in &embed_requests {
                let embed_request = embed_request.read().unwrap();
                debug!("Embed Request Metadata: {:?}", embed_request.metadata);
            }

            // Initialize the http client outside the thread // TODO wrap in Arc<Mutex>
            let http_client = HttpClient::new();

            // @TODO: Properly initialize the db connection with anyhow
            let db_uri = get_file_name(&path) + "_db/";
            let mut db = rt
                .block_on(lancedb::connect(&db_uri).execute())
                .context("Failed to connect to the database")?;

            // create table
            let table_name = get_file_name(&path) + "_table";
            let table_schema = TableSchema::new(table_name);

            rt.block_on(load_lancedb::create_lance_table(&mut db, &table_schema))
                .context("Failed to create table")?;

            for embed_request in embed_requests {
                let embed_response: EmbedResponse =
                    rt.block_on(fetch_embedding(&url, &embed_request, &http_client));
                info!("Embedding Response: {:?}", embed_response.embeddings.len());

                let request: Arc<std::sync::RwLock<EmbedRequest>> = Arc::clone(&embed_request);
                // create record batch
                let record_batch =
                    load_lancedb::create_record_batch(request, embed_response, &table_schema)
                        .context("Failed to create record batch")?;

                // insert records
                rt.block_on(load_lancedb::insert_embeddings(
                    &mut db,
                    &table_schema,
                    record_batch,
                ))
                .context("Failed to insert embeddings")?;
            }

            // query the table
            let input_list = vec!["what is mirostat".to_string()];
            let embed_model = constants::EMBEDDING_MODEL.to_string();
            let vector_table = "codebase_table".to_string();
            rt.block_on(lancevectordb::query::run_query(
                &mut db,
                embed_model,
                &input_list,
                vector_table,
                &http_client,
            ));

            // shutdown the runtime after the embedding is done
            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        _ => {
            warn!("No embedding command provided");
        }
    }

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
