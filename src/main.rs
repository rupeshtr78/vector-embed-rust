use std::thread::{self, Thread};

use embedding::vector_embedding::EmbedResponse;
use log::{error, info};
use tokio::runtime::Runtime;
use tokio::task;

mod embedding;
mod vectordb;

#[tokio::main]
async fn main() {
    colog::init();
    info!("Starting");

    let url = "http://0.0.0.0:11434/api/embed";
    let model = "nomic-embed-text";
    let input = vec!["hello".to_string()];
    let data = embedding::vector_embedding::EmbedRequest {
        model: model.to_string(),
        input: input,
    };

    let input_clone = data.input.clone();

    let embedding = task::spawn(async {
        match embedding::vector_embedding::create_embed_request(url, data).await {
            Ok(response) => response,
            Err(e) => {
                error!("Error: {}", e);
                return EmbedResponse {
                    model: "".to_string(),
                    embeddings: vec![],
                };
            }
        }
    });

    let response = embedding.await.unwrap_or_else(|e| {
        error!("Error: {:?}", e);
        EmbedResponse {
            model: "".to_string(),
            embeddings: vec![],
        }
    });

    // create new thread

    let embed_thread = thread::spawn(|| {
        let mut config = match vectordb::pg_vector::pg_client() {
            Ok(config) => config,
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        };

        let table = "from_rust";
        let dim = 768;
        match vectordb::pg_vector::create_table(&mut config, table, dim) {
            Ok(_) => {
                info!("Create table successful");
            }
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        }

        match vectordb::pg_vector::load_vector_data(
            &mut config,
            table,
            input_clone,
            response.embeddings,
        ) {
            Ok(_) => {
                info!("Load vector data successful");
            }
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        }

        match vectordb::pg_vector::select_embeddings(&mut config, &table) {
            Ok(_) => {
                info!("Select main successful");
            }
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        };
    });

    if let Err(e) = embed_thread.join() {
        error!("Error: {:?}", e);
        return;
    }

    info!("Done");
}
