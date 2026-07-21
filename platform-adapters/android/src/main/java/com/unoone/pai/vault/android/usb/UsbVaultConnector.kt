package com.unoone.pai.vault.android.usb

import android.content.Context
import android.hardware.usb.UsbManager
import android.net.Uri
import android.os.storage.StorageManager
import android.os.storage.StorageVolume
import android.provider.DocumentsContract
import android.provider.OpenableColumns
import java.io.File
import java.io.InputStream
import java.io.OutputStream

/**
 * Detects and manages UnoOne Pocket USB drive connections on Android.
 *
 * Uses Android's Storage Access Framework (SAF) for USB drive access
 * and USB Host API for direct USB communication where available.
 *
 * Flow:
 * 1. Detect USB connection via StorageManager / broadcast receiver
 * 2. Verify it's a valid UnoOne vault (check for VAULT/identity/vault.id)
 * 3. Present password-only unlock screen
 * 4. Derive key and unlock vault
 * 5. Monitor USB disconnection → flush, lock, clear cache
 */
class UsbVaultConnector(private val context: Context) {

    companion object {
        /** Marker file that identifies a UnoOne Pocket USB */
        private const val VAULT_ID_FILE = "VAULT/identity/vault.id"

        /** Marker file for vault header */
        private const val VAULT_HEADER_FILE = "VAULT/identity/vault.json.enc"

        /** UnoOne directory name on the USB drive */
        private const val UNOONE_DIR = "UNOONE"
    }

    /**
     * Check if a USB drive is connected and contains a UnoOne vault.
     *
     * On Android 10+, USB drives are accessed via Storage Access Framework.
     * On older devices, direct file access may be available.
     */
    fun detectVault(): VaultDetectionResult {
        // Method 1: Check external storage directories (pre-Android 10)
        val externalDirs = context.getExternalFilesDirs(null)
        for (dir in externalDirs) {
            if (dir != null) {
                val unooneDir = File(dir.parentFile?.parentFile?.parentFile?.parentFile, UNOONE_DIR)
                if (unooneDir.exists()) {
                    val vaultIdFile = File(unooneDir, VAULT_ID_FILE)
                    if (vaultIdFile.exists()) {
                        return VaultDetectionResult(
                            detected = true,
                            vaultRoot = unooneDir,
                            vaultId = vaultIdFile.readText().trim(),
                            connectionType = ConnectionType.DIRECT_FILE
                        )
                    }
                }
            }
        }

        // Method 2: Check /storage/ for USB drives (requires MANAGE_EXTERNAL_STORAGE on Android 11+)
        val storageDir = File("/storage")
        if (storageDir.exists()) {
            for (volume in storageDir.listFiles() ?: emptyArray()) {
                val unooneDir = File(volume, UNOONE_DIR)
                if (unooneDir.exists()) {
                    val vaultIdFile = File(unooneDir, VAULT_ID_FILE)
                    if (vaultIdFile.exists()) {
                        return VaultDetectionResult(
                            detected = true,
                            vaultRoot = unooneDir,
                            vaultId = vaultIdFile.readText().trim(),
                            connectionType = ConnectionType.DIRECT_FILE
                        )
                    }
                }
            }
        }

        // Method 3: Check D:\ drive (Windows-like path, common on some devices)
        val dDrive = File("D:/UNOONE")
        if (dDrive.exists()) {
            val vaultIdFile = File(dDrive, VAULT_ID_FILE)
            if (vaultIdFile.exists()) {
                return VaultDetectionResult(
                    detected = true,
                    vaultRoot = dDrive,
                    vaultId = vaultIdFile.readText().trim(),
                    connectionType = ConnectionType.DIRECT_FILE
                )
            }
        }

        return VaultDetectionResult(detected = false)
    }

    /**
     * Check if the detected vault is valid (has required structure).
     */
    fun isValidVault(vaultRoot: File): Boolean {
        val vaultId = File(vaultRoot, VAULT_ID_FILE)
        val vaultHeader = File(vaultRoot, VAULT_HEADER_FILE)
        return vaultId.exists() && vaultHeader.exists()
    }

    /**
     * Get the vault root directory on the USB drive.
     * Returns null if no vault is detected.
     */
    fun getVaultRoot(): File? {
        val result = detectVault()
        return if (result.detected) result.vaultRoot else null
    }
}

data class VaultDetectionResult(
    val detected: Boolean,
    val vaultRoot: File? = null,
    val vaultId: String? = null,
    val connectionType: ConnectionType = ConnectionType.NONE
)

enum class ConnectionType {
    NONE,           // No USB detected
    DIRECT_FILE,   // Direct file access (pre-Android 10 or desktop)
    SAF,            // Storage Access Framework (Android 10+)
    USB_HOST       // USB Host API (direct USB communication)
}