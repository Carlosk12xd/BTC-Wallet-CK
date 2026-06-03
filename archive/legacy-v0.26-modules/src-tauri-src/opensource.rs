use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenSourceStatus {
    pub project_name: String,
    pub license: String,
    pub repository_target: String,
    pub contribution_areas: Vec<String>,
    pub good_first_issues: Vec<String>,
    pub maintainer_rules: Vec<String>,
    pub security_policy: String,
}

pub fn status() -> OpenSourceStatus {
    OpenSourceStatus {
        project_name: "CarlosK Wallet".to_string(),
        license: "MIT License for v0.5 starter. Revisit dual MIT/Apache-2.0 before public launch if contributors prefer Rust ecosystem norms.".to_string(),
        repository_target: "GitHub open-source repository with issues, PR template, security policy, roadmap, and architecture docs.".to_string(),
        contribution_areas: vec![
            "Bitcoin wallet/address generation and BIP-322 signing".to_string(),
            "OCEAN pool setup wizard and Lightning payout message flow".to_string(),
            "LDK Node/BOLT12 offer generation and receive testing".to_string(),
            "Miner profiles for Avalon, FutureBit Apollo, Bitaxe, and other SHA-256 miners".to_string(),
            "Security reviews, backup UX, encrypted persistence, and recovery testing".to_string(),
            "Dashboard integrations for pool stats, payouts, and miner health".to_string(),
        ],
        good_first_issues: vec![
            "Add copy buttons for backup and pool config sections".to_string(),
            "Add validation for OCEAN worker labels and BTC address format before saving".to_string(),
            "Add unit tests for worker label sanitization and BOLT12 offer validation".to_string(),
            "Improve miner profile copy for Avalon Mini 3 and Apollo III".to_string(),
            "Create screenshots for README setup guide".to_string(),
        ],
        maintainer_rules: vec![
            "Never request or log seed phrases in GitHub issues.".to_string(),
            "No custodial server design without a separate legal/security review.".to_string(),
            "No production Lightning receiving until channel-state backup strategy is tested.".to_string(),
            "All wallet/key changes must include tests and explicit threat-model notes.".to_string(),
        ],
        security_policy: "Security issues should be reported privately first. Do not post seeds, private keys, signed payout secrets, wallet backups, or screenshots containing recoverable secrets.".to_string(),
    }
}
