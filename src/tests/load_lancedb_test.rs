#[allow(unused)]
mod load_lancedb_test {
    use crate::app::constants::VECTOR_DB_DIM_SIZE;
    use crate::embedder::config::{EmbedRequest, EmbedResponse};
    use crate::lancevectordb::load_lancedb::{
        create_index_on_embedding, create_inverted_index, create_lance_table, create_record_batch,
        insert_embeddings, TableSchema,
    };
    use anyhow::Context;
    use anyhow::Result;
    use arrow::array::{FixedSizeListArray, Int32Array, StringArray};
    use lancedb::connection::Connection;
    use lancedb::table;
    use serde::de;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Mock constants for testing
    const TEST_DB_URI: &str = "test_db";

    fn delete_test_db() {
        // delete the test_db if it exists
        let _ = std::fs::remove_dir_all(TEST_DB_URI);
        // println!("Deleted test_db");
    }
    // Helper function to create a test connection
    async fn create_test_connection() -> Result<Connection> {
        // delete the test_db if it exists
        delete_test_db();
        let db = lancedb::connect(TEST_DB_URI)
            .execute()
            .await
            .context("Failed to create test connection")?;

        Ok(db)
    }

    // Helper function to create a test table schema
    fn create_test_table_schema(table_name: &str) -> TableSchema {
        TableSchema::new(&table_name.to_string())
    }

    #[tokio::test]
    async fn test_create_lance_table() -> Result<()> {
        let mut db = create_test_connection().await?;
        let table_name = "TEST_TABLE_NAME_CREATE";
        let table_schema = create_test_table_schema(&table_name);

        // drop table if it exists
        if db
            .table_names()
            .execute()
            .await?
            .contains(&table_name.to_string())
        {
            db.drop_table(table_name).await?;
        }
        // Test table creation
        create_lance_table(&mut db, &table_schema).await?;

        // Verify table exists
        let table = db.open_table(table_name).execute().await?;
        assert_eq!(table.name(), table_name);

        // Cleanup
        db.drop_table(table_name).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_create_record_batch() -> Result<()> {
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

        let record_batch = create_record_batch(1, request, response, &table_schema).await?;

        assert_eq!(record_batch.num_rows(), 1);
        assert_eq!(record_batch.num_columns(), 7);

        // Verify content
        let content = record_batch
            .column(1)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        assert_eq!(content.value(0), "test content");

        Ok(())
    }

    #[tokio::test]
    async fn test_create_inverted_index() -> Result<()> {
        let mut db = create_test_connection().await?;
        let table_name = "TEST_TABLE_NAME_INVERTED_INDEX";
        let table_schema = create_test_table_schema(&table_name);
        // Ensure table does not exist before creation
        if db
            .table_names()
            .execute()
            .await?
            .contains(&table_name.to_string())
        {
            db.drop_table(table_name).await?;
        }

        // Create table first
        create_lance_table(&mut db, &table_schema).await?;

        // Create a mock record batch
        let record_batch = table_schema.empty_batch()?;
        let table = db.open_table(table_name).execute().await?;
        insert_embeddings(&table_schema, record_batch, table).await?;

        // Test index creation
        let metadata_col = table_schema.metadata.name();
        create_inverted_index(&mut db, table_name, vec![metadata_col.as_str()]).await?;

        // Cleanup
        db.drop_table(table_name).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_create_index_on_embedding() -> Result<()> {
        let mut db = create_test_connection().await?;
        let table_name = "TEST_TABLE_NAME_INDEX";
        let table_schema = create_test_table_schema(&table_name);

        // Ensure table does not exist before creation
        if db
            .table_names()
            .execute()
            .await?
            .contains(&table_name.to_string())
        {
            db.drop_table(table_name).await?;
        }

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

        // Cleanup
        db.drop_table(table_name).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_table_schema_creation() {
        let table_name = "TEST_TABLE_NAME_SCHEMA";
        let table_schema = create_test_table_schema(&table_name);

        assert_eq!(table_schema.name, table_name);
        assert_eq!(table_schema.id.name(), "id");
        assert_eq!(table_schema.content.name(), "content");
        assert_eq!(table_schema.vector.name(), "vector");

        let arrow_schema = table_schema.create_schema();
        assert_eq!(arrow_schema.fields().len(), 7);
    }

    #[tokio::test]
    async fn test_empty_batch_creation() -> Result<()> {
        let table_name = "TEST_TABLE_NAME_EMPTY_BATCH";
        let table_schema = create_test_table_schema(&table_name);
        let batch = table_schema.empty_batch()?;

        assert_eq!(batch.num_rows(), 256);
        assert_eq!(batch.num_columns(), 7);
        // verify embedding column
        let embedding_col = batch
            .column(3)
            .as_any()
            .downcast_ref::<FixedSizeListArray>()
            .unwrap();
        assert_eq!(embedding_col.value_length(), VECTOR_DB_DIM_SIZE);
        Ok(())
    }

    #[tokio::test]
    async fn zz_clean_up() {
        delete_test_db(); // Runs last due to name sorting
    }
}
