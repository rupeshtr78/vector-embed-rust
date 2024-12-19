#[cfg(test)]
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
