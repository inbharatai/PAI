// UnoOne Power — Desktop Browser Workspace
// Manages Chromium browser instances for PageAgent-based web interaction
// All browser actions go through SafetyGuard before execution

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

/// Browser workspace state
pub struct BrowserWorkspace {
    config: BrowserConfig,
    session_active: bool,
    current_url: String,
}

impl BrowserWorkspace {
    pub fn new(config: BrowserConfig) -> Self {
        Self {
            config,
            session_active: false,
            current_url: String::new(),
        }
    }

    /// Start a browser session
    /// In production, this launches Chromium via Playwright
    pub fn start_session(&mut self) -> Result<(), String> {
        // TODO: Launch Chromium process with Playwright
        // For now, mark as active
        self.session_active = true;
        Ok(())
    }

    /// Stop the browser session
    pub fn stop_session(&mut self) -> Result<(), String> {
        self.session_active = false;
        self.current_url = String::new();
        Ok(())
    }

    /// Execute a browser action (after SafetyGuard approval)
    pub fn execute_action(&mut self, action: &BrowserAction) -> Result<BrowserActionResult, String> {
        if !self.session_active {
            return Err("No active browser session".to_string());
        }

        match action {
            BrowserAction::Navigate { url } => {
                self.current_url = url.clone();
                Ok(BrowserActionResult {
                    success: true,
                    data: serde_json::json!({ "url": url, "status": "navigating" }),
                    error: None,
                    screenshot_path: None,
                })
            }
            BrowserAction::Click { selector } => {
                Ok(BrowserActionResult {
                    success: true,
                    data: serde_json::json!({ "selector": selector, "action": "clicked" }),
                    error: None,
                    screenshot_path: None,
                })
            }
            BrowserAction::Type { selector, text } => {
                Ok(BrowserActionResult {
                    success: true,
                    data: serde_json::json!({ "selector": selector, "text_length": text.len() }),
                    error: None,
                    screenshot_path: None,
                })
            }
            BrowserAction::Screenshot => {
                Ok(BrowserActionResult {
                    success: true,
                    data: serde_json::json!({ "screenshot": "placeholder" }),
                    error: None,
                    screenshot_path: Some("screenshot_placeholder.png".to_string()),
                })
            }
            BrowserAction::ExtractText { selector } => {
                Ok(BrowserActionResult {
                    success: true,
                    data: serde_json::json!({ "text": "Page text extraction placeholder", "selector": selector }),
                    error: None,
                    screenshot_path: None,
                })
            }
            BrowserAction::Scroll { direction, amount } => {
                Ok(BrowserActionResult {
                    success: true,
                    data: serde_json::json!({ "direction": format!("{:?}", direction), "amount": amount }),
                    error: None,
                    screenshot_path: None,
                })
            }
            BrowserAction::Wait { milliseconds } => {
                Ok(BrowserActionResult {
                    success: true,
                    data: serde_json::json!({ "waited_ms": milliseconds }),
                    error: None,
                    screenshot_path: None,
                })
            }
            BrowserAction::ExecuteScript { script } => {
                Ok(BrowserActionResult {
                    success: true,
                    data: serde_json::json!({ "script_length": script.len() }),
                    error: None,
                    screenshot_path: None,
                })
            }
            BrowserAction::GetPageInfo => {
                Ok(BrowserActionResult {
                    success: true,
                    data: serde_json::json!({
                        "url": &self.current_url,
                        "title": "Page title placeholder"
                    }),
                    error: None,
                    screenshot_path: None,
                })
            }
            BrowserAction::FillForm { fields } => {
                Ok(BrowserActionResult {
                    success: true,
                    data: serde_json::json!({ "fields_filled": fields.len() }),
                    error: None,
                    screenshot_path: None,
                })
            }
        }
    }

    pub fn is_active(&self) -> bool {
        self.session_active
    }

    pub fn get_config(&self) -> &BrowserConfig {
        &self.config
    }
}

// Tauri commands

#[tauri::command]
pub fn browser_start_session(config: Option<BrowserConfig>) -> Result<String, String> {
    let cfg = config.unwrap_or_default();
    let mut workspace = BrowserWorkspace::new(cfg);
    workspace.start_session()?;
    Ok("Browser session started".to_string())
}

#[tauri::command]
pub fn browser_stop_session() -> Result<String, String> {
    let mut workspace = BrowserWorkspace::new(BrowserConfig::default());
    workspace.stop_session()
}

#[tauri::command]
pub fn browser_execute(action: BrowserAction) -> Result<BrowserActionResult, String> {
    let mut workspace = BrowserWorkspace::new(BrowserConfig::default());
    workspace.execute_action(&action)
}