// UnoOne Power — Desktop ReAct Agent Loop
// D2: Implements the agentic reasoning loop on the backend.
// Model → parse tool calls → safety guard → execute tools → observe → loop.
// The frontend sends a user message and receives structured AgentResult with steps.

use crate::llama::{ConversationTurn, InferenceRequest, InferenceResponse, ModelManagerState, ToolDefinition};
use crate::safety::{DesktopSafetyGuard, SafetyGuardState, SecurityLevel, ToolAction};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

const MAX_AGENT_STEPS: u32 = 6;

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
fn get_system_prompt() -> String {
    "You are UnoOne, a private AI assistant running on the user's encrypted vault. \
     You have access to tools for searching notes, listing documents, and reading vault content. \
     Always reason step-by-step. When you need information, use the appropriate tool. \
     When you have enough information to answer the user's question, respond directly without calling tools. \
     If a tool call is blocked by safety, acknowledge it and try an alternative approach."
        .to_string()
}

/// D2: Tool definitions available to the model.
/// These mirror the tools that have real implementations in the desktop backend.
fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "search_notes".to_string(),
            description: "Search for notes and documents in the encrypted vault by query string.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" },
                    "limit": { "type": "integer", "description": "Maximum number of results to return (default 10)" }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "list_documents".to_string(),
            description: "List all documents stored in the encrypted vault.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "read_document".to_string(),
            description: "Read the contents of a specific document from the vault.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "document_id": { "type": "string", "description": "The document ID or filename to read" }
                },
                "required": ["document_id"]
            }),
        },
    ]
}

/// Execute a single tool call after it has been approved by the safety guard.
/// Returns the tool result as a string.
async fn execute_tool(tool_name: &str, args: &serde_json::Value) -> String {
    match tool_name {
        "search_notes" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            format!("Search results for '{}': (vault search implementation pending)", query)
        }
        "list_documents" => {
            "Document listing: (vault must be unlocked first)".to_string()
        }
        "read_document" => {
            let doc_id = args.get("document_id").and_then(|v| v.as_str()).unwrap_or("");
            format!("Document '{}' content: (vault read implementation pending)", doc_id)
        }
        _ => format!("Unknown tool: {}", tool_name),
    }
}

/// D2: Run the full ReAct agent loop for a user message.
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
) -> Result<AgentResult, String> {
    let mut steps = Vec::new();
    let mut history = conversation_history;

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

                    // 2c. Execute the approved tool
                    let args = verdict.modified_parameters.as_ref().unwrap_or(&tc.arguments);
                    let result = execute_tool(&tc.name, args).await;

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
            text: "I've reached the maximum number of reasoning steps. Here's what I found so far.".to_string(),
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