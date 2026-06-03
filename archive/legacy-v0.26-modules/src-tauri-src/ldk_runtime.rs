use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdkRuntimeStatus {
    pub version: String,
    pub cargo_feature: String,
    pub compiled_in: bool,
    pub supported_networks_now: Vec<String>,
    pub mainnet_enabled: bool,
    pub adapter_layer: String,
    pub implemented_now: Vec<String>,
    pub still_stubbed: Vec<String>,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdkRuntimePreflightInput {
    pub network: String,
    pub alias: String,
    pub data_dir: String,
    pub esplora_url: String,
    pub peer_pubkey: String,
    pub peer_address: String,
    pub confirmed_signet_only: bool,
    pub recovery_phrase_written: bool,
    pub backup_restore_tested: bool,
    pub has_inbound_liquidity_plan: bool,
    pub wants_mainnet: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdkRuntimePreflight {
    pub safe_to_start_experimental_node: bool,
    pub network: String,
    pub alias: String,
    pub data_dir: String,
    pub esplora_url: String,
    pub peer_summary: String,
    pub passed: Vec<String>,
    pub blockers: Vec<String>,
    pub next_local_commands: Vec<String>,
    pub test_plan: Vec<String>,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignetOfferAcceptanceTest {
    pub test_name: String,
    pub input_offer_prefix: String,
    pub safe_for_ocean_mainnet: bool,
    pub acceptance_criteria: Vec<String>,
    pub failure_modes: Vec<String>,
    pub contributor_notes: Vec<String>,
}

pub fn runtime_status() -> LdkRuntimeStatus {
    LdkRuntimeStatus {
        version: "v0.13".to_string(),
        cargo_feature: "ldk-node-signet".to_string(),
        compiled_in: cfg!(feature = "ldk-node-signet"),
        supported_networks_now: vec!["signet".to_string(), "testnet".to_string()],
        mainnet_enabled: false,
        adapter_layer: "CarlosK Wallet keeps Lightning behind an adapter boundary so the UI, OCEAN pairing checker, and wallet backup flow can be tested before real mainnet receive is enabled.".to_string(),
        implemented_now: vec![
            "Runtime preflight model for signet/testnet LDK Node startup".to_string(),
            "Mainnet lock in command layer".to_string(),
            "Contributor-facing local command plan for an ldk-node signet branch".to_string(),
            "BOLT12 acceptance-test checklist for future generated offers".to_string(),
        ],
        still_stubbed: vec![
            "Real ldk-node Builder initialization".to_string(),
            "Persistent Lightning node seed storage".to_string(),
            "Channel open and peer connection".to_string(),
            "Real BOLT12 offer generation".to_string(),
            "Payment receipt event stream".to_string(),
        ],
        warning: "v0.13 is still a safe signet/testnet runtime preflight. It does not create a real OCEAN mainnet Lightning receive wallet yet.".to_string(),
    }
}

pub fn preflight(input: LdkRuntimePreflightInput) -> LdkRuntimePreflight {
    let network = input.network.trim().to_lowercase();
    let is_test_network = network == "signet" || network == "testnet";
    let has_alias = !input.alias.trim().is_empty();
    let has_data_dir = !input.data_dir.trim().is_empty();
    let has_esplora = input.esplora_url.starts_with("https://") || input.esplora_url.starts_with("http://");
    let has_peer = !input.peer_pubkey.trim().is_empty() && !input.peer_address.trim().is_empty();

    let mut passed = Vec::new();
    let mut blockers = Vec::new();

    if is_test_network { passed.push(format!("Network is locked to {network}.")); } else { blockers.push("Network must be signet or testnet for the experimental runtime.".to_string()); }
    if !input.wants_mainnet { passed.push("User did not request mainnet Lightning receive.".to_string()); } else { blockers.push("Mainnet Lightning receive is disabled until backups, liquidity, and force-close recovery are proven.".to_string()); }
    if input.confirmed_signet_only { passed.push("User confirmed this workflow is signet/testnet-only.".to_string()); } else { blockers.push("Confirm the signet/testnet-only checkbox before starting the runtime branch.".to_string()); }
    if has_alias { passed.push("Node alias is set.".to_string()); } else { blockers.push("Set a node alias.".to_string()); }
    if has_data_dir { passed.push("Data directory is set.".to_string()); } else { blockers.push("Set a local data directory for the Lightning node.".to_string()); }
    if has_esplora { passed.push("Esplora endpoint looks usable.".to_string()); } else { blockers.push("Set an http(s) signet/testnet Esplora endpoint.".to_string()); }
    if input.recovery_phrase_written { passed.push("Recovery phrase / node secret backup is acknowledged.".to_string()); } else { blockers.push("Write down and protect the node recovery material before any funded test.".to_string()); }
    if input.backup_restore_tested { passed.push("Backup restore test has been acknowledged.".to_string()); } else { blockers.push("Run a clean-machine restore test before mainnet beta.".to_string()); }
    if input.has_inbound_liquidity_plan { passed.push("Inbound liquidity plan exists.".to_string()); } else { blockers.push("Define how the app will get inbound liquidity for OCEAN payouts.".to_string()); }
    if has_peer { passed.push("Optional peer target is set.".to_string()); } else { blockers.push("Add a signet/testnet peer or document an LSP/test-peer strategy.".to_string()); }

    let safe = blockers.is_empty();

    LdkRuntimePreflight {
        safe_to_start_experimental_node: safe,
        network: if network.is_empty() { "signet".to_string() } else { network },
        alias: input.alias.trim().to_string(),
        data_dir: input.data_dir.trim().to_string(),
        esplora_url: input.esplora_url.trim().to_string(),
        peer_summary: if has_peer { format!("{}@{}", input.peer_pubkey.trim(), input.peer_address.trim()) } else { "No peer configured".to_string() },
        passed,
        blockers,
        next_local_commands: vec![
            "git checkout -b feature/ldk-node-signet-runtime".to_string(),
            "cd src-tauri".to_string(),
            "cargo add ldk-node --optional".to_string(),
            "cargo check --features ldk-node-signet".to_string(),
            "cargo test --features ldk-node-signet".to_string(),
            "npm run dev".to_string(),
        ],
        test_plan: vec![
            "Start the runtime on signet only.".to_string(),
            "Generate a signet receive address and fund it with signet coins.".to_string(),
            "Connect to a known signet Lightning peer.".to_string(),
            "Open a small signet channel.".to_string(),
            "Generate a real signet BOLT12 offer through ldk-node.".to_string(),
            "Pay that offer from another signet wallet and verify the app records the payment.".to_string(),
            "Delete local data and restore from backup before any mainnet consideration.".to_string(),
        ],
        warning: if safe { "Preflight passed for a local experimental signet branch. Still do not use mainnet funds.".to_string() } else { "Preflight has blockers. Do not start a funded Lightning runtime yet.".to_string() },
    }
}

pub fn offer_acceptance_test() -> SignetOfferAcceptanceTest {
    SignetOfferAcceptanceTest {
        test_name: "Generated BOLT12 offer acceptance test".to_string(),
        input_offer_prefix: "lno1".to_string(),
        safe_for_ocean_mainnet: false,
        acceptance_criteria: vec![
            "Generated offer starts with lno1.".to_string(),
            "Offer is produced by the embedded signet/testnet Lightning node, not pasted from an external wallet.".to_string(),
            "Offer is saved to local encrypted storage with the node identity metadata.".to_string(),
            "OCEAN JSON pairing checker confirms the exact generated offer before signing.".to_string(),
            "A test payment to the offer is received and visible in the local event log.".to_string(),
        ],
        failure_modes: vec![
            "Offer belongs to a different wallet than the local Lightning node.".to_string(),
            "Offer is copied into OCEAN without verifying the JSON pairing.".to_string(),
            "Node lacks inbound liquidity, so OCEAN payouts fail or retry.".to_string(),
            "Backup cannot restore the node identity/channel state.".to_string(),
        ],
        contributor_notes: vec![
            "Keep this test on signet/testnet until the mainnet release gates are completed.".to_string(),
            "Do not merge code that stores Lightning secrets unencrypted.".to_string(),
            "Do not remove the pairing verification step; it prevents payout redirection mistakes.".to_string(),
        ],
    }
}
