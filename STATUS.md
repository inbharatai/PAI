# UnoOne Pocket AI — Status

**Updated:** 2026-07-21  
**Repository:** https://github.com/inbharatai/PAI  
**Baseline tag:** `v0.4.0-alpha-v2-baseline`  
**USB vault:** `D:\PAI\UNOONE\` (460 GB SanDisk)  
**Release state:** **Alpha / not production-ready — Phase 0 complete, Phase 1 next**

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
| 1 | 🔲 Next | Shared contracts (core-contracts package) |
| 2 | 🔲 | Encrypted shared vault (PocketMemoryVault) |
| 3 | 🔲 | Android vault integration |
| 4 | 🔲 | Mobile recording workspace |
| 5 | 🔲 | Desktop foundation (Tauri + USB launch) |
| 6 | 🔲 | Gemma 4 12B desktop model |
| 7 | 🔲 | Desktop recording and voice |
| 8 | 🔲 | Browser workspace (Playwright + PageAgent) |
| 9 | 🔲 | Documents and memory retrieval |
| 10 | 🔲 | Accessibility and camera |
| 11 | 🔲 | Security hardening |
| 12 | 🔲 | Cross-platform validation |

## Phase 0 Deliverables

- ✅ Repository initialized at `github.com/inbharatai/PAI`
- ✅ Upstream remote set to `UnoOne-Local-Agent`
- ✅ Baseline commit: all 14 Android modules preserved intact
- ✅ Tag: `v0.4.0-alpha-v2-baseline`
- ✅ Codebase audit: 256 Kotlin files, ~44,800 lines
- ✅ Bug audit: 25 findings (6 HIGH, 11 MEDIUM, 8 LOW)
- ✅ USB vault directory structure created at `D:\UNOONE\`
- ✅ Migration plan documented in `MIGRATION-PLAN.md`

## Known Bugs (HIGH priority)

1. AccessibilityNodeInfo double-recycling in `UnoOneAccessibilityService`
2. FloatingAgentService ComposeView disposal order (remove before dispose)
3. SecurityLevel stored in plain SharedPreferences (should use EncryptedSharedPreferences)
4. `captureScreen()` blocks with Thread.sleep under @Synchronized lock (500ms max)
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
│   ├── identity/
│   ├── memory/{personal,preferences,conversations,tasks,knowledge,accessibility,skills}
│   ├── chats/
│   ├── recordings/{audio,transcripts,summaries}
│   ├── documents/
│   ├── reports/
│   ├── browser/
│   ├── camera/
│   ├── settings/
│   ├── indexes/
│   ├── audit/
│   └── recovery/
├── UPDATES/
├── RECOVERY/
└── MANIFESTS/
```

All vault content will be encrypted (Argon2id + XChaCha20-Poly1305).
USB is the single source of truth. Host storage is temporary cache only.