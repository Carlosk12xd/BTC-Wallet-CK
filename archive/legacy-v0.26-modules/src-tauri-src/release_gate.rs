use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalViabilityStatus {
    pub version: String,
    pub project_name: String,
    pub viable_product: bool,
    pub viable_scope_now: String,
    pub functional_today: Vec<String>,
    pub still_experimental: Vec<String>,
    pub release_blockers: Vec<String>,
    pub next_version_goal: String,
    pub honesty_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseGateInput {
    pub wallet_created: bool,
    pub backup_confirmed: bool,
    pub bolt12_offer_saved: bool,
    pub ocean_message_checked: bool,
    pub ocean_pairing_matches: bool,
    pub signature_created: bool,
    pub wants_embedded_lightning_mainnet: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseGateReport {
    pub external_bolt12_signer_mvp_ready: bool,
    pub embedded_lightning_ready: bool,
    pub safe_to_use_for_real_ocean_external_bolt12: bool,
    pub safe_to_use_for_mainnet_embedded_bolt12: bool,
    pub passed: Vec<String>,
    pub blockers: Vec<String>,
    pub next_actions: Vec<String>,
    pub acceptance_tests: Vec<String>,
    pub warning: String,
}

pub fn viability_status() -> FunctionalViabilityStatus {
    FunctionalViabilityStatus {
        version: "v0.15".to_string(),
        project_name: "CarlosK Wallet".to_string(),
        viable_product: false,
        viable_scope_now: "Developer preview: local mining-address wallet, OCEAN pool config builder, BOLT12 offer checker, OCEAN JSON pairing checker, BIP-322 signing boundary, encrypted backup flow, payout estimator, and signet-only Lightning runtime scaffolding.".to_string(),
        functional_today: vec![
            "Generate or restore a local Bitcoin mining identity wallet".to_string(),
            "Build OCEAN worker settings for Avalon Mini 3 / Apollo III / other SHA-256 miners".to_string(),
            "Save and validate an external BOLT12 offer".to_string(),
            "Inspect OCEAN Lightning payout JSON before signing".to_string(),
            "Check that OCEAN JSON contains the same BOLT12 offer the app saved".to_string(),
            "Sign the OCEAN message with the local mining wallet after backup confirmation".to_string(),
            "Estimate pool payouts, fees, power cost, and days to payout threshold".to_string(),
            "Export and restore encrypted development wallet backups".to_string(),
        ],
        still_experimental: vec![
            "In-app BOLT12 generation with LDK Node".to_string(),
            "Real signet/testnet Lightning receive tests".to_string(),
            "Mainnet Lightning receiving".to_string(),
            "Channel management and inbound liquidity UX".to_string(),
            "Automatic OCEAN API submission".to_string(),
            "Production installer signing and auto-updates".to_string(),
        ],
        release_blockers: vec![
            "Run cargo check locally for default and ldk-node-signet-runtime builds".to_string(),
            "Prove backup restore on a clean machine".to_string(),
            "Generate a real signet/testnet BOLT12 offer inside the app".to_string(),
            "Receive a real signet/testnet Lightning payment".to_string(),
            "Document recovery and channel backup procedure".to_string(),
            "Complete external security review before mainnet embedded Lightning".to_string(),
        ],
        next_version_goal: "v0.15 should be the first locally compile-tested backend milestone: default cargo check, feature-gated cargo check, and a real signet/testnet funding address path if ldk-node APIs line up.".to_string(),
        honesty_note: "Do not call this a finished viable wallet yet. v0.15 is a useful developer preview for OCEAN signing with an external BOLT12 wallet, but the all-in-one embedded Lightning wallet is not production-ready.".to_string(),
    }
}

pub fn build_release_gate_report(input: ReleaseGateInput) -> ReleaseGateReport {
    let mut passed = Vec::new();
    let mut blockers = Vec::new();

    if input.wallet_created {
        passed.push("Mining wallet/address is available".to_string());
    } else {
        blockers.push("Create or restore a local mining wallet".to_string());
    }

    if input.backup_confirmed {
        passed.push("Wallet backup confirmation completed".to_string());
    } else {
        blockers.push("Confirm seed/backup before signing any OCEAN message".to_string());
    }

    if input.bolt12_offer_saved {
        passed.push("External BOLT12 offer saved or pasted".to_string());
    } else {
        blockers.push("Paste and save a BOLT12 offer from a compatible Lightning wallet".to_string());
    }

    if input.ocean_message_checked {
        passed.push("OCEAN JSON was inspected".to_string());
    } else {
        blockers.push("Paste and inspect the exact OCEAN JSON message".to_string());
    }

    if input.ocean_pairing_matches {
        passed.push("OCEAN JSON BOLT12 offer matches saved offer".to_string());
    } else {
        blockers.push("Make sure the OCEAN JSON contains the same BOLT12 offer saved in CarlosK Wallet".to_string());
    }

    if input.signature_created {
        passed.push("OCEAN signature was generated".to_string());
    } else {
        blockers.push("Generate the OCEAN signature after pairing and backup checks pass".to_string());
    }

    if input.wants_embedded_lightning_mainnet {
        blockers.push("Mainnet embedded Lightning is intentionally locked in v0.15".to_string());
    } else {
        passed.push("Mainnet embedded Lightning was not requested".to_string());
    }

    let external_ready = input.wallet_created
        && input.backup_confirmed
        && input.bolt12_offer_saved
        && input.ocean_message_checked
        && input.ocean_pairing_matches
        && input.signature_created
        && !input.wants_embedded_lightning_mainnet;

    ReleaseGateReport {
        external_bolt12_signer_mvp_ready: external_ready,
        embedded_lightning_ready: false,
        safe_to_use_for_real_ocean_external_bolt12: external_ready,
        safe_to_use_for_mainnet_embedded_bolt12: false,
        passed,
        blockers,
        next_actions: vec![
            "Run the signed OCEAN config with a small miner first".to_string(),
            "Verify OCEAN stats show the expected worker name".to_string(),
            "Confirm Lightning payout succeeds with a tiny amount before adding more miners".to_string(),
            "Move to v0.15 for local Cargo checks and real signet runtime work".to_string(),
        ],
        acceptance_tests: vec![
            "App refuses to sign before backup confirmation".to_string(),
            "App warns when OCEAN JSON BOLT12 offer differs from saved offer".to_string(),
            "Generated signature is accepted by OCEAN for the local mining address".to_string(),
            "No mainnet embedded Lightning offer is generated in v0.15".to_string(),
            "Encrypted backup restores to the same mining address".to_string(),
        ],
        warning: if external_ready {
            "External-BOLT12 OCEAN signer MVP appears ready for cautious real-world testing. Embedded Lightning is still not ready.".to_string()
        } else {
            "This setup is not ready yet. Resolve blockers before using it for real OCEAN payout changes.".to_string()
        },
    }
}
