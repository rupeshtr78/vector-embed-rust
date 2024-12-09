use log::{error, info};

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
    }

    let mut client = match vectordb::pg_vector::pg_client().await {
        Ok(client) => client,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };

    vectordb::pg_vector::select_embeddings(&mut client)
        .await
        .unwrap_or_else(|e| error!("Error: {}", e));

    info!("Done");
}
