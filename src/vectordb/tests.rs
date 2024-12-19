#[cfg(test)]
use super::pg_vector::pg_client;
use crate::app::config::VectorDbConfig;

// run docker compose up -d to start the database
use std::io::{self, Write};
use std::process::Command;

const PORT: u16 = 5555;
const HOST: &str = "0.0.0.0";
const USER: &str = "rupesh";
const DBNAME: &str = "vectordb";

#[test]
fn test_setup_docker() -> io::Result<()> {
    // Create a docker-compose.yml string
    let docker_compose_yml = r#"
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
    file.write_all(docker_compose_yml.as_bytes())?;

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

#[test]
fn stop_docker() -> io::Result<()> {
    // Execute docker-compose down
    let output = Command::new("docker-compose").arg("down").output()?;

    // Check if the command was successful
    if output.status.success() {
        println!("Docker compose stopped successfully.");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error running docker-compose: {}", stderr);
    }

    // delete the docker-compose-test.yml file
    std::fs::remove_file("docker-compose-test.yml")?;

    Ok(())
}
