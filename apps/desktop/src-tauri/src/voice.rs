// UnoOne Power — Desktop Voice Module
// D4: Interface layer for STT (Whisper.cpp) and TTS (Piper).
// Provides trait-based abstraction so the agent loop can request
// speech-to-text and text-to-speech without knowing the backend.
//
// STATUS: Interface is wired. Whisper.cpp and Piper integrations return
// honest "not available" errors until their native libraries are compiled
// for the target platform. The interface is ready for progressive rollout.

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
/// Currently returns "not available" for both, but the interface
/// is wired for progressive integration of Whisper.cpp and Piper.
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
    /// Currently returns a not-available error until Whisper.cpp is compiled.
    pub fn transcribe(&self, audio_path: &str) -> SttResult {
        let status = self.check_stt_availability();

        if status != VoiceCapabilityStatus::Available {
            return SttResult {
                text: String::new(),
                language: self.config.language.clone(),
                confidence: 0.0,
                processing_time_ms: 0,
                status: VoiceCapabilityStatus::NotAvailable,
            };
        }

        // Whisper.cpp integration: would invoke the binary or C API here.
        // For now, the availability check above returns NotAvailable,
        // so this code path is not reached until Whisper.cpp is compiled.
        SttResult {
            text: String::new(),
            language: self.config.language.clone(),
            confidence: 0.0,
            processing_time_ms: 0,
            status: VoiceCapabilityStatus::Error,
        }
    }

    /// Synthesize speech using Piper (TTS)
    /// Currently returns a not-available error until Piper is compiled.
    pub fn synthesize(&self, text: &str) -> TtsResult {
        let status = self.check_tts_availability();

        if status != VoiceCapabilityStatus::Available {
            return TtsResult {
                audio_path: None,
                duration_seconds: None,
                sample_rate: 22050,
                status: VoiceCapabilityStatus::NotAvailable,
                error: Some("TTS is not yet available. Piper integration is pending.".to_string()),
            };
        }

        // Piper integration: would invoke the binary or C API here.
        TtsResult {
            audio_path: None,
            duration_seconds: None,
            sample_rate: 22050,
            status: VoiceCapabilityStatus::Error,
            error: Some("Piper integration is pending".to_string()),
        }
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