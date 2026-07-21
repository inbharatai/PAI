// UnoOne Power — Desktop Recording Engine
// Cross-platform audio recording with encrypted chunked writing to vault
// State is shared across Tauri commands via tauri::State

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Instant;

/// Recording state machine (mirrors Android RecordingState)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecordingState {
    Idle,
    Recording,
    Paused,
    Processing,
    Stopped,
    Error,
}

/// Recording type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecordingType {
    VoiceMemo,
    Meeting,
    Lecture,
    Interview,
    Note,
}

/// Privacy level for recordings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrivacyLevel {
    Full,           // Full audio + transcript + summary
    TranscriptOnly, // No audio, just transcript + summary
    SummaryOnly,    // No audio or transcript, just summary
    PrivateSession, // Nothing saved after session ends
}

/// Recording session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSession {
    pub id: String,
    pub title: String,
    pub state: RecordingState,
    pub recording_type: RecordingType,
    pub privacy_level: PrivacyLevel,
    pub started_at: Option<String>,
    pub stopped_at: Option<String>,
    pub duration_seconds: u64,
    pub bookmarks: Vec<RecordingBookmark>,
    pub source_platform: String,
    pub source_device_id: String,
    pub audio_path: Option<String>,
    pub transcript_path: Option<String>,
    pub summary_path: Option<String>,
}

/// Bookmark in a recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingBookmark {
    pub timestamp_seconds: u64,
    pub label: Option<String>,
}

/// Shared recording engine state
pub struct RecordingStateHolder {
    pub current_session: Mutex<Option<RecordingSession>>,
    pub start_time: Mutex<Option<Instant>>,
}

impl RecordingStateHolder {
    pub fn new() -> Self {
        Self {
            current_session: Mutex::new(None),
            start_time: Mutex::new(None),
        }
    }
}

fn get_device_id() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "desktop-unknown".to_string())
}

// Tauri commands — all use shared state
// SECURITY: Until vault encryption is implemented (directive section 15),
// recording to the vault would write unencrypted audio to disk.
// start_recording MUST NOT create directories or write files until
// Argon2id + XChaCha20-Poly1305 vault encryption is operational.

#[tauri::command]
pub fn start_recording(
    recording_type: RecordingType,
    privacy_level: PrivacyLevel,
    vault_root: String,
    state: tauri::State<'_, RecordingStateHolder>,
) -> Result<RecordingSession, String> {
    // SECURITY BLOCK: Audio recording writes unencrypted data to the vault.
    // Until vault encryption exists, recording MUST NOT create directories
    // or write files. The state machine tracks intent, but no audio is captured.
    // See directive section 15: "Do not write memory, recordings or documents."
    let _ = vault_root; // Acknowledged but not used — no directory creation until encryption
    let session_id = uuid::Uuid::new_v4().to_string();

    let session = RecordingSession {
        id: session_id.clone(),
        title: format!("Recording {}", chrono::Utc::now().format("%Y-%m-%d %H:%M")),
        state: RecordingState::Error,
        recording_type,
        privacy_level,
        started_at: None,
        stopped_at: None,
        duration_seconds: 0,
        bookmarks: Vec::new(),
        source_platform: "DESKTOP".to_string(),
        source_device_id: get_device_id(),
        audio_path: None,
        transcript_path: None,
        summary_path: None,
    };

    *state.current_session.lock().map_err(|e| format!("State lock error: {}", e))? = Some(session.clone());
    *state.start_time.lock().map_err(|e| format!("State lock error: {}", e))? = Some(Instant::now());

    Err("NOT_IMPLEMENTED_SECURITY_BLOCK: Audio recording is not yet available. \
         Writing unencrypted audio to the vault would violate the data-sovereignty rule. \
         Recording will be enabled after Argon2id + XChaCha20-Poly1305 vault encryption \
         is implemented and verified.".to_string())
}

#[tauri::command]
pub fn pause_recording(state: tauri::State<'_, RecordingStateHolder>) -> Result<RecordingSession, String> {
    let mut session = state.current_session.lock().map_err(|e| format!("State lock error: {}", e))?;
    if let Some(ref mut s) = *session {
        s.state = RecordingState::Paused;
        Ok(s.clone())
    } else {
        Err("No active recording session".to_string())
    }
}

#[tauri::command]
pub fn resume_recording(state: tauri::State<'_, RecordingStateHolder>) -> Result<RecordingSession, String> {
    let mut session = state.current_session.lock().map_err(|e| format!("State lock error: {}", e))?;
    if let Some(ref mut s) = *session {
        s.state = RecordingState::Recording;
        Ok(s.clone())
    } else {
        Err("No active recording session".to_string())
    }
}

#[tauri::command]
pub fn stop_recording(state: tauri::State<'_, RecordingStateHolder>) -> Result<RecordingSession, String> {
    let mut session = state.current_session.lock().map_err(|e| format!("State lock error: {}", e))?;
    let start_time = state.start_time.lock().map_err(|e| format!("State lock error: {}", e))?;

    if let Some(ref mut s) = *session {
        s.state = RecordingState::Processing;
        s.stopped_at = Some(chrono::Utc::now().to_rfc3339());

        if let Some(start) = *start_time {
            s.duration_seconds = start.elapsed().as_secs();
        }

        Ok(s.clone())
    } else {
        Err("No active recording session".to_string())
    }
}

#[tauri::command]
pub fn add_bookmark(label: Option<String>, state: tauri::State<'_, RecordingStateHolder>) -> Result<RecordingSession, String> {
    let mut session = state.current_session.lock().map_err(|e| format!("State lock error: {}", e))?;
    let start_time = state.start_time.lock().map_err(|e| format!("State lock error: {}", e))?;

    if let Some(ref mut s) = *session {
        let timestamp = if let Some(start) = *start_time {
            start.elapsed().as_secs()
        } else {
            0
        };

        s.bookmarks.push(RecordingBookmark {
            timestamp_seconds: timestamp,
            label,
        });

        Ok(s.clone())
    } else {
        Err("No active recording session".to_string())
    }
}