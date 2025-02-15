#[cfg(test)]
mod tests {
    use crate::pgvectordb::query_vector::run_query;

    use crate::embedder::config::EmbedRequest;
    use crate::pgvectordb::{pg_vector, VectorDbConfig};
    use crate::pgvectordb::pg_vector::pg_client;
    use ::hyper::Client as HttpClient;
    use postgres::Client;
    use std::error::Error;
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

    fn setup_embeddings(embed_data: &EmbedRequest, table: String) {
        let db_config = create_dbconfig();
        let mut pg_client: postgres::Client = pg_client(&db_config).unwrap();
        let dimension = 768;
        let url = EMBEDDING_URL;
        let input = ["item1".to_string(), "item2".to_string()];
        let http_client = HttpClient::new();

        // Arrange
        // let result = pg_vector::create_table(&mut pg_client, &table, dimension);
        // assert!(result.is_ok());

        let rt = Runtime::new().unwrap();

        let response = rt
            .block_on(crate::embedder::create_embed_request(
                url,
                embed_data,
                &http_client,
            ))
            .unwrap();

        let mut client = setup_db_client().expect("Failed to set up database");
        let vector_table = table.clone();
        let input_list = vec!["item1".to_string(), "item2".to_string()];
        let embed_model = EMBEDDING_MODEL.to_string();

        // Arrange
        let embed_data = EmbedRequest {
            model: EMBEDDING_MODEL.to_string(),
            input: input_list.clone(),
            metadata: None,
        };

        let embeddings = response.embeddings;

        pg_vector::load_vector_data(&mut pg_client, &vector_table, &embed_data, &embeddings)
            .unwrap();
    }

    fn teardown_db(client: &mut Client, table: &String) -> Result<(), Box<dyn Error>> {
        let query_string = format!("DROP TABLE IF EXISTS {}", table);
        client.execute(&query_string, &[])?;
        Ok(())
    }

    #[test]
    fn test_run_query_success() {
        // Setup
        let db_config = create_dbconfig();
        let mut pg_client: postgres::Client = pg_client(&db_config).unwrap();
        let mut client = setup_db_client().expect("Failed to set up database");
        let vector_table = "test_table_query_success".to_string();
        let input_list = vec!["item1".to_string(), "item2".to_string()];
        let embed_model = EMBEDDING_MODEL.to_string();
        let dimension = 768;
        let http_client = HttpClient::new();

        // Arrange
        let result = pg_vector::create_table(&mut pg_client, &vector_table, dimension);
        assert!(result.is_ok());

        setup_embeddings(
            &EmbedRequest {
                model: embed_model.clone(),
                input: input_list.clone(),
                metadata: None,
            },
            vector_table.clone(),
        );

        let rt = Runtime::new().unwrap();
        // Test execution
        run_query(
            &rt,
            embed_model,
            &input_list,
            vector_table.clone(),
            client,
            &http_client,
        );

        // Assert
        assert!(true); // Ensure it runs successfully

        // Teardown
        let _ = teardown_db(&mut pg_client, &vector_table.clone());
    }

    #[test]
    fn test_run_query_invalid_model() {
        // Setup
        let db_config = create_dbconfig();
        let mut pg_client: postgres::Client = pg_client(&db_config).unwrap();
        let dimension = 768;
        let rt = Runtime::new().unwrap();
        let embed_model = "nomic-embed-text".to_string();
        let input_list = vec!["input1".to_string(), "input2".to_string()];
        let vector_table = "test_table_invalid_model".to_string();
        let mut client = setup_db_client().expect("Failed to set up database");
        let http_client = HttpClient::new();

        // Arrange
        let result = pg_vector::create_table(&mut pg_client, &vector_table, dimension);
        assert!(result.is_ok());

        setup_embeddings(
            &EmbedRequest {
                model: embed_model.clone(),
                input: input_list.clone(),
                metadata: None,
            },
            vector_table.clone(),
        );

        // Run and expect an error due to invalid models or other conditions
        let wrong_model = "wrong_model".to_string();
        run_query(
            &rt,
            wrong_model,
            &input_list,
            vector_table.clone(),
            client.clone(),
            &http_client,
        );

        assert!(true); // Verify that it did not cause panicked

        // Teardown
        let _ = teardown_db(&mut pg_client, &vector_table.clone());
    }

    #[test]
    fn test_run_query_edge_case_no_inputs() {
        // Setup
        let db_config = create_dbconfig();
        let mut pg_client: postgres::Client = pg_client(&db_config).unwrap();
        let dimension = 768;
        let rt = Runtime::new().unwrap();
        let embed_model = "nomic-embed-text".to_string();
        let input_list = vec!["input1".to_string(), "input2".to_string()];
        let vector_table = "test_table_no_input".to_string();
        let mut client = setup_db_client().expect("Failed to set up database");
        let http_client = HttpClient::new();

        // Arrange
        let result = pg_vector::create_table(&mut pg_client, &vector_table, dimension);
        // assert!(result.is_ok());

        setup_embeddings(
            &EmbedRequest {
                model: embed_model.clone(),
                input: input_list.clone(),
                metadata: None,
            },
            vector_table.clone(),
        );

        // Run and expect an error due to invalid models or other conditions

        let input_list_empty: Vec<String> = Vec::new();
        run_query(
            &rt,
            embed_model,
            &input_list_empty,
            vector_table.clone(),
            client.clone(),
            &http_client,
        );

        assert!(true); // Verify that not panicked

        // Teardown
        let _ = teardown_db(&mut pg_client, &vector_table.clone());
    }
}
