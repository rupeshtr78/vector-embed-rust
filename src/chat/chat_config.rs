use ::serde_json::de;
use anyhow::Result;
use anyhow::{anyhow, Context};
use hyper::client::HttpConnector;
use hyper::{body, Client, Uri};
use hyper::{Body, Request};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ChatRequest {
    model: String,
    api_url: String,
    api_key: String,
    prompt: String,
    stream: bool,
    format: String,
    options: Option<Options>,
}
#[derive(Serialize, Deserialize)]
struct ChatBody {
    model: String,
    prompt: String,
    stream: bool,
    format: String,
    options: Option<Options>,
}

impl ChatRequest {
    pub(crate) fn new(
        model: String,
        api_url: String,
        api_key: String,
        prompt: String,
        stream: bool,
        format: String,
        options: Option<Options>,
    ) -> ChatRequest {
        ChatRequest {
            model,
            api_url,
            api_key,
            prompt,
            stream,
            format,
            options,
        }
    }

    pub(crate) fn create_chat_body(&self) -> Result<String> {
        let chat_body = ChatBody {
            model: self.model.to_string(),
            prompt: self.prompt.to_string(),
            stream: self.stream,
            format: self.format.to_string(),
            options: self.options.clone(),
        };

        let body = serde_json::to_string(&chat_body).context("Failed to serialize ChatBody")?;

        Ok(body)
    }
}

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

    // get raw string from bytes and print for debugging
    // let body_str = String::from_utf8(body.to_vec()).unwrap();
    // debug!("Response body: {:?}", body_str);

    let response_body: ChatResponse =
        serde_json::from_slice(&body).context("Failed to parse response")?;

    // response.print_response();

    Ok(response_body)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatResponse {
    model: String,
    created_at: String,
    response: String,
    done: bool,
    done_reason: Option<String>,
    context: Option<Vec<i32>>,
    total_duration: i64,
    load_duration: i64,
    prompt_eval_count: i32,
    prompt_eval_duration: i64,
    eval_count: i32,
    eval_duration: i64,
}

impl ChatResponse {
    fn get_response(&self) -> String {
        self.response.to_string()
    }

    fn get_context(&self) -> Vec<i32> {
        self.context.clone().unwrap()
    }

    fn get_model(&self) -> String {
        self.model.to_string()
    }

    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn print_response(&self) {
        println!("{}", self.response);
    }
    
}

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
