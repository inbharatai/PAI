// UnoOne Power — Desktop Model Manager
// Manages Gemma 4 12B Q4 GGUF model via llama.cpp

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

/// Model configuration for Gemma 4 12B
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_path: String,
    pub context_size: u32,
    pub batch_size: u32,
    pub threads: u32,
    pub gpu_layers: i32,  // -1 = all, 0 = CPU only
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: u32,
    pub repeat_penalty: f32,
    pub max_tokens: u32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            context_size: 4096,
            batch_size: 512,
            threads: 0, // 0 = auto-detect
            gpu_layers: -1, // -1 = offload all layers
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            max_tokens: 4096,
        }
    }
}

/// Hardware-acceleration backend
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccelerationBackend {
    Cuda,
    Metal,
    Vulkan,
    Cpu,
}

/// Model loading status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelStatus {
    NotLoaded,
    Loading,
    Loaded,
    Generating,
    Error,
}

/// Inference request — D1: used by the agentic loop to send completions to llama-server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub conversation_history: Vec<ConversationTurn>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub tools: Option<Vec<ToolDefinition>>,
}

/// Conversation turn — D1: extended with tool_calls for multi-turn function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub role: String,  // "user", "assistant", or "tool"
    pub content: String,
    /// For assistant turns with tool calls, the OpenAI-format tool_calls array
    pub tool_calls: Option<Vec<ToolCallResult>>,
    /// For tool role turns, the ID of the tool call this responds to
    pub tool_call_id: Option<String>,
}

/// D1: Tool definition for OpenAI-compatible function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,  // JSON Schema
}

/// D1: Parsed tool call from model output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Inference response — D1: extended with tool_calls and finish_reason
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub text: String,
    pub tokens_generated: u32,
    pub tokens_per_second: f32,
    pub model_id: String,
    pub tool_calls: Option<Vec<ToolCallResult>>,
    pub finish_reason: Option<String>,  // "stop", "tool_calls", "length"
}

/// Model info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub model_type: String,
    pub quantization: String,
    pub file_size_gb: f64,
    pub context_length: u32,
    pub available: bool,
    pub path: String,
}

/// Model manager state
pub struct ModelManager {
    config: Mutex<Option<ModelConfig>>,
    status: Mutex<ModelStatus>,
    backend: Mutex<AccelerationBackend>,
    llama_process: Mutex<Option<std::process::Child>>,
    model_info: Mutex<Option<ModelInfo>>,
}

impl ModelManager {
    pub fn new() -> Self {
        Self {
            config: Mutex::new(None),
            status: Mutex::new(ModelStatus::NotLoaded),
            backend: Mutex::new(AccelerationBackend::Cpu),
            llama_process: Mutex::new(None),
            model_info: Mutex::new(None),
        }
    }

    /// Detect available acceleration backends
    pub fn detect_backends(&self) -> Vec<AccelerationBackend> {
        let mut backends = vec![AccelerationBackend::Cpu];

        // Check CUDA
        if self.check_cuda() {
            backends.insert(0, AccelerationBackend::Cuda);
        }

        // Check Metal (macOS only)
        if cfg!(target_os = "macos") && self.check_metal() {
            backends.insert(0, AccelerationBackend::Metal);
        }

        // Check Vulkan
        if self.check_vulkan() {
            backends.insert(0, AccelerationBackend::Vulkan);
        }

        backends
    }

    fn check_cuda(&self) -> bool {
        // Check for CUDA by trying to find nvcuda.dll (Windows) or libcuda.so (Linux)
        if cfg!(target_os = "windows") {
            std::path::Path::new("C:\\Windows\\System32\\nvcuda.dll").exists()
                || std::path::Path::new("C:\\Windows\\System32\\nvcuda64.dll").exists()
        } else if cfg!(target_os = "linux") {
            std::path::Path::new("/usr/lib/x86_64-linux-gnu/libcuda.so").exists()
                || std::path::Path::new("/usr/lib/libcuda.so").exists()
        } else {
            false
        }
    }

    fn check_metal(&self) -> bool {
        // Metal is always available on macOS
        cfg!(target_os = "macos")
    }

    fn check_vulkan(&self) -> bool {
        // Check for Vulkan runtime
        if cfg!(target_os = "windows") {
            std::path::Path::new("C:\\Windows\\System32\\vulkan-1.dll").exists()
        } else if cfg!(target_os = "linux") {
            std::path::Path::new("/usr/lib/x86_64-linux-gnu/libvulkan.so").exists()
                || std::path::Path::new("/usr/lib/libvulkan.so").exists()
        } else {
            false
        }
    }

    /// Find GGUF model files in the MODELS directory
    /// Uses manifest-based discovery: reads manifest.json for model metadata,
    /// then scans MODELS/DESKTOP/ and MODELS/MOBILE/ directories
    pub fn find_models(&self, vault_root: &str) -> Vec<ModelInfo> {
        let mut models = Vec::new();

        // Try manifest-based discovery first
        let manifest_path = PathBuf::from(vault_root).join("manifest.json");
        if let Ok(manifest_content) = std::fs::read_to_string(&manifest_path) {
            if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&manifest_content) {
                // Read desktop models from manifest
                if let Some(desktop) = manifest.get("models").and_then(|m| m.get("desktop")) {
                    if let Some(obj) = desktop.as_object() {
                        for (_key, model) in obj {
                            let model_path = model.get("path")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            let full_path = PathBuf::from(vault_root).join(model_path);

                            if full_path.exists() {
                                let name = model.get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown Model")
                                    .to_string();
                                let file_size = std::fs::metadata(&full_path)
                                    .map(|m| m.len() as f64 / (1024.0 * 1024.0 * 1024.0))
                                    .unwrap_or(0.0);

                                models.push(ModelInfo {
                                    name,
                                    model_type: model.get("architecture")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown")
                                        .to_string(),
                                    quantization: model.get("quantisation")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown")
                                        .to_string(),
                                    file_size_gb: file_size,
                                    context_length: 8192,
                                    available: true,
                                    path: full_path.to_string_lossy().to_string(),
                                });
                            }
                        }
                    }
                }

                // Read mobile models from manifest
                if let Some(mobile) = manifest.get("models").and_then(|m| m.get("mobile")) {
                    if let Some(obj) = mobile.as_object() {
                        for (_key, model) in obj {
                            let model_path = model.get("path")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            let full_path = PathBuf::from(vault_root).join(model_path);

                            if full_path.exists() {
                                let file_size = std::fs::metadata(&full_path)
                                    .map(|m| m.len() as f64 / (1024.0 * 1024.0 * 1024.0))
                                    .unwrap_or(0.0);

                                models.push(ModelInfo {
                                    name: model.get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("Unknown Mobile Model")
                                        .to_string(),
                                    model_type: model.get("architecture")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown")
                                        .to_string(),
                                    quantization: model.get("quantisation")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown")
                                        .to_string(),
                                    file_size_gb: file_size,
                                    context_length: 4096,
                                    available: true,
                                    path: full_path.to_string_lossy().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Fallback: scan directories directly if manifest parsing fails
        if models.is_empty() {
            // Desktop models (Gemma 12B)
            let desktop_dir = PathBuf::from(vault_root)
                .join("MODELS").join("DESKTOP").join("Gemma-12B");
            if desktop_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&desktop_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if name.ends_with(".gguf") && !name.contains("mmproj") {
                                let file_size = std::fs::metadata(&path)
                                    .map(|m| m.len() as f64 / (1024.0 * 1024.0 * 1024.0))
                                    .unwrap_or(0.0);
                                models.push(ModelInfo {
                                    name: "Gemma 4 12B Q4_K_M".to_string(),
                                    model_type: "gemma-4-12b".to_string(),
                                    quantization: "Q4_K_M".to_string(),
                                    file_size_gb: file_size,
                                    context_length: 8192,
                                    available: true,
                                    path: path.to_string_lossy().to_string(),
                                });
                            }
                        }
                    }
                }
            }

            // Mobile models (E2B)
            let mobile_dir = PathBuf::from(vault_root)
                .join("MODELS").join("MOBILE");
            if mobile_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&mobile_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if name.ends_with(".gguf") {
                                let file_size = std::fs::metadata(&path)
                                    .map(|m| m.len() as f64 / (1024.0 * 1024.0 * 1024.0))
                                    .unwrap_or(0.0);
                                models.push(ModelInfo {
                                    name: "Gemma 4 E2B Q4_K_M".to_string(),
                                    model_type: "gemma-4-e2b".to_string(),
                                    quantization: "Q4_K_M".to_string(),
                                    file_size_gb: file_size,
                                    context_length: 4096,
                                    available: true,
                                    path: path.to_string_lossy().to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // If no models found, mark as not available
        if models.is_empty() {
            models.push(ModelInfo {
                name: "Gemma 4 12B Q4_K_M".to_string(),
                model_type: "gemma-4-12b".to_string(),
                quantization: "Q4_K_M".to_string(),
                file_size_gb: 7.14,
                context_length: 8192,
                available: false,
                path: String::new(),
            });
        }

        models
    }

    /// Get the llama.cpp binary path for the current platform
    /// Uses manifest-informed directory structure (uppercase CUDA/CPU/VULKAN)
    /// Prefers CUDA > Vulkan > CPU based on detected hardware
    #[allow(dead_code)]
    fn get_llama_binary_path(&self, vault_root: &str) -> PathBuf {
        let base_dir = if cfg!(target_os = "windows") {
            PathBuf::from(vault_root).join("RUNTIMES").join("WINDOWS")
        } else if cfg!(target_os = "macos") {
            PathBuf::from(vault_root).join("RUNTIMES").join("MACOS")
        } else {
            PathBuf::from(vault_root).join("RUNTIMES").join("LINUX")
        };

        let binary_name = if cfg!(target_os = "windows") {
            "llama-server.exe"
        } else {
            "llama-server"
        };

        // Order: CUDA > Vulkan > CPU (matches hardware acceleration priority)
        let backends = if cfg!(target_os = "macos") {
            vec!["METAL"]
        } else {
            vec!["CUDA", "VULKAN", "CPU"]
        };

        for backend in &backends {
            let path = base_dir.join(backend).join(binary_name);
            if path.exists() {
                // Verify the implementation DLL exists (9KB launcher is useless without it)
                let impl_path = base_dir.join(backend).join("llama-server-impl.dll");
                let impl_path_mac = base_dir.join(backend).join("llama-server-impl.dylib");
                if impl_path.exists() || impl_path_mac.exists() || !cfg!(target_os = "windows") {
                    return path;
                }
                // On Windows, if the impl DLL doesn't exist, skip this backend
            }
        }

        // Fallback: check lowercase paths for backwards compatibility
        let backends_compat = if cfg!(target_os = "macos") {
            vec!["metal"]
        } else {
            vec!["cuda", "vulkan", "cpu"]
        };

        for backend in &backends_compat {
            let path = base_dir.join(backend).join(binary_name);
            if path.exists() {
                return path;
            }
        }

        // Last resort: direct in runtime dir
        base_dir.join(binary_name)
    }

    /// Start llama-server for inference
    #[allow(dead_code)]
    pub fn start_server(&self, config: &ModelConfig, vault_root: &str) -> Result<u16, String> {
        let llama_path = self.get_llama_binary_path(vault_root);

        if !llama_path.exists() {
            return Err(format!(
                "llama-server not found at {:?}. Please install llama.cpp runtime.",
                llama_path
            ));
        }

        if config.model_path.is_empty() {
            return Err("No model path configured".to_string());
        }

        let model_path = PathBuf::from(&config.model_path);
        if !model_path.exists() {
            return Err(format!("Model file not found: {:?}", config.model_path));
        }

        // Find an available port
        let port = 8342; // UnoOne default port

        let mut cmd = Command::new(&llama_path);
        cmd.args([
            "-m", &config.model_path,
            "--port", &port.to_string(),
            "-c", &config.context_size.to_string(),
            "-b", &config.batch_size.to_string(),
            "--temp", &config.temperature.to_string(),
            "--top-p", &config.top_p.to_string(),
            "--top-k", &config.top_k.to_string(),
            "--repeat-penalty", &config.repeat_penalty.to_string(),
            "-n", &config.max_tokens.to_string(),
        ]);

        // GPU layers
        if config.gpu_layers != 0 {
            cmd.args(["-ngl", &config.gpu_layers.to_string()]);
        }

        // Threads
        if config.threads > 0 {
            cmd.args(["-t", &config.threads.to_string()]);
        }

        // Backend-specific flags
        let backend = self.backend.lock().unwrap();
        match *backend {
            AccelerationBackend::Cuda => {
                cmd.args(["--gpu", "cuda"]);
            }
            AccelerationBackend::Metal => {
                // Metal is default on macOS
            }
            AccelerationBackend::Vulkan => {
                cmd.args(["--gpu", "vulkan"]);
            }
            AccelerationBackend::Cpu => {
                cmd.args(["-ngl", "0"]);
            }
        }

        // Start the process — DO NOT mark as Loaded until health check passes
        let child = cmd.spawn()
            .map_err(|e| format!("Failed to start llama-server: {}", e))?;

        *self.llama_process.lock().unwrap() = Some(child);
        *self.status.lock().unwrap() = ModelStatus::Loading;

        Ok(port)
    }

    /// Stop the llama-server process
    #[allow(dead_code)]
    pub fn stop_server(&self) -> Result<(), String> {
        let mut process = self.llama_process.lock().unwrap();
        if let Some(ref mut child) = *process {
            child.kill()
                .map_err(|e| format!("Failed to kill llama-server: {}", e))?;
            *process = None;
        }
        *self.status.lock().unwrap() = ModelStatus::NotLoaded;
        Ok(())
    }

    /// Get current model status
    #[allow(dead_code)]
    pub fn get_status(&self) -> ModelStatus {
        self.status.lock().unwrap().clone()
    }

    /// Get current backend
    #[allow(dead_code)]
    pub fn get_backend(&self) -> AccelerationBackend {
        self.backend.lock().unwrap().clone()
    }

    /// Set backend
    #[allow(dead_code)]
    pub fn set_backend(&self, backend: AccelerationBackend) {
        *self.backend.lock().unwrap() = backend;
    }

    /// D1: Send a chat completion request to llama-server via HTTP.
    /// Uses reqwest to POST to the OpenAI-compatible /v1/chat/completions endpoint.
    /// Supports both plain text responses and function/tool calling.
    pub async fn send_completion(&self, request: &InferenceRequest, port: u16) -> Result<InferenceResponse, String> {
        let url = format!("http://127.0.0.1:{}/v1/chat/completions", port);

        // Build OpenAI-compatible request body
        let mut messages = Vec::new();
        if let Some(sys) = &request.system_prompt {
            messages.push(serde_json::json!({"role": "system", "content": sys}));
        }
        for turn in &request.conversation_history {
            let mut msg = serde_json::json!({"role": turn.role, "content": turn.content});
            if let Some(tool_calls) = &turn.tool_calls {
                msg["tool_calls"] = serde_json::json!(tool_calls.iter().map(|tc| {
                    serde_json::json!({
                        "id": tc.id,
                        "type": "function",
                        "function": {
                            "name": tc.name,
                            "arguments": tc.arguments.to_string(),
                        }
                    })
                }).collect::<Vec<_>>());
            }
            if let Some(tool_call_id) = &turn.tool_call_id {
                msg["tool_call_id"] = serde_json::json!(tool_call_id);
            }
            messages.push(msg);
        }
        // Add the current user prompt if not already in history
        if !request.prompt.is_empty() {
            messages.push(serde_json::json!({"role": "user", "content": &request.prompt}));
        }

        let mut body = serde_json::json!({
            "model": "gemma-4-12b",
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "temperature": request.temperature.unwrap_or(0.7),
            "stream": false,
        });

        if let Some(tools) = &request.tools {
            body["tools"] = serde_json::json!(tools.iter().map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters,
                    }
                })
            }).collect::<Vec<_>>());
        }

        let client = reqwest::Client::new();
        let response = client.post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to llama-server: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("llama-server error {}: {}", status, text));
        }

        let data: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Parse OpenAI-compatible response
        let choice = data.get("choices").and_then(|c| c.get(0));
        let message = choice.and_then(|c| c.get("message"));

        let text = message
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        // Parse tool calls if present
        let tool_calls = message
            .and_then(|m| m.get("tool_calls"))
            .and_then(|tc| tc.as_array())
            .map(|arr| {
                arr.iter().filter_map(|tc| {
                    let id = tc.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let name = tc.get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("").to_string();
                    let arguments = tc.get("function")
                        .and_then(|f| f.get("arguments"))
                        .and_then(|a| a.as_str())
                        .and_then(|s| serde_json::from_str(s).ok())
                        .unwrap_or(serde_json::Value::Object(Default::default()));
                    Some(ToolCallResult { id, name, arguments })
                }).collect::<Vec<_>>())
            });

        let finish_reason = choice
            .and_then(|c| c.get("finish_reason"))
            .and_then(|f| f.as_str())
            .map(|s| s.to_string());

        // If no structured tool_calls but text contains tool-call JSON, parse as fallback
        let final_tool_calls = if tool_calls.as_ref().map_or(true, |tc| tc.is_empty()) {
            Self::parse_text_tool_calls(&text)
        } else {
            tool_calls
        };

        Ok(InferenceResponse {
            text,
            tokens_generated: data.get("usage")
                .and_then(|u| u.get("completion_tokens"))
                .and_then(|t| t.as_u64())
                .unwrap_or(0) as u32,
            tokens_per_second: 0.0,
            model_id: data.get("model")
                .and_then(|m| m.as_str())
                .unwrap_or("gemma-4-12b")
                .to_string(),
            tool_calls: final_tool_calls,
            finish_reason,
        })
    }

    /// Fallback parser for models that emit tool calls as text instead of structured JSON.
    /// Looks for ```json\n{"tool": "...", "args": {...}}\n``` blocks in the response.
    fn parse_text_tool_calls(text: &str) -> Option<Vec<ToolCallResult>> {
        let mut results = Vec::new();
        let mut id_counter = 0u32;

        // Pattern 1: ```json blocks containing tool calls
        for block in text.split("```json").skip(1) {
            if let Some(end) = block.find("```") {
                let json_str = block[..end].trim();
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                    let tool_name = parsed.get("tool")
                        .or_else(|| parsed.get("name"))
                        .or_else(|| parsed.get("action"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("").to_string();
                    if !tool_name.is_empty() {
                        let args = parsed.get("args")
                            .or_else(|| parsed.get("arguments"))
                            .or_else(|| parsed.get("parameters"))
                            .cloned()
                            .unwrap_or(serde_json::Value::Object(Default::default()));
                        results.push(ToolCallResult {
                            id: format!("call_{}", id_counter),
                            name: tool_name,
                            arguments: args,
                        });
                        id_counter += 1;
                    }
                }
            }
        }

        if results.is_empty() { None } else { Some(results) }
    }
}

/// D1: State wrapper for ModelManager so it can be held as Tauri managed state.
/// The ModelManager itself is not Send+Sync, so we wrap it in Mutex<Option<ModelManager>>.
pub struct ModelManagerState {
    pub manager: Mutex<Option<ModelManager>>,
    pub server_port: u16,
}

impl ModelManagerState {
    pub fn new() -> Self {
        Self {
            manager: Mutex::new(None),
            server_port: 8342,
        }
    }
}

// Tauri command wrappers

#[tauri::command]
pub fn list_models(vault_root: String) -> Result<Vec<ModelInfo>, String> {
    let manager = ModelManager::new();
    Ok(manager.find_models(&vault_root))
}

#[tauri::command]
pub fn detect_acceleration() -> Vec<AccelerationBackend> {
    let manager = ModelManager::new();
    manager.detect_backends()
}

#[tauri::command]
pub fn get_model_config() -> ModelConfig {
    ModelConfig::default()
}

#[tauri::command]
pub fn get_model_status() -> String {
    // SECURITY: A TCP port check alone does NOT prove a model is loaded.
    // Any process on port 8342 would pass this check. The correct verification
    // requires: (1) confirming the PID matches our managed child process,
    // (2) calling the /health endpoint, (3) verifying the loaded model identity.
    //
    // Until a proper HTTP client (e.g., reqwest) is added to Cargo.toml,
    // we report STARTING if we have a managed process, but we do NOT
    // claim LOADED without a health check.

    // Step 1: Check if we have a managed child process
    // This is tracked by ModelManager, but since this is a free function
    // we can only check TCP connectivity. We deliberately return a
    // conservative status rather than claiming LOADED.

    if std::net::TcpStream::connect_timeout(
        &"127.0.0.1:8342".parse().unwrap(),
        std::time::Duration::from_secs(2),
    ).is_err() {
        return "NOT_LOADED".to_string();
    }

    // TCP port is open — but we have NOT verified:
    // - That this is our llama-server process (not another process on this port)
    // - That the model is actually loaded (not still loading)
    // - That the model identity matches what we expect
    // - That the model hash is correct
    //
    // Return PENDING_VERIFICATION to avoid false confidence.
    "PENDING_VERIFICATION".to_string()
}

/// D1: Send a chat completion request to llama-server. Async because it uses reqwest.
#[tauri::command]
pub async fn send_chat_completion(
    request: InferenceRequest,
    state: tauri::State<'_, ModelManagerState>,
) -> Result<InferenceResponse, String> {
    let manager = state.manager.lock().map_err(|e| format!("State lock error: {}", e))?;
    let manager = manager.as_ref().ok_or("Model manager not initialized")?;
    manager.send_completion(&request, state.server_port).await
}

/// D1: Proper health check using reqwest instead of raw TCP.
#[tauri::command]
pub async fn check_model_health() -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let response = client.get("http://127.0.0.1:8342/health")
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
        .map_err(|e| format!("Health check failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Health check returned status: {}", response.status()));
    }

    let body: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse health response: {}", e))?;
    Ok(body)
}