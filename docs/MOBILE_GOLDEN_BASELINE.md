# UnoOne Mobile Golden Baseline — Protection and Status

**Created:** 2026-07-21
**Tag:** `mobile-golden-baseline-v1`
**Commit:** `ae35ec7`
**Protected path:** `android-app/UnoOneAgent/` (315 files)
**Status:** FROZEN — no changes authorized during desktop development

## Protection

Two verification scripts exist:

- `scripts/verify-mobile-untouched.sh` — bash script for CI/terminal
- `scripts/verify-mobile-untouched.py` — Python script for environments with Python 3

Both compare the current state of `android-app/UnoOneAgent/` against the `mobile-golden-baseline-v1` tag.

### CI Integration

Add to any CI pipeline:

```yaml
- name: Verify mobile untouched
  run: bash scripts/verify-mobile-untouched.sh
```

Or:

```yaml
- name: Verify mobile untouched
  run: python3 scripts/verify-mobile-untouched.py
```

### Pre-commit Hook (optional)

```bash
# .git/hooks/pre-commit
bash scripts/verify-mobile-untouched.sh || exit 1
```

## Mobile Integration Plan

The mobile app will NOT be modified for USB vault support in the current repository.

Instead, a separate integration target will be created:

```
UnoOne-Mobile-USB-Integration
```

This will only happen AFTER:
1. Desktop vault format passes all Windows and macOS tests
2. The vault specification is finalized and versioned
3. A separate clone or branch is created for integration
4. Full regression tests pass on the integration copy
5. The original golden baseline remains untouched

## Current Android Status

- **Last verified commit:** ae35ec7 (mobile-golden-baseline-v1 tag)
- **Build status:** NOT TESTED in this session (blocked by environment)
- **Tests:** NOT RUN in this session
- **Changes since baseline:** ZERO (verified by `git diff`)

## What NOT to do

- Do not edit any Kotlin file in android-app/
- Do not modify Android Gradle files
- Do not add USB vault code to the mobile app
- Do not refactor Android modules
- Do not copy desktop code into Android
- Do not change Android dependencies
- Do not update Android documentation in a way that changes behavior
- Do not apply formatting-only changes to Android code