use crate::chat::chat_config::ChatMessage;
use anyhow::Context;
use anyhow::Result;
use handlebars::Handlebars;
use serde::Serialize;

/// Prompt struct
#[derive(Serialize, Debug)]
pub(crate) struct Prompt {
    pub(crate) system_message: String,
    pub content: Vec<Option<ChatMessage>>,
    pub prompt: String,
}

impl Prompt {
    pub(crate) async fn new(
        path: &str,
        contents: &Vec<Option<ChatMessage>>,
        prompt: &str,
    ) -> Result<Prompt> {
        let system_prompt = get_system_prompt(path)
            .await
            .context("Failed to get system prompt")?;
        let prompt = Prompt {
            system_message: system_prompt,
            content: contents.clone(),
            prompt: prompt.to_string(),
        };
        Ok(prompt)
    }
}

/// Get system prompt from file
/// # Arguments
/// * `prompt_path` - Path to the system prompt file
/// # Returns
/// * `Result<String>` - System prompt
async fn get_system_prompt(prompt_path: &str) -> Result<String> {
    let path = std::path::Path::new(prompt_path);

    if std::fs::metadata(path).is_err() || !std::fs::metadata(path).unwrap().is_file() {
        anyhow::bail!("System prompt file not found: {}", path.display());
    }

    let system_prompt = tokio::fs::read_to_string(path)
        .await
        .context("Failed to read system prompt")?;

    Ok(system_prompt)
}

/// Get template from file
/// # Arguments
/// * `prompt` - Prompt struct
/// * `template_file` - Path to the template file
/// # Returns
/// * `Result<String>` - Rendered template
#[allow(dead_code)]
pub fn get_template(prompt: &Prompt, template_file: &str) -> Result<String> {
    let template = std::fs::read_to_string(template_file).expect("Failed to read template file");

    let mut handlebars = Handlebars::new();
    //
    // let template = r#"
    // <|system|> {{ system_prompt }}</s>
    // <|content|> {{ content }}</s>
    // <|user|> {{ prompt }}</s>
    // "#;

    handlebars
        .register_template_string("tpl", template)
        .context("Failed to register template")?;

    let data = Prompt {
        system_message: prompt.system_message.to_string(),
        content: prompt.content.clone(),
        prompt: prompt.prompt.to_string(),
    };

    // Render the template with the data
    let rendered = handlebars
        .render("tpl", &data)
        .context("Failed to render template")?;
    // debug!("Rendered Template {}", rendered);

    Ok(rendered)
}
