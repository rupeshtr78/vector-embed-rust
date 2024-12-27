# PG Vector Embedding and Query Rust

This repository contains a Rust-based system for managing vector embeddings and querying them using a PostgreSQL-backed vector database. The system is designed to handle embedding generation, storage, and querying.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Directory Structure](#directory-structure)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Running the Application](#running-the-application)
- [Usage](#usage)
  - [Commands](#commands)
  - [Configuration](#configuration)
  - [Embedding and Querying](#embedding-and-querying)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

## Overview

The system is composed of several modules that handle different aspects of the embedding and querying process:

- **Commands**: Handles command-line arguments and subcommands.
- **Config**: Manages configuration settings for embedding requests and database connections.
- **Constants**: Provides constant values used throughout the application.
- **Embedding**: Contains logic for generating embeddings and persisting them to the database.
- **VectorDB**: Handles interactions with the PostgreSQL database for storing and querying vector embeddings.

## Features

- **Embedding Generation**: Generate vector embeddings from input data.
- **Database Persistence**: Store embeddings in a PostgreSQL database.
- **Querying**: Query the database to find nearest neighbors based on vector embeddings.

## Directory Structure

```
src/
├── app/
│   ├── commands.rs
│   ├── config.rs
│   └── constants.rs
├── embedding/
│   ├── run_embedding.rs
│   └── vector_embedding.rs
├── main.rs
├── tests/
│   ├── setup_docker.rs
│   ├── test_pgclient.rs
│   ├── test_query_vector.rs
│   ├── test_run_embedding.rs
│   └── test_vector_embedding.rs
├── vectordb/
│   ├── pg_vector.rs
│   └── query_vector.rs
├── lib.rs
├── vectordb/mod.rs
├── tests/mod.rs
├── embedding/mod.rs
└── app/mod.rs
```

## Getting Started

### Prerequisites

- Rust (latest stable version)
- PostgreSQL Vector Db.
- Docker (for running tests)
- Active Ollama Service with `nomic-embed-text` or similar model. 

### Installation

1. Clone the repository:
   ```sh
   git clone https://github.com/rupeshtr78/pg-vector-embed-rust.git
   cd pg-vector-embed-rust
   ```

2. Install dependencies:
   ```sh
   cargo build
   ```

### Running the Application

1. Start the PostgreSQL vector database (if not already running).
2. Ollama service should be running with the specified model.

3. Run the application:
   ```sh
   cargo run
      

   ```

## Usage

### Commands

The application supports various commands and subcommands. Use the `--help` flag to see available options:

```sh
   cargo run -- --help
   cargo run -- write --input "dog sound is called bark" --input "cat sounds is called purr" --model "nomic-embed-text" --table "from_rust2" --dim 768 --log-level "debug"
   cargo run -- query --input "who is barking" --model "nomic-embed-text" --table "from_rust2"  
```

### Configuration

Configuration settings for embedding requests and database connections are managed in `src/app/config.rs`. You can modify these settings as needed.

### Embedding and Querying

- **Generate Embeddings**: Use the `run_embedding` function to generate embeddings and persist them to the database.
- **Query Embeddings**: Use the `run_query` function to query the database for nearest neighbors based on vector embeddings.

## Testing

The test suite requires postgres vectordb and ollama with embedding model to be running in the correct configuration.

```sh
cargo test
```

## Contributing

Contributions are welcome! Please read the [CONTRIBUTING.md](CONTRIBUTING.md) file for details on how to contribute to this project.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

Feel free to customize this README further based on your specific needs and project details.
