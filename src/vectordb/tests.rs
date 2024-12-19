#[cfg(test)]
mod tests {
    use super::super::pg_vector::pg_client;
    use crate::app::config::VectorDbConfig;
    use std::io::{self, Write};
    use std::process::Command;

    fn setup_docker() -> io::Result<()> {
        // Create a docker-compose.yml string
        let docker_compose_yml = r#"
version: '3.8'

services:
  postgres:
    image: rupeshtr/pg-vector:v01
    container_name: pg-vector-test
    environment:
      POSTGRES_USER: test_user
      POSTGRES_PASSWORD: test_password
      POSTGRES_DB: test_db
    ports:
      - "5432:5432"
    volumes:
      - pg_data:/var/lib/postgresql/data

volumes:
  pg_data:
"#;

        // Write the docker-compose.yml to a file
        let file_name = "docker-compose-test.yml";
        let mut file = std::fs::File::create(file_name)?;
        write!(file, "{}", docker_compose_yml)?;

        // Execute docker-compose up
        let output = Command::new("docker-compose")
            .arg("up")
            .arg("-d")
            .arg("-f")
            .arg(file_name)
            .output()?;

        // Check if the command was successful
        if output.status.success() {
            println!("Docker compose set up successfully.");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Error running docker-compose: {}", stderr);
        }

        Ok(())
    }

    fn teardown_docker() -> io::Result<()> {
        // Execute docker-compose down
        let output = Command::new("docker-compose").arg("down").output()?;

        // Check if the command was successful
        if output.status.success() {
            println!("Docker compose stopped successfully.");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("Error running docker-compose: {}", stderr);
        }

        // Delete the docker-compose-test.yml file
        std::fs::remove_file("docker-compose-test.yml")?;

        Ok(())
    }

    #[test]
    fn docker_setup_and_pg_client_success() {
        // Setup Docker
        setup_docker().expect("Failed to set up Docker");

        // Run the pg_client success test
        let db_config = VectorDbConfig {
            host: String::from("localhost"),
            port: 5432,
            user: String::from("test_user"),
            dbname: String::from("test_db"),
            timeout: 5,
        };

        let result = pg_client(&db_config);
        assert!(result.is_ok());

        // Teardown Docker
        teardown_docker().expect("Failed to tear down Docker");
    }

    #[test]
    fn docker_setup_and_pg_client_invalid_host() {
        // Setup Docker
        setup_docker().expect("Failed to set up Docker");

        // Run the pg_client invalid host test
        let db_config = VectorDbConfig {
            host: String::from("invalid_host"),
            port: 5432,
            user: String::from("test_user"),
            dbname: String::from("test_db"),
            timeout: 5,
        };

        let result = pg_client(&db_config);
        assert!(result.is_err());

        // Teardown Docker
        teardown_docker().expect("Failed to tear down Docker");
    }

    #[test]
    fn docker_setup_and_pg_client_timeout() {
        // Setup Docker
        setup_docker().expect("Failed to set up Docker");

        // Run the pg_client timeout test
        let db_config = VectorDbConfig {
            host: String::from("localhost"),
            port: 5432,
            user: String::from("test_user"),
            dbname: String::from("test_db"),
            timeout: 0,
        };

        let result = pg_client(&db_config);
        assert!(result.is_err());

        // Teardown Docker
        teardown_docker().expect("Failed to tear down Docker");
    }

    #[test]
    fn docker_setup_and_pg_client_null_config() {
        // Setup Docker
        setup_docker().expect("Failed to set up Docker");

        // Run the pg_client null config test
        let db_config = VectorDbConfig {
            host: String::from(""),
            port: 0,
            user: String::from(""),
            dbname: String::from(""),
            timeout: 0,
        };

        let result = pg_client(&db_config);
        assert!(result.is_err());

        // Teardown Docker
        teardown_docker().expect("Failed to tear down Docker");
    }
}
