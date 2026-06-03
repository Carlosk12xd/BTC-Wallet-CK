use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bolt12OfferInfo {
    pub offer: String,
    pub source: String,
    pub status: String,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdkSpikeStatus {
    pub phase: String,
    pub implemented_now: Vec<String>,
    pub next_code_steps: Vec<String>,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InAppBolt12Status {
    pub feature_name: String,
    pub compiled_feature_flag: String,
    pub available_now: bool,
    pub current_mode: String,
    pub planned_dependencies: Vec<String>,
    pub next_steps: Vec<String>,
    pub warning: String,
}

pub fn validate_bolt12_offer(offer: &str) -> Result<Bolt12OfferInfo> {
    let trimmed = offer.trim();

    if trimmed.is_empty() {
        return Err(anyhow!("Paste a BOLT12 offer first"));
    }

    if !trimmed.to_lowercase().starts_with("lno1") {
        return Err(anyhow!("This does not look like a BOLT12 offer. OCEAN Lightning payout offers usually start with lno1..."));
    }

    if trimmed.len() < 80 {
        return Err(anyhow!("This BOLT12 offer looks too short. Copy the full lno1... offer from your Lightning wallet."));
    }

    Ok(Bolt12OfferInfo {
        offer: trimmed.to_string(),
        source: "External BOLT12 offer".to_string(),
        status: "Valid-looking BOLT12 offer saved in app memory".to_string(),
        warning: "v0.10 can validate/persist external BOLT12 offer text and includes a signet-only LDK Node preview. Embedded mainnet BOLT12 generation remains disabled until channel backup and liquidity handling are ready.".to_string(),
    })
}

pub fn create_placeholder_offer() -> Result<Bolt12OfferInfo> {
    Err(anyhow!(
        "In-app BOLT12 generation is not enabled in v0.10. The LDK Node integration is scaffolded behind the future ldk-node-experimental feature, but production Lightning receiving must wait for channel-state backup, inbound liquidity, and recovery testing."
    ))
}

pub fn in_app_bolt12_status() -> InAppBolt12Status {
    InAppBolt12Status {
        feature_name: "Embedded BOLT12 receive wallet".to_string(),
        compiled_feature_flag: "ldk-node-experimental".to_string(),
        available_now: false,
        current_mode: "external-offer-only".to_string(),
        planned_dependencies: vec![
            "ldk-node".to_string(),
            "lightning / lightning-invoice / lightning-types crates as required by selected LDK version".to_string(),
            "BDK/bdk_wallet for on-chain wallet coordination".to_string(),
            "SQLite + encrypted channel-state backups".to_string(),
        ],
        next_steps: vec![
            "Create a Lightning data directory separate from the Bitcoin mining identity wallet".to_string(),
            "Initialize LDK Node on testnet/signet before mainnet".to_string(),
            "Generate a BOLT12 offer and display it as lno1...".to_string(),
            "Document inbound-liquidity requirements in the UI".to_string(),
            "Test OCEAN payout configuration with a small miner and no meaningful funds at risk".to_string(),
        ],
        warning: "Do not receive meaningful Lightning funds in-app until channel backups and recovery are tested. Use external Lexe/Coinos/Core Lightning offers for live payouts during development.".to_string(),
    }
}

pub fn ldk_spike_status() -> LdkSpikeStatus {
    LdkSpikeStatus {
        phase: "v0.10 signet-first LDK/BOLT12 prototype scaffold".to_string(),
        implemented_now: vec![
            "External BOLT12 offer validation".to_string(),
            "OCEAN JSON message checking".to_string(),
            "BIP-322 signing workflow scaffold".to_string(),
            "Encrypted backup export and restore".to_string(),
            "Contributor-ready open-source project files".to_string(),
            "In-app BOLT12 status command that explains current limitations".to_string(),
            "Signet/testnet Lightning node preview is available in the UI".to_string(),
        ],
        next_code_steps: vec![
            "Add ldk-node dependency behind a Cargo feature in an isolated branch".to_string(),
            "Run first Lightning prototype on signet/testnet only".to_string(),
            "Create local Lightning node data directory".to_string(),
            "Generate a BOLT12 offer from the embedded node".to_string(),
            "Surface inbound-liquidity and online-status warnings".to_string(),
            "Add channel/backups plan before holding value".to_string(),
        ],
        blockers: vec![
            "Lightning receiving requires liquidity and an online wallet".to_string(),
            "Production app must protect channel state backups".to_string(),
            "BOLT12 support must be verified against the exact ldk-node version used".to_string(),
            "Mainnet funds should not be used until the restore path is tested".to_string(),
        ],
    }
}
