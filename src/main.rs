use std::thread::{self, Thread};

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
    let data = embedding::vector_embedding::EmbedRequest {
        model: model.to_string(),
        input: vec!["hello".to_string()],
    };

    if let Err(e) = embedding::vector_embedding::hyper_builder_post(url, data).await {
        error!("Error: {}", e);
        return;
    };

    // create new thread

    let embed_thread = thread::spawn(|| {
        let mut config = match vectordb::pg_vector::pg_client() {
            Ok(config) => config,
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        };

        match vectordb::pg_vector::select_embeddings(&mut config) {
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
