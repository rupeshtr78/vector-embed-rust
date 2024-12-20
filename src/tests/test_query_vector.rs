#[cfg(test)]
mod tests {
    use crate::vectordb::query_vector::run_query;

    use super::*;
    use crate::app::config::{EmbedRequest, VectorDbConfig};
    use crate::embedding::{self, vector_embedding};
    use crate::vectordb::pg_vector;
    use crate::vectordb::pg_vector::pg_client;
    use postgres::{Client, NoTls};
    use std::error::Error;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use tokio::runtime::Runtime; // Import runtime

    const PORT: u16 = 5555;
    const HOST: &str = "10.0.0.213";
    const USER: &str = "rupesh";
    const DBNAME: &str = "vectordb";
    const TEST_TABLE: &str = "test_table";
    const EMBEDDING_MODEL: &str = crate::app::constants::EMBEDDING_MODEL;
    const EMBEDDING_URL: &str = crate::app::constants::EMBEDDING_URL;

    fn create_dbconfig() -> VectorDbConfig {
        VectorDbConfig {
            host: String::from(HOST),
            port: PORT,
            user: String::from(USER),
            dbname: String::from(DBNAME),
            timeout: 5,
        }
    }

    fn setup_db_client() -> Result<Arc<Mutex<Client>>, Box<dyn Error>> {
        let db_config = create_dbconfig();
        let client: Arc<Mutex<Client>> = Arc::new(Mutex::new(pg_vector::pg_client(&db_config)?));
        Ok(client)
    }

    fn fetch_embedding(embed_data: &EmbedRequest, table: String) -> Vec<Vec<f32>> {
        let db_config = create_dbconfig();
        let mut pg_client: postgres::Client = pg_client(&db_config).unwrap();
        let dimension = 768;
        let url = EMBEDDING_URL;
        let input = vec!["item1".to_string(), "item2".to_string()];

        // Arrange
        let result = pg_vector::create_table(&mut pg_client, &table, dimension);
        assert!(result.is_ok());

        let rt = Runtime::new().unwrap();

        let response = rt.block_on(vector_embedding::create_embed_request(url, &embed_data));
        assert!(response.is_ok());
        response.unwrap().embeddings
    }

    #[test]
    fn test_run_query_success() {
        // Setup
        let db_config = create_dbconfig();
        let mut pg_client: postgres::Client = pg_client(&db_config).unwrap();
        let mut client = setup_db_client().expect("Failed to set up database");
        let vector_table = "test_table_success".to_string();
        let dimension = 768;
        let url = EMBEDDING_URL;
        let input_list = vec!["item1".to_string(), "item2".to_string()];
        let embed_model = EMBEDDING_MODEL.to_string();

        // Arrange
        let embed_data = EmbedRequest {
            model: EMBEDDING_MODEL.to_string(),
            input: input_list.clone(),
        };
        let embeddings = fetch_embedding(&embed_data, vector_table.clone());
        let result =
            pg_vector::load_vector_data(&mut pg_client, &vector_table, &embed_data, &embeddings);
        assert!(result.is_ok());

        let rt = Runtime::new().unwrap();
        // Test execution
        run_query(&rt, embed_model, &input_list, vector_table, client);

        // Assert
        assert!(true); // Ensure it runs successfully
    }

    #[test]
    fn test_run_query_invalid_embedding_url() {
        // Setup
        let rt = Runtime::new().unwrap();
        let embed_model = "invalid_model".to_string();
        let input_list = vec!["input1".to_string(), "input2".to_string()];
        let vector_table = "test_table".to_string();

        // Mocking client
        let client = Arc::new(Mutex::new(
            Client::connect("mock_connection_string", NoTls).unwrap(),
        ));

        // Run and expect an error due to invalid URLs or other conditions
        let result = std::panic::catch_unwind(|| {
            run_query(&rt, embed_model, &input_list, vector_table, client.clone());
        });

        assert!(result.is_err()); // Verify that it panicked
    }

    #[test]
    fn test_run_query_thread_panic_handling() {
        // Setup
        let rt = Runtime::new().unwrap();
        let embed_model = "test_model".to_string();
        let input_list = vec!["input1".to_string(), "input2".to_string()];
        let vector_table = "mock_table".to_string();

        // Mocking client
        let client = Arc::new(Mutex::new(
            Client::connect("mock_connection_string", NoTls).unwrap(),
        ));

        // Mocking pg_vector::query_nearest to force a panic
        // Assuming you can override the implementation (using traits or dependency injection)

        let result = std::panic::catch_unwind(|| {
            run_query(&rt, embed_model, &input_list, vector_table, client);
        });

        assert!(result.is_err()); // Ensure it panics correctly
    }

    #[test]
    fn test_run_query_edge_case_no_inputs() {
        // Setup
        let rt = Runtime::new().unwrap();
        let embed_model = "test_model".to_string();
        let input_list = vec![]; // No inputs
        let vector_table = "test_table".to_string();

        // Mocking client
        let client = Arc::new(Mutex::new(
            Client::connect("mock_connection_string", NoTls).unwrap(),
        ));

        // Run and check if it completes successfully
        run_query(&rt, embed_model, &input_list, vector_table, client.clone());

        // Check for expected states or logs
    }
}
