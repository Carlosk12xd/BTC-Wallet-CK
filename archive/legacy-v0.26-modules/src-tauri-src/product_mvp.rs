use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductMvpStatus {
    pub version: String,
    pub product_name: String,
    pub mvp_track: String,
    pub functional_viable_product: bool,
    pub external_bolt12_flow_status: String,
    pub embedded_lightning_status: String,
    pub what_can_be_used_now: Vec<String>,
    pub what_still_blocks_v1: Vec<String>,
    pub next_version_targets: Vec<String>,
    pub honest_note: String,
}

pub fn status() -> ProductMvpStatus {
    ProductMvpStatus {
        version: "0.22.0".to_string(),
        product_name: "CarlosK Wallet".to_string(),
        mvp_track: "External BOLT12 + OCEAN signer MVP hardening".to_string(),
        functional_viable_product: false,
        external_bolt12_flow_status: "Close to supervised MVP, but still requires real local Cargo/Tauri testing and a manual OCEAN end-to-end test.".to_string(),
        embedded_lightning_status: "Still locked. The app does not yet generate or custody a production BOLT12 receive wallet.".to_string(),
        what_can_be_used_now: vec![
            "Create or restore a Bitcoin mining identity wallet locally.".to_string(),
            "Generate OCEAN pool settings for SHA-256 miners.".to_string(),
            "Save and validate an external BOLT12 offer from Lexe/Coinos/Core Lightning.".to_string(),
            "Inspect OCEAN payout JSON before signing.".to_string(),
            "Verify that the OCEAN JSON contains the same BOLT12 offer saved in CarlosK Wallet.".to_string(),
            "Sign the OCEAN message only after backup confirmation.".to_string(),
            "Export a non-secret OCEAN setup bundle.".to_string(),
        ],
        what_still_blocks_v1: vec![
            "Default Rust/Tauri backend must pass cargo check on a real developer machine.".to_string(),
            "Encrypted backup restore must be tested in a clean install and confirm the same mining address.".to_string(),
            "A real OCEAN payout-configuration signature must be accepted by OCEAN on mainnet using an external BOLT12 offer.".to_string(),
            "Embedded LDK Node must produce a real signet/testnet funding address before mainnet Lightning is even considered.".to_string(),
            "Embedded BOLT12 receive must pass signet/testnet receive, backup, restore, and recovery tests.".to_string(),
            "Security review must confirm no seed/private-key material is exported in setup bundles or logs.".to_string(),
        ],
        next_version_targets: vec![
            "Use the v0.22 manual end-to-end test harness to prove the external OCEAN/BOLT12 flow.".to_string(),
            "Use the v0.22 redaction helper before GitHub issues are filed.".to_string(),
            "Add release notes that clearly separate the supervised MVP from the future embedded Lightning wallet.".to_string(),
            "Prepare v0.23 to focus on first-run onboarding and real local Cargo results.".to_string(),
        ],
        honest_note: "v0.22 is not yet the finished all-in-one wallet. It is a hardening checkpoint that makes the first realistic MVP path explicit: use CarlosK Wallet as the mining address + OCEAN signer while payouts still go to an external BOLT12 wallet.".to_string(),
    }
}
