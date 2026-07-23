// UnoOne Power — Desktop Browser Workspace
// Uses Tauri's built-in WebView instead of Playwright/Chromium.
// The webview is already on the system (WebView2 on Windows, WKWebView on macOS).
// No external browser download needed — this is the pendrive-native path.

use serde::{Deserialize, Serialize};
use std::sync::Mutex;

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

/// Active browser session state
pub struct BrowserSession {
    pub active: bool,
    pub current_url: Option<String>,
    pub title: Option<String>,
}

pub struct BrowserStateHolder {
    pub session: Mutex<Option<BrowserSession>>,
}

impl BrowserStateHolder {
    pub fn new() -> Self {
        Self {
            session: Mutex::new(None),
        }
    }
}

// Tauri commands — browser workspace via built-in WebView
//
// The implementation uses Tauri's WebView2 (Windows) / WKWebView (macOS)
// to render web pages. JavaScript injection via evaluate_script()
// enables DOM querying, form filling, and text extraction.
//
// For full browser automation (screenshot, form fill, etc.), the webview
// injects a bridge script that exposes __unooneBrowserBridge with methods:
// - extractText(selector?) → extracted text content
// - click(selector) → clicks element
// - type(selector, text) → types into element
// - fillForm(fields[]) → fills multiple form fields
// - scroll(direction, amount) → scrolls the page
// - getPageInfo() → returns {url, title, readyState}

const BROWSER_BRIDGE_SCRIPT: &str = r#"
window.__unooneBrowserBridge = {
    extractText: function(selector) {
        if (selector) {
            const el = document.querySelector(selector);
            return el ? el.innerText : null;
        }
        return document.body.innerText;
    },
    click: function(selector) {
        const el = document.querySelector(selector);
        if (el) { el.click(); return true; }
        return false;
    },
    type: function(selector, text) {
        const el = document.querySelector(selector);
        if (el) {
            el.focus();
            el.value = text;
            el.dispatchEvent(new Event('input', {bubbles: true}));
            el.dispatchEvent(new Event('change', {bubbles: true}));
            return true;
        }
        return false;
    },
    fillForm: function(fields) {
        let filled = 0;
        for (const f of fields) {
            const el = document.querySelector(f.selector);
            if (el) {
                el.focus();
                el.value = f.value;
                el.dispatchEvent(new Event('input', {bubbles: true}));
                el.dispatchEvent(new Event('change', {bubbles: true}));
                filled++;
            }
        }
        return filled;
    },
    scroll: function(direction, amount) {
        const y = direction === 'down' ? amount : -amount;
        window.scrollBy(0, y);
        return true;
    },
    getPageInfo: function() {
        return JSON.stringify({
            url: window.location.href,
            title: document.title,
            readyState: document.readyState
        });
    }
};
"#;

#[tauri::command]
pub fn browser_start_session(config: Option<BrowserConfig>, state: tauri::State<'_, BrowserStateHolder>) -> Result<String, String> {
    let _config = config.unwrap_or_default();

    // Mark session as active — the actual webview window is created on the frontend
    // side via Tauri's WebviewWindow API, which is more appropriate for window management.
    let mut session = state.session.lock().map_err(|e| format!("State lock error: {}", e))?;
    *session = Some(BrowserSession {
        active: true,
        current_url: None,
        title: None,
    });

    Ok("Browser session initialized. Use frontend WebviewWindow to create the browser view. Bridge script is ready for injection.".to_string())
}

#[tauri::command]
pub fn browser_stop_session(state: tauri::State<'_, BrowserStateHolder>) -> Result<String, String> {
    let mut session = state.session.lock().map_err(|e| format!("State lock error: {}", e))?;
    *session = None;
    Ok("Browser session closed.".to_string())
}

#[tauri::command]
pub fn browser_execute(action: BrowserAction, state: tauri::State<'_, BrowserStateHolder>) -> Result<BrowserActionResult, String> {
    let session_lock = state.session.lock().map_err(|e| format!("State lock error: {}", e))?;
    if session_lock.is_none() {
        return Err("No active browser session. Call browser_start_session first.".to_string());
    }
    drop(session_lock);

    // The actual execution happens in the frontend via Tauri's evaluate_script()
    // on the webview window. This command returns the JavaScript that should be
    // executed in the webview, and the frontend handles the execution and result parsing.
    match action {
        BrowserAction::Navigate { url } => {
            Ok(BrowserActionResult {
                success: true,
                data: serde_json::json!({
                    "action": "navigate",
                    "url": url,
                    "script": format!("window.location.href = '{}'", url.replace('\'', "\\'"))
                }),
                error: None,
                screenshot_path: None,
            })
        }
        BrowserAction::Click { selector } => {
            Ok(BrowserActionResult {
                success: true,
                data: serde_json::json!({
                    "action": "click",
                    "script": format!("window.__unooneBrowserBridge.click('{}')", selector.replace('\'', "\\'"))
                }),
                error: None,
                screenshot_path: None,
            })
        }
        BrowserAction::Type { selector, text } => {
            Ok(BrowserActionResult {
                success: true,
                data: serde_json::json!({
                    "action": "type",
                    "script": format!("window.__unooneBrowserBridge.type('{}', '{}')", selector.replace('\'', "\\'"), text.replace('\'', "\\'"))
                }),
                error: None,
                screenshot_path: None,
            })
        }
        BrowserAction::ExtractText { selector } => {
            let script = match selector {
                Some(sel) => format!("window.__unooneBrowserBridge.extractText('{}')", sel.replace('\'', "\\'")),
                None => "window.__unooneBrowserBridge.extractText()".to_string(),
            };
            Ok(BrowserActionResult {
                success: true,
                data: serde_json::json!({
                    "action": "extractText",
                    "script": script
                }),
                error: None,
                screenshot_path: None,
            })
        }
        BrowserAction::Scroll { direction, amount } => {
            let dir_str = match direction {
                ScrollDirection::Up => "up",
                ScrollDirection::Down => "down",
            };
            Ok(BrowserActionResult {
                success: true,
                data: serde_json::json!({
                    "action": "scroll",
                    "script": format!("window.__unooneBrowserBridge.scroll('{}', {})", dir_str, amount)
                }),
                error: None,
                screenshot_path: None,
            })
        }
        BrowserAction::Wait { milliseconds } => {
            Ok(BrowserActionResult {
                success: true,
                data: serde_json::json!({
                    "action": "wait",
                    "milliseconds": milliseconds
                }),
                error: None,
                screenshot_path: None,
            })
        }
        BrowserAction::ExecuteScript { script } => {
            Ok(BrowserActionResult {
                success: true,
                data: serde_json::json!({
                    "action": "executeScript",
                    "script": script
                }),
                error: None,
                screenshot_path: None,
            })
        }
        BrowserAction::GetPageInfo => {
            Ok(BrowserActionResult {
                success: true,
                data: serde_json::json!({
                    "action": "getPageInfo",
                    "script": "window.__unooneBrowserBridge.getPageInfo()"
                }),
                error: None,
                screenshot_path: None,
            })
        }
        BrowserAction::FillForm { fields } => {
            let fields_json = serde_json::to_string(&fields).unwrap_or_else(|_| "[]".to_string());
            Ok(BrowserActionResult {
                success: true,
                data: serde_json::json!({
                    "action": "fillForm",
                    "script": format!("window.__unooneBrowserBridge.fillForm({})", fields_json)
                }),
                error: None,
                screenshot_path: None,
            })
        }
        BrowserAction::Screenshot => {
            // Screenshots are handled by the frontend via webview.screenshot() Tauri API
            Ok(BrowserActionResult {
                success: true,
                data: serde_json::json!({
                    "action": "screenshot",
                    "note": "Frontend should use Tauri webview.screenshot() API"
                }),
                error: None,
                screenshot_path: None,
            })
        }
    }
}

/// Get the browser bridge script that should be injected into the webview
/// on page load to enable DOM interaction.
#[tauri::command]
pub fn get_browser_bridge_script() -> String {
    BROWSER_BRIDGE_SCRIPT.to_string()
}