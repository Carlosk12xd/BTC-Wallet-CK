use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanDashboardInput {
    pub bitcoin_address: String,
    pub worker_label: String,
    pub pool_url: String,
    pub expected_hashrate_ths: f64,
    pub observed_miner_hashrate_ths: f64,
    pub pool_reported_hashrate_ths: f64,
    pub worker_seen: bool,
    pub accepted_work_seen: bool,
    pub unpaid_credit_seen: bool,
    pub payout_seen: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanDashboardReport {
    pub version: String,
    pub ocean_stats_url: String,
    pub worker_string: String,
    pub pool_url_ok: bool,
    pub worker_seen: bool,
    pub accepted_work_seen: bool,
    pub unpaid_credit_seen: bool,
    pub payout_seen: bool,
    pub hashrate_gap_percent: f64,
    pub dashboard_ready: bool,
    pub payout_path_confirmed: bool,
    pub next_actions: Vec<String>,
    pub expected_pool_signals: Vec<String>,
    pub evidence_to_capture: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn build_ocean_dashboard_report(input: OceanDashboardInput) -> OceanDashboardReport {
    let address = input.bitcoin_address.trim().to_string();
    let clean_label = sanitize_worker_label(&input.worker_label);
    let worker_string = if address.is_empty() {
        String::from("missing-address")
    } else if clean_label.is_empty() {
        address.clone()
    } else {
        format!("{address}.{clean_label}")
    };

    let ocean_stats_url = if address.is_empty() {
        String::from("https://ocean.xyz/stats")
    } else {
        format!("https://ocean.xyz/stats/{address}")
    };

    let pool_url_ok = input.pool_url.contains("mine.ocean.xyz") && input.pool_url.contains("3334");
    let gap = hashrate_gap_percent(input.observed_miner_hashrate_ths, input.pool_reported_hashrate_ths);

    let dashboard_ready = pool_url_ok
        && !address.is_empty()
        && input.worker_seen
        && input.accepted_work_seen
        && input.pool_reported_hashrate_ths > 0.0;

    let payout_path_confirmed = dashboard_ready && (input.unpaid_credit_seen || input.payout_seen);

    let mut next_actions = Vec::new();
    if address.is_empty() {
        next_actions.push("Create or restore a Bitcoin mining wallet before checking OCEAN stats.".into());
    }
    if !pool_url_ok {
        next_actions.push("Confirm the miner is pointed at stratum+tcp://mine.ocean.xyz:3334.".into());
    }
    if !input.worker_seen {
        next_actions.push("Open the OCEAN stats page for the mining address and wait until the worker appears online.".into());
    }
    if !input.accepted_work_seen {
        next_actions.push("Wait for accepted work/proofs to appear. If they do not appear after 30 minutes, check worker spelling and pool URL.".into());
    }
    if input.pool_reported_hashrate_ths <= 0.0 {
        next_actions.push("Wait for OCEAN's rolling hashrate window to populate. Pool-side hashrate can lag local miner hashrate.".into());
    }
    if gap > 35.0 && input.pool_reported_hashrate_ths > 0.0 {
        next_actions.push("Investigate stale/rejected work, Wi-Fi issues, or miner power mode because pool hashrate is far below local hashrate.".into());
    }
    if !input.unpaid_credit_seen && !input.payout_seen {
        next_actions.push("Keep mining until unpaid credit or payout evidence appears. Worker online alone proves mining, not payout readiness.".into());
    }
    if next_actions.is_empty() {
        next_actions.push("Continue mining and capture a screenshot of worker online, hashrate, and credit/payout evidence for the supervised MVP test.".into());
    }

    let mut warnings = Vec::new();
    warnings.push("This verifier does not query OCEAN automatically yet. It records manual evidence from the dashboard.".into());
    if input.payout_seen {
        warnings.push("Payout seen. Verify the received destination matches the intended BOLT12/on-chain payout path before calling the flow complete.".into());
    } else if input.unpaid_credit_seen {
        warnings.push("Unpaid credit seen. This proves the pool has accounted work, but it is not the same as a wallet payout.".into());
    } else {
        warnings.push("No credit or payout evidence was marked. Keep the miner online before treating the setup as end-to-end tested.".into());
    }

    OceanDashboardReport {
        version: "0.26.0".into(),
        ocean_stats_url,
        worker_string,
        pool_url_ok,
        worker_seen: input.worker_seen,
        accepted_work_seen: input.accepted_work_seen,
        unpaid_credit_seen: input.unpaid_credit_seen,
        payout_seen: input.payout_seen,
        hashrate_gap_percent: round2(gap),
        dashboard_ready,
        payout_path_confirmed,
        next_actions,
        expected_pool_signals: vec![
            "Worker appears online under the Bitcoin mining address.".into(),
            "Pool-side hashrate becomes non-zero after the rolling window updates.".into(),
            "Accepted work/proofs appear without sustained reject/stale errors.".into(),
            "Unpaid credit increases after OCEAN accounts pool work.".into(),
            "Lightning/on-chain payout evidence appears only after the configured payout path is active and eligible.".into(),
        ],
        evidence_to_capture: vec![
            "OCEAN stats page URL for the mining address.".into(),
            "Worker name shown online.".into(),
            "Local miner hashrate screenshot.".into(),
            "Pool-side hashrate screenshot.".into(),
            "Unpaid credit or payout history screenshot with secrets redacted.".into(),
        ],
        warnings,
    }
}

fn sanitize_worker_label(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

fn hashrate_gap_percent(local: f64, pool: f64) -> f64 {
    if local <= 0.0 || pool <= 0.0 {
        return 0.0;
    }
    ((local - pool).abs() / local) * 100.0
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
