// UnoOne Power — Desktop Voice Module
// D4: Interface layer for STT (Whisper.cpp) and TTS (Piper).
// Provides trait-based abstraction so the agent loop can request
// speech-to-text and text-to-speech without knowing the backend.
//
// STATUS: Fully wired. Whisper.cpp and Piper binaries are invoked
// via std::process::Command when found on PATH or in RUNTIMES directory.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Voice capability status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VoiceCapabilityStatus {
    Available,
    NotAvailable,
    Initializing,
    Error,
}

/// STT (speech-to-text) result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttResult {
    pub text: String,
    pub language: String,
    pub confidence: f32,
    pub processing_time_ms: u64,
    pub status: VoiceCapabilityStatus,
}

/// TTS (text-to-speech) result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsResult {
    pub audio_path: Option<String>,
    pub duration_seconds: Option<f32>,
    pub sample_rate: u32,
    pub status: VoiceCapabilityStatus,
    pub error: Option<String>,
    pub processing_time_ms: u64,
}

/// Voice engine type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VoiceEngine {
    WhisperCpp,
    Piper,
    SystemDefault,
}

/// Voice module configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub stt_engine: VoiceEngine,
    pub tts_engine: VoiceEngine,
    pub language: String,
    /// Path to the Whisper.cpp model (e.g., ggml-base.en.bin)
    pub whisper_model_path: Option<String>,
    /// Path to the Piper voice model (.onnx)
    pub piper_model_path: Option<String>,
    /// Path to the Piper voice config (.onnx.json)
    pub piper_config_path: Option<String>,
    /// Output directory for TTS audio files
    pub output_dir: Option<String>,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            stt_engine: VoiceEngine::WhisperCpp,
            tts_engine: VoiceEngine::Piper,
            language: "en".to_string(),
            whisper_model_path: None,
            piper_model_path: None,
            piper_config_path: None,
            output_dir: None,
        }
    }
}

/// D4: Voice module — orchestrates STT and TTS backends.
/// Invokes Whisper.cpp and Piper binaries when available.
pub struct VoiceModule {
    config: VoiceConfig,
}

impl VoiceModule {
    pub fn new(config: VoiceConfig) -> Self {
        Self { config }
    }

    /// Check whether Whisper.cpp STT is available on this system
    pub fn check_stt_availability(&self) -> VoiceCapabilityStatus {
        // Check for Whisper.cpp binary
        let whisper_names = if cfg!(target_os = "windows") {
            vec!["whisper.exe", "main.exe"]
        } else {
            vec!["whisper", "main"]
        };

        // Check RUNTIMES directory first
        if let Some(vault_root) = &self.config.output_dir {
            for name in &whisper_names {
                let path = PathBuf::from(vault_root)
                    .join("RUNTIMES")
                    .join(if cfg!(target_os = "windows") { "WINDOWS" } else if cfg!(target_os = "macos") { "MACOS" } else { "LINUX" })
                    .join("VOICE")
                    .join(name);
                if path.exists() {
                    // Also check for model file
                    if let Some(model_path) = &self.config.whisper_model_path {
                        if PathBuf::from(model_path).exists() {
                            return VoiceCapabilityStatus::Available;
                        }
                    }
                }
            }
        }

        // Check system PATH
        for name in &whisper_names {
            if which_exists(name) {
                return VoiceCapabilityStatus::Available;
            }
        }

        VoiceCapabilityStatus::NotAvailable
    }

    /// Check whether Piper TTS is available on this system
    pub fn check_tts_availability(&self) -> VoiceCapabilityStatus {
        let piper_names = if cfg!(target_os = "windows") {
            vec!["piper.exe"]
        } else {
            vec!["piper"]
        };

        // Check RUNTIMES directory first
        if let Some(vault_root) = &self.config.output_dir {
            for name in &piper_names {
                let path = PathBuf::from(vault_root)
                    .join("RUNTIMES")
                    .join(if cfg!(target_os = "windows") { "WINDOWS" } else if cfg!(target_os = "macos") { "MACOS" } else { "LINUX" })
                    .join("VOICE")
                    .join(name);
                if path.exists() {
                    // Also check for model file
                    if let Some(model_path) = &self.config.piper_model_path {
                        if PathBuf::from(model_path).exists() {
                            return VoiceCapabilityStatus::Available;
                        }
                    }
                }
            }
        }

        // Check system PATH
        for name in &piper_names {
            if which_exists(name) {
                return VoiceCapabilityStatus::Available;
            }
        }

        VoiceCapabilityStatus::NotAvailable
    }

    /// Transcribe audio using Whisper.cpp (STT)
    /// Invokes the Whisper binary found during availability check.
    pub fn transcribe(&self, audio_path: &str) -> SttResult {
        let start = std::time::Instant::now();

        let status = self.check_stt_availability();

        if status != VoiceCapabilityStatus::Available {
            return SttResult {
                text: String::new(),
                language: self.config.language.clone(),
                confidence: 0.0,
                processing_time_ms: start.elapsed().as_millis() as u64,
                status: VoiceCapabilityStatus::NotAvailable,
            };
        }

        // Find the Whisper binary path (RUNTIMES directory or system PATH)
        let whisper_bin = self.find_whisper_binary();
        let whisper_bin = match whisper_bin {
            Some(bin) => bin,
            None => {
                return SttResult {
                    text: String::new(),
                    language: self.config.language.clone(),
                    confidence: 0.0,
                    processing_time_ms: start.elapsed().as_millis() as u64,
                    status: VoiceCapabilityStatus::Error,
                };
            }
        };

        // Create a temp directory for Whisper output
        let temp_dir = std::env::temp_dir().join("unoone-whisper");
        if let Err(e) = std::fs::create_dir_all(&temp_dir) {
            return SttResult {
                text: format!("Failed to create temp directory: {}", e),
                language: self.config.language.clone(),
                confidence: 0.0,
                processing_time_ms: start.elapsed().as_millis() as u64,
                status: VoiceCapabilityStatus::Error,
            };
        }

        let model_path = match &self.config.whisper_model_path {
            Some(path) => path.clone(),
            None => {
                return SttResult {
                    text: "No Whisper model path configured".to_string(),
                    language: self.config.language.clone(),
                    confidence: 0.0,
                    processing_time_ms: start.elapsed().as_millis() as u64,
                    status: VoiceCapabilityStatus::Error,
                };
            }
        };

        let output_prefix = temp_dir.join("transcription").to_string_lossy().to_string();

        let result = std::process::Command::new(&whisper_bin)
            .args([
                "--model", &model_path,
                "--language", &self.config.language,
                "-otxt",
                "-of", &output_prefix,
                audio_path,
            ])
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    // Read the transcription output file (Whisper appends .txt)
                    let output_file = temp_dir.join("transcription.txt");
                    let text = std::fs::read_to_string(&output_file)
                        .unwrap_or_default()
                        .trim()
                        .to_string();

                    // Clean up temp file
                    let _ = std::fs::remove_file(&output_file);

                    SttResult {
                        text,
                        language: self.config.language.clone(),
                        confidence: 0.9, // Whisper doesn't provide per-transcription confidence
                        processing_time_ms: start.elapsed().as_millis() as u64,
                        status: VoiceCapabilityStatus::Available,
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    SttResult {
                        text: format!("Whisper transcription failed: {}", stderr.trim()),
                        language: self.config.language.clone(),
                        confidence: 0.0,
                        processing_time_ms: start.elapsed().as_millis() as u64,
                        status: VoiceCapabilityStatus::Error,
                    }
                }
            }
            Err(e) => SttResult {
                text: format!("Failed to run Whisper: {}", e),
                language: self.config.language.clone(),
                confidence: 0.0,
                processing_time_ms: start.elapsed().as_millis() as u64,
                status: VoiceCapabilityStatus::Error,
            },
        }
    }

    /// Synthesize speech using Piper (TTS)
    /// Invokes the Piper binary found during availability check.
    pub fn synthesize(&self, text: &str) -> TtsResult {
        let start = std::time::Instant::now();

        let status = self.check_tts_availability();

        if status != VoiceCapabilityStatus::Available {
            return TtsResult {
                audio_path: None,
                duration_seconds: None,
                sample_rate: 22050,
                status: VoiceCapabilityStatus::NotAvailable,
                error: Some("TTS is not available. Piper binary not found.".to_string()),
                processing_time_ms: start.elapsed().as_millis() as u64,
            };
        }

        // Find the Piper binary path
        let piper_bin = self.find_piper_binary();
        let piper_bin = match piper_bin {
            Some(bin) => bin,
            None => {
                return TtsResult {
                    audio_path: None,
                    duration_seconds: None,
                    sample_rate: 22050,
                    status: VoiceCapabilityStatus::Error,
                    error: Some("Piper binary not found".to_string()),
                    processing_time_ms: start.elapsed().as_millis() as u64,
                };
            }
        };

        // Create output directory
        let output_dir = match &self.config.output_dir {
            Some(dir) => {
                let p = std::path::PathBuf::from(dir).join("VAULT").join("recordings");
                let _ = std::fs::create_dir_all(&p);
                p
            }
            None => {
                let p = std::env::temp_dir().join("unoone-piper");
                let _ = std::fs::create_dir_all(&p);
                p
            }
        };

        let output_file = output_dir.join(format!("tts_{}.wav", chrono::Utc::now().timestamp_millis()));

        let model_path = match &self.config.piper_model_path {
            Some(path) => path.clone(),
            None => {
                return TtsResult {
                    audio_path: None,
                    duration_seconds: None,
                    sample_rate: 22050,
                    status: VoiceCapabilityStatus::Error,
                    error: Some("No Piper model path configured".to_string()),
                    processing_time_ms: start.elapsed().as_millis() as u64,
                };
            }
        };

        let config_path = self.config.piper_config_path.clone().unwrap_or_default();

        // Run: echo "text" | piper --model <model> [--config <config>] --output_file <file>
        let mut cmd = std::process::Command::new(&piper_bin);
        cmd.args(["--model", &model_path])
            .arg("--output_file").arg(&output_file)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if !config_path.is_empty() {
            cmd.arg("--config").arg(&config_path);
        }

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                return TtsResult {
                    audio_path: None,
                    duration_seconds: None,
                    sample_rate: 22050,
                    status: VoiceCapabilityStatus::Error,
                    error: Some(format!("Failed to start Piper: {}", e)),
                    processing_time_ms: start.elapsed().as_millis() as u64,
                };
            }
        };

        // Write text to Piper's stdin
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(text.as_bytes());
        }

        let output = match child.wait_with_output() {
            Ok(output) => output,
            Err(e) => {
                return TtsResult {
                    audio_path: None,
                    duration_seconds: None,
                    sample_rate: 22050,
                    status: VoiceCapabilityStatus::Error,
                    error: Some(format!("Piper process error: {}", e)),
                    processing_time_ms: start.elapsed().as_millis() as u64,
                };
            }
        };

        if output.status.success() && output_file.exists() {
            // Estimate duration from file size (WAV at 22050 Hz, 16-bit mono ≈ 44100 bytes/sec)
            let file_size = std::fs::metadata(&output_file)
                .map(|m| m.len())
                .unwrap_or(0);
            let duration = file_size as f32 / 44100.0;

            TtsResult {
                audio_path: Some(output_file.to_string_lossy().to_string()),
                duration_seconds: Some(duration),
                sample_rate: 22050,
                status: VoiceCapabilityStatus::Available,
                error: None,
                processing_time_ms: start.elapsed().as_millis() as u64,
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            TtsResult {
                audio_path: None,
                duration_seconds: None,
                sample_rate: 22050,
                status: VoiceCapabilityStatus::Error,
                error: Some(format!("Piper synthesis failed: {}", stderr.trim())),
                processing_time_ms: start.elapsed().as_millis() as u64,
            }
        }
    }

    /// Find the Whisper binary on the system
    fn find_whisper_binary(&self) -> Option<String> {
        let whisper_names = if cfg!(target_os = "windows") {
            vec!["whisper.exe", "main.exe"]
        } else {
            vec!["whisper", "main"]
        };

        // Check RUNTIMES directory first
        if let Some(vault_root) = &self.config.output_dir {
            for name in &whisper_names {
                let path = PathBuf::from(vault_root)
                    .join("RUNTIMES")
                    .join(if cfg!(target_os = "windows") { "WINDOWS" } else if cfg!(target_os = "macos") { "MACOS" } else { "LINUX" })
                    .join("VOICE")
                    .join(name);
                if path.exists() {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }

        // Check system PATH
        for name in &whisper_names {
            if which_exists(name) {
                return Some(name.to_string());
            }
        }

        None
    }

    /// Find the Piper binary on the system
    fn find_piper_binary(&self) -> Option<String> {
        let piper_names = if cfg!(target_os = "windows") {
            vec!["piper.exe"]
        } else {
            vec!["piper"]
        };

        // Check RUNTIMES directory first
        if let Some(vault_root) = &self.config.output_dir {
            for name in &piper_names {
                let path = PathBuf::from(vault_root)
                    .join("RUNTIMES")
                    .join(if cfg!(target_os = "windows") { "WINDOWS" } else if cfg!(target_os = "macos") { "MACOS" } else { "LINUX" })
                    .join("VOICE")
                    .join(name);
                if path.exists() {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }

        // Check system PATH
        for name in &piper_names {
            if which_exists(name) {
                return Some(name.to_string());
            }
        }

        None
    }
}

/// Check if a command exists in PATH
fn which_exists(name: &str) -> bool {
    std::process::Command::new(if cfg!(target_os = "windows") { "where" } else { "which" })
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// Tauri commands for voice module

#[tauri::command]
pub fn get_voice_status(vault_root: String) -> serde_json::Value {
    let config = VoiceConfig {
        output_dir: Some(vault_root),
        ..Default::default()
    };
    let module = VoiceModule::new(config);

    serde_json::json!({
        "stt": module.check_stt_availability(),
        "tts": module.check_tts_availability(),
        "language": "en",
    })
}

#[tauri::command]
pub fn transcribe_audio(audio_path: String, vault_root: String) -> SttResult {
    let config = VoiceConfig {
        output_dir: Some(vault_root),
        ..Default::default()
    };
    let module = VoiceModule::new(config);
    module.transcribe(&audio_path)
}

#[tauri::command]
pub fn synthesize_speech(text: String, vault_root: String) -> TtsResult {
    let config = VoiceConfig {
        output_dir: Some(vault_root),
        ..Default::default()
    };
    let module = VoiceModule::new(config);
    module.synthesize(&text)
}