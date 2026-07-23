# UnoOne Pocket AI (PAI)

**Private AI — two platforms, one encrypted vault, zero cloud.**

UnoOne PAI is a private AI system that runs entirely on-device with no cloud dependency. It spans two platforms sharing one encrypted USB vault:

| | UnoOne Mobile | UnoOne Power |
|---|---|---|
| **Platform** | Android 9+ | Windows / macOS desktop |
| **Model** | Gemma 4 E2B/E4B (LiteRT) | Gemma 4 12B Q4 GGUF (llama.cpp) |
| **UI** | Jetpack Compose | Tauri 2 + React 19 |
| **Storage** | Room cache → USB vault | RAM → USB vault |
| **Voice** | Sherpa-ONNX STT/TTS | Whisper STT / Piper TTS |
| **Eyes-free** | TalkBack, Blind Aid, Camera OCR | Screen reader, high-contrast, OCR |

```
UnoOne Mobile (Android)          UnoOne Power (Desktop)
     Gemma 4 E2B/E4B              Gemma 4 12B Q4 GGUF
          ↕                                ↕
          └──── Shared encrypted USB vault ────┘
                 (Argon2id + XChaCha20-Poly1305)
```

## What works today

### Android assistant

- Native Kotlin and Jetpack Compose app for Android 9 and later (API 28+), organized into 15 Gradle modules.
- One local Gemma 4 E2B planning engine through LiteRT-LM, with E4B Medium mode available on devices with ≥ 10 GB RAM. Deterministic rules handle common commands before model inference.
- A canonical 42-tool registry (29 legacy + 13 atomic accessibility, messaging, and calendar tools) rejects unknown tools, validates required arguments, and checks argument types.
- Dynamic tool exposure: the orchestrator selects 2–3 candidate tools for E2B (Lite) tasks and 3–6 for E4B (Medium), preventing hallucinated tool calls and reducing context-window waste. The model never sees all 42 tools at once.
- DeterministicIntentRouter handles wake commands, language switches, blind-mode toggles, simple app launches, accessibility shortcuts, and fast replies without model involvement.
- LanguageNormalizer detects 7 languages (en, hi, bn, ta, te, kn, ml) from speech patterns, normalizes filler words, and enforces the hard output-language rule (speak Hindi → reply in Hindi). Low-confidence transcripts (< 0.5) trigger clarification instead of execution.
- ToolProposalValidator validates model proposals against the candidate set, checks required arguments and types, and always allows `speak_response` as a fallback escape hatch.
- ActionResult with verified evidence: the orchestrator independently verifies action outcomes (foreground-package checks for app launches, deterministic-action confirmation for accessibility). The model may announce success only when `verified == true`.
- Direct commands, compound tasks, model-planned actions, and Skills use the same permission, risk, confirmation, execution, verification, and audit pipeline.
- Offline Sherpa-ONNX speech recognition and speech output, with explicit model-health checks. English uses the streaming transducer; Hindi uses the Omnilingual recognizer and its own offline voice.
- Selectable English and Hindi speech profiles. The selection controls STT routing, deterministic tool-status replies, wake acknowledgement, and TTS.
- One-tap hands-free sessions that listen, run the command, speak the result, and re-arm. The foreground session and background wake service coordinate ownership of the microphone.
- Background activation uses a low-latency offline keyword spotter plus an independent bounded offline-STT fallback for one-breath English and Hindi commands. The short **"Uno"** keyword and longer activation variants are supported, and a monotonic cooldown prevents the two detectors from firing twice for one speech burst. Wake acknowledgement finishes before command capture begins, and foreground recording and TTS exclusively own the microphone to prevent self-transcription.
- Phone actions for opening apps, Calendar, Chrome, WhatsApp, the dialer, URLs, and system screens.
- Calendar events, WhatsApp messages, and emails are prepared as reviewable drafts. UnoOne does not press the external app's final Send or Save control.
- Local notes, memory, Skills, activity logs, browser audit records, and preferences.
- A floating assistant and background voice service.
- A collapsible **Agent activity** panel that shows what UnoOne understood, which checks ran, what is executing, and whether it succeeded.
- A persistent **Disable UnoOne** master control on the Agent and Settings screens. Disabled mode stops and blocks microphone capture, STT, TTS, inference, Blind Aid, screen reading, accessibility actions, browser work, floating services, pending recovery, and network-backed page activity until the user explicitly enables the app.

### Desktop app

- Tauri 2 + React 19 desktop shell with USB vault detection, manifest validation, hardware profiling, and vault-core integration (Argon2id unlock/create/lock).
- Real Tauri API calls (no mock data), real SHA-256 verification, honest error states.
- Rust CI configured (fmt/check/test/clippy on Windows + macOS); WDAC blocks local builds.

## Current Status

| Component | Status |
|-----------|--------|
| Mobile app (Android) | V2 agent pipeline merged — 42 tools, E2B/E4B dual mode, deterministic routing |
| Desktop frontend (React) | BUILDS — Vite build passes, real Tauri API calls, no mock data |
| Desktop backend (Rust) | IMPLEMENTED — compiles on CI; WDAC blocks local builds |
| Vault encryption (`packages/vault-core`) | IMPLEMENTED — Argon2id + XChaCha20-Poly1305 + HKDF-SHA-256 + BIP-39 recovery + write-ahead journal; wired to recording, documents, and vault writes |
| Model inference | WDAC FALLBACK — detects llama-server / Ollama / LM Studio; verified via Ollama proxy; mmproj vision loaded when available |
| Recording | IMPLEMENTED — cpal audio capture + hound WAV encoding + vault-core XChaCha20-Poly1305 encryption; 4 privacy levels (Full, TranscriptOnly, SummaryOnly, PrivateSession) |
| Browser workspace | IMPLEMENTED — Tauri WebView bridge (WebView2/WKWebView); DOM query, click, type, extract text, fill form, scroll, screenshot; no Playwright/Chromium needed |
| Document parsing | IMPLEMENTED — PDF (lopdf), DOCX/XLSX/PPTX (zip+quick-xml), TXT/MD/CSV/HTML; TF-IDF search |
| Accessibility (OCR, Blind View) | IMPLEMENTED — OCR and image description via Gemma mmproj model; camera info via getUserMedia; encode_image_for_vision base64 pipeline |
| Security (vault writes) | IMPLEMENTED — vault_write_record Tauri command writes encrypted records; recording and document content encrypted end-to-end |
| macOS | **NOT BUILT, NOT TESTED** |

See `docs/EVIDENCE_AUDIT.md` for the full honest status of every feature.

## Architecture

```text
voice / text / floating assistant / accessibility input
                         │
                         ▼
               LanguageNormalizer
                         │
                         ▼
              DeterministicIntentRouter
              ┌────────────┼────────────┐
              │            │            │
        wake/language/  app launch/  accessibility
        blind mode     blind mode   shortcuts
        (no model)      (no model)   (no model)
                             │
                    NO_DETERMINISTIC_MATCH
                             │
                             ▼
                     ModelTierSelector
                    ┌────────┴────────┐
                    │                 │
              E2B (Lite)        E4B (Medium)
              2-3 tools          3-6 tools
              2 steps            4 steps
                    │                 │
                    └────────┬────────┘
                             │
                     CandidateToolSelector
                             │
                             ▼
                  fresh planning conversation
                             │
                             ▼
                       local Gemma
                             │
                             ▼
                    ToolProposalValidator
                             │
                             ▼
                permissions + SafetyGuard + security mode
                             │
                             ▼
       phone tools / notes / memory / Skills / Blind Aid / documents
                             │
                             ▼
                      ActionVerifier
                             │
                    ┌────────┴────────┐
                    verified success  unverified/partial
                             │                │
                    ObservationBuilder   ObservationBuilder
                             │                │
                    ┌────────┴────────────────┘
                    │
              AgentLoopController
              (ReAct: max steps per profile)
                    │
              speak_response
                    │
                    ▼
                  TTS out
```

### Local model contract

UnoOne has two planning-brain profiles.

| Field | Lite (E2B) | Medium (E4B) |
|---|---|---|
| Model id | `gemma-4-e2b` | `gemma-4-e4b` |
| File | `gemma-4-E2B-it.litertlm` | `gemma-4-E4B-it.litertlm` |
| Runtime | LiteRT-LM | LiteRT-LM |
| Exact size | `2,588,147,712` bytes | `~3,700,000,000` bytes |
| SHA-256 | `181938105e0eefd105961417e8da75903eacda102c4fce9ce90f50b97139a63c` | `0b2a8980ce155fd97673d8e820b4d29d9c7d99b8fa6806f425d969b145bd52e0` |
| Maximum context | 32,768 tokens | 32,768 tokens |
| Configured context | 2,048 tokens (Lite) | 4,096 tokens (Medium) |
| Minimum RAM gate | 6,144 MB | 8,192 MB |
| Recommended RAM | 8,192 MB | 10,240 MB |
| Candidate tools per task | 2–3 | 3–6 |
| Max agent steps | 2 | 4 |
| Max browser steps | 0 | 8 |
| Action temperature | 0.1 | 0.1 |
| Chat temperature | 0.3 | 0.7 |
| Tested backend on Xiaomi 14 | CPU fallback | Not yet tested on device |

The model is selected **before** a task starts and **never switches mid-task**. If E4B fails, the task stops, state is preserved, and the orchestrator offers Lite or retry. `ModelTierSelector` uses command complexity and device RAM to decide: simple deterministic commands always use Lite, compound commands (messaging, calendar, notes, web) use Medium when E4B is available and RAM ≥ 10,240 MB, otherwise fall back to Lite.

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
- **Vault writes**: `vault_write_record` Tauri command encrypts content via XChaCha20-Poly1305 and stores in `VAULT/records/`; recording and document content flows through this pipeline

> ⚠️ **CI verification pending**: vault-core compiles locally (frontend passes) but Rust CI has not yet verified `cargo check/test/clippy`. WDAC blocks local Rust builds. Do not store sensitive data until CI confirms all tests pass.

### Safety Pipeline

```
User input → Model → Parser → ToolAction → SafetyGuard → Execution
```

- **Raw model output never executes tools directly**
- Three security levels: STANDARD (balanced), RELAXED (reduced), OFF (testing only)
- Blocked actions: shell_execute, file_delete_system, network_raw_socket, registry_modify
- Harm detection: system manipulation, data exfiltration, unauthorized access patterns

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

## Project Structure

```
PAI/
├── android-app/UnoOneAgent/    # V2 agent pipeline + 42 tools (mobile-golden-baseline-v1 tag for rollback)
├── packages/
│   ├── core-contracts/           # Kotlin multiplatform contracts
│   ├── encrypted-vault/          # Kotlin Argon2id + XChaCha20-Poly1305 vault engine (Android)
│   └── vault-core/               # Rust vault library (desktop + shared test vectors)
├── platform-adapters/
│   └── android/                  # USB vault connector, recording engine
├── apps/
│   └── desktop/                  # Tauri 2 + React 19 desktop app
│       ├── src/                  # React frontend (11 components)
│       └── src-tauri/            # Rust backend (8 modules: main, llama, safety, recording, browser, documents, accessibility, security)
├── scripts/
│   ├── verify-mobile-untouched.sh  # CI protection: zero changes to Android
│   ├── verify-mobile-untouched.py  # Python equivalent
│   └── verify-mobile-untouched.ps1 # PowerShell equivalent
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
| `main.rs` | USB vault detection, manifest validation, hardware profiling, vault-core integration, vault_write_record command | IMPLEMENTED |
| `llama.rs` | Model manager: manifest-based discovery, CUDA/Metal/Vulkan/CPU detection, mmproj vision, WDAC fallback (llama-server/Ollama/LM Studio), multimodal Content enum | IMPLEMENTED |
| `safety.rs` | SafetyGuard (STANDARD/RELAXED/OFF), blocked actions, harm detection | IMPLEMENTED |
| `recording.rs` | Desktop recording: cpal microphone capture, hound WAV encoding, vault-core XChaCha20-Poly1305 encryption, 4 privacy levels | IMPLEMENTED |
| `browser.rs` | Browser workspace: Tauri WebView bridge (no Playwright), DOM query/click/type/extract/fill/scroll/screenshot | IMPLEMENTED |
| `documents.rs` | Document processing: PDF (lopdf), DOCX/XLSX/PPTX (zip+quick-xml), TXT/MD/CSV/HTML, TF-IDF search | IMPLEMENTED |
| `accessibility.rs` | Blind View, OCR and image description via Gemma mmproj, camera info, encode_image_for_vision | IMPLEMENTED |
| `security.rs` | Signed manifests, SHA-256, vault-core encryption wired, crash recovery, emergency lock | IMPLEMENTED |

### Desktop React Frontend (`apps/desktop/src/src/`)

| Component | Purpose | Status |
|-----------|---------|--------|
| `UnlockScreen` | Password-only vault unlock, USB detection, new vault setup | BUILDS_NOT_RUNTIME_TESTED |
| `ChatView` | Gemma 4 conversation via llama-server HTTP | IMPLEMENTED (needs running model) |
| `RecordingView` | Recording with type/privacy, pause/resume/bookmarks, vault encryption | IMPLEMENTED (backend wired, needs UI testing) |
| `MemoryExplorer` | 7 memory types, search, cross-platform sync | BUILDS_NOT_RUNTIME_TESTED |
| `VaultView` | Vault status, emergency lock | BUILDS_NOT_RUNTIME_TESTED |
| `BrowserWorkspace` | URL bar, WebView viewport, DOM bridge actions | IMPLEMENTED (backend wired, needs UI testing) |
| `DocumentsView` | Document import, search (PDF/DOCX/XLSX/PPTX/TXT/MD/CSV/HTML) | IMPLEMENTED (needs UI testing) |
| `AccessibilityView` | Blind View, OCR, high contrast, camera capture | IMPLEMENTED (backend wired, needs UI testing) |

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

### Desktop Pure-Rust Dependencies (Pendrive-Compatible)

All desktop dependencies are pure Rust — no C/C++ system libraries, no external runtimes, no WDAC-blocked build scripts.

| Crate | Purpose | WDAC-safe |
|-------|---------|-----------|
| `cpal 0.15` | Microphone audio capture (WASAPI/CoreAudio/ALSA) | ✅ |
| `hound 3.5` | WAV encoding for recordings | ✅ |
| `lopdf 0.33` | PDF text extraction (nom_parser, no rayon) | ✅ |
| `zip 2` | ZIP/DOCX/XLSX/PPTX parsing (deflate only, no bzip2-sys) | ✅ |
| `quick-xml 0.37` | XML parsing for DOCX/XLSX/PPTX content | ✅ |
| `base64 0.22` | Base64 encoding for vision/OCR image transport | ✅ |
| `reqwest 0.12` | HTTP client for llama-server inference | ✅ |
| `tauri 2` | WebView shell (uses system WebView2/WKWebView) | ✅ |
| `unoone-vault-core` | Argon2id + XChaCha20-Poly1305 encryption | ✅ |

**Total added dependency weight: ~3.8 MB.** No Tesseract, no Playwright, no separate Gemma download, no C/C++ build tools required.

## Tests

```bash
# Core contracts (9 tests)
cd packages/core-contracts && ./gradlew test

# Vault core — Rust (CI only, WDAC blocks local builds)
cd packages/vault-core && cargo test

# Encrypted vault — Kotlin (17 tests)
cd packages/encrypted-vault && ./gradlew test

# Android app (~550 JVM tests + 42 instrumented)
cd android-app/UnoOneAgent && ./gradlew test

# Desktop CI (GitHub Actions)
# .github/workflows/desktop-ci.yml
# - Mobile protection check (verify Android untouched)
# - Frontend build (Vite)
# - Rust check/test/clippy on Windows + macOS
# - Secret scan
# - Artifact scan (no model binaries in git)
```

## Latest verified results

Automated gates rerun on July 22, 2026 (after V2 agent architecture overhaul):

| Gate | Result |
|---|---|
| Android lint | Pass; no new issues against the existing baseline |
| Android JVM unit tests | Pass; ~550 tests, 0 failures, 0 errors |
| Debug, release, and Android-test assembly/compilation | Pass |
| Repository invariant check | Pass |
| Page Agent TypeScript and unit tests | Pass; 8 unit tests |
| Page Agent Playwright | Pass; 5 browser scenarios |

Physical evidence on the Xiaomi 14 (`7f8cafef`), Android 15:

| Gate | Result |
|---|---|
| Historical connected-device instrumentation, July 17 | `OK (55 tests)` in 206.218 seconds |
| Historical no-network subset, July 17 | `OK (20 tests)` in 77.202 seconds with Wi-Fi and mobile data disabled |
| Latest instrumentation attempt | Test APK compiled; Xiaomi rejected installation with `INSTALL_FAILED_USER_RESTRICTED` |
| PDF and DOCX Android round trips | `OK (2 tests)` in the July 17 suite; exact values persisted and original bytes unchanged |
| Phone actions | WhatsApp Business, Gmail, and Calendar opened with foreground-package verification; drafts remain reviewable |
| Read Screen | Read actual Accessibility Settings content through the accessibility/OCR path and spoke the result |
| Page Agent physical WebView | Read rendered page text and executed guarded text-entry actions; complex public-site completion is not yet qualified |
| Hindi speech and Blind Aid | Hindi STT/TTS engines initialized; person/mobile-phone and other COCO labels were detected and spoken through native Hindi narration, with no stale narration after stop |
| Master disable | Blocked commands and speech, survived process restart, and did not replay the old request after re-enable |
| Crash/ANR/OOM scan | No UnoOne crash, ANR, OOM, missing crypto class, or UnoOne low-memory kill in the final inspected run |

Current-revision checks on July 21 additionally verified a combined Hindi-language + Blind Aid command, real-time detection of `person`, `cell phone`, `bottle`, `book`, and other supported COCO classes, Hindi offline TTS generation, clean camera shutdown with no later detection callbacks, Accessibility-based Read Screen with spoken output, foreground opening of WhatsApp Business/Gmail/Calendar, review-only WhatsApp and Gmail drafts, a Calendar review form containing the exact title/date/5–6 PM time without pressing Save, Secure Browser handoff and local Page Agent model loading, and master-disable persistence across force-stop/restart. Installing the final APK reset Accessibility on HyperOS; it was explicitly re-enabled before the final cross-app verification.

Detailed evidence and honest boundaries are recorded in [Connected-device validation](docs/DEVICE_VALIDATION_2026-07-17.md) and [Device verification](DEVICE_VERIFICATION.md).

## Mobile Protection

The Android app is protected by CI golden-baseline verification. The V2 agent pipeline is merged to main. The original golden baseline tag `mobile-golden-baseline-v1` (commit `ae35ec7`) remains for rollback.

```bash
# Local check
bash scripts/verify-mobile-untouched.sh

# CI check
# .github/workflows/desktop-ci.yml — mobile-protection job
```

See `docs/MOBILE_GOLDEN_BASELINE.md` for the full protection policy.

## Not production-ready yet

The following gates remain open:

- a second Android device and broader OEM/API matrix;
- repeatable recorded-speech accuracy tests for every enabled language and multiple accents/noise levels;
- a controlled Blind Aid corpus covering lighting, distance, people, vehicles, phones, and product classes;
- fine-grained product recognition beyond the bundled COCO detector;
- a sustained thermal, memory, battery, and 50-task planning/Page Agent benchmark;
- human verification of audible speech quality and TalkBack announcements;
- final visual reruns of Android document/file pickers while the physical device is unlocked;
- approved-site-by-site Page Agent qualification and prompt-injection testing;
- release dependency/licence review, SBOM, protected signing key, and signed release APK;
- production object storage, catalogue signing key, signed catalogues, deployment, update, and rollback testing;
- E4B (Medium) model loading and inference on device;
- live voice, camera, and TalkBack UX validation;
- desktop runtime testing: recording with real microphone, browser workspace with real pages, OCR with real images;
- macOS build and testing;
- WDAC policy environment testing: verify llama-server, recording, and browser work under real WDAC constraints.

The installer PWA is implemented but intentionally keeps downloads locked when a production catalogue public key is not configured. No production deployment or production-approved release is claimed.

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
- ❌ No external runtimes — no Playwright, no Tesseract, no separate Gemma download
- ❌ No C/C++ build dependencies that WDAC blocks (rayon-core, bzip2-sys)

## Documentation

- [Android build and validation](android-app/UnoOneAgent/README.md)
- [Phone-control implementation](android-app/UnoOneAgent/phonecontrol/README.md)
- [Offline Document Skills](docs/OFFLINE_DOCUMENT_SKILLS.md)
- [Connected-device validation](docs/DEVICE_VALIDATION_2026-07-17.md)
- [Device verification matrix](DEVICE_VERIFICATION.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Safety](docs/SAFETY.md)
- [Model acquisition and distribution](docs/MODEL_ACQUISITION_AND_DISTRIBUTION.md)
- [Speech model qualification](docs/SPEECH_MODEL_QUALIFICATION.md)
- [Privacy policy](docs/play-review/privacy-policy.md)
- [Data safety](docs/play-review/data-safety.md)

## License

Proprietary — Uni Guru Technologies LLP / InBharat.ai. Repository code, libraries, model weights, and speech artifacts may use different licences or usage terms. Review and preserve the notice attached to every component before redistribution.