use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerProfile {
    pub id: String,
    pub name: String,
    pub algorithm: String,
    pub default_worker_label: String,
    pub recommended_pool_url: String,
    pub recommended_password: String,
    pub expected_hashrate: String,
    pub notes: Vec<String>,
    pub supports_ocean: bool,
}

pub fn defaults() -> Vec<MinerProfile> {
    vec![
        MinerProfile {
            id: "avalon-mini-3".to_string(),
            name: "Canaan Avalon Mini 3".to_string(),
            algorithm: "SHA-256 / Bitcoin".to_string(),
            default_worker_label: "EpicAvalon".to_string(),
            recommended_pool_url: "stratum+tcp://mine.ocean.xyz:3334".to_string(),
            recommended_password: "x".to_string(),
            expected_hashrate: "37.5 TH/s advertised; 33-43 TH/s is a normal observed range while tuning".to_string(),
            supports_ocean: true,
            notes: vec![
                "Best current target for your first OCEAN configuration.".to_string(),
                "Use a BTC address.worker label as username, not your BOLT12 offer.".to_string(),
                "Keep airflow open because it behaves like a small space heater.".to_string(),
            ],
        },
        MinerProfile {
            id: "futurebit-apollo-iii".to_string(),
            name: "FutureBit Apollo III Full Node 2TB".to_string(),
            algorithm: "SHA-256 / Bitcoin".to_string(),
            default_worker_label: "CarlosApollo".to_string(),
            recommended_pool_url: "stratum+tcp://mine.ocean.xyz:3334".to_string(),
            recommended_password: "x".to_string(),
            expected_hashrate: "10-12 TH/s Eco; 16-18 TH/s Turbo".to_string(),
            supports_ocean: true,
            notes: vec![
                "Can run your Bitcoin full node separately from mining.".to_string(),
                "Good future target for DATUM/OCEAN sovereignty workflows.".to_string(),
                "Eco mode should be the quiet default.".to_string(),
            ],
        },
        MinerProfile {
            id: "goldshell-mini-doge-iii".to_string(),
            name: "Goldshell Mini DOGE III".to_string(),
            algorithm: "Scrypt / Litecoin + Dogecoin".to_string(),
            default_worker_label: "MiniDoge3".to_string(),
            recommended_pool_url: "Not supported by OCEAN BTC pool".to_string(),
            recommended_password: "Depends on the Scrypt pool".to_string(),
            expected_hashrate: "700 MH/s advertised".to_string(),
            supports_ocean: false,
            notes: vec![
                "This is a Scrypt miner, so it does not mine on OCEAN's Bitcoin/SHA-256 pool.".to_string(),
                "Use a Litecoin/Dogecoin merged-mining pool instead.".to_string(),
                "This app will track it later, but v0.3 focuses on BTC/OCEAN signing.".to_string(),
            ],
        },
        MinerProfile {
            id: "gold-nugget-nerdminer".to_string(),
            name: "Gold Nugget NerdMiner".to_string(),
            algorithm: "SHA-256 / Bitcoin lottery".to_string(),
            default_worker_label: "GoldNugget".to_string(),
            recommended_pool_url: "Use solo/lottery pools only".to_string(),
            recommended_password: "x".to_string(),
            expected_hashrate: "KH/s to low MH/s range; not meaningful for income".to_string(),
            supports_ocean: false,
            notes: vec![
                "Treat it as a desk gadget and lottery miner.".to_string(),
                "It is not useful for normal monthly payouts.".to_string(),
            ],
        },
    ]
}
