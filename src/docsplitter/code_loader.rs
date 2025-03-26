use crate::embedder::config::EmbedRequest;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use log::debug;
use std::cmp::PartialEq;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use text_splitter::{Characters, ChunkConfig, CodeSplitter, CodeSplitterError};
use tokio::sync::RwLock;
use tree_sitter_language::LanguageFn;

#[derive(Debug, PartialEq)]
enum Language {
    Rust,
    Python,
    Cpp,
    Java,
    JavaScript,
    TypeScript,
    Tsx,
    C,
    Header,
    Go,
    Scala,
    Text,
    SPARKLOG,
    UNKNOWN,
}

impl Language {
    fn from_str(s: &str) -> Self {
        match s {
            "rs" => Language::Rust,
            "py" => Language::Python,
            "cpp" => Language::Cpp,
            "java" => Language::Java,
            "js" => Language::JavaScript,
            "ts" => Language::TypeScript,
            "tsx" => Language::Tsx,
            "c" => Language::C,
            "h" => Language::Header,
            "go" => Language::Go,
            "scala" => Language::Scala,
            "txt" => Language::Text,
            "log" => Language::SPARKLOG,
            // "sh" => Language::Text,
            "unknown" => Language::UNKNOWN,
            _ => Language::UNKNOWN,
        }
    }
}

pub struct FileChunk {
    content: Vec<String>,
    file_path: PathBuf,
    chunk_number: i32,
}

/// A struct that represents a codebase.
impl FileChunk {
    fn new(content: String, file_path: PathBuf, chunk_number: i32) -> Self {
        let content_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Self {
            content: content_lines,
            file_path,
            chunk_number,
        }
    }

    pub fn get_content(&self) -> String {
        self.content.join("\n")
    }

    pub fn get_file_path(&self) -> &PathBuf {
        &self.file_path
    }

    pub fn get_chunk_number(&self) -> i32 {
        self.chunk_number
    }

    pub fn print_file_chunk(&self) {
        println!(
            "File: {}, Chunk {}: {}",
            self.file_path.display(),
            self.chunk_number,
            self.content.join("\n")
        );
    }

    pub fn get_file_name(&self) -> String {
        self.file_path.file_name().map_or_else(
            || "None".to_string(),
            |s| s.to_str().unwrap_or("None").to_string(),
        )
    }
}

pub fn chunk_embed_request(
    chunk: &FileChunk,
    provider: &str,
    api_url: &str,
    api_key: &str,
    model: &str,
) -> EmbedRequest {
    EmbedRequest {
        provider: provider.to_string(),
        api_url: api_url.to_string(),
        api_key: api_key.to_string(),
        model: model.to_string(),
        input: chunk.content.clone(),
        metadata: Some(
            chunk
                .file_path
                .file_name()
                .unwrap_or(OsStr::new("None"))
                .to_str()
                .unwrap_or("None")
                .to_string(),
        ),
        chunk_number: Some(chunk.chunk_number),
    }
}

pub fn chunk_embed_request_arc(
    chunk: &FileChunk,
    provider: &str,
    api_url: &str,
    api_key: &str,
    model: &str,
) -> Arc<RwLock<EmbedRequest>> {
    Arc::new(RwLock::new(chunk_embed_request(
        chunk, provider, api_url, api_key, model,
    )))
}

/// Load a codebase into chunks of text.
pub async fn load_codebase_into_chunks(
    root_dir: &str,
    max_chunk_size: usize,
) -> Result<Vec<FileChunk>> {
    let root_path = PathBuf::from(root_dir);

    if root_path.is_file() {
        let chunk = split_file_into_chunks(&root_path, max_chunk_size)
            .await
            .context("Failed to split file into chunks")?;
        return Ok(chunk);
    }

    if root_path.is_dir() {
        return process_directory(&root_path, max_chunk_size)
            .await
            .context("Failed to process directory");
    }

    Err(anyhow!(
        "The path provided is neither a file nor a directory"
    ))
}

/// process_directory recursively processes a directory and its subdirectories to extract code chunks.
async fn process_directory(path: &PathBuf, max_chunk_size: usize) -> Result<Vec<FileChunk>> {
    async fn inner_process_directory(
        path: &PathBuf,
        max_chunk_size: usize,
        chunks: &mut Vec<FileChunk>,
    ) -> Result<()> {
        let mut entries = tokio::fs::read_dir(path)
            .await
            .context(format!("Failed to read directory {}", path.display()))?;

        // let is_code = is_code_file(path);

        while let Some(entry) = entries.next_entry().await? {
            let file_path = entry.path();
            debug!("File Path: {:?}", file_path);
            let file_type = entry.file_type().await?;
            if file_type.is_file() {
                let chunk = split_file_into_chunks(&file_path, max_chunk_size).await?;
                chunks.extend(chunk);
            } else if file_type.is_dir() {
                Box::pin(inner_process_directory(&file_path, max_chunk_size, chunks)).await?;
            }
        }
        Ok(())
    }

    let mut chunks = Vec::new();
    inner_process_directory(path, max_chunk_size, &mut chunks).await?;
    Ok(chunks)
}

/// Split a file into chunks of text based on language-specific rules.
async fn split_file_into_chunks(
    file_path: &PathBuf,
    max_chunk_size: usize,
) -> Result<Vec<FileChunk>> {
    let file = File::open(file_path).context("Failed to open file")?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    // Create a chunk config with the specified max chunk size
    let chunk_config = ChunkConfig::new(max_chunk_size)
        .with_overlap(256)
        .context("Failed to create chunk config")?
        .with_trim(true);

    let (language, is_supported) = is_supported_file(file_path);

    debug!(
        "File Extension: {:?} Is Supported File {:}",
        language, is_supported
    );

    if !is_supported {
        debug!("Unsupported file extension");
    }

    // let mut chunks: Vec<FileChunk> = Vec::new();
    if is_supported && language == Language::Text {
        // user tree_sitter_markdown
        let splitter = text_splitter::TextSplitter::new(chunk_config);
        let chunks = splitter
            .chunks(&content)
            .enumerate()
            .map(|(i, chunk)| {
                Ok(FileChunk::new(
                    chunk.to_string(),
                    file_path.clone(),
                    i as i32,
                ))
            })
            .collect::<Result<Vec<FileChunk>, CodeSplitterError>>()?;

        return Ok(chunks);
    }

    if is_supported && language != Language::Text && language != Language::SPARKLOG {
        let splitter = CodeSplitter::new(
            get_language_from_file_extension(language).context("Unsupported file extension")?,
            chunk_config,
        )
        .context("Failed to create code splitter")?;

        let code_chunks = splitter.chunks(&content);

        let chunks: Vec<FileChunk> = code_chunks
            .enumerate()
            .map(|(i, chunk)| {
                Ok(FileChunk::new(
                    chunk.to_string(),
                    file_path.clone(),
                    i as i32,
                ))
            })
            .collect::<Result<Vec<FileChunk>, CodeSplitterError>>()?;

        return Ok(chunks);
    }

    if is_supported && language == Language::SPARKLOG {
        return process_spark_log_file(file_path, chunk_config).await;
    }

    Ok(vec![])
}

/// Checks if the given file is a supported code file based on its extension.
///
/// # Arguments
///
/// * `file_path` - A reference to the path of the file to check.
///
/// # Returns
///
/// A tuple where the first element is the detected `Language` enum type,
/// and the second element is a boolean indicating if the file is supported.
fn is_supported_file(file_path: &Path) -> (Language, bool) {
    // Extracts the file extension and converts it to a string slice
    let ext = file_path.extension().and_then(|e| e.to_str());

    if let Some(ext_str) = ext {
        debug!("File Extension: {}", ext_str);

        match Language::from_str(ext_str) {
            Language::UNKNOWN => (Language::UNKNOWN, false),
            lang => (lang, true),
        }
    } else {
        debug!("No valid extension found.");
        (Language::UNKNOWN, false)
    }
}

fn get_language_from_file_extension(language: Language) -> Result<LanguageFn> {
    let language = match language {
        Language::Rust => tree_sitter_rust::LANGUAGE,
        Language::Python => tree_sitter_python::LANGUAGE,
        Language::Cpp => tree_sitter_cpp::LANGUAGE,
        Language::Java => tree_sitter_java::LANGUAGE,
        Language::JavaScript => tree_sitter_javascript::LANGUAGE,
        Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
        Language::Tsx => tree_sitter_typescript::LANGUAGE_TSX,
        Language::C => tree_sitter_c::LANGUAGE,
        Language::Go => tree_sitter_go::LANGUAGE,
        Language::Scala => tree_sitter_scala::LANGUAGE,
        Language::UNKNOWN => return Err(anyhow!("Unsupported file extension")),
        _ => return Err(anyhow!("Unsupported file extension")),
    };

    Ok(language)
}

async fn process_spark_log_file(
    file_path: &PathBuf,
    chunk_config: ChunkConfig<Characters>,
) -> Result<Vec<FileChunk>> {
    let file = File::open(file_path).context("Failed to open file")?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    // remove lines without error or exception
    let error_lines = capture_context_lines(&content, 20);

    let splitter = text_splitter::TextSplitter::new(chunk_config);
    let chunks = splitter
        .chunks(&error_lines)
        .enumerate()
        .map(|(i, chunk)| {
            Ok(FileChunk::new(
                chunk.to_string(),
                file_path.clone(),
                i as i32,
            ))
        })
        .collect::<Result<Vec<FileChunk>, CodeSplitterError>>()?;

    Ok(chunks)
}

fn capture_context_lines(content: &str, num_lines: usize) -> String {
    let filtered_lines: Vec<&str> = content
        .lines()
        .filter(|line| !line.to_lowercase().contains("lineage"))
        .collect();

    let mut result = Vec::new();

    for (i, line) in filtered_lines.iter().enumerate() {
        // Check for "ERROR" or "EXCEPTION" after filtering out "LINEAGE"
        if line.to_lowercase().contains("error") || line.to_lowercase().contains("exception") {
            let start = if i >= num_lines { i - num_lines } else { 0 };
            let end = usize::min(i + num_lines + 1, filtered_lines.len());

            for context_line in &filtered_lines[start..end] {
                if !context_line.trim().is_empty() {
                    result.push(*context_line);
                }
            }
        }
    }

    debug!("Error Lines: {}", result.join("\n"));

    result.join("\n")
}
