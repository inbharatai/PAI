// UnoOne Power — Private AI Desktop Workstation
// Tauri backend: USB vault detection, hardware profiling, model management, safety guard

mod llama;
mod safety;
mod recording;
mod browser;
mod documents;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // Vault commands
            detect_vault,
            unlock_vault,
            setup_vault,
            lock_vault,
            get_vault_status,
            // Hardware profile
            get_hardware_profile,
            // Model management
            llama::list_models,
            llama::detect_acceleration,
            llama::get_model_config,
            llama::get_model_status,
            // Safety guard
            safety::get_security_level,
            safety::set_security_level,
            safety::review_tool_action,
            // Recording
            recording::start_recording,
            recording::pause_recording,
            recording::resume_recording,
            recording::stop_recording,
            recording::add_bookmark,
            // Browser workspace
            browser::browser_start_session,
            browser::browser_stop_session,
            browser::browser_execute,
            // Documents and memory
            documents::list_documents,
            documents::process_document,
            documents::search_memories,
        ])
        .run(tauri::generate_context!())
        .expect("error while running UnoOne Power");
}

#[tauri::command]
fn detect_vault() -> Result<VaultInfo, String> {
    // Detect UnoOne Pocket USB drive
    // Check for D:\UNOONE\VAULT\identity\vault.id (Windows)
    // Check for /Volumes/PAI/UNOONE/VAULT/identity/vault.id (macOS)
    let vault_paths = if cfg!(target_os = "windows") {
        vec!["D:\\UNOONE", "E:\\UNOONE", "F:\\UNOONE"]
    } else if cfg!(target_os = "macos") {
        vec!["/Volumes/PAI/UNOONE", "/Volumes/UNOONE"]
    } else {
        vec!["/mnt/usb/UNOONE", "/media/usb/UNOONE"]
    };

    for path in vault_paths {
        let vault_id_path = std::path::Path::new(path).join("VAULT").join("identity").join("vault.id");
        if vault_id_path.exists() {
            let vault_id = std::fs::read_to_string(&vault_id_path)
                .map_err(|e| format!("Failed to read vault ID: {}", e))?;
            return Ok(VaultInfo {
                detected: true,
                vault_root: path.to_string(),
                vault_id: vault_id.trim().to_string(),
            });
        }
    }

    Ok(VaultInfo {
        detected: false,
        vault_root: String::new(),
        vault_id: String::new(),
    })
}

#[tauri::command]
fn unlock_vault(password: String, vault_root: String) -> Result<VaultUnlockResult, String> {
    // This will call the Kotlin/Native or JVM vault library
    // For now, return a placeholder
    Ok(VaultUnlockResult {
        success: false,
        vault_id: String::new(),
        error: "Vault unlock not yet implemented in Rust backend".to_string(),
    })
}

#[tauri::command]
fn setup_vault(password: String, profile_name: Option<String>, vault_root: String) -> Result<VaultSetupResult, String> {
    Ok(VaultSetupResult {
        success: false,
        vault_id: String::new(),
        recovery_key: String::new(),
        error: "Vault setup not yet implemented in Rust backend".to_string(),
    })
}

#[tauri::command]
fn lock_vault() -> Result<(), String> {
    // Clear all keys from memory
    Ok(())
}

#[tauri::command]
fn get_hardware_profile() -> Result<HardwareProfile, String> {
    let total_ram_bytes = sys_info::mem_info()
        .map(|m| m.total * 1024)
        .unwrap_or(0);
    let total_ram_gb = total_ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    let cpu_count = num_cpus::get();
    let cpu_speed_ghz = 0.0; // Would need platform-specific detection

    let os_name = sys_info::os_type().unwrap_or_else(|_| "Unknown".to_string());
    let os_version = sys_info::os_release().unwrap_or_else(|_| "Unknown".to_string());

    Ok(HardwareProfile {
        total_ram_gb,
        available_ram_gb: total_ram_gb, // Approximation
        cpu_count,
        cpu_speed_ghz,
        gpu_name: String::new(), // Would need platform-specific detection
        gpu_vram_gb: 0.0,
        os_name,
        os_version,
        has_cuda: false, // Will be detected by llama module
        has_metal: cfg!(target_os = "macos"),
        has_vulkan: false, // Will be detected by llama module
        usb_speed: String::new(),
    })
}

#[tauri::command]
fn get_vault_status() -> Result<VaultStatus, String> {
    Ok(VaultStatus {
        is_connected: false,
        is_unlocked: false,
        vault_id: String::new(),
        profile_name: String::new(),
        used_space_gb: 0.0,
        total_space_gb: 0.0,
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
struct VaultInfo {
    detected: bool,
    vault_root: String,
    vault_id: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct VaultUnlockResult {
    success: bool,
    vault_id: String,
    error: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct VaultSetupResult {
    success: bool,
    vault_id: String,
    recovery_key: String,
    error: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct HardwareProfile {
    total_ram_gb: f64,
    available_ram_gb: f64,
    cpu_count: usize,
    cpu_speed_ghz: f64,
    gpu_name: String,
    gpu_vram_gb: f64,
    os_name: String,
    os_version: String,
    has_cuda: bool,
    has_metal: bool,
    has_vulkan: bool,
    usb_speed: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct VaultStatus {
    is_connected: bool,
    is_unlocked: bool,
    vault_id: String,
    profile_name: String,
    used_space_gb: f64,
    total_space_gb: f64,
}