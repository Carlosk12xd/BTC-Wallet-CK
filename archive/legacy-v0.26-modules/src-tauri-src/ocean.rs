use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const OCEAN_POOL_URL: &str = "stratum+tcp://mine.ocean.xyz:3334";
pub const OCEAN_POOL_HOST_PORT: &str = "mine.ocean.xyz:3334";
pub const OCEAN_PASSWORD: &str = "x";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanPoolConfig {
    pub pool_url: String,
    pub pool_host_port: String,
    pub worker: String,
    pub password: String,
    pub instructions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanMessageCheck {
    pub valid_json: bool,
    pub has_lightning_bolt12: bool,
    pub height: Option<u64>,
    pub preview: String,
    pub warning: String,
}

pub fn build_pool_config(address: &str, label: &str) -> OceanPoolConfig {
    let clean_label = sanitize_worker_label(label);
    OceanPoolConfig {
        pool_url: OCEAN_POOL_URL.to_string(),
        pool_host_port: OCEAN_POOL_HOST_PORT.to_string(),
        worker: format!("{}.{}", address, clean_label),
        password: OCEAN_PASSWORD.to_string(),
        instructions: vec![
            "Use pool URL, worker, and password in your Avalon/Apollo miner settings.".to_string(),
            "If a miner rejects the full stratum URL, use mine.ocean.xyz:3334 instead.".to_string(),
            "The worker label after the dot is optional but helps identify each miner.".to_string(),
        ],
    }
}

pub fn inspect_ocean_message(message: &str) -> Result<OceanMessageCheck> {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("Paste the exact OCEAN JSON message first"));
    }

    let value: Value = serde_json::from_str(trimmed)
        .map_err(|e| anyhow!("OCEAN message is not valid JSON: {e}"))?;

    let has_lightning_bolt12 = value.get("lightning_bolt12").is_some();
    let height = value.get("height").and_then(|v| v.as_u64());

    let preview = if trimmed.len() > 240 {
        format!("{}...", &trimmed[..240])
    } else {
        trimmed.to_string()
    };

    let warning = if has_lightning_bolt12 {
        "Looks like an OCEAN Lightning/BOLT12 configuration message. Sign only if the offer and mining address are yours.".to_string()
    } else {
        "JSON is valid, but it does not contain lightning_bolt12. Make sure you copied the correct OCEAN message.".to_string()
    };

    Ok(OceanMessageCheck {
        valid_json: true,
        has_lightning_bolt12,
        height,
        preview,
        warning,
    })
}

pub fn sanitize_worker_label(label: &str) -> String {
    let cleaned: String = label
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    if cleaned.is_empty() { "Miner".to_string() } else { cleaned }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanLightningPairingCheck {
    pub valid_json: bool,
    pub offer_matches: bool,
    pub message_offer_preview: String,
    pub expected_offer_preview: String,
    pub warning: String,
}

fn preview_text(value: &str, keep: usize) -> String {
    let clean = value.trim();
    if clean.len() <= keep {
        clean.to_string()
    } else {
        format!("{}...", &clean[..keep])
    }
}

pub fn check_lightning_pairing(message: &str, expected_offer: &str) -> Result<OceanLightningPairingCheck> {
    let trimmed_message = message.trim();
    let expected = expected_offer.trim();

    if trimmed_message.is_empty() {
        return Err(anyhow!("Paste the exact OCEAN JSON message first"));
    }
    if expected.is_empty() {
        return Err(anyhow!("Paste/save the expected BOLT12 offer first"));
    }
    if !expected.to_lowercase().starts_with("lno1") {
        return Err(anyhow!("Expected offer does not look like BOLT12. It should start with lno1..."));
    }

    let value: Value = serde_json::from_str(trimmed_message)
        .map_err(|e| anyhow!("OCEAN message is not valid JSON: {e}"))?;

    let message_offer = value
        .get("lightning_bolt12")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("OCEAN message JSON does not contain a lightning_bolt12 string"))?;

    let offer_matches = message_offer.trim() == expected;
    let warning = if offer_matches {
        "The OCEAN message BOLT12 offer matches the offer saved in CarlosK Wallet. This is the message you can sign if the mining address is yours.".to_string()
    } else {
        "BOLT12 mismatch. Do not sign this message unless you intentionally changed the payout offer. A mismatch could send payouts to the wrong Lightning wallet.".to_string()
    };

    Ok(OceanLightningPairingCheck {
        valid_json: true,
        offer_matches,
        message_offer_preview: preview_text(message_offer, 90),
        expected_offer_preview: preview_text(expected, 90),
        warning,
    })
}
