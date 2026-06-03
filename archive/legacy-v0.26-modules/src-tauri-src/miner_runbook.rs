use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerRunbookInput {
    pub miner_name: String,
    pub pool_url: String,
    pub worker: String,
    pub password: String,
    pub has_bolt12_offer: bool,
    pub backup_confirmed: bool,
    pub pool_is_solo: bool,
    pub expected_hashrate_ths: f64,
    pub watts: f64,
    pub electricity_usd_per_kwh: f64,
    pub hashprice_usd_per_ph_day: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerRunbookReport {
    pub version: String,
    pub ready_for_24_7_pool_mining: bool,
    pub safe_mode: String,
    pub estimated_gross_usd_month: f64,
    pub estimated_power_cost_usd_month: f64,
    pub estimated_net_usd_month: f64,
    pub checklist: Vec<String>,
    pub warnings: Vec<String>,
    pub first_24h_steps: Vec<String>,
    pub daily_checks: Vec<String>,
    pub weekly_checks: Vec<String>,
    pub troubleshooting: Vec<String>,
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn looks_like_pool(url: &str) -> bool {
    let u = url.trim().to_ascii_lowercase();
    u.starts_with("stratum+tcp://") && u.contains(':')
}

fn looks_like_worker(worker: &str) -> bool {
    let w = worker.trim();
    (w.starts_with("bc1") || w.starts_with("tb1") || w.starts_with("bcrt1")) && w.contains('.')
}

pub fn build_runbook(input: MinerRunbookInput) -> MinerRunbookReport {
    let pool_ok = looks_like_pool(&input.pool_url);
    let worker_ok = looks_like_worker(&input.worker);
    let password_ok = !input.password.trim().is_empty();
    let hashrate_ok = input.expected_hashrate_ths > 0.0;
    let electricity_ok = input.electricity_usd_per_kwh >= 0.0 && input.electricity_usd_per_kwh <= 1.0;

    let gross_month = input.expected_hashrate_ths / 1000.0 * input.hashprice_usd_per_ph_day * 30.0;
    let power_month = input.watts / 1000.0 * 24.0 * 30.0 * input.electricity_usd_per_kwh;
    let net_month = gross_month - power_month;

    let ready = pool_ok
        && worker_ok
        && password_ok
        && hashrate_ok
        && electricity_ok
        && input.backup_confirmed
        && !input.pool_is_solo;

    let mut checklist = Vec::new();
    checklist.push(if pool_ok { "Pool URL has stratum+tcp:// and a port." } else { "Pool URL needs stratum+tcp://host:port." }.to_string());
    checklist.push(if worker_ok { "Worker looks like BTC_ADDRESS.WorkerName." } else { "Worker should be BTC_ADDRESS.WorkerName with no spaces." }.to_string());
    checklist.push(if password_ok { "Pool password is filled." } else { "Pool password is missing; use x for OCEAN/Braiins-style placeholders when appropriate." }.to_string());
    checklist.push(if input.backup_confirmed { "Mining wallet backup is confirmed." } else { "Backup is not confirmed. Do not mine to an unrecoverable wallet." }.to_string());
    checklist.push(if input.has_bolt12_offer { "BOLT12 payout offer is saved for OCEAN Lightning setup." } else { "No BOLT12 offer saved. OCEAN can still track work, but Lightning payouts are not configured." }.to_string());
    checklist.push(if input.pool_is_solo { "Current setup is solo/lottery mining." } else { "Current setup is pool-mining oriented." }.to_string());

    let mut warnings = Vec::new();
    if input.pool_is_solo {
        warnings.push("Solo mining is not a realistic monthly payout setup. Use a normal pool for real payouts.".to_string());
    }
    if !input.backup_confirmed {
        warnings.push("Backup confirmation is missing. This blocks the recommended 24/7 setup.".to_string());
    }
    if net_month < 0.0 {
        warnings.push("Estimated net after electricity is negative at this power rate. Treat the miner as Bitcoin stacking plus heat, not profit.".to_string());
    }
    if input.watts >= 700.0 {
        warnings.push("This miner behaves like a space heater. Keep airflow open and avoid cheap extension cords or overloaded power strips.".to_string());
    }
    if warnings.is_empty() {
        warnings.push("Runbook looks ready for a controlled 24/7 pool-mining test. Keep monitoring heat, fan, rejects, and pool-side hashrate.".to_string());
    }

    MinerRunbookReport {
        version: "0.20.0".to_string(),
        ready_for_24_7_pool_mining: ready,
        safe_mode: if ready { "Ready for 24/7 pool-mining trial".to_string() } else { "Needs fixes before unattended 24/7 use".to_string() },
        estimated_gross_usd_month: round2(gross_month),
        estimated_power_cost_usd_month: round2(power_month),
        estimated_net_usd_month: round2(net_month),
        checklist,
        warnings,
        first_24h_steps: vec![
            "Run the miner in a ventilated location for at least 30 minutes before leaving it unattended.".to_string(),
            "Confirm miner-side hashrate is nonzero and close to the expected range after warmup.".to_string(),
            "Confirm pool-side worker appears online and accepted shares are increasing.".to_string(),
            "Check wall plug, cable, and miner temperature after 1 hour and again after 4 hours.".to_string(),
            "If using OCEAN Lightning, confirm BOLT12 pairing and signed message are accepted before expecting Lightning payouts.".to_string(),
        ],
        daily_checks: vec![
            "Worker online on the pool dashboard.".to_string(),
            "Hashrate is near expected average over several hours, not just the 5-minute number.".to_string(),
            "Rejected/stale shares are low.".to_string(),
            "Room heat and fan noise are acceptable.".to_string(),
        ],
        weekly_checks: vec![
            "Verify payout destination and pool settings did not change.".to_string(),
            "Compare estimated gross earnings to pool credit.".to_string(),
            "Check power cost against real kWh usage if possible.".to_string(),
            "Update the payout planner with current hashprice and BTC price.".to_string(),
        ],
        troubleshooting: vec![
            "If miner shows hashrate but pool does not, re-check pool URL, worker, password, DNS, and firewall/router settings.".to_string(),
            "If hashrate is low, wait for warmup, then check power mode, temperature, fan setting, and firmware.".to_string(),
            "If OCEAN Lightning does not pay, verify the BOLT12 offer, signed message, and wallet inbound liquidity/online status.".to_string(),
            "If the miner reboots or drops, test Ethernet, remove weak extension cords, and reduce power mode.".to_string(),
        ],
    }
}
