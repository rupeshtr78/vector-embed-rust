mod pg_client_tests {

    use crate::app::config::VectorDbConfig;
    use crate::vectordb::pg_vector::pg_client;
    #[cfg(test)]
    use std::io::{self, Write};
    use std::process::Command;

    const PORT: u16 = 5555;
    const HOST: &str = "10.0.0.213";
    const USER: &str = "rupesh";
    const DBNAME: &str = "vectordb";

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
        let result = pg_client(&db_config);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_pg_client_invalid_host() {
        // Arrange
        let db_config = VectorDbConfig {
            host: String::from("invalid_host"),
            port: PORT,
            user: String::from("USER"),
            dbname: String::from("DBNAME"),
            timeout: 5,
        };

        // Act
        let result = pg_client(&db_config);

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
        let result = pg_client(&db_config);

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
        let result = pg_client(&db_config);

        // Assert
        assert!(result.is_err());
    }
}
