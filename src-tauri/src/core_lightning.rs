use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningWalletInfo {
    pub alias: String,
    pub network: String,
    pub status: String,
    pub can_receive_bolt12_in_app: bool,
    pub warning: String,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bolt12OfferInfo {
    pub offer: String,
    pub source: String,
    pub status: String,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreLightningConnectionInput {
    pub node_url: String,
    pub rune: String,
    pub network: String,
    pub alias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreLightningConnectionStatus {
    pub online: bool,
    pub backend: String,
    pub node_url: String,
    pub node_alias: String,
    pub node_id: String,
    pub network: String,
    pub status: String,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanBolt12Request {
    pub node_url: String,
    pub rune: String,
    pub label: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OceanSetupPlan {
    pub btc_address: String,
    pub bolt12_offer: String,
    pub expected_description: String,
    pub ocean_config_url: String,
    pub ocean_stats_url: String,
    pub message_to_sign_source: String,
    pub steps: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn create_lightning_wallet(alias: &str) -> LightningWalletInfo {
    let clean_alias = if alias.trim().is_empty() {
        "CarlosK Lightning".to_string()
    } else {
        alias.trim().to_string()
    };

    LightningWalletInfo {
        alias: clean_alias,
        network: "online Core Lightning connector + future embedded LDK".to_string(),
        status: "Lightning profile created. v0.99.6 can now connect to an online Core Lightning node to generate a real OCEAN BOLT12 offer.".to_string(),
        can_receive_bolt12_in_app: true,
        warning: "The app can generate a real BOLT12 offer through your connected node. The node must stay online and have inbound liquidity to receive OCEAN payouts.".to_string(),
        next_steps: vec![
            "Connect to an online Core Lightning node with REST/Rune access.".to_string(),
            "Generate the OCEAN BOLT12 offer with amount=any and the required OCEAN description.".to_string(),
            "Paste the offer into OCEAN, then sign OCEAN's generated message with this BTC wallet.".to_string(),
            "Keep the Lightning node online with inbound liquidity so payouts can arrive.".to_string(),
        ],
    }
}

pub fn validate_bolt12_offer(offer: &str) -> Result<Bolt12OfferInfo> {
    let trimmed = offer.trim();

    if trimmed.is_empty() {
        return Err(anyhow!("Paste a BOLT12 offer first"));
    }

    if !looks_like_bolt12_offer(trimmed) {
        return Err(anyhow!("This does not look like a BOLT12 offer. BOLT12 offers usually start with lno1..."));
    }

    Ok(Bolt12OfferInfo {
        offer: trimmed.to_string(),
        source: "BOLT12 offer".to_string(),
        status: "BOLT12 offer saved.".to_string(),
        warning: "The wallet cannot fully decode and prove inbound liquidity from the offer alone. The Lightning node must stay online and have inbound liquidity.".to_string(),
    })
}

pub fn check_core_lightning_node(input: CoreLightningConnectionInput) -> Result<CoreLightningConnectionStatus> {
    let base = normalize_node_url(&input.node_url)?;
    let mut last_error = String::new();

    for (url, body) in getinfo_requests(&base) {
        match post_json(&url, &input.rune, body) {
            Ok(value) => {
                let alias = find_string_field(&value, &["alias"]).unwrap_or_else(|| input.alias.trim().to_string());
                let node_id = find_string_field(&value, &["id", "node_id", "nodeId"]).unwrap_or_default();
                return Ok(CoreLightningConnectionStatus {
                    online: true,
                    backend: "Core Lightning REST/JSON-RPC".to_string(),
                    node_url: base,
                    node_alias: if alias.is_empty() { "Core Lightning node".to_string() } else { alias },
                    node_id,
                    network: input.network,
                    status: "Connected to the online Lightning node. You can now request an OCEAN BOLT12 offer from this node.".to_string(),
                    warning: "This confirms the node answered, not that it has inbound liquidity. OCEAN payouts still require the node to stay online with inbound liquidity.".to_string(),
                });
            }
            Err(err) => last_error = err.to_string(),
        }
    }

    Err(anyhow!("Could not connect to the Core Lightning node. Last error: {last_error}. Check URL, REST plugin, rune/token, and whether the node is reachable from this computer."))
}

pub fn create_ocean_bolt12_offer(input: OceanBolt12Request, btc_address: &str) -> Result<Bolt12OfferInfo> {
    let base = normalize_node_url(&input.node_url)?;
    let address = btc_address.trim();
    if address.is_empty() {
        return Err(anyhow!("Create or unlock the BTC wallet first so the OCEAN description can include the mining address"));
    }

    let amount = if input.amount.trim().is_empty() { "any" } else { input.amount.trim() };
    if amount != "any" {
        return Err(anyhow!("OCEAN payout offers should use amount=any. Leave the amount as any."));
    }

    let description = ocean_description(address);
    let label = if input.label.trim().is_empty() {
        format!("carlosk-ocean-{address}")
    } else {
        input.label.trim().to_string()
    };

    let mut last_error = String::new();
    for (url, body) in offer_requests(&base, amount, &description, &label) {
        match post_json(&url, &input.rune, body) {
            Ok(value) => {
                if let Some(offer) = extract_bolt12_offer(&value) {
                    let info = Bolt12OfferInfo {
                        offer,
                        source: "Online Core Lightning node".to_string(),
                        status: "Real BOLT12 offer generated by your connected Lightning node for OCEAN payouts.".to_string(),
                        warning: format!("Generated with description: '{description}'. Keep the node online with inbound liquidity or OCEAN Lightning payouts can fail."),
                    };
                    return validate_generated_offer(info);
                }
                last_error = format!("Node responded but no BOLT12 offer field was found. Response: {value}");
            }
            Err(err) => last_error = err.to_string(),
        }
    }

    Err(anyhow!("Could not generate BOLT12 offer through the connected node. Last error: {last_error}"))
}

pub fn create_in_app_bolt12_offer() -> Result<Bolt12OfferInfo> {
    Err(anyhow!(
        "Use the online Lightning node connector in v0.99.6. A fully embedded LDK node is still separate work because Lightning channel state backup and inbound liquidity must be solved before mainnet."
    ))
}

pub fn build_ocean_setup_plan(btc_address: &str, bolt12_offer: &str) -> Result<OceanSetupPlan> {
    let address = btc_address.trim();
    let offer = bolt12_offer.trim();
    if address.is_empty() {
        return Err(anyhow!("Create or unlock the BTC wallet first"));
    }
    if !looks_like_bolt12_offer(offer) {
        return Err(anyhow!("Generate or paste a valid BOLT12 offer first"));
    }

    Ok(OceanSetupPlan {
        btc_address: address.to_string(),
        bolt12_offer: offer.to_string(),
        expected_description: ocean_description(address),
        ocean_config_url: format!("https://ocean.xyz/config/{address}"),
        ocean_stats_url: format!("https://ocean.xyz/stats/{address}"),
        message_to_sign_source: "OCEAN website configuration form".to_string(),
        steps: vec![
            "Open the OCEAN configuration page for the BTC mining address.".to_string(),
            "Paste the BOLT12 offer generated by this wallet's online Lightning node connector.".to_string(),
            "Use amount=any and description exactly matching the OCEAN payout description shown below.".to_string(),
            "Let OCEAN generate the unsigned configuration message.".to_string(),
            "Paste that unsigned message back into this wallet and sign it with the BTC wallet.".to_string(),
            "Paste the signature into OCEAN to activate Lightning payouts.".to_string(),
        ],
        warnings: vec![
            "The Lightning node must remain online for OCEAN to pay it.".to_string(),
            "The Lightning node needs inbound liquidity. A valid offer alone does not guarantee receive capacity.".to_string(),
            "Do not paste wallet seed words, private keys, or backup passphrases into OCEAN or any website.".to_string(),
        ],
    })
}

fn validate_generated_offer(info: Bolt12OfferInfo) -> Result<Bolt12OfferInfo> {
    if !looks_like_bolt12_offer(&info.offer) {
        return Err(anyhow!("Lightning node returned a value, but it does not look like a BOLT12 lno1 offer"));
    }
    Ok(info)
}

fn looks_like_bolt12_offer(value: &str) -> bool {
    let trimmed = value.trim().to_lowercase();
    trimmed.starts_with("lno1") && trimmed.len() >= 60
}

fn ocean_description(address: &str) -> String {
    format!("OCEAN Payouts for {address}")
}

fn normalize_node_url(url: &str) -> Result<String> {
    let trimmed = url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(anyhow!("Enter your Core Lightning REST URL first, for example http://127.0.0.1:3010"));
    }
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err(anyhow!("Node URL must start with http:// or https://"));
    }
    Ok(trimmed.to_string())
}

fn getinfo_requests(base: &str) -> Vec<(String, Value)> {
    vec![
        (format!("{base}/v1/getinfo"), json!({})),
        (format!("{base}/getinfo"), json!({})),
        (base.to_string(), json!({"jsonrpc":"2.0","id":"carlosk-wallet-getinfo","method":"getinfo","params":{}})),
    ]
}

fn offer_requests(base: &str, amount: &str, description: &str, label: &str) -> Vec<(String, Value)> {
    vec![
        (format!("{base}/v1/offer"), json!({"amount": amount, "description": description, "label": label})),
        (format!("{base}/offer"), json!({"amount": amount, "description": description, "label": label})),
        (base.to_string(), json!({"jsonrpc":"2.0","id":"carlosk-wallet-offer","method":"offer","params":{"amount": amount, "description": description, "label": label}})),
        (base.to_string(), json!({"jsonrpc":"2.0","id":"carlosk-wallet-offer-array","method":"offer","params":[amount, description, label]})),
    ]
}

fn post_json(url: &str, rune: &str, body: Value) -> Result<Value> {
    let mut request = ureq::post(url)
        .set("Content-Type", "application/json")
        .set("Accept", "application/json");

    let token = rune.trim();
    if !token.is_empty() {
        let auth = if token.to_lowercase().starts_with("bearer ") || token.to_lowercase().starts_with("rune ") {
            token.to_string()
        } else {
            format!("Rune {token}")
        };
        request = request.set("Authorization", &auth).set("Rune", token);
    }

    let response = request
        .send_string(&body.to_string())
        .map_err(|err| anyhow!("POST {url} failed: {err}"))?;

    let text = response
        .into_string()
        .map_err(|err| anyhow!("Could not read response from {url}: {err}"))?;

    serde_json::from_str(&text).map_err(|err| anyhow!("Response from {url} was not JSON: {err}. Body: {text}"))
}

fn extract_bolt12_offer(value: &Value) -> Option<String> {
    match value {
        Value::String(s) if looks_like_bolt12_offer(s) => Some(s.clone()),
        Value::Object(map) => {
            for key in ["bolt12", "offer", "bolt12_offer", "bolt12Offer"] {
                if let Some(Value::String(s)) = map.get(key) {
                    if looks_like_bolt12_offer(s) {
                        return Some(s.clone());
                    }
                }
            }
            for child in map.values() {
                if let Some(found) = extract_bolt12_offer(child) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(values) => values.iter().find_map(extract_bolt12_offer),
        _ => None,
    }
}

fn find_string_field(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for key in keys {
                if let Some(Value::String(s)) = map.get(*key) {
                    return Some(s.clone());
                }
            }
            for child in map.values() {
                if let Some(found) = find_string_field(child, keys) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(values) => values.iter().find_map(|v| find_string_field(v, keys)),
        _ => None,
    }
}
