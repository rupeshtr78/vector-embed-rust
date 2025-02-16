use crate::app::commands::Commands;
use crate::app::constants::{VECTOR_DB_HOST, VECTOR_DB_NAME, VECTOR_DB_PORT, VECTOR_DB_USER};
use crate::lancevectordb;
use crate::pgvectordb::run_embedding::run_embedding_load;
use crate::pgvectordb::VectorDbConfig;
use crate::pgvectordb::{pg_vector, query_vector};
use anyhow::Context;
use anyhow::Result;
use hyper::Client as HttpClient;
use log::info;
use postgres::Client;
use tokio::sync::Mutex;

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
            let client: Mutex<Client> = pg_vector::pg_client(&db_config)
                .context("Failed to create a new client")?
                .into();

            // Initialize the http client outside the thread // TODO wrap in Arc<Mutex>
            let http_client = HttpClient::new();

            rt.block_on(run_embedding_load(
                url,
                embed_model,
                &input_list,
                vector_table,
                dimension,
                client,
                &http_client,
            ))
            .context("Failed to run embedding load")?;

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
            // let client: Arc<Mutex<Client>> =
            //     Arc::new(Mutex::new(match pg_vector::pg_client(&db_config) {
            //         Ok(client) => client,
            //         Err(e) => {
            //             error!("Error: {}", e);
            //             return Err(anyhow!("Error: {}", e));
            //         }
            //     }));

            let mut client =
                pg_vector::pg_client(&db_config).context("Failed to create a new client")?;

            // Initialize the http client outside the thread // TODO wrap in Arc<Mutex>
            let http_client = HttpClient::new();

            rt.block_on(query_vector::run_query(
                &rt,
                embed_model,
                &input_list,
                vector_table,
                &mut client,
                &http_client,
            ))
            .context("Failed to run query")?;

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
            rt.block_on(crate::chat::run_chat(
                input_list.first().unwrap(),
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

            rt.block_on(crate::chat::run_chat(&prompt, context, &client))
                .context("Failed to run chat")?;

            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        Commands::Version { version } => {
            info!("Version: {}", version);
        }
    }

    Ok(())
}
