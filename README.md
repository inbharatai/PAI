# UnoOne Pocket AI (PAI)

**Private AI — two platforms, one encrypted vault, zero cloud.**

UnoOne PAI is a private AI system that runs entirely on-device with no cloud dependency. It spans two platforms sharing one encrypted USB vault:

| | UnoOne Mobile | UnoOne Power |
|---|---|---|
| **Platform** | Android 9+ | Windows / macOS desktop |
| **Model** | Gemma 4 E2B (LiteRT) | Gemma 4 12B Q4 GGUF (llama.cpp) |
| **UI** | Jetpack Compose | Tauri 2 + React 19 |
| **Storage** | Room cache → USB vault | RAM → USB vault |
| **Voice** | Sherpa-ONNX STT/TTS | Whisper STT / Piper TTS |
| **Eyes-free** | TalkBack, Blind Aid, Camera OCR | Screen reader, high-contrast, OCR |

```
UnoOne Mobile (Android)          UnoOne Power (Desktop)
     Gemma 4 E2B                   Gemma 4 12B Q4 GGUF
          ↕                                ↕
          └──── Shared encrypted USB vault ────┘
                 (Argon2id + XChaCha20-Poly1305)
```

## Quick Start

### Android (Mobile)

```bash
# Clone and build
git clone https://github.com/inbharatai/PAI.git
cd PAI
./gradlew assembleDebug
# Install on device
adb install app/build/outputs/apk/debug/app-debug.apk
```

### Desktop (Power)

```bash
# Prerequisites: Rust, Node 18+, USB drive with UNOONE structure

# Build frontend
cd apps/desktop/src
npm install
npm run build

# Build Tauri app (requires Rust toolchain)
cd ../src-tauri
cargo build --release

# Or run in dev mode
cd ../src
npm run tauri dev
```

### USB Drive Setup

Insert your USB drive and run the launcher:

- **Windows**: Double-click `D:\UNOONE\START_UNOONE_WINDOWS.bat`
- **macOS**: Double-click `/Volumes/PAI/UNOONE/START_UNOONE_MAC.command`

The USB must contain:
```
UNOONE/
├── START_UNOONE_WINDOWS.bat
├── START_UNOONE_MAC.command
├── VAULT/identity/vault.id
├── MODELS/gemma4-12b-q4-gguf/
├── RUNTIMES/windows/llama-server.exe
└── RUNTIMES/macos/llama-server
```

## Architecture

### Encryption

- **KDF**: Argon2id (256MB memory, 3 iterations, parallelism 4)
- **Cipher**: XChaCha20-Poly1305 (desktop) / AES-256-GCM (Android hardware-accelerated)
- **Key isolation**: Master key → HMAC-SHA256 → per-domain keys (memories, chats, recordings, documents, settings, audit)
- **Journaling**: Write-ahead log (PENDING → COMMITTED / ROLLED_BACK) for crash recovery
- **Deletion**: Tombstone records propagate across platforms
- **Password-only login**: No username, no email, no cloud account

### Safety Pipeline

```
User input → Model → Parser → ToolAction → SafetyGuard → Execution
```

- **Raw model output never executes tools directly**
- Three security levels: STANDARD (balanced), RELAXED (reduced), OFF (testing only)
- Blocked actions: shell_execute, file_delete_system, network_raw_socket, registry_modify
- Harm detection: system manipulation, data exfiltration, unauthorized access patterns

### Project Structure

```
PAI/
├── app/                          # Android app (Jetpack Compose)
├── packages/
│   ├── core-contracts/           # Kotlin multiplatform contracts
│   └── encrypted-vault/          # Argon2id + XChaCha20-Poly1305 vault engine
├── platform-adapters/
│   └── android/                  # USB vault connector, recording engine
├── apps/
│   └── desktop/                  # Tauri 2 + React 19 desktop app
│       ├── src/                  # React frontend (11 components)
│       └── src-tauri/            # Rust backend (8 modules)
├── scripts/                      # USB launcher scripts
├── docs/                         # Audit docs, migration plan
├── STATUS.md                     # Project status tracker
└── MIGRATION-PLAN.md             # 12-phase development plan
```

### Core Contracts (`packages/core-contracts`)

Shared Kotlin contracts used by both platforms:

- `VaultRecord` — Base metadata (id, timestamps, platform, revision, deletion)
- `Memory` — 7 memory types (Personal, Preference, Conversation, Task, Knowledge, Accessibility, Skill)
- `Conversation` — Chat with messages and tool call records
- `Task` — Cross-platform task continuation with steps, failures, approvals
- `Recording` — Recording with bookmarks, summaries, privacy levels
- `Transcript` — Transcript with segments, confidence, bookmarks
- `Skill` — User-defined skill sequences
- `Document` — 10 document types (PDF, DOCX, TXT, Markdown, CSV, XLSX, PPTX, Image, Audio, WebPage)
- `Preferences` — Cross-platform preferences (language, security level, theme)
- `ToolAction` — Canonical safety pipeline (model → parser → SafetyGuard → execution)
- `AuditRecord` — Append-only audit with SHA-256 input hash
- `VaultMetadata` — Vault manifest (device sessions, KDF params, manifest hashes)
- `PocketMemoryVault` — Interface (setupVault, unlockVault, lockVault, emergencyLock, CRUD, search, integrity)

### Encrypted Vault (`packages/encrypted-vault`)

- `Argon2idKdf` — Key derivation (256MB, 3 iterations, parallelism 4)
- `XChaCha20Poly1305Cipher` — HKDF-SHA256 key derivation + AES-256-GCM (24-byte nonce via HKDF)
- `Aes256GcmCipher` — JDK built-in AES-256-GCM (hardware-accelerated on Android)
- `VaultCipher` — Algorithm selection (XChaCha20 on desktop, AES-256-GCM on Android)
- `VaultStorage` — Encrypted read/write/delete/list with domain keys, write-ahead journal
- `WriteAheadJournal` — PENDING/COMMITTED/ROLLED_BACK states, crash recovery
- `DeviceSessionManager` — Device session tracking, vault-level file locking, USB connect/disconnect
- `PocketMemoryVaultImpl` — Full implementation of PocketMemoryVault interface

### Desktop Rust Backend (`apps/desktop/src-tauri/src/`)

| Module | Purpose |
|--------|---------|
| `main.rs` | Tauri entry, USB vault detection, hardware profiling |
| `llama.rs` | Gemma 4 12B model manager, CUDA/Metal/Vulkan/CPU detection |
| `safety.rs` | SafetyGuard (STANDARD/RELAXED/OFF), blocked actions, harm detection |
| `recording.rs` | Desktop recording engine, privacy levels, bookmarks |
| `browser.rs` | Browser workspace, PageAgent safety pipeline |
| `documents.rs` | Document processing, 10 file types, encrypted search |
| `accessibility.rs` | Blind View, OCR, screen reader, camera adapters |
| `security.rs` | Signed manifests, SHA-256 verification, crash recovery, emergency lock |

### Desktop React Frontend (`apps/desktop/src/src/`)

| Component | Purpose |
|-----------|---------|
| `UnlockScreen` | Password-only vault unlock, USB detection, new vault setup |
| `Sidebar` | Navigation, vault status indicator, lock button |
| `ChatView` | Gemma 4 conversation interface |
| `RecordingView` | Recording with type/privacy selectors, pause/resume/bookmarks |
| `MemoryExplorer` | 7 memory types, search, cross-platform sync |
| `VaultView` | Vault status, domain breakdown, encryption info, emergency lock |
| `ModelManager` | Model list, acceleration backends, config, safety pipeline |
| `BrowserWorkspace` | URL bar, Chromium viewport, SafetyGuard-protected actions |
| `DocumentsView` | Document import, search, memory retrieval |
| `AccessibilityView` | Blind View, OCR, high contrast, font scale, keyboard shortcuts |
| `HardwareProfile` | CPU/RAM/GPU detection, acceleration recommendation |
| `SettingsView` | Language, security level, model path, temperature, auto-lock |

## Tests

```bash
# Core contracts (9 tests)
cd packages/core-contracts && ./gradlew test

# Encrypted vault (17 tests)
cd packages/encrypted-vault && ./gradlew test

# Android app (549 JVM tests + 42 instrumented)
cd app && ./gradlew test
```

## Prohibitions

- ❌ No username/email login — password-only
- ❌ No plaintext storage on disk
- ❌ No cloud fallback without explicit approval
- ❌ No raw model output executing tools directly
- ❌ No weakening SafetyGuard or PageAgent
- ❌ Host disk is not canonical — USB is the single source of truth
- ❌ MLX is not the only Power model distribution — GGUF is canonical

## License

Proprietary — InBharat Ai