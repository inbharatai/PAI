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

## Current Status

| Component | Status |
|-----------|--------|
| Mobile app (Android) | **FROZEN** at tag `mobile-golden-baseline-v1` — no changes authorized |
| Desktop frontend (React) | BUILDS — Vite build passes, real Tauri API calls, no mock data |
| Desktop backend (Rust) | CI_CONFIGURED — WDAC blocks local build; Rust CI configured (fmt/check/test/clippy on Windows+macOS) |
| Vault encryption (`packages/vault-core`) | IMPLEMENTED, CI_PENDING — Argon2id + XChaCha20-Poly1305 + HKDF-SHA-256 + BIP-39 recovery + write-ahead journal |
| Model inference | **NOT PROVEN from USB** — WDAC blocks llama-server.exe on this machine; verified via Ollama proxy |
| Recording | **NOT IMPLEMENTED** — state machine only, no audio capture |
| Browser | **NOT IMPLEMENTED** — returns "not available" |
| macOS | **NOT BUILT, NOT TESTED** |
| E2B mobile model on USB | **MISSING** — must be re-downloaded (was on old FAT32 partition) |
| macOS Metal runtime | **NOT POPULATED** — download from llama.cpp releases |

See `docs/EVIDENCE_AUDIT.md` for the full honest status of every feature.

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
# Prerequisites: Rust, Node 24+, USB drive formatted exFAT with UNOONE structure

# Build frontend
cd apps/desktop/src
npm install
npm run build

# Build Tauri app (requires Rust toolchain — may need CI if WDAC blocks builds)
cd ../src-tauri
cargo build --release

# Or run in dev mode
cd ../src
npm run tauri dev
```

### USB Drive Setup

The USB drive must be formatted exFAT (FAT32 cannot hold the 7.14 GiB 12B model). Insert it and the desktop app detects it automatically via manifest validation.

Expected structure:
```
UNOONE/
├── manifest.json              # Vault metadata (versioned, relative paths)
├── VERSION                    # e.g. "0.5.0-alpha"
├── APPS/
│   ├── WINDOWS/
│   └── MACOS/
├── RUNTIMES/
│   ├── WINDOWS/
│   │   ├── CUDA/              # llama.cpp CUDA 12.4 (NVIDIA GPU)
│   │   ├── CPU/               # llama.cpp CPU (AVX2+ fallback)
│   │   └── VULKAN/            # llama.cpp Vulkan (AMD/Intel GPU)
│   └── MACOS/
│       └── METAL/             # llama.cpp Metal (Apple Silicon)
├── MODELS/
│   ├── MOBILE/                # E2B models (must be re-downloaded)
│   └── DESKTOP/
│       └── Gemma-12B/
│           ├── gemma-4-12B-it-Q4_K_M.gguf  (7.14 GiB)
│           └── mmproj-gemma-4-12B-it-f16.gguf (116 MiB)
├── VAULT/
│   ├── identity/vault.id
│   ├── header/
│   ├── records/
│   ├── indexes/
│   ├── journal/
│   ├── transactions/
│   ├── attachments/
│   └── recovery/
├── CONFIG/
├── RECOVERY/
├── UPDATES/
└── LOGS/
```

The desktop app discovers the USB drive by:
1. Scanning all removable drives (WMI on Windows, `/Volumes/` on macOS)
2. Validating `manifest.json` + `VERSION` + `vault.id` — not hardcoded drive letters

## Architecture

### Encryption

- **KDF**: Argon2id (256 MiB memory, 3 iterations, parallelism 4) — ✅ implemented in `packages/vault-core`
- **Cipher**: XChaCha20-Poly1305 (desktop) / AES-256-GCM (Android hardware-accelerated)
- **Key wrapping**: Password → Argon2id → KEK → wraps random vault master key (allows password changes without re-encryption)
- **Key isolation**: Master key → HKDF-SHA-256 → per-domain keys (records, journal, indexes, etc.)
- **Header**: Double-buffered (A/B slots), HMAC-SHA-256 authenticated, constant-time comparison
- **Recovery**: 24-word BIP-39 mnemonic (NOT UUID fragments) with independent key wrapping
- **Journaling**: Write-ahead log (PENDING → COMMITTED / ROLLED_BACK) for exFAT crash safety
- **Deletion**: Tombstone records propagate across platforms
- **Password-only login**: No username, no email, no cloud account
- **Memory safety**: Master key zeroed on lock and drop; no passwords in files/logs

> ⚠️ **CI verification pending**: vault-core compiles locally (frontend passes) but Rust CI has not yet verified `cargo check/test/clippy`. WDAC blocks local Rust builds. Do not store sensitive data until CI confirms all tests pass.

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
├── android-app/UnoOneAgent/    # FROZEN — mobile golden baseline (tag: mobile-golden-baseline-v1)
├── packages/
│   ├── core-contracts/           # Kotlin multiplatform contracts
│   ├── encrypted-vault/          # Kotlin Argon2id + XChaCha20-Poly1305 vault engine (Android)
│   └── vault-core/               # Rust vault library (desktop + shared test vectors)
├── platform-adapters/
│   └── android/                  # USB vault connector, recording engine
├── apps/
│   └── desktop/                  # Tauri 2 + React 19 desktop app
│       ├── src/                  # React frontend (11 components)
│       └── src-tauri/            # Rust backend (8 modules)
├── scripts/
│   ├── verify-mobile-untouched.sh  # CI protection: zero changes to Android
│   └── verify-mobile-untouched.py   # Python equivalent
├── docs/
│   ├── EVIDENCE_AUDIT.md         # Honest status of every feature
│   └── MOBILE_GOLDEN_BASELINE.md  # Frozen mobile baseline documentation
├── .github/workflows/
│   ├── android-ci.yml
│   ├── desktop-ci.yml            # Rust CI: fmt, check, test, clippy (Win+macOS)
│   └── distribution-ci.yml
└── STATUS.md
```

### Desktop Rust Backend (`apps/desktop/src-tauri/src/`)

| Module | Purpose | Status |
|--------|---------|--------|
| `main.rs` | USB vault detection, manifest validation, hardware profiling, vault-core integration (Argon2id unlock/create/lock) | PARTIALLY_IMPLEMENTED |
| `llama.rs` | Model manager: manifest-based discovery, CUDA/Metal/Vulkan/CPU detection, health endpoint | PARTIALLY_IMPLEMENTED |
| `safety.rs` | SafetyGuard (STANDARD/RELAXED/OFF), blocked actions, harm detection | PARTIALLY_IMPLEMENTED |
| `recording.rs` | Desktop recording engine, privacy levels, bookmarks | NOT_IMPLEMENTED (state machine only, start_recording returns SECURITY_NOT_IMPLEMENTED) |
| `browser.rs` | Browser workspace, PageAgent safety pipeline | NOT_IMPLEMENTED |
| `documents.rs` | Document processing, 10 file types, search | PARTIALLY_IMPLEMENTED (TXT/Markdown only) |
| `accessibility.rs` | Blind View, OCR, screen reader, camera adapters | NOT_IMPLEMENTED |
| `security.rs` | Signed manifests, SHA-256, crash recovery, emergency lock | PARTIALLY_IMPLEMENTED (SHA-256 real, encryption not) |

### Desktop React Frontend (`apps/desktop/src/src/`)

| Component | Purpose | Status |
|-----------|---------|--------|
| `UnlockScreen` | Password-only vault unlock, USB detection, new vault setup | BUILDS_NOT_RUNTIME_TESTED |
| `ChatView` | Gemma 4 conversation via llama-server HTTP | PARTIALLY_IMPLEMENTED (no model loaded) |
| `RecordingView` | Recording with type/privacy, pause/resume/bookmarks | NOT_IMPLEMENTED (no audio capture) |
| `MemoryExplorer` | 7 memory types, search, cross-platform sync | BUILDS_NOT_RUNTIME_TESTED |
| `VaultView` | Vault status, emergency lock | BUILDS_NOT_RUNTIME_TESTED |
| `BrowserWorkspace` | URL bar, Chromium viewport | NOT_IMPLEMENTED ("Coming Soon") |
| `DocumentsView` | Document import, search | BUILDS_NOT_RUNTIME_TESTED |
| `AccessibilityView` | Blind View, OCR, high contrast | NOT_IMPLEMENTED (vision disabled) |

## Model Verification

| Property | Value |
|----------|-------|
| Model file | `gemma-4-12B-it-Q4_K_M.gguf` |
| Size | 7,662,531,872 bytes (7.14 GiB) |
| SHA-256 | `D333B368BE6CD655563FCE18AEDE26027E208FDB13816D35EB06983CE054044B` |
| GGUF version | 3 |
| Architecture | `gemma4` |
| Quantisation | Q4_K_M |
| Source | Google Gemma 4 12B IT, GGUF Q4_K_M by llama.cpp community |
| Licence | [Gemma Terms of Use](https://ai.google.dev/gemma/terms) |
| Inference verified | Yes — via Ollama proxy (direct llama-server blocked by WDAC) |
| Source = Destination SHA-256 | ✅ Exact match |

## Tests

```bash
# Core contracts (9 tests)
cd packages/core-contracts && ./gradlew test

# Vault core — Rust (CI only, WDAC blocks local builds)
cd packages/vault-core && cargo test

# Encrypted vault — Kotlin (17 tests)
cd packages/encrypted-vault && ./gradlew test

# Android app (549 JVM tests + 42 instrumented)
cd android-app/UnoOneAgent && ./gradlew test

# Desktop CI (GitHub Actions)
# .github/workflows/desktop-ci.yml
# - Mobile protection check (verify Android untouched)
# - Frontend build (Vite)
# - Rust check/test/clippy on Windows + macOS
# - Secret scan
# - Artifact scan (no model binaries in git)
```

## Mobile Protection

The Android app is frozen at tag `mobile-golden-baseline-v1` (commit `ae35ec7`, 315 protected files). The CI workflow and local scripts verify zero changes:

```bash
# Local check
bash scripts/verify-mobile-untouched.sh

# CI check
# .github/workflows/desktop-ci.yml — mobile-protection job
```

See `docs/MOBILE_GOLDEN_BASELINE.md` for the full protection policy.

## Prohibitions

- ❌ No username/email login — password-only
- ❌ No plaintext storage on disk
- ❌ No cloud fallback without explicit approval
- ❌ No raw model output executing tools directly
- ❌ No weakening SafetyGuard or PageAgent
- ❌ Host disk is not canonical — USB is the single source of truth
- ❌ No mock data, no placeholder success states, no fake functionality
- ❌ No modifying the Android app during desktop development
- ❌ No hardcoded drive letters — discover USB via removable-drive scan + manifest validation
- ❌ No claiming features work without test evidence (command, exit code, OS, hardware, date, commit)

## License

Proprietary — InBharat Ai