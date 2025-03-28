use crate::app::constants::VECTOR_DB_DIM_SIZE;
use crate::embedder::config::{EmbedRequest, EmbedResponse};
use anyhow::Result;
use anyhow::{Context, Ok};
use arrow::array::{FixedSizeListArray, StringArray, TimestampSecondArray};
use arrow_array::types::Float32Type;
use arrow_array::{Int32Array, RecordBatch, RecordBatchIterator};
use arrow_schema::TimeUnit;
use arrow_schema::{DataType, Field};
use arrow_schema::{Schema as ArrowSchema, Schema};
use lancedb::index::scalar::FtsIndexBuilder;
use lancedb::index::Index;
use lancedb::{Connection, Table};
use std::sync::Arc;
use std::vec;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct TableSchema {
    pub name: String,
    pub id: Arc<Field>,
    pub content: Arc<Field>,
    pub metadata: Arc<Field>,
    pub model: Arc<Field>,
    pub vector: Arc<Field>,
    pub created_at: Arc<Field>,
    pub chunk_number: Arc<Field>,
}

impl TableSchema {
    pub fn new(table_name: &String) -> Self {
        TableSchema {
            name: table_name.to_string(),
            id: Arc::new(Field::new("id", DataType::Int32, false)),
            content: Arc::new(Field::new("content", DataType::Utf8, false)),
            metadata: Arc::new(Field::new("metadata", DataType::Utf8, false)),
            vector: Arc::new(Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    VECTOR_DB_DIM_SIZE,
                ),
                true,
            )),
            model: Arc::new(Field::new("model", DataType::Utf8, false)),
            created_at: Arc::new(Field::new(
                "created_at",
                DataType::Timestamp(TimeUnit::Second, None),
                false,
            )),
            chunk_number: Arc::new(Field::new("chunk_number", DataType::Int32, true)),
        }
    }

    pub fn create_schema(&self) -> ArrowSchema {
        ArrowSchema::new(vec![
            Arc::clone(&self.id),
            Arc::clone(&self.content),
            Arc::clone(&self.metadata),
            Arc::clone(&self.vector),
            Arc::clone(&self.model),
            Arc::clone(&self.created_at),
            Arc::clone(&self.chunk_number),
        ])
    }

    fn get_table_name(&self) -> &str {
        self.name.as_str()
    }

    #[allow(dead_code)]
    /// Create an empty RecordBatch with the schema can be used for testing
    /// Arguments:
    /// - &self: &TableSchema
    /// Returns:
    /// - Result<RecordBatch> - The RecordBatch (Arrow)
    pub fn empty_batch(&self) -> Result<RecordBatch> {
        RecordBatch::try_new(
            Arc::new(self.create_schema()),
            vec![
                Arc::new(Int32Array::from_iter_values(0..256)),
                Arc::new(StringArray::from_iter_values((0..256).map(|_| ""))),
                Arc::new(StringArray::from_iter_values((0..256).map(|_| ""))),
                Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        (0..256).map(|_| Some(vec![Some(1.0); VECTOR_DB_DIM_SIZE as usize])),
                        VECTOR_DB_DIM_SIZE,
                    ),
                ),
                Arc::new(StringArray::from_iter_values((0..256).map(|_| ""))),
                Arc::new(TimestampSecondArray::from_iter_values(
                    (0..256).map(|_| chrono::Utc::now().timestamp()),
                )),
                Arc::new(Int32Array::from_iter_values((0..256).map(|_| 0))),
            ],
        )
        .context("Failed to create a RecordBatch")
    }
}

/// Create a table in the database with the given schema.
/// Arguments:
/// - db: &mut Connection
/// - table_schema: &TableSchema
/// Returns:
/// - Result<(), Box<dyn Error>>
pub async fn create_lance_table(db: &mut Connection, table_schema: &TableSchema) -> Result<()> {
    let table_name = table_schema.get_table_name();
    let all_tables = db.table_names().execute().await?;
    if all_tables.contains(&table_name.to_string()) {
        db.drop_table(table_name)
            .await
            .context("Failed to drop a table")?;
    }

    let arrow_schema = Arc::new(table_schema.create_schema());
    db.create_empty_table(table_name, arrow_schema.clone())
        .execute()
        .await
        .context("Failed to create a table")?;

    log::info!("Table created successfully");

    Ok(())
}

#[allow(dead_code)]
/// Insert an empty batch into the database
/// Arguments:
/// - db: &mut Connection
/// - table_schema: &TableSchema
/// - table_name: &str
/// - arrow_schema: Arc<Schema>
/// Returns:
/// - Result<(), Box<dyn Error>>
async fn insert_empty_batch(
    db: &mut Connection,
    table_schema: &TableSchema,
    table_name: &str,
    arrow_schema: Arc<Schema>,
) -> Result<()> {
    let table = db.open_table(table_name).execute().await?;
    let mut writer = table.merge_insert(&["id", "content"]);
    writer.when_not_matched_insert_all();
    writer.when_matched_update_all(None);

    // add rows to the writer
    let batch = table_schema.empty_batch()?;
    let record_batch = RecordBatchIterator::new(
        vec![batch].into_iter().map(std::result::Result::Ok),
        arrow_schema.clone(),
    );

    // Pass the record batch to the writer.
    writer
        .execute(Box::new(record_batch))
        .await
        .context("Failed to insert records")?;
    Ok(())
}

/// Insert embeddings into the database
/// Arguments:
/// - table_schema: &TableSchema
/// - records: RecordBatch (Arrow)
/// - table: Table (lancedb)
/// Returns:
/// - Result<(), Box<dyn Error>>
pub async fn insert_embeddings(
    table_schema: &TableSchema,
    records: RecordBatch,
    table: Table,
) -> Result<()> {
    let arrow_schema = Arc::new(table_schema.create_schema());
    let record_iter = vec![records].into_iter().map(std::result::Result::Ok);
    let record_batch = RecordBatchIterator::new(record_iter, arrow_schema);

    let mut writer =
        table.merge_insert(&["content", "metadata", "vector", "model", "chunk_number"]);
    // add merge options to writer
    writer.when_not_matched_insert_all();

    let write_result = writer.execute(Box::new(record_batch)).await;

    if let Err(e) = write_result {
        log::error!("Failed to insert records: {:?}", e);
        return Err(anyhow::Error::msg("Failed to insert records"));
    }

    log::info!("Records inserted successfully");

    Ok(())
}

/// Create a RecordBatch from the EmbedRequest and EmbedResponse
/// Arguments:
/// - id: i32
/// - request: Arc<RwLock<EmbedRequest>>
/// - response: EmbedResponse
/// - table_schema: &TableSchema
/// Returns:
/// - Result<RecordBatch, Box<dyn Error>> - The RecordBatch (Arrow)
pub async fn create_record_batch(
    id: i32,
    request: Arc<RwLock<EmbedRequest>>,
    response: EmbedResponse,
    table_schema: &TableSchema,
) -> Result<RecordBatch> {
    if response.embeddings.is_empty() {
        return Err(anyhow::Error::msg("No embeddings found in the response"));
    }
    let request = request.read().await;

    // let num_embeddings = response.embeddings.len();
    let len = response.embeddings.len();

    let id_array = Arc::new(Int32Array::from_iter_values((0..len).map(|_| id)));
    let content_array = Arc::new(StringArray::from_iter_values(
        request.input.iter().take(len).map(|s| s.to_string()),
    ));

    let dir_name = match request.metadata {
        Some(ref dir_name) => dir_name.to_string(),
        None => String::from("Empty"),
    };

    let metadata_array = Arc::new(StringArray::from_iter_values(
        std::iter::repeat(dir_name).take(len).map(|s| s.to_string()),
    ));

    // let metadata_array = Arc::new(StringArray::from_iter_values(
    //     request.metadata.iter().map(|s| s.to_string()).chain(
    //         std::iter::repeat(String::from(""))
    //             .take(len - 1)
    //             .map(|s| s.to_string()),
    //     ),
    // ));

    let model_array = Arc::new(StringArray::from_iter_values(
        (0..len).map(|_| request.model.to_string()),
    ));

    let vectors: Vec<Option<Vec<Option<f32>>>> = response
        .embeddings
        .into_iter() // Iterate over the outer Vec
        .map(|embedding| {
            let inner_vec: Vec<Option<f32>> = embedding
                .into_iter() // Iterate over the inner Vec
                .map(Some) // Convert each item to Some(item)
                .collect(); // Collect into Vec<Option<f32>>
            Some(inner_vec) // Wrap the inner Vec in Some
        })
        .collect(); // Collect into Vec<Option<Vec<Option<f32>>>>

    let embedding_array = Arc::new(
        FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(vectors, VECTOR_DB_DIM_SIZE),
    );

    let created_at_array = Arc::new(TimestampSecondArray::from_iter_values(
        (0..len).map(|_| chrono::Utc::now().timestamp()),
    ));

    let chunk_number_array = Arc::new(Int32Array::from_iter_values(
        (0..len).map(|_| request.chunk_number.unwrap_or(0)),
    ));

    let record_batch = RecordBatch::try_new(
        Arc::new(table_schema.create_schema()),
        vec![
            id_array,
            content_array,
            metadata_array,
            embedding_array,
            model_array,
            created_at_array,
            chunk_number_array,
        ],
    )
    .context("Failed to create a Embedding Records")?;

    Ok(record_batch)
}

/// Create an index on the embedding column
/// IVF_PQ Index: LanceDB also supports the IVF_PQ (Inverted File with Product Quantization) index,
/// which divides the dataset into partitions and applies product quantization for efficient vector compression.
/// This index type is used for performing ANN searches in LanceDB.
/// Approximate Nearest Neighbor (ANN)
/// LanceDB does not automatically create the ANN index.
/// need to explicitly create the index with the appropriate index type
/// (e.g., IVF_HNSW_SQ)
/// Arguments:
/// - db: &mut Connection
/// - table_name: &str
/// - column: Vec<&str>
/// Returns:
/// - Result<(), Box<dyn Error>>
pub async fn create_index_on_embedding(
    db: &mut Connection,
    table_name: &str,
    column: Vec<&str>,
) -> Result<()> {
    let table = db.open_table(table_name).execute().await?;

    // Initialize the builder first
    let hns_index = lancedb::index::vector::IvfHnswSqIndexBuilder::default()
        .distance_type(crate::app::constants::LANCEDB_DISTANCE_FN) // Set the desired distance type, e.g., L2
        .num_partitions(100) // Set the number of partitions, e.g., 100
        .sample_rate(256) // Set the sample rate
        .max_iterations(50) // Set the max iterations for training
        .ef_construction(300); // Set the ef_construction value

    // Now create the Index using the builder
    let index = Index::IvfHnswSq(hns_index);

    table
        .create_index(&column, index)
        .execute()
        .await
        .with_context(|| {
            format!(
                "Failed to create an index on table: {:?} column: {:?}",
                table_name, column
            )
        })?;

    log::info!(
        "Created inverted index on table: {:?} column: {:?}",
        table_name,
        column
    );

    Ok(())
}

/// Create an inverted index on the specified column for full-text search
/// Arguments:
/// - db: &mut Connection
/// - table_name: &str
/// - column: Vec<&str>
/// Returns:
/// - Result<(), Box<dyn Error>>
pub async fn create_inverted_index(
    db: &mut Connection,
    table_name: &str,
    columns: Vec<&str>,
) -> Result<()> {
    let table = db
        .open_table(table_name)
        .execute()
        .await
        .with_context(|| format!("Failed to open table: {:?}", table_name))?;

    // columns &["metadata", "content"]
    table
        .create_index(&columns, Index::FTS(FtsIndexBuilder::default()))
        .execute()
        .await
        .with_context(|| {
            format!(
                "Failed to create an inverted index on table: {:?} column: {:?}",
                table_name, columns
            )
        })?;

    log::info!(
        "Created inverted index on table: {:?} column: {:?}",
        table_name,
        columns
    );

    Ok(())
}
