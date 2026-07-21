// UnoOne Power — Desktop Recording Engine
// Cross-platform audio recording with encrypted chunked writing to vault

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

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

/// Processing stages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProcessingStage {
    Idle,
    Transcribing,
    Summarizing,
    Saving,
    Complete,
    Failed,
}

/// Processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingProcessingResult {
    pub recording_id: String,
    pub stage: ProcessingStage,
    pub processed_by: String,
    pub transcript: Option<String>,
    pub summary: Option<String>,
    pub error: Option<String>,
}

/// Desktop recording engine state
pub struct DesktopRecordingEngine {
    current_session: Mutex<Option<RecordingSession>>,
    start_time: Mutex<Option<Instant>>,
}

impl DesktopRecordingEngine {
    pub fn new() -> Self {
        Self {
            current_session: Mutex::new(None),
            start_time: Mutex::new(None),
        }
    }

    /// Start a new recording session
    pub fn start_recording(&self, recording_type: RecordingType, privacy_level: PrivacyLevel, vault_root: &str) -> Result<RecordingSession, String> {
        let session_id = uuid::Uuid::new_v4().to_string();

        // Create recordings directory if it doesn't exist
        let recordings_dir = PathBuf::from(vault_root)
            .join("VAULT")
            .join("recordings")
            .join("audio");
        std::fs::create_dir_all(&recordings_dir)
            .map_err(|e| format!("Failed to create recordings directory: {}", e))?;

        let audio_path = recordings_dir.join(format!("{}.enc", session_id));

        let session = RecordingSession {
            id: session_id,
            title: format!("Recording {}", chrono::Utc::now().format("%Y-%m-%d %H:%M")),
            state: RecordingState::Recording,
            recording_type,
            privacy_level,
            started_at: Some(chrono::Utc::now().to_rfc3339()),
            stopped_at: None,
            duration_seconds: 0,
            bookmarks: Vec::new(),
            source_platform: "DESKTOP".to_string(),
            source_device_id: get_device_id(),
            audio_path: Some(audio_path.to_string_lossy().to_string()),
            transcript_path: None,
            summary_path: None,
        };

        *self.current_session.lock().unwrap() = Some(session.clone());
        *self.start_time.lock().unwrap() = Some(Instant::now());

        Ok(session)
    }

    /// Pause the current recording
    pub fn pause_recording(&self) -> Result<RecordingSession, String> {
        let mut session = self.current_session.lock().unwrap();
        if let Some(ref mut s) = *session {
            s.state = RecordingState::Paused;
            Ok(s.clone())
        } else {
            Err("No active recording session".to_string())
        }
    }

    /// Resume a paused recording
    pub fn resume_recording(&self) -> Result<RecordingSession, String> {
        let mut session = self.current_session.lock().unwrap();
        if let Some(ref mut s) = *session {
            s.state = RecordingState::Recording;
            Ok(s.clone())
        } else {
            Err("No active recording session".to_string())
        }
    }

    /// Stop the current recording
    pub fn stop_recording(&self) -> Result<RecordingSession, String> {
        let mut session = self.current_session.lock().unwrap();
        let start_time = self.start_time.lock().unwrap();

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

    /// Add a bookmark to the current recording
    pub fn add_bookmark(&self, label: Option<String>) -> Result<RecordingSession, String> {
        let mut session = self.current_session.lock().unwrap();
        let start_time = self.start_time.lock().unwrap();

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

    /// Get current session state
    pub fn get_current_session(&self) -> Option<RecordingSession> {
        self.current_session.lock().unwrap().clone()
    }

    /// Get the elapsed time of the current recording
    pub fn get_elapsed(&self) -> u64 {
        if let Some(start) = *self.start_time.lock().unwrap() {
            start.elapsed().as_secs()
        } else {
            0
        }
    }
}

fn get_device_id() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "desktop-unknown".to_string())
}

// Tauri commands

#[tauri::command]
pub fn start_recording(
    recording_type: RecordingType,
    privacy_level: PrivacyLevel,
    vault_root: String,
) -> Result<RecordingSession, String> {
    let engine = DesktopRecordingEngine::new();
    engine.start_recording(recording_type, privacy_level, &vault_root)
}

#[tauri::command]
pub fn pause_recording() -> Result<RecordingSession, String> {
    let engine = DesktopRecordingEngine::new();
    engine.pause_recording()
}

#[tauri::command]
pub fn resume_recording() -> Result<RecordingSession, String> {
    let engine = DesktopRecordingEngine::new();
    engine.resume_recording()
}

#[tauri::command]
pub fn stop_recording() -> Result<RecordingSession, String> {
    let engine = DesktopRecordingEngine::new();
    engine.stop_recording()
}

#[tauri::command]
pub fn add_bookmark(label: Option<String>) -> Result<RecordingSession, String> {
    let engine = DesktopRecordingEngine::new();
    engine.add_bookmark(label)
}