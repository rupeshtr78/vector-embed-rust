// use crate::pgvectordb::VectorDbConfig;
// use postgres::Client;
// use std::error::Error;
// use std::io::Write;
// const PORT: u16 = 5555;
// const HOST: &str = "10.0.0.213";
// const USER: &str = "rupesh";
// const DBNAME: &str = "vectordb";
// const TEST_TABLE: &str = "test_table";
// const EMBEDDING_MODEL: &str = crate::app::constants::EMBEDDING_MODEL;
// const EMBEDDING_URL: &str = crate::app::constants::EMBEDDING_URL;

// fn create_dbconfig() -> VectorDbConfig {
//     VectorDbConfig {
//         host: String::from(HOST),
//         port: PORT,
//         user: String::from(USER),
//         dbname: String::from(DBNAME),
//         timeout: 5,
//     }
// }

// fn teardown_db(client: &mut Client, table: String) -> Result<(), Box<dyn Error>> {
//     let query_string = format!("DROP TABLE IF EXISTS {}", table);
//     client.execute(&query_string, &[])?;
//     Ok(())
// }

// #[cfg(test)]
// mod pg_client_tests {

//     use super::*;
//     use crate::pgvectordb::pg_vector;
//     use crate::pgvectordb::VectorDbConfig;
//     #[cfg(test)]
//     #[test]
//     fn test_pg_client_success() {
//         // Arrange
//         let db_config = VectorDbConfig {
//             host: String::from(HOST),
//             port: PORT,
//             user: String::from(USER),
//             dbname: String::from(DBNAME),
//             timeout: 5,
//         };

//         // Act
//         let result = pg_vector::pg_client(&db_config);

//         // Assert
//         assert!(result.is_ok());
//     }

//     #[test]
//     fn test_pg_client_invalid_host() {
//         // Arrange
//         let db_config = VectorDbConfig {
//             host: String::from("HOST"),
//             port: PORT,
//             user: String::from(USER),
//             dbname: String::from(DBNAME),
//             timeout: 5,
//         };

//         // Act
//         let result = pg_vector::pg_client(&db_config);

//         // Assert
//         assert!(result.is_err());
//         // Uncomment the line below to see the error message
//         // println!("{:?}", result.err());
//     }

//     #[test]
//     fn test_pg_client_timeout() {
//         // Arrange
//         let db_config = VectorDbConfig {
//             host: String::from("localhost"),
//             port: 5432,
//             user: String::from("test_user"),
//             dbname: String::from("test_db"),
//             timeout: 0, // Timeout set to 0 to simulate an immediate timeout
//         };

//         // Act
//         let result = pg_vector::pg_client(&db_config);

//         // Assert
//         assert!(result.is_err());
//     }

//     #[test]
//     fn test_pg_client_null_config() {
//         // Arrange
//         let db_config: VectorDbConfig = VectorDbConfig {
//             host: String::from(""),
//             port: 0,
//             user: String::from(""),
//             dbname: String::from(""),
//             timeout: 0,
//         };

//         // Act
//         let result = pg_vector::pg_client(&db_config);

//         // Assert
//         assert!(result.is_err());
//     }

//     #[test]
//     fn test_create_table_success() {
//         // Arrange
//         let table = TEST_TABLE.to_string();
//         let dimension = 768;
//         let db_config = create_dbconfig();
//         let mut client = pg_vector::pg_client(&db_config).unwrap();

//         // Act
//         let result = pg_vector::create_table(&mut client, &table, dimension);

//         // Assert
//         assert!(result.is_ok());

//         // Teardown
//         let _ = teardown_db(&mut client, table);
//     }

//     #[test]
//     fn test_create_table_duplicate() {
//         // Arrange
//         let table = String::from("duplicate_table");
//         let dimension = 768;
//         let db_config = create_dbconfig();
//         let mut client = pg_vector::pg_client(&db_config).unwrap();

//         // Act: First table creation should succeed
//         let result = pg_vector::create_table(&mut client, &table, dimension);
//         assert!(result.is_ok());

//         // Act: Second table creation should also succeed (should drop first if exists)
//         let result = pg_vector::create_table(&mut client, &table, dimension);
//         assert!(result.is_ok());

//         // Teardown
//         let _ = teardown_db(&mut client, table);
//     }

//     #[test]
//     fn test_create_table_invalid_dimension() {
//         // Arrange
//         let table = String::from("invalid_table");
//         let invalid_dimension = -5; // Invalid dimension; should be a positive integer
//         let db_config = create_dbconfig();
//         let mut client = pg_vector::pg_client(&db_config).unwrap();

//         // Act
//         let result = pg_vector::create_table(&mut client, &table, invalid_dimension);

//         // Assert
//         assert!(result.is_err());
//     }
// }

// #[cfg(test)]
// mod load_vector_data_tests {
//     use super::*;
//     use crate::embedder::config::EmbedRequest;
//     use crate::pgvectordb::pg_vector;
//     use crate::pgvectordb::pg_vector::pg_client;
//     use hyper::Client as HttpClient;
//     use postgres::Client;
//     use std::error::Error;
//     use tokio::runtime::Runtime; // Import runtime

//     fn setup_db() -> Result<Client, Box<dyn Error>> {
//         let db_config = create_dbconfig();
//         let pg_client: Client = pg_client(&db_config)?;
//         Ok(pg_client)
//     }

//     fn fetch_embedding(embed_data: &EmbedRequest, table: String) -> Vec<Vec<f32>> {
//         let mut client = setup_db().expect("Failed to set up database");
//         let dimension = 768;
//         let url = EMBEDDING_URL;
//         let input = ["item1".to_string(), "item2".to_string()];
//         let http_client = HttpClient::new();

//         // Arrange
//         let result = pg_vector::create_table(&mut client, &table, dimension);
//         assert!(result.is_ok());

//         let rt = Runtime::new().unwrap();

//         let response = rt.block_on(crate::embedder::create_embed_request(
//             url,
//             embed_data,
//             &http_client,
//         ));
//         assert!(response.is_ok());
//         response.unwrap().embeddings
//     }

//     #[test]
//     fn test_load_vector_data_success() {
//         let mut client = setup_db().expect("Failed to set up database");
//         let table = "test_table_success".to_string();
//         let dimension = 768;
//         let url = EMBEDDING_URL;
//         let input = vec!["item1".to_string(), "item2".to_string()];

//         // Arrange
//         let embed_data = EmbedRequest {
//             model: EMBEDDING_MODEL.to_string(),
//             input: input.clone(),
//             metadata: None,
//         };
//         let embeddings = fetch_embedding(&embed_data, table.clone());

//         // Act
//         let result = pg_vector::load_vector_data(&mut client, &table, &embed_data, &embeddings);

//         // Assert
//         assert!(result.is_ok());

//         // Check the data insertion
//         let query = format!("SELECT id, content FROM {}", table);
//         let rows = client.query(&query, &[]);
//         assert!(rows.is_ok());
//         match rows {
//             Ok(rows) => {
//                 assert_eq!(rows.len(), 2);
//                 assert_eq!(rows[0].get::<_, String>(1), "item1");
//                 assert_eq!(rows[1].get::<_, String>(1), "item2");
//             }
//             Err(e) => {
//                 println!("Error: {:?}", e);
//             }
//         }

//         // Teardown
//         let _ = teardown_db(&mut client, table);
//     }

//     #[test]
//     fn test_load_vector_data_length_mismatch() {
//         // Arrange
//         let mut client = setup_db().expect("Failed to set up database");
//         let table = "test_table_length_bad".to_string();
//         let dimension = 768;
//         let url = EMBEDDING_URL;
//         let input = vec!["item1".to_string(), "item2".to_string()];

//         // Arrange
//         let embed_data = EmbedRequest {
//             model: EMBEDDING_MODEL.to_string(),
//             input: input.clone(),
//             metadata: None,
//         };
//         let embeddings = fetch_embedding(&embed_data, table.clone());

//         // Add an extra item to the input
//         let embed_data_wrong = EmbedRequest {
//             model: EMBEDDING_MODEL.to_string(),
//             input: vec![
//                 "item1".to_string(),
//                 "item2".to_string(),
//                 "item3".to_string(),
//             ],
//             metadata: None,
//         };
//         // Act
//         let result =
//             pg_vector::load_vector_data(&mut client, &table, &embed_data_wrong, &embeddings);

//         // Assert
//         assert!(result.is_err());

//         assert_eq!(
//             result.unwrap_err().to_string(),
//             "Invalid Input and Embeddings length or mismatch"
//         );
//     }

//     #[test]
//     fn test_load_vector_data_empty_input() {
//         // Arrange

//         let mut client = setup_db().expect("Failed to set up database");
//         let table = "test_table_wrong_input".to_string();
//         let dimension = 768;
//         let url = EMBEDDING_URL;
//         let input = vec!["item1".to_string(), "item2".to_string()];

//         // Arrange
//         let embed_data = EmbedRequest {
//             model: EMBEDDING_MODEL.to_string(),
//             input: input.clone(),
//             metadata: None,
//         };
//         let embeddings = fetch_embedding(&embed_data, table.clone());

//         // Add an extra item to the input
//         let embed_data_wrong = EmbedRequest {
//             model: EMBEDDING_MODEL.to_string(),
//             input: vec![],
//             metadata: None,
//         };

//         // Act
//         let result =
//             pg_vector::load_vector_data(&mut client, &table, &embed_data_wrong, &embeddings);

//         // Assert
//         assert!(result.is_err());

//         let wrong_embeddings: Vec<Vec<f32>> = vec![vec![]];

//         let result2 =
//             pg_vector::load_vector_data(&mut client, &table, &embed_data_wrong, &wrong_embeddings);

//         assert!(result2.is_err());

//         let wrong_embeddings2 = vec![vec![1.0, 768.123, 768.89]];

//         let result3 =
//             pg_vector::load_vector_data(&mut client, &table, &embed_data_wrong, &wrong_embeddings2);

//         assert!(result3.is_err());
//     }

//     #[test]
//     fn test_query_nearest_success() {
//         // Arrange
//         let mut client = setup_db().expect("Failed to set up database");

//         // Insert test data
//         client
//             .execute(
//                 "CREATE TABLE IF NOT EXISTS test_table_2 (content TEXT, embedding VECTOR(768))",
//                 &[],
//             )
//             .expect("Failed to create test_table");

//         let query_vec = vec![vec![0.1; 768]]; // Query a vector similar to item1

//         // Act
//         let result = pg_vector::query_nearest(&mut client, &"test_table_2".to_string(), &query_vec);

//         // Assert
//         assert!(result.await.is_ok());
//         // You may want to validate the output if you have access to capture it

//         // Teardown
//         let _ = teardown_db(&mut client, "test_table".to_string());
//     }

//     #[test]
//     fn test_query_nearest_invalid_vector_length() {
//         // Arrange
//         let mut client = setup_db().expect("Failed to set up database");

//         let query_vec = vec![vec![0.1; 768], vec![0.2; 768]]; // Invalid length

//         // Act
//         let result = pg_vector::query_nearest(&mut client, &"test_table_3".to_string(), &query_vec);

//         // Assert
//         assert!(result.is_err());
//         assert_eq!(
//             result.unwrap_err().to_string(),
//             "Failed to fetch query embedding Query vector length should be 1"
//         );

//         let _ = teardown_db(&mut client, "test_table".to_string());
//     }

//     #[test]
//     fn test_query_nearest_no_results() {
//         // Arrange
//         let mut client = setup_db().expect("Failed to set up database");
//         let query_vec = vec![vec![0.1; 768]]; // Query vector

//         // Insert test data
//         client
//             .execute(
//                 "CREATE TABLE IF NOT EXISTS test_table_empty (content TEXT, embedding VECTOR(768))",
//                 &[],
//             )
//             .expect("Failed to create test_table");

//         // Act
//         let result =
//             pg_vector::query_nearest(&mut client, &"test_table_empty".to_string(), &query_vec);

//         // Assert
//         assert!(result.is_ok()); // No errors expected

//         // Teardown
//         let _ = teardown_db(&mut client, "test_table_empty".to_string());
//     }

//     #[test]
//     fn test_query_nearest_invalid_table() {
//         // Arrange
//         let mut client = setup_db().expect("Failed to set up database");
//         let query_vec = vec![vec![0.1; 768]]; // Query vector

//         // Act
//         let result =
//             pg_vector::query_nearest(&mut client, &"table_not_exist".to_string(), &query_vec);

//         // Assert
//         assert!(true); // Should not error, but no results will be returned
//     }
// }
