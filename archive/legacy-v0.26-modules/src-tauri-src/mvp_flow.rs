use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvpFlowInput {
    pub miner_name: String,
    pub bitcoin_address: String,
    pub worker: String,
    pub pool_url: String,
    pub password: String,
    pub bolt12_offer: String,
    pub ocean_message_json: String,
    pub ocean_message_valid: bool,
    pub bolt12_pairing_matches: bool,
    pub signature: String,
    pub backup_confirmed: bool,
    pub encrypted_backup_exported: bool,
    pub wallet_restored_from_backup: bool,
    pub solo_pool_selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvpFlowReport {
    pub version: String,
    pub product_mode: String,
    pub viable_external_bolt12_mvp: bool,
    pub ready_to_submit_ocean_signature: bool,
    pub ready_for_unattended_pool_mining: bool,
    pub miner_config_complete: bool,
    pub payout_config_complete: bool,
    pub signing_complete: bool,
    pub backup_complete: bool,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub acceptance_tests: Vec<String>,
    pub next_release_targets: Vec<String>,
}

fn is_probably_btc_address(address: &str) -> bool {
    let trimmed = address.trim();
    (trimmed.starts_with("bc1") || trimmed.starts_with("tb1") || trimmed.starts_with("bcrt1")) && trimmed.len() >= 14
}

fn is_probably_stratum(url: &str) -> bool {
    let trimmed = url.trim();
    trimmed.starts_with("stratum+tcp://") && trimmed.contains(':')
}

fn is_probably_bolt12(offer: &str) -> bool {
    offer.trim().to_lowercase().starts_with("lno1") && offer.trim().len() > 80
}

pub fn build_mvp_flow_report(input: MvpFlowInput) -> MvpFlowReport {
    let miner_config_complete = is_probably_stratum(&input.pool_url)
        && !input.worker.trim().is_empty()
        && input.worker.contains('.')
        && !input.password.trim().is_empty()
        && is_probably_btc_address(&input.bitcoin_address);

    let payout_config_complete = is_probably_bolt12(&input.bolt12_offer);
    let signing_complete = input.ocean_message_valid
        && input.bolt12_pairing_matches
        && !input.signature.trim().is_empty();
    let backup_complete = input.backup_confirmed && input.encrypted_backup_exported && input.wallet_restored_from_backup;

    let ready_to_submit_ocean_signature = miner_config_complete && payout_config_complete && signing_complete && input.backup_confirmed;
    let ready_for_unattended_pool_mining = ready_to_submit_ocean_signature && backup_complete && !input.solo_pool_selected;
    let viable_external_bolt12_mvp = ready_for_unattended_pool_mining;

    let mut blockers = Vec::new();
    if !is_probably_btc_address(&input.bitcoin_address) {
        blockers.push("Missing or invalid Bitcoin mining address.".to_string());
    }
    if !is_probably_stratum(&input.pool_url) {
        blockers.push("Pool URL must be a stratum+tcp:// URL.".to_string());
    }
    if input.worker.trim().is_empty() || !input.worker.contains('.') {
        blockers.push("Worker should look like bc1...WorkerName so the pool can identify this miner.".to_string());
    }
    if input.password.trim().is_empty() {
        blockers.push("Pool password field cannot be blank. Most pools accept x.".to_string());
    }
    if !payout_config_complete {
        blockers.push("Missing BOLT12 offer. External BOLT12 MVP still needs a Lexe/Core Lightning/Coinos-style lno1... offer.".to_string());
    }
    if !input.ocean_message_valid {
        blockers.push("OCEAN JSON message is missing or invalid.".to_string());
    }
    if !input.bolt12_pairing_matches {
        blockers.push("OCEAN JSON lightning_bolt12 value does not match the saved BOLT12 offer.".to_string());
    }
    if input.signature.trim().is_empty() {
        blockers.push("OCEAN message has not been signed yet.".to_string());
    }
    if !input.backup_confirmed {
        blockers.push("Seed backup confirmation is required before signing or unattended mining.".to_string());
    }
    if !input.encrypted_backup_exported {
        blockers.push("Encrypted backup export has not been completed.".to_string());
    }
    if !input.wallet_restored_from_backup {
        blockers.push("Restore-from-backup test has not been confirmed.".to_string());
    }
    if input.solo_pool_selected {
        blockers.push("Solo/lottery mining is selected. MVP unattended earning mode requires pool mining.".to_string());
    }

    let mut warnings = vec![
        "v0.20 can validate the external BOLT12 OCEAN workflow, but it still does not generate an in-app mainnet BOLT12 offer.".to_string(),
        "Keep mining identity funds small. Do not import a main wallet seed for normal product use.".to_string(),
        "Do not submit a signature if the OCEAN JSON BOLT12 value does not exactly match the saved payout offer.".to_string(),
    ];
    if ready_to_submit_ocean_signature && !backup_complete {
        warnings.push("The signature may be ready, but unattended use should wait until encrypted backup restore is tested.".to_string());
    }

    let acceptance_tests = vec![
        format!("Miner config complete for {}: {}", input.miner_name, miner_config_complete),
        format!("BOLT12 payout offer present: {}", payout_config_complete),
        format!("OCEAN JSON valid and paired: {}", input.ocean_message_valid && input.bolt12_pairing_matches),
        format!("Signature present: {}", !input.signature.trim().is_empty()),
        format!("Backup confirmed/exported/restored: {}", backup_complete),
        format!("Pool mode, not solo mode: {}", !input.solo_pool_selected),
    ];

    let next_release_targets = vec![
        "Run cargo check on the default backend and fix compile/API errors.".to_string(),
        "Run a clean external BOLT12 OCEAN flow using a disposable mining wallet and signet/testnet where possible.".to_string(),
        "Add persistent encrypted app state for the mining wallet instead of memory-only state.".to_string(),
        "Continue real ldk-node signet runtime work before unlocking any in-app mainnet Lightning receiving.".to_string(),
    ];

    MvpFlowReport {
        version: "0.20.0".to_string(),
        product_mode: if viable_external_bolt12_mvp {
            "External BOLT12 OCEAN signer flow is MVP-ready for supervised testing.".to_string()
        } else {
            "Developer preview with remaining blockers.".to_string()
        },
        viable_external_bolt12_mvp,
        ready_to_submit_ocean_signature,
        ready_for_unattended_pool_mining,
        miner_config_complete,
        payout_config_complete,
        signing_complete,
        backup_complete,
        blockers,
        warnings,
        acceptance_tests,
        next_release_targets,
    }
}
