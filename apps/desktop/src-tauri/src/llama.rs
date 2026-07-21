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
    pub fn find_models(&self, vault_root: &str) -> Vec<ModelInfo> {
        let mut models = Vec::new();

        // Gemma 4 12B Q4
        let model_dir_12b = PathBuf::from(vault_root)
            .join("MODELS")
            .join("gemma4-12b-q4-gguf");

        if model_dir_12b.exists() {
            if let Ok(entries) = std::fs::read_dir(&model_dir_12b) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "gguf" || ext == "GGUF" {
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

        // Gemma 4 E2B (mobile model, also usable on desktop)
        let model_dir_e2b = PathBuf::from(vault_root)
            .join("MODELS")
            .join("gemma4-e2b");

        if model_dir_e2b.exists() {
            if let Ok(entries) = std::fs::read_dir(&model_dir_e2b) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "gguf" || ext == "GGUF" {
                            let file_size = std::fs::metadata(&path)
                                .map(|m| m.len() as f64 / (1024.0 * 1024.0 * 1024.0))
                                .unwrap_or(0.0);

                            models.push(ModelInfo {
                                name: "Gemma 4 E2B".to_string(),
                                model_type: "gemma-4-e2b".to_string(),
                                quantization: "F16".to_string(),
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

        // If no models found in USB, mark as not available
        if models.is_empty() {
            models.push(ModelInfo {
                name: "Gemma 4 12B Q4_K_M".to_string(),
                model_type: "gemma-4-12b".to_string(),
                quantization: "Q4_K_M".to_string(),
                file_size_gb: 7.4,
                context_length: 8192,
                available: false,
                path: String::new(),
            });
        }

        models
    }

    /// Get the llama.cpp binary path for the current platform
    fn get_llama_binary_path(&self, vault_root: &str) -> PathBuf {
        let runtime_dir = if cfg!(target_os = "windows") {
            PathBuf::from(vault_root).join("RUNTIMES").join("windows")
        } else if cfg!(target_os = "macos") {
            PathBuf::from(vault_root).join("RUNTIMES").join("macos")
        } else {
            PathBuf::from(vault_root).join("RUNTIMES").join("linux")
        };

        let binary_name = if cfg!(target_os = "windows") {
            "llama-server.exe"
        } else {
            "llama-server"
        };

        runtime_dir.join(binary_name)
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

        // Start the process
        let child = cmd.spawn()
            .map_err(|e| format!("Failed to start llama-server: {}", e))?;

        *self.llama_process.lock().unwrap() = Some(child);
        *self.status.lock().unwrap() = ModelStatus::Loaded;

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
    let manager = ModelManager::new();
    serde_json::to_string(&manager.get_status()).unwrap_or_else(|_| "\"NOT_LOADED\"".to_string())
}