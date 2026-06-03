use aes_gcm::aead::{Aead, OsRng, rand_core::RngCore};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use anyhow::{anyhow, Result};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
pub struct PersistedWalletStatus {
    pub exists: bool,
    pub path: String,
    pub status: String,
    pub warning: String,
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
        version: "carlosk-wallet-backup-v0.90".to_string(),
        kdf: "argon2-default".to_string(),
        cipher: "aes-256-gcm".to_string(),
        salt_b64: B64.encode(salt),
        nonce_b64: B64.encode(nonce),
        ciphertext_b64: B64.encode(ciphertext),
        warning: "Keep this encrypted backup and passphrase separate. Anyone with both can restore the wallet.".to_string(),
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

pub fn save_encrypted_wallet_to_disk(backup: &EncryptedBackup) -> Result<PersistedWalletStatus> {
    let path = wallet_store_path()?;
    let parent = path.parent().ok_or_else(|| anyhow!("invalid wallet storage path"))?;
    fs::create_dir_all(parent)?;
    let json = serde_json::to_string_pretty(backup)?;
    fs::write(&path, json)?;
    persisted_status_with("Encrypted wallet saved locally. You will need the passphrase to unlock it after restart.")
}

pub fn load_encrypted_wallet_from_disk() -> Result<EncryptedBackup> {
    let path = wallet_store_path()?;
    let json = fs::read_to_string(&path)
        .map_err(|e| anyhow!("No saved wallet found at {}: {e}", path.display()))?;
    serde_json::from_str(&json).map_err(|e| anyhow!("Saved wallet file is not valid CarlosK Wallet backup JSON: {e}"))
}

pub fn delete_encrypted_wallet_from_disk() -> Result<PersistedWalletStatus> {
    let path = wallet_store_path()?;
    if path.exists() {
        fs::remove_file(&path)?;
    }
    persisted_status_with("Saved encrypted wallet file removed from local disk.")
}

pub fn persisted_wallet_status() -> Result<PersistedWalletStatus> {
    persisted_status_with("Local encrypted wallet storage status loaded.")
}

fn persisted_status_with(status: &str) -> Result<PersistedWalletStatus> {
    let path = wallet_store_path()?;
    Ok(PersistedWalletStatus {
        exists: path.exists(),
        path: path.display().to_string(),
        status: status.to_string(),
        warning: "This file is encrypted, but the passphrase protects it. Losing the passphrase means the app cannot decrypt the saved wallet.".to_string(),
    })
}

fn wallet_store_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| anyhow!("Could not find HOME/USERPROFILE for local wallet storage"))?;
    Ok(PathBuf::from(home).join(".carlosk-wallet").join("wallet.encrypted.json"))
}
