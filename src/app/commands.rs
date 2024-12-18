use clap::{Parser, Subcommand};
use log::info;

use super::constants::{EMBEDDING_MODEL, VECTOR_DB_DIM_STR, VECTOR_DB_TABLE, VERSION};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    cmd: Option<Commands>,
}

#[derive(Subcommand, Debug)]

pub enum Commands {
    /// Write embedding to Postgres Vector Database
    Write {
        /// provide the input string to use
        #[clap(short, long)]
        input: Vec<String>,
        /// Provide the model to use default is "nomic-embed-text"
        /// Provide the model to use default is "nomic-embed-text"
        #[clap(default_value = &EMBEDDING_MODEL)]
        #[clap(short, long)]
        embed_model: String,
        /// Provide the table to use default is "from_rust"
        #[clap(default_value = VECTOR_DB_TABLE)]
        #[clap(short, long)]
        table: String,
        /// Provide the vector dimension to use default is 768
        #[clap(default_value = VECTOR_DB_DIM_STR)]
        #[clap(short, long)]
        dim: String,
    },

    /// Query the Vector Database
    Query {
        /// The query string to use
        #[clap(short, long)]
        query: Vec<String>,
        /// Provide the model to use default is "nomic-embed-text"
        #[clap(default_value = EMBEDDING_MODEL)]
        model: String,
        /// Provide the table to use default is "from_rust"
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
    Empty,
}

impl Commands {
    /// Checks if the command is a `Write` command.
    pub fn is_write(&self) -> bool {
        matches!(self, Commands::Write { .. })
    }

    /// Returns an `Option` of `&Commands` if the command is a `Write` command.
    pub fn write(&self) -> Option<&Commands> {
        if let Commands::Write { .. } = self {
            Some(self)
        } else {
            None
        }
    }

    /// Checks if the command is a `Query` command.
    pub fn is_query(&self) -> bool {
        matches!(self, Commands::Query { .. })
    }

    /// Returns an `Option` of `&Commands` if the command is a `Query` command.
    pub fn query(&self) -> Option<&Commands> {
        if let Commands::Query { .. } = self {
            Some(self)
        } else {
            None
        }
    }
}

pub fn build_args() -> Commands {
    let args = Args::parse();
    let commands = match args.cmd {
        Some(command) => command,
        None => {
            info!("No subcommand provided. Use --help for more information.");
            // return None;
            return Commands::Empty;
        }
    };

    commands
}

/// quick and dirty way to test the command line arguments
#[allow(dead_code)]
pub fn dbg_cmd() {
    // cargo run -- help
    // cargo run -- write --help
    // cargo run -- write --input "hello" "world"
    // cargo run -- write --input "hello","world" --model "nomic-embed-text1" --table "from_rust2" --dim 7681

    let args = Args::parse();
    let commands = match args.cmd {
        Some(command) => command,
        None => {
            println!("No subcommand provided. Use --help for more information.");
            return;
        }
    };

    match &commands {
        Commands::Write {
            input,
            embed_model,
            table,
            dim,
        } => {
            println!("Write command");
            println!("Input: {:?}", input);
            println!("Model: {:?}", embed_model);
            println!("Table: {:?}", table);
            println!("Dimension: {:?}", dim);
        }
        Commands::Query {
            query,
            model,
            table,
        } => {
            println!("Query command");
            println!("Query: {:?}", query);
            println!("Model: {:?}", model);
            println!("Table: {:?}", table);
        }
        Commands::Version { version } => {
            println!("Version command");
            println!("Version: {:?}", version);
        }
        Commands::Empty => {
            println!("Empty command");
        }
    }

    if commands.is_write() {
        if let Some(Commands::Write {
            input,
            embed_model,
            table,
            dim,
        }) = commands.write()
        {
            println!("Write command");
            println!("Input: {:?}", input);
            println!("Model: {:?}", embed_model);
            println!("Table: {:?}", table);
            println!("Dimension: {:?}", dim);
            // println!("Dimension: {:?}", dim); // ensure this statement is consistent with your original code requirements
        }
    }
}
