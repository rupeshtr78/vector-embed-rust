use std::io::Write;
use crate::app::constants::{
    AI_MODEL, CHAT_API_KEY, CHAT_API_URL, CHAT_RESPONSE_FORMAT, SYSTEM_PROMPT_PATH,
};
use crate::chat::chat_config::{ai_chat, ChatMessage};
use anyhow::Context;
use chat_config::ChatResponse;
use hyper::client::HttpConnector;
use hyper::Client;
use log::{debug, error, info};
use std::process::exit;
use std::sync::Arc;
use serde_json::Value;
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
    
    let cm= ChatMessage::new(chat_config::ChatRole::User, context.map(|s| s.to_string()).unwrap_or("".to_string()));
    let contexts = vec![Some(cm)];

    let prompt = prompt_template::Prompt::new(SYSTEM_PROMPT_PATH, &contexts, ai_prompt)
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


    // Create a new Arc<RwLock<ChatRequest>> to share the request between threads
    let request = Arc::new(RwLock::new(chat_request));

    // Call the AI chat API
    let response = ai_chat(&request, client)
        .await
        .context("Failed to get ai chat response")?;

    response.print_message();

    Ok(response)
}

/// Run the chatbot with history
/// # Arguments
/// * `initial_prompt` - The initial prompt to start the chat
/// * `context` - The context to send to the AI model
/// * `client` - The HTTP client to use for requests
/// # Returns
/// * `Result<()>` - The result of the chatbot
pub async fn run_chat_with_history(
    initial_prompt: &str,
    context: Option<&str>,
    client: &Client<HttpConnector>,
) -> anyhow::Result<()> {
    info!("Starting LLM chat with history...");

    let paths = [SYSTEM_PROMPT_PATH];
    paths.iter().for_each(|path| {
        if std::fs::metadata(path).is_err() || !std::fs::metadata(path).unwrap().is_file() {
            error!("File does not exist: {}", path);
            exit(1);
        }
    });
    
    let mut history = Vec::new();
    let query_content = ChatMessage::new(chat_config::ChatRole::User, context.map(|s| s.to_string()).unwrap_or("".to_string()));
    history.push(Some(query_content));
    let mut current_prompt = initial_prompt.to_string();

    loop {
        let prompt = prompt_template::Prompt::new(SYSTEM_PROMPT_PATH, &history, &current_prompt)
            .await
            .context("Failed to create prompt")?;

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
        
        debug!("Chat Content with history: {:?}", chat_request);

        let request = Arc::new(RwLock::new(chat_request));

        let response = ai_chat(&request, client)
            .await
            .context("Failed to get AI chat response")?;

        // response.print_message();

        let ai_message = response.get_message();
        if let Some(message) = ai_message {
            let content = message.get_content();
            let chat_history = ChatMessage::new(chat_config::ChatRole::User, content.to_string());
            history.push(Some(chat_history));
            
            // Parse the JSON string into a serde_json::Value
            let json_value: Value = serde_json::from_str(content).context("Failed to parse JSON")?;

            // Pretty-print the JSON with indentation
            let pretty_json = serde_json::to_string_pretty(&json_value).context("Failed to pretty print JSON")?;

            println!("AI Response: {}", pretty_json);
        } else {
            println!("AI Response: None");
        }
        

        // Prompt the user for the next input @TODO: Fix this is not printing the prompt
        print!("Ask Followup: ");
        std::io::stdout().flush().expect("Failed to flush stdout");
        let mut user_input = String::new();
        std::io::stdin().read_line(&mut user_input).expect("Failed to read line");
        current_prompt = user_input.trim().to_string();

        if current_prompt.to_lowercase() == "exit" {
            break;
        }
    }

    Ok(())
}