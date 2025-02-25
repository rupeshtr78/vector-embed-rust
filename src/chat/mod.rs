use crate::app::constants::{
    AI_MODEL, CHAT_API_KEY, CHAT_API_URL, CHAT_RESPONSE_FORMAT, SYSTEM_PROMPT_PATH,
};
use crate::chat::chat_config::ai_chat;
use anyhow::Context;
use chat_config::ChatResponse;
use hyper::client::HttpConnector;
use hyper::Client;
use log::{debug, error, info};
use std::process::exit;
use std::sync::Arc;
use tokio::sync::RwLock;

mod chat_config;
mod prompt_template;

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
) -> anyhow::Result<ChatResponse> {
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

    let req2 = chat_request.clone();

    // Create a new Arc<RwLock<ChatRequest>> to share the request between threads
    let request = Arc::new(RwLock::new(chat_request));

    // Call the AI chat API
    let response = ai_chat(&request, client)
        .await
        .context("Failed to get ai chat response")?;

    // debug!("{:?}", response);

    response.print_message();

    let ai_message = response.get_message();

    // @TODO: Implement the logic in loop wait for use input
    let res = req2.request_with_history(&ai_message, ai_prompt);

    res.messages.iter().for_each(|m| {
        debug!("Request2 ...{:?}", m);
    });

    Ok(response)
}
