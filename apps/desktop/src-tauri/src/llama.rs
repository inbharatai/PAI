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

/// Inference request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub conversation_history: Vec<ConversationTurn>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
}

/// Conversation turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub role: String,  // "user" or "assistant"
    pub content: String,
}

/// Inference response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub text: String,
    pub tokens_generated: u32,
    pub tokens_per_second: f32,
    pub model_id: String,
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
    pub fn get_status(&self) -> ModelStatus {
        self.status.lock().unwrap().clone()
    }

    /// Get current backend
    pub fn get_backend(&self) -> AccelerationBackend {
        self.backend.lock().unwrap().clone()
    }

    /// Set backend
    pub fn set_backend(&self, backend: AccelerationBackend) {
        *self.backend.lock().unwrap() = backend;
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
    // TODO: Add reqwest to Cargo.toml and implement proper health verification:
    //   1. GET /health → confirm status "ok"
    //   2. Verify PID matches our managed process
    //   3. Verify executable path is from validated USB
    //   4. Verify loaded model hash matches manifest
    //
    // Until then, return PENDING_VERIFICATION to avoid false confidence.
    "PENDING_VERIFICATION".to_string()
}