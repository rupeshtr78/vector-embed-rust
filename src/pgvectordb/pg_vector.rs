use crate::embedder::config::EmbedRequest;
use crate::app::constants::QUERY_LIMIT;
use log::{debug, error, info};
use pgvector::Vector;
use postgres::{Client, Config, NoTls};
use std::{error::Error, time::Duration};
use crate::pgvectordb::VectorDbConfig;

/// Create a connection to the Postgres database
/// Argumemts:
/// - db_config: &VectorDbConfig
/// Returns:
/// - Result<Client, Box<dyn Error>>
pub fn pg_client(db_config: &VectorDbConfig) -> Result<Client, Box<dyn Error>> {
    let mut config = Config::new();
    config
        .host(db_config.host.as_str())
        .port(db_config.port)
        .user(db_config.user.as_str())
        .dbname(db_config.dbname.as_str())
        .connect_timeout(Duration::from_secs(db_config.timeout));

    let client = config.connect(NoTls)?;

    Ok(client)
}

/// Create a table in the Postgres database
/// Arguments:
/// - pg_client: &mut Client
/// - table: &str
/// - dimension: i32 (dimension of the vector)
/// Returns:
/// - Result<(), Box<dyn Error>>
pub fn create_table(
    pg_client: &mut Client,
    table: &String,
    dimension: i32,
) -> Result<(), Box<dyn Error>> {
    let mut transaction = pg_client.transaction()?;

    // drop table if exists
    let drop_query = format!("DROP TABLE IF EXISTS {}", table);
    transaction.execute(&drop_query, &[])?;
    debug!("Table dropped: {}", table);

    // create table with id, content and embedding columns
    let query = format!(
        "CREATE TABLE {} (id bigserial PRIMARY KEY, content text, metadata text, embedding vector({}))",
        table, dimension
    );
    transaction.execute(&query, &[])?;

    // create index on embedding column
    let index_query = format!(
        "CREATE INDEX ON {} USING hnsw (embedding vector_cosine_ops)",
        table
    );
    transaction.execute(&index_query, &[])?;

    info!("Vector Table created: {}", table);
    transaction.commit()?;

    return Ok(());
}

///  Load vector data into the Postgres database
/// Arguments:
/// - pg_client: &mut Client
/// - table: &str
/// - input: &EmbedRequest get input data from the request
/// - embeddings: &Vec<Vec<f32>> get embeddings from the response
/// Returns:
/// - Result<(), Box<dyn Error>>
pub fn load_vector_data(
    pg_client: &mut Client,
    table: &str,
    input: &EmbedRequest,
    embeddings: &Vec<Vec<f32>>,
) -> Result<(), Box<dyn Error>> {
    let mut transaction = pg_client.transaction()?;
    let query = format!("INSERT INTO {} (content, embedding) VALUES ($1, $2)", table);

    // convert input to pg vector
    let pgv = embeddings
        .iter()
        .map(|v| Vector::from(v.clone()))
        .collect::<Vec<Vector>>();

    if input.get_input().len() != pgv.len() || input.get_input().len() == 0 || pgv.len() == 0 {
        return Err("Invalid Input and Embeddings length or mismatch".into());
    }

    // iterate over input and embeddings
    for i in 0..input.input.len() {
        let content = &input.input[i];
        let embedding = &pgv[i];
        debug!("Content: {}, Embedding: {:?}", content, embedding);
        transaction.execute(&query, &[&content, &embedding])?;
    }

    info!("Embedding Data inserted to vector db table: {}", table);
    transaction.commit()?;
    Ok(())
}

/// Query the nearest vector in the Postgres database
/// Arguments:
/// - client: &mut Client
/// - table: &str
/// - query_vec: &Vec<Vec<f32>>
/// Returns:
/// - Result<(), Box<dyn Error>>
pub fn query_nearest(
    client: &mut Client,
    table: &String,
    query_vec: &Vec<Vec<f32>>,
) -> Result<Vec<String>, Box<dyn Error>> {
    // convert input to pg vector
    let pgv = query_vec
        .iter()
        .map(|v| Vector::from(v.clone()))
        .collect::<Vec<Vector>>();

    if query_vec.len() != 1 {
        return Err("Failed to fetch query embedding Query vector length should be 1".into());
    }

    let query = format!(
        "SELECT content FROM {} ORDER BY embedding <-> $1 LIMIT {}",
        table, QUERY_LIMIT
    );

    let row = client.query(&query, &[&pgv[0]]);
    if row.is_err() {
        error!("Error: {}", row.err().unwrap());
        return Err("Failed to fetch query embedding".into());
    }

    let mut result: Vec<String> = Vec::new();
    match row {
        Ok(rows) => {
            if rows.len() == 0 {
                info!("No results found");
                return Ok(result);
            }
            for row in rows {
                let text: &str = row.get(0);
                result.push(text.to_string());
                info!("Query Result: {}", text);
            }
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    };

    Ok(result)
}

/// Select embeddings from the Postgres database
#[allow(dead_code)]
pub fn select_embeddings(client: &mut Client, table: &str) -> Result<(), Box<dyn Error>> {
    info!("Select method started");

    let query = format!("SELECT id, content FROM {}", table);
    let rows = client.query(&query, &[]);
    match rows {
        Ok(rows) => {
            for row in rows {
                // let id = row.get(0);
                let text: &str = row.get(1);
                info!("id: {}, content: {}", 1, text);
            }

            info!("Select statement successful");
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    };

    info!("Select method successful");

    Ok(())
}
