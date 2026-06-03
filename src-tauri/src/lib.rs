use std::sync::Mutex;
use tauri::State;

mod core_lightning;
mod storage;
mod wallet;

struct AppState {
    wallet: Mutex<Option<wallet::MiningWallet>>,
    bolt12_offer: Mutex<Option<core_lightning::Bolt12OfferInfo>>,
    lightning_wallet: Mutex<Option<core_lightning::LightningWalletInfo>>,
}

#[tauri::command]
fn create_bitcoin_wallet(label: String, state: State<AppState>) -> Result<wallet::WalletInfo, String> {
    let wallet = wallet::MiningWallet::new_random(&label).map_err(|e| e.to_string())?;
    let info = wallet.info().map_err(|e| e.to_string())?;
    *state.wallet.lock().map_err(|_| "wallet state lock failed")? = Some(wallet);
    Ok(info)
}

#[tauri::command]
fn restore_bitcoin_wallet(mnemonic: String, label: String, state: State<AppState>) -> Result<wallet::WalletInfo, String> {
    let wallet = wallet::MiningWallet::from_mnemonic(&mnemonic, &label).map_err(|e| e.to_string())?;
    let info = wallet.info().map_err(|e| e.to_string())?;
    *state.wallet.lock().map_err(|_| "wallet state lock failed")? = Some(wallet);
    Ok(info)
}

#[tauri::command]
fn get_current_wallet(state: State<AppState>) -> Result<Option<wallet::WalletInfo>, String> {
    let guard = state.wallet.lock().map_err(|_| "wallet state lock failed")?;
    match guard.as_ref() {
        Some(wallet) => wallet.info().map(Some).map_err(|e| e.to_string()),
        None => Ok(None),
    }
}

#[tauri::command]
fn generate_receive_address(state: State<AppState>) -> Result<wallet::ReceiveAddressInfo, String> {
    let mut guard = state.wallet.lock().map_err(|_| "wallet state lock failed")?;
    let wallet = guard.as_mut().ok_or("Create or restore a Bitcoin wallet first")?;
    wallet.next_receive_address().map_err(|e| e.to_string())
}

#[tauri::command]
fn create_send_draft(input: wallet::SendDraftInput) -> Result<wallet::SendDraft, String> {
    wallet::create_send_draft(input).map_err(|e| e.to_string())
}

#[tauri::command]
fn export_encrypted_wallet_backup(passphrase: String, state: State<AppState>) -> Result<storage::EncryptedBackup, String> {
    let guard = state.wallet.lock().map_err(|_| "wallet state lock failed")?;
    let wallet = guard.as_ref().ok_or("Create or restore a Bitcoin wallet first")?;
    let payload = wallet.backup_payload().map_err(|e| e.to_string())?;
    storage::encrypt_wallet_backup(&payload, &passphrase).map_err(|e| e.to_string())
}

#[tauri::command]
fn restore_encrypted_wallet_backup(backup_json: String, passphrase: String, label: String, state: State<AppState>) -> Result<wallet::WalletInfo, String> {
    let backup: storage::EncryptedBackup = serde_json::from_str(&backup_json)
        .map_err(|e| format!("Backup JSON is invalid: {e}"))?;
    let payload = storage::decrypt_wallet_backup(&backup, &passphrase).map_err(|e| e.to_string())?;
    let wallet = wallet::MiningWallet::from_mnemonic(&payload.mnemonic, &label).map_err(|e| e.to_string())?;
    let info = wallet.info().map_err(|e| e.to_string())?;
    if info.address != payload.address {
        return Err(format!(
            "Restored wallet address {} does not match backup address {}. Check derivation/version.",
            info.address, payload.address
        ));
    }
    *state.wallet.lock().map_err(|_| "wallet state lock failed")? = Some(wallet);
    Ok(info)
}

#[tauri::command]
fn sign_message(message: String, address: String, state: State<AppState>) -> Result<wallet::SignatureResponse, String> {
    let mut guard = state.wallet.lock().map_err(|_| "wallet state lock failed")?;
    let wallet = guard.as_mut().ok_or("Create or restore a Bitcoin wallet first")?;
    wallet.sign_bip322_simple(&message, &address).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_lightning_wallet(alias: String, state: State<AppState>) -> Result<core_lightning::LightningWalletInfo, String> {
    let info = core_lightning::create_lightning_wallet(&alias);
    *state.lightning_wallet.lock().map_err(|_| "lightning state lock failed")? = Some(info.clone());
    Ok(info)
}

#[tauri::command]
fn get_lightning_wallet(state: State<AppState>) -> Result<Option<core_lightning::LightningWalletInfo>, String> {
    let guard = state.lightning_wallet.lock().map_err(|_| "lightning state lock failed")?;
    Ok(guard.clone())
}

#[tauri::command]
fn save_bolt12_offer(offer: String, state: State<AppState>) -> Result<core_lightning::Bolt12OfferInfo, String> {
    let info = core_lightning::validate_bolt12_offer(&offer).map_err(|e| e.to_string())?;
    *state.bolt12_offer.lock().map_err(|_| "bolt12 state lock failed")? = Some(info.clone());
    Ok(info)
}

#[tauri::command]
fn get_bolt12_offer(state: State<AppState>) -> Result<Option<core_lightning::Bolt12OfferInfo>, String> {
    let guard = state.bolt12_offer.lock().map_err(|_| "bolt12 state lock failed")?;
    Ok(guard.clone())
}

#[tauri::command]
fn create_in_app_bolt12_offer() -> Result<core_lightning::Bolt12OfferInfo, String> {
    core_lightning::create_in_app_bolt12_offer().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            wallet: Mutex::new(None),
            bolt12_offer: Mutex::new(None),
            lightning_wallet: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            create_bitcoin_wallet,
            restore_bitcoin_wallet,
            get_current_wallet,
            generate_receive_address,
            create_send_draft,
            export_encrypted_wallet_backup,
            restore_encrypted_wallet_backup,
            sign_message,
            create_lightning_wallet,
            get_lightning_wallet,
            save_bolt12_offer,
            get_bolt12_offer,
            create_in_app_bolt12_offer
        ])
        .run(tauri::generate_context!())
        .expect("error while running CarlosK Wallet");
}
