// use arrow::{ArrayRef, FixedSizeListArray, Float32Array, RecordBatch, StringArray};
use arrow::array::{FixedSizeListArray, Float32Type};
use arrow_schema::Schema;
use lancedb::{ConnectOptions, Database};

/// A struct that represents a chunk of text with its corresponding embeddings.
async fn create_lance_table(
    db: &mut Database,
    table_name: &str,
) -> Result<(), lancedb::error::Error> {
    let schema = Schema::new(vec![
        arrow_schema::Field::new("content", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new(
            "vector",
            arrow_schema::DataType::FixedSizeList(
                Box::new(arrow_schema::Field::new(
                    "item",
                    arrow_schema::DataType::Float32,
                    true,
                )),
                128,
            ),
            true,
        ),
    ]);

    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(StringArray::from(vec!["example content"])),
            Arc::new(
                FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                    (0..128).map(|_| Some(1.0f32)),
                    128,
                ),
            ),
        ],
    )?;

    db.create_table(table_name, Box::pin(std::iter::once(Ok(batch))))?
        .execute()
        .await?;
    Ok(())
}

/// Insert a list of chunks with their embeddings into the LanceDB.
async fn insert_into_lance_db(
    db: &mut Database,
    table_name: &str,
    chunks_with_embeddings: Vec<(String, Vec<f32>)>,
) -> Result<(), lancedb::error::Error> {
    let schema = Arc::new(Schema::new(vec![
        arrow_schema::Field::new("content", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new(
            "vector",
            arrow_schema::DataType::FixedSizeList(
                Box::new(arrow_schema::Field::new(
                    "item",
                    arrow_schema::DataType::Float32,
                    true,
                )),
                128,
            ),
            true,
        ),
    ]));

    let batches = RecordBatchIterator::from(vec![RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(
                chunks_with_embeddings.iter().map(|c| c.0.clone()).collect(),
            )),
            Arc::new(
                FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                    chunks_with_embeddings.into_iter().map(|(_, v)| Some(v)),
                    128,
                )?,
            ),
        ],
    )?]);

    db.table(table_name)?
        .insert(Box::pin(batches))
        .execute()
        .await?;

    Ok(())
}
