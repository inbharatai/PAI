/**
 * UnoOne Power — Tauri API bindings
 * Type-safe bridge between React frontend and Rust backend
 *
 * In production (Tauri runtime available), all calls go to the Rust backend.
 * In development without Tauri, calls throw errors — no mock data.
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

export interface DocumentMetadata {
  id: string;
  title: string;
  document_type: string;
  file_path: string;
  file_size_bytes: number;
  created_at: string;
  modified_at: string;
  source_platform: string;
  tags: string[];
  page_count: number | null;
  word_count: number | null;
}

export interface AccessibilityStatus {
  screen_reader_detected: boolean;
  high_contrast: boolean;
  reduced_motion: boolean;
  font_scale: number;
  screen_reader_name: string;
}

export interface AgentStep {
  type: 'Thinking' | 'ToolCall' | 'ToolResult' | 'SafetyBlock' | 'FinalResponse';
  tool?: string;
  args?: Record<string, unknown>;
  result?: string;
  reason?: string;
  text?: string;
  confidence?: number;
  approved?: boolean;
}

export interface AgentResult {
  final_text: string;
  steps: AgentStep[];
  iterations: number;
}

export interface SecurityVerificationResult {
  vault_id: string;
  manifest_valid: boolean;
  hmac_valid: boolean;
  entries_verified: number;
  entries_failed: number;
  total_entries: number;
  errors: string[];
}

export interface VoiceStatus {
  stt: string;
  tts: string;
  language: string;
}

// Tauri invoke — works in Tauri runtime only
async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
  return tauriInvoke<T>(command, args);
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

  // Documents
  listDocuments: (vaultRoot: string) => invoke<DocumentMetadata[]>('list_documents', { vault_root: vaultRoot }),
  searchMemories: (query: { query: string; memory_types: string[]; limit: number; min_relevance: number }, vaultRoot: string) =>
    invoke<Array<{ id: string; memory_type: string; title: string; preview: string; relevance: number; created_at: string }>>('search_memories', { query, vault_root: vaultRoot }),

  // Accessibility
  getAccessibilityStatus: () => invoke<AccessibilityStatus>('get_accessibility_status'),

  // Security
  emergencyLock: (vaultRoot: string) => invoke<{ success: boolean; keys_cleared: boolean; vault_locked: boolean; timestamp: string }>('emergency_lock', { vault_root: vaultRoot }),
  generateManifest: (vaultRoot: string) => invoke<VaultInfo & { entries: number; manifest_sha256: string }>('generate_manifest', { vault_root: vaultRoot }),
  verifyManifest: (vaultRoot: string) => invoke<SecurityVerificationResult>('verify_manifest', { vault_root: vaultRoot }),
  recoverFromCrash: (vaultRoot: string) => invoke<{ state: string; recovered_files: number; rolled_back_files: number; errors: string[] }>('recover_from_crash', { vault_root: vaultRoot }),

  // Vault state (D7 additions)
  vaultIsUnlocked: () => invoke<boolean>('vault_is_unlocked'),
  vaultReadRecord: (recordId: string) => invoke<string>('vault_read_record', { record_id: recordId }),

  // Agent loop (D2)
  agentChat: (message: string, conversationHistory: ConversationTurn[]) =>
    invoke<AgentResult>('agent_chat', { message, conversation_history: conversationHistory }),
  checkModelHealth: () => invoke<Record<string, unknown>>('check_model_health'),
  sendChatCompletion: (request: InferenceRequest) => invoke<InferenceResponse>('send_chat_completion', { request }),

  // Voice module (D4)
  getVoiceStatus: (vaultRoot: string) => invoke<VoiceStatus>('get_voice_status', { vault_root: vaultRoot }),
  transcribeAudio: (audioPath: string, vaultRoot: string) =>
    invoke<{ text: string; language: string; confidence: number; status: string }>('transcribe_audio', { audio_path: audioPath, vault_root: vaultRoot }),
  synthesizeSpeech: (text: string, vaultRoot: string) =>
    invoke<{ audio_path: string | null; duration_seconds: number | null; sample_rate: number; status: string; error: string | null }>('synthesize_speech', { text, vault_root: vaultRoot }),
};