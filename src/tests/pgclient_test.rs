use crate::app::config::VectorDbConfig;
use postgres::Client;
use std::io::{self, Write};
use std::{error::Error, process::Command};
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

fn teardown_db(client: &mut Client, table: String) -> Result<(), Box<dyn Error>> {
    let query_string = format!("DROP TABLE IF EXISTS {}", table);
    client.execute(&query_string, &[])?;
    Ok(())
}

#[cfg(test)]
mod pg_client_tests {

    use super::*;
    use crate::app::config::VectorDbConfig;
    use crate::vectordb::pg_vector;
    #[cfg(test)]
    #[test]
    fn test_pg_client_success() {
        // Arrange
        let db_config = VectorDbConfig {
            host: String::from(HOST),
            port: PORT,
            user: String::from(USER),
            dbname: String::from(DBNAME),
            timeout: 5,
        };

        // Act
        let result = pg_vector::pg_client(&db_config);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_pg_client_invalid_host() {
        // Arrange
        let db_config = VectorDbConfig {
            host: String::from("HOST"),
            port: PORT,
            user: String::from(USER),
            dbname: String::from(DBNAME),
            timeout: 5,
        };

        // Act
        let result = pg_vector::pg_client(&db_config);

        // Assert
        assert!(result.is_err());
        // Uncomment the line below to see the error message
        // println!("{:?}", result.err());
    }

    #[test]
    fn test_pg_client_timeout() {
        // Arrange
        let db_config = VectorDbConfig {
            host: String::from("localhost"),
            port: 5432,
            user: String::from("test_user"),
            dbname: String::from("test_db"),
            timeout: 0, // Timeout set to 0 to simulate an immediate timeout
        };

        // Act
        let result = pg_vector::pg_client(&db_config);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_pg_client_null_config() {
        // Arrange
        let db_config: VectorDbConfig = VectorDbConfig {
            host: String::from(""),
            port: 0,
            user: String::from(""),
            dbname: String::from(""),
            timeout: 0,
        };

        // Act
        let result = pg_vector::pg_client(&db_config);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_create_table_success() {
        // Arrange
        let table = TEST_TABLE.to_string();
        let dimension = 768;
        let db_config = create_dbconfig();
        let mut client = pg_vector::pg_client(&db_config).unwrap();

        // Act
        let result = pg_vector::create_table(&mut client, &table, dimension);

        // Assert
        assert!(result.is_ok());

        // Teardown
        let _ = teardown_db(&mut client, table);
    }

    #[test]
    fn test_create_table_duplicate() {
        // Arrange
        let table = String::from("duplicate_table");
        let dimension = 768;
        let db_config = create_dbconfig();
        let mut client = pg_vector::pg_client(&db_config).unwrap();

        // Act: First table creation should succeed
        let result = pg_vector::create_table(&mut client, &table, dimension);
        assert!(result.is_ok());

        // Act: Second table creation should also succeed (should drop first if exists)
        let result = pg_vector::create_table(&mut client, &table, dimension);
        assert!(result.is_ok());

        // Teardown
        let _ = teardown_db(&mut client, table);
    }

    #[test]
    fn test_create_table_invalid_dimension() {
        // Arrange
        let table = String::from("invalid_table");
        let invalid_dimension = -5; // Invalid dimension; should be a positive integer
        let db_config = create_dbconfig();
        let mut client = pg_vector::pg_client(&db_config).unwrap();

        // Act
        let result = pg_vector::create_table(&mut client, &table, invalid_dimension);

        // Assert
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod load_vector_data_tests {
    use super::*;
    use crate::app::config::{EmbedRequest, VectorDbConfig};
    use crate::embedding::vector_embedding;
    use crate::vectordb::pg_vector;
    use crate::vectordb::pg_vector::pg_client;
    use postgres::Client;
    use std::error::Error;
    use std::pin::Pin;
    use tokio::runtime::Runtime; // Import runtime

    fn setup_db() -> Result<Client, Box<dyn Error>> {
        let db_config = create_dbconfig();
        let mut pg_client: postgres::Client = pg_client(&db_config)?;
        Ok(pg_client)
    }

    #[test]
    fn test_load_vector_data_success() {
        let mut client = setup_db().expect("Failed to set up database");
        let table = "test_table".to_string();
        let dimension = 768;
        let url = EMBEDDING_URL;
        let input = vec!["item1".to_string(), "item2".to_string()];

        // Arrange
        let result = pg_vector::create_table(&mut client, &table, dimension);
        assert!(result.is_ok());

        let embed_data = EmbedRequest {
            model: EMBEDDING_MODEL.to_string(),
            input: vec!["item1".to_string(), "item2".to_string()],
        };
        let rt = Runtime::new().unwrap();

        let response = rt.block_on(vector_embedding::create_embed_request(url, &embed_data));
        assert!(response.is_ok());
        let embeddings = response.unwrap().embeddings;

        // Act
        let result = pg_vector::load_vector_data(&mut client, &table, &embed_data, &embeddings);

        // Assert
        assert!(result.is_ok());

        // Check the data insertion
        // let rows: Vec<(String, Vec<f32>)> = client
        //     .query("SELECT content, embedding FROM test_table", &[])
        //     .expect("Failed to query table")
        //     .iter()
        //     .map(|row| {
        //         let content: String = row.get(0);
        //         let embedding: Vec<f32> = row.get(1);
        //         (content, embedding)
        //     })
        //     .collect();

        // assert_eq!(rows.len(), 2);
        // assert_eq!(rows[0].0, "item1");
        // assert_eq!(rows[1].0, "item2");
    }

    // #[test]
    // fn test_load_vector_data_length_mismatch() {
    //     // Arrange
    //     let mut client = setup_db().expect("Failed to set up database");
    //     client
    //         .execute(
    //             "CREATE TABLE test_table (content TEXT, embedding VECTOR(768))",
    //             &[],
    //         )
    //         .expect("Failed to create table");

    //     let input = EmbedRequest {
    //         input: vec!["item1".to_string(), "item2".to_string()],
    //     };
    //     let embeddings = vec![vec![0.1; 768]]; // One less than input

    //     // Act
    //     let result = load_vector_data(&mut client, "test_table", &input, &embeddings);

    //     // Assert
    //     assert!(result.is_err());
    //     assert_eq!(
    //         result.unwrap_err().to_string(),
    //         "Input and Embeddings length mismatch"
    //     );
    // }

    // #[test]
    // fn test_load_vector_data_empty_input() {
    //     // Arrange
    //     let mut client = setup_db().expect("Failed to set up database");
    //     client
    //         .execute(
    //             "CREATE TABLE test_table (content TEXT, embedding VECTOR(768))",
    //             &[],
    //         )
    //         .expect("Failed to create table");

    //     let input = EmbedRequest { input: vec![] };
    //     let embeddings: Vec<Vec<f32>> = vec![]; // No embeddings

    //     // Act
    //     let result = load_vector_data(&mut client, "test_table", &input, &embeddings);

    //     // Assert
    //     assert!(result.is_ok()); // Should succeed, as no data to insert
    // }

    // #[test]
    // fn test_load_vector_data_invalid_embeddings() {
    //     // Arrange
    //     let mut client = setup_db().expect("Failed to set up database");
    //     client
    //         .execute(
    //             "CREATE TABLE test_table (content TEXT, embedding VECTOR(768))",
    //             &[],
    //         )
    //         .expect("Failed to create table");

    //     let input = EmbedRequest {
    //         input: vec!["item1".to_string()],
    //     };
    //     let embeddings = vec![vec![0.1; 1000]]; // Invalid size, assuming dimension is 768

    //     // Act
    //     let result = load_vector_data(&mut client, "test_table", &input, &embeddings);

    //     // Assert
    //     assert!(result.is_err());
    //     // You would also want to validate that the error is of the expected type
    // }

    // #[test]
    // fn test_load_vector_data_database_error_handling() {
    //     // Here you can simulate potential database error scenarios, like invalid table names, etc.
    //     // For simplicity, this example won't run an actual test due to its complexity.
    // }
}

// #[cfg(test)]
// mod query_tests {
//     use super::*;
//     use crate::vectordb::pg_vector::pg_client;

//     fn setup_db() -> Result<Client, Box<dyn Error>> {
//         let db_config = VectorDbConfig {
//             host: String::from(HOST),
//             port: PORT,
//             user: String::from(USER),
//             dbname: String::from(DBNAME),
//             timeout: 5,
//         };
//         let mut client = pg_client(&db_config)?;
//         client.execute(
//             "CREATE TABLE IF NOT EXISTS test_table (content TEXT, embedding VECTOR(768))",
//             &[],
//         )?;
//         Ok(client)
//     }

//     fn teardown_db(client: &mut Client) -> Result<(), Box<dyn Error>> {
//         client.execute("DROP TABLE IF EXISTS test_table", &[])?;
//         Ok(())
//     }

//     #[test]
//     fn test_query_nearest_success() {
//         // Arrange
//         let mut client = setup_db().expect("Failed to set up database");

//         // Insert test data
//         client
//             .execute(
//                 "INSERT INTO test_table (content, embedding) VALUES ($1, $2)",
//                 &[&"item1", &vec![0.1; 768]],
//             )
//             .expect("Failed to insert item1");
//         client
//             .execute(
//                 "INSERT INTO test_table (content, embedding) VALUES ($1, $2)",
//                 &[&"item2", &vec![0.2; 768]],
//             )
//             .expect("Failed to insert item2");

//         let query_vec = vec![vec![0.1; 768]]; // Query a vector similar to item1

//         // Act
//         let result = query_nearest(&mut client, &"test_table".to_string(), &query_vec);

//         // Assert
//         assert!(result.is_ok());
//         // You may want to validate the output if you have access to capture it
//     }

//     #[test]
//     fn test_query_nearest_invalid_vector_length() {
//         // Arrange
//         let mut client = setup_db().expect("Failed to set up database");

//         let query_vec = vec![vec![0.1; 768], vec![0.2; 768]]; // Invalid length

//         // Act
//         let result = query_nearest(&mut client, &"test_table".to_string(), &query_vec);

//         // Assert
//         assert!(result.is_err());
//         assert_eq!(
//             result.unwrap_err().to_string(),
//             "Failed to fetch query embedding Query vector length should be 1"
//         );
//     }

//     #[test]
//     fn test_query_nearest_no_results() {
//         // Arrange
//         let mut client = setup_db().expect("Failed to set up database");
//         let query_vec = vec![vec![0.1; 768]]; // Query vector

//         // Act
//         let result = query_nearest(&mut client, &"test_table".to_string(), &query_vec);

//         // Assert
//         assert!(result.is_ok()); // No errors expected
//                                  // Further checks can be conducted if required, maybe using mock or capturing logs
//     }

//     #[test]
//     fn test_query_nearest_empty_table() {
//         // Arrange
//         let mut client = setup_db().expect("Failed to set up database");
//         let query_vec = vec![vec![0.1; 768]]; // Query vector

//         // Act
//         let result = query_nearest(&mut client, &"test_table".to_string(), &query_vec);

//         // Assert
//         assert!(result.is_ok()); // Should not error, but no results will be returned
//     }
// }
