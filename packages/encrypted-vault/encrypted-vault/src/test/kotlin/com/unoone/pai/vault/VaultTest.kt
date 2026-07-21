package com.unoone.pai.vault

import com.unoone.pai.contracts.*
import com.unoone.pai.vault.crypto.Argon2idKdf
import com.unoone.pai.vault.crypto.Aes256GcmCipher
import com.unoone.pai.vault.crypto.VaultCipher
import com.unoone.pai.vault.crypto.XChaCha20Poly1305Cipher
import com.unoone.pai.vault.journal.WriteAheadJournal
import com.unoone.pai.vault.session.DeviceSessionManager
import com.unoone.pai.vault.storage.VaultStorage
import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.io.File

class VaultCryptoTest {

    @Test
    fun argon2idConsistentKeyDerivation() {
        val password = "test-vault-password-2026".toCharArray()
        val salt = Argon2idKdf.generateSalt()
        val key1 = Argon2idKdf.deriveKey(password, salt)
        val key2 = Argon2idKdf.deriveKey(password, salt)
        assertArrayEquals(key1, key2, "Same password + salt must produce same key")
        assertEquals(32, key1.size, "Key must be 32 bytes")
    }

    @Test
    fun argon2idDifferentPasswordsProduceDifferentKeys() {
        val salt = Argon2idKdf.generateSalt()
        val key1 = Argon2idKdf.deriveKey("password-one".toCharArray(), salt)
        val key2 = Argon2idKdf.deriveKey("password-two".toCharArray(), salt)
        assertFalse(key1.contentEquals(key2), "Different passwords must produce different keys")
    }

    @Test
    fun argon2idDomainKeyDerivation() {
        val masterKey = ByteArray(32).also { java.security.SecureRandom().nextBytes(it) }
        val memoriesKey = Argon2idKdf.deriveDomainKey(masterKey, "memories")
        val chatsKey = Argon2idKdf.deriveDomainKey(masterKey, "chats")
        assertFalse(memoriesKey.contentEquals(chatsKey), "Different domains must produce different keys")
        assertEquals(32, memoriesKey.size, "Domain key must be 32 bytes")
    }

    @Test
    fun xchacha20Roundtrip() {
        val key = ByteArray(32).also { java.security.SecureRandom().nextBytes(it) }
        val plaintext = "Hello, UnoOne Pocket AI!".encodeToByteArray()
        val aad = "vault-memories".encodeToByteArray()
        val encrypted = XChaCha20Poly1305Cipher.encrypt(key, plaintext, aad)
        val decrypted = XChaCha20Poly1305Cipher.decrypt(key, encrypted, aad)
        assertArrayEquals(plaintext, decrypted, "Decrypted data must match original")
        assertTrue(encrypted.size > plaintext.size, "Encrypted data must be larger (nonce + tag)")
    }

    @Test
    fun xchacha20RejectsTamperedData() {
        val key = ByteArray(32).also { java.security.SecureRandom().nextBytes(it) }
        val plaintext = "Sensitive data".encodeToByteArray()
        val encrypted = XChaCha20Poly1305Cipher.encrypt(key, plaintext)
        val tampered = encrypted.copyOf()
        tampered[tampered.size - 1] = (tampered[tampered.size - 1].toInt() xor 0xFF).toByte()
        assertThrows(SecurityException::class.java) {
            XChaCha20Poly1305Cipher.decrypt(key, tampered)
        }
    }

    @Test
    fun xchacha20RejectsWrongKey() {
        val key1 = ByteArray(32).also { java.security.SecureRandom().nextBytes(it) }
        val key2 = ByteArray(32).also { java.security.SecureRandom().nextBytes(it) }
        val plaintext = "Secret message".encodeToByteArray()
        val encrypted = XChaCha20Poly1305Cipher.encrypt(key1, plaintext)
        assertThrows(SecurityException::class.java) {
            XChaCha20Poly1305Cipher.decrypt(key2, encrypted)
        }
    }

    @Test
    fun aes256GcmRoundtrip() {
        val key = ByteArray(32).also { java.security.SecureRandom().nextBytes(it) }
        val plaintext = "Hello from AES-GCM!".encodeToByteArray()
        val aad = "vault-chats".encodeToByteArray()
        val encrypted = Aes256GcmCipher.encrypt(key, plaintext, aad)
        val decrypted = Aes256GcmCipher.decrypt(key, encrypted, aad)
        assertArrayEquals(plaintext, decrypted, "AES-GCM decrypted data must match original")
    }

    @Test
    fun aes256GcmRejectsTamperedData() {
        val key = ByteArray(32).also { java.security.SecureRandom().nextBytes(it) }
        val plaintext = "AES secret".encodeToByteArray()
        val encrypted = Aes256GcmCipher.encrypt(key, plaintext)
        val tampered = encrypted.copyOf()
        tampered[0] = (tampered[0].toInt() xor 0xFF).toByte()
        assertThrows(SecurityException::class.java) {
            Aes256GcmCipher.decrypt(key, tampered)
        }
    }

    @Test
    fun vaultCipherDetectsBestAlgorithm() {
        val algorithm = VaultCipher.detectBestAlgorithm()
        assertNotNull(algorithm)
        assertTrue(algorithm == VaultCipher.Algorithm.XCHACHA20_POLY1305 ||
                   algorithm == VaultCipher.Algorithm.AES_256_GCM)
    }

    @Test
    fun vaultCipherRoundtripBothAlgorithms() {
        for (algo in VaultCipher.Algorithm.entries) {
            val key = ByteArray(32).also { java.security.SecureRandom().nextBytes(it) }
            val plaintext = "Testing $algo".encodeToByteArray()
            val encrypted = VaultCipher.encrypt(algo, key, plaintext)
            val decrypted = VaultCipher.decrypt(algo, key, encrypted)
            assertArrayEquals(plaintext, decrypted, "$algo roundtrip must work")
        }
    }
}

class VaultStorageTest {

    @TempDir
    lateinit var tempDir: File

    @Test
    fun vaultStorageWritesAndReadsEncryptedData() {
        val vaultDir = File(tempDir, "vault")
        vaultDir.mkdirs()
        val journalDir = File(tempDir, "journal")
        journalDir.mkdirs()

        val storage = VaultStorage(vaultDir, journalDir)
        val key = ByteArray(32).also { java.security.SecureRandom().nextBytes(it) }
        storage.setMasterKey(key)
        storage.setCipherAlgorithm(VaultCipher.Algorithm.XCHACHA20_POLY1305)

        val data = "Hello, vault storage!".encodeToByteArray()
        storage.write("memory/personal", "test-001", data)

        val read = storage.read("memory/personal", "test-001")
        assertArrayEquals(data, read, "Read data must match written data")
    }

    @Test
    fun vaultStorageDeleteCreatesTombstone() {
        val vaultDir = File(tempDir, "vault-delete")
        vaultDir.mkdirs()
        val journalDir = File(tempDir, "journal-delete")
        journalDir.mkdirs()

        val storage = VaultStorage(vaultDir, journalDir)
        val key = ByteArray(32).also { java.security.SecureRandom().nextBytes(it) }
        storage.setMasterKey(key)
        storage.setCipherAlgorithm(VaultCipher.Algorithm.XCHACHA20_POLY1305)

        val data = "Delete me".encodeToByteArray()
        storage.write("memory/personal", "test-delete", data)
        assertTrue(storage.exists("memory/personal", "test-delete"))

        storage.delete("memory/personal", "test-delete")
        assertFalse(storage.exists("memory/personal", "test-delete"))
    }

    @Test
    fun vaultStorageListsRecordsExcludingTombstones() {
        val vaultDir = File(tempDir, "vault-list")
        vaultDir.mkdirs()
        val journalDir = File(tempDir, "journal-list")
        journalDir.mkdirs()

        val storage = VaultStorage(vaultDir, journalDir)
        val key = ByteArray(32).also { java.security.SecureRandom().nextBytes(it) }
        storage.setMasterKey(key)
        storage.setCipherAlgorithm(VaultCipher.Algorithm.XCHACHA20_POLY1305)

        storage.write("memory/personal", "record-1", "Data 1".encodeToByteArray())
        storage.write("memory/personal", "record-2", "Data 2".encodeToByteArray())
        storage.write("memory/personal", "record-3", "Data 3".encodeToByteArray())
        storage.delete("memory/personal", "record-2")

        val list = storage.list("memory/personal")
        assertEquals(2, list.size)
        assertTrue(list.contains("record-1"))
        assertTrue(list.contains("record-3"))
        assertFalse(list.contains("record-2"))
    }

    @Test
    fun vaultStorageRejectsReadWhenLocked() {
        val vaultDir = File(tempDir, "vault-locked")
        vaultDir.mkdirs()
        val journalDir = File(tempDir, "journal-locked")
        journalDir.mkdirs()

        val storage = VaultStorage(vaultDir, journalDir)
        assertThrows(IllegalArgumentException::class.java) {
            storage.read("memory/personal", "test-001")
        }
    }
}

class PocketMemoryVaultIntegrationTest {

    @TempDir
    lateinit var tempDir: File

    @Test
    fun fullVaultLifecycleSetupUnlockReadWriteLock() = runBlocking {
        val vaultRoot = File(tempDir, "vault-lifecycle")
        val vault = PocketMemoryVaultImpl(vaultRoot)

        // 1. Setup vault
        val setupResult = vault.setupVault("MySecurePassword2026!", "My UnoOne")
        assertTrue(setupResult.success)
        assertNotNull(setupResult.vaultId)
        assertNotNull(setupResult.recoveryKey)
        assertTrue(vault.isUnlocked)

        // 2. Write a memory
        val memory = Memory(
            metadata = VaultRecordMetadata(
                id = "mem-001",
                createdAt = "2026-07-21T10:00:00Z",
                updatedAt = "2026-07-21T10:00:00Z",
                sourcePlatform = Platform.WINDOWS,
                sourceDeviceId = "laptop-001",
                revision = 1
            ),
            type = MemoryType.PREFERENCE,
            key = "preferred_written_language",
            content = "English for written, Hindi for spoken"
        )
        val memoryId = vault.addMemory(memory) { kotlinx.serialization.json.Json.encodeToString(Memory.serializer(), it) }
        assertNotNull(memoryId)

        // 3. Save a conversation
        val conversation = Conversation(
            metadata = VaultRecordMetadata(
                id = "conv-001",
                createdAt = "2026-07-21T10:00:00Z",
                updatedAt = "2026-07-21T10:05:00Z",
                sourcePlatform = Platform.WINDOWS,
                sourceDeviceId = "laptop-001",
                revision = 1
            ),
            title = "University analysis",
            messages = listOf(
                ConversationMessage(
                    id = "msg-001",
                    role = MessageRole.USER,
                    content = "Compare these university offers",
                    timestamp = "2026-07-21T10:00:00Z",
                    platform = Platform.WINDOWS,
                    modelUsed = "gemma-4-12b"
                )
            )
        )
        val convId = vault.saveConversation(conversation)
        assertEquals("conv-001", convId)

        // 4. Lock the vault
        vault.lockVault()
        assertFalse(vault.isUnlocked)

        // 5. Unlock again
        val unlockResult = vault.unlockVault("MySecurePassword2026!")
        assertTrue(unlockResult.success)

        // 6. Verify metadata is accessible
        val metadata = vault.getMetadata()
        assertEquals("My UnoOne", metadata.profileName)
        assertEquals("Argon2id", metadata.kdfAlgorithm)
    }

    @Test
    fun wrongPasswordIsRejected() = runBlocking {
        val vaultRoot = File(tempDir, "vault-wrong-password")
        val vault = PocketMemoryVaultImpl(vaultRoot)

        vault.setupVault("CorrectPassword123!", null)
        vault.lockVault()
        val result = vault.unlockVault("WrongPassword456!")
        assertFalse(result.success)
        assertTrue(result.failedAttempts > 0)
    }

    @Test
    fun emergencyLockClearsAllKeysImmediately() = runBlocking {
        val vaultRoot = File(tempDir, "vault-emergency")
        val vault = PocketMemoryVaultImpl(vaultRoot)

        vault.setupVault("EmergencyTest2026!", null)
        assertTrue(vault.isUnlocked)

        vault.emergencyLock()
        assertFalse(vault.isUnlocked)
    }
}