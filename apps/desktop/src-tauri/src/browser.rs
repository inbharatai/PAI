// UnoOne Power — Desktop Browser Workspace
// Manages Chromium browser instances for PageAgent-based web interaction
// All browser actions go through SafetyGuard before execution
//
// STATUS: Browser workspace structure is defined but Chromium/Playwright
// integration is not yet wired. Commands return honest "not available" errors.

use serde::{Deserialize, Serialize};

/// Browser session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub headless: bool,
    pub user_data_dir: Option<String>,
    pub proxy: Option<String>,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub disable_images: bool,
    pub disable_javascript: bool,
    pub accept_languages: String,
    pub user_agent: Option<String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            user_data_dir: None,
            proxy: None,
            viewport_width: 1280,
            viewport_height: 800,
            disable_images: false,
            disable_javascript: false,
            accept_languages: "en-US,en;q=0.9".to_string(),
            user_agent: None,
        }
    }
}

/// Browser action (from SafetyGuard-approved ToolAction)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BrowserAction {
    Navigate { url: String },
    Click { selector: String },
    Type { selector: String, text: String },
    Screenshot,
    ExtractText { selector: Option<String> },
    Scroll { direction: ScrollDirection, amount: u32 },
    Wait { milliseconds: u64 },
    ExecuteScript { script: String },
    GetPageInfo,
    FillForm { fields: Vec<FormFillField> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormFillField {
    pub selector: String,
    pub value: String,
}

/// Browser action result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserActionResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
    pub screenshot_path: Option<String>,
}

/// Page information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub url: String,
    pub title: String,
    pub status_code: u16,
    pub content_type: Option<String>,
    pub load_time_ms: u64,
}

// Tauri commands — all return honest "not available" until Playwright is integrated

#[tauri::command]
pub fn browser_start_session(_config: Option<BrowserConfig>) -> Result<String, String> {
    Err("Browser workspace is not yet available. Chromium/Playwright integration is pending.".to_string())
}

#[tauri::command]
pub fn browser_stop_session() -> Result<String, String> {
    Err("No active browser session".to_string())
}

#[tauri::command]
pub fn browser_execute(_action: BrowserAction) -> Result<BrowserActionResult, String> {
    Err("Browser workspace is not yet available. Chromium/Playwright integration is pending.".to_string())
}