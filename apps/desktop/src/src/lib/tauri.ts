/**
 * UnoOne Power — Tauri API bindings
 * Type-safe bridge between React frontend and Rust backend
 */

export interface VaultInfo {
  detected: boolean;
  vault_root: string;
  vault_id: string;
}

export interface VaultUnlockResult {
  success: boolean;
  vault_id: string;
  error: string;
}

export interface VaultSetupResult {
  success: boolean;
  vault_id: string;
  recovery_key: string;
  error: string;
}

export interface HardwareProfile {
  total_ram_gb: number;
  available_ram_gb: number;
  cpu_count: number;
  cpu_speed_ghz: number;
  gpu_name: string;
  gpu_vram_gb: number;
  os_name: string;
  os_version: string;
  has_cuda: boolean;
  has_metal: boolean;
  has_vulkan: boolean;
  usb_speed: string;
}

export interface VaultStatus {
  is_connected: boolean;
  is_unlocked: boolean;
  vault_id: string;
  profile_name: string;
  used_space_gb: number;
  total_space_gb: number;
}

// Tauri invoke helper — works with @tauri-apps/api or falls back to mock
async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
    return await tauriInvoke<T>(command, args);
  } catch {
    // Mock fallback for development without Tauri runtime
    console.warn(`[Tauri Mock] ${command}`, args);
    return mockInvoke(command, args) as T;
  }
}

function mockInvoke(command: string, args?: Record<string, unknown>): unknown {
  switch (command) {
    case 'detect_vault':
      return {
        detected: true,
        vault_root: 'D:\\UNOONE',
        vault_id: 'mock-vault-id-001',
      };
    case 'unlock_vault':
      if ((args?.password as string) === 'test') {
        return {
          success: true,
          vault_id: 'mock-vault-id-001',
          error: '',
        };
      }
      return {
        success: false,
        vault_id: '',
        error: 'Incorrect password',
      };
    case 'setup_vault':
      return {
        success: true,
        vault_id: 'new-vault-' + Date.now(),
        recovery_key: 'alpha bravo charlie delta echo foxtrot golf hotel india juliet kilo lima mike november oscar papa quebec romeo sierra tango uniform victor whiskey xray yankee zulu',
        error: '',
      };
    case 'lock_vault':
      return null;
    case 'get_hardware_profile':
      return {
        total_ram_gb: 32,
        available_ram_gb: 24,
        cpu_count: 16,
        cpu_speed_ghz: 3.5,
        gpu_name: 'NVIDIA RTX 4070',
        gpu_vram_gb: 12,
        os_name: 'Windows',
        os_version: '11',
        has_cuda: true,
        has_metal: false,
        has_vulkan: true,
        usb_speed: 'USB 3.2',
      };
    case 'get_vault_status':
      return {
        is_connected: true,
        is_unlocked: true,
        vault_id: 'mock-vault-id-001',
        profile_name: 'Personal',
        used_space_gb: 8.4,
        total_space_gb: 460,
      };
    default:
      return null;
  }
}

export const tauriApi = {
  detectVault: () => invoke<VaultInfo>('detect_vault'),
  unlockVault: (password: string, vaultRoot: string) =>
    invoke<VaultUnlockResult>('unlock_vault', { password, vault_root: vaultRoot }),
  setupVault: (password: string, profileName: string | null, vaultRoot: string) =>
    invoke<VaultSetupResult>('setup_vault', { password, profile_name: profileName, vault_root: vaultRoot }),
  lockVault: () => invoke<void>('lock_vault'),
  getHardwareProfile: () => invoke<HardwareProfile>('get_hardware_profile'),
  getVaultStatus: () => invoke<VaultStatus>('get_vault_status'),
};