use anyhow::Result;
use anyhow::{anyhow, Context};
use hyper::client::HttpConnector;
use hyper::{body, Client, Uri};
use hyper::{Body, Request};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, RwLock};

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

impl ChatMessage {
    pub fn new(role: ChatRole, content: String) -> ChatMessage {
        ChatMessage { role, content }
    }

    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    pub fn print_chat(&self) {
        println!("{:?}: {}", &self.role, &self.content);
    }
}

/// ChatRequest is a struct that represents a chat request
// @TODO: Add tools
#[derive(Serialize, Deserialize)]
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
    options: Option<Options>,
}

impl ChatRequest {
    pub(crate) fn new(
        model: String,
        api_url: String,
        api_key: String,
        stream: bool,
        format: String,
        options: Option<Options>,
        prompt: Prompt,
    ) -> ChatRequest {
        let mut messages = Vec::<ChatMessage>::new();

        messages.push(ChatMessage::new(ChatRole::System, prompt.system_message));
        messages.push(ChatMessage::new(
            ChatRole::User,
            prompt.content.unwrap_or_else(|| "".to_string()),
        ));
        messages.push(ChatMessage::new(ChatRole::User, prompt.prompt));

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
    let chat_request = match chat_request.read() {
        Ok(data) => data,
        Err(e) => {
            error!("Error: {}", e);
            return Err(anyhow!("Error: {}", e));
        }
    };

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
    debug!("Response: {:?}", response.status());

    // Parse the response body
    let body = body::to_bytes(response).await?;
    debug!("Response body: {:?}", body.len());

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
    pub fn print_message(&self) {
        let message: &ChatMessage = &self.message;
        println!("{:?}", message.print_chat());
    }

    #[allow(dead_code)]
    pub fn pretty_print_message(&self) {
        let message: &ChatMessage = &self.message;

        // Assuming `print_content()` returns a `&str` that is a JSON string
        if let Ok(parsed_json) = serde_json::from_str::<Value>(&message.to_string()) {
            if let Ok(pretty_json) = serde_json::to_string_pretty(&parsed_json) {
                println!("{}", pretty_json);
            } else {
                eprintln!("Failed to pretty print the JSON.");
            }
        } else {
            eprintln!("Failed to parse JSON string.");
        }
    }
}

/// Options is a struct that represents the options for the chat request
// @TODO: Add options support
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Options {
    num_keep: i32,
    seed: i32,
    num_predict: i32,
    top_k: i32,
    top_p: f32,
    min_p: f32,
    typical_p: f32,
    repeat_last_n: i32,
    temperature: f32,
    repeat_penalty: f32,
    presence_penalty: f32,
    frequency_penalty: f32,
    mirostat: i32,
    mirostat_tau: f32,
    mirostat_eta: f32,
    penalize_newline: bool,
    stop: Vec<String>,
    numa: bool,
    num_ctx: i32,
    num_batch: i32,
    num_gpu: i32,
    main_gpu: i32,
    low_vram: bool,
    vocab_only: bool,
    use_mmap: bool,
    use_mlock: bool,
    num_thread: i32,
}

#[allow(dead_code)]
impl Options {
    fn new(
        num_keep: i32,
        seed: i32,
        num_predict: i32,
        top_k: i32,
        top_p: f32,
        min_p: f32,
        typical_p: f32,
        repeat_last_n: i32,
        temperature: f32,
        repeat_penalty: f32,
        presence_penalty: f32,
        frequency_penalty: f32,
        mirostat: i32,
        mirostat_tau: f32,
        mirostat_eta: f32,
        penalize_newline: bool,
        stop: Vec<String>,
        numa: bool,
        num_ctx: i32,
        num_batch: i32,
        num_gpu: i32,
        main_gpu: i32,
        low_vram: bool,
        vocab_only: bool,
        use_mmap: bool,
        use_mlock: bool,
        num_thread: i32,
    ) -> Options {
        Options {
            num_keep,
            seed,
            num_predict,
            top_k,
            top_p,
            min_p,
            typical_p,
            repeat_last_n,
            temperature,
            repeat_penalty,
            presence_penalty,
            frequency_penalty,
            mirostat,
            mirostat_tau,
            mirostat_eta,
            penalize_newline,
            stop,
            numa,
            num_ctx,
            num_batch,
            num_gpu,
            main_gpu,
            low_vram,
            vocab_only,
            use_mmap,
            use_mlock,
            num_thread,
        }
    }

    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
