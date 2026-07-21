# PAI Evidence-Based Audit — 2026-07-21 19:30 UTC

**Commit:** 83def01 (pushed to main)
**Auditor:** Claude Code
**OS:** Windows 11 Home, Application Control policy blocks Rust build scripts
**Hardware:** Unknown GPU (not yet detected by nvidia-smi in this session)

---

## Status Legend

| Status | Meaning |
|--------|---------|
| VERIFIED_WORKING | Build passes, runtime tested, output evidence attached |
| BUILDS_NOT_RUNTIME_TESTED | Compiles/builds but never executed |
| PARTIALLY_IMPLEMENTED | Structure exists, some logic works, core missing |
| NOT_IMPLEMENTED | Structure defined, no working logic |
| BLOCKED_BY_ENVIRONMENT | Cannot verify due to OS/hardware/tooling |

---

## 1. Frontend (React + Vite)

| Component | Status | Evidence |
|-----------|--------|----------|
| Vite build | VERIFIED_WORKING | `vite build` exits 0, 4 assets produced (156ms) |
| ChatView | PARTIALLY_IMPLEMENTED | HTTP client to `/v1/chat/completions` exists but **no model has been loaded, no prompt has been sent, no token has been generated** |
| VaultView | BUILDS_NOT_RUNTIME_TESTED | Shows real Tauri API calls but never executed in Tauri runtime |
| MemoryExplorer | BUILDS_NOT_RUNTIME_TESTED | Same — Tauri API calls but never executed |
| DocumentsView | BUILDS_NOT_RUNTIME_TESTED | Same |
| BrowserWorkspace | NOT_IMPLEMENTED | Shows "Coming Soon" — no browser functionality |
| AccessibilityView | NOT_IMPLEMENTED | Vision features disabled, display settings local-only |
| RecordingView | **PARTIALLY_IMPLEMENTED WITH FALSE-SUCCESS PATHS** | See §2 below |
| UnlockScreen | BUILDS_NOT_RUNTIME_TESTED | Tauri API calls but never executed |
| tauri.ts | BUILDS_NOT_RUNTIME_TESTED | Mock removed, real `invoke()` calls, but never executed in Tauri runtime |

## 2. False-Success Paths (Critical)

These are places where the UI shows success even when the backend fails:

### RecordingView.tsx

**Line 89-91:** `stopRecording` catch block swallows error — UI shows "Recording stopped" even if backend failed:
```ts
try {
  await tauriApi.stopRecording();
} catch {
  // If Tauri not available, still update UI
}
```

**Line 115-118:** `startRecording` catch block enters recording state even on failure:
```ts
} catch (e) {
  setError(e instanceof Error ? e.message : String(e));
  // Still allow UI recording even if Tauri is not available (for development)
  setIsRecording(true);
```
This means the UI shows "Recording…" with a timer ticking even when no microphone is capturing audio.

**Lines 130, 135:** Pause/resume and bookmark errors silently swallowed:
```ts
} catch {} // pause
} catch {} // resume
} catch {} // bookmark
```

### VaultView.tsx

**Line 86:** Emergency lock failure silently swallowed:
```ts
try { await tauriApi.lockVault(); } catch {}
```

### MemoryExplorer.tsx, DocumentsView.tsx

**Lines 37-39, 40:** Vault detection failures silently set empty state without informing user:
```ts
} catch {
  setMemories([]);
}
```

## 3. Hardcoded Paths

| File | Line | Hardcoded Path | Platform | Fix Required |
|------|------|---------------|----------|--------------|
| `RecordingView.tsx` | 39 | `D:\\UNOONE` | Windows only | ❌ Must use `detectVault()` result |
| `VaultView.tsx` | 72 | `D:\\UNOONE` | Windows only | ❌ Same |
| `main.rs` | 135 | `"D:\\UNOONE", "E:\\UNOONE", "F:\\UNOONE"` | Windows only | ❌ Must scan removable drives |
| `main.rs` | 139 | `"/Volumes/PAI/UNOONE", "/Volumes/UNOONE"` | macOS only | ❌ Hardcoded volume names |

No cross-platform removable-drive detection exists. macOS and Linux will fail unless volumes happen to match these exact names.

## 4. Rust Backend — DID NOT COMPILE

| Module | Status | Evidence |
|--------|--------|----------|
| `main.rs` | BLOCKED_BY_ENVIRONMENT | `cargo check` fails with Windows App Control policy error (os error 4551). Build scripts cannot execute. |
| `security.rs` | BLOCKED_BY_ENVIRONMENT | Same — depends on `sha2` crate which needs build scripts |
| `recording.rs` | BLOCKED_BY_ENVIRONMENT | Same |
| `documents.rs` | BLOCKED_BY_ENVIRONMENT | Same |
| `browser.rs` | BLOCKED_BY_ENVIRONMENT | Same |
| `accessibility.rs` | BLOCKED_BY_ENVIRONMENT | Same |
| `llama.rs` | BLOCKED_BY_ENVIRONMENT | Same |
| `safety.rs` | BLOCKED_BY_ENVIRONMENT | Same |

**No Rust module has been compiled, tested, or executed.** The code was pushed without verification.

The Windows Application Control policy (WDAC/AppLocker) blocks execution of build scripts from the `target/debug/build/` directory. This is a legitimate policy restriction, not a code error, but it means **no Rust code can be verified on this machine.**

## 5. Model Loading & Inference — NOT PROVEN

| Claim | Reality | Evidence |
|-------|---------|----------|
| "Gemma 4 12B Q4_K_M loaded" | **File is MISSING** | `find` shows only `mmproj-gemma-4-12B-it-f16.gguf` (122 MB) in `gemma4-12b-q4-gguf/`. The 3.1 GB model file does not exist. Previous report of 3,140,534,272 bytes was wrong. |
| "llama-server.exe present" | Stub only (9 KB) | `llama-server.exe` is 9,216 bytes — a thin wrapper that loads `llama-server-impl.dll` (9.5 MB). Never launched. |
| "Model inference works" | **NEVER TESTED** | No prompt has been sent. No token has been generated. No latency measurement exists. |
| `get_model_status()` | TCP port check only | Checks if port 8342 is open. Any process on that port would report "LOADED". Does not verify model identity, hash, or health. |
| Gemma 4 E2B model | File exists (3.2 GB) | `gemma-4-e2b-q4_k_m.gguf` present. Never loaded or tested. |

### Model File Status

| File | Size | SHA-256 | Status |
|------|------|---------|--------|
| `gemma4-12b-q4-gguf/gemma-4-12B-it-Q4_K_M.gguf` | **MISSING** | N/A | ❌ NOT PRESENT |
| `gemma4-12b-q4-gguf/mmproj-gemma-4-12B-it-f16.gguf` | 122,031,552 bytes | Not computed | Present but never loaded |
| `gemma4-e2b/gemma-4-e2b-q4_k_m.gguf` | 3,427,861,568 bytes | `ec6d37feb0fd1df54e72168c150b821e671f2adfbb62f254983677f4776925b6` | Present but never loaded |

## 6. Vault Encryption — NOT IMPLEMENTED

| Feature | Status | Evidence |
|---------|--------|----------|
| Argon2id key derivation | NOT_IMPLEMENTED | `unlock_vault` validates vault ID exists but does NOT derive keys. Comment says "TODO: Implement real Argon2id key derivation" |
| XChaCha20-Poly1305 encryption | NOT_IMPLEMENTED | No encryption/decryption of vault data |
| Password verification | NOT_IMPLEMENTED | `unlock_vault` accepts any password if vault ID file exists |
| Vault lock (clear keys from memory) | NOT_IMPLEMENTED | `lock_vault` clears state strings but no cryptographic keys exist to clear |
| Emergency lock | PARTIALLY_IMPLEMENTED | Writes `.lock` marker file but does not actually clear any in-memory keys |
| Write-ahead journaling | NOT_IMPLEMENTED | `security.rs` scans for `.pending` and `.committed` files but nothing creates them |

## 7. Feature Implementation Status

| Feature | Status | What's Missing |
|---------|--------|---------------|
| Vault unlock/setup | PARTIALLY_IMPLEMENTED | Directory creation works, no key derivation or encryption |
| Chat with Gemma 4 | NOT_IMPLEMENTED | HTTP client exists but no model is loaded, no prompt tested |
| Recording | NOT_IMPLEMENTED | State machine exists but no audio capture. UI shows fake recording on backend failure |
| Browser | NOT_IMPLEMENTED | Returns "not available" error |
| OCR | NOT_IMPLEMENTED | Returns "Tesseract integration pending" error |
| Image description | NOT_IMPLEMENTED | Returns "Gemma 4 vision pending" error |
| Camera | NOT_IMPLEMENTED | Returns "Camera not available" error |
| TTS | NOT_IMPLEMENTED | Language selectors shown but disabled |
| STT | NOT_IMPLEMENTED | Language selectors shown but disabled |
| Document parsing | PARTIALLY_IMPLEMENTED | TXT/Markdown reads work, PDF/DOCX returns error |
| Memory search | PARTIALLY_IMPLEMENTED | Text-based file matching, not vector embeddings |
| SHA-256 manifest | PARTIALLY_IMPLEMENTED | `sha2` crate referenced in code but **never compiled** |
| USB vault detection | PARTIALLY_IMPLEMENTED | Hardcoded drive letters only, no removable-drive scan |

## 8. Android Baseline — NOT TESTED

| Test | Status | Evidence |
|------|--------|----------|
| `./gradlew assembleDebug` | NOT_RUN | No build attempt made |
| Core contracts (9 tests) | NOT_RUN | No test attempt made |
| Encrypted vault (17 tests) | NOT_RUN | No test attempt made |
| Android JVM tests (549 claimed) | NOT_RUN | No test attempt made |
| Instrumented tests (42 claimed) | NOT_RUN | No test attempt made |

The previous audit claimed "should still pass" — this is not evidence.

## 9. macOS — NOT BUILT, NOT TESTED

No macOS build has been attempted. No macOS-specific code has been executed.

## 10. Cross-Platform Memory Sync — NOT TESTED

The Kotlin `core-contracts` define shared data types, but no test has verified that data written by Android can be read by Desktop, or vice versa. The Kotlin/Native interop layer for Rust does not exist.

## 11. Uncompiled Modules

| Module | Language | Lines | Compiled | Tested |
|--------|----------|-------|----------|--------|
| `main.rs` | Rust | ~260 | ❌ | ❌ |
| `llama.rs` | Rust | ~400 | ❌ | ❌ |
| `safety.rs` | Rust | ~200 | ❌ | ❌ |
| `recording.rs` | Rust | ~260 | ❌ | ❌ |
| `browser.rs` | Rust | ~70 | ❌ | ❌ |
| `documents.rs` | Rust | ~200 | ❌ | ❌ |
| `accessibility.rs` | Rust | ~120 | ❌ | ❌ |
| `security.rs` | Rust | ~260 | ❌ | ❌ |
| All 11 React components | TypeScript | ~2000 | ✅ Vite | ❌ No Tauri runtime |

## 12. Summary

| Category | VERIFIED_WORKING | BUILDS_NOT_RUNTIME_TESTED | PARTIALLY_IMPLEMENTED | NOT_IMPLEMENTED | BLOCKED_BY_ENVIRONMENT |
|----------|----------------|--------------------------|----------------------|----------------|----------------------|
| Frontend build | ✅ Vite | | | | |
| Frontend runtime | | | Chat UI | Browser, OCR, Vision, Camera, TTS, STT | |
| Rust backend | | | | | ❌ All 8 modules |
| Vault encryption | | | | ❌ No key derivation | |
| Model inference | | | | ❌ Never tested | |
| Recording audio | | | State machine only | Audio capture | |
| Android | | | | | ❌ Not built |
| macOS | | | | | ❌ Not built |
| USB detection | | | Hardcoded paths | Removable-drive scan | |
| Gemma 12B model | | | | ❌ File missing from USB | |

**The repository is NOT production-ready. The previous "COMPLETE" claim was false.**

---

## 13. False-Success Fixes (COMMITTED afac5ed, NOT PUSHED)

These fixes were made locally but **NOT committed or pushed** per instructions:

| File | Fix | Status |
|------|-----|--------|
| `RecordingView.tsx` | `startRecording` catch: no longer enters recording state on backend failure | FIXED, not committed |
| `RecordingView.tsx` | `stopRecording` catch: no longer updates UI on backend failure, shows error | FIXED, not committed |
| `RecordingView.tsx` | `pause`/`resume` catch: report errors, don't silently update state | FIXED, not committed |
| `RecordingView.tsx` | `bookmark` catch: report error, only increment on success | FIXED, not committed |
| `VaultView.tsx` | Emergency lock catch: report error instead of swallowing | FIXED, not committed |
| `VaultView.tsx` | `listModels` → `listDocuments` (was wrong API) | FIXED, not committed |
| `RecordingView.tsx` | Removed hardcoded `D:\UNOONE` default, uses detectVault() | FIXED, not committed |
| `llama.rs` | Model status: documented that port check is not model verification | FIXED, not committed |

## 14. No-Push Repair Plan

### Must be done BEFORE any push:

1. **Get Rust compiling** — Either:
   - Fix Windows App Control policy on this machine, OR
   - Set up CI (GitHub Actions) that runs `cargo check` + `cargo test`, OR
   - Build on a different Windows/macOS machine

2. **Download Gemma 4 12B Q4_K_M model** — The file is MISSING from USB. Must re-download and verify SHA-256.

3. **Verify model inference** — Launch `llama-server`, send one prompt, confirm tokens are generated.

4. **Run Android tests** — `./gradlew test` for JVM tests, `./gradlew connectedAndroidTest` for instrumented.

5. **Implement Argon2id vault unlock** — Either use `argon2` Rust crate or integrate Kotlin/Native.

6. **Cross-platform USB detection** — Replace hardcoded drive letters with real removable-drive enumeration.

7. **Fix model status** — Replace TCP port check with health endpoint verification + model hash check.

8. **Commit false-success fixes** — Once items 1-7 are verified, commit the RecordingView/VaultView fixes.

9. **Do NOT push** until all items above produce evidence with:
   - command executed
   - exit code
   - OS + hardware
   - log output
   - commit hash