use crate::app::config::{ArcEmbedRequest, EmbedRequest, EmbedResponse, NewVectorDbConfig};
use crate::app::constants::{
    EMBEDDING_MODEL, EMBEDDING_URL, VECTOR_DB_DIM, VECTOR_DB_HOST, VECTOR_DB_NAME, VECTOR_DB_PORT,
    VECTOR_DB_TABLE, VECTOR_DB_USER,
};

use app::commands::{build_args, Args, Commands};
use app::config::NewArcEmbedRequest;
use log::LevelFilter;
use log::{debug, error, info};
use postgres::Client;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::thread::{self};

mod app;
mod embedding;
mod vectordb;

fn main() {
    // colog::init();
    colog::basic_builder()
        .filter_level(LevelFilter::Info)
        .init();

    info!("Starting");

    let commands = build_args();

    let url = EMBEDDING_URL;
    let mut embed_model = String::new();
    let mut input_list: &Vec<String> = &Vec::new();
    let mut vector_table = String::new();

    if commands.is_write() {
        if let Some(Commands::Write {
            input,
            model,
            table,
            dim,
        }) = commands.write()
        {
            input_list = input;
            embed_model = model.to_string();
            vector_table = table.to_string();
            info!("Write command");
            info!("Input: {:?}", input);
            debug!("Model: {:?}", model);
            debug!("Table: {:?}", table);
            debug!("Dimension: {:?}", dim);
        }
    }

    // Arc (Atomic Reference Counted) pointer. It is a thread-safe reference-counting pointer.
    let embed_request_arc = NewArcEmbedRequest(&embed_model, input_list);
    let embed_request_arc_clone = Arc::clone(&embed_request_arc);
    let query_table = vector_table.clone();

    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            error!("Failed to build runtime: {}", e);
            return;
        }
    };

    // Run embedding request
    let embed_response = rt.block_on(run_embedding(&url, &embed_request_arc));

    let db_config = NewVectorDbConfig(
        VECTOR_DB_HOST,
        VECTOR_DB_PORT,
        VECTOR_DB_USER,
        VECTOR_DB_NAME,
    );

    let table = VECTOR_DB_TABLE;
    let dim = VECTOR_DB_DIM;

    let db1 = db_config.clone();

    let embed_thread = thread::Builder::new().name("embedding_thread".to_owned());

    let run_embed_thread = embed_thread.spawn(move || {
        let mut client = match vectordb::pg_vector::pg_client(&db1) {
            Ok(client) => client,
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        };
        let embed_data = match embed_request_arc_clone.read() {
            Ok(data) => data,
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        };

        match run_load_data(
            &mut client,
            &vector_table,
            dim,
            &embed_data,
            &embed_response.embeddings,
        ) {
            Ok(_) => {}
            Err(e) => {
                error!("Error: {}", e);
            }
        }

        if let Err(e) = client.close() {
            error!("Error: {}", e);
            return;
        }
    });

    // Wait for the thread to finish and retrieve its result
    match run_embed_thread {
        Ok(handle) => {
            // Join the thread and get the result
            match handle.join() {
                Ok(_) => info!("Finished running embed thread!"),
                Err(_) => error!("Thread panicked!"),
            }
        }
        Err(_) => eprintln!("Failed to spawn thread!"),
    }

    // query the embeddings in a separate thread
    let query = "who is barking".to_string();
    let query_input = vec![query];

    let query_request_arc = NewArcEmbedRequest(&embed_model, &query_input);
    let query_response = rt.block_on(run_embedding(&url, &query_request_arc));

    let query_thread = thread::Builder::new().name("query_thread".to_owned());

    let run_query_thread = query_thread.spawn(move || {
        let mut client = match vectordb::pg_vector::pg_client(&db_config) {
            Ok(client) => client,
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        };

        // query the embeddings
        let query = vectordb::pg_vector::query_nearest(
            &mut client,
            &query_table,
            &query_response.embeddings,
        );
        match query {
            Ok(_) => {}
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        }

        if let Err(e) = client.close() {
            error!("Error: {}", e);
            return;
        }
    });

    match run_query_thread {
        Ok(handle) => match handle.join() {
            Ok(_) => info!("Finished running query thread!"),
            Err(_) => eprintln!("Thread panicked!"),
        },
        Err(_) => error!("Failed to spawn thread!"),
    }

    debug!("Done with main");
}

async fn run_embedding(url: &str, embed_data: &Arc<RwLock<EmbedRequest>>) -> EmbedResponse {
    let embed_data = match embed_data.read() {
        Ok(data) => data,
        Err(e) => {
            error!("Error: {}", e);
            return EmbedResponse {
                model: "".to_string(),
                embeddings: vec![],
            };
        }
    };

    match embedding::vector_embedding::create_embed_request(url, &embed_data).await {
        Ok(embed_response) => embed_response,
        Err(e) => {
            error!("Error: {}", e);
            return EmbedResponse {
                model: "".to_string(),
                embeddings: vec![],
            };
        }
    }
}

fn run_load_data(
    pg_client: &mut Client,
    table: &String,
    dimension: i32,
    embed_request: &EmbedRequest,
    embeddings: &Vec<Vec<f32>>,
) -> Result<(), Box<dyn Error>> {
    match vectordb::pg_vector::create_table(pg_client, &table, dimension) {
        Ok(_) => {
            debug!("Create table successful");
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }

    match vectordb::pg_vector::load_vector_data(pg_client, &table, &embed_request, embeddings) {
        Ok(_) => {
            debug!("Load vector data successful");
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    }

    Ok(())
}
