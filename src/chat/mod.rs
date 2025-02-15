use hyper::Client;
use hyper::client::HttpConnector;
use log::{debug, error, info};
use std::process::exit;
use std::sync::{Arc};
use tokio::sync::RwLock;
use anyhow::Context;
use crate::chat::chat_config::ai_chat;

mod chat_config;
mod prompt_template;

const CHAT_API_URL: &str = "http://10.0.0.213:11434";
const CHAT_API_KEY: &str = "api_key";
const CHAT_RESPONSE_FORMAT: &str = "json";
const SYSTEM_PROMPT_PATH: &str = "template/system_prompt.txt";
const AI_MODEL: &str = "mistral:latest";

// const PROMPT_TEMPLATE_PATH: &str = "template/chat_template.hbs";

/// Run the chatbot
/// # Arguments
/// * `ai_prompt` - The prompt to send to the AI model
/// * `context` - The context to send to the AI model
/// # Returns
/// * `Result<()>` - The result of the chatbot
pub async fn run_chat(
    ai_prompt: &str,
    context: Option<&str>,
    client: &Client<HttpConnector>,
) -> anyhow::Result<()> {
    info!("Starting LLM chat...");

    let paths = [SYSTEM_PROMPT_PATH];
    paths.iter().for_each(|path| {
        if std::fs::metadata(path).is_err() || !std::fs::metadata(path).unwrap().is_file() {
            error!("File does not exist: {}", path);
            exit(1);
        }
    });

    let prompt = prompt_template::Prompt::new(SYSTEM_PROMPT_PATH, context, ai_prompt)
        .await
        .context("Failed to create prompt")?;

    // @TODO: Implement the template
    // let template = prompt_template::get_template(&prompt, PROMPT_TEMPLATE_PATH)
    //     .context("Failed to get template")?;

    let chat_url = format!("{}/{}", CHAT_API_URL, "api/chat");

    let chat_request = chat_config::ChatRequest::new(
        AI_MODEL.to_string(),
        chat_url,
        CHAT_API_KEY.to_string(),
        false,
        CHAT_RESPONSE_FORMAT.to_string(),
        None,
        prompt,
    );

    let chat_body = chat_request.create_chat_body()?;
    debug!("Chat Body {:?}", chat_body);

    // Create a new HTTP client
    // let client = HttpClient::new();

    // Create a new Arc<RwLock<ChatRequest>> to share the request between threads
    let request = Arc::new(RwLock::new(chat_request));

    // Call the AI chat API
    let response = ai_chat(&request, client)
        .await
        .context("Failed to get ai chat response")?;

    // debug!("{:?}", response);

    response.print_message();

    Ok(())
}