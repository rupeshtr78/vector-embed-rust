mod chat_config;
mod prompt_template;

use crate::chat_config::ai_chat;
use anyhow::{Context, Result};
use env_logger::Env;
use hyper::Client as HttpClient;
use log::{debug, error, info};
use std::{
    process::exit,
    sync::{Arc, RwLock},
};

const CHAT_API_URL: &str = "http://10.0.0.213:11434/api/generate";
const CHAT_API_KEY: &str = "api_key";

const CHAT_RESPONSE_FORMAT: &str = "json";
const SYSTEM_PROMPT_PATH : &str = "/Users/rupeshraghavan/apl/gits/gits-rupesh/rtr-rust-lab/multi-workspace/ai-chat/template/system_prompt.txt";
const PROMPT_TEMPLATE_PATH : &str = "/Users/rupeshraghavan/apl/gits/gits-rupesh/rtr-rust-lab/multi-workspace/ai-chat/template/chat_template.hbs";

const AI_MODEL: &str = "mistral:latest";

async fn run_chat() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    info!("Starting chat application...");

    let paths = [SYSTEM_PROMPT_PATH, PROMPT_TEMPLATE_PATH];
    paths.iter().for_each(|path| {
        if std::fs::metadata(path).is_err() || !std::fs::metadata(path).unwrap().is_file() {
            error!("File does not exist: {}", path);
            exit(1);
        }
    });

    let content = std::fs::read_to_string("/Users/rupeshraghavan/apl/gits/gits-rupesh/rtr-rust-lab/multi-workspace/ai-chat/template/sample.txt")
        .context("Failed to read system prompt")?;

    let prompt =
        prompt_template::Prompt::new(&SYSTEM_PROMPT_PATH, Some(&content), "What is mirostat_eta?")
            .await
            .context("Failed to create prompt")?;

    let template = prompt_template::get_template(&prompt, PROMPT_TEMPLATE_PATH)
        .context("Failed to get template")?;

    let chat_request = chat_config::ChatRequest::new(
        AI_MODEL.to_string(),
        CHAT_API_URL.to_string(),
        CHAT_API_KEY.to_string(),
        template,
        false,
        CHAT_RESPONSE_FORMAT.to_string(),
        None,
    );

    let chat_body = chat_request.create_chat_body()?;

    debug!("Chay Body {:?}", chat_body);

    let client = HttpClient::new();

    let request = Arc::new(RwLock::new(chat_request));

    let response = ai_chat(&request, &client)
        .await
        .context("Failed to get response")?;

    // debug!("{:?}", response);

    response.print_response();

    Ok(())
}
