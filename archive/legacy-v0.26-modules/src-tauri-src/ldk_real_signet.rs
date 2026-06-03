use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdkRealSignetPrototypeStatus {
    pub version: String,
    pub feature_flag: String,
    pub compiled_with_ldk: bool,
    pub target_networks: Vec<String>,
    pub api_targets: Vec<String>,
    pub files_touched: Vec<String>,
    pub cargo_check_commands: Vec<String>,
    pub mainnet_blocked: bool,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdkRealSignetStartInput {
    pub network: String,
    pub alias: String,
    pub storage_dir: String,
    pub esplora_url: String,
    pub rgs_url: String,
    pub confirmed_signet_or_testnet_only: bool,
    pub allow_mainnet: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdkRealSignetStartPreview {
    pub safe_to_attempt_local_run: bool,
    pub network: String,
    pub alias: String,
    pub storage_dir: String,
    pub feature_flag: String,
    pub builder_steps: Vec<String>,
    pub rust_api_scaffold: Vec<String>,
    pub local_test_commands: Vec<String>,
    pub expected_success_signals: Vec<String>,
    pub blockers: Vec<String>,
    pub warning: String,
}

pub fn prototype_status() -> LdkRealSignetPrototypeStatus {
    LdkRealSignetPrototypeStatus {
        version: "v0.13".to_string(),
        feature_flag: "ldk-node-signet-runtime".to_string(),
        compiled_with_ldk: cfg!(feature = "ldk-node-signet-runtime"),
        target_networks: vec!["signet".to_string(), "testnet".to_string()],
        api_targets: vec![
            "ldk_node::Builder::new()".to_string(),
            "builder.set_network(Network::Signet/Testnet)".to_string(),
            "builder.set_storage_dir_path(...)".to_string(),
            "builder.set_chain_source_esplora(esplora_url, None)".to_string(),
            "builder.set_gossip_source_rgs(rgs_url)".to_string(),
            "builder.build()".to_string(),
            "node.start()".to_string(),
            "node.bolt12_payment() for future receive-offer generation".to_string(),
        ],
        files_touched: vec![
            "src-tauri/Cargo.toml".to_string(),
            "src-tauri/src/ldk_real_signet.rs".to_string(),
            "src-tauri/src/main.rs".to_string(),
            "src/main.tsx".to_string(),
            "BUILD_NOTES.md".to_string(),
        ],
        cargo_check_commands: vec![
            "cd src-tauri".to_string(),
            "cargo check".to_string(),
            "cargo check --features ldk-node-signet-runtime".to_string(),
            "RUST_LOG=debug cargo run --features ldk-node-signet-runtime".to_string(),
        ],
        mainnet_blocked: true,
        warning: "v0.13 adds the first real ldk-node Builder integration scaffold, but mainnet Lightning receiving remains locked until local cargo checks, signet receive tests, backup restore tests, and liquidity tests pass.".to_string(),
    }
}

pub fn start_preview(input: LdkRealSignetStartInput) -> LdkRealSignetStartPreview {
    let normalized_network = input.network.trim().to_lowercase();
    let mainnet_requested = normalized_network == "mainnet" || input.allow_mainnet;
    let mut blockers: Vec<String> = Vec::new();

    if mainnet_requested {
        blockers.push("Mainnet Lightning is intentionally blocked in v0.13.".to_string());
    }
    if normalized_network != "signet" && normalized_network != "testnet" {
        blockers.push("Choose signet or testnet only for the embedded LDK prototype.".to_string());
    }
    if !input.confirmed_signet_or_testnet_only {
        blockers.push("Confirm that this run uses only signet/testnet funds.".to_string());
    }
    if input.storage_dir.trim().is_empty() {
        blockers.push("Storage directory is required so LDK state can persist across restarts.".to_string());
    }
    if input.esplora_url.trim().is_empty() {
        blockers.push("Esplora URL is required for chain sync.".to_string());
    }
    if input.rgs_url.trim().is_empty() {
        blockers.push("Rapid Gossip Sync URL is required for practical routing data.".to_string());
    }

    LdkRealSignetStartPreview {
        safe_to_attempt_local_run: blockers.is_empty(),
        network: normalized_network,
        alias: if input.alias.trim().is_empty() { "CarlosK-Signet".to_string() } else { input.alias },
        storage_dir: input.storage_dir,
        feature_flag: "ldk-node-signet-runtime".to_string(),
        builder_steps: vec![
            "Create an ldk_node::Builder".to_string(),
            "Set Network::Signet or Network::Testnet".to_string(),
            "Set a persistent storage directory".to_string(),
            "Configure Esplora chain source".to_string(),
            "Configure Rapid Gossip Sync".to_string(),
            "Build node".to_string(),
            "Start node background tasks".to_string(),
            "Fetch an on-chain funding address for signet/testnet funds".to_string(),
            "Only after a channel/liquidity plan exists, generate a BOLT12 offer".to_string(),
        ],
        rust_api_scaffold: vec![
            "let mut builder = ldk_node::Builder::new();".to_string(),
            "builder.set_network(ldk_node::bitcoin::Network::Signet);".to_string(),
            "builder.set_storage_dir_path(storage_dir);".to_string(),
            "builder.set_chain_source_esplora(esplora_url, None);".to_string(),
            "builder.set_gossip_source_rgs(rgs_url);".to_string(),
            "let node = builder.build()?;".to_string(),
            "node.start()?;".to_string(),
            "let funding_address = node.onchain_payment().new_address()?;".to_string(),
            "// Next: node.bolt12_payment().receive(...) once local API signature is confirmed by cargo check.".to_string(),
        ],
        local_test_commands: vec![
            "cd src-tauri".to_string(),
            "cargo check --features ldk-node-signet-runtime".to_string(),
            "cargo run --features ldk-node-signet-runtime".to_string(),
            "Open CarlosK Wallet and run the v0.13 LDK signet start preview.".to_string(),
        ],
        expected_success_signals: vec![
            "Feature build compiles without touching mainnet.".to_string(),
            "Node storage directory is created.".to_string(),
            "LDK node can start and stop on signet/testnet.".to_string(),
            "App can display a signet/testnet funding address.".to_string(),
            "No OCEAN mainnet BOLT12 offer is generated yet.".to_string(),
        ],
        blockers,
        warning: "This is still a signet/testnet-only runtime scaffold. Do not use it for real OCEAN payouts until the app passes restore, channel, liquidity, and receive tests.".to_string(),
    }
}
