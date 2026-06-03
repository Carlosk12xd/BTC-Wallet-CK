use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanSetupBundleInput {
    pub miner_name: String,
    pub bitcoin_address: String,
    pub worker_label: String,
    pub pool_url: String,
    pub pool_password: String,
    pub bolt12_offer: String,
    pub ocean_message_valid: bool,
    pub pairing_matches: bool,
    pub signature: String,
    pub backup_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanMinerConfig {
    pub pool_url: String,
    pub worker: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanPayoutConfig {
    pub bitcoin_mining_address: String,
    pub bolt12_offer_preview: String,
    pub payout_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanSigningStatus {
    pub backup_confirmed: bool,
    pub ocean_message_valid: bool,
    pub bolt12_pairing_matches: bool,
    pub signature_present: bool,
    pub safe_to_submit_signature: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanSetupBundle {
    pub version: String,
    pub miner_name: String,
    pub ready_for_ocean_confirmation: bool,
    pub miner_config: OceanMinerConfig,
    pub payout_config: OceanPayoutConfig,
    pub signing_status: OceanSigningStatus,
    pub copy_paste_steps: Vec<String>,
    pub warnings: Vec<String>,
    pub export_json: String,
}

fn clean_worker_label(label: &str) -> String {
    let cleaned: String = label
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    if cleaned.trim().is_empty() {
        "EpicAvalon".to_string()
    } else {
        cleaned
    }
}

fn preview_offer(offer: &str) -> String {
    let trimmed = offer.trim();
    if trimmed.len() <= 32 {
        return trimmed.to_string();
    }
    format!("{}...{}", &trimmed[..16], &trimmed[trimmed.len() - 12..])
}

pub fn build_ocean_setup_bundle(input: OceanSetupBundleInput) -> OceanSetupBundle {
    let worker_label = clean_worker_label(&input.worker_label);
    let bitcoin_address = input.bitcoin_address.trim().to_string();
    let pool_url = if input.pool_url.trim().is_empty() {
        "stratum+tcp://mine.ocean.xyz:3334".to_string()
    } else {
        input.pool_url.trim().to_string()
    };
    let password = if input.pool_password.trim().is_empty() {
        "x".to_string()
    } else {
        input.pool_password.trim().to_string()
    };
    let worker = format!("{bitcoin_address}.{worker_label}");
    let bolt12_offer = input.bolt12_offer.trim().to_string();
    let signature_present = input.signature.trim().len() >= 24;
    let address_looks_valid = bitcoin_address.starts_with("bc1") || bitcoin_address.starts_with("tb1") || bitcoin_address.starts_with("bcrt1");
    let bolt12_looks_valid = bolt12_offer.starts_with("lno1");

    let safe_to_submit_signature = input.backup_confirmed
        && address_looks_valid
        && bolt12_looks_valid
        && input.ocean_message_valid
        && input.pairing_matches
        && signature_present;

    let mut warnings = Vec::new();
    if !address_looks_valid {
        warnings.push("Bitcoin mining address does not look like a bech32 address. Re-check before mining.".to_string());
    }
    if !bolt12_looks_valid {
        warnings.push("BOLT12 offer does not start with lno1. Do not submit this to OCEAN.".to_string());
    }
    if !input.backup_confirmed {
        warnings.push("Wallet backup is not confirmed. Do not mine to an address you cannot restore.".to_string());
    }
    if !input.ocean_message_valid {
        warnings.push("OCEAN JSON message has not been validated.".to_string());
    }
    if !input.pairing_matches {
        warnings.push("OCEAN JSON BOLT12 offer does not match the saved BOLT12 offer.".to_string());
    }
    if !signature_present {
        warnings.push("No signature is present yet. Sign the OCEAN message before submitting.".to_string());
    }
    if warnings.is_empty() {
        warnings.push("Bundle looks ready for OCEAN confirmation. Still verify every value on OCEAN before submitting.".to_string());
    }

    let miner_config = OceanMinerConfig {
        pool_url: pool_url.clone(),
        worker: worker.clone(),
        password: password.clone(),
    };
    let payout_config = OceanPayoutConfig {
        bitcoin_mining_address: bitcoin_address.clone(),
        bolt12_offer_preview: preview_offer(&bolt12_offer),
        payout_path: "OCEAN mining address -> BOLT12 Lightning offer".to_string(),
    };
    let signing_status = OceanSigningStatus {
        backup_confirmed: input.backup_confirmed,
        ocean_message_valid: input.ocean_message_valid,
        bolt12_pairing_matches: input.pairing_matches,
        signature_present,
        safe_to_submit_signature,
    };

    let copy_paste_steps = vec![
        format!("Set miner pool URL to {pool_url}"),
        format!("Set miner worker/username to {worker}"),
        format!("Set miner password to {password}"),
        "Paste the BOLT12 offer into OCEAN Lightning payout settings.".to_string(),
        "Paste OCEAN's exact JSON message into CarlosK Wallet and verify the offer match.".to_string(),
        "Sign locally, then paste the signature into OCEAN only if all checks are green.".to_string(),
    ];

    let export_payload = serde_json::json!({
        "version": "0.20.0",
        "project": "CarlosK Wallet",
        "miner_name": input.miner_name,
        "pool_url": pool_url,
        "worker": worker,
        "password": password,
        "bitcoin_mining_address": bitcoin_address,
        "bolt12_offer_preview": preview_offer(&bolt12_offer),
        "ready_for_ocean_confirmation": safe_to_submit_signature,
        "signature_present": signature_present,
        "warning": "This export intentionally does not include seed words or private keys."
    });

    OceanSetupBundle {
        version: "0.20.0".to_string(),
        miner_name: input.miner_name,
        ready_for_ocean_confirmation: safe_to_submit_signature,
        miner_config,
        payout_config,
        signing_status,
        copy_paste_steps,
        warnings,
        export_json: serde_json::to_string_pretty(&export_payload).unwrap_or_else(|_| "{}".to_string()),
    }
}
