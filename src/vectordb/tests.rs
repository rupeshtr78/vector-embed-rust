#[cfg(test)]
mod tests {
    use super::super::pg_vector::pg_client;
    use crate::app::config::VectorDbConfig;

    #[test]
    fn test_pg_client_success() {
        // Arrange
        let db_config = VectorDbConfig {
            host: String::from("localhost"),
            port: 5432,
            user: String::from("test_user"),
            dbname: String::from("test_db"),
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
            port: 5432,
            user: String::from("test_user"),
            dbname: String::from("test_db"),
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
