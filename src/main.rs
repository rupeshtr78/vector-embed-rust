use crate::app::config::NewVectorDbConfig;
use crate::app::constants::{VECTOR_DB_HOST, VECTOR_DB_NAME, VECTOR_DB_PORT, VECTOR_DB_USER};

use app::commands::{build_args, Commands};
use log::LevelFilter;
use log::{error, info};

mod app;
mod embedding;
mod vectordb;

fn main() {
    // colog::init();
    colog::basic_builder()
        .filter_level(LevelFilter::Info)
        .init();

    info!("Starting");

    let commands = build_args();

    // let url = EMBEDDING_URL;

    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            error!("Failed to build runtime: {}", e);
            return;
        }
    };

    let db_config = NewVectorDbConfig(
        VECTOR_DB_HOST,
        VECTOR_DB_PORT,
        VECTOR_DB_USER,
        VECTOR_DB_NAME,
    );

    if commands.is_write() {
        if let Some(Commands::Write {
            input,
            model,
            table,
            dim,
        }) = commands.write()
        {
            let input_list = input;
            let embed_model = model.to_string();
            let vector_table = table.to_string();
            let dimension = dim.to_string();
            info!("Using the Write arguments below:");
            info!(" Input: {:?}", input);
            info!(" Model: {:?}", model);
            info!(" Table: {:?}", table);
            info!(" Dimension: {:?}", dim);

            let embed_handler = embedding::run_embedding::run_embedding_load(
                &rt,
                embed_model,
                input_list,
                vector_table,
                dimension,
                db_config,
            );

            match embed_handler {
                Ok(_) => {
                    info!("Embedding loaded successfully");
                }
                Err(e) => {
                    error!("Error: {}", e);
                }
            }
        }
        rt.shutdown_timeout(std::time::Duration::from_secs(1));
    } else if commands.is_query() {
        if let Some(Commands::Query {
            input,
            model,
            table,
        }) = commands.query()
        {
            let input_list = input;
            let embed_model = model.to_string();
            let vector_table = table.to_string();
            info!("Query command is run with below arguments:");
            info!(" Query: {:?}", input);
            info!(" Model: {:?}", model);
            info!(" Table: {:?}", table);

            vectordb::query_vector::run_query(
                &rt,
                embed_model,
                input_list,
                vector_table,
                db_config,
            );

            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
    } else {
        error!("No command provided");
    }
}
