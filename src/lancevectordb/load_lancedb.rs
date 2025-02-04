use arrow::array::{FixedSizeListArray, Float32Array, StringArray};
use arrow_array::types::Float32Type;
use arrow_array::{Int32Array, RecordBatch, RecordBatchIterator};
use arrow_schema::TimeUnit;
use arrow_schema::{DataType, Field, Schema};
use lancedb::connect;
use lancedb::index::Index;
use lancedb::Connection;
use std::sync::Arc;

pub struct TableSchema {
    pub id: Field,
    pub content: Field,
    pub metadata: Field,
    pub embedding: Field,
    pub created_at: Field,
}

impl TableSchema {
    pub fn new(id: i32, content: String, metadata: String, embedding: Vec<Vec<f32>>) -> Self {
        TableSchema {
            id: Field::new("id", DataType::Int32, false),
            content: Field::new("content", DataType::Utf8, false),
            metadata: Field::new("metadata", DataType::Utf8, false),
            embedding: Field::new(
                "embedding",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    1536,
                ),
                true,
            ),
            created_at: Field::new(
                "created_at",
                DataType::Timestamp(TimeUnit::Second, None),
                false,
            ),
        }
    }
}

/// A struct that represents a chunk of text with its corresponding embeddings.
async fn create_lance_table(
    db: &mut Connection,
    table_name: &str,
) -> Result<(), lancedb::error::Error> {
    // Define the schema of the table.
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("content", DataType::Utf8, false),
        Field::new("metadata", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 128),
            true,
        ),
    ]));

    // Create a RecordBatch stream.
    let batches = RecordBatchIterator::new(
        vec![RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int32Array::from_iter_values(0..256)),
                Arc::new(
                    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
                        (0..256).map(|_| Some(vec![Some(1.0); 128])),
                        128,
                    ),
                ),
            ],
        )
        .unwrap()]
        .into_iter()
        .map(Ok),
        schema.clone(),
    );

    db.create_table(table_name, Box::pin(std::iter::once(Ok(batches))))?
        .execute()
        .await?;

    // table_name.create_index(&["vector"], Index::Auto)
    //     .execute()
    //     .await
    //     .unwrap();

    Ok(())
}

/// Insert a list of chunks with their embeddings into the LanceDB.
// async fn insert_into_lance_db(
//     db: &mut Connection,
//     table_name: &str,
//     chunks_with_embeddings: Vec<(String, Vec<f32>)>,
// ) -> Result<(), lancedb::error::Error> {
//     let schema = Arc::new(Schema::new(vec![
//         arrow_schema::Field::new("content", arrow_schema::DataType::Utf8, false),
//         arrow_schema::Field::new(
//             "vector",
//             arrow_schema::DataType::FixedSizeList(
//                 Box::new(arrow_schema::Field::new(
//                     "item",
//                     arrow_schema::DataType::Float32,
//                     true,
//                 )),
//                 128,
//             ),
//             true,
//         ),
//     ]));
//
//     let batches = RecordBatchIterator::from(vec![RecordBatch::try_new(
//         schema.clone(),
//         vec![
//             Arc::new(StringArray::from(
//                 chunks_with_embeddings.iter().map(|c| c.0.clone()).collect(),
//             )),
//             Arc::new(
//                 FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
//                     chunks_with_embeddings.into_iter().map(|(_, v)| Some(v)),
//                     128,
//                 )?,
//             ),
//         ],
//     )?]);
//
//     db.table(table_name)?
//         .insert(Box::pin(batches))
//         .execute()
//         .await?;
//
//     Ok(())
// }

fn query_lance_db(
    db: &mut Connection,
    table_name: &str,
    query: &str,
) -> Result<Vec<String>, lancedb::error::Error> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("content", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 128),
            true,
        ),
    ]));

    let batches = db
        .table(table_name)?
        .query(query)?
        .nearest_to(&[1.0; 128])
        .execute()?;

    let mut results = Vec::new();
    for batch in batches {
        let batch = batch?;
        let content = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        results.extend(content.iter().map(|s| s.to_string()));
    }

    Ok(results)
}
