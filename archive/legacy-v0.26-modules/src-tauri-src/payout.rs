use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutPoolPreset {
    pub id: String,
    pub name: String,
    pub payout_method: String,
    pub pool_fee_percent: f64,
    pub payout_threshold_btc: Option<f64>,
    pub lightning_friendly: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutPlannerInput {
    pub hashrate_ths: f64,
    pub watts: f64,
    pub electricity_usd_per_kwh: f64,
    pub hashprice_usd_per_ph_day: f64,
    pub btc_usd: f64,
    pub pool_fee_percent: f64,
    pub payout_threshold_btc: Option<f64>,
    pub days_per_month: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutEstimate {
    pub hashrate_phs: f64,
    pub gross_usd_day: f64,
    pub pool_fee_usd_day: f64,
    pub electricity_usd_day: f64,
    pub net_usd_day: f64,
    pub gross_usd_month: f64,
    pub pool_fee_usd_month: f64,
    pub electricity_usd_month: f64,
    pub net_usd_month: f64,
    pub gross_btc_day: f64,
    pub after_pool_fee_btc_day: f64,
    pub payout_threshold_btc: Option<f64>,
    pub estimated_days_to_threshold: Option<f64>,
    pub estimated_months_to_threshold: Option<f64>,
    pub recommendation: String,
    pub warnings: Vec<String>,
}

pub fn presets() -> Vec<PayoutPoolPreset> {
    vec![
        PayoutPoolPreset {
            id: "ocean-standard-onchain".to_string(),
            name: "OCEAN standard on-chain".to_string(),
            payout_method: "TIDES / on-chain threshold".to_string(),
            pool_fee_percent: 2.0,
            payout_threshold_btc: Some(0.010_485_76),
            lightning_friendly: false,
            notes: vec![
                "Highest-sovereignty BTC pool choice, but on-chain threshold is too high for most small home miners.".to_string(),
                "Use BOLT12 Lightning if you want frequent small payouts.".to_string(),
            ],
        },
        PayoutPoolPreset {
            id: "ocean-bolt12".to_string(),
            name: "OCEAN with BOLT12 Lightning".to_string(),
            payout_method: "TIDES / BOLT12 Lightning payouts".to_string(),
            pool_fee_percent: 2.0,
            payout_threshold_btc: None,
            lightning_friendly: true,
            notes: vec![
                "Best match for CarlosK Wallet once the BOLT12 receive wallet is production-ready.".to_string(),
                "Requires a valid BOLT12 offer and a signed OCEAN payout configuration message.".to_string(),
            ],
        },
        PayoutPoolPreset {
            id: "ocean-datum-bolt12".to_string(),
            name: "OCEAN DATUM with BOLT12".to_string(),
            payout_method: "DATUM / BOLT12 Lightning payouts".to_string(),
            pool_fee_percent: 1.0,
            payout_threshold_btc: None,
            lightning_friendly: true,
            notes: vec![
                "Future advanced target: lower OCEAN fee, more decentralized block template flow.".to_string(),
                "Keep this behind an advanced-user warning until DATUM setup is documented and tested.".to_string(),
            ],
        },
        PayoutPoolPreset {
            id: "braiins-onchain".to_string(),
            name: "Braiins Pool on-chain".to_string(),
            payout_method: "FPPS / lower on-chain threshold".to_string(),
            pool_fee_percent: 2.5,
            payout_threshold_btc: Some(0.000_2),
            lightning_friendly: true,
            notes: vec![
                "Practical fallback for small miners who want to see on-chain payouts sooner.".to_string(),
                "Not the lowest fee, but easier than OCEAN on-chain for small home hashrate.".to_string(),
            ],
        },
        PayoutPoolPreset {
            id: "viabtc-pplns".to_string(),
            name: "ViaBTC PPLNS".to_string(),
            payout_method: "PPLNS / variable rewards".to_string(),
            pool_fee_percent: 2.0,
            payout_threshold_btc: Some(0.001),
            lightning_friendly: false,
            notes: vec![
                "Traditional dashboard pool fallback.".to_string(),
                "Verify the current threshold and payout method in the pool account before using real funds.".to_string(),
            ],
        },
    ]
}

pub fn estimate(input: PayoutPlannerInput) -> Result<PayoutEstimate, String> {
    if input.hashrate_ths <= 0.0 {
        return Err("Hashrate must be greater than 0 TH/s".to_string());
    }
    if input.watts < 0.0 {
        return Err("Watts cannot be negative".to_string());
    }
    if input.hashprice_usd_per_ph_day <= 0.0 {
        return Err("Hashprice must be greater than 0".to_string());
    }
    if input.btc_usd <= 0.0 {
        return Err("BTC price must be greater than 0".to_string());
    }
    if input.days_per_month <= 0.0 {
        return Err("Days per month must be greater than 0".to_string());
    }
    if !(0.0..=100.0).contains(&input.pool_fee_percent) {
        return Err("Pool fee must be between 0 and 100 percent".to_string());
    }

    let hashrate_phs = input.hashrate_ths / 1000.0;
    let gross_usd_day = hashrate_phs * input.hashprice_usd_per_ph_day;
    let pool_fee_usd_day = gross_usd_day * (input.pool_fee_percent / 100.0);
    let electricity_usd_day = (input.watts / 1000.0) * 24.0 * input.electricity_usd_per_kwh;
    let net_usd_day = gross_usd_day - pool_fee_usd_day - electricity_usd_day;

    let gross_btc_day = gross_usd_day / input.btc_usd;
    let after_pool_fee_btc_day = gross_btc_day * (1.0 - input.pool_fee_percent / 100.0);

    let estimated_days_to_threshold = input.payout_threshold_btc.and_then(|threshold| {
        if threshold > 0.0 && after_pool_fee_btc_day > 0.0 {
            Some(threshold / after_pool_fee_btc_day)
        } else {
            None
        }
    });
    let estimated_months_to_threshold = estimated_days_to_threshold.map(|days| days / input.days_per_month);

    let mut warnings = vec![
        "This is an estimate, not a guarantee. Bitcoin price, hashprice, difficulty, fees, uptime, and pool luck change constantly.".to_string(),
        "Always verify current pool fees and payout thresholds before mining real hashrate.".to_string(),
    ];

    if net_usd_day < 0.0 {
        warnings.push("Estimated net is negative at this electricity rate. You may still mine to stack BTC or use the heat, but it is not cash-profitable.".to_string());
    }
    if input.payout_threshold_btc.is_none() {
        warnings.push("Lightning/BOLT12 payouts do not use this on-chain threshold planner. Confirm your wallet can receive BOLT12 payments before relying on it.".to_string());
    }

    let recommendation = match (net_usd_day >= 0.0, estimated_months_to_threshold) {
        (true, Some(months)) if months <= 1.0 => "Good small-miner payout cadence if the pool settings are correct.".to_string(),
        (true, Some(months)) if months <= 3.0 => "Profitable estimate, but payout cadence may be slow. Lightning payouts may feel better.".to_string(),
        (true, Some(_)) => "Profitable estimate, but the on-chain payout threshold is too high for frequent payouts.".to_string(),
        (true, None) => "Profitable estimate. For Lightning pools, focus on BOLT12 compatibility, liquidity, and backup safety.".to_string(),
        (false, Some(_)) => "Not profitable at this electricity rate, but you may still mine for BTC accumulation or heat reuse.".to_string(),
        (false, None) => "Not profitable at this electricity rate. Use Lightning only if you still want small BTC payouts without waiting for on-chain thresholds.".to_string(),
    };

    Ok(PayoutEstimate {
        hashrate_phs,
        gross_usd_day,
        pool_fee_usd_day,
        electricity_usd_day,
        net_usd_day,
        gross_usd_month: gross_usd_day * input.days_per_month,
        pool_fee_usd_month: pool_fee_usd_day * input.days_per_month,
        electricity_usd_month: electricity_usd_day * input.days_per_month,
        net_usd_month: net_usd_day * input.days_per_month,
        gross_btc_day,
        after_pool_fee_btc_day,
        payout_threshold_btc: input.payout_threshold_btc,
        estimated_days_to_threshold,
        estimated_months_to_threshold,
        recommendation,
        warnings,
    })
}
