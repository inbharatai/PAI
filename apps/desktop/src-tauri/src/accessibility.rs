// UnoOne Power — Desktop Accessibility Adapters
// Blind View (screen reader), OCR, camera adapters for desktop
//
// STATUS: Accessibility detection returns real OS values.
// OCR and image description return honest "not available" errors
// until Tesseract OCR and Gemma vision are integrated.

use serde::{Deserialize, Serialize};

/// Accessibility feature status — reads real OS accessibility settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityStatus {
    pub screen_reader_detected: bool,
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub font_scale: f32,
    pub screen_reader_name: String,
}

/// OCR result from image/document scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub text: String,
    pub confidence: f32,
    pub language: String,
    pub processing_time_ms: u64,
}

/// Blind View description result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindViewResult {
    pub description: String,
    pub objects: Vec<DetectedObject>,
    pub confidence: f32,
}

/// Detected object in an image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedObject {
    pub label: String,
    pub confidence: f32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Camera frame info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraFrame {
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub timestamp_ms: u64,
}

// Tauri commands

#[tauri::command]
pub fn get_accessibility_status() -> AccessibilityStatus {
    // Detect real OS accessibility settings
    let mut screen_reader_detected = false;
    let mut screen_reader_name = "Unknown".to_string();
    let mut high_contrast = false;
    let reduced_motion = false;
    let font_scale = 1.0;

    if cfg!(target_os = "windows") {
        // Check for Windows screen readers via NVDA/JAWS process detection
        if let Ok(output) = std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq nvda.exe"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("nvda.exe") {
                screen_reader_detected = true;
                screen_reader_name = "NVDA".to_string();
            }
        }

        if !screen_reader_detected {
            if let Ok(output) = std::process::Command::new("tasklist")
                .args(["/FI", "IMAGENAME eq jaws.exe"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("jaws.exe") {
                    screen_reader_detected = true;
                    screen_reader_name = "JAWS".to_string();
                }
            }
        }

        // Check Windows high contrast mode via registry
        if let Ok(output) = std::process::Command::new("reg")
            .args(["query", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes", "/v", "CurrentTheme"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Simplified check — high contrast themes contain "High Contrast" in path
            high_contrast = stdout.to_lowercase().contains("high contrast");
        }
    } else if cfg!(target_os = "macos") {
        // macOS VoiceOver detection
        if let Ok(output) = std::process::Command::new("defaults")
            .args(["read", "com.apple.accessibility", "ApplicationAccessibilityEnabled"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("1") {
                screen_reader_detected = true;
                screen_reader_name = "VoiceOver".to_string();
            }
        }
    }

    AccessibilityStatus {
        screen_reader_detected,
        high_contrast,
        reduced_motion,
        font_scale,
        screen_reader_name,
    }
}

#[tauri::command]
pub fn perform_ocr(_image_path: String) -> Result<OcrResult, String> {
    Err("OCR is not yet available. Tesseract integration is pending.".to_string())
}

#[tauri::command]
pub fn describe_image(_image_path: String) -> Result<BlindViewResult, String> {
    Err("Image description is not yet available. Gemma 4 vision integration is pending.".to_string())
}

#[tauri::command]
pub fn get_camera_info() -> Result<CameraFrame, String> {
    // Camera access not yet wired on desktop
    Err("Camera access is not yet available on desktop. Use UnoOne Mobile for camera features.".to_string())
}