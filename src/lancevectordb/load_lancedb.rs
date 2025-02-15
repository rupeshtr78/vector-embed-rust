use crate::embedder::config::{EmbedRequest, EmbedResponse};
use crate::app::constants::{self, VECTOR_DB_DIM_SIZE};
use anyhow::Context;
use anyhow::Result;
use arrow::array::{FixedSizeListArray, StringArray, TimestampSecondArray};
use arrow_array::types::Float32Type;
use arrow_array::{Int32Array, RecordBatch, RecordBatchIterator};
use arrow_schema::Schema as ArrowSchema;
use arrow_schema::TimeUnit;
use arrow_schema::{DataType, Field};
use lancedb::index::Index;
use lancedb::Connection;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TableSchema {
    pub name: String,
    pub id: Arc<Field>,
    pub content: Arc<Field>,
    pub metadata: Arc<Field>,
    pub model: Arc<Field>,
    pub vector: Arc<Field>,
    pub created_at: Arc<Field>,
}

impl TableSchema {
    pub fn new(table_name: String) -> Self {
        TableSchema {
            name: table_name,
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
        }
    }

    fn create_schema(&self) -> ArrowSchema {
        ArrowSchema::new(vec![
            Arc::clone(&self.id),
            Arc::clone(&self.content),
            Arc::clone(&self.metadata),
            Arc::clone(&self.vector),
            Arc::clone(&self.model),
            Arc::clone(&self.created_at),
        ])
    }

    fn get_table_name(&self) -> &str {
        &self.name.as_str()
    }

    fn empty_batch(&self) -> Result<RecordBatch> {
        RecordBatch::try_new(
            Arc::new(self.create_schema()),
            vec![
                Arc::new(Int32Array::from_iter_values(0..256)),
                Arc::new(StringArray::from_iter_values((0..256).map(|_| ""))),
                Arc::new(StringArray::from_iter_values((0..256).map(|_| ""))),
                Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        (0..256).map(|_| Some(vec![Some(1.0); 768])),
                        VECTOR_DB_DIM_SIZE,
                    ),
                ),
                Arc::new(StringArray::from_iter_values((0..256).map(|_| ""))),
                Arc::new(TimestampSecondArray::from_iter_values(
                    (0..256).map(|_| chrono::Utc::now().timestamp()),
                )),
            ],
        )
        .context("Failed to create a RecordBatch")
    }
}

/// Create a table in the database with the given schema
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

    // insert into table
    let table = db.open_table(table_name).execute().await?;
    let mut writer = table.merge_insert(&["id", "content"]);
    writer.when_not_matched_insert_all();
    writer.when_matched_update_all(None);

    // add rows to the writer
    let batch = table_schema.empty_batch()?;
    let record_batch =
        RecordBatchIterator::new(vec![batch].into_iter().map(Ok), arrow_schema.clone());

    // Pass the record batch to the writer.
    writer
        .execute(Box::new(record_batch))
        .await
        .context("Failed to insert records")?;

    // Create an index on the embedding column.
    // let table = db.open_table(table_name).execute().await?;
    // table
    //     .create_index(&["embedding"], Index::Auto)
    //     .execute()
    //     .await
    //     .context("Failed to create an index")?;

    log::info!("Table created successfully");

    Ok(())
}

/// Insert embeddings into the database
/// Arguments:
/// - db: &mut Connection
/// - table_schema: &TableSchema
/// - records: RecordBatch (Arrow)
/// Returns:
/// - Result<(), Box<dyn Error>>
pub async fn insert_embeddings(
    db: &mut Connection,
    table_schema: &TableSchema,
    records: RecordBatch,
) -> Result<()> {
    let table_name = table_schema.get_table_name();
    let arrow_schema = Arc::new(table_schema.create_schema());
    let table = db.open_table(table_name).execute().await?;
    let mut writer = table.merge_insert(&["content", "metadata", "vector", "model"]);
    writer.when_not_matched_insert_all();
    writer.when_matched_update_all(None);

    let record_batch = RecordBatchIterator::new(vec![records].into_iter().map(Ok), arrow_schema);

    writer
        .execute(Box::new(record_batch))
        .await
        .context("Failed to insert records")?;

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
pub fn create_record_batch(
    id: i32,
    request: Arc<std::sync::RwLock<EmbedRequest>>,
    response: EmbedResponse,
    table_schema: &TableSchema,
) -> Result<RecordBatch> {
    if response.embeddings.is_empty() {
        return Err(anyhow::Error::msg("No embeddings found in the response"));
    }
    let request = request
        .read()
        .map_err(|e| anyhow::Error::msg(format!("Error: {}", e)))?;

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

    let record_batch = RecordBatch::try_new(
        Arc::new(table_schema.create_schema()),
        vec![
            id_array,
            content_array,
            metadata_array,
            embedding_array,
            model_array,
            created_at_array,
        ],
    )
    .context("Failed to create a Embedding Records")?;

    Ok(record_batch)
}

/// Create an index on the embedding column
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
        .distance_type(constants::LANCEDB_DISTANCE_FN) // Set the desired distance type, e.g., L2
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
        .context("Failed to create an index")?;

    log::info!(
        "Created index on table: {:?} column: {:?}",
        table_name,
        column
    );

    Ok(())
}
