// UnoOne Power — Desktop Document Processing
// Lists real documents from vault; text extraction requires real parsers (not yet wired)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Supported document types (mirrors core-contracts Document.kt)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentType {
    Pdf,
    Docx,
    Txt,
    Markdown,
    Csv,
    Xlsx,
    Pptx,
    Image,
    Audio,
    WebPage,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub id: String,
    pub title: String,
    pub document_type: DocumentType,
    pub file_path: String,
    pub file_size_bytes: u64,
    pub created_at: String,
    pub modified_at: String,
    pub source_platform: String,
    pub tags: Vec<String>,
    pub page_count: Option<u32>,
    pub word_count: Option<u32>,
}

/// Document processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentProcessResult {
    pub document_id: String,
    pub success: bool,
    pub extracted_text: Option<String>,
    pub summary: Option<String>,
    pub error: Option<String>,
    pub processing_time_ms: u64,
}

/// Memory search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchQuery {
    pub query: String,
    pub memory_types: Vec<String>,
    pub limit: u32,
    pub min_relevance: f32,
}

/// Memory search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    pub id: String,
    pub memory_type: String,
    pub title: String,
    pub preview: String,
    pub relevance: f32,
    pub created_at: String,
}

/// Document processor — reads real files from vault
pub struct DocumentProcessor {
    vault_root: String,
}

impl DocumentProcessor {
    pub fn new(vault_root: &str) -> Self {
        Self {
            vault_root: vault_root.to_string(),
        }
    }

    /// List all documents in the vault directory
    pub fn list_documents(&self) -> Vec<DocumentMetadata> {
        let docs_dir = PathBuf::from(&self.vault_root)
            .join("VAULT")
            .join("documents");

        let mut documents = Vec::new();

        if !docs_dir.exists() {
            return documents;
        }

        if let Ok(entries) = std::fs::read_dir(&docs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    let doc_type = match ext.to_string_lossy().to_lowercase().as_str() {
                        "pdf" => DocumentType::Pdf,
                        "docx" | "doc" => DocumentType::Docx,
                        "txt" => DocumentType::Txt,
                        "md" | "markdown" => DocumentType::Markdown,
                        "csv" => DocumentType::Csv,
                        "xlsx" | "xls" => DocumentType::Xlsx,
                        "pptx" | "ppt" => DocumentType::Pptx,
                        "png" | "jpg" | "jpeg" | "gif" | "webp" => DocumentType::Image,
                        "mp3" | "wav" | "ogg" | "flac" => DocumentType::Audio,
                        "html" | "htm" => DocumentType::WebPage,
                        _ => continue,
                    };

                    let file_size = std::fs::metadata(&path)
                        .map(|m| m.len())
                        .unwrap_or(0);

                    let created = std::fs::metadata(&path)
                        .ok()
                        .and_then(|m| m.created().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs().to_string())
                        .unwrap_or_default();

                    let modified = std::fs::metadata(&path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs().to_string())
                        .unwrap_or_default();

                    // For text files, count words
                    let word_count = match doc_type {
                        DocumentType::Txt | DocumentType::Markdown => {
                            std::fs::read_to_string(&path)
                                .map(|s| s.split_whitespace().count() as u32)
                                .ok()
                        }
                        _ => None,
                    };

                    documents.push(DocumentMetadata {
                        id: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                        title: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                        document_type: doc_type,
                        file_path: path.to_string_lossy().to_string(),
                        file_size_bytes: file_size,
                        created_at: created,
                        modified_at: modified,
                        source_platform: "DESKTOP".to_string(),
                        tags: Vec::new(),
                        page_count: None,
                        word_count,
                    });
                }
            }
        }

        documents
    }

    /// Process a document — extract text from supported formats
    /// TXT and Markdown are fully supported; other formats require parsers
    pub fn process_document(&self, document_id: &str) -> DocumentProcessResult {
        let start = std::time::Instant::now();

        let docs_dir = PathBuf::from(&self.vault_root)
            .join("VAULT")
            .join("documents");

        // Find the document by ID (filename stem)
        if let Ok(entries) = std::fs::read_dir(&docs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(stem) = path.file_stem() {
                    if stem.to_string_lossy() == document_id {
                        if let Some(ext) = path.extension() {
                            let ext_str = ext.to_string_lossy().to_lowercase();
                            match ext_str.as_str() {
                                "txt" | "md" | "markdown" => {
                                    // Text formats — read directly
                                    match std::fs::read_to_string(&path) {
                                        Ok(text) => {
                                            let word_count = text.split_whitespace().count();
                                            let preview = if text.len() > 500 {
                                                format!("{}...[truncated, {} words total]", &text[..500], word_count)
                                            } else {
                                                text.clone()
                                            };
                                            return DocumentProcessResult {
                                                document_id: document_id.to_string(),
                                                success: true,
                                                extracted_text: Some(text),
                                                summary: Some(format!("[Auto-extracted text, {} words]", word_count)),
                                                error: None,
                                                processing_time_ms: start.elapsed().as_millis() as u64,
                                            };
                                        }
                                        Err(e) => {
                                            return DocumentProcessResult {
                                                document_id: document_id.to_string(),
                                                success: false,
                                                extracted_text: None,
                                                summary: None,
                                                error: Some(format!("Failed to read file: {}", e)),
                                                processing_time_ms: start.elapsed().as_millis() as u64,
                                            };
                                        }
                                    }
                                }
                                _ => {
                                    // Binary formats — parser not yet integrated
                                    return DocumentProcessResult {
                                        document_id: document_id.to_string(),
                                        success: false,
                                        extracted_text: None,
                                        summary: None,
                                        error: Some(format!(
                                            "Document format .{} is not yet supported. Supported: .txt, .md, .markdown",
                                            ext_str
                                        )),
                                        processing_time_ms: start.elapsed().as_millis() as u64,
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }

        DocumentProcessResult {
            document_id: document_id.to_string(),
            success: false,
            extracted_text: None,
            summary: None,
            error: Some(format!("Document '{}' not found in vault", document_id)),
            processing_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Search memories — reads from vault memory directory
    /// Returns matching memory files; full vector search not yet implemented
    pub fn search_memories(&self, query: &MemorySearchQuery) -> Vec<MemorySearchResult> {
        let memory_dir = PathBuf::from(&self.vault_root)
            .join("VAULT")
            .join("memory");

        let mut results = Vec::new();

        if !memory_dir.exists() {
            return results;
        }

        // Simple text-based search through memory files
        if let Ok(entries) = std::fs::read_dir(&memory_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json" || e == "txt" || e == "md").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let query_lower = query.query.to_lowercase();
                        if query.query == "*" || content.to_lowercase().contains(&query_lower) {
                            let file_stem = path.file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();

                            let preview = if content.len() > 200 {
                                format!("{}...", &content[..200])
                            } else {
                                content.clone()
                            };

                            results.push(MemorySearchResult {
                                id: file_stem.clone(),
                                memory_type: path.extension().unwrap_or_default().to_string_lossy().to_string(),
                                title: file_stem,
                                preview,
                                relevance: if query.query == "*" { 1.0 } else { 0.5 },
                                created_at: std::fs::metadata(&path)
                                    .ok()
                                    .and_then(|m| m.modified().ok())
                                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                    .map(|d| d.as_secs().to_string())
                                    .unwrap_or_default(),
                            });

                            if results.len() >= query.limit as usize {
                                break;
                            }
                        }
                    }
                }
            }
        }

        results
    }
}

// Tauri commands

#[tauri::command]
pub fn list_documents(vault_root: String) -> Vec<DocumentMetadata> {
    let processor = DocumentProcessor::new(&vault_root);
    processor.list_documents()
}

#[tauri::command]
pub fn process_document(document_id: String, vault_root: String) -> DocumentProcessResult {
    let processor = DocumentProcessor::new(&vault_root);
    processor.process_document(&document_id)
}

#[tauri::command]
pub fn search_memories(query: MemorySearchQuery, vault_root: String) -> Vec<MemorySearchResult> {
    let processor = DocumentProcessor::new(&vault_root);
    processor.search_memories(&query)
}