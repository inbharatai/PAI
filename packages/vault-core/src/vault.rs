// UnoOne Vault Core — Main Vault API
// This is the primary interface for vault operations.
// It ties together crypto, header, recovery, record, and journal modules.
//
// Design (directive §16):
//   Password → Argon2id → Key-encryption key → Wraps random vault master key
//   This allows password changes without re-encrypting every record.

use std::path::{Path, PathBuf};
use zeroize::Zeroize;

use crate::crypto::*;
use crate::error::VaultError;
use crate::header::{VaultHeader, HEADER_A_FILE, HEADER_B_FILE};
use crate::journal::Journal;
use crate::record::{EncryptedRecord, PrivacyLevel, Record, RecordType};
use crate::recovery::RecoveryPhrase;

/// Minimum password length (directive §16: no short passwords)
pub const MIN_PASSWORD_LEN: usize = 8;

/// Vault state
#[derive(Debug, Clone, PartialEq)]
pub enum VaultState {
    /// Vault is locked — no keys in memory
    Locked,
    /// Vault is unlocked — master key is in memory
    Unlocked,
}

/// The main Vault struct
/// Holds the vault root path, header, and (when unlocked) the master key
pub struct Vault {
    /// Root path of the UNOONE directory on the USB
    vault_root: PathBuf,
    /// Current state
    state: VaultState,
    /// Vault header (loaded from disk, always available when vault exists)
    header: Option<VaultHeader>,
    /// The vault master key (only in memory when unlocked)
    /// Zeroed on lock
    master_key: Option<[u8; MASTER_KEY_LEN]>,
    /// Current active header slot (A or B) for double-buffering
    active_header_slot: HeaderSlot,
    /// Journal manager
    journal: Journal,
}

/// Which header slot is currently active (directive §17: double-buffered headers)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeaderSlot {
    A,
    B,
}

/// Result of vault creation
#[derive(Debug)]
pub struct VaultCreateResult {
    /// The generated recovery phrase (24 words)
    /// MUST be shown to the user exactly once, then never stored in plaintext
    pub recovery_phrase: Vec<String>,
    /// The vault ID
    pub vault_id: String,
}

/// Result of vault unlock
#[derive(Debug)]
pub struct VaultUnlockResult {
    pub vault_id: String,
    pub is_recovery_unlock: bool,
}

impl Vault {
    /// Create a new vault at the specified path
    /// This creates the directory structure, generates the master key,
    /// wraps it with the password, and stores the header
    pub fn create(vault_root: &Path, password: &[u8]) -> Result<VaultCreateResult, VaultError> {
        if password.len() < MIN_PASSWORD_LEN {
            return Err(VaultError::InvalidPassword(format!(
                "Password must be at least {} characters", MIN_PASSWORD_LEN
            )));
        }

        // Validate vault root path
        Self::validate_vault_root(vault_root)?;

        // Create directory structure
        Self::create_directory_structure(vault_root)?;

        // Generate vault ID
        let vault_id = uuid::Uuid::new_v4().to_string();

        // Create vault header with password-derived key wrapping
        let (header, master_key) = VaultHeader::create(password, &vault_id)?;

        // Enable recovery
        let (header, recovery_secret) = header.enable_recovery(password)?;
        let recovery_phrase = RecoveryPhrase::from_secret(recovery_secret);

        // Write header to slot A (initial creation always uses slot A)
        let header_path = vault_root.join("VAULT").join("header").join(HEADER_A_FILE);
        std::fs::create_dir_all(header_path.parent().unwrap())?;
        header.save_to_file(&header_path)?;

        // Create the vault.id file
        let vault_id_path = vault_root.join("VAULT").join("identity").join("vault.id");
        std::fs::create_dir_all(vault_id_path.parent().unwrap())?;
        std::fs::write(&vault_id_path, &vault_id)?;

        // Write a lock marker indicating vault is created but not yet unlocked
        let lock_marker = vault_root.join("VAULT").join("locks").join(".vault-created");
        std::fs::create_dir_all(lock_marker.parent().unwrap())?;
        std::fs::write(&lock_marker, chrono::Utc::now().to_rfc3339())?;

        // Zero the master key from memory
        drop(master_key);

        Ok(VaultCreateResult {
            recovery_phrase: recovery_phrase.words,
            vault_id,
        })
    }

    /// Open an existing vault (without unlocking it)
    /// Loads the header from disk but does NOT derive any keys
    pub fn open(vault_root: &Path) -> Result<Vault, VaultError> {
        Self::validate_vault_root(vault_root)?;

        // Try to load header from slot A first, then slot B
        let header_path_a = vault_root.join("VAULT").join("header").join(HEADER_A_FILE);
        let header_path_b = vault_root.join("VAULT").join("header").join(HEADER_B_FILE);

        let (header, active_slot) = if header_path_a.exists() {
            let header = VaultHeader::load_from_file(&header_path_a)?;
            (header, HeaderSlot::A)
        } else if header_path_b.exists() {
            let header = VaultHeader::load_from_file(&header_path_b)?;
            (header, HeaderSlot::B)
        } else {
            return Err(VaultError::VaultNotFound(format!(
                "No vault header found at {}", vault_root.display()
            )));
        };

        let journal = Journal::new(vault_root);

        Ok(Vault {
            vault_root: vault_root.to_path_buf(),
            state: VaultState::Locked,
            header: Some(header),
            master_key: None,
            active_header_slot: active_slot,
            journal,
        })
    }

    /// Unlock the vault with a password
    /// Derives the key-encryption key from the password, verifies the header HMAC,
    /// and unwraps the master key
    pub fn unlock(&mut self, password: &[u8]) -> Result<VaultUnlockResult, VaultError> {
        if password.is_empty() {
            return Err(VaultError::InvalidPassword("Password cannot be empty".to_string()));
        }

        if self.state == VaultState::Unlocked {
            return Err(VaultError::VaultUnlocked);
        }

        let header = self.header.as_ref()
            .ok_or_else(|| VaultError::VaultNotFound("No vault header loaded".to_string()))?;

        let master_key = header.unlock_with_password(password)?;

        self.master_key = Some(master_key);
        self.state = VaultState::Unlocked;

        // Recover from any crash (roll back pending transactions)
        let recovery = self.journal.recover_from_crash()?;
        if recovery.recovery_needed {
            eprintln!("Vault journal recovery: {} committed, {} rolled back",
                recovery.committed_count, recovery.rolled_back_count);
        }

        Ok(VaultUnlockResult {
            vault_id: header.vault_id.clone(),
            is_recovery_unlock: false,
        })
    }

    /// Unlock the vault with a recovery phrase (24 words)
    pub fn unlock_with_recovery(&mut self, words: &[String]) -> Result<VaultUnlockResult, VaultError> {
        if self.state == VaultState::Unlocked {
            return Err(VaultError::VaultUnlocked);
        }

        let header = self.header.as_ref()
            .ok_or_else(|| VaultError::VaultNotFound("No vault header loaded".to_string()))?;

        let phrase = RecoveryPhrase::from_words(words)?;
        let master_key = header.unlock_with_recovery(phrase.secret())?;

        self.master_key = Some(master_key);
        self.state = VaultState::Unlocked;

        // Recover from any crash
        let _ = self.journal.recover_from_crash()?;

        Ok(VaultUnlockResult {
            vault_id: header.vault_id.clone(),
            is_recovery_unlock: true,
        })
    }

    /// Lock the vault — zero all keys from memory
    pub fn lock(&mut self) -> Result<(), VaultError> {
        if self.state == VaultState::Locked {
            return Err(VaultError::VaultLocked);
        }

        // Zero the master key from memory
        if let Some(mut key) = self.master_key.take() {
            secure_zero(&mut key);
        }

        // Zero all domain keys from memory
        // (They're computed on-demand from the master key, so dropping the
        //  master key is sufficient, but we clear the state explicitly)
        self.master_key = None;
        self.state = VaultState::Locked;

        // Write a lock marker
        let lock_marker = self.vault_root.join("VAULT").join("locks").join(".vault-locked");
        if let Some(parent) = lock_marker.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&lock_marker, chrono::Utc::now().to_rfc3339());

        Ok(())
    }

    /// Change the vault password
    /// This re-wraps the master key with the new password
    /// and writes a new header to the inactive slot (double-buffered)
    pub fn change_password(&mut self, old_password: &[u8], new_password: &[u8]) -> Result<(), VaultError> {
        if new_password.len() < MIN_PASSWORD_LEN {
            return Err(VaultError::InvalidPassword(format!(
                "New password must be at least {} characters", MIN_PASSWORD_LEN
            )));
        }

        let header = self.header.as_ref()
            .ok_or_else(|| VaultError::VaultNotFound("No vault header loaded".to_string()))?;

        let new_header = header.change_password(old_password, new_password)?;

        // Write to the inactive slot
        let inactive_slot = match self.active_header_slot {
            HeaderSlot::A => HeaderSlot::B,
            HeaderSlot::B => HeaderSlot::A,
        };

        let header_path = self.vault_root.join("VAULT").join("header").join(
            match inactive_slot {
                HeaderSlot::A => HEADER_A_FILE,
                HeaderSlot::B => HEADER_B_FILE,
            }
        );

        std::fs::create_dir_all(header_path.parent().unwrap())?;
        new_header.save_to_file(&header_path)?;

        self.header = Some(new_header);
        self.active_header_slot = inactive_slot;

        Ok(())
    }

    /// Get the current vault state
    pub fn state(&self) -> &VaultState {
        &self.state
    }

    /// Get the vault ID
    pub fn vault_id(&self) -> Option<&str> {
        self.header.as_ref().map(|h| h.vault_id.as_str())
    }

    /// Get the vault root path
    pub fn vault_root(&self) -> &Path {
        &self.vault_root
    }

    /// Check if the vault is unlocked
    pub fn is_unlocked(&self) -> bool {
        self.state == VaultState::Unlocked && self.master_key.is_some()
    }

    /// Get the master key (only available when unlocked)
    pub fn master_key(&self) -> Option<&[u8; MASTER_KEY_LEN]> {
        self.master_key.as_ref()
    }

    /// Encrypt a record and write it to the vault
    pub fn write_record(&mut self, record: Record, content: &[u8]) -> Result<(), VaultError> {
        if !self.is_unlocked() {
            return Err(VaultError::VaultLocked);
        }

        let master_key = self.master_key.as_ref()
            .ok_or(VaultError::VaultLocked)?;

        // Derive domain-specific key for records
        let domain_key = derive_domain_key(master_key, crate::crypto::DOMAIN_RECORDS);
        let nonce = generate_nonce();

        // AAD = JSON of the record metadata (authenticated but not encrypted)
        let aad = serde_json::to_vec(&record)
            .map_err(|e| VaultError::Serialization(e.to_string()))?;

        // Encrypt content with domain key
        let encrypted_content = encrypt(&domain_key, &nonce, content, &aad)?;

        // Compute content hash
        let content_hash = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(content);
            hex::encode(hasher.finalize())
        };

        // Create encrypted record
        let mut encrypted_record = EncryptedRecord {
            metadata: record.clone(),
            encrypted_content: hex::encode(&encrypted_content),
            nonce: hex::encode(&nonce),
            associated_data: hex::encode(&aad),
        };

        encrypted_record.metadata.content_hash = content_hash;
        encrypted_record.metadata.mark_updated();

        // Write to vault
        let record_path = self.vault_root
            .join("VAULT")
            .join("records")
            .join(format!("{}.enc.json", record.record_id));

        std::fs::create_dir_all(record_path.parent().unwrap())?;

        let json = serde_json::to_string_pretty(&encrypted_record)
            .map_err(|e| VaultError::Serialization(e.to_string()))?;

        // Atomic write: write to temp, then rename
        let temp_path = record_path.with_extension("tmp");
        std::fs::write(&temp_path, &json)?;
        std::fs::rename(&temp_path, &record_path)?;

        Ok(())
    }

    /// Read and decrypt a record from the vault
    pub fn read_record(&self, record_id: &str) -> Result<(Record, Vec<u8>), VaultError> {
        if !self.is_unlocked() {
            return Err(VaultError::VaultLocked);
        }

        let master_key = self.master_key.as_ref()
            .ok_or(VaultError::VaultLocked)?;

        let record_path = self.vault_root
            .join("VAULT")
            .join("records")
            .join(format!("{}.enc.json", record_id));

        if !record_path.exists() {
            return Err(VaultError::VaultNotFound(format!(
                "Record {} not found", record_id
            )));
        }

        let content = std::fs::read_to_string(&record_path)?;
        let encrypted_record: EncryptedRecord = serde_json::from_str(&content)
            .map_err(|e| VaultError::Serialization(e.to_string()))?;

        // Derive domain key
        let domain_key = derive_domain_key(master_key, crate::crypto::DOMAIN_RECORDS);

        // Decode nonce and ciphertext
        let nonce: [u8; NONCE_LEN] = hex::decode(&encrypted_record.nonce)
            .map_err(|e| VaultError::DecryptionFailed(format!("Invalid nonce: {}", e)))?
            .try_into()
            .map_err(|_| VaultError::DecryptionFailed("Nonce has wrong length".to_string()))?;

        let ciphertext = hex::decode(&encrypted_record.encrypted_content)
            .map_err(|e| VaultError::DecryptionFailed(format!("Invalid ciphertext: {}", e)))?;

        let aad = hex::decode(&encrypted_record.associated_data)
            .map_err(|e| VaultError::DecryptionFailed(format!("Invalid AAD: {}", e)))?;

        // Decrypt
        let plaintext = decrypt(&domain_key, &nonce, &ciphertext, &aad)?;

        // Check for tombstone
        if encrypted_record.metadata.tombstone {
            return Err(VaultError::NotPermitted(format!(
                "Record {} has been deleted", record_id
            )));
        }

        Ok((encrypted_record.metadata, plaintext))
    }

    /// Delete a record by creating a tombstone
    pub fn delete_record(&mut self, record_id: &str, origin_platform: &str, origin_device_id: &str) -> Result<(), VaultError> {
        if !self.is_unlocked() {
            return Err(VaultError::VaultLocked);
        }

        // Create tombstone record
        let tombstone = Record::create_tombstone(record_id, origin_platform, origin_device_id);

        // Write tombstone (overwrites the original record)
        self.write_record(tombstone, b"DELETED")?;

        Ok(())
    }

    /// Validate that a vault root path is safe (no path traversal)
    fn validate_vault_root(vault_root: &Path) -> Result<(), VaultError> {
        let canonical = vault_root.canonicalize()
            .or_else(|_| {
                // Path may not exist yet for creation
                std::fs::create_dir_all(vault_root)?;
                vault_root.canonicalize()
            })?;

        let path_str = canonical.to_string_lossy();

        // Check for path traversal
        if path_str.contains("..") {
            return Err(VaultError::PathTraversal("Vault path contains '..'".to_string()));
        }

        Ok(())
    }

    /// Create the vault directory structure (directive §7)
    fn create_directory_structure(vault_root: &Path) -> Result<(), VaultError> {
        let dirs = [
            "VAULT/identity",
            "VAULT/header",
            "VAULT/records",
            "VAULT/indexes",
            "VAULT/journal",
            "VAULT/transactions",
            "VAULT/attachments",
            "VAULT/snapshots",
            "VAULT/locks",
            "VAULT/recovery",
            "CONFIG",
            "RECOVERY",
            "UPDATES/staging",
            "UPDATES/rollback",
            "LOGS/encrypted",
        ];

        for dir in &dirs {
            std::fs::create_dir_all(vault_root.join(dir))?;
        }

        Ok(())
    }
}

impl Drop for Vault {
    fn drop(&mut self) {
        // Zero the master key from memory on drop
        if let Some(mut key) = self.master_key.take() {
            secure_zero(&mut key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_vault(dir: &tempfile::TempDir) -> Vault {
        let vault_root = dir.path().join("UNOONE");
        let result = Vault::create(&vault_root, b"test-password-12345").unwrap();

        // Verify recovery phrase was generated
        assert_eq!(result.recovery_phrase.len(), 24);
        assert!(!result.vault_id.is_empty());

        // Open the vault
        let mut vault = Vault::open(&vault_root).unwrap();
        assert_eq!(vault.state(), &VaultState::Locked);

        // Unlock with password
        let unlock_result = vault.unlock(b"test-password-12345").unwrap();
        assert_eq!(vault.state(), &VaultState::Unlocked);
        assert!(!unlock_result.is_recovery_unlock);

        vault
    }

    #[test]
    fn test_vault_create_and_unlock() {
        let dir = tempfile::tempdir().unwrap();
        let _vault = create_test_vault(&dir);
    }

    #[test]
    fn test_wrong_password_fails() {
        let dir = tempfile::tempdir().unwrap();
        let vault_root = dir.path().join("UNOONE");
        Vault::create(&vault_root, b"correct-password-12345").unwrap();

        let mut vault = Vault::open(&vault_root).unwrap();
        let result = vault.unlock(b"wrong-password-12345!!!");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VaultError::WrongPassword));
    }

    #[test]
    fn test_empty_password_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let vault_root = dir.path().join("UNOONE");

        let result = Vault::create(&vault_root, b"");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VaultError::InvalidPassword(_)));
    }

    #[test]
    fn test_short_password_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let vault_root = dir.path().join("UNOONE");

        let result = Vault::create(&vault_root, b"short");
        assert!(result.is_err());
    }

    #[test]
    fn test_lock_and_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let vault_root = dir.path().join("UNOONE");
        Vault::create(&vault_root, b"test-password-12345").unwrap();

        let mut vault = Vault::open(&vault_root).unwrap();
        vault.unlock(b"test-password-12345").unwrap();
        assert!(vault.is_unlocked());

        // Lock the vault
        vault.lock().unwrap();
        assert_eq!(vault.state(), &VaultState::Locked);
        assert!(!vault.is_unlocked());

        // Unlock again
        vault.unlock(b"test-password-12345").unwrap();
        assert!(vault.is_unlocked());
    }

    #[test]
    fn test_change_password() {
        let dir = tempfile::tempdir().unwrap();
        let vault_root = dir.path().join("UNOONE");
        Vault::create(&vault_root, b"old-password-12345").unwrap();

        let mut vault = Vault::open(&vault_root).unwrap();
        vault.unlock(b"old-password-12345").unwrap();

        // Change password
        vault.change_password(b"old-password-12345", b"new-password-12345").unwrap();

        // Lock and reopen with new password
        vault.lock().unwrap();
        vault.unlock(b"new-password-12345").unwrap();
        assert!(vault.is_unlocked());

        // Old password should fail
        vault.lock().unwrap();
        let result = vault.unlock(b"old-password-12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_recovery_unlock() {
        let dir = tempfile::tempdir().unwrap();
        let vault_root = dir.path().join("UNOONE");

        let result = Vault::create(&vault_root, b"test-password-12345").unwrap();
        let recovery_words = result.recovery_phrase;

        let mut vault = Vault::open(&vault_root).unwrap();
        let unlock_result = vault.unlock_with_recovery(&recovery_words).unwrap();
        assert!(vault.is_unlocked());
        assert!(unlock_result.is_recovery_unlock);
    }

    #[test]
    fn test_invalid_recovery_words() {
        let dir = tempfile::tempdir().unwrap();
        let vault_root = dir.path().join("UNOONE");
        Vault::create(&vault_root, b"test-password-12345").unwrap();

        let mut vault = Vault::open(&vault_root).unwrap();
        let wrong_words: Vec<String> = (0..24).map(|i| format!("wrongword{}", i)).collect();
        let result = vault.unlock_with_recovery(&wrong_words);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_and_read_record() {
        let dir = tempfile::tempdir().unwrap();
        let mut vault = create_test_vault(&dir);

        let record = Record::new(RecordType::Conversation, "DESKTOP", "device-001");
        let content = b"Hello, this is a conversation message.";

        vault.write_record(record.clone(), content).unwrap();

        let (read_record, read_content) = vault.read_record(&record.record_id).unwrap();
        assert_eq!(read_content, content);
        assert_eq!(read_record.record_id, record.record_id);
        assert_eq!(read_record.record_type, RecordType::Conversation);
    }

    #[test]
    fn test_delete_record_creates_tombstone() {
        let dir = tempfile::tempdir().unwrap();
        let mut vault = create_test_vault(&dir);

        let record = Record::new(RecordType::Memory, "DESKTOP", "device-001");
        let content = b"Important memory";

        vault.write_record(record.clone(), content).unwrap();

        // Delete the record
        vault.delete_record(&record.record_id, "DESKTOP", "device-001").unwrap();

        // Reading the deleted record should fail
        let result = vault.read_record(&record.record_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_password_absent_from_files() {
        let dir = tempfile::tempdir().unwrap();
        let vault_root = dir.path().join("UNOONE");
        Vault::create(&vault_root, b"my-secret-password-12345").unwrap();

        // Walk all files in the vault and check that the password
        // does not appear in plaintext
        fn check_no_password(dir: &Path, password: &[u8]) -> bool {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if !check_no_password(&path, password) {
                            return false;
                        }
                    } else if let Ok(content) = std::fs::read_to_string(&path) {
                        let password_str = String::from_utf8_lossy(password);
                        if content.contains(&*password_str) {
                            return false;
                        }
                    }
                }
            }
            true
        }

        assert!(check_no_password(&vault_root, b"my-secret-password-12345"),
            "Password found in plaintext in vault files!");
    }

    #[test]
    fn test_master_key_absent_from_files() {
        let dir = tempfile::tempdir().unwrap();
        let vault_root = dir.path().join("UNOONE");
        Vault::create(&vault_root, b"test-password-12345").unwrap();

        let mut vault = Vault::open(&vault_root).unwrap();
        vault.unlock(b"test-password-12345").unwrap();

        // The master key should not appear in any file
        let master_key = vault.master_key().unwrap();
        let key_hex = hex::encode(master_key);

        fn check_no_key(dir: &Path, key_hex: &str) -> bool {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if !check_no_key(&path, key_hex) {
                            return false;
                        }
                    } else if let Ok(content) = std::fs::read_to_string(&path) {
                        if content.contains(key_hex) {
                            return false;
                        }
                    }
                }
            }
            true
        }

        assert!(check_no_key(&vault_root, &key_hex),
            "Master key hex found in plaintext in vault files!");
    }
}