use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdkSignetFundingAddressStatus {
    pub version: String,
    pub cargo_feature: String,
    pub compiled_with_ldk: bool,
    pub mainnet_locked: bool,
    pub implemented_now: Vec<String>,
    pub still_required_for_real_address: Vec<String>,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignetFundingAddressRequest {
    pub network: String,
    pub alias: String,
    pub storage_dir: String,
    pub esplora_url: String,
    pub confirmed_signet_or_testnet_only: bool,
    pub backup_plan_confirmed: bool,
    pub allow_mainnet: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignetFundingAddressPreview {
    pub safe_to_request_real_address: bool,
    pub network: String,
    pub alias: String,
    pub storage_dir: String,
    pub address_available_now: bool,
    pub funding_address: Option<String>,
    pub api_contract: Vec<String>,
    pub local_test_commands: Vec<String>,
    pub blockers: Vec<String>,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignetFundingAddressAcceptancePlan {
    pub goal: String,
    pub must_pass_before_v017: Vec<String>,
    pub must_not_do: Vec<String>,
    pub expected_success_signal: Vec<String>,
    pub contributor_notes: Vec<String>,
}

pub fn status() -> LdkSignetFundingAddressStatus {
    LdkSignetFundingAddressStatus {
        version: "0.20.0".to_string(),
        cargo_feature: "ldk-node-signet-runtime".to_string(),
        compiled_with_ldk: cfg!(feature = "ldk-node-signet-runtime"),
        mainnet_locked: true,
        implemented_now: vec![
            "Funding-address command contract for the embedded LDK node path".to_string(),
            "Mainnet guardrails before any generated address can be shown".to_string(),
            "Acceptance checklist for replacing preview data with a real ldk-node address".to_string(),
            "Frontend wiring for the signet/testnet funding-address workflow".to_string(),
        ],
        still_required_for_real_address: vec![
            "Run cargo check locally with the ldk-node feature enabled".to_string(),
            "Resolve any real ldk-node API mismatches from the local compiler".to_string(),
            "Start a real signet/testnet ldk-node instance".to_string(),
            "Call the real wallet funding/address API from ldk-node".to_string(),
            "Verify address persists across restart and restore tests".to_string(),
        ],
        warning: "v0.20 defines the safe funding-address adapter boundary. If the LDK feature is not locally compiled and tested, it must not pretend to return a real Lightning node funding address.".to_string(),
    }
}

pub fn preview_funding_address(request: SignetFundingAddressRequest) -> SignetFundingAddressPreview {
    let normalized_network = request.network.trim().to_lowercase();
    let mut blockers = Vec::new();

    if request.allow_mainnet || normalized_network == "bitcoin" || normalized_network == "mainnet" {
        blockers.push("Mainnet embedded Lightning is locked. Use signet or testnet only.".to_string());
    }
    if normalized_network != "signet" && normalized_network != "testnet" {
        blockers.push("Network must be signet or testnet for the embedded Lightning prototype.".to_string());
    }
    if !request.confirmed_signet_or_testnet_only {
        blockers.push("Confirm this workflow is signet/testnet only.".to_string());
    }
    if !request.backup_plan_confirmed {
        blockers.push("Confirm the recovery/backup plan before requesting a funding address.".to_string());
    }
    if request.storage_dir.trim().is_empty() {
        blockers.push("Storage directory is required so the node can persist keys and channel state.".to_string());
    }
    if request.esplora_url.trim().is_empty() {
        blockers.push("A signet/testnet Esplora URL is required for the first prototype.".to_string());
    }

    let compiled_with_ldk = cfg!(feature = "ldk-node-signet-runtime");
    if !compiled_with_ldk {
        blockers.push("The ldk-node-signet-runtime feature is not compiled in this build.".to_string());
    }

    let safe = blockers.is_empty();

    SignetFundingAddressPreview {
        safe_to_request_real_address: safe,
        network: if normalized_network.is_empty() { "signet".to_string() } else { normalized_network },
        alias: request.alias,
        storage_dir: request.storage_dir,
        address_available_now: false,
        funding_address: None,
        api_contract: vec![
            "Initialize or load ldk_node::Node with persistent storage".to_string(),
            "Select signet/testnet network only".to_string(),
            "Start node against Esplora/RGS test services".to_string(),
            "Request a fresh on-chain funding address from the embedded wallet".to_string(),
            "Return the address only after the node confirms network and persistence".to_string(),
        ],
        local_test_commands: vec![
            "cd src-tauri".to_string(),
            "cargo check".to_string(),
            "cargo check --features ldk-node-signet-runtime".to_string(),
            "RUST_LOG=debug cargo test --features ldk-node-signet-runtime".to_string(),
        ],
        blockers,
        warning: "This is a v0.20 adapter preview. A real address must come from the locally compiled LDK node, not a fake placeholder.".to_string(),
    }
}

pub fn acceptance_plan() -> SignetFundingAddressAcceptancePlan {
    SignetFundingAddressAcceptancePlan {
        goal: "Replace the v0.20 preview with the first real signet/testnet funding address from ldk-node.".to_string(),
        must_pass_before_v017: vec![
            "Default cargo check passes without the LDK feature".to_string(),
            "Feature cargo check passes with ldk-node-signet-runtime".to_string(),
            "Node starts on signet/testnet and never on mainnet".to_string(),
            "Funding address is generated by ldk-node and is not hardcoded".to_string(),
            "Address remains available after app restart using the same storage directory".to_string(),
        ],
        must_not_do: vec![
            "Do not return a fake tb1/tb1q address".to_string(),
            "Do not enable mainnet embedded Lightning".to_string(),
            "Do not accept real OCEAN mainnet payouts into the embedded node yet".to_string(),
            "Do not hide compiler errors behind success messages".to_string(),
        ],
        expected_success_signal: vec![
            "The UI shows address_available_now = true only in a feature build".to_string(),
            "The returned address is tied to the LDK node wallet".to_string(),
            "The same node data directory can be re-opened without losing the address".to_string(),
        ],
        contributor_notes: vec![
            "Use signet/testnet faucets only".to_string(),
            "Commit compiler errors in GitHub issues before changing APIs blindly".to_string(),
            "Keep the external BOLT12 signer MVP working while Lightning remains experimental".to_string(),
        ],
    }
}
