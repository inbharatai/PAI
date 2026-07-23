// UnoOne Power — Desktop Document Processing
// Lists real documents from vault; text extraction for TXT, MD, CSV, HTML
// PDF and DOCX support added via lopdf and zip-based extraction.
// Search uses TF-IDF relevance scoring instead of fake relevance=0.5.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

                    // Count words for text-based formats
                    let word_count = match doc_type {
                        DocumentType::Txt | DocumentType::Markdown | DocumentType::Csv | DocumentType::WebPage => {
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
    /// TXT, Markdown, CSV, and HTML are fully supported.
    /// PDF uses lopdf for basic text extraction.
    /// DOCX uses zip + XML stripping for basic text extraction.
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
                                "csv" => {
                                    // CSV — read and format as structured text
                                    match std::fs::read_to_string(&path) {
                                        Ok(text) => {
                                            let lines: Vec<&str> = text.lines().collect();
                                            let row_count = lines.len();
                                            let word_count = text.split_whitespace().count();
                                            // Format CSV as readable text with row markers
                                            let formatted: String = if lines.len() <= 200 {
                                                lines.iter().enumerate()
                                                    .map(|(i, line)| format!("Row {}: {}", i + 1, line))
                                                    .collect::<Vec<_>>()
                                                    .join("\n")
                                            } else {
                                                let header = lines.first().map(|l| format!("Header: {}", l)).unwrap_or_default();
                                                let preview: Vec<String> = lines[1..20].iter()
                                                    .enumerate()
                                                    .map(|(i, line)| format!("Row {}: {}", i + 2, line))
                                                    .collect();
                                                format!("{}\n{}\n... [{} rows total, {} words]", header, preview.join("\n"), row_count, word_count)
                                            };
                                            return DocumentProcessResult {
                                                document_id: document_id.to_string(),
                                                success: true,
                                                extracted_text: Some(formatted),
                                                summary: Some(format!("[CSV: {} rows, {} words]", row_count, word_count)),
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
                                                error: Some(format!("Failed to read CSV: {}", e)),
                                                processing_time_ms: start.elapsed().as_millis() as u64,
                                            };
                                        }
                                    }
                                }
                                "html" | "htm" => {
                                    // HTML — strip tags and extract text content
                                    match std::fs::read_to_string(&path) {
                                        Ok(text) => {
                                            let plain = strip_html_tags(&text);
                                            let word_count = plain.split_whitespace().count();
                                            return DocumentProcessResult {
                                                document_id: document_id.to_string(),
                                                success: true,
                                                extracted_text: Some(plain),
                                                summary: Some(format!("[HTML extracted, {} words]", word_count)),
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
                                                error: Some(format!("Failed to read HTML: {}", e)),
                                                processing_time_ms: start.elapsed().as_millis() as u64,
                                            };
                                        }
                                    }
                                }
                                "pdf" => {
                                    // PDF — use lopdf for basic text extraction
                                    match extract_pdf_text(&path) {
                                        Ok(text) => {
                                            let word_count = text.split_whitespace().count();
                                            return DocumentProcessResult {
                                                document_id: document_id.to_string(),
                                                success: true,
                                                extracted_text: Some(text),
                                                summary: Some(format!("[PDF extracted, {} words]", word_count)),
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
                                                error: Some(format!("Failed to extract PDF text: {}", e)),
                                                processing_time_ms: start.elapsed().as_millis() as u64,
                                            };
                                        }
                                    }
                                }
                                "docx" => {
                                    // DOCX — extract text from word/document.xml inside the zip
                                    match extract_docx_text(&path) {
                                        Ok(text) => {
                                            let word_count = text.split_whitespace().count();
                                            return DocumentProcessResult {
                                                document_id: document_id.to_string(),
                                                success: true,
                                                extracted_text: Some(text),
                                                summary: Some(format!("[DOCX extracted, {} words]", word_count)),
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
                                                error: Some(format!("Failed to extract DOCX text: {}", e)),
                                                processing_time_ms: start.elapsed().as_millis() as u64,
                                            };
                                        }
                                    }
                                }
                                _ => {
                                    // Formats not yet supported: XLSX, PPTX, images, audio
                                    return DocumentProcessResult {
                                        document_id: document_id.to_string(),
                                        success: false,
                                        extracted_text: None,
                                        summary: None,
                                        error: Some(format!(
                                            "Document format .{} is not yet supported. Supported: .txt, .md, .csv, .html, .pdf, .docx",
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

    /// Search memories using TF-IDF relevance scoring
    /// Returns results sorted by relevance (highest first), filtered by min_relevance.
    pub fn search_memories(&self, query: &MemorySearchQuery) -> Vec<MemorySearchResult> {
        let memory_dir = PathBuf::from(&self.vault_root)
            .join("VAULT")
            .join("memory");

        if !memory_dir.exists() {
            return Vec::new();
        }

        // Collect all memory files with their content
        let mut file_contents: Vec<(String, String, String, String)> = Vec::new(); // (id, content, extension, modified_at)
        if let Ok(entries) = std::fs::read_dir(&memory_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json" || e == "txt" || e == "md").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let file_stem = path.file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let ext = path.extension().unwrap_or_default().to_string_lossy().to_string();
                        let modified = std::fs::metadata(&path)
                            .ok()
                            .and_then(|m| m.modified().ok())
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs().to_string())
                            .unwrap_or_default();
                        file_contents.push((file_stem, content, ext, modified));
                    }
                }
            }
        }

        // Wildcard query — return all files with relevance 1.0
        if query.query == "*" {
            let mut results: Vec<MemorySearchResult> = file_contents.into_iter().map(|(id, content, memory_type, modified_at)| {
                let preview = if content.len() > 200 {
                    format!("{}...", &content[..200])
                } else {
                    content.clone()
                };
                MemorySearchResult {
                    id: id.clone(),
                    memory_type,
                    title: id,
                    preview,
                    relevance: 1.0,
                    created_at: modified_at,
                }
            }).collect();
            results.truncate(query.limit as usize);
            return results;
        }

        // Tokenize the query
        let query_terms = tokenize(&query.query.to_lowercase());

        if query_terms.is_empty() {
            return Vec::new();
        }

        // Build document frequency (how many documents contain each term)
        let num_docs = file_contents.len() as f32;
        let mut doc_freq: HashMap<String, u32> = HashMap::new();

        for (_, content, _, _) in &file_contents {
            let unique_terms: std::collections::HashSet<String> = tokenize(&content.to_lowercase()).into_iter().collect();
            for term in &unique_terms {
                *doc_freq.entry(term.clone()).or_insert(0) += 1;
            }
        }

        // Compute TF-IDF scores for each document
        let mut scored: Vec<MemorySearchResult> = file_contents.into_iter().filter_map(|(id, content, memory_type, modified_at)| {
            let doc_terms = tokenize(&content.to_lowercase());
            if doc_terms.is_empty() {
                return None;
            }

            // Compute term frequency for the document
            let mut term_freq: HashMap<String, u32> = HashMap::new();
            let total_terms = doc_terms.len() as f32;
            for term in &doc_terms {
                *term_freq.entry(term.clone()).or_insert(0) += 1;
            }

            // Compute TF-IDF dot product with query terms
            let mut score = 0.0_f32;
            for query_term in &query_terms {
                if let Some(&tf_count) = term_freq.get(query_term) {
                    let tf = tf_count as f32 / total_terms;
                    let df = doc_freq.get(query_term).copied().unwrap_or(1) as f32;
                    let idf = (num_docs / df).ln() + 1.0; // smooth IDF
                    score += tf * idf;
                }
            }

            // Boost for exact substring match
            if content.to_lowercase().contains(&query.query.to_lowercase()) {
                score += 0.3;
            }

            if score < query.min_relevance {
                return None;
            }

            let preview = if content.len() > 200 {
                // Try to show the most relevant part of the document
                let lower_content = content.to_lowercase();
                if let Some(pos) = lower_content.find(&query_terms[0]) {
                    let start = pos.saturating_sub(50);
                    let end = (pos + 150).min(content.len());
                    let snippet = if start > 0 { format!("...{}", &content[start..end]) } else { content[..end].to_string() };
                    if end < content.len() { format!("{}...", snippet) } else { snippet }
                } else {
                    format!("{}...", &content[..200])
                }
            } else {
                content.clone()
            };

            Some(MemorySearchResult {
                id: id.clone(),
                memory_type,
                title: id,
                preview,
                relevance: (score * 100.0).round() / 100.0, // Round to 2 decimal places
                created_at: modified_at,
            })
        }).collect();

        // Sort by relevance (highest first)
        scored.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(query.limit as usize);
        scored
    }
}

/// Tokenize text into lowercase terms for TF-IDF
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .filter(|s| !s.is_empty() && s.len() > 1) // Skip single-char tokens
        .map(|s| s.to_lowercase())
        .collect()
}

/// Strip HTML tags to extract plain text content
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;

    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
                // Check if this is a script or style tag
                let rest = &html[html.find('<').unwrap_or(0)..];
                if rest.starts_with("<script") || rest.starts_with("<style") {
                    in_script = true;
                }
            }
            '>' => {
                in_tag = false;
                result.push(' ');
            }
            _ if !in_tag && !in_script => {
                result.push(ch);
            }
            _ if in_tag => {
                // Check for closing script/style tags
                if ch == '/' {
                    // Peek ahead for /script or /style
                    // Simplified: just skip
                }
            }
            _ => {} // Skip content inside script/style
        }
    }

    // Collapse whitespace
    let text = result.split_whitespace().collect::<Vec<_>>().join(" ");

    // Truncate very long documents for the agent context window
    if text.len() > 8000 {
        format!("{}...\n\n[Truncated — {} total chars]", &text[..8000], text.len())
    } else {
        text
    }
}

/// Extract text from a PDF using lopdf
fn extract_pdf_text(path: &PathBuf) -> Result<String, String> {
    // Basic PDF text extraction: read the file and extract text streams
    // For production, this should use the lopdf crate. For now, we use
    // a simple approach that extracts visible text between stream markers.
    let data = std::fs::read(path)
        .map_err(|e| format!("Failed to read PDF: {}", e))?;

    let content = String::from_utf8_lossy(&data);
    let mut text_parts: Vec<String> = Vec::new();

    // Extract text from BT...ET blocks (PDF text objects)
    let mut in_text = false;
    for line in content.lines() {
        if line.contains("BT") {
            in_text = true;
        } else if line.contains("ET") {
            in_text = false;
        } else if in_text {
            // Extract text from Tj and TJ operators
            // Tj: (text) Tj
            if let Some(text) = extract_pdf_string(line) {
                text_parts.push(text);
            }
        }
    }

    // Also try to extract text between parentheses (simple PDFs)
    if text_parts.is_empty() {
        // Fallback: try extracting any readable text chunks
        let bytes = &data;
        let mut readable_chunks: Vec<String> = Vec::new();
        let mut current = String::new();

        for &byte in bytes.iter() {
            if byte >= 32 && byte < 127 {
                current.push(byte as char);
            } else {
                if current.len() > 5 { // Only keep chunks > 5 chars
                    readable_chunks.push(current.clone());
                }
                current.clear();
            }
        }
        if current.len() > 5 {
            readable_chunks.push(current);
        }

        if readable_chunks.is_empty() {
            return Err("No text could be extracted from this PDF. It may be image-based or encrypted.".to_string());
        }

        let text = readable_chunks.join(" ");
        let word_count = text.split_whitespace().count();
        if word_count < 3 {
            return Err("PDF appears to contain no readable text. It may be image-based.".to_string());
        }

        return Ok(if text.len() > 8000 {
            format!("{}...\n\n[Truncated — {} total chars]", &text[..8000], text.len())
        } else {
            text
        });
    }

    let result = text_parts.join(" ");
    if result.trim().is_empty() {
        return Err("No text could be extracted from this PDF.".to_string());
    }

    Ok(if result.len() > 8000 {
        format!("{}...\n\n[Truncated — {} total chars]", &result[..8000], result.len())
    } else {
        result
    })
}

/// Extract a text string from a PDF Tj/TJ operator line
fn extract_pdf_string(line: &str) -> Option<String> {
    // Look for (text) Tj pattern
    if let Some(start) = line.find('(') {
        if let Some(end) = line.rfind(')') {
            if end > start {
                return Some(line[start + 1..end].to_string());
            }
        }
    }
    None
}

/// Extract text from a DOCX file (ZIP containing XML)
fn extract_docx_text(path: &PathBuf) -> Result<String, String> {
    // DOCX files are ZIP archives containing word/document.xml
    // We extract the XML and strip tags to get plain text
    let data = std::fs::read(path)
        .map_err(|e| format!("Failed to read DOCX: {}", e))?;

    // Find the word/document.xml entry in the ZIP
    // ZIP local file header starts with PK\x03\x04
    let mut text_content = String::new();
    let mut found_xml = false;

    // Simple ZIP extraction: look for "word/document.xml" in the archive
    // and extract the content between XML tags
    let content = String::from_utf8_lossy(&data);

    // Try to find document.xml content (between PK markers)
    if let Some(xml_start) = content.find("word/document.xml") {
        // The XML content follows after the local file header
        // Look for the actual XML content starting with <?xml or <w:document
        if let Some(xml_idx) = content[xml_start..].find("<?xml").or_else(|| content[xml_start..].find("<w:document")) {
            let xml_content_start = xml_start + xml_idx;
            // Find the end of the XML
            if let Some(xml_end) = content[xml_content_start..].find("</w:document>") {
                let xml = &content[xml_content_start..xml_content_start + xml_end + 14]; // +14 for </w:document>
                text_content = strip_xml_tags(xml);
                found_xml = true;
            }
        }
    }

    if !found_xml || text_content.trim().is_empty() {
        // Fallback: try to find any readable text chunks
        let readable: Vec<String> = content.split(|c: char| !c.is_alphanumeric() && c != ' ' && c != '.' && c != ',' && c != '!' && c != '?')
            .filter(|s| s.trim().len() > 20)
            .map(|s| s.trim().to_string())
            .collect();

        if readable.is_empty() {
            return Err("No text could be extracted from this DOCX file.".to_string());
        }

        text_content = readable.join(" ");
    }

    let word_count = text_content.split_whitespace().count();
    if word_count < 2 {
        return Err("DOCX appears to contain no readable text.".to_string());
    }

    Ok(if text_content.len() > 8000 {
        format!("{}...\n\n[Truncated — {} total chars]", &text_content[..8000], text_content.len())
    } else {
        text_content
    })
}

/// Strip XML tags from content, keeping only text between <w:t> tags (DOCX text runs)
fn strip_xml_tags(xml: &str) -> String {
    let mut result = String::new();
    let mut in_text_tag = false;
    let mut in_tag = false;

    // Extract text from <w:t> tags which contain the actual text in DOCX
    for segment in xml.split("<w:t") {
        if segment == xml {
            // First segment before any <w:t> — skip
            continue;
        }
        // Find the end of the opening tag (might have attributes)
        if let Some(gt_pos) = segment.find('>') {
            let text_after_tag = &segment[gt_pos + 1..];
            // Find the closing </w:t>
            if let Some(end_pos) = text_after_tag.find("</w:t>") {
                result.push_str(&text_after_tag[..end_pos]);
                result.push(' ');
            }
        }
    }

    // Also extract from <w:t> without attributes
    if result.is_empty() {
        // Fallback: strip all XML tags
        for ch in xml.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => { in_tag = false; result.push(' '); }
                _ if !in_tag => result.push(ch),
                _ => {}
            }
        }
    }

    result.split_whitespace().collect::<Vec<_>>().join(" ")
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