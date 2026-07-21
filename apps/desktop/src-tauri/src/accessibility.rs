// UnoOne Power — Desktop Accessibility Adapters
// Blind View (screen reader), OCR, camera adapters for desktop

use serde::{Deserialize, Serialize};

/// Accessibility feature status
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
    AccessibilityStatus {
        screen_reader_detected: false,
        high_contrast: false,
        reduced_motion: false,
        font_scale: 1.0,
        screen_reader_name: "Unknown".to_string(),
    }
}

#[tauri::command]
pub fn perform_ocr(image_path: String) -> Result<OcrResult, String> {
    // TODO: Implement actual OCR using Tesseract or bundled ML Kit
    Ok(OcrResult {
        text: "[OCR text extraction placeholder]".to_string(),
        confidence: 0.0,
        language: "en".to_string(),
        processing_time_ms: 0,
    })
}

#[tauri::command]
pub fn describe_image(image_path: String) -> Result<BlindViewResult, String> {
    // TODO: Implement image description using Gemma 4 12B vision
    Ok(BlindViewResult {
        description: "[Image description placeholder]".to_string(),
        objects: vec![],
        confidence: 0.0,
    })
}

#[tauri::command]
pub fn get_camera_info() -> Result<CameraFrame, String> {
    Ok(CameraFrame {
        width: 1920,
        height: 1080,
        format: "RGB24".to_string(),
        timestamp_ms: 0,
    })
}