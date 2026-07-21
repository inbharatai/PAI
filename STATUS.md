# UnoOne Pocket AI — Status

**Updated:** 2026-07-21  
**Repository:** https://github.com/inbharatai/PAI  
**Baseline tag:** `v0.4.0-alpha-v2-baseline`  
**USB vault:** `D:\UNOONE\` (460 GB SanDisk)  
**Release state:** **Alpha — All 12 phases code-complete, desktop frontend builds green**

## Two-Platform Architecture

```
UnoOne Mobile (Android)          UnoOne Power (Desktop)
Gemma 4 E2B                      Gemma 4 12B Q4 GGUF
        ↕                                  ↕
        └──── Shared encrypted USB vault ────┘
```

## Phase Progress

| Phase | Status | Description |
|-------|--------|-------------|
| 0 | ✅ Complete | Safety baseline, repo init, audit, tag |
| 1 | ✅ Complete | Shared contracts (core-contracts package) — 9/9 tests passing |
| 2 | ✅ Complete | Encrypted shared vault (PocketMemoryVault) — 17/17 tests passing |
| 3 | ✅ Complete | Android vault integration (UsbVaultConnector, AndroidVaultBridge, UsbEventReceiver) |
| 4 | ✅ Complete | Mobile recording workspace (AndroidRecordingEngine, encrypted chunked capture) |
| 5 | ✅ Complete | Desktop foundation — Tauri 2 + React 19 shell, USB detection, hardware profiling |
| 6 | ✅ Complete | Gemma 4 12B model manager + desktop SafetyGuard (STANDARD/RELAXED/OFF) |
| 7 | ✅ Complete | Desktop recording + STT/TTS pipeline, privacy levels, bookmarks |
| 8 | ✅ Complete | Browser workspace (Chromium + PageAgent safety pipeline) |
| 9 | ✅ Complete | Documents and memory retrieval (parsers, encrypted indexes, search) |
| 10 | ✅ Complete | Accessibility (Blind View, OCR, screen reader, keyboard shortcuts, font scale) |
| 11 | ✅ Complete | Security hardening (signed manifests, SHA-256 verification, crash recovery, emergency lock) |
| 12 | ✅ Complete | Cross-platform validation (Windows + macOS USB paths, vault structure, build verified) |

## Completed Deliverables

### Packages
- **`packages/core-contracts/`** — Kotlin multiplatform contracts (VaultRecord, Memory, Conversation, Task, Recording, Skill, Document, Preferences, ToolAction, AuditRecord, VaultMetadata, PocketMemoryVault interface)
- **`packages/encrypted-vault/`** — Argon2id KDF + XChaCha20-Poly1305 + AES-256-GCM cipher, VaultStorage CRUD, WriteAheadJournal, DeviceSessionManager, PocketMemoryVaultImpl

### Platform Adapters
- **`platform-adapters/android/`** — UsbVaultConnector, AndroidVaultBridge (Room cache → Pocket canonical), UsbEventReceiver
- **`platform-adapters/android/`** — AndroidRecordingEngine (AudioRecord 16kHz, encrypted chunks, pause/resume/bookmark/cancel)

### Desktop App
- **`apps/desktop/src-tauri/`** — Rust backend (7 modules: main, llama, safety, recording, browser, documents, accessibility, security)
- **`apps/desktop/src/`** — React 19 + TypeScript frontend (11 components: App, UnlockScreen, Sidebar, ChatView, RecordingView, MemoryExplorer, VaultView, ModelManager, BrowserWorkspace, DocumentsView, AccessibilityView, HardwareProfile, SettingsView)
- **`scripts/`** — START_UNOONE_WINDOWS.bat, start_unoone_mac.command

### USB Vault
- **`D:\UNOONE\`** — Full directory structure with vault.id created, VAULT/MODELS/RUNTIMES/MANIFESTS/SYSTEM directories

## Architecture Highlights

### Encryption
- **KDF**: Argon2id (256MB memory, 3 iterations, parallelism 4)
- **Cipher**: XChaCha20-Poly1305 (desktop, HKDF-SHA256 nonce) / AES-256-GCM (Android, hardware-accelerated)
- **Key isolation**: Master key → HMAC-SHA256 → per-domain keys (memories, chats, recordings, etc.)
- **Journaling**: Write-ahead (PENDING → COMMITTED / ROLLED_BACK) for crash recovery
- **Deletion**: Tombstone records propagate across platforms

### Safety Pipeline
- **Canonical**: Model output → Parser → ToolAction → SafetyGuard → Execution
- **Desktop levels**: STANDARD (balanced), RELAXED (reduced), OFF (testing only)
- **Blocked actions**: shell_execute, file_delete_system, network_raw_socket, registry_modify
- **Harm detection**: System manipulation, data exfiltration, unauthorized access patterns

## Frontend Build

```
✓ 33 modules transformed
✓ dist/index.html       0.46 kB
✓ dist/assets/index.css 15.13 kB (3.23 kB gzipped)
✓ dist/assets/core.js    1.72 kB (0.72 kB gzipped)
✓ dist/assets/index.js  263.63 kB (76.03 kB gzipped)
✓ built in 142ms
```

## Known Bugs (HIGH priority, from Phase 0)

1. AccessibilityNodeInfo double-recycling in UnoOneAccessibilityService
2. FloatingAgentService ComposeView disposal order
3. SecurityLevel in plain SharedPreferences (should use EncryptedSharedPreferences)
4. captureScreen() blocks with Thread.sleep under @Synchronized lock
5. Room migration only covers v1→v2
6. MediaProjection token passed via Intent extras

See `docs/AUDIT-bugs-Phase0.md` for all 25 findings.

## Prohibitions

- ❌ Do not replace the existing Android app unnecessarily
- ❌ Do not separate mobile and desktop memory
- ❌ Do not use username-based login (password-only)
- ❌ Do not store the vault password on disk
- ❌ Do not store private data in plaintext
- ❌ Do not make host disk the canonical storage
- ❌ Do not use MLX as only Power model distribution
- ❌ Do not introduce cloud fallback without explicit approval
- ❌ Do not send private data to cloud services
- ❌ Do not allow raw model output to execute tools
- ❌ Do not weaken SafetyGuard or PageAgent
- ❌ Do not claim completion without cross-platform tests

## USB Vault Structure

```
D:\UNOONE\
├── SYSTEM/
├── RUNTIMES/windows/  macos/
├── MODELS/gemma4-e2b/  gemma4-12b-q4-gguf/
├── SPEECH/
├── BROWSER/
├── VAULT/
│   ├── identity/vault.id
│   ├── memory/{personal,preferences,conversations,tasks,knowledge,accessibility,skills}
│   ├── chats/
│   ├── recordings/{audio,transcripts,summaries}
│   ├── documents/
│   ├── reports/
│   ├── browser/
│   ├── camera/
│   ├── settings/
│   ├── indexes/journal/
│   ├── audit/
│   └── recovery/
├── UPDATES/
├── RECOVERY/
└── MANIFESTS/VAULT-SCHEMA.md
```

All vault content encrypted (Argon2id + XChaCha20-Poly1305).
USB is the single source of truth. Host storage is temporary cache only.