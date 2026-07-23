// UnoOne Power — Private AI Desktop Workstation
// Tauri backend: USB vault detection, hardware profiling, model management, safety guard, agent loop

mod llama;
mod safety;
mod recording;
mod browser;
mod documents;
mod accessibility;
mod security;
mod agent;

use std::sync::Mutex;
use std::path::PathBuf;
use unoone_vault_core::Vault;

/// D7: Rich vault state that holds the live Vault object (with decrypted master key)
/// after unlock, instead of dropping it. The Vault's Drop impl zeros the master key
/// on drop, so locking sets Option to None (which drops the Vault, zeroing the key).
struct DesktopVaultState {
    /// The live Vault struct. None when locked, Some(Vault) when unlocked.
    vault: Mutex<Option<Vault>>,
    /// Fast metadata mirrors (for reads without locking the vault mutex).
    unlocked: Mutex<bool>,
    vault_id: Mutex<String>,
    vault_root: Mutex<String>,
}

fn main() {
    let vault_state = DesktopVaultState {
        vault: Mutex::new(None),
        unlocked: Mutex::new(false),
        vault_id: Mutex::new(String::new()),
        vault_root: Mutex::new(String::new()),
    };

    let recording_state = recording::RecordingStateHolder::new();

    // D5: Safety guard held as Tauri managed state — persists across calls,
    // accumulates audit log, and respects the current security level.
    let safety_state = safety::SafetyGuardState {
        guard: Mutex::new(safety::DesktopSafetyGuard::new(safety::SecurityLevel::Standard)),
    };

    // D1: Model manager state for inference pipeline
    let model_state = llama::ModelManagerState::new();

    // D2: Agent loop state
    let agent_state = agent::AgentLoopState::new();

    tauri::Builder::default()
        .manage(vault_state)
        .manage(recording_state)
        .manage(safety_state)
        .manage(model_state)
        .manage(agent_state)
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
            llama::send_chat_completion,
            llama::check_model_health,
            // Safety guard
            safety::get_security_level,
            safety::set_security_level,
            safety::review_tool_action,
            safety::get_audit_log,
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
            // Accessibility
            accessibility::get_accessibility_status,
            accessibility::perform_ocr,
            accessibility::describe_image,
            accessibility::get_camera_info,
            // Security hardening
            security::generate_manifest,
            security::verify_manifest,
            security::recover_from_crash,
            security::emergency_lock,
            // D2: Agent loop
            agent::agent_chat,
            // D7: Vault state commands
            vault_is_unlocked,
            vault_read_record,
        ])
        .run(tauri::generate_context!())
        .expect("error while running UnoOne Power");
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

/// Scan removable drives for a valid UnoOne vault.
/// Validates via manifest.json + VERSION + vault.id — not hardcoded drive letters.
fn scan_removable_drives() -> Vec<String> {
    let mut drives = Vec::new();

    if cfg!(target_os = "windows") {
        // Enumerate all logical drives and filter to removable ones
        // Use WMI to find removable drives, then check each for UNOONE
        if let Ok(output) = std::process::Command::new("powershell")
            .args([
                "-NoProfile", "-Command",
                "Get-CimInstance Win32_LogicalDisk | Where-Object { $_.DriveType -eq 2 } | Select-Object -ExpandProperty DeviceID",
            ])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let drive = line.trim().to_string();
                    if !drive.is_empty() && drive.len() == 2 && drive.ends_with(':') {
                        drives.push(format!("{}\\", drive));
                    }
                }
            }
        }

        // Fallback: check common drive letters if WMI fails
        if drives.is_empty() {
            for letter in "DEFGHIJKLMNOP".chars() {
                let path = format!("{}:\\", letter);
                if std::path::Path::new(&path).exists() {
                    // Check if it looks like a removable drive
                    let unoone_path = std::path::Path::new(&path).join("UNOONE");
                    if unoone_path.exists() {
                        drives.push(path);
                    }
                }
            }
        }
    } else if cfg!(target_os = "macos") {
        // macOS: scan /Volumes/ for UNOONE directory
        if let Ok(entries) = std::fs::read_dir("/Volumes") {
            for entry in entries.flatten() {
                let path = entry.path();
                let unoone_path = path.join("UNOONE");
                if unoone_path.exists() {
                    drives.push(path.to_string_lossy().to_string());
                }
            }
        }
    } else {
        // Linux: scan common mount points
        for base in &["/mnt", "/media"] {
            if let Ok(entries) = std::fs::read_dir(base) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let unoone_path = path.join("UNOONE");
                    if unoone_path.exists() {
                        drives.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    drives
}

/// Validate that a directory is a legitimate UnoOne vault by checking
/// manifest.json, VERSION, and vault.id — not just the directory name.
fn validate_vault_root(vault_root: &str) -> Result<(String, String), String> {
    let root = std::path::Path::new(vault_root);
    let unoone_root = if root.join("UNOONE").exists() {
        root.join("UNOONE")
    } else if root.join("manifest.json").exists() {
        root.to_path_buf()
    } else {
        return Err("No UNOONE directory or manifest.json found".to_string());
    };

    // Check manifest.json exists and is valid JSON
    let manifest_path = unoone_root.join("manifest.json");
    if !manifest_path.exists() {
        return Err("manifest.json not found — not a valid UnoOne vault".to_string());
    }
    let manifest_content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest.json: {}", e))?;
    let _manifest: serde_json::Value = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("manifest.json is not valid JSON: {}", e))?;

    // Check VERSION file exists
    let version_path = unoone_root.join("VERSION");
    if !version_path.exists() {
        return Err("VERSION file not found — not a valid UnoOne vault".to_string());
    }

    // Check vault.id exists
    let vault_id_path = unoone_root.join("VAULT").join("identity").join("vault.id");
    if !vault_id_path.exists() {
        return Err("vault.id not found — vault not initialized".to_string());
    }
    let vault_id = std::fs::read_to_string(&vault_id_path)
        .map_err(|e| format!("Failed to read vault ID: {}", e))?
        .trim().to_string();

    Ok((unoone_root.to_string_lossy().to_string(), vault_id))
}

#[tauri::command]
#[allow(unexpected_cfgs)]
fn detect_vault() -> Result<VaultInfo, String> {
    // Scan removable drives for a valid UnoOne vault
    // Validates via manifest.json + VERSION + vault.id — not hardcoded paths
    let drives = scan_removable_drives();

    for drive_root in drives {
        if let Ok((vault_root, vault_id)) = validate_vault_root(&drive_root) {
            return Ok(VaultInfo {
                detected: true,
                vault_root,
                vault_id,
            });
        }
    }

    // NOTE: Production builds MUST NOT fall back to local/development paths.
    // The C:\UNOONE and /tmp/UNOONE fallbacks are gated behind a compile-time
    // feature flag "dev-local-vault" to prevent accidental use in production.
    // Only removable, validated USB volumes are accepted in production builds.
    #[cfg(feature = "dev-local-vault")]
    {
        let fallback_paths = if cfg!(target_os = "windows") {
            vec!["C:\\UNOONE"]
        } else if cfg!(target_os = "macos") {
            vec!["/tmp/UNOONE"]
        } else {
            vec!["/tmp/UNOONE"]
        };

        for path in fallback_paths {
            if let Ok((vault_root, vault_id)) = validate_vault_root(path) {
                return Ok(VaultInfo {
                    detected: true,
                    vault_root,
                    vault_id,
                });
            }
        }
    }

    Ok(VaultInfo {
        detected: false,
        vault_root: String::new(),
        vault_id: String::new(),
    })
}

#[tauri::command]
fn unlock_vault(password: String, vault_root: String, state: tauri::State<'_, DesktopVaultState>) -> Result<VaultUnlockResult, String> {
    if password.is_empty() {
        return Ok(VaultUnlockResult {
            success: false,
            vault_id: String::new(),
            error: "Password cannot be empty".to_string(),
        });
    }

    if vault_root.is_empty() {
        return Ok(VaultUnlockResult {
            success: false,
            vault_id: String::new(),
            error: "No vault root specified".to_string(),
        });
    }

    // Use vault-core to unlock the vault with Argon2id key derivation
    // and XChaCha20-Poly1305 authenticated encryption.
    // D7: The Vault object is stored in Tauri managed state so it persists
    // after unlock — the decrypted master key remains in memory for vault operations.
    let vault_path = PathBuf::from(&vault_root);
    let mut vault = unoone_vault_core::Vault::open(&vault_path)
        .map_err(|e| format!("Failed to open vault: {}", e))?;

    match vault.unlock(password.as_bytes()) {
        Ok(result) => {
            // Store the live Vault in managed state (not dropped!)
            *state.vault.lock().map_err(|e| format!("State lock error: {}", e))? = Some(vault);
            *state.unlocked.lock().map_err(|e| format!("State lock error: {}", e))? = true;
            *state.vault_id.lock().map_err(|e| format!("State lock error: {}", e))? = result.vault_id.clone();
            *state.vault_root.lock().map_err(|e| format!("State lock error: {}", e))? = vault_root.clone();

            Ok(VaultUnlockResult {
                success: true,
                vault_id: result.vault_id,
                error: String::new(),
            })
        }
        Err(unoone_vault_core::VaultError::WrongPassword) => {
            Ok(VaultUnlockResult {
                success: false,
                vault_id: String::new(),
                error: "Wrong password".to_string(),
            })
        }
        Err(e) => {
            Ok(VaultUnlockResult {
                success: false,
                vault_id: String::new(),
                error: format!("Unlock failed: {}", e),
            })
        }
    }
}

#[tauri::command]
fn setup_vault(password: String, _profile_name: Option<String>, vault_root: String) -> Result<VaultSetupResult, String> {
    if password.len() < 8 {
        return Ok(VaultSetupResult {
            success: false,
            vault_id: String::new(),
            recovery_key: String::new(),
            error: "Password must be at least 8 characters".to_string(),
        });
    }

    if vault_root.is_empty() {
        return Ok(VaultSetupResult {
            success: false,
            vault_id: String::new(),
            recovery_key: String::new(),
            error: "No vault root specified".to_string(),
        });
    }

    // Use vault-core to create a new vault with Argon2id key derivation
    // and XChaCha20-Poly1305 authenticated encryption
    let vault_path = PathBuf::from(&vault_root);

    match unoone_vault_core::Vault::create(&vault_path, password.as_bytes()) {
        Ok(result) => {
            Ok(VaultSetupResult {
                success: true,
                vault_id: result.vault_id,
                recovery_key: result.recovery_phrase.join(" "),
                error: String::new(),
            })
        }
        Err(unoone_vault_core::VaultError::InvalidPassword(msg)) => {
            Ok(VaultSetupResult {
                success: false,
                vault_id: String::new(),
                recovery_key: String::new(),
                error: msg,
            })
        }
        Err(e) => {
            Ok(VaultSetupResult {
                success: false,
                vault_id: String::new(),
                recovery_key: String::new(),
                error: format!("Vault creation failed: {}", e),
            })
        }
    }
}

#[tauri::command]
fn lock_vault(state: tauri::State<'_, DesktopVaultState>) -> Result<(), String> {
    // D7: Properly lock and drop the Vault, zeroing the master key.
    let mut vault_opt = state.vault.lock().map_err(|e| format!("State lock error: {}", e))?;

    if let Some(vault) = vault_opt.as_mut() {
        // Vault::lock() zeros the master key via secure_zero.
        vault.lock().map_err(|e| format!("Lock failed: {}", e))?;
    }

    // Drop the Vault — its Drop impl also zeros any remaining key material.
    *vault_opt = None;

    // Clear metadata mirrors
    *state.unlocked.lock().map_err(|e| format!("State lock error: {}", e))? = false;
    state.vault_id.lock().map_err(|e| format!("State lock error: {}", e))?.clear();
    state.vault_root.lock().map_err(|e| format!("State lock error: {}", e))?.clear();

    Ok(())
}

/// D7: Check if the vault is currently unlocked (fast metadata read, no vault lock needed).
#[tauri::command]
fn vault_is_unlocked(state: tauri::State<'_, DesktopVaultState>) -> bool {
    *state.unlocked.lock().unwrap()
}

/// D7: Read a record from the unlocked vault. Returns the decrypted content as a string.
#[tauri::command]
fn vault_read_record(record_id: String, state: tauri::State<'_, DesktopVaultState>) -> Result<String, String> {
    let vault_opt = state.vault.lock().map_err(|e| format!("State lock error: {}", e))?;
    let vault = vault_opt.as_ref().ok_or("Vault is not unlocked")?;

    let (_record, plaintext) = vault.read_record(&record_id)
        .map_err(|e| format!("Read failed: {}", e))?;

    String::from_utf8(plaintext)
        .map_err(|e| format!("Record content is not valid UTF-8: {}", e))
}

#[tauri::command]
fn get_hardware_profile() -> Result<HardwareProfile, String> {
    let total_ram_bytes = sys_info::mem_info()
        .map(|m| m.total * 1024)
        .unwrap_or(0);
    let total_ram_gb = total_ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    let avail_ram_bytes = sys_info::mem_info()
        .map(|m| m.avail * 1024)
        .unwrap_or(0);
    let available_ram_gb = avail_ram_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    let cpu_count = num_cpus::get();

    let os_name = sys_info::os_type().unwrap_or_else(|_| "Unknown".to_string());
    let os_version = sys_info::os_release().unwrap_or_else(|_| "Unknown".to_string());

    // Detect GPU via nvidia-smi
    let (gpu_name, gpu_vram_gb, has_cuda) = detect_gpu();

    // Detect Vulkan via DLL/so presence
    let has_vulkan = if cfg!(target_os = "windows") {
        std::path::Path::new("C:\\Windows\\System32\\vulkan-1.dll").exists()
    } else if cfg!(target_os = "linux") {
        std::path::Path::new("/usr/lib/x86_64-linux-gnu/libvulkan.so").exists()
            || std::path::Path::new("/usr/lib/libvulkan.so").exists()
    } else {
        false
    };

    // Detect USB speed by checking the vault drive
    let usb_speed = detect_usb_speed();

    Ok(HardwareProfile {
        total_ram_gb: (total_ram_gb * 10.0).round() / 10.0,
        available_ram_gb: (available_ram_gb * 10.0).round() / 10.0,
        cpu_count,
        cpu_speed_ghz: 0.0, // Not easily available on all platforms
        gpu_name,
        gpu_vram_gb,
        os_name,
        os_version,
        has_cuda,
        has_metal: cfg!(target_os = "macos"),
        has_vulkan,
        usb_speed,
    })
}

fn detect_gpu() -> (String, f64, bool) {
    // Try nvidia-smi for NVIDIA GPU detection
    if let Ok(output) = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total", "--format=csv,noheader,nounits"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let line = stdout.lines().next().unwrap_or("");
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                let name = parts[0].trim().to_string();
                let vram: f64 = parts[1].trim().parse().unwrap_or(0.0);
                return (name, vram, true);
            }
        }
    }

    (String::new(), 0.0, false)
}

fn detect_usb_speed() -> String {
    // On Windows, check USB drive speed via WMI
    if cfg!(target_os = "windows") {
        if let Ok(output) = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-CimInstance Win32_DiskDrive | Where-Object { $_.InterfaceType -eq 'USB' } | Select-Object -ExpandProperty MediaType"
            ])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let speed = stdout.trim();
                if !speed.is_empty() {
                    return speed.to_string();
                }
            }
        }

        // Fallback: check if a validated UNOONE vault exists on any drive
        let drives = scan_removable_drives();
        for drive in &drives {
            if let Ok((vault_root, _)) = validate_vault_root(drive) {
                // Found a valid vault — report as USB 3.0+
                let _ = vault_root; // used for validation only
                return "USB 3.0+".to_string();
            }
        }
    }

    "Unknown".to_string()
}

#[tauri::command]
fn get_vault_status(state: tauri::State<'_, DesktopVaultState>) -> Result<VaultStatus, String> {
    let vault_root = state.vault_root.lock().map_err(|e| format!("State lock error: {}", e))?;
    let unlocked = *state.unlocked.lock().map_err(|e| format!("State lock error: {}", e))?;
    let vault_id = state.vault_id.lock().map_err(|e| format!("State lock error: {}", e))?;

    if vault_root.is_empty() {
        return Ok(VaultStatus {
            is_connected: false,
            is_unlocked: false,
            vault_id: String::new(),
            profile_name: String::new(),
            used_space_gb: 0.0,
            total_space_gb: 0.0,
        });
    }

    // Calculate real disk usage
    let vault_path = std::path::Path::new(&*vault_root);
    let used_space_bytes = dir_size(vault_path).unwrap_or(0);
    let used_space_gb = used_space_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    // Get total disk space for the drive
    let total_space_gb = std::fs::metadata(vault_path)
        .ok()
        .and_then(|_m| {
            // Try to get filesystem stats
            std::fs::metadata(vault_path).ok()
        })
        .and_then(|_| {
            // Use sys_info for disk stats
            sys_info::disk_info().ok()
        })
        .map(|d| d.total as f64 / (1024.0 * 1024.0 * 1024.0))
        .unwrap_or(0.0);

    // SECURITY: Profile name is stored inside the encrypted vault.
    // Until vault encryption is implemented, profile_name is empty.
    // Reading plaintext profile.txt from an unencrypted vault would
    // violate the data-sovereignty rule (directive section 11).
    let profile_name = String::new();

    Ok(VaultStatus {
        is_connected: true,
        is_unlocked: unlocked,
        vault_id: vault_id.clone(),
        profile_name,
        used_space_gb: (used_space_gb * 10.0).round() / 10.0,
        total_space_gb: (total_space_gb * 10.0).round() / 10.0,
    })
}

/// Recursively compute directory size in bytes
fn dir_size(path: &std::path::Path) -> Result<u64, String> {
    let mut total: u64 = 0;
    if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    total += dir_size(&entry_path)?;
                } else if let Ok(metadata) = entry.metadata() {
                    total += metadata.len();
                }
            }
        }
    }
    Ok(total)
}