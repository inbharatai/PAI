# UnoOne Mobile Golden Baseline Protection Check — PowerShell
# Verifies that no file under android-app/UnoOneAgent/ has changed since the golden baseline.
# Uses both git diff and SHA-256 hash manifest verification.
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts/verify-mobile-untouched.ps1
# Exit codes: 0 = PASS, 1 = FAIL (changes detected), 2 = ERROR

param()

$ErrorActionPreference = "Stop"
$GoldenTag = "mobile-golden-baseline-v1"
$ProtectedPath = "android-app/UnoOneAgent/"
$HashManifest = "scripts/MOBILE_GOLDEN_HASHES.txt"

# Step 1: Verify the golden tag exists
$tagList = git tag -l $GoldenTag 2>$null
if ($LASTEXITCODE -ne 0 -or -not $tagList) {
    Write-Error "FAIL: Golden tag '$GoldenTag' not found."
    Write-Error "Run: git tag -a $GoldenTag -m 'Golden baseline' <commit>"
    exit 2
}

# Step 2: Git diff check
$diff = git diff $GoldenTag HEAD -- $ProtectedPath 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Error "FAIL: git diff returned error."
    exit 2
}

if ($diff) {
    Write-Error "FAIL: Changes detected in protected path '$ProtectedPath'"
    Write-Error "Golden baseline tag: $GoldenTag"
    Write-Error ""
    Write-Error "Changed files:"
    git diff --name-only $GoldenTag HEAD -- $ProtectedPath
    Write-Error ""
    Write-Error "The Android application must not be modified during desktop development."
    exit 1
}

# Step 3: Hash manifest verification
if (Test-Path $HashManifest) {
    $manifestLines = Get-Content $HashManifest | Where-Object {
        $_ -match '^[0-9a-f]{64}\s{2}' -and -not $_.StartsWith('#')
    }

    $hashCount = 0
    $mismatchCount = 0

    foreach ($line in $manifestLines) {
        $parts = $line -split '\s{2}', 2
        if ($parts.Length -ne 2) { continue }

        $expectedHash = $parts[0]
        $relativePath = $parts[1]
        $hashCount++

        if (-not (Test-Path $relativePath)) {
            Write-Error "FAIL: Protected file missing: $relativePath"
            $mismatchCount++
            continue
        }

        $actualHash = (Get-FileHash -Path $relativePath -Algorithm SHA256).Hash.ToLower()
        if ($actualHash -ne $expectedHash) {
            Write-Error "FAIL: Hash mismatch for $relativePath"
            Write-Error "  Expected: $expectedHash"
            Write-Error "  Actual:   $actualHash"
            $mismatchCount++
        }
    }

    if ($mismatchCount -gt 0) {
        Write-Error ""
        Write-Error "FAIL: $mismatchCount of $hashCount protected files have hash mismatches."
        exit 1
    }

    Write-Output "PASS: All $hashCount protected files match the golden baseline hashes."
} else {
    Write-Warning "WARN: Hash manifest not found at $HashManifest. Skipping hash verification."
}

# Step 4: Verify no files were added or removed
$currentFiles = git ls-tree -r --name-only HEAD -- $ProtectedPath
$baselineFiles = git ls-tree -r --name-only $GoldenTag -- $ProtectedPath

$currentSet = $currentFiles -split "`n" | Where-Object { $_.Trim() -ne '' } | Sort-Object
$baselineSet = $baselineFiles -split "`n" | Where-Object { $_.Trim() -ne '' } | Sort-Object

$added = $currentSet | Where-Object { $_ -notin $baselineSet }
$removed = $baselineSet | Where-Object { $_ -notin $currentSet }

if ($added.Count -gt 0) {
    Write-Error "FAIL: Files added to protected path:"
    $added | ForEach-Object { Write-Error "  + $_" }
    exit 1
}

if ($removed.Count -gt 0) {
    Write-Error "FAIL: Files removed from protected path:"
    $removed | ForEach-Object { Write-Error "  - $_" }
    exit 1
}

Write-Output "PASS: No changes detected in protected path '$ProtectedPath'"
Write-Output "Golden baseline tag: $GoldenTag"
Write-Output "Protected file count: $($currentSet.Count)"
exit 0