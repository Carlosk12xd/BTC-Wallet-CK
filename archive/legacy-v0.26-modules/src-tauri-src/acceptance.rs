use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceInput {
    pub frontend_build_passed: bool,
    pub default_cargo_check_passed: bool,
    pub clean_clone_tested: bool,
    pub wallet_created: bool,
    pub encrypted_backup_exported: bool,
    pub restore_tested: bool,
    pub ocean_json_valid: bool,
    pub bolt12_pairing_matches: bool,
    pub signature_created: bool,
    pub setup_bundle_exported: bool,
    pub runbook_ready: bool,
    pub solo_mode_disabled: bool,
    pub no_seed_in_export: bool,
    pub mainnet_embedded_lightning_locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceReport {
    pub version: String,
    pub product_track: String,
    pub external_bolt12_release_candidate: bool,
    pub embedded_lightning_release_candidate: bool,
    pub safe_for_supervised_ocean_setup: bool,
    pub passed: Vec<String>,
    pub blockers: Vec<String>,
    pub manual_test_script: Vec<String>,
    pub release_notes: Vec<String>,
    pub next_engineering_targets: Vec<String>,
    pub warning: String,
}

fn pass_or_block(value: bool, pass: &str, block: &str, passed: &mut Vec<String>, blockers: &mut Vec<String>) {
    if value {
        passed.push(pass.to_string());
    } else {
        blockers.push(block.to_string());
    }
}

pub fn build_acceptance_report(input: AcceptanceInput) -> AcceptanceReport {
    let mut passed = Vec::new();
    let mut blockers = Vec::new();

    pass_or_block(input.frontend_build_passed, "Frontend TypeScript/Vite build passed.", "Run npm run check:frontend and fix any UI/type errors.", &mut passed, &mut blockers);
    pass_or_block(input.default_cargo_check_passed, "Default Rust/Tauri cargo check passed.", "Run npm run check:tauri and fix Rust compile errors.", &mut passed, &mut blockers);
    pass_or_block(input.clean_clone_tested, "Clean unzip/clone validation was tested.", "Test the ZIP/repository from a clean folder, not only the working tree.", &mut passed, &mut blockers);
    pass_or_block(input.wallet_created, "Mining identity wallet can be created/restored.", "Create or restore a mining wallet first.", &mut passed, &mut blockers);
    pass_or_block(input.encrypted_backup_exported, "Encrypted wallet backup export exists.", "Export an encrypted backup before signing or mining to the address.", &mut passed, &mut blockers);
    pass_or_block(input.restore_tested, "Encrypted backup restore was tested and returns the same address.", "Restore the encrypted backup and confirm it returns the same mining address.", &mut passed, &mut blockers);
    pass_or_block(input.ocean_json_valid, "OCEAN JSON message validates.", "Paste and validate OCEAN's exact JSON message.", &mut passed, &mut blockers);
    pass_or_block(input.bolt12_pairing_matches, "OCEAN JSON BOLT12 offer matches the saved payout offer.", "Do not sign until OCEAN's JSON offer matches the offer stored in CarlosK Wallet.", &mut passed, &mut blockers);
    pass_or_block(input.signature_created, "OCEAN payout message signature was created.", "Sign the OCEAN message with the mining address wallet.", &mut passed, &mut blockers);
    pass_or_block(input.setup_bundle_exported, "Non-secret OCEAN setup bundle was exported.", "Build/export the OCEAN setup bundle for review.", &mut passed, &mut blockers);
    pass_or_block(input.runbook_ready, "Miner operations runbook is ready for 24/7 pool mining.", "Build the miner runbook and clear operational blockers.", &mut passed, &mut blockers);
    pass_or_block(input.solo_mode_disabled, "Solo/lottery mode is disabled for production payout flow.", "Switch from solo mining to pool mining for realistic payouts.", &mut passed, &mut blockers);
    pass_or_block(input.no_seed_in_export, "Export safety confirmed: no seed/private keys in setup bundle.", "Confirm setup exports do not contain seed words, xprv, private keys, or wallet passwords.", &mut passed, &mut blockers);
    pass_or_block(input.mainnet_embedded_lightning_locked, "Embedded mainnet Lightning remains locked.", "Keep embedded mainnet Lightning locked until LDK recovery/liquidity tests pass.", &mut passed, &mut blockers);

    let external_blockers = blockers.iter().filter(|b| {
        !b.contains("Embedded") && !b.contains("LDK")
    }).count();

    let external_ready = external_blockers == 0;

    AcceptanceReport {
        version: "v0.20.0".to_string(),
        product_track: "External BOLT12 + OCEAN signer release-candidate gate".to_string(),
        external_bolt12_release_candidate: external_ready,
        embedded_lightning_release_candidate: false,
        safe_for_supervised_ocean_setup: external_ready && input.mainnet_embedded_lightning_locked,
        passed,
        blockers,
        manual_test_script: vec![
            "Create a fresh mining wallet and write down the seed offline.".to_string(),
            "Export an encrypted backup, then restore it and confirm the same bc1q address appears.".to_string(),
            "Paste an external BOLT12 offer and save it.".to_string(),
            "Paste OCEAN's JSON payout configuration and verify the offer pairing check passes.".to_string(),
            "Sign the OCEAN JSON message and paste the signature into OCEAN.".to_string(),
            "Build the OCEAN setup bundle and confirm it contains no secrets.".to_string(),
            "Configure a miner with pool URL, worker, and password from the bundle.".to_string(),
            "Confirm the worker appears online on the pool dashboard.".to_string(),
        ],
        release_notes: vec![
            "v0.20 is aimed at making the external BOLT12 OCEAN flow a supervised release candidate.".to_string(),
            "It does not claim embedded in-app Lightning/BOLT12 receiving is production ready.".to_string(),
            "Mainnet embedded Lightning stays locked until signet/testnet funding, backup, restore, liquidity, and receive tests pass.".to_string(),
        ],
        next_engineering_targets: vec![
            "Run Cargo checks locally and fix all default-backend compile errors.".to_string(),
            "Add Playwright-style UI smoke tests or a lightweight scripted test harness.".to_string(),
            "Make the OCEAN setup bundle downloadable as a file from the UI.".to_string(),
            "Begin real ldk-node signet funding address implementation once backend compile results are available.".to_string(),
        ],
        warning: "This report can approve the supervised external-BOLT12 signer workflow, not a fully embedded Lightning wallet. Do not market embedded mainnet Lightning as working yet.".to_string(),
    }
}
