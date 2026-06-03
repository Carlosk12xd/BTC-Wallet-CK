use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningWalletInfo {
    pub alias: String,
    pub network: String,
    pub status: String,
    pub can_receive_bolt12_in_app: bool,
    pub warning: String,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bolt12OfferInfo {
    pub offer: String,
    pub source: String,
    pub status: String,
    pub warning: String,
}

pub fn create_lightning_wallet(alias: &str) -> LightningWalletInfo {
    let clean_alias = if alias.trim().is_empty() {
        "CarlosK Lightning".to_string()
    } else {
        alias.trim().to_string()
    };

    LightningWalletInfo {
        alias: clean_alias,
        network: "signet/testnet planned first; mainnet locked".to_string(),
        status: "Lightning wallet profile created. Embedded BOLT12 receiving is not enabled yet.".to_string(),
        can_receive_bolt12_in_app: false,
        warning: "A real Lightning wallet must protect channel state backups and have inbound liquidity. v0.90 keeps mainnet in-app Lightning disabled.".to_string(),
        next_steps: vec![
            "Initialize LDK Node on signet/testnet.".to_string(),
            "Generate a real test BOLT12 offer.".to_string(),
            "Receive a test Lightning payment.".to_string(),
            "Prove backup/restore before mainnet.".to_string(),
        ],
    }
}

pub fn validate_bolt12_offer(offer: &str) -> Result<Bolt12OfferInfo> {
    let trimmed = offer.trim();

    if trimmed.is_empty() {
        return Err(anyhow!("Paste a BOLT12 offer first"));
    }

    if !trimmed.to_lowercase().starts_with("lno1") {
        return Err(anyhow!("This does not look like a BOLT12 offer. BOLT12 offers usually start with lno1..."));
    }

    if trimmed.len() < 80 {
        return Err(anyhow!("This BOLT12 offer looks too short. Copy the full lno1... offer from your Lightning wallet."));
    }

    Ok(Bolt12OfferInfo {
        offer: trimmed.to_string(),
        source: "External BOLT12 offer".to_string(),
        status: "Valid-looking external BOLT12 offer saved.".to_string(),
        warning: "This stores an external BOLT12 offer only. In-app BOLT12 generation/receiving is still locked until LDK signet/testnet receive and backup recovery are proven.".to_string(),
    })
}

pub fn create_in_app_bolt12_offer() -> Result<Bolt12OfferInfo> {
    Err(anyhow!(
        "In-app BOLT12 offer generation is not enabled in v0.90. Next implementation step: LDK Node on signet/testnet, then a real test BOLT12 receive flow."
    ))
}
