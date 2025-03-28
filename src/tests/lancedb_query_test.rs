#[allow(unused_imports)]
#[allow(unused)]
mod test_lancedb_query {
    use anyhow::Context;
    use anyhow::Result;
    use lancedb::connection::Connection;
    use lancedb::{table, Table};

    use crate::app::cli;
    use crate::app::constants::CHAT_API_KEY;
    use crate::app::constants::CHAT_API_URL;
    use crate::app::constants::EMBEDDING_MODEL;
    use crate::app::constants::OLLAMA_CHAT_API;
    use crate::app::constants::VECTOR_DB_DIM_SIZE;
    use crate::embedder;
    use crate::embedder::config::{EmbedRequest, EmbedResponse};
    use crate::lancevectordb;
    use crate::lancevectordb::load_lancedb::{
        create_index_on_embedding, create_inverted_index, create_lance_table, create_record_batch,
        insert_embeddings, TableSchema,
    };
    use crate::lancevectordb::query;
    use crate::lancevectordb::query::get_content_from_stream;
    use crate::tests;
    use futures::StreamExt;
    use lancedb::query::IntoQueryVector;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Mock constants for testing
    const TEST_DB_URI: &str = "test_db";
    const TEST_FILES_DIR: &str = "src/tests/resources";

    // create a table
    async fn a_create_test_table_data(path: &str) -> Result<()> {
        let https_client = cli::get_https_client().context("Failed to create HTTPS client")?;

        lancevectordb::run_embedding_pipeline(
            &path.to_string(),
            100,
            "ollama",
            CHAT_API_URL,
            CHAT_API_KEY,
            EMBEDDING_MODEL,
            &https_client,
        )
        .await
        .context("Failed to run embedding pipeline")?;

        Ok(())
    }

    // Helper function to create a test table schema
    fn create_test_table_schema(table_name: &str) -> TableSchema {
        TableSchema::new(&table_name.to_string())
    }

    async fn get_query_vector(input_list: &[String]) -> Result<Vec<f32>> {
        let https_client = cli::get_https_client().context("Failed to create HTTPS client")?;
        // create embedder request for query
        let query_request_arc = EmbedRequest::NewArcEmbedRequest(
            "ollama",
            CHAT_API_URL,
            CHAT_API_KEY,
            EMBEDDING_MODEL,
            input_list,
            &"".to_string(),
            None,
        );

        let embed_url = query_request_arc.read().await.get_embed_url();

        let query_response = embedder::fetch_embedding(&query_request_arc, &https_client)
            .await
            .with_context(|| format!("Failed to fetch embedding response from {}", &embed_url))?;

        let query_vector = query_response.embeddings[0].clone();

        Ok(query_vector)
    }

    async fn create_test_connection(path: &str) -> Result<Connection> {
        // delete the test_db if it exists

        let db = lancedb::connect(path)
            .execute()
            .await
            .context("Failed to create test connection")?;

        Ok(db)
    }

    async fn create_test_embedding_table(db_uri: &str, table_name: &str) -> Result<()> {
        let mut db = create_test_connection(db_uri).await?;
        // let table_name = "TEST_TABLE_NAME_INDEX";
        let table_schema = create_test_table_schema(&table_name);

        // Create table first
        create_lance_table(&mut db, &table_schema).await?;

        // Create enough vectors to satisfy k-means clustering
        // We'll create 100 vectors for the 100 centroids
        let num_vectors = 120;

        let mut input_texts = Vec::with_capacity(num_vectors);
        let mut embeddings = Vec::with_capacity(num_vectors);

        for i in 0..num_vectors {
            input_texts.push(format!("test content {}", i));

            // Create a unique vector for each input
            let vec_data: Vec<f32> = (0..VECTOR_DB_DIM_SIZE as usize)
                .map(|j| (i as f32 * 0.01) + (j as f32 * 0.001))
                .collect();

            embeddings.push(vec_data);
        }

        let request = Arc::new(RwLock::new(EmbedRequest {
            provider: "test-provider".to_string(),
            api_url: "http://localhost:8000".to_string(),
            api_key: "test-key".to_string(),
            input: input_texts,
            model: "test-model".to_string(),
            metadata: Some("test-dir".to_string()),
            chunk_number: Some(0),
        }));

        let response = EmbedResponse {
            model: "test-model".to_string(),
            embeddings,
        };

        let record_batch = create_record_batch(100, request, response, &table_schema).await?;
        let table = db.open_table(table_name).execute().await?;
        insert_embeddings(&table_schema, record_batch.clone(), table).await?;

        // Test index creation with sufficient vector data
        create_index_on_embedding(
            &mut db,
            table_name,
            vec![table_schema.vector.name().as_str()],
        )
        .await?;

        let metadata_col = table_schema.metadata.name();
        create_inverted_index(&mut db, table_name, vec![metadata_col.as_str()]).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_get_column_data_from_batch() {
        let table_name = "TEST_TABLE_NAME_RECORD_BATCH";
        let table_schema = create_test_table_schema(&table_name);
        let request = Arc::new(RwLock::new(EmbedRequest {
            provider: "test-provider".to_string(),
            api_url: "http://localhost:8000".to_string(),
            api_key: "test-key".to_string(),
            input: vec!["test content".to_string()],
            model: "test-model".to_string(),
            metadata: Some("test-dir".to_string()),
            chunk_number: Some(0),
        }));

        let response = EmbedResponse {
            model: "test-model".to_string(),
            embeddings: vec![vec![1.0; VECTOR_DB_DIM_SIZE as usize]],
        };

        let record_batch = create_record_batch(1, request, response, &table_schema)
            .await
            .unwrap();

        assert_eq!(record_batch.num_rows(), 1);
        assert_eq!(record_batch.num_columns(), 7);

        let column_name = "metadata";
        let column_data =
            query::get_column_data_from_batch(column_name, &record_batch, record_batch.schema())
                .expect("Failed to get column data");

        assert_eq!(column_data.len(), 1);
        assert_eq!(column_data[0], "test-dir");
    }

    #[tokio::test]
    async fn query_content_based_on_metadata_test() {
        // table: &Table,
        // metadata: Vec<String>,

        let table_name = "test_table";
        create_test_embedding_table(TEST_DB_URI, table_name)
            .await
            .expect("Failed to create test embedding table");

        let table = create_test_connection(TEST_DB_URI)
            .await
            .expect("Failed to create test connection")
            .open_table(table_name)
            .execute()
            .await
            .expect("Failed to open table");

        let metadata = vec!["test-dir".to_string()];
        // test querying content based on metadata
        let file_content = query::query_content_based_on_metadata(&table, metadata)
            .await
            .expect("Failed to query content based on metadata");

        let file_batch = file_content.collect::<Vec<_>>().await;

        assert_eq!(file_batch.len(), 1);
        for batch in file_batch {
            let batch = batch.expect("Failed to get batch");
            assert_eq!(batch.num_rows(), 120);
            assert_eq!(batch.num_columns(), 3); // "id metadata content"
            let id_col = batch.column(0);
            let metadata_col = batch.column(1);
            let content_col = batch.column(2);
            assert_eq!(id_col.data_type().to_string(), "Int32");
            assert_eq!(metadata_col.data_type().to_string(), "Utf8");
            assert_eq!(content_col.data_type().to_string(), "Utf8");
        }
    }

    #[tokio::test]
    async fn get_content_from_stream_test() {
        // table: &Table,
        // metadata: Vec<String>,

        let table_name = "test_table";
        create_test_embedding_table(TEST_DB_URI, table_name)
            .await
            .expect("Failed to create test embedding table");

        let table = create_test_connection(TEST_DB_URI)
            .await
            .expect("Failed to create test connection")
            .open_table(table_name)
            .execute()
            .await
            .expect("Failed to open table");

        let metadata = vec!["test-dir".to_string()];
        // test querying content based on metadata
        let file_content = query::query_content_based_on_metadata(&table, metadata)
            .await
            .expect("Failed to query content based on metadata");

        let file_batch = file_content.collect::<Vec<_>>().await;

        let file_data = get_content_from_stream(&file_batch, "content")
            .context("Failed to get content from record batch")
            .expect("Failed to get content from record batch");

        assert_eq!(file_data.len(), 120);
    }

    #[tokio::test]
    async fn query_vector_table_test() {
        // table: &Table,
        // metadata: Vec<String>,

        let table_name = "test_table";
        create_test_embedding_table(TEST_DB_URI, table_name)
            .await
            .expect("Failed to create test embedding table");

        let table = create_test_connection(TEST_DB_URI)
            .await
            .expect("Failed to create test connection")
            .open_table(table_name)
            .execute()
            .await
            .expect("Failed to open table");

        // db: &mut Connection,
        // table_name: &str,
        // query_vector: impl IntoQueryVector,
        // whole_query: bool,
        // file_context: bool,
        let db = &mut create_test_connection(TEST_DB_URI)
            .await
            .expect("Failed to create test connection");

        let query_vector = vec![0.0; VECTOR_DB_DIM_SIZE as usize];
        let content = query::query_vector_table(db, table_name, query_vector, false, false)
            .await
            .expect("Failed to query vector table");

        assert_eq!(content.len(), 30);
    }
}
