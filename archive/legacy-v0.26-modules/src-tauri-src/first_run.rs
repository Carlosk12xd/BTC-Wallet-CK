use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstRunWizardInput {
    pub wallet_created: bool,
    pub encrypted_backup_exported: bool,
    pub backup_restore_tested: bool,
    pub bolt12_offer_saved: bool,
    pub ocean_json_valid: bool,
    pub bolt12_pairing_matches: bool,
    pub signature_created: bool,
    pub setup_bundle_exported: bool,
    pub worker_online: bool,
    pub wants_embedded_lightning: bool,
    pub selected_miner: String,
    pub selected_pool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstRunWizardStep {
    pub id: String,
    pub title: String,
    pub complete: bool,
    pub action: String,
    pub safety_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstRunWizardReport {
    pub version: String,
    pub product_track: String,
    pub selected_miner: String,
    pub selected_pool: String,
    pub completion_percent: u8,
    pub ready_to_mine_with_external_bolt12: bool,
    pub ready_for_mainnet_embedded_lightning: bool,
    pub next_action: String,
    pub steps: Vec<FirstRunWizardStep>,
    pub copy_ready_pool_fields: Vec<String>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
}

fn step(id: &str, title: &str, complete: bool, action: &str, safety_note: &str) -> FirstRunWizardStep {
    FirstRunWizardStep {
        id: id.to_string(),
        title: title.to_string(),
        complete,
        action: action.to_string(),
        safety_note: safety_note.to_string(),
    }
}

pub fn build_first_run_wizard_report(input: FirstRunWizardInput) -> FirstRunWizardReport {
    let steps = vec![
        step(
            "wallet",
            "Create or restore the mining identity wallet",
            input.wallet_created,
            "Create a new CarlosK Wallet mining identity or restore the encrypted backup.",
            "This wallet should be a dedicated mining identity, not a main savings wallet.",
        ),
        step(
            "backup-export",
            "Export encrypted backup",
            input.encrypted_backup_exported,
            "Export the encrypted wallet backup before signing payout messages.",
            "Never export or share plaintext seed words. Store encrypted backup and passphrase separately.",
        ),
        step(
            "backup-restore",
            "Verify restore to the same address",
            input.backup_restore_tested,
            "Restore the encrypted backup and confirm it returns the same bc1 address.",
            "This is required before the app should be trusted with repeated payout signing.",
        ),
        step(
            "bolt12",
            "Save external BOLT12 offer",
            input.bolt12_offer_saved,
            "Paste the external lno1... offer from Lexe or another supported BOLT12 wallet.",
            "External BOLT12 is the current MVP path; embedded Lightning remains locked.",
        ),
        step(
            "ocean-json",
            "Validate OCEAN payout JSON",
            input.ocean_json_valid,
            "Paste the OCEAN configuration JSON and inspect it before signing.",
            "Only sign the exact message shown by OCEAN for the expected mining address.",
        ),
        step(
            "pairing",
            "Confirm OCEAN JSON matches saved BOLT12 offer",
            input.bolt12_pairing_matches,
            "Run pairing verification and confirm the lightning_bolt12 field matches your saved offer.",
            "If it does not match, do not sign. A mismatched offer can redirect payouts.",
        ),
        step(
            "signature",
            "Sign OCEAN message",
            input.signature_created,
            "Create the BIP-322 signature and paste it into OCEAN.",
            "Signing proves you control the BTC mining address; it does not reveal your seed.",
        ),
        step(
            "setup-bundle",
            "Export non-secret setup bundle",
            input.setup_bundle_exported,
            "Export the pool URL, worker, password, and checklist for the miner setup.",
            "The setup bundle must never include seed words, private keys, or passwords.",
        ),
        step(
            "worker-online",
            "Confirm worker online in pool dashboard",
            input.worker_online,
            "Check that the worker is online and hashrate appears after 10-30 minutes.",
            "Pool-side hashrate and accepted shares are the evidence that mining is credited.",
        ),
    ];

    let completed = steps.iter().filter(|s| s.complete).count() as u32;
    let completion_percent = ((completed * 100) / steps.len() as u32) as u8;

    let ready_to_mine_with_external_bolt12 = input.wallet_created
        && input.encrypted_backup_exported
        && input.backup_restore_tested
        && input.bolt12_offer_saved
        && input.ocean_json_valid
        && input.bolt12_pairing_matches
        && input.signature_created
        && input.setup_bundle_exported;

    let ready_for_mainnet_embedded_lightning = false;

    let mut blockers = Vec::new();
    for s in &steps {
        if !s.complete {
            blockers.push(format!("{}: {}", s.title, s.action));
        }
    }
    if input.wants_embedded_lightning {
        blockers.push("Embedded mainnet Lightning remains locked until signet/testnet receive, backup, recovery, and liquidity tests pass.".to_string());
    }

    let next_action = steps
        .iter()
        .find(|s| !s.complete)
        .map(|s| s.action.clone())
        .unwrap_or_else(|| {
            if input.worker_online {
                "Run the first 24-hour supervised mining test and capture evidence of credited rewards.".to_string()
            } else {
                "Point the miner to the generated pool fields and confirm worker online.".to_string()
            }
        });

    let copy_ready_pool_fields = vec![
        "Pool URL: stratum+tcp://mine.ocean.xyz:3334".to_string(),
        "Worker: bc1qGeneratedByCarlosKWallet.<miner-label>".to_string(),
        "Password: x".to_string(),
    ];

    let mut warnings = vec![
        "v0.23 is a guided first-run wizard for the external BOLT12 MVP path, not the finished embedded Lightning wallet.".to_string(),
        "Use pool mining for realistic payouts; solo mining remains lottery mining.".to_string(),
        "Do not paste seed words into websites or GitHub issues.".to_string(),
    ];
    if !input.selected_pool.to_lowercase().contains("ocean") {
        warnings.push("This wizard is optimized for OCEAN + BOLT12. Other pools may not use the same signing flow.".to_string());
    }

    FirstRunWizardReport {
        version: "0.23.0".to_string(),
        product_track: "guided external BOLT12 MVP".to_string(),
        selected_miner: input.selected_miner,
        selected_pool: input.selected_pool,
        completion_percent,
        ready_to_mine_with_external_bolt12,
        ready_for_mainnet_embedded_lightning,
        next_action,
        steps,
        copy_ready_pool_fields,
        blockers,
        warnings,
    }
}
