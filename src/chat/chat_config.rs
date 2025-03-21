use crate::chat::model_options::Options;
use anyhow::Result;
use anyhow::{anyhow, Context};
use hyper::client::HttpConnector;
use hyper::{body, Client, Uri};
use hyper::{Body, Request};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::chat::prompt_template::Prompt;

/// ChatRole is an enum that represents the role of the chat message
#[derive(Serialize, Deserialize, Debug, Clone)]

pub enum ChatRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "tool")]
    Tool,
}

/// ChatMessage is a struct that represents a chat message
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    role: ChatRole,
    content: String,
}

impl fmt::Display for ChatMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: {}", self.role, self.content)
    }
}

impl ChatMessage {
    pub fn new(role: ChatRole, content: String) -> ChatMessage {
        ChatMessage { role, content }
    }

    pub fn get_content(&self) -> &String {
        &self.content
    }

    #[allow(dead_code)]
    pub fn pretty_print_chat(&self) {
        let content = self.get_content();
        if let Ok(parsed_json) = serde_json::from_str::<Value>(content) {
            if let Ok(pretty_json) = serde_json::to_string_pretty(&parsed_json) {
                println!("AI Response {}", pretty_json);
            } else {
                eprintln!("Failed to pretty print the JSON.");
            }
        } else {
            eprintln!("Failed to parse JSON string.");
        }
    }
}

/// ChatRequest is a struct that represents a chat request
// @TODO: Add tools
#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct ChatRequest {
    pub model: String,
    pub api_url: String,
    pub api_key: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    pub format: String,
    pub options: Option<Options>,
}

/// ChatBody is a struct that represents the body of a chat request
#[derive(Serialize, Deserialize)]
struct ChatBody {
    model: String,
    pub messages: Vec<ChatMessage>,
    stream: bool,
    format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<Options>,
}

impl ChatRequest {
    pub(crate) fn new(
        model: &str,
        api_url: String,
        api_key: String,
        stream: bool,
        format: String,
        options: Option<Options>,
        prompt: Prompt,
    ) -> ChatRequest {
        let mut messages = Vec::new();
        let system_message = ChatMessage::new(ChatRole::System, prompt.system_message);
        messages.push(system_message);

        for content in prompt.content.clone().into_iter().flatten() {
            let user_content = ChatMessage::new(ChatRole::User, content.get_content().to_string());
            messages.push(user_content);
        }

        let user_prompt = ChatMessage::new(ChatRole::User, prompt.prompt);
        messages.push(user_prompt);

        let model = model.to_string();
        ChatRequest {
            model,
            api_url,
            api_key,
            messages,
            stream,
            format,
            options,
        }
    }

    pub(crate) fn create_chat_body(&self) -> Result<String> {
        let chat_body = ChatBody {
            model: self.model.to_string(),
            messages: self.messages.clone(),
            stream: self.stream,
            format: self.format.to_string(),
            options: self.options.clone(),
        };

        let body = serde_json::to_string(&chat_body).context("Failed to serialize ChatBody")?;
        debug!("Chat Body: {:?}", body);

        Ok(body)
    }
}

/// Get chat response from the AI model
/// # Arguments
/// * `chat_request` - The chat request to send to the AI model
/// * `http_client` - The HTTP client to use for the request
/// # Returns
/// * `Result<ChatResponse>` - The result of the chat response
pub async fn ai_chat(
    chat_request: &Arc<RwLock<ChatRequest>>,
    http_client: &Client<HttpConnector>,
) -> Result<ChatResponse> {
    let chat_request = chat_request.read().await;

    let url = chat_request
        .api_url
        .parse::<Uri>()
        .context("Failed to parse URL")?;

    // Serialize the data to a JSON string, handling potential errors
    let chat_body = chat_request.create_chat_body()?;
    let request_body = Body::from(chat_body);

    let request = Request::builder()
        .method("POST")
        .uri(url)
        .header("Content-Type", "application/json")
        .body(request_body)
        .context("Failed to build request")?;

    // Send the request and await the response.
    let response = http_client.request(request).await?;
    if response.status() != 200 {
        return Err(anyhow!("Failed to get response: {}", response.status()));
    }
    debug!("Chat Response Status: {:?}", response.status());

    // Parse the response body
    let body = body::to_bytes(response).await?;
    // debug!("Response body: {:?}", body.len());

    let response_body: ChatResponse =
        serde_json::from_slice(&body).context("Failed to parse response")?;

    Ok(response_body)
}

/// ChatResponse is a struct that represents a chat response
#[derive(Serialize, Deserialize, Debug)]
pub struct ChatResponse {
    model: String,
    created_at: String,
    message: ChatMessage,
    done_reason: Option<String>,
    done: bool,
    context: Option<Vec<i32>>,
    total_duration: Option<i64>,
    load_duration: Option<i64>,
    prompt_eval_count: Option<i32>,
    prompt_eval_duration: Option<i64>,
    eval_count: Option<i32>,
    eval_duration: Option<i64>,
}

impl ChatResponse {
    pub fn get_message(&self) -> Option<&ChatMessage> {
        Some(&self.message)
    }
}
