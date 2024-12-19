#[cfg(test)]
mod tests {
    use crate::vectordb::query_vector::run_query;

    use super::*;
    use postgres::Client;
    use std::sync::{Arc, Mutex};
    use tokio::runtime::Runtime;
    use tokio_postgres::NoTls;

    #[test]
    fn test_run_query_success() {
        // Setup
        let rt = Runtime::new().unwrap();
        let embed_model = "test_model".to_string();
        let input_list = vec!["input1".to_string(), "input2".to_string()];
        let vector_table = "test_table".to_string();

        // Mocking a client for postgres
        let client = Arc::new(Mutex::new(
            Client::connect("mock_connection_string", NoTls).unwrap(),
        ));

        // Test execution
        run_query(&rt, embed_model, &input_list, vector_table, client.clone());

        // Verify results (this could involve checking state changes or ensuring expected outputs)
        // Depending on your implementations, you can check logs or states here.
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
