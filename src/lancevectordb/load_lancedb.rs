use anyhow::Context;
use anyhow::Result;
use arrow::array::{FixedSizeListArray, Float32Array, StringArray};
use arrow_array::types::Float32Type;
use arrow_array::{Int32Array, RecordBatch, RecordBatchIterator, RecordBatchReader};
use arrow_schema::Schema as ArrowSchema;
use arrow_schema::SchemaRef;
use arrow_schema::TimeUnit;
use arrow_schema::{DataType, Field};
use futures::StreamExt;
use futures::TryStreamExt;
use lancedb::index::Index;
use lancedb::query::QueryExecutionOptions;
use lancedb::query::{ExecutableQuery, QueryBase, Select};
use lancedb::Connection;
use std::sync::Arc;

pub struct TableSchema {
    pub name: String,
    pub id: Arc<Field>,
    pub content: Arc<Field>,
    pub metadata: Arc<Field>,
    pub embedding: Arc<Field>,
    pub created_at: Arc<Field>,
}

impl TableSchema {
    pub fn new(
        table_name: String,
        id: i32,
        content: String,
        metadata: String,
        embedding: Vec<Vec<f32>>,
    ) -> Self {
        TableSchema {
            name: table_name,
            id: Arc::new(Field::new("id", DataType::Int32, false)),
            content: Arc::new(Field::new("content", DataType::Utf8, false)),
            metadata: Arc::new(Field::new("metadata", DataType::Utf8, false)),
            embedding: Arc::new(Field::new(
                "embedding",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    1536,
                ),
                true,
            )),
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
            Arc::clone(&self.embedding),
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
                        (0..256).map(|_| Some(vec![Some(1.0); 128])),
                        128,
                    ),
                ),
                Arc::new(Int32Array::from_iter_values(
                    (0..256)
                        .map(|_| chrono::Utc::now().timestamp() as i32)
                        .collect::<Vec<i32>>(),
                )),
            ],
        )
        .context("Failed to create a RecordBatch")
    }
}

/// A struct that represents a chunk of text with its corresponding embeddings.
pub async fn create_lance_table(db: &mut Connection, table_schema: TableSchema) -> Result<()> {
    let table_name = table_schema.get_table_name();
    let all_tables = db.table_names().execute().await?;
    if all_tables.contains(&table_name.to_string()) {
        db.drop_table(table_name);
    }

    // Define the schema of the table.
    // let table_schema = TableSchema::new(
    //     "table_name".to_string(),
    //     1,
    //     "content".to_string(),
    //     "metadata".to_string(),
    //     vec![vec![1.0; 128]],
    // );

    let arrow_schema = Arc::new(table_schema.create_schema());

    // insert into table
    let table = db.open_table(table_name).execute().await?;
    let mut writer = table.merge_insert(&["id", "content"]);
    writer.when_not_matched_insert_all();
    writer.when_matched_update_all(None);

    // add rows to the writer
    for _ in 0..10 {
        let batch = table_schema.empty_batch()?;
        let record_batch =
            RecordBatchIterator::new(vec![batch].into_iter().map(Ok), arrow_schema.clone());

        // Pass the record batch to the writer.
        writer
            .clone()
            .execute(Box::new(record_batch))
            .await
            .context("Failed to execute a writer")?;
    }

    // read the table

    // let options = QueryExecutionOptions::default();
    // let query_stream = table
    //     .query()
    //     .execute_with_options(options)
    //     .await
    //     .context("Failed to execute a query")?;

    // let result_batch = query_stream
    //     .try_collect::<Vec<_>>()
    //     .await
    //     .context("Failed to collect a query stream")?;

    // Create an index on the embedding column.
    let table = db.open_table(table_name).execute().await?;
    table
        .create_index(&["embedding"], Index::Auto)
        .execute()
        .await?;

    Ok(())
}

// let conn = lancedb::connect("/tmp").execute().await.unwrap();
