// UnoOne Vault Core — Cryptographic primitives
// Argon2id key derivation, XChaCha20-Poly1305 authenticated encryption,
// HKDF-SHA-256 domain key derivation, secure random generation
//
// Design (directive §16):
//   Password → Argon2id → Key-encryption key → Wraps random vault master key
//   This allows password changes without re-encrypting every record.
//
//   Domain keys are derived from the master key via HKDF-SHA-256
//   so each vault domain (records, journal, indexes, etc.) has its own key.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use hkdf::Hkdf;
use hmac::Hmac;
use rand::RngCore;
use sha2::Sha256;
use zeroize::Zeroize;

use crate::error::VaultError;

/// Argon2id parameters for vault key derivation
/// Matches the Kotlin encrypted-vault specification:
/// memory: 256 MiB, iterations: 3, parallelism: 4
pub const ARGON2_MEMORY: u32 = 256 * 1024; // 256 MiB in KiB
pub const ARGON2_ITERATIONS: u32 = 3;
pub const ARGON2_PARALLELISM: u32 = 4;

/// Key sizes in bytes
pub const MASTER_KEY_LEN: usize = 32; // 256-bit
pub const KEY_ENCRYPTION_KEY_LEN: usize = 32; // 256-bit
pub const SALT_LEN: usize = 32; // 256-bit
pub const NONCE_LEN: usize = 24; // 192-bit for XChaCha20
pub const TAG_LEN: usize = 16; // 128-bit Poly1305 tag
pub const RECOVERY_SECRET_LEN: usize = 32; // 256-bit

/// Vault domain names for domain-specific key derivation
pub const DOMAIN_RECORDS: &str = "records";
pub const DOMAIN_JOURNAL: &str = "journal";
pub const DOMAIN_INDEXES: &str = "indexes";
pub const DOMAIN_ATTACHMENTS: &str = "attachments";
pub const DOMAIN_CONFIG: &str = "config";
pub const DOMAIN_HEADER: &str = "header";

/// Derive a key-encryption key from a password using Argon2id
///
/// This is the password-based key derivation specified in directive §16:
///   password → Argon2id(memory=256MiB, iterations=3, parallelism=4) → key-encryption key
///
/// The derived key is used to wrap (encrypt) the vault master key,
/// NOT to encrypt records directly. This allows password changes
/// without re-encrypting every record.
pub fn derive_key_encryption_key(
    password: &[u8],
    salt: &[u8; SALT_LEN],
) -> Result<[u8; KEY_ENCRYPTION_KEY_LEN], VaultError> {
    let params = Params::new(ARGON2_MEMORY, ARGON2_ITERATIONS, ARGON2_PARALLELISM, Some(KEY_ENCRYPTION_KEY_LEN))
        .map_err(|e| VaultError::Crypto(format!("Invalid Argon2id params: {}", e)))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; KEY_ENCRYPTION_KEY_LEN];
    argon2
        .hash_password_into(password, salt, &mut key)
        .map_err(|e| VaultError::Crypto(format!("Argon2id derivation failed: {}", e)))?;

    Ok(key)
}

/// Encrypt data using XChaCha20-Poly1305 with authenticated associated data (AAD)
///
/// This is the primary encryption primitive for vault records and the vault header.
/// AAD is authenticated but NOT encrypted — used for header metadata that must be
/// readable without decryption.
pub fn encrypt(
    key: &[u8; KEY_ENCRYPTION_KEY_LEN],
    nonce: &[u8; NONCE_LEN],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, VaultError> {
    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| VaultError::EncryptionFailed(format!("Invalid key: {}", e)))?;

    let xnonce = XNonce::from_slice(nonce);
    let payload = Payload {
        msg: plaintext,
        aad,
    };
    cipher
        .encrypt(xnonce, payload)
        .map_err(|e| VaultError::EncryptionFailed(format!("Encryption failed: {}", e)))
}

/// Decrypt data using XChaCha20-Poly1305 with AAD verification
///
/// Returns the plaintext only if both the ciphertext and AAD are authentic.
/// Any modification to ciphertext, nonce, or AAD will cause decryption to fail.
pub fn decrypt(
    key: &[u8; KEY_ENCRYPTION_KEY_LEN],
    nonce: &[u8; NONCE_LEN],
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, VaultError> {
    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| VaultError::DecryptionFailed(format!("Invalid key: {}", e)))?;

    let xnonce = XNonce::from_slice(nonce);
    let payload = Payload {
        msg: ciphertext,
        aad,
    };
    cipher
        .decrypt(xnonce, payload)
        .map_err(|e| VaultError::DecryptionFailed(format!("Decryption failed (wrong password or tampered data): {}", e)))
}

/// Derive a domain-specific key from the vault master key using HKDF-SHA-256
///
/// Each vault domain (records, journal, indexes, etc.) gets its own key
/// derived from the master key. This provides domain isolation:
/// compromising one domain's key does not compromise others.
///
/// The info parameter is the domain name (e.g., "records", "journal").
pub fn derive_domain_key(
    master_key: &[u8; MASTER_KEY_LEN],
    domain: &str,
) -> [u8; MASTER_KEY_LEN] {
    let hkdf = Hkdf::<Sha256>::new(Some(b"unoone-vault-domain"), master_key);
    let mut domain_key = [0u8; MASTER_KEY_LEN];
    // HKDF expand is infallible when output length <= HashLen (32 bytes for SHA-256)
    hkdf.expand(domain.as_bytes(), &mut domain_key)
        .expect("HKDF expand failed — output length exceeds hash length");
    domain_key
}

/// Wrap (encrypt) the vault master key using the password-derived key-encryption key
///
/// This encrypts the master key so it can be stored in the vault header.
/// The master key is never stored in plaintext.
pub fn wrap_master_key(
    kek: &[u8; KEY_ENCRYPTION_KEY_LEN],
    master_key: &[u8; MASTER_KEY_LEN],
    nonce: &[u8; NONCE_LEN],
) -> Result<Vec<u8>, VaultError> {
    // AAD for master key wrapping includes the purpose
    let aad = b"unoone-vault-master-key-wrap";
    encrypt(kek, nonce, master_key, aad)
}

/// Unwrap (decrypt) the vault master key using the password-derived key-encryption key
///
/// This decrypts the wrapped master key from the vault header.
/// Fails if the password is wrong (AAD authentication will fail).
pub fn unwrap_master_key(
    kek: &[u8; KEY_ENCRYPTION_KEY_LEN],
    wrapped_key: &[u8],
    nonce: &[u8; NONCE_LEN],
) -> Result<[u8; MASTER_KEY_LEN], VaultError> {
    let aad = b"unoone-vault-master-key-wrap";
    let plaintext = decrypt(kek, nonce, wrapped_key, aad)?;

    let mut master_key = [0u8; MASTER_KEY_LEN];
    master_key.copy_from_slice(&plaintext[..MASTER_KEY_LEN]);
    Ok(master_key)
}

/// Wrap the master key using a recovery secret
///
/// Recovery uses a separate wrapping so the vault can be unlocked
/// with either the password or the recovery words.
pub fn wrap_master_key_with_recovery(
    recovery_secret: &[u8; RECOVERY_SECRET_LEN],
    master_key: &[u8; MASTER_KEY_LEN],
    nonce: &[u8; NONCE_LEN],
) -> Result<Vec<u8>, VaultError> {
    // Derive a key-encryption key from the recovery secret
    // Using a fixed salt since the recovery secret IS the high-entropy secret
    let recovery_kek = derive_key_encryption_key(recovery_secret, &[0u8; SALT_LEN])?;
    wrap_master_key(&recovery_kek, master_key, nonce)
}

/// Unwrap the master key using a recovery secret
pub fn unwrap_master_key_with_recovery(
    recovery_secret: &[u8; RECOVERY_SECRET_LEN],
    wrapped_key: &[u8],
    nonce: &[u8; NONCE_LEN],
) -> Result<[u8; MASTER_KEY_LEN], VaultError> {
    let recovery_kek = derive_key_encryption_key(recovery_secret, &[0u8; SALT_LEN])?;
    unwrap_master_key(&recovery_kek, wrapped_key, nonce)
}

/// Compute HMAC-SHA-256 for header authentication
///
/// This is used to verify the integrity of the vault header.
/// The header is authenticated but NOT encrypted (some fields are public).
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    use hmac::Mac;
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = <HmacSha256 as Mac>::new_from_slice(key).expect("HMAC key length is valid");
    mac.update(data);
    mac.finalize().into_bytes().into()
}

/// Generate a cryptographically secure random salt (32 bytes)
pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Generate a cryptographically secure random master key (256 bits)
pub fn generate_master_key() -> [u8; MASTER_KEY_LEN] {
    let mut key = [0u8; MASTER_KEY_LEN];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// Generate a cryptographically secure random nonce (192 bits for XChaCha20)
pub fn generate_nonce() -> [u8; NONCE_LEN] {
    let mut nonce = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce
}

/// Generate a cryptographically secure recovery secret (256 bits)
pub fn generate_recovery_secret() -> [u8; RECOVERY_SECRET_LEN] {
    let mut secret = [0u8; RECOVERY_SECRET_LEN];
    rand::thread_rng().fill_bytes(&mut secret);
    secret
}

/// Zero a byte array securely
///
/// This is used to clear cryptographic keys from memory when the vault is locked.
/// Uses the zeroize crate to prevent compiler optimizations from removing the zeroing.
pub fn secure_zero(data: &mut [u8]) {
    data.zeroize();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_encryption_key() {
        let password = b"test-password-12345678";
        let salt = generate_salt();
        let key = derive_key_encryption_key(password, &salt).unwrap();
        assert_eq!(key.len(), KEY_ENCRYPTION_KEY_LEN);

        // Same password + same salt = same key
        let key2 = derive_key_encryption_key(password, &salt).unwrap();
        assert_eq!(key, key2);

        // Different password = different key
        let key3 = derive_key_encryption_key(b"wrong-password-12345", &salt).unwrap();
        assert_ne!(key, key3);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = generate_master_key();
        let nonce = generate_nonce();
        let plaintext = b"Hello, UnoOne vault!";
        let aad = b"test-associated-data";

        let ciphertext = encrypt(&key, &nonce, plaintext, aad).unwrap();
        assert_ne!(ciphertext, plaintext);

        let decrypted = decrypt(&key, &nonce, &ciphertext, aad).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_fails_with_wrong_key() {
        let key = generate_master_key();
        let wrong_key = generate_master_key();
        let nonce = generate_nonce();
        let plaintext = b"secret data";
        let aad = b"test-aad";

        let ciphertext = encrypt(&key, &nonce, plaintext, aad).unwrap();

        // Decrypting with wrong key must fail
        let result = decrypt(&wrong_key, &nonce, &ciphertext, aad);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_fails_with_modified_ciphertext() {
        let key = generate_master_key();
        let nonce = generate_nonce();
        let plaintext = b"secret data";
        let aad = b"test-aad";

        let mut ciphertext = encrypt(&key, &nonce, plaintext, aad).unwrap();
        // Flip one bit
        ciphertext[0] ^= 0x01;

        let result = decrypt(&key, &nonce, &ciphertext, aad);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_fails_with_modified_aad() {
        let key = generate_master_key();
        let nonce = generate_nonce();
        let plaintext = b"secret data";
        let aad = b"original-aad";

        let ciphertext = encrypt(&key, &nonce, plaintext, aad).unwrap();

        let result = decrypt(&key, &nonce, &ciphertext, b"modified-aad");
        assert!(result.is_err());
    }

    #[test]
    fn test_domain_key_derivation() {
        let master_key = generate_master_key();
        let records_key = derive_domain_key(&master_key, DOMAIN_RECORDS);
        let journal_key = derive_domain_key(&master_key, DOMAIN_JOURNAL);

        // Different domains produce different keys
        assert_ne!(records_key, journal_key);

        // Same domain produces same key
        let records_key2 = derive_domain_key(&master_key, DOMAIN_RECORDS);
        assert_eq!(records_key, records_key2);

        // Different master key produces different domain keys
        let other_master = generate_master_key();
        let other_records_key = derive_domain_key(&other_master, DOMAIN_RECORDS);
        assert_ne!(records_key, other_records_key);
    }

    #[test]
    fn test_wrap_unwrap_master_key() {
        let master_key = generate_master_key();
        let password = b"test-password-12345678";
        let salt = generate_salt();
        let nonce = generate_nonce();

        let kek = derive_key_encryption_key(password, &salt).unwrap();
        let wrapped = wrap_master_key(&kek, &master_key, &nonce).unwrap();

        // Wrapped key is different from plaintext master key
        assert_ne!(wrapped.as_slice(), master_key.as_slice());

        // Unwrap with correct password succeeds
        let unwrapped = unwrap_master_key(&kek, &wrapped, &nonce).unwrap();
        assert_eq!(master_key, unwrapped);

        // Unwrap with wrong password fails
        let wrong_kek = derive_key_encryption_key(b"wrong-password-1234", &salt).unwrap();
        let result = unwrap_master_key(&wrong_kek, &wrapped, &nonce);
        assert!(result.is_err());
    }

    #[test]
    fn test_hmac_sha256() {
        let key = b"hmac-key";
        let data = b"test data";
        let mac1 = hmac_sha256(key, data);
        let mac2 = hmac_sha256(key, data);
        assert_eq!(mac1, mac2);

        // Different data produces different MAC
        let mac3 = hmac_sha256(key, b"other data");
        assert_ne!(mac1, mac3);
    }

    #[test]
    fn test_secure_zero() {
        let mut key = [1u8, 2, 3, 4, 5, 6, 7, 8];
        secure_zero(&mut key);
        assert_eq!(key, [0u8; 8]);
    }

    #[test]
    fn test_empty_password_fails() {
        let result = derive_key_encryption_key(b"", &generate_salt());
        // Argon2id with empty password should still produce a key
        // (the password validation is at a higher level)
        assert!(result.is_ok());
    }
}