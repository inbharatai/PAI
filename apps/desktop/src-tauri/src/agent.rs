// UnoOne Power — Desktop ReAct Agent Loop
// D2: Implements the agentic reasoning loop on the backend.
// D3: Tool implementations now read from the live vault state instead of stubs.
// Model → parse tool calls → safety guard → execute tools → observe → loop.

use crate::documents;
use crate::llama::{ConversationTurn, InferenceRequest, ModelManagerState, ToolDefinition};
use crate::safety::{SafetyGuardState, ToolAction};
use crate::security;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

const MAX_AGENT_STEPS: u32 = 5;

/// A single step in the agent's reasoning process.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentStep {
    Thinking { text: String },
    ToolCall { tool: String, args: serde_json::Value, confidence: f32 },
    ToolResult { tool: String, result: String, approved: bool },
    SafetyBlock { tool: String, reason: String },
    FinalResponse { text: String },
}

/// The result of running the agentic loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub steps: Vec<AgentStep>,
    pub final_text: String,
    pub iterations: u32,
}

/// D3: Tool implementations that use real vault state.
/// Each tool reads from the live DesktopVaultState or document processor.
struct ToolExecutor<'a> {
    vault_root: &'a str,
}

impl<'a> ToolExecutor<'a> {
    fn new(vault_root: &'a str) -> Self {
        Self { vault_root }
    }

    /// Search notes and documents in the vault
    fn search_notes(&self, query: &str, limit: Option<u64>) -> String {
        let search_query = documents::MemorySearchQuery {
            query: query.to_string(),
            memory_types: vec!["note".to_string(), "document".to_string(), "memory".to_string()],
            limit: limit.unwrap_or(10) as u32,
            min_relevance: 0.1,
        };

        let results = documents::search_memories(search_query, self.vault_root.to_string());

        if results.is_empty() {
            format!("No results for '{}'.", query)
        } else {
            let mut output = format!("Found {} result(s):\n", results.len());
            for result in &results {
                output.push_str(&format!(
                    "- {} [{}] {:.0}% relevant\n  {}\n",
                    result.title,
                    result.memory_type,
                    result.relevance * 100.0,
                    result.preview
                ));
            }
            output
        }
    }

    /// List all documents in the vault
    fn list_documents(&self) -> String {
        let docs = documents::list_documents(self.vault_root.to_string());

        if docs.is_empty() {
            "No documents in the vault.".to_string()
        } else {
            let mut output = format!("{} document(s):\n", docs.len());
            for doc in &docs {
                let type_tag = match doc.document_type {
                    documents::DocumentType::Txt => "TXT",
                    documents::DocumentType::Markdown => "MD",
                    documents::DocumentType::Pdf => "PDF",
                    documents::DocumentType::Docx => "DOCX",
                    documents::DocumentType::Csv => "CSV",
                    documents::DocumentType::Xlsx => "XLSX",
                    documents::DocumentType::Pptx => "PPTX",
                    documents::DocumentType::Image => "IMG",
                    documents::DocumentType::Audio => "AUDIO",
                    documents::DocumentType::WebPage => "WEB",
                };
                output.push_str(&format!("- {} [{}] {} bytes\n", doc.title, type_tag, doc.file_size_bytes));
                if let Some(wc) = doc.word_count {
                    output.push_str(&format!("  {} words\n", wc));
                }
            }
            output
        }
    }

    /// Read a specific document from the vault
    fn read_document(&self, document_id: &str) -> String {
        let result = documents::process_document(document_id.to_string(), self.vault_root.to_string());

        if result.success {
            let text = result.extracted_text.unwrap_or_default();
            // Truncate very long documents for the agent context window
            if text.len() > 4000 {
                format!("{}...\n\n[Truncated — {} total chars]", &text[..4000], text.len())
            } else {
                text
            }
        } else {
            let error = result.error.unwrap_or_default();
            if error.contains("not yet supported") {
                format!("Cannot read '{}' — binary format. Only .txt and .md are supported.", document_id)
            } else {
                format!("Cannot read '{}': {}", document_id, error)
            }
        }
    }

    /// Verify vault manifest integrity
    fn verify_vault(&self) -> String {
        match security::verify_manifest(self.vault_root.to_string()) {
            Ok(result) => {
                if result.manifest_valid && result.hmac_valid {
                    format!("Vault OK — {} files verified, HMAC valid.", result.entries_verified)
                } else {
                    let mut output = format!(
                        "Vault check failed: {}/{} files failed.",
                        result.entries_failed, result.total_entries
                    );
                    if !result.hmac_valid {
                        output.push_str(" HMAC signature INVALID.");
                    }
                    for err in &result.errors {
                        output.push_str(&format!("\n  - {}", err));
                    }
                    output
                }
            }
            Err(e) => format!("Cannot verify vault: {}", e),
        }
    }
}

/// D2: The agent loop state, held as Tauri managed state.
/// Contains the safety guard for tool review and the max steps limit.
pub struct AgentLoopState {
    pub max_steps: u32,
}

impl AgentLoopState {
    pub fn new() -> Self {
        Self { max_steps: MAX_AGENT_STEPS }
    }
}

/// System prompt for the agentic loop.
/// Clean and direct — like Gemini/ChatGPT: identity first, tool rules second.
fn get_system_prompt() -> String {
    "You are UnoOne, a private AI assistant. You run entirely on the user's encrypted USB vault — no data leaves the device.\n\
     \n\
     Tools: search_notes, list_documents, read_document, verify_vault.\n\
     - Use tools when you need information from the vault to answer a question.\n\
     - Answer directly from your knowledge when tools aren't needed.\n\
     - If a tool call is blocked, explain briefly and try an alternative.\n\
     - Never reveal internal tool mechanics to the user — respond naturally."
        .to_string()
}

/// D3: Tool definitions available to the model.
/// Concise descriptions — models work best with brief, clear tool specs.
fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "search_notes".to_string(),
            description: "Search notes and documents in the vault by keyword.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search keywords" },
                    "limit": { "type": "integer", "description": "Max results (default 10)" }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "list_documents".to_string(),
            description: "List all documents stored in the vault.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "read_document".to_string(),
            description: "Read a document's contents from the vault.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "document_id": { "type": "string", "description": "Document filename or ID" }
                },
                "required": ["document_id"]
            }),
        },
        ToolDefinition {
            name: "verify_vault".to_string(),
            description: "Verify vault integrity — checks file hashes and HMAC signatures.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
    ]
}

/// D3: Execute a single tool call using real vault state.
/// Returns the tool result as a string.
async fn execute_tool(tool_name: &str, args: &serde_json::Value, vault_root: &str) -> String {
    let executor = ToolExecutor::new(vault_root);

    match tool_name {
        "search_notes" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let limit = args.get("limit").and_then(|v| v.as_u64());
            executor.search_notes(query, limit)
        }
        "list_documents" => {
            executor.list_documents()
        }
        "read_document" => {
            let doc_id = args.get("document_id").and_then(|v| v.as_str()).unwrap_or("");
            executor.read_document(doc_id)
        }
        "verify_vault" => {
            executor.verify_vault()
        }
        _ => format!("Unknown tool: {}", tool_name),
    }
}

/// D2/D3: Run the full ReAct agent loop for a user message.
///
/// The loop:
/// 1. Send user message + system prompt + tools to the model via llama-server
/// 2. If model responds with tool calls: parse each → safety review → execute → observe → loop
/// 3. If model responds with plain text (no tool calls): return as final answer
/// 4. Maximum iterations: MAX_AGENT_STEPS
#[tauri::command]
pub async fn agent_chat(
    message: String,
    conversation_history: Vec<ConversationTurn>,
    model_state: tauri::State<'_, ModelManagerState>,
    safety_state: tauri::State<'_, SafetyGuardState>,
    agent_state: tauri::State<'_, AgentLoopState>,
    vault_state: tauri::State<'_, crate::DesktopVaultState>,
) -> Result<AgentResult, String> {
    let mut steps = Vec::new();
    let mut history = conversation_history;

    // D3: Get vault root from state for real tool execution
    let vault_root = {
        let root = vault_state.vault_root.lock().map_err(|e| format!("State lock error: {}", e))?;
        root.clone()
    };

    if vault_root.is_empty() {
        return Err("No vault is connected. Please connect a vault first.".to_string());
    }

    // Add the user message to history
    history.push(ConversationTurn {
        role: "user".to_string(),
        content: message,
        tool_calls: None,
        tool_call_id: None,
    });

    let tool_definitions = get_tool_definitions();
    let max_steps = agent_state.max_steps;

    for iteration in 1..=max_steps {
        // 1. Call the model
        let request = InferenceRequest {
            prompt: String::new(), // Using conversation history instead
            system_prompt: Some(get_system_prompt()),
            conversation_history: history.clone(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            stop_sequences: None,
            tools: Some(tool_definitions.clone()),
        };

        let response = model_state.manager.lock()
            .map_err(|e| format!("State lock error: {}", e))?
            .as_ref()
            .ok_or("Model manager not initialized")?
            .send_completion(&request, model_state.server_port)
            .await?;

        // 2. Check if model wants to call tools
        if let Some(tool_calls) = &response.tool_calls {
            if !tool_calls.is_empty() {
                // Process each tool call through safety then execute
                for tc in tool_calls {
                    // 2a. Parse as ToolAction for safety review
                    let action = ToolAction {
                        action_id: tc.id.clone(),
                        tool_name: tc.name.clone(),
                        parameters: tc.arguments.clone(),
                        confidence: 0.8, // Default; model doesn't provide confidence
                        raw_output: String::new(),
                    };

                    // 2b. Safety review
                    let verdict = {
                        let mut guard = safety_state.guard.lock().map_err(|e| format!("State lock error: {}", e))?;
                        guard.review_action(&action)
                    };

                    if !verdict.approved {
                        steps.push(AgentStep::SafetyBlock {
                            tool: tc.name.clone(),
                            reason: verdict.reason.clone(),
                        });
                        // Feed the blocked result back as a tool observation
                        history.push(ConversationTurn {
                            role: "tool".to_string(),
                            content: format!("Tool '{}' was blocked: {}", tc.name, verdict.reason),
                            tool_calls: None,
                            tool_call_id: Some(tc.id.clone()),
                        });
                        continue;
                    }

                    steps.push(AgentStep::ToolCall {
                        tool: tc.name.clone(),
                        args: tc.arguments.clone(),
                        confidence: action.confidence,
                    });

                    // 2c. Execute the approved tool using real vault state
                    let args = verdict.modified_parameters.as_ref().unwrap_or(&tc.arguments);
                    let result = execute_tool(&tc.name, args, &vault_root).await;

                    steps.push(AgentStep::ToolResult {
                        tool: tc.name.clone(),
                        result: result.clone(),
                        approved: true,
                    });

                    // 2d. Feed result back to model
                    history.push(ConversationTurn {
                        role: "tool".to_string(),
                        content: result,
                        tool_calls: None,
                        tool_call_id: Some(tc.id.clone()),
                    });
                }

                // Continue the loop — the model will see the tool results
                continue;
            }
        }

        // 3. No tool calls — model gave a final answer
        steps.push(AgentStep::FinalResponse {
            text: response.text.clone(),
        });
        history.push(ConversationTurn {
            role: "assistant".to_string(),
            content: response.text,
            tool_calls: None,
            tool_call_id: None,
        });
        break;
    }

    // If we exhausted max steps without a final response
    if !steps.iter().any(|s| matches!(s, AgentStep::FinalResponse { .. })) {
        steps.push(AgentStep::FinalResponse {
            text: "I need more steps to complete this. Could you rephrase or be more specific?".to_string(),
        });
    }

    let final_text = steps.iter().rev()
        .find_map(|s| if let AgentStep::FinalResponse { text } = s { Some(text.clone()) } else { None })
        .unwrap_or_default();

    Ok(AgentResult {
        final_text,
        steps,
        iterations: steps.len() as u32,
    })
}