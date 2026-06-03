use anyhow::{anyhow, Result};
use bdk_bip322::{BIP322, MessageProof, SignatureFormat};
use bdk_esplora::{esplora_client, EsploraExt};
use bdk_wallet::{psbt::PsbtUtils, KeychainKind, SignOptions, Wallet};
use bip39::{Language, Mnemonic};
use bitcoin::bip32::Xpriv;
use bitcoin::{consensus::encode::serialize_hex, Address, Amount, FeeRate, Network};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct MiningWallet {
    mnemonic: Mnemonic,
    wallet: Wallet,
    label: String,
    network: Network,
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
    pub label: String,
    pub mnemonic: String,
    pub address: String,
    pub network: String,
    pub derivation: String,
    pub next_external_index: u32,
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
pub struct BackendSyncReport {
    pub network: String,
    pub esplora_url: String,
    pub total_sats: u64,
    pub confirmed_sats: u64,
    pub pending_sats: u64,
    pub utxo_count: usize,
    pub utxo_sats: u64,
    pub status: String,
    pub warning: String,
}

#[derive(Debug, Serialize)]
pub struct SignedTransactionResult {
    pub txid: String,
    pub tx_hex: String,
    pub recipient: String,
    pub amount_sats: u64,
    pub fee_sats: u64,
    pub fee_rate_sat_vb: f32,
    pub finalized: bool,
    pub ready_to_broadcast: bool,
    pub status: String,
    pub warning: String,
}

#[derive(Debug, Serialize)]
pub struct SignatureResponse {
    pub signature: String,
    pub address: String,
    pub format: String,
}

#[derive(Debug, Serialize)]
pub struct BackupVerifyResult {
    pub backup_address: String,
    pub current_address: String,
    pub same_address: bool,
    pub network: String,
    pub next_external_index: u32,
    pub status: String,
}

impl MiningWallet {
    pub fn new_random(label: &str, network_choice: &str) -> Result<Self> {
        let mnemonic = Mnemonic::generate_in(Language::English, 12)?;
        let network = parse_network(network_choice)?;
        Self::from_mnemonic_obj(mnemonic, label, network, 1)
    }

    pub fn from_mnemonic(words: &str, label: &str, network_choice: &str) -> Result<Self> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, words.trim())?;
        let network = parse_network(network_choice)?;
        Self::from_mnemonic_obj(mnemonic, label, network, 1)
    }

    pub fn from_backup_payload(payload: &WalletBackupPayload, label_override: &str) -> Result<Self> {
        let label = if label_override.trim().is_empty() {
            payload.label.as_str()
        } else {
            label_override
        };
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, payload.mnemonic.trim())?;
        let network = parse_network(&payload.network)?;
        Self::from_mnemonic_obj(mnemonic, label, network, payload.next_external_index.max(1))
    }

    fn from_mnemonic_obj(mnemonic: Mnemonic, label: &str, network: Network, next_external_index: u32) -> Result<Self> {
        let seed = mnemonic.to_seed_normalized("");
        let xprv = Xpriv::new_master(network, &seed)?;
        let coin_type = if network == Network::Bitcoin { 0 } else { 1 };

        // Native SegWit BIP84 account 0.
        // Mainnet external path: m/84'/0'/0'/0/*
        // Testnet/signet external path: m/84'/1'/0'/0/*
        let descriptor = format!("wpkh({}/84'/{}'/0'/0/*)", xprv, coin_type);
        let change_descriptor = format!("wpkh({}/84'/{}'/0'/1/*)", xprv, coin_type);

        let wallet = Wallet::create(descriptor, change_descriptor)
            .network(network)
            .create_wallet_no_persist()?;

        Ok(Self {
            mnemonic,
            wallet,
            label: sanitize_label(label),
            network,
            next_external_index,
        })
    }

    pub fn info(&self) -> Result<WalletInfo> {
        let address = self.wallet.peek_address(KeychainKind::External, 0).address.to_string();
        Ok(WalletInfo {
            label: self.label.clone(),
            mnemonic: self.mnemonic.to_string(),
            address,
            network: network_id(self.network).to_string(),
            derivation: format!("m/84'/{}'/0'/0/0", if self.network == Network::Bitcoin { 0 } else { 1 }),
            next_external_index: self.next_external_index,
            warning: "Development wallet. v0.90 adds backend Esplora sync and wallet-built signed transaction creation. Test on signet/testnet first before using meaningful mainnet funds.".to_string(),
        })
    }

    pub fn next_receive_address(&mut self) -> Result<ReceiveAddressInfo> {
        let index = self.next_external_index;
        let address = self.wallet.peek_address(KeychainKind::External, index).address.to_string();
        self.next_external_index = self.next_external_index.saturating_add(1);
        Ok(ReceiveAddressInfo {
            address,
            index,
            network: network_id(self.network).to_string(),
            warning: "Receive address generated locally. v0.90 can sync the wallet with public Esplora APIs from the Send/Receive tab. Save encrypted wallet storage to persist the address index after restart.".to_string(),
        })
    }

    pub fn backup_payload(&self) -> Result<WalletBackupPayload> {
        let address = self.wallet.peek_address(KeychainKind::External, 0).address.to_string();
        let created_at_unix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        Ok(WalletBackupPayload {
            app: "CarlosK Wallet".to_string(),
            backup_version: "0.90".to_string(),
            label: self.label.clone(),
            mnemonic: self.mnemonic.to_string(),
            address,
            derivation: format!("m/84'/{}'/0'/0/0", if self.network == Network::Bitcoin { 0 } else { 1 }),
            network: network_id(self.network).to_string(),
            next_external_index: self.next_external_index,
            created_at_unix,
            warning: "This backup contains the seed phrase after decryption. Keep it offline and private.".to_string(),
        })
    }

    pub fn verify_backup_payload(&self, payload: &WalletBackupPayload) -> Result<BackupVerifyResult> {
        let backup_wallet = MiningWallet::from_backup_payload(payload, &payload.label)?;
        let backup_address = backup_wallet.info()?.address;
        let current = self.info()?;
        let same_address = backup_address == current.address && payload.network == current.network;
        Ok(BackupVerifyResult {
            backup_address,
            current_address: current.address,
            same_address,
            network: current.network,
            next_external_index: payload.next_external_index,
            status: if same_address {
                "Backup restore proof passed. The encrypted backup recreates the same wallet address.".to_string()
            } else {
                "Backup restore proof failed. The backup does not recreate the same current wallet address/network.".to_string()
            },
        })
    }

    pub fn create_send_draft(&self, input: SendDraftInput) -> Result<SendDraft> {
        let checked_address = Address::from_str(input.to_address.trim())?
            .require_network(self.network)
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
            status: "Send draft validated against the current wallet network. This validates the recipient network/amount. Use Build & Sign Transaction after backend sync to create a real signed transaction.".to_string(),
            next_steps: vec![
                "Add Esplora/Electrum chain sync.".to_string(),
                "Detect confirmed balance and spendable UTXOs.".to_string(),
                "Build a PSBT with BDK.".to_string(),
                "Sign the PSBT locally.".to_string(),
                "Broadcast only after fee and recipient confirmation screens are implemented.".to_string(),
            ],
            warning: "Do not use this to send real BTC yet. v0.90 can build and sign a real wallet transaction after backend sync. Review the signed transaction before broadcasting.".to_string(),
        })
    }

    pub fn sync_with_esplora(&mut self, esplora_url_override: Option<String>) -> Result<BackendSyncReport> {
        let url = esplora_url_override
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| default_esplora_url(self.network).to_string());

        let client = esplora_client::Builder::new(&url).build_blocking();
        let request = self.wallet.start_full_scan();
        let update = client.full_scan(request, 20, 6)?;
        self.wallet.apply_update(update)?;

        let balance = self.wallet.balance();
        let total_sats = balance.total().to_sat();
        let confirmed_sats = balance.confirmed.to_sat();
        let pending_sats = balance.trusted_pending.to_sat().saturating_add(balance.untrusted_pending.to_sat());
        let utxos: Vec<_> = self.wallet.list_unspent().collect();
        let utxo_sats = utxos.iter().map(|utxo| utxo.txout.value.to_sat()).sum();

        Ok(BackendSyncReport {
            network: network_id(self.network).to_string(),
            esplora_url: url,
            total_sats,
            confirmed_sats,
            pending_sats,
            utxo_count: utxos.len(),
            utxo_sats,
            status: format!("Backend wallet sync complete. Total wallet balance: {total_sats} sats."),
            warning: "Sync uses a public Esplora endpoint. For stronger privacy, configure your own Esplora/Electrum backend before serious mainnet use.".to_string(),
        })
    }

    pub fn create_signed_transaction(&mut self, input: SendDraftInput) -> Result<SignedTransactionResult> {
        let checked_address = Address::from_str(input.to_address.trim())?
            .require_network(self.network)
            .map_err(|e| anyhow!("recipient address network mismatch: {e}"))?;

        if input.amount_sats == 0 {
            return Err(anyhow!("Amount must be greater than 0 sats"));
        }
        if input.fee_rate_sat_vb <= 0.0 {
            return Err(anyhow!("Fee rate must be greater than 0 sat/vB"));
        }

        let available = self.wallet.balance().total().to_sat();
        if available == 0 {
            return Err(anyhow!("Wallet balance is zero. Run backend sync after funding this wallet."));
        }
        if input.amount_sats >= available {
            return Err(anyhow!("Amount must be lower than total wallet balance so there is room for fees. Available: {available} sats"));
        }

        let fee_rate = FeeRate::from_sat_per_vb(input.fee_rate_sat_vb.ceil().max(1.0) as u64)
            .ok_or_else(|| anyhow!("Invalid fee rate"))?;

        let mut builder = self.wallet.build_tx();
        builder.add_recipient(checked_address.script_pubkey(), Amount::from_sat(input.amount_sats));
        builder.fee_rate(fee_rate);

        let mut psbt = builder.finish()?;
        let fee_sats = psbt.fee_amount().map(|amount| amount.to_sat()).unwrap_or(0);
        let finalized = self.wallet.sign(&mut psbt, SignOptions::default())?;
        if !finalized {
            return Err(anyhow!("Wallet signed the PSBT, but it was not finalized. Do not broadcast."));
        }

        let tx = psbt.extract_tx()?;
        let txid = tx.compute_txid().to_string();
        let tx_hex = serialize_hex(&tx);

        Ok(SignedTransactionResult {
            txid,
            tx_hex,
            recipient: checked_address.to_string(),
            amount_sats: input.amount_sats,
            fee_sats,
            fee_rate_sat_vb: input.fee_rate_sat_vb,
            finalized,
            ready_to_broadcast: true,
            status: "Real signed transaction created locally. Review the recipient, amount, fee, and network before broadcasting.".to_string(),
            warning: "This is a real signed transaction. Broadcasting it spends wallet funds. Test on signet/testnet first.".to_string(),
        })
    }

    pub fn sign_bip322_simple(&mut self, message: &str, address: &str) -> Result<SignatureResponse> {
        let checked_address = Address::from_str(address)?
            .require_network(self.network)
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

pub fn parse_network(choice: &str) -> Result<Network> {
    match choice.trim().to_lowercase().as_str() {
        "bitcoin" | "mainnet" | "bitcoin-mainnet" => Ok(Network::Bitcoin),
        "testnet" | "bitcoin-testnet" => Ok(Network::Testnet),
        "signet" | "bitcoin-signet" => Ok(Network::Signet),
        other => Err(anyhow!("Unsupported Bitcoin network: {other}. Use bitcoin, testnet, or signet.")),
    }
}

pub fn network_id(network: Network) -> &'static str {
    match network {
        Network::Bitcoin => "bitcoin",
        Network::Testnet => "testnet",
        Network::Signet => "signet",
        Network::Regtest => "regtest",
        _ => "unknown",
    }
}

pub fn default_esplora_url(network: Network) -> &'static str {
    match network {
        Network::Bitcoin => "https://mempool.space/api",
        Network::Testnet => "https://mempool.space/testnet/api",
        Network::Signet => "https://mempool.space/signet/api",
        Network::Regtest => "http://127.0.0.1:3002",
        _ => "https://mempool.space/api",
    }
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
