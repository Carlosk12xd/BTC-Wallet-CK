use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdkDependencyManifest {
    pub version: String,
    pub branch_name: String,
    pub cargo_feature: String,
    pub pinned_dependency_candidates: Vec<String>,
    pub cargo_commands: Vec<String>,
    pub implementation_notes: Vec<String>,
    pub mainnet_lock_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignetFundingPlanInput {
    pub receive_test_sats: u64,
    pub channel_capacity_sats: u64,
    pub reserve_percent: f64,
    pub peer_pubkey: String,
    pub peer_address: String,
    pub user_understands_signet_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignetFundingPlan {
    pub network: String,
    pub safe_to_continue: bool,
    pub requested_receive_test_sats: u64,
    pub channel_capacity_sats: u64,
    pub estimated_channel_reserve_sats: u64,
    pub estimated_spendable_after_reserve_sats: u64,
    pub peer_summary: String,
    pub steps: Vec<String>,
    pub warnings: Vec<String>,
    pub release_gates_before_mainnet: Vec<String>,
}

pub fn dependency_manifest() -> LdkDependencyManifest {
    LdkDependencyManifest {
        version: "v0.10".to_string(),
        branch_name: "feature/ldk-node-signet".to_string(),
        cargo_feature: "ldk-node-signet".to_string(),
        pinned_dependency_candidates: vec![
            "ldk-node = pin exact version after local Cargo check".to_string(),
            "lightning = pin to the ldk-node-compatible version".to_string(),
            "bitcoin = keep compatible with bdk_wallet and ldk-node".to_string(),
            "bdk_wallet = keep wallet layer separate from Lightning node storage".to_string(),
        ],
        cargo_commands: vec![
            "git checkout -b feature/ldk-node-signet".to_string(),
            "cargo add ldk-node --optional".to_string(),
            "cargo check --features ldk-node-signet".to_string(),
            "npm run vite".to_string(),
            "npm run dev".to_string(),
        ],
        implementation_notes: vec![
            "Keep the default build dependency-light until the experimental branch compiles on macOS/Windows/Linux.".to_string(),
            "Initialize ldk-node only on signet/testnet first; reject mainnet in the command layer.".to_string(),
            "Store Lightning node data in a separate app directory from the Bitcoin mining identity wallet.".to_string(),
            "Do not expose an OCEAN-compatible mainnet BOLT12 offer until backup, restore, channel close, and liquidity flows are tested.".to_string(),
        ],
        mainnet_lock_reason: "Lightning receiving can lose funds if channel backups, force-close handling, or liquidity are wrong. v0.10 prepares the real signet branch without enabling mainnet receive.".to_string(),
    }
}

pub fn funding_plan(input: SignetFundingPlanInput) -> SignetFundingPlan {
    let reserve_percent = if input.reserve_percent.is_finite() {
        input.reserve_percent.clamp(1.0, 20.0)
    } else {
        1.0
    };
    let reserve = ((input.channel_capacity_sats as f64) * (reserve_percent / 100.0)).round() as u64;
    let spendable = input.channel_capacity_sats.saturating_sub(reserve);
    let required = input.receive_test_sats.saturating_add(reserve).saturating_add(5_000);
    let has_peer = !input.peer_pubkey.trim().is_empty() && !input.peer_address.trim().is_empty();

    let mut warnings = vec![
        "Signet coins have no market value. Never send mainnet BTC to a signet/testnet address.".to_string(),
        "This plan does not open a real channel yet. It is the checklist for the next experimental branch.".to_string(),
        "Inbound liquidity is required before OCEAN-style Lightning payouts can be received reliably.".to_string(),
    ];
    if !input.user_understands_signet_only {
        warnings.push("You must explicitly confirm this is signet-only before running the real node workflow.".to_string());
    }
    if !has_peer {
        warnings.push("No Lightning peer was entered. The real ldk-node branch will need a signet peer or LSP strategy.".to_string());
    }
    if input.channel_capacity_sats < required {
        warnings.push(format!(
            "Channel capacity is probably too small. Suggested minimum is about {required} sats for this test target."
        ));
    }

    SignetFundingPlan {
        network: "signet".to_string(),
        safe_to_continue: input.user_understands_signet_only && input.channel_capacity_sats >= required,
        requested_receive_test_sats: input.receive_test_sats,
        channel_capacity_sats: input.channel_capacity_sats,
        estimated_channel_reserve_sats: reserve,
        estimated_spendable_after_reserve_sats: spendable,
        peer_summary: if has_peer {
            format!("{}@{}", input.peer_pubkey.trim(), input.peer_address.trim())
        } else {
            "No peer configured yet".to_string()
        },
        steps: vec![
            "Create or load the signet-only Lightning node data directory.".to_string(),
            "Get signet coins from a trusted faucet or local signet setup.".to_string(),
            "Fund the ldk-node on-chain signet wallet.".to_string(),
            "Connect to a signet peer or LSP test endpoint.".to_string(),
            "Open a signet channel with enough capacity for the receive test.".to_string(),
            "Generate a real signet BOLT12 offer.".to_string(),
            "Pay the offer from another signet Lightning wallet and verify receipt.".to_string(),
            "Delete and restore from backup in a test environment before enabling any mainnet path.".to_string(),
        ],
        warnings,
        release_gates_before_mainnet: vec![
            "Real signet BOLT12 offer generation works.".to_string(),
            "A signet payment can be received and displayed in the app.".to_string(),
            "Channel backup can be restored on a clean machine.".to_string(),
            "Force-close and rescan recovery are documented and tested.".to_string(),
            "OCEAN pairing message still verifies the exact BOLT12 offer before signing.".to_string(),
            "Mainnet receive requires a separate explicit user opt-in and small-limit beta.".to_string(),
        ],
    }
}
