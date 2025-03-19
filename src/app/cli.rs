use crate::app::commands::Commands;
use crate::app::constants::{VECTOR_DB_HOST, VECTOR_DB_NAME, VECTOR_DB_PORT, VECTOR_DB_USER};
use crate::lancevectordb;
use crate::pgvectordb::run_embedding::run_embedding_load;
use crate::pgvectordb::VectorDbConfig;
use crate::pgvectordb::{pg_vector, query_vector};
use anyhow::Result;
use anyhow::{Context, Ok};
use hyper::client::connect::HttpInfo;
use hyper::client::HttpConnector;
use hyper::Client as HttpClient;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use log::{debug, info};
use postgres::Client;
use std::time::Duration;
use tokio::sync::Mutex;

pub fn cli(commands: Commands, rt: tokio::runtime::Runtime, url: &str) -> Result<()> {
    match commands {
        Commands::PgWrite {
            input,
            model,
            table,
            dim,
        } => {
            let input_list = input;
            let embed_model = model.to_string();
            let vector_table = table.to_string();
            let dimension = dim.to_string();
            info!("Using the Write arguments below:");
            info!(" Input Length: {:?}", input_list.len());
            info!(" Model: {:?}", model);
            info!(" Table: {:?}", table);
            info!(" Dimension: {:?}", dim);

            let db_config = VectorDbConfig::NewVectorDbConfig(
                VECTOR_DB_HOST,
                VECTOR_DB_PORT,
                VECTOR_DB_USER,
                VECTOR_DB_NAME,
            );

            // Initialize the client outside the thread and wrap it in Arc<Mutex>
            let client: Mutex<Client> = pg_vector::pg_client(&db_config)
                .context("Failed to create a new client")?
                .into();

            // Initialize the http client outside the thread // TODO wrap in Arc<Mutex>
            let http_client = HttpClient::new();

            rt.block_on(run_embedding_load(
                url,
                embed_model,
                &input_list,
                vector_table,
                dimension,
                client,
                &http_client,
            ))
            .context("Failed to run embedding load")?;

            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        Commands::PgQuery {
            input,
            model,
            table,
        } => {
            let input_list = input;
            let embed_model = model.to_string();
            let vector_table = table.to_string();

            println!("Query to fetch context is run with below arguments:");
            println!(" Query: {:?}", input_list);
            println!(" Model: {:?}", model);
            println!(" Table: {:?}", table);

            let db_config = VectorDbConfig::NewVectorDbConfig(
                VECTOR_DB_HOST,
                VECTOR_DB_PORT,
                VECTOR_DB_USER,
                VECTOR_DB_NAME,
            );

            let mut client =
                pg_vector::pg_client(&db_config).context("Failed to create a new client")?;

            // Initialize the http client outside the thread // TODO wrap in Arc<Mutex>
            let http_client = HttpClient::new();

            rt.block_on(query_vector::run_pg_vector_query(
                &rt,
                embed_model,
                &input_list,
                vector_table,
                &mut client,
                &http_client,
            ))
            .context("Failed to run query")?;

            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        Commands::Load { path, chunk_size } => {
            info!("Using the Load arguments below:");
            info!(" Path: {:?}", path);
            info!(" Chunk Size: {:?}", chunk_size);

            let http_client = HttpClient::new();
            let embed_url = format!("{}/{}", url, "api/embed");

            rt.block_on(check_connection(&format!("{}/{}", url, "api/version")))
                .context("Failed to check connection")?;

            // rt.block_on(check_client(
            //     &http_client,
            //     &format!("{}/{}", url, "api/version"),
            // ))
            // .context("Failed to check client")?;

            rt.block_on(lancevectordb::run_embedding_pipeline(
                path,
                chunk_size,
                &embed_url,
                &http_client,
            ))
            .context("Failed to run lancevectordb")?;

            // shutdown the runtime after the embedding is done
            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        Commands::LanceQuery {
            input,
            model,
            table,
            database,
            whole_query,
            file_context,
        } => {
            // let input_list = Commands::fetch_prompt_from_cli(input.clone(), "Enter query: ");
            let input_list = rt
                .block_on(Commands::fetch_prompt_from_cli(
                    input.clone(),
                    "Enter query: ",
                ))
                .with_context(|| format!("Failed to fetch prompt from CLI: {:?}", input))?;
            let embed_model = model.to_string();
            let vector_table = table.to_string();
            let db_uri = database.to_string();
            let whole_query: bool = whole_query
                .parse()
                .context("Failed to parse whole_query flag")?;
            let file_context: bool = file_context
                .parse()
                .context("Failed to parse file_query flag")?;

            info!(" Query: {:?}", input_list);
            info!(" Model: {:?}", model);
            info!(" Table: {:?}", table);
            info!(" Whole Query: {:?}", whole_query);
            info!(" File Query: {:?}", file_context);

            // Initialize the http client outside the thread // TODO wrap in Arc<Mutex>
            let http_client = HttpClient::new();

            // Initialize the database
            let mut db = rt
                .block_on(lancedb::connect(&db_uri).execute())
                .context("Failed to connect to the database")?;

            // Query the database
            let content = rt
                .block_on(lancevectordb::query::run_query(
                    &mut db,
                    embed_model,
                    &input_list,
                    &vector_table,
                    &http_client,
                    whole_query,
                    file_context,
                ))
                .context("Failed to run query")?;

            debug!("Query Response: {:?}", content);
        }
        Commands::RagQuery {
            input,
            model,
            ai_model,
            table,
            database,
            whole_query,
            file_context,
            system_prompt,
        } => {
            let input_list = rt
                .block_on(Commands::fetch_prompt_from_cli(
                    input.clone(),
                    "Enter query: ",
                ))
                .with_context(|| format!("Failed to fetch prompt from CLI: {:?}", input))?;

            info!("Using the RagQuery arguments below:");
            info!(" Query: {:?}", input_list);

            let embed_model = model.to_string();
            let vector_table = table.to_string();
            let db_uri = database.to_string();
            let whole_query: bool = whole_query
                .parse()
                .context("Failed to parse whole_query flag")?;
            let file_context: bool = file_context
                .parse()
                .context("Failed to parse file_query flag")?;
            let system_prompt = system_prompt.as_str();

            // Initialize the http client outside the thread // TODO wrap in Arc<Mutex>
            let http_client: HttpClient<HttpConnector> = HttpClient::new();
            // do a check to see if client is up

            // Initialize the database
            let mut db = rt
                .block_on(lancedb::connect(&db_uri).execute())
                .context("Failed to connect to the database")?;

            // Query the database
            let content = rt
                .block_on(lancevectordb::query::run_query(
                    &mut db,
                    embed_model,
                    &input_list,
                    &vector_table,
                    &http_client,
                    whole_query,
                    file_context,
                ))
                .context("Failed to run query")?;

            debug!("Query Response: {:?}", content);

            // start a spinner @TODO Fix this
            let pb = cli_spinner().context("Failed to create spinner")?;
            pb.set_message("Generating...");

            let context = content.join(" ");
            // @ TODO: make this a command line argument
            // let system_prompt = "template/rag_prompt.txt";
            // let system_prompt = "template/software-engineer.txt";
            // let system_prompt = "template/spark_prompt.txt";
            // let system_prompt = "template/spark-engineer.txt";
            rt.block_on(crate::chat::run_chat_with_history(
                system_prompt,
                input_list.first().unwrap(),
                Some(&context),
                &http_client,
                &ai_model,
                &pb,
            ))
            .context("Failed to run chat")?;

            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        Commands::Generate { prompt, ai_model } => {
            // let prompt = Commands::fetch_prompt_from_cli(Vec::new(), "Enter prompt: ");
            // println!("Chat command is run with below arguments:");
            // println!(" Prompt: {:?}", prompt);
            // println!(" AI Model: {:?}", ai_model);

            let context: Option<&str> = None;
            let client = HttpClient::new();

            // start a spinner @TODO Fix this
            let pb = cli_spinner().context("Failed to create spinner")?;
            pb.set_message("Generating...");

            let system_prompt = "template/general_prompt.txt";
            rt.block_on(crate::chat::run_chat(
                system_prompt,
                &prompt,
                context,
                &client,
                &ai_model,
            ))
            .context("Failed to run chat")?;

            pb.finish_with_message("End of Response!");
            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        Commands::Version { version } => {
            info!("Version: {}", version);
        }
    }

    Ok(())
}

async fn check_connection(url: &str) -> Result<()> {
    let client = HttpClient::new();
    // let uri = hyper::Uri::from_static(&url);
    let uri = url.parse::<hyper::Uri>()?;

    let res = client.get(uri).await?;
    if res.status().is_success() {
        if let Some(info) = res.extensions().get::<HttpInfo>() {
            info!("remote addr = {}", info.remote_addr())
        }
    } else {
        anyhow::bail!(anyhow::anyhow!("Failed to connect to the server {}", url));
    }

    Ok(())
}

pub fn cli_spinner() -> Result<ProgressBar> {
    // let spin_chars = ["🐒", "🐵", "🙈", "🙉", "🙊"];
    let bar_chars = [
        "⠀", "⠁", "⠂", "⠃", "⠄", "⠅", "⠆", "⠇", "⡀", "⡁", "⡂", "⡃", "⡄", "⡅", "⡆", "⡇", "⠈", "⠉",
        "⠊", "⠋", "⠌", "⠍", "⠎", "⠏", "⡈", "⡉", "⡊", "⡋", "⡌", "⡍", "⡎", "⡏", "⠐", "⠑", "⠒", "⠓",
        "⠔", "⠕", "⠖", "⠗", "⡐", "⡑", "⡒", "⡓", "⡔", "⡕", "⡖", "⡗", "⠘", "⠙", "⠚", "⠛", "⠜", "⠝",
        "⠞", "⠟", "⡘", "⡙", "⡚", "⡛", "⡜", "⡝", "⡞", "⡟", "⠠", "⠡", "⠢", "⠣", "⠤", "⠥", "⠦", "⠧",
        "⡠", "⡡", "⡢", "⡣", "⡤", "⡥", "⡦", "⡧", "⠨", "⠩", "⠪", "⠫", "⠬", "⠭", "⠮", "⠯", "⡨", "⡩",
        "⡪", "⡫", "⡬", "⡭", "⡮", "⡯", "⠰", "⠱", "⠲", "⠳", "⠴", "⠵", "⠶", "⠷", "⡰", "⡱", "⡲", "⡳",
        "⡴", "⡵", "⡶", "⡷", "⠸", "⠹", "⠺", "⠻", "⠼", "⠽", "⠾", "⠿", "⡸", "⡹", "⡺", "⡻", "⡼", "⡽",
        "⡾", "⡿", "⢀", "⢁", "⢂", "⢃", "⢄", "⢅", "⢆", "⢇", "⣀", "⣁", "⣂", "⣃", "⣄", "⣅", "⣆", "⣇",
        "⢈", "⢉", "⢊", "⢋", "⢌", "⢍", "⢎", "⢏", "⣈", "⣉", "⣊", "⣋", "⣌", "⣍", "⣎", "⣏", "⢐", "⢑",
        "⢒", "⢓", "⢔", "⢕", "⢖", "⢗", "⣐", "⣑", "⣒", "⣓", "⣔", "⣕", "⣖", "⣗", "⢘", "⢙", "⢚", "⢛",
        "⢜", "⢝", "⢞", "⢟", "⣘", "⣙", "⣚", "⣛", "⣜", "⣝", "⣞", "⣟", "⢠", "⢡", "⢢", "⢣", "⢤", "⢥",
        "⢦", "⢧", "⣠", "⣡", "⣢", "⣣", "⣤", "⣥", "⣦", "⣧", "⢨", "⢩", "⢪", "⢫", "⢬", "⢭", "⢮", "⢯",
        "⣨", "⣩", "⣪", "⣫", "⣬", "⣭", "⣮", "⣯", "⢰", "⢱", "⢲", "⢳", "⢴", "⢵", "⢶", "⢷", "⣰", "⣱",
        "⣲", "⣳", "⣴", "⣵", "⣶", "⣷", "⢸", "⢹", "⢺", "⢻", "⢼", "⢽", "⢾", "⢿", "⣸", "⣹", "⣺", "⣻",
        "⣼", "⣽", "⣾", "⣿",
    ];

    let pb = ProgressBar::new(1024);

    let style = ProgressStyle::default_spinner()
        .tick_strings(&bar_chars)
        .template("{spinner} {red} {msg} ")
        .with_context(|| "Failed to create progress style")?;
    pb.set_style(style);
    pb.enable_steady_tick(Duration::from_millis(40));

    // pb.tick();
    // pb.inc(1);

    Ok(pb)
}
