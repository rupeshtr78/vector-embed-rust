use ::std::io::{self, Write};

use crate::app::constants::{
    AI_MODEL, EMBEDDING_MODEL, SYSTEM_PROMPT_PATH, VECTOR_DB_DIM_STR, VECTOR_DB_TABLE, VERSION,
};
use anyhow::Context;
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use log::info;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[clap(name = "pg-vector-embed-rust")]
#[clap(about = "A CLI application for embedding and querying a vector database", long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    cmd: Option<Commands>,
    #[clap(short, long, global = true)]
    log_level: Option<LogLevel>,
}

#[derive(Subcommand, Debug)]

pub enum Commands {
    /// Write embedding to Postgres Vector Database
    PgWrite {
        /// provide the input string to use
        #[clap(short, long)]
        input: Vec<String>,
        /// Provide the model to use
        #[clap(default_value = &EMBEDDING_MODEL)]
        #[clap(short, long)]
        model: String,
        /// Provide the table to use default is "from_rust"
        #[clap(default_value = VECTOR_DB_TABLE)]
        #[clap(short, long)]
        table: String,
        /// Provide the vector dimension to use default is 768
        #[clap(default_value = VECTOR_DB_DIM_STR)]
        #[clap(short, long)]
        dim: String,
    },

    /// Query the PG Vector Database   
    PgQuery {
        /// The query string to use
        #[clap(short, long)]
        input: Vec<String>,
        /// Provide the model to use for query embedding
        #[clap(short, long)]
        #[clap(default_value = EMBEDDING_MODEL)]
        model: String,
        /// Provide the pg table to use to query
        #[clap(short, long)]
        #[clap(default_value = VECTOR_DB_TABLE)]
        table: String,
    },
    /// Get the version of the application
    Version {
        /// The version of the application
        #[clap(short, long)]
        #[clap(default_value = VERSION)]
        version: String,
    },

    /// Load a directory of files into the lance vector database
    Load {
        /// The path to the directory to load
        #[clap(short, long)]
        path: String,
        // chunk size
        #[clap(short, long)]
        #[clap(default_value = "2048")]
        chunk_size: usize,
    },
    /// Query the Lance Vector Database
    LanceQuery {
        /// The query string to use
        #[clap(short, long)]
        input: Vec<String>,
        /// Provide the model to use for query embedding
        #[clap(short, long)]
        #[clap(default_value = EMBEDDING_MODEL)]
        model: String,
        /// Provide the table to use to query
        #[clap(short, long)]
        table: String,
        /// Provide the database to use
        #[clap(short, long)]
        database: String,
        /// specify if the whole table query is to be used default is false
        #[clap(short, long)]
        #[clap(default_value = "false")]
        whole_query: String,
        /// specify if the additional file context default is false
        #[clap(short, long)]
        #[clap(default_value = "false")]
        file_context: String,
    },
    /// Query the Lance Vector Database and chat with the AI
    RagQuery {
        /// The query string to use
        #[clap(short, long)]
        input: Vec<String>,
        /// Provide the model to use for query embedding
        #[clap(short, long)]
        #[clap(default_value = EMBEDDING_MODEL)]
        model: String,
        /// Provide the AI model to use for generation
        #[clap(short, long)]
        #[clap(default_value = AI_MODEL)]
        ai_model: String,
        /// Provide the table to use to query
        #[clap(short, long)]
        table: String,
        /// Provide the database to use
        #[clap(short, long)]
        database: String,
        /// specify if the whole table query is to be used default is false
        #[clap(short, long)]
        #[clap(default_value = "false")]
        whole_query: String,
        /// specify if the additional file context default is false
        #[clap(short, long)]
        #[clap(default_value = "false")]
        file_context: String,
        /// specify if the system prompt is to be used default is false
        #[clap(short, long)]
        #[clap(default_value = SYSTEM_PROMPT_PATH)]
        system_prompt: String,
    },
    /// Chat with the AI
    Generate {
        /// Prompt for AI
        #[clap(short, long)]
        prompt: String,
        /// Provide the AI model to use for generation
        #[clap(short, long)]
        #[clap(default_value = AI_MODEL)]
        ai_model: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl Commands {
    /// Checks if the command is a `Write` command.
    pub fn is_write(&self) -> bool {
        matches!(self, Commands::PgWrite { .. })
    }

    /// Returns an `Option` of `&Commands` if the command is a `Write` command.
    pub fn write(&self) -> Option<&Commands> {
        if let Commands::PgWrite { .. } = self {
            Some(self)
        } else {
            None
        }
    }

    /// Checks if the command is a `Query` command.
    pub fn is_query(&self) -> bool {
        matches!(self, Commands::PgQuery { .. })
    }

    /// Returns an `Option` of `&Commands` if the command is a `Query` command.
    pub fn query(&self) -> Option<&Commands> {
        if let Commands::PgQuery { .. } = self {
            Some(self)
        } else {
            None
        }
    }

    /// Checks if the command is a `Version` command.
    pub fn is_version(&self) -> bool {
        matches!(self, Commands::Version { .. })
    }

    /// Returns an `Option` of `&Commands` if the command is a `Version` command.
    pub fn version(&self) -> Option<&Commands> {
        if let Commands::Version { .. } = self {
            Some(self)
        } else {
            None
        }
    }

    /// Checks if the command is a `Load` command.
    pub fn is_load(&self) -> bool {
        matches!(self, Commands::Load { .. })
    }

    /// Returns an `Option` of `&Commands` if the command is a `Load` command.
    pub fn load(&self) -> Option<&Commands> {
        if let Commands::Load { .. } = self {
            Some(self)
        } else {
            None
        }
    }

    pub fn fetch_args_from_cli(input: String, prompt_message: &str) -> String {
        if input.is_empty() {
            fetch_value(prompt_message)
        } else {
            input
        }
    }

    pub async fn fetch_prompt_from_cli(
        input: Vec<String>,
        prompt_message: &str,
    ) -> Result<Vec<String>> {
        if input.is_empty() {
            let user_input = fetch_valueV2(prompt_message)
                .await
                .with_context(|| format!("Failed to fetch prompt from CLI: {}", prompt_message))?;
            Ok(vec![user_input])
        } else {
            Ok(input)
        }
    }

    pub fn print_command(&self) {
        match self {
            Commands::PgWrite {
                input,
                model,
                table,
                dim,
            } => {
                println!("Write command");
                println!("Input: {:?}", input);
                println!("Model: {:?}", model);
                println!("Table: {:?}", table);
                println!("Dimension: {:?}", dim);
            }
            Commands::PgQuery {
                input,
                model,
                table,
            } => {
                println!("Query command");
                println!("Query: {:?}", input);
                println!("Model: {:?}", model);
                println!("Table: {:?}", table);
            }
            Commands::Version { version } => {
                println!("Version command");
                println!("Version: {:?}", version);
            }
            Commands::Load { path, chunk_size } => {
                println!("Load command");
                println!("Path: {:?}", path);
                println!("Chunk size: {:?}", chunk_size);
            }
            Commands::LanceQuery {
                input,
                model,
                table,
                database,
                whole_query,
                file_context,
            } => {
                println!("Lance Query command");
                println!("Query: {:?}", input);
                println!("Model: {:?}", model);
                println!("Table: {:?}", table);
                println!("Database: {:?}", database);
                println!("Whole Query: {:?}", whole_query);
                println!("File Context: {:?}", file_context);
            }
            Commands::RagQuery {
                input,
                model,
                ai_model,
                table,
                database,
                whole_query,
                file_context: file_query,
                system_prompt,
            } => {
                println!("Rag Query command");
                println!("Query: {:?}", input);
                println!("Model: {:?}", model);
                println!("AI Model: {:?}", ai_model);
                println!("Table: {:?}", table);
                println!("Database: {:?}", database);
                println!("Whole Query: {:?}", whole_query);
                println!("File Query: {:?}", file_query);
                println!("System Prompt: {:?}", system_prompt);
            }
            Commands::Generate { prompt, ai_model } => {
                println!("Chat command");
                println!("Prompt: {:?}", prompt);
                println!("AI Model: {:?}", ai_model);
            }
        }
    }
}

impl LogLevel {
    /// map loglevel enum to log::LevelFilter
    pub fn get_log_level_filter(&self) -> log::LevelFilter {
        match self {
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Error => log::LevelFilter::Error,
        }
    }
}

fn colog_init(log_level: LogLevel) {
    // colog::basic_builder()
    //     .filter_level(log_level.get_log_level_filter())
    //     .format_module_path(true)
    //     .init();

    // Initialize env_logger with module path formatting
    let mut builder = env_logger::Builder::new();
    builder
        .filter_level(log_level.get_log_level_filter())
        .format_module_path(true)
        .init();

    println!("Log level set to: {:?}", log_level);
}

/// Initiates the log builds the command line arguments and return the command to run.
pub fn build_args() -> Commands {
    let args = Args::parse();

    // Handle log level (if provided)
    if let Some(log_level) = args.log_level {
        match log_level {
            LogLevel::Error => colog_init(LogLevel::Error),
            LogLevel::Warn => colog_init(LogLevel::Warn),
            LogLevel::Info => colog_init(LogLevel::Info),
            LogLevel::Debug => colog_init(LogLevel::Debug),
        }
    } else {
        colog_init(LogLevel::Info);
    }

    // match args.cmd {
    //     Some(command) => command,
    //     None => {
    //         info!("No subcommand provided. Use --help for more information.");
    //         Commands::Version {
    //             version: VERSION.to_string(),
    //         }
    //     }
    // }

    args.cmd.map_or_else(
        || {
            info!("No subcommand provided. Use --help for more information.");
            Commands::Version {
                version: VERSION.to_string(),
            }
        },
        |cmd: Commands| cmd,
    )
}

/// Generic function to fetch a value from the command line if not provided as an argument.
///
/// # Arguments.
/// * `prompt_message` - The message to display when prompting the user for input.
/// # Returns
/// A `String` containing the value provided by the user or from the argument.
fn fetch_value(prompt_message: &str) -> String {
    print!("{}", prompt_message);
    io::stdout()
        .flush()
        .map_err(|e| e.to_string())
        .unwrap_or_else(|e| {
            panic!("Failed to flush stdout: {}", e);
        });
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

async fn fetch_valueV2(prompt_message: &str) -> Result<String> {
    // Print the prompt message
    print!("{}", prompt_message);
    tokio::io::stdout()
        .flush()
        .await
        .with_context(|| "Failed to flush stdout")?;

    // Create a buffer for reading input
    let stdin = tokio::io::stdin();
    let mut reader = tokio::io::BufReader::new(stdin);

    // Get a single line of input
    let mut input = String::new();
    reader
        .read_line(&mut input)
        .await
        .with_context(|| "Failed to read line")?;

    // Check if the input is empty
    if input.trim().is_empty() {
        return Err(anyhow::anyhow!("Input cannot be empty"));
    }

    // Return the trimmed input
    Ok(input.trim().to_string())
}
