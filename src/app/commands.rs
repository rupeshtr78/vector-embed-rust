use clap::{Parser, Subcommand};

use super::constants::{EMBEDDING_MODEL, VECTOR_DB_DIM, VECTOR_DB_TABLE, VERSION};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    cmd: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Write embedding to Postgres Vector Database
    Write {
        /// provide the input string to use
        #[clap(short, long)]
        input: Vec<String>,
        /// Provide the model to use default is "nomic-embed-text"
        /// Provide the model to use default is "nomic-embed-text"
        #[clap(default_value = EMBEDDING_MODEL)]
        model: String,
        /// Provide the table to use default is "from_rust"
        #[clap(default_value = VECTOR_DB_TABLE)]
        table: String,
        /// Provide the vector dimension to use default is 768
        #[clap(default_value_t = VECTOR_DB_DIM)]
        dim: i32,
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
}

pub fn build_args() {
    let args = Args::parse();
    let commands = match args.cmd {
        Some(command) => command,
        None => {
            println!("No subcommand provided. Use --help for more information.");
            return;
        }
    };

    match commands {
        Commands::Write {
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
    }
}

/// quick and dirty way to test the command line arguments
pub fn dbg_cmd() {
    // cargo run -- help
    // cargo run -- write --help
    // cargo run -- write --input "hello" "world"
    // cargo run -- write --input "hello" "world" --model "nomic-embed-text" --table "from_rust" --dim 768

    let args = Args::parse();
    match args.cmd {
        Some(command) => {
            dbg!(command);
        }
        None => {
            println!("No subcommand provided. Use --help for more information.");
        }
    }
}
