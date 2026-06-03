use anyhow::{anyhow, Result};
use bitcoin::{Address, Network};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBookEntry {
    pub id: String,
    pub label: String,
    pub address: String,
    pub network: String,
    pub notes: String,
    pub created_at_unix: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveAddressBookEntryInput {
    pub label: String,
    pub address: String,
    pub network: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBookResult {
    pub entries: Vec<AddressBookEntry>,
    pub path: String,
    pub status: String,
    pub warning: String,
}

pub fn list_address_book() -> Result<AddressBookResult> {
    let entries = load_entries()?;
    result(entries, "Address book loaded.")
}

pub fn save_address_book_entry(input: SaveAddressBookEntryInput) -> Result<AddressBookResult> {
    let label = input.label.trim();
    let address = input.address.trim();
    let notes = input.notes.trim();

    if label.is_empty() {
        return Err(anyhow!("Recipient label is required"));
    }
    if address.is_empty() {
        return Err(anyhow!("Recipient address is required"));
    }

    let network = parse_network(&input.network)?;
    let checked = Address::from_str(address)?
        .require_network(network)
        .map_err(|e| anyhow!("address/network mismatch: {e}"))?;

    let mut entries = load_entries()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let id = format!("{}-{}", sanitize_id(label), now);

    entries.retain(|entry| !(entry.address == checked.to_string() && entry.network == network_id(network)));
    entries.push(AddressBookEntry {
        id,
        label: label.to_string(),
        address: checked.to_string(),
        network: network_id(network).to_string(),
        notes: notes.to_string(),
        created_at_unix: now,
    });
    entries.sort_by(|a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase()));
    save_entries(&entries)?;
    result(entries, "Recipient saved to local address book.")
}

pub fn delete_address_book_entry(id: String) -> Result<AddressBookResult> {
    let mut entries = load_entries()?;
    let before = entries.len();
    entries.retain(|entry| entry.id != id);
    if entries.len() == before {
        return Err(anyhow!("No address book entry matched that id"));
    }
    save_entries(&entries)?;
    result(entries, "Recipient removed from local address book.")
}

fn result(entries: Vec<AddressBookEntry>, status: &str) -> Result<AddressBookResult> {
    let path = address_book_path()?;
    Ok(AddressBookResult {
        entries,
        path: path.display().to_string(),
        status: status.to_string(),
        warning: "Address book labels and addresses are stored unencrypted. Do not put private notes, seed words, or sensitive identity information here.".to_string(),
    })
}

fn load_entries() -> Result<Vec<AddressBookEntry>> {
    let path = address_book_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let json = fs::read_to_string(&path)?;
    serde_json::from_str(&json).map_err(|e| anyhow!("Address book JSON is invalid: {e}"))
}

fn save_entries(entries: &[AddressBookEntry]) -> Result<()> {
    let path = address_book_path()?;
    let parent = path.parent().ok_or_else(|| anyhow!("invalid address book path"))?;
    fs::create_dir_all(parent)?;
    fs::write(path, serde_json::to_string_pretty(entries)?)?;
    Ok(())
}

fn address_book_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| anyhow!("Could not find HOME/USERPROFILE for local address book storage"))?;
    Ok(PathBuf::from(home).join(".carlosk-wallet").join("address_book.json"))
}

fn parse_network(choice: &str) -> Result<Network> {
    match choice.trim().to_lowercase().as_str() {
        "bitcoin" | "mainnet" | "bitcoin-mainnet" => Ok(Network::Bitcoin),
        "testnet" | "bitcoin-testnet" => Ok(Network::Testnet),
        "signet" | "bitcoin-signet" => Ok(Network::Signet),
        other => Err(anyhow!("Unsupported Bitcoin network: {other}. Use bitcoin, testnet, or signet.")),
    }
}

fn network_id(network: Network) -> &'static str {
    match network {
        Network::Bitcoin => "bitcoin",
        Network::Testnet => "testnet",
        Network::Signet => "signet",
        Network::Regtest => "regtest",
        _ => "unknown",
    }
}

fn sanitize_id(input: &str) -> String {
    let clean: String = input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(32)
        .collect();
    if clean.is_empty() {
        "recipient".to_string()
    } else {
        clean
    }
}
