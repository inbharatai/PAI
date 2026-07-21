# PAI Audit Report — Placeholder & Mock Code

**Date:** 2026-07-21
**Auditor:** Claude Code
**Baseline:** v0.4.0-alpha-v2-baseline → 6c9b869
**Updated:** After systematic mock removal pass

## Summary

| Category | Original Count | Fixed | Remaining |
|----------|---------------|-------|-----------|
| Mock/fake data in React components | 4 | 4 | 0 |
| Placeholder return values in Rust | 8 | 5 | 3 |
| TODO-only implementations in Rust | 5 | 2 | 3 |
| Mock fallback in tauri.ts | 1 | 1 | 0 |
| Placeholder browser viewport | 1 | 1 | 0 |
| Fake chat echo response | 1 | 1 | 0 |
| Hard-coded mock hardware data | 1 | 1 | 0 |
| Recording state not shared across commands | 1 | 1 | 0 |

## Fixes Applied

### React Components — ALL MOCK DATA REMOVED

**F1. `MemoryExplorer.tsx`** ✅ FIXED — Uses real `tauriApi.searchMemories()` with empty state fallback

**F2. `VaultView.tsx`** ✅ FIXED — No more MOCK_DOMAINS; shows real vault status from Tauri API, empty state when disconnected

**F3. `ChatView.tsx`** ✅ FIXED — Replaced fake echo with real llama-server HTTP API connection (`/v1/chat/completions`). Shows model status indicator, error messages when server unavailable

**F4. `HardwareProfile.tsx`** ✅ Already uses Tauri API (no changes needed)

**F5. `tauri.ts`** ✅ FIXED — Removed entire `mockInvoke` function. Now throws errors when Tauri is unavailable (no silent fake data). All types preserved for real API calls.

**F6. `RecordingView.tsx`** ✅ FIXED — Wired to real Tauri API calls for start/pause/resume/stop/bookmark. Detects vault root automatically.

**F7. `DocumentsView.tsx`** ✅ FIXED — Fixed wrong API call (was calling `listModels`, now calls `listDocuments`). Shows real files from vault.

**F8. `BrowserWorkspace.tsx`** ✅ FIXED — No longer pretends to have a browser. Shows honest "Coming Soon" state with status explanation.

**F9. `AccessibilityView.tsx`** ✅ FIXED — Vision features (OCR, Blind Aid, camera) shown as disabled/pending. High contrast, reduced motion, font scale now apply real CSS changes. Reads real screen reader detection from Tauri API.

### Rust Backend — Real Implementations

**F10. `security.rs`** ✅ FIXED — Real SHA-256 hashing via `sha2` crate (replaces `format!("sha256:{}", data.len())`). Real file hashing in `compute_file_sha256()`. Real crash recovery (processes both .pending and .committed journal entries). Emergency lock writes a `.lock` marker file.

**F11. `main.rs`** ✅ FIXED — Real vault status (uses shared `VaultState` via `tauri::State`). Real disk usage calculation. Real GPU detection via `nvidia-smi`. Real USB speed detection. `unlock_vault` now validates vault ID exists and stores unlocked state. `setup_vault` creates full directory structure. `lock_vault` clears state properly. `get_vault_status` reads profile name and calculates real disk space.

**F12. `recording.rs`** ✅ FIXED — Uses `tauri::State<RecordingStateHolder>` for shared state across all commands. No more `DesktopRecordingEngine::new()` per call.

**F13. `documents.rs`** ✅ FIXED — `list_documents` reads real files from vault directory. `process_document` fully supports TXT/Markdown. Honest error messages for unsupported formats. `search_memories` reads real JSON/TXT/MD files from vault memory directory.

**F14. `browser.rs`** ✅ FIXED — All commands return honest "not available" errors. No fake data returned. Clean API structure preserved for future Playwright integration.

**F15. `accessibility.rs`** ✅ FIXED — `get_accessibility_status` detects real OS accessibility settings (NVDA/JAWS on Windows, VoiceOver on macOS, high contrast mode). `perform_ocr`, `describe_image`, `get_camera_info` return honest "not yet available" errors.

**F16. `llama.rs`** ✅ FIXED — Binary path now checks `llama-cuda/` subdirectory first, then `llama-cpu/` fallback. `get_model_status` checks if llama-server is actually running via TCP connect. Models found from real GGUF files on USB.

## Remaining Items (Honest Placeholders, Not Fakes)

These are features that are **structurally defined but not yet wired to real backends**. They return honest error messages, not fake data:

| Item | Status | Notes |
|------|--------|-------|
| Argon2id vault unlock/setup | Partially wired | `unlock_vault` validates vault exists but doesn't derive keys yet. TODO: integrate Rust argon2 crate |
| PDF/DOCX parsing | Error returned | `process_document` supports TXT/Markdown only; returns clear error for PDF/DOCX |
| Browser workspace | Error returned | Returns "Chromium/Playwright integration is pending" |
| OCR | Error returned | Returns "Tesseract integration is pending" |
| Image description | Error returned | Returns "Gemma 4 vision integration is pending" |
| Camera access | Error returned | Returns "Camera not available on desktop" |
| TTS/STT | UI pending | Language selectors shown but disabled |
| Vector search | Simple text search | `search_memories` does text matching, not embeddings |

## Android Baseline — NOT TOUCHED

The following modules were NOT modified and remain working:
- `android-app/UnoOneAgent/` — All 15 Gradle modules intact
- All existing Kotlin code untouched
- All 549 JVM tests should still pass
- All 42 instrumented tests should still pass

## USB Drive — Real Binaries Present

| Item | Status | Size |
|------|--------|------|
| llama-server.exe (CUDA) | ✅ Present | 9 KB (thin wrapper + DLLs) |
| llama-server.exe (CPU) | ✅ Present | 9 KB (thin wrapper + DLLs) |
| Gemma 4 12B Q4_K_M GGUF | ✅ Present | 3.1 GB |
| mmproj (vision projector) | ✅ Present | 122 MB |
| CUDA runtime DLLs | ✅ Present | ~930 MB |