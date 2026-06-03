use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalReleaseInput {
    pub wallet_created: bool,
    pub encrypted_backup_exported: bool,
    pub backup_restore_tested: bool,
    pub bolt12_offer_saved: bool,
    pub ocean_json_valid: bool,
    pub bolt12_pairing_matches: bool,
    pub signature_created: bool,
    pub setup_bundle_exported: bool,
    pub runbook_ready: bool,
    pub worker_online: bool,
    pub first_ocean_credit_seen: bool,
    pub no_secrets_in_exports: bool,
    pub embedded_lightning_requested: bool,
    pub user_understands_external_bolt12: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalReleaseReport {
    pub version: String,
    pub product_track: String,
    pub can_be_used_as_supervised_mvp: bool,
    pub is_all_in_one_wallet: bool,
    pub honest_status: String,
    pub passed: Vec<String>,
    pub blockers: Vec<String>,
    pub recommended_release_label: String,
    pub required_manual_evidence: Vec<String>,
    pub next_engineering_steps: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn build_external_release_report(input: ExternalReleaseInput) -> ExternalReleaseReport {
    let mut passed = Vec::new();
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();

    check(input.wallet_created, "Mining identity wallet exists", "Create or restore a CarlosK Wallet mining identity wallet", &mut passed, &mut blockers);
    check(input.encrypted_backup_exported, "Encrypted wallet backup was exported", "Export an encrypted wallet backup before using a real miner", &mut passed, &mut blockers);
    check(input.backup_restore_tested, "Backup restore test passed", "Restore the encrypted backup and confirm it returns the same BTC address", &mut passed, &mut blockers);
    check(input.bolt12_offer_saved, "External BOLT12 offer is saved", "Paste and validate an external BOLT12 offer, such as Lexe", &mut passed, &mut blockers);
    check(input.ocean_json_valid, "OCEAN payout JSON is valid", "Paste the exact OCEAN payout JSON and inspect it", &mut passed, &mut blockers);
    check(input.bolt12_pairing_matches, "OCEAN JSON BOLT12 offer matches saved offer", "Run the BOLT12 pairing check before signing", &mut passed, &mut blockers);
    check(input.signature_created, "OCEAN message signature created", "Sign the OCEAN message with the CarlosK mining wallet", &mut passed, &mut blockers);
    check(input.setup_bundle_exported, "Non-secret OCEAN setup bundle exported", "Export the non-secret OCEAN setup bundle for review", &mut passed, &mut blockers);
    check(input.runbook_ready, "Miner runbook says 24/7 pool mining is ready", "Build the miner runbook and fix any blockers", &mut passed, &mut blockers);
    check(input.worker_online, "Worker appears online in the pool dashboard", "Confirm the worker appears online in OCEAN/Braiins before calling the setup complete", &mut passed, &mut blockers);
    check(input.no_secrets_in_exports, "No secrets are present in exported setup/evidence", "Review exports/screenshots and remove seed words, xprvs, passwords, and private keys", &mut passed, &mut blockers);
    check(input.user_understands_external_bolt12, "User understands this MVP still depends on an external BOLT12 wallet", "Acknowledge that v0.24 is not yet the all-in-one embedded Lightning wallet", &mut passed, &mut blockers);

    if !input.first_ocean_credit_seen {
        warnings.push("No OCEAN credited reward or payout evidence has been confirmed yet. This does not block setup, but it blocks calling the full real-world test complete.".to_string());
    } else {
        passed.push("First OCEAN credit/payout evidence captured".to_string());
    }

    if input.embedded_lightning_requested {
        blockers.push("Embedded mainnet Lightning was requested, but it remains locked until signet/testnet receive and recovery tests pass.".to_string());
    } else {
        passed.push("Embedded mainnet Lightning remains locked".to_string());
    }

    let can_be_used_as_supervised_mvp = blockers.is_empty();
    let honest_status = if can_be_used_as_supervised_mvp {
        "The external BOLT12 + OCEAN signer flow is ready for supervised real-world MVP testing. It is not yet the finished all-in-one wallet because Lightning receiving still depends on an external BOLT12 wallet.".to_string()
    } else {
        "Not ready for supervised MVP use yet. Finish the blockers before pointing more real mining hashpower at this wallet.".to_string()
    };

    ExternalReleaseReport {
        version: "0.24.0".to_string(),
        product_track: "External BOLT12 OCEAN signer MVP".to_string(),
        can_be_used_as_supervised_mvp,
        is_all_in_one_wallet: false,
        honest_status,
        passed,
        blockers,
        recommended_release_label: if can_be_used_as_supervised_mvp { "v0.24 supervised MVP candidate" } else { "v0.24 developer preview" }.to_string(),
        required_manual_evidence: vec![
            "Screenshot or copied text showing OCEAN worker online, after redaction".to_string(),
            "Encrypted backup restore evidence showing the same BTC address, without exposing seed words".to_string(),
            "OCEAN pairing check showing the saved BOLT12 offer matches the JSON message".to_string(),
            "Signature accepted by OCEAN".to_string(),
            "At least one credited reward/payout event before claiming the full external flow is proven".to_string(),
        ],
        next_engineering_steps: vec![
            "Run cargo check locally on the default backend".to_string(),
            "Run cargo check with the ldk-node-signet-runtime feature".to_string(),
            "Replace LDK scaffolds with the first real signet funding address path".to_string(),
            "Generate a real signet/testnet BOLT12 offer inside the app".to_string(),
            "Perform a backup/restore test before enabling any mainnet embedded Lightning path".to_string(),
        ],
        warnings,
    }
}

fn check(condition: bool, pass: &str, block: &str, passed: &mut Vec<String>, blockers: &mut Vec<String>) {
    if condition {
        passed.push(pass.to_string());
    } else {
        blockers.push(block.to_string());
    }
}
