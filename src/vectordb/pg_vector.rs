use log::{error, info};
use postgres::{config, Client, Config, NoTls};
use std::{error::Error, time::Duration};
use tokio::runtime::Runtime;

// fn main() {
//     // env_logger::builder().filter_level(LevelFilter::Debug).init();
//     colog::init();
//     info!("Starting");

//     let mut client = match pg_client() {
//         Ok(client) => client,
//         Err(e) => {
//             error!("Error: {}", e);
//             return;
//         }
//     };

//     if let Err(e) = select_embeddings(&mut client) {
//         error!("Error: {}", e);
//     }

//     let table = "from_rust";
//     let dim = 768;

//     if let Err(e) = create_table(&mut client, table, dim) {
//         error!("Error: {}", e);
//     }

//     drop(client);

//     info!("Done");
// }

pub fn pg_client() -> Result<Client, Box<dyn Error>> {
    let mut config = Config::new();
    config
        .host("10.0.0.213")
        .port(5555)
        .user("rupesh")
        .dbname("vectordb")
        .connect_timeout(Duration::from_secs(5));

    let client = config.connect(NoTls)?;

    Ok(client)
}

pub fn select_embeddings(client: &mut Client) -> Result<(), Box<dyn Error>> {
    info!("Select method started");

    let query = "SELECT id, text FROM embeddings";
    let rows = client.query(query, &[]);
    match rows {
        Ok(rows) => {
            for row in rows {
                let id: i32 = row.get(0);
                let text: &str = row.get(1);
                info!("id: {}, name: {}", id, text);
            }

            info!("Select successful");
        }
        Err(e) => {
            error!("Error: {}", e);
        }
    };

    info!("Select method successful");

    Ok(())
}

fn create_table(pg_client: &mut Client, table: &str, dimension: i32) -> Result<(), Box<dyn Error>> {
    let mut transaction = pg_client.transaction()?;

    let drop_query = format!("DROP TABLE IF EXISTS {}", table);
    transaction.execute(&drop_query, &[])?;
    info!("Table dropped: {}", table);

    let query = format!(
        "CREATE TABLE {} (id bigserial PRIMARY KEY, content text, embedding vector({}))",
        table, dimension
    );
    transaction.execute(&query, &[])?;

    info!("Table created: {}", table);
    transaction.commit()?;

    return Ok(());
}

// input []string, embeddings [][]float32, conn *pgx.Conn
fn load_vector_data(
    pg_client: &mut Client,
    table: &str,
    input: Vec<String>,
    embeddings: Vec<Vec<f32>>,
) -> Result<(), Box<dyn Error>> {
    let mut transaction = pg_client.transaction()?;
    let query = format!("INSERT INTO {} (content, embedding) VALUES ($1, $2)", table);

    for i in 0..input.len() {
        let content = &input[i];
        let embedding = &embeddings[i];
        transaction.execute(&query, &[&content, &embedding])?;
    }

    info!("Data inserted");
    transaction.commit()?;
    Ok(())
}
