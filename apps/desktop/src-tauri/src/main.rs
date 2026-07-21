// UnoOne Power — Private AI Desktop Workstation
// Tauri backend: USB vault detection, hardware profiling, model management, safety guard

mod llama;
mod safety;
mod recording;
mod browser;
mod documents;
mod accessibility;
mod security;

use std::sync::Mutex;

/// Shared vault state for cross-command persistence
struct VaultState {
    unlocked: bool,
    vault_id: String,
    vault_root: String,
}

fn main() {
    let vault_state = Mutex::new(VaultState {
        unlocked: false,
        vault_id: String::new(),
        vault_root: String::new(),
    });

    let recording_state = recording::RecordingStateHolder::new();

    tauri::Builder::default()
        .manage(vault_state)
        .manage(recording_state)
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
fn unlock_vault(password: String, vault_root: String, state: tauri::State<'_, Mutex<VaultState>>) -> Result<VaultUnlockResult, String> {
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

    // Read the vault ID to verify the vault exists
    let vault_id_path = std::path::Path::new(&vault_root)
        .join("VAULT")
        .join("identity")
        .join("vault.id");

    if !vault_id_path.exists() {
        return Ok(VaultUnlockResult {
            success: false,
            vault_id: String::new(),
            error: "Vault not found at specified path".to_string(),
        });
    }

    let vault_id = std::fs::read_to_string(&vault_id_path)
        .map_err(|e| format!("Failed to read vault ID: {}", e))?
        .trim().to_string();

    // TODO: Implement real Argon2id key derivation + vault decryption verification
    // This requires integrating the Kotlin/Native vault library or a Rust Argon2id implementation
    // For now, store the unlocked state so the UI can proceed
    let mut vault_state = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    vault_state.unlocked = true;
    vault_state.vault_id = vault_id.clone();
    vault_state.vault_root = vault_root.clone();

    Ok(VaultUnlockResult {
        success: true,
        vault_id,
        error: String::new(),
    })
}

#[tauri::command]
fn setup_vault(password: String, profile_name: Option<String>, vault_root: String) -> Result<VaultSetupResult, String> {
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

    // Create the vault directory structure
    let vault_dirs = ["VAULT/identity", "VAULT/memory", "VAULT/chats", "VAULT/recordings/audio",
                      "VAULT/recordings/transcripts", "VAULT/documents", "VAULT/settings",
                      "VAULT/audit", "VAULT/indexes/journal",
                      "MODELS/gemma4-12b-q4-gguf", "MODELS/gemma4-e2b",
                      "RUNTIMES/windows", "RUNTIMES/macos"];

    for dir in &vault_dirs {
        let path = std::path::Path::new(&vault_root).join(dir);
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create directory {}: {}", dir, e))?;
    }

    // Generate a new vault ID
    let vault_id = uuid::Uuid::new_v4().to_string();

    // Write the vault ID file
    let vault_id_path = std::path::Path::new(&vault_root)
        .join("VAULT")
        .join("identity")
        .join("vault.id");
    std::fs::write(&vault_id_path, &vault_id)
        .map_err(|e| format!("Failed to write vault ID: {}", e))?;

    // Write a profile name file if provided
    if let Some(name) = profile_name {
        let profile_path = std::path::Path::new(&vault_root)
            .join("VAULT")
            .join("identity")
            .join("profile.txt");
        std::fs::write(&profile_path, &name)
            .map_err(|e| format!("Failed to write profile: {}", e))?;
    }

    // Generate a 12-word recovery key (simplified — production uses BIP-39 wordlist)
    let recovery_words: Vec<String> = (0..12)
        .map(|_| uuid::Uuid::new_v4().to_string().split('-').next().unwrap().to_string())
        .collect();
    let recovery_key = recovery_words.join(" ");

    // TODO: Implement real Argon2id key derivation + XChaCha20-Poly1305 vault encryption
    // The password should derive a master key which encrypts a verification blob
    // For now, the vault structure is created and the ID is written

    Ok(VaultSetupResult {
        success: true,
        vault_id,
        recovery_key,
        error: String::new(),
    })
}

#[tauri::command]
fn lock_vault(state: tauri::State<'_, Mutex<VaultState>>) -> Result<(), String> {
    let mut vault_state = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    vault_state.unlocked = false;
    vault_state.vault_id.clear();
    vault_state.vault_root.clear();
    Ok(())
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

        // Fallback: check if UNOONE drive exists
        for drive in &["D:\\", "E:\\", "F:\\"] {
            let vault_id = std::path::Path::new(drive)
                .join("UNOONE")
                .join("VAULT")
                .join("identity")
                .join("vault.id");
            if vault_id.exists() {
                return "USB 3.0+".to_string(); // Assume USB 3.0+ if vault detected
            }
        }
    }

    "Unknown".to_string()
}

#[tauri::command]
fn get_vault_status(state: tauri::State<'_, Mutex<VaultState>>) -> Result<VaultStatus, String> {
    let vault_state = state.lock().map_err(|e| format!("State lock error: {}", e))?;

    if vault_state.vault_root.is_empty() {
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
    let vault_path = std::path::Path::new(&vault_state.vault_root);
    let used_space_bytes = dir_size(vault_path).unwrap_or(0);
    let used_space_gb = used_space_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    // Get total disk space for the drive
    let total_space_gb = std::fs::metadata(vault_path)
        .ok()
        .and_then(|m| {
            // Try to get filesystem stats
            std::fs::metadata(vault_path).ok()
        })
        .and_then(|_| {
            // Use sys_info for disk stats
            sys_info::disk_info().ok()
        })
        .map(|d| d.total as f64 / (1024.0 * 1024.0 * 1024.0))
        .unwrap_or(0.0);

    // Read profile name if available
    let profile_name = std::path::Path::new(&vault_state.vault_root)
        .join("VAULT")
        .join("identity")
        .join("profile.txt")
        .exists()
        .then(|| {
            std::fs::read_to_string(
                std::path::Path::new(&vault_state.vault_root)
                    .join("VAULT")
                    .join("identity")
                    .join("profile.txt")
            ).unwrap_or_default()
        })
        .unwrap_or_default();

    Ok(VaultStatus {
        is_connected: true,
        is_unlocked: vault_state.unlocked,
        vault_id: vault_state.vault_id.clone(),
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