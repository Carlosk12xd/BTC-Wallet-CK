use anyhow::{anyhow, Result};
use bdk_bip322::{BIP322, MessageProof, SignatureFormat};
use bdk_wallet::{KeychainKind, Wallet};
use bip39::{Language, Mnemonic};
use bitcoin::bip32::Xpriv;
use bitcoin::{Address, Network};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct MiningWallet {
    mnemonic: Mnemonic,
    wallet: Wallet,
    label: String,
    next_external_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBackupPayload {
    pub app: String,
    pub backup_version: String,
    pub label: String,
    pub mnemonic: String,
    pub address: String,
    pub derivation: String,
    pub network: String,
    #[serde(default = "default_next_external_index")]
    pub next_external_index: u32,
    pub created_at_unix: u64,
    pub warning: String,
}

#[derive(Debug, Serialize)]
pub struct WalletInfo {
    pub mnemonic: String,
    pub address: String,
    pub network: String,
    pub derivation: String,
    pub warning: String,
}

#[derive(Debug, Serialize)]
pub struct ReceiveAddressInfo {
    pub address: String,
    pub index: u32,
    pub network: String,
    pub warning: String,
}

#[derive(Debug, Deserialize)]
pub struct SendDraftInput {
    pub to_address: String,
    pub amount_sats: u64,
    pub fee_rate_sat_vb: f32,
}

#[derive(Debug, Serialize)]
pub struct SendDraft {
    pub to_address: String,
    pub amount_sats: u64,
    pub fee_rate_sat_vb: f32,
    pub ready_to_broadcast: bool,
    pub status: String,
    pub next_steps: Vec<String>,
    pub warning: String,
}

#[derive(Debug, Serialize)]
pub struct SignatureResponse {
    pub signature: String,
    pub address: String,
    pub format: String,
}

impl MiningWallet {
    pub fn new_random(label: &str) -> Result<Self> {
        let mnemonic = Mnemonic::generate_in(Language::English, 12)?;
        Self::from_mnemonic_obj(mnemonic, label, 1)
    }

    pub fn from_mnemonic(words: &str, label: &str) -> Result<Self> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, words.trim())?;
        Self::from_mnemonic_obj(mnemonic, label, 1)
    }

    fn from_mnemonic_obj(mnemonic: Mnemonic, label: &str, next_external_index: u32) -> Result<Self> {
        let seed = mnemonic.to_seed_normalized("");
        let xprv = Xpriv::new_master(Network::Bitcoin, &seed)?;

        // Native SegWit BIP84 account 0.
        // External path: m/84'/0'/0'/0/*
        // Internal path: m/84'/0'/0'/1/*
        let descriptor = format!("wpkh({}/84'/0'/0'/0/*)", xprv);
        let change_descriptor = format!("wpkh({}/84'/0'/0'/1/*)", xprv);

        let wallet = Wallet::create(descriptor, change_descriptor)
            .network(Network::Bitcoin)
            .create_wallet_no_persist()?;

        Ok(Self {
            mnemonic,
            wallet,
            label: sanitize_label(label),
            next_external_index,
        })
    }

    pub fn info(&self) -> Result<WalletInfo> {
        let address = self.wallet.peek_address(KeychainKind::External, 0).address.to_string();
        Ok(WalletInfo {
            mnemonic: self.mnemonic.to_string(),
            address,
            network: "bitcoin-mainnet".to_string(),
            derivation: "m/84'/0'/0'/0/0".to_string(),
            warning: "Development wallet. v0.27 is a simplified core-wallet UI. Do not store meaningful funds until persistence, sync, transaction broadcast, and Lightning recovery are fully tested.".to_string(),
        })
    }

    pub fn next_receive_address(&mut self) -> Result<ReceiveAddressInfo> {
        let index = self.next_external_index;
        let address = self.wallet.peek_address(KeychainKind::External, index).address.to_string();
        self.next_external_index = self.next_external_index.saturating_add(1);
        Ok(ReceiveAddressInfo {
            address,
            index,
            network: "bitcoin-mainnet".to_string(),
            warning: "Receive address generated locally. The app does not yet sync the chain or confirm received funds.".to_string(),
        })
    }

    pub fn backup_payload(&self) -> Result<WalletBackupPayload> {
        let address = self.wallet.peek_address(KeychainKind::External, 0).address.to_string();
        let created_at_unix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        Ok(WalletBackupPayload {
            app: "CarlosK Wallet".to_string(),
            backup_version: "0.27".to_string(),
            label: self.label.clone(),
            mnemonic: self.mnemonic.to_string(),
            address,
            derivation: "m/84'/0'/0'/0/0".to_string(),
            network: "bitcoin-mainnet".to_string(),
            next_external_index: self.next_external_index,
            created_at_unix,
            warning: "This backup contains the seed phrase after decryption. Keep it offline and private.".to_string(),
        })
    }

    pub fn sign_bip322_simple(&mut self, message: &str, address: &str) -> Result<SignatureResponse> {
        let checked_address = Address::from_str(address)?
            .require_network(Network::Bitcoin)
            .map_err(|e| anyhow!("address network mismatch: {e}"))?;

        let proof = self.wallet.sign_message(
            message,
            SignatureFormat::Simple,
            &checked_address,
            None,
        )?;

        let signature = match proof {
            MessageProof::Signed(sig) => sig,
            other => other.to_base64(),
        };

        Ok(SignatureResponse {
            signature,
            address: checked_address.to_string(),
            format: "BIP-322 Simple".to_string(),
        })
    }
}

pub fn create_send_draft(input: SendDraftInput) -> Result<SendDraft> {
    let checked_address = Address::from_str(input.to_address.trim())?
        .require_network(Network::Bitcoin)
        .map_err(|e| anyhow!("recipient address network mismatch: {e}"))?;

    if input.amount_sats == 0 {
        return Err(anyhow!("Amount must be greater than 0 sats"));
    }
    if input.fee_rate_sat_vb <= 0.0 {
        return Err(anyhow!("Fee rate must be greater than 0 sat/vB"));
    }

    Ok(SendDraft {
        to_address: checked_address.to_string(),
        amount_sats: input.amount_sats,
        fee_rate_sat_vb: input.fee_rate_sat_vb,
        ready_to_broadcast: false,
        status: "Send draft validated, but transaction creation/broadcast is not enabled in v0.27.".to_string(),
        next_steps: vec![
            "Add chain sync against Esplora or Electrum.".to_string(),
            "List wallet UTXOs and confirmed balance.".to_string(),
            "Build a PSBT with BDK.".to_string(),
            "Sign the PSBT locally.".to_string(),
            "Broadcast only after fee and recipient confirmation screens are implemented.".to_string(),
        ],
        warning: "Do not use this to send real BTC yet. v0.27 validates the send form but does not broadcast transactions.".to_string(),
    })
}

fn sanitize_label(label: &str) -> String {
    let sanitized: String = label
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    if sanitized.is_empty() {
        "CarlosKWallet".to_string()
    } else {
        sanitized
    }
}

fn default_next_external_index() -> u32 { 1 }
