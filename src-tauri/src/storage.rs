use aes_gcm::aead::{Aead, OsRng, rand_core::RngCore};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use anyhow::{anyhow, Result};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::{Deserialize, Serialize};

use crate::wallet::WalletBackupPayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBackup {
    pub version: String,
    pub kdf: String,
    pub cipher: String,
    pub salt_b64: String,
    pub nonce_b64: String,
    pub ciphertext_b64: String,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRoadmap {
    pub current: String,
    pub next: Vec<String>,
}

pub fn roadmap() -> StorageRoadmap {
    StorageRoadmap {
        current: "v0.6 can export and restore encrypted wallet backups using a passphrase. Non-secret UI data is persisted by the frontend in localStorage.".to_string(),
        next: vec![
            "Move wallet persistence into OS keychain / secure storage.".to_string(),
            "Add encrypted SQLite for miner profiles, payout settings, and history.".to_string(),
            "Require backup confirmation before enabling production mining workflows.".to_string(),
            "Move restore-from-encrypted-backup into a polished production recovery wizard.".to_string(),
        ],
    }
}

pub fn encrypt_wallet_backup(payload: &WalletBackupPayload, passphrase: &str) -> Result<EncryptedBackup> {
    if passphrase.trim().len() < 12 {
        return Err(anyhow!("Use a backup passphrase with at least 12 characters"));
    }

    let plaintext = serde_json::to_string_pretty(payload)?;

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(passphrase.as_bytes(), &salt, &mut key)
        .map_err(|e| anyhow!("failed to derive encryption key: {e}"))?;

    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| anyhow!("invalid key length"))?;

    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);

    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes())
        .map_err(|_| anyhow!("failed to encrypt wallet backup"))?;

    Ok(EncryptedBackup {
        version: "carlosk-wallet-backup-v0.6".to_string(),
        kdf: "argon2-default".to_string(),
        cipher: "aes-256-gcm".to_string(),
        salt_b64: B64.encode(salt),
        nonce_b64: B64.encode(nonce),
        ciphertext_b64: B64.encode(ciphertext),
        warning: "Keep this encrypted backup and passphrase separate. Anyone with both can restore the mining wallet.".to_string(),
    })
}


pub fn decrypt_wallet_backup(backup: &EncryptedBackup, passphrase: &str) -> Result<WalletBackupPayload> {
    if passphrase.trim().len() < 12 {
        return Err(anyhow!("Use the same backup passphrase, at least 12 characters"));
    }

    if backup.cipher != "aes-256-gcm" {
        return Err(anyhow!("Unsupported backup cipher: {}", backup.cipher));
    }

    let salt = B64
        .decode(&backup.salt_b64)
        .map_err(|e| anyhow!("Invalid backup salt: {e}"))?;
    let nonce = B64
        .decode(&backup.nonce_b64)
        .map_err(|e| anyhow!("Invalid backup nonce: {e}"))?;
    let ciphertext = B64
        .decode(&backup.ciphertext_b64)
        .map_err(|e| anyhow!("Invalid backup ciphertext: {e}"))?;

    if salt.len() != 16 {
        return Err(anyhow!("Invalid backup salt length"));
    }
    if nonce.len() != 12 {
        return Err(anyhow!("Invalid backup nonce length"));
    }

    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(passphrase.as_bytes(), &salt, &mut key)
        .map_err(|e| anyhow!("failed to derive decryption key: {e}"))?;

    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| anyhow!("invalid key length"))?;
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| anyhow!("failed to decrypt backup. Check the passphrase and backup JSON."))?;

    let payload: WalletBackupPayload = serde_json::from_slice(&plaintext)
        .map_err(|e| anyhow!("backup decrypted but payload was invalid: {e}"))?;

    if payload.app != "CarlosK Wallet" {
        return Err(anyhow!("This backup was not created by CarlosK Wallet"));
    }

    Ok(payload)
}
