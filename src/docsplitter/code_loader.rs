use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::path::PathBuf;
use text_splitter::{ChunkConfig, CodeSplitter, CodeSplitterError};
use tree_sitter_language::LanguageFn;
use crate::app::config::EmbedRequest;
use crate::app::constants::EMBEDDING_MODEL;

pub struct FileChunk {
    content: Vec<String>,
    file_path: PathBuf,
    chunk_number: usize,
}

/// A struct that represents a codebase.
impl FileChunk {
    fn new(content: String, file_path: PathBuf, chunk_number: usize) -> Self {
        let content_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Self {
            content: content_lines,
            file_path: file_path,
            chunk_number: chunk_number,
        }
    }

    pub fn get_content(&self) -> String {
        self.content.join("\n")
    }

    pub fn get_file_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    pub fn get_chunk_number(&self) -> usize {
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

    pub fn embed_request(&self) -> EmbedRequest {
        EmbedRequest {
            model: EMBEDDING_MODEL.to_string(),
            input: self.content.clone(),
            metadata: Some(self.file_path.file_name().unwrap().to_str().unwrap().to_string()),
        }
    }

    pub fn embed_request_arc(&self) -> std::sync::Arc<std::sync::RwLock<EmbedRequest>> {
        std::sync::Arc::new(std::sync::RwLock::new(self.embed_request()))
    }
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

    let chunk_config = ChunkConfig::new(max_chunk_size)
        .with_overlap(256)
        .context("Failed to create chunk config")?;

    let is_code = is_code_file(file_path);

    log::info!("File: {:?} Extension {:}", is_code.0, is_code.1);

    // let mut chunks: Vec<FileChunk> = Vec::new();
    if is_code.1 == false && is_code.0 == Some("txt") {
        // user tree_sitter_markdown
        let splitter = text_splitter::TextSplitter::new(chunk_config);
        let chunks = splitter
            .chunks(&content)
            .enumerate()
            .map(|(i, chunk)| Ok(FileChunk::new(chunk.to_string(), file_path.clone(), i)))
            .collect::<Result<Vec<FileChunk>, CodeSplitterError>>()?;

        return Ok(chunks);
    }

    if is_code.1 == true {
        let splitter = CodeSplitter::new(
            get_language_from_file_extension(is_code.0).context("Unsupported file extension")?,
            chunk_config,
        )
            .context("Failed to create code splitter")?;

        let code_chunks = splitter.chunks(&content);

        let chunks: Vec<FileChunk> = code_chunks
            .enumerate()
            .map(|(i, chunk)| Ok(FileChunk::new(chunk.to_string(), file_path.clone(), i)))
            .collect::<Result<Vec<FileChunk>, CodeSplitterError>>()?;

        return Ok(chunks);
    }

    Ok(vec![])
}

/// Check if a file is a code file based on its extension.
fn is_code_file(file_path: &Path) -> (Option<&str>, bool) {
    // Add your own logic to determine if the file is a code file
    let ext = file_path.extension().and_then(|e| e.to_str());

    let is_code = match ext {
        Some("rs" | "py" | "cpp" | "java" | "js" | "ts" | "tsx" | "c" | "h" | "go" | "scala") => {
            true
        }
        _ => false,
    };

    (ext, is_code)
}

fn get_language_from_file_extension(ext: Option<&str>) -> Result<LanguageFn> {
    // let ext: Option<&str> = file_path.extension().and_then(|e| e.to_str());
    let language = match ext {
        Some("rs") => tree_sitter_rust::LANGUAGE,
        Some("py") => tree_sitter_python::LANGUAGE,
        Some("cpp") => tree_sitter_cpp::LANGUAGE,
        Some("java") => tree_sitter_java::LANGUAGE,
        Some("js") => tree_sitter_javascript::LANGUAGE,
        Some("ts") => tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
        Some("tsx") => tree_sitter_typescript::LANGUAGE_TSX,
        Some("c") => tree_sitter_c::LANGUAGE,
        Some("h") => tree_sitter_c::LANGUAGE,
        Some("go") => tree_sitter_go::LANGUAGE,
        Some("scala") => tree_sitter_scala::LANGUAGE,
        _ => return Err(anyhow!("Unsupported file extension")),
    };

    Ok(language)
}
