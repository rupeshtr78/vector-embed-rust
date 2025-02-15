use crate::app::commands::Commands;
use crate::pgvectordb::VectorDbConfig;
use crate::app::constants::{VECTOR_DB_HOST, VECTOR_DB_NAME, VECTOR_DB_PORT, VECTOR_DB_USER};
use crate::pgvectordb::run_embedding::{run_embedding_load};
use crate::lancevectordb;
use crate::pgvectordb::{pg_vector, query_vector};
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use hyper::Client as HttpClient;
use log::{error, info};
use postgres::Client;
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

            let http_client = HttpClient::new();

            rt.block_on(lancevectordb::run(path, chunk_size, url, &http_client))
                .context("Failed to run lancevectordb")?;

            // shutdown the runtime after the embedding is done
            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        Commands::RagQuery {
            input,
            model,
            table,
            database,
        } => {
            let input_list = input;
            let embed_model = model.to_string();
            let vector_table = table.to_string();
            let db_uri = database.to_string();

            info!("Query command is run with below arguments:");
            info!(" Query: {:?}", input_list);
            info!(" Model: {:?}", model);
            info!(" Table: {:?}", table);

            // Initialize the http client outside the thread // TODO wrap in Arc<Mutex>
            let http_client = HttpClient::new();

            // Initialize the database
            let mut db = rt
                .block_on(lancedb::connect(&db_uri).execute())
                .context("Failed to connect to the database")?;

            // Query the database
            let content = rt
                .block_on(lancevectordb::query::run_query(
                    &mut db,
                    embed_model,
                    &input_list,
                    &vector_table,
                    &http_client,
                ))
                .context("Failed to run query")?;

            println!("Query Response: {:?}", content);

            let context = content.join(" ");

            // @TODO: Properly get the prompt from from cli
            rt.block_on(crate::chat::chat_main::run_chat(
                input_list.get(0).unwrap(),
                Some(&context),
                &http_client,
            ))
            .context("Failed to run chat")?;

            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        Commands::Chat { prompt } => {
            info!("Chat command is run with below arguments:");
            info!(" Prompt: {:?}", prompt);

            let context: Option<&str> = None;
            let client = HttpClient::new();

            rt.block_on(crate::chat::chat_main::run_chat(&prompt, context, &client))
                .context("Failed to run chat")?;

            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        Commands::Version { version } => {
            info!("Version: {}", version);
        }
    }

    Ok(())
}


