# Vector Embedding and RAG with LLM Integration in Rust

This repository features a Rust-based system for managing vector embeddings, enabling queries through a LanceDB vector database. It supports interaction with LLM models using the retrieved context (currently compatible with Ollama). The system is designed to handle tasks such as embedding generation, storage, querying, and engaging in interactive chat sessions.

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
    - [Chat Integration](#chat-integration)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

## Overview

The system is composed of several modules that handle different aspects of the embedding, querying, and chat processes:

- **Commands**: Handles command-line arguments and subcommands.
- **Config**: Manages configuration settings for embedding requests, database connections, and chat interactions.
- **Constants**: Provides constant values used throughout the application.
- **Embedding**: Contains logic for generating embeddings and persisting them to the database.
- **VectorDB**: Handles interactions with the lancedb vector database for storing and querying vector embeddings.
- **Chat**: Integrates with the Ollama LLM model to provide interactive chat functionalities based on retrieved embeddings.
- **TODO**: Https support, Adding Tests, Adding PDF Support, interactive cli 
## Features

- **File Type Support**: The tool supports multiple file types including Rust (`rs`), Python (`py`), C++ (`cpp`), Java (`java`), JavaScript (`js`), TypeScript (`ts`), and text files. (TODO: Add PDF support)
- **LanceDB Integration**:
    - Create and manage vector tables in LanceDB.
    - Insert and update records in LanceDB tables.
    - Query the nearest vectors in LanceDB tables.
- **Embedding**:
    - Generate embeddings for text data using an external embedding service.
    - Store embeddings in LanceDB tables.
- **CLI Interface**: Command-line interface for easy interaction with the tool.
- **Database Persistence**: Store embeddings in a lance vector database.
- **Querying**: Query the database to find nearest neighbors based on vector embeddings.
- **Chat Integration**: Interact with the Ollama LLM model using embeddings retrieved from the database.

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Lancedb Vector DB.
- Docker (for running tests)
- Active Ollama Service with `nomic-embed-text` or similar model.

### Installation

1. Clone the repository:

   ```sh
   git clone https://github.com/rupeshtr78/vector-embed-rust.git
   cd vector-embed-rust
   ```

2. Install dependencies:
   ```sh
   cargo build
   ```

### Running the Application

1. Ensure the Ollama service is running with the specified model.
2. Run the application:

   ```sh
   cargo run -- --help

   
   ```

## Usage

```
Commands:
  version      Get the version of the application
  load         Load a directory of files into the lance vector database
  lance-query  Query the Lance Vector Database
  rag-query    Query the Lancedb and chat with the AI with context
  generate     Chat with the AI
  help         Print this message or the help of the given subcommand(s)
```

### Commands

The application supports various commands and subcommands. Use the `--help` flag to see available options:

```sh
# Generate embeddings and store them in the database
cargo run -- load -p /home/rupesh/aqrtr/gits/vector-embed-rust/src/scripts

# Query the database for nearest neighbors
cargo run -- rag-query -t scripts_table -d scripts_db -i "what is temperature"

# Start an interactive chat session
cargo run -- chat -p "what is mirostat"
```

### Configuration

Configuration settings for embedding requests, database connections, and chat interactions are managed in `src/app/config.rs`. You can modify these settings as needed.

### Embedding and Querying

- **Generate Embeddings**: Use the `run_embedding` function to generate embeddings and persist them to the database.
- **Query Embeddings**: Use the `run_query` function to query the database for nearest neighbors based on vector embeddings.

### Chat Integration

- **Interactive Chat**: Use the `chat` command to start an interactive chat session with the Ollama LLM model. The chat session will use embeddings retrieved from the database to provide context-aware responses.

## Testing

The test suite requires Ollama with an embedding and LLM model to be running in the correct configuration.

* TODO missing test

```sh
cargo test
```

## Code Structure

- **`code_loader.rs`**: Handles file type detection and content loading.
- **`load_lancedb.rs`**: Manages LanceDB table creation, insertion, and querying.
- **`query.rs`**: Contains logic for running queries on LanceDB tables.
- **`chat.rs`**: Implements chat functionalities using the Ollama LLM model.
- **`main.rs`**: Entry point for the CLI application.

## Contributing

Contributions are welcome! Please read the [CONTRIBUTING.md](CONTRIBUTING.md) file for details on how to contribute to this project.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
