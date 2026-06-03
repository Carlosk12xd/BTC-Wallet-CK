use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eHarnessInput {
    pub wallet_created: bool,
    pub encrypted_backup_exported: bool,
    pub restore_verified: bool,
    pub bolt12_offer_saved: bool,
    pub ocean_json_valid: bool,
    pub bolt12_pairing_matches: bool,
    pub signature_created: bool,
    pub setup_bundle_exported: bool,
    pub worker_online: bool,
    pub first_payout_seen: bool,
    pub no_secrets_in_logs: bool,
    pub embedded_lightning_requested: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2eHarnessReport {
    pub version: String,
    pub track: String,
    pub supervised_external_mvp_ready: bool,
    pub end_to_end_test_complete: bool,
    pub can_call_external_flow_functional: bool,
    pub passed: Vec<String>,
    pub blockers: Vec<String>,
    pub manual_steps: Vec<String>,
    pub evidence_to_capture: Vec<String>,
    pub redaction_warnings: Vec<String>,
    pub honest_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionInput {
    pub text: String,
    pub btc_address: String,
    pub bolt12_offer: String,
    pub signature: String,
    pub worker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionResult {
    pub redacted_text: String,
    pub replaced_items: Vec<String>,
    pub safe_to_share: bool,
    pub warnings: Vec<String>,
}

fn add_check(condition: bool, label: &str, passed: &mut Vec<String>, blockers: &mut Vec<String>) {
    if condition {
        passed.push(label.to_string());
    } else {
        blockers.push(label.to_string());
    }
}

pub fn build_e2e_harness_report(input: E2eHarnessInput) -> E2eHarnessReport {
    let mut passed = Vec::new();
    let mut blockers = Vec::new();

    add_check(input.wallet_created, "Mining wallet was created/restored locally.", &mut passed, &mut blockers);
    add_check(input.encrypted_backup_exported, "Encrypted backup was exported.", &mut passed, &mut blockers);
    add_check(input.restore_verified, "Backup restore test recreated the same BTC mining address.", &mut passed, &mut blockers);
    add_check(input.bolt12_offer_saved, "External BOLT12 offer was saved/validated.", &mut passed, &mut blockers);
    add_check(input.ocean_json_valid, "OCEAN payout JSON was parsed as valid JSON.", &mut passed, &mut blockers);
    add_check(input.bolt12_pairing_matches, "OCEAN JSON BOLT12 offer matches the saved offer.", &mut passed, &mut blockers);
    add_check(input.signature_created, "OCEAN payout message signature was created.", &mut passed, &mut blockers);
    add_check(input.setup_bundle_exported, "Non-secret OCEAN setup bundle was exported.", &mut passed, &mut blockers);
    add_check(input.worker_online, "Miner worker appeared online on the pool dashboard.", &mut passed, &mut blockers);
    add_check(input.no_secrets_in_logs, "Screenshots/logs were reviewed for seed/private-key leakage.", &mut passed, &mut blockers);

    if input.first_payout_seen {
        passed.push("First payout or pool-side credited reward was observed.".to_string());
    } else {
        blockers.push("First payout/credited reward has not been observed yet. This can be expected until the pool payout method triggers.".to_string());
    }

    if input.embedded_lightning_requested {
        blockers.push("Embedded in-app Lightning mainnet was requested. It remains locked until signet/testnet receive and recovery pass.".to_string());
    } else {
        passed.push("Embedded mainnet Lightning remained locked.".to_string());
    }

    let supervised_external_mvp_ready = input.wallet_created
        && input.encrypted_backup_exported
        && input.restore_verified
        && input.bolt12_offer_saved
        && input.ocean_json_valid
        && input.bolt12_pairing_matches
        && input.signature_created
        && input.setup_bundle_exported
        && input.worker_online
        && input.no_secrets_in_logs
        && !input.embedded_lightning_requested;

    let end_to_end_test_complete = supervised_external_mvp_ready && input.first_payout_seen;

    E2eHarnessReport {
        version: "0.22.0".to_string(),
        track: "External BOLT12 + OCEAN signer supervised MVP".to_string(),
        supervised_external_mvp_ready,
        end_to_end_test_complete,
        can_call_external_flow_functional: supervised_external_mvp_ready,
        passed,
        blockers,
        manual_steps: vec![
            "Create or restore a dedicated mining identity wallet in CarlosK Wallet.".to_string(),
            "Export an encrypted backup and store the passphrase separately.".to_string(),
            "Restore that encrypted backup in a clean app session and verify the same BTC mining address.".to_string(),
            "Paste and save an external BOLT12 offer from Lexe/Coinos/Core Lightning.".to_string(),
            "Paste OCEAN's exact payout JSON and run the JSON checker.".to_string(),
            "Run the BOLT12 pairing checker and confirm the saved offer matches the OCEAN JSON offer.".to_string(),
            "Sign the OCEAN message only after backup confirmation.".to_string(),
            "Export the non-secret setup bundle and copy miner pool settings to the Avalon/Apollo.".to_string(),
            "Confirm the worker appears online on OCEAN/Braiins/etc. and capture redacted evidence.".to_string(),
            "Wait for credited reward/payout evidence based on the selected pool's payout policy.".to_string(),
        ],
        evidence_to_capture: vec![
            "Redacted screenshot of wallet address and generated worker.".to_string(),
            "Redacted screenshot of OCEAN JSON validity and BOLT12 pairing match.".to_string(),
            "Redacted screenshot of signature accepted by OCEAN.".to_string(),
            "Redacted screenshot of pool worker online with hashrate.".to_string(),
            "Redacted payout/credited-reward screenshot when available.".to_string(),
        ],
        redaction_warnings: vec![
            "Never share seed words, private keys, backup passphrases, or full encrypted backup JSON publicly.".to_string(),
            "Redact full BOLT12 offers unless the recipient specifically needs them for debugging.".to_string(),
            "Redact signatures in public GitHub issues unless maintainers ask for them privately.".to_string(),
            "Pool usernames can contain BTC addresses; show shortened versions in public screenshots.".to_string(),
        ],
        honest_status: if end_to_end_test_complete {
            "The external BOLT12 signer path has completed a full supervised end-to-end test. Embedded in-app Lightning is still not part of this claim.".to_string()
        } else if supervised_external_mvp_ready {
            "The external BOLT12 signer path is functionally ready for supervised pool testing, but payout evidence is still pending.".to_string()
        } else {
            "The external BOLT12 signer path still has blockers. Do not call this a functional product yet.".to_string()
        },
    }
}

fn short_token(value: &str, left: usize, right: usize) -> String {
    let trimmed = value.trim();
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() <= left + right + 6 {
        return "[redacted]".to_string();
    }
    let start: String = chars.iter().take(left).collect();
    let end: String = chars.iter().rev().take(right).collect::<Vec<_>>().into_iter().rev().collect();
    format!("{start}…{end}")
}

pub fn redact_for_issue(input: RedactionInput) -> RedactionResult {
    let mut output = input.text.clone();
    let mut replaced_items = Vec::new();
    let mut warnings = Vec::new();

    let replacements = vec![
        (input.btc_address.trim().to_string(), format!("[btc-address:{}]", short_token(&input.btc_address, 8, 6)), "BTC address"),
        (input.bolt12_offer.trim().to_string(), format!("[bolt12-offer:{}]", short_token(&input.bolt12_offer, 10, 8)), "BOLT12 offer"),
        (input.signature.trim().to_string(), format!("[signature:{}]", short_token(&input.signature, 8, 8)), "signature"),
        (input.worker.trim().to_string(), format!("[worker:{}]", short_token(&input.worker, 12, 8)), "worker"),
    ];

    for (needle, replacement, label) in replacements {
        if !needle.is_empty() && output.contains(&needle) {
            output = output.replace(&needle, &replacement);
            replaced_items.push(label.to_string());
        }
    }

    let lower = output.to_lowercase();
    let seed_red_flags = ["seed phrase", "mnemonic", "private key", "xprv", "zprv", "yprv"];
    for flag in seed_red_flags {
        if lower.contains(flag) {
            warnings.push(format!("Text still contains the phrase '{flag}'. Review before sharing."));
        }
    }

    if output.split_whitespace().count() >= 12 && (lower.contains("abandon") || lower.contains("wallet backup")) {
        warnings.push("This text may contain wallet backup material. Do not share publicly until reviewed manually.".to_string());
    }

    let safe_to_share = warnings.is_empty();

    RedactionResult {
        redacted_text: output,
        replaced_items,
        safe_to_share,
        warnings,
    }
}
