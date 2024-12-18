use crate::app::config::{ArcEmbedRequest, EmbedRequest, EmbedResponse, NewVectorDbConfig};
use crate::app::constants::{
    EMBEDDING_MODEL, EMBEDDING_URL, VECTOR_DB_DIM, VECTOR_DB_HOST, VECTOR_DB_NAME, VECTOR_DB_PORT,
    VECTOR_DB_TABLE, VECTOR_DB_USER,
};

use app::commands::{build_args, Args, Commands};
use app::config::NewArcEmbedRequest;
use log::{debug, error, info};
use postgres::Client;
use std::borrow::Borrow;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::thread::{self};

mod app;
mod embedding;
mod vectordb;

fn main() {
    colog::init();
    info!("Starting");

    let commands = build_args();

    let url = EMBEDDING_URL;
    let mut model = EMBEDDING_MODEL.to_string();

    // let input = vec!["hello".to_string()];
    let iput_str = "The dog is barking".to_string();
    let mut input_list = &vec![iput_str];

    if commands.is_write() {
        if let Some(Commands::Write {
            input,
            embed_model,
            table,
            dim,
        }) = commands.write()
        {
            println!("Write command");
            println!("Input: {:?}", input);
            input_list = input;

            model = embed_model.to_string();
            println!("Model: {:?}", model);
            println!("Table: {:?}", table);
            println!("Dimension: {:?}", dim);
            // println!("Dimension: {:?}", dim); // ensure this statement is consistent with your original code requirements
        }
    }

    // Arc (Atomic Reference Counted) pointer. It is a thread-safe reference-counting pointer.
    let embed_request_arc = NewArcEmbedRequest(&model, input_list);

    let embed_request_arc_clone = Arc::clone(&embed_request_arc);

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

    let embed_thread = thread::spawn(move || {
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
            table,
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

    // Wait for the embed thread to finish
    if let Err(e) = embed_thread.join() {
        error!("Error: {:?}", e);
    }

    // query the embeddings in a separate thread
    let query = "who is barking".to_string();
    let query_input = vec![&query];

    let query_request_arc = NewArcEmbedRequest(&model, query_input);
    let query_response = rt.block_on(run_embedding(&url, &query_request_arc));

    let query_thread = thread::spawn(move || {
        let mut client = match vectordb::pg_vector::pg_client(&db_config) {
            Ok(client) => client,
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        };

        // query the embeddings
        let query =
            vectordb::pg_vector::query_nearest(&mut client, &table, &query_response.embeddings);
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

    if let Err(e) = query_thread.join() {
        error!("Error: {:?}", e);
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
    table: &str,
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
