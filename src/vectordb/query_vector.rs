use super::pg_vector;
use crate::app::config::NewArcEmbedRequest;
use crate::app::constants::EMBEDDING_URL;
use crate::embedding;
use ::hyper::Client as HttpClient;
use hyper::client::HttpConnector;
use log::{debug, error, info};
use postgres::Client;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

/// Run the query to get the nearest embeddings
/// Arguments:
/// - rt: &tokio::runtime::Runtime
/// - embed_model: String
/// - input_list: &Vec<String>
/// - vector_table: String
/// - db_config: VectorDbConfig
/// Returns: ()
pub fn run_query(
    rt: &tokio::runtime::Runtime,
    embed_model: String,
    input_list: &Vec<String>,
    vector_table: String,
    client: Arc<Mutex<Client>>,
    http_client: &HttpClient<HttpConnector>,
) {
    // colog::init();

    info!("Starting query");

    // let commands = build_args();
    info!("Length of input list: {}", input_list[0].len());
    // check if list is length one String is length one
    if input_list.len() == 1 && input_list[0].len() == 0 {
        error!("Query Input is empty");
        return;
    }

    let url = EMBEDDING_URL;

    let query_request_arc = NewArcEmbedRequest(&embed_model, &input_list);
    let query_response = rt.block_on(embedding::run_embedding::fetch_embedding(
        &url,
        &query_request_arc,
        http_client,
    ));

    let query_thread = thread::Builder::new().name("query_thread".to_owned());

    let run_query_thread = query_thread.spawn(move || {
        let client_clone = Arc::clone(&client);
        let mut client = match client_clone.lock() {
            Ok(client) => client,
            Err(p) => {
                error!("Error: {:?}", p);
                return;
            }
        };

        // query the embeddings from the vector table TODO - handle the query response
        let query =
            pg_vector::query_nearest(&mut client, &vector_table, &query_response.embeddings);
        match query {
            Ok(_) => {} // TODO - handle the query response
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
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
