// UnoOne Power — Desktop Document Processing
// Document parsers, encrypted index, and memory retrieval

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

/// Document processor state
pub struct DocumentProcessor {
    vault_root: String,
}

impl DocumentProcessor {
    pub fn new(vault_root: &str) -> Self {
        Self {
            vault_root: vault_root.to_string(),
        }
    }

    /// List all documents in the vault
    pub fn list_documents(&self) -> Vec<DocumentMetadata> {
        let docs_dir = PathBuf::from(&self.vault_root)
            .join("VAULT")
            .join("documents");

        let mut documents = Vec::new();

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

                    documents.push(DocumentMetadata {
                        id: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                        title: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                        document_type: doc_type,
                        file_path: path.to_string_lossy().to_string(),
                        file_size_bytes: file_size,
                        created_at: String::new(),
                        modified_at: String::new(),
                        source_platform: "DESKTOP".to_string(),
                        tags: Vec::new(),
                        page_count: None,
                        word_count: None,
                    });
                }
            }
        }

        documents
    }

    /// Process a document — extract text, generate summary
    /// In production, this would use actual parsers (pdf, docx, etc.)
    pub fn process_document(&self, document_id: &str) -> DocumentProcessResult {
        let start = std::time::Instant::now();

        // TODO: Implement actual document parsing
        // - PDF: pdf-extract or poppler
        // - DOCX: docx-rs or similar
        // - TXT/MD: direct read
        // - CSV/XLSX: parse and extract
        // - Image: OCR via Tesseract
        // - Audio: STT via Whisper

        DocumentProcessResult {
            document_id: document_id.to_string(),
            success: true,
            extracted_text: Some("[Document text extraction placeholder]".to_string()),
            summary: Some("[LLM-generated summary placeholder]".to_string()),
            error: None,
            processing_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Search memories by query
    /// In production, this would use encrypted vector embeddings
    pub fn search_memories(&self, query: &MemorySearchQuery) -> Vec<MemorySearchResult> {
        // TODO: Implement actual encrypted vector search
        // - Use sentence-transformers for embeddings
        // - Store embeddings in encrypted index on vault
        // - Cosine similarity search against query embedding

        vec![MemorySearchResult {
            id: "mock-result-1".to_string(),
            memory_type: "PERSONAL".to_string(),
            title: format!("Results for: {}", query.query),
            preview: "This is a placeholder search result. Actual search will use encrypted vector embeddings.".to_string(),
            relevance: 0.85,
            created_at: chrono::Utc::now().to_rfc3339(),
        }]
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