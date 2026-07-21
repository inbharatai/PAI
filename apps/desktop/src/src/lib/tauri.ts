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

export interface ModelInfo {
  name: string;
  model_type: string;
  quantization: string;
  file_size_gb: number;
  context_length: number;
  available: boolean;
  path: string;
}

export interface ModelConfig {
  model_path: string;
  context_size: number;
  batch_size: number;
  threads: number;
  gpu_layers: number;
  temperature: number;
  top_p: number;
  top_k: number;
  repeat_penalty: number;
  max_tokens: number;
}

export interface ConversationTurn {
  role: 'user' | 'assistant';
  content: string;
}

export interface InferenceRequest {
  prompt: string;
  system_prompt?: string;
  conversation_history: ConversationTurn[];
  max_tokens?: number;
  temperature?: number;
  stop_sequences?: string[];
}

export interface InferenceResponse {
  text: string;
  tokens_generated: number;
  tokens_per_second: number;
  model_id: string;
}

export type AccelerationBackend = 'CUDA' | 'METAL' | 'VULKAN' | 'CPU';
export type SecurityLevel = 'STANDARD' | 'RELAXED' | 'OFF';
export type ModelStatus = 'NOT_LOADED' | 'LOADING' | 'LOADED' | 'GENERATING' | 'ERROR';

export interface ToolAction {
  action_id: string;
  tool_name: string;
  parameters: Record<string, unknown>;
  confidence: number;
  raw_output: string;
}

export interface SafetyVerdict {
  action_id: string;
  approved: boolean;
  reason: string;
  risk_level: 'SAFE' | 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
  modified_parameters: Record<string, unknown> | null;
}

export interface RecordingBookmark {
  timestamp_seconds: number;
  label: string | null;
}

export interface RecordingSession {
  id: string;
  title: string;
  state: 'IDLE' | 'RECORDING' | 'PAUSED' | 'PROCESSING' | 'STOPPED' | 'ERROR';
  recording_type: string;
  privacy_level: string;
  started_at: string | null;
  stopped_at: string | null;
  duration_seconds: number;
  bookmarks: RecordingBookmark[];
  source_platform: string;
  source_device_id: string;
  audio_path: string | null;
  transcript_path: string | null;
  summary_path: string | null;
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
        vault_id: '440d5ce9-7fb1-4858-a8b4-8102691009ca',
      };
    case 'unlock_vault':
      if ((args?.password as string) === 'test') {
        return { success: true, vault_id: '440d5ce9-7fb1-4858-a8b4-8102691009ca', error: '' };
      }
      return { success: false, vault_id: '', error: 'Incorrect password' };
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
        vault_id: '440d5ce9-7fb1-4858-a8b4-8102691009ca',
        profile_name: 'Personal',
        used_space_gb: 8.4,
        total_space_gb: 460,
      };
    case 'list_models':
      return [
        {
          name: 'Gemma 4 12B Q4_K_M',
          model_type: 'gemma-4-12b',
          quantization: 'Q4_K_M',
          file_size_gb: 7.4,
          context_length: 8192,
          available: true,
          path: 'D:\\UNOONE\\MODELS\\gemma4-12b-q4-gguf\\gemma-4-12b-q4_k_m.gguf',
        },
      ];
    case 'detect_acceleration':
      return ['CUDA', 'VULKAN', 'CPU'];
    case 'get_model_config':
      return {
        model_path: 'D:\\UNOONE\\MODELS\\gemma4-12b-q4-gguf\\gemma-4-12b-q4_k_m.gguf',
        context_size: 4096,
        batch_size: 512,
        threads: 0,
        gpu_layers: -1,
        temperature: 0.7,
        top_p: 0.9,
        top_k: 40,
        repeat_penalty: 1.1,
        max_tokens: 4096,
      };
    case 'get_model_status':
      return 'NOT_LOADED';
    case 'get_security_level':
      return 'STANDARD';
    case 'set_security_level':
      return `Security level set to ${args?.level}`;
    case 'review_tool_action':
      return {
        action_id: (args?.action as Record<string, unknown>)?.action_id || 'mock',
        approved: true,
        reason: 'Approved by mock safety guard',
        risk_level: 'SAFE',
        modified_parameters: null,
      };
    case 'start_recording':
      return {
        id: 'rec-' + Date.now(),
        title: 'Recording ' + new Date().toLocaleTimeString(),
        state: 'RECORDING',
        recording_type: args?.recording_type || 'VOICE_MEMO',
        privacy_level: args?.privacy_level || 'FULL',
        started_at: new Date().toISOString(),
        stopped_at: null,
        duration_seconds: 0,
        bookmarks: [],
        source_platform: 'DESKTOP',
        source_device_id: 'mock-desktop',
        audio_path: 'D:\\UNOONE\\VAULT\\recordings\\audio\\rec-' + Date.now() + '.enc',
        transcript_path: null,
        summary_path: null,
      };
    case 'pause_recording':
    case 'resume_recording':
      return { ...mockInvoke('start_recording', args), state: command === 'pause_recording' ? 'PAUSED' : 'RECORDING' };
    case 'stop_recording':
      return { ...mockInvoke('start_recording', args), state: 'PROCESSING' };
    case 'add_bookmark':
      return { ...mockInvoke('start_recording', args), bookmarks: [{ timestamp_seconds: 30, label: args?.label || null }] };
    default:
      return null;
  }
}

export const tauriApi = {
  // Vault
  detectVault: () => invoke<VaultInfo>('detect_vault'),
  unlockVault: (password: string, vaultRoot: string) =>
    invoke<VaultUnlockResult>('unlock_vault', { password, vault_root: vaultRoot }),
  setupVault: (password: string, profileName: string | null, vaultRoot: string) =>
    invoke<VaultSetupResult>('setup_vault', { password, profile_name: profileName, vault_root: vaultRoot }),
  lockVault: () => invoke<void>('lock_vault'),
  getHardwareProfile: () => invoke<HardwareProfile>('get_hardware_profile'),
  getVaultStatus: () => invoke<VaultStatus>('get_vault_status'),

  // Model management
  listModels: (vaultRoot: string) => invoke<ModelInfo[]>('list_models', { vault_root: vaultRoot }),
  detectAcceleration: () => invoke<AccelerationBackend[]>('detect_acceleration'),
  getModelConfig: () => invoke<ModelConfig>('get_model_config'),
  getModelStatus: () => invoke<ModelStatus>('get_model_status'),

  // Safety guard
  getSecurityLevel: () => invoke<SecurityLevel>('get_security_level'),
  setSecurityLevel: (level: SecurityLevel) => invoke<string>('set_security_level', { level }),
  reviewToolAction: (action: ToolAction, securityLevel: SecurityLevel) =>
    invoke<SafetyVerdict>('review_tool_action', { action, security_level: securityLevel }),

  // Recording
  startRecording: (recordingType: string, privacyLevel: string, vaultRoot: string) =>
    invoke<RecordingSession>('start_recording', { recording_type: recordingType, privacy_level: privacyLevel, vault_root: vaultRoot }),
  pauseRecording: () => invoke<RecordingSession>('pause_recording'),
  resumeRecording: () => invoke<RecordingSession>('resume_recording'),
  stopRecording: () => invoke<RecordingSession>('stop_recording'),
  addBookmark: (label: string | null) => invoke<RecordingSession>('add_bookmark', { label }),
};