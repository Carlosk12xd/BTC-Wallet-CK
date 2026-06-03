use std::sync::Mutex;
use tauri::State;

mod address_book;
mod core_lightning;
mod storage;
mod wallet;

struct AppState {
    wallet: Mutex<Option<wallet::MiningWallet>>,
    bolt12_offer: Mutex<Option<core_lightning::Bolt12OfferInfo>>,
    lightning_wallet: Mutex<Option<core_lightning::LightningWalletInfo>>,
    lightning_connection: Mutex<Option<core_lightning::CoreLightningConnectionStatus>>,
}

#[tauri::command]
fn create_bitcoin_wallet(label: String, network: String, state: State<AppState>) -> Result<wallet::WalletInfo, String> {
    let wallet = wallet::MiningWallet::new_random(&label, &network).map_err(|e| e.to_string())?;
    let info = wallet.info().map_err(|e| e.to_string())?;
    *state.wallet.lock().map_err(|_| "wallet state lock failed")? = Some(wallet);
    Ok(info)
}

#[tauri::command]
fn restore_bitcoin_wallet(mnemonic: String, label: String, network: String, state: State<AppState>) -> Result<wallet::WalletInfo, String> {
    let wallet = wallet::MiningWallet::from_mnemonic(&mnemonic, &label, &network).map_err(|e| e.to_string())?;
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
fn lock_wallet_in_memory(state: State<AppState>) -> Result<String, String> {
    *state.wallet.lock().map_err(|_| "wallet state lock failed")? = None;
    Ok("Wallet locked in memory. Saved encrypted wallet file was not deleted.".to_string())
}

#[tauri::command]
fn generate_receive_address(state: State<AppState>) -> Result<wallet::ReceiveAddressInfo, String> {
    let mut guard = state.wallet.lock().map_err(|_| "wallet state lock failed")?;
    let wallet = guard.as_mut().ok_or("Create or restore a Bitcoin wallet first")?;
    wallet.next_receive_address().map_err(|e| e.to_string())
}

#[tauri::command]
fn create_send_draft(input: wallet::SendDraftInput, state: State<AppState>) -> Result<wallet::SendDraft, String> {
    let guard = state.wallet.lock().map_err(|_| "wallet state lock failed")?;
    let wallet = guard.as_ref().ok_or("Create or restore a Bitcoin wallet first")?;
    wallet.create_send_draft(input).map_err(|e| e.to_string())
}

#[tauri::command]
fn sync_wallet_backend(esplora_url: Option<String>, state: State<AppState>) -> Result<wallet::BackendSyncReport, String> {
    let mut guard = state.wallet.lock().map_err(|_| "wallet state lock failed")?;
    let wallet = guard.as_mut().ok_or("Create or restore a Bitcoin wallet first")?;
    wallet.sync_with_esplora(esplora_url).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_signed_send_transaction(input: wallet::SendDraftInput, state: State<AppState>) -> Result<wallet::SignedTransactionResult, String> {
    let mut guard = state.wallet.lock().map_err(|_| "wallet state lock failed")?;
    let wallet = guard.as_mut().ok_or("Create or restore a Bitcoin wallet first")?;
    wallet.create_signed_transaction(input).map_err(|e| e.to_string())
}

#[tauri::command]
fn review_raw_transaction(tx_hex: String, network: String) -> Result<wallet::RawTransactionReview, String> {
    wallet::review_raw_transaction_hex(&tx_hex, &network).map_err(|e| e.to_string())
}

#[tauri::command]
fn broadcast_raw_transaction_backend(input: wallet::BackendBroadcastInput) -> Result<wallet::BackendBroadcastResult, String> {
    wallet::broadcast_raw_transaction_backend(input).map_err(|e| e.to_string())
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
        .map_err(|e| format!("Backup JSON was invalid: {e}"))?;
    let payload = storage::decrypt_wallet_backup(&backup, &passphrase).map_err(|e| e.to_string())?;
    let wallet = wallet::MiningWallet::from_backup_payload(&payload, &label).map_err(|e| e.to_string())?;
    let info = wallet.info().map_err(|e| e.to_string())?;
    *state.wallet.lock().map_err(|_| "wallet state lock failed")? = Some(wallet);
    Ok(info)
}

#[tauri::command]
fn verify_encrypted_wallet_backup(backup_json: String, passphrase: String, state: State<AppState>) -> Result<wallet::BackupVerifyResult, String> {
    let backup: storage::EncryptedBackup = serde_json::from_str(&backup_json)
        .map_err(|e| format!("Backup JSON was invalid: {e}"))?;
    let payload = storage::decrypt_wallet_backup(&backup, &passphrase).map_err(|e| e.to_string())?;
    let guard = state.wallet.lock().map_err(|_| "wallet state lock failed")?;
    let wallet = guard.as_ref().ok_or("Create or restore the current wallet before verifying a backup")?;
    wallet.verify_backup_payload(&payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_wallet_to_disk(passphrase: String, state: State<AppState>) -> Result<storage::PersistedWalletStatus, String> {
    let guard = state.wallet.lock().map_err(|_| "wallet state lock failed")?;
    let wallet = guard.as_ref().ok_or("Create or restore a Bitcoin wallet first")?;
    let payload = wallet.backup_payload().map_err(|e| e.to_string())?;
    let encrypted = storage::encrypt_wallet_backup(&payload, &passphrase).map_err(|e| e.to_string())?;
    storage::save_encrypted_wallet_to_disk(&encrypted).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_wallet_from_disk(passphrase: String, label: String, state: State<AppState>) -> Result<wallet::WalletInfo, String> {
    let encrypted = storage::load_encrypted_wallet_from_disk().map_err(|e| e.to_string())?;
    let payload = storage::decrypt_wallet_backup(&encrypted, &passphrase).map_err(|e| e.to_string())?;
    let wallet = wallet::MiningWallet::from_backup_payload(&payload, &label).map_err(|e| e.to_string())?;
    let info = wallet.info().map_err(|e| e.to_string())?;
    *state.wallet.lock().map_err(|_| "wallet state lock failed")? = Some(wallet);
    Ok(info)
}

#[tauri::command]
fn get_wallet_persistence_status() -> Result<storage::PersistedWalletStatus, String> {
    storage::persisted_wallet_status().map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_saved_wallet() -> Result<storage::PersistedWalletStatus, String> {
    storage::delete_encrypted_wallet_from_disk().map_err(|e| e.to_string())
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
fn connect_core_lightning_node(input: core_lightning::CoreLightningConnectionInput, state: State<AppState>) -> Result<core_lightning::CoreLightningConnectionStatus, String> {
    let status = core_lightning::check_core_lightning_node(input).map_err(|e| e.to_string())?;
    *state.lightning_connection.lock().map_err(|_| "lightning connection state lock failed")? = Some(status.clone());
    Ok(status)
}

#[tauri::command]
fn get_core_lightning_connection(state: State<AppState>) -> Result<Option<core_lightning::CoreLightningConnectionStatus>, String> {
    let guard = state.lightning_connection.lock().map_err(|_| "lightning connection state lock failed")?;
    Ok(guard.clone())
}

#[tauri::command]
fn create_ocean_bolt12_offer(input: core_lightning::OceanBolt12Request, state: State<AppState>) -> Result<core_lightning::Bolt12OfferInfo, String> {
    let wallet_guard = state.wallet.lock().map_err(|_| "wallet state lock failed")?;
    let wallet = wallet_guard.as_ref().ok_or("Create or unlock the BTC wallet first")?;
    let info = wallet.info().map_err(|e| e.to_string())?;
    let offer = core_lightning::create_ocean_bolt12_offer(input, &info.address).map_err(|e| e.to_string())?;
    *state.bolt12_offer.lock().map_err(|_| "bolt12 state lock failed")? = Some(offer.clone());
    Ok(offer)
}

#[tauri::command]
fn build_ocean_setup_plan(offer: String, state: State<AppState>) -> Result<core_lightning::OceanSetupPlan, String> {
    let wallet_guard = state.wallet.lock().map_err(|_| "wallet state lock failed")?;
    let wallet = wallet_guard.as_ref().ok_or("Create or unlock the BTC wallet first")?;
    let info = wallet.info().map_err(|e| e.to_string())?;
    let offer_value = if offer.trim().is_empty() {
        let offer_guard = state.bolt12_offer.lock().map_err(|_| "bolt12 state lock failed")?;
        offer_guard.as_ref().map(|o| o.offer.clone()).unwrap_or_default()
    } else {
        offer
    };
    core_lightning::build_ocean_setup_plan(&info.address, &offer_value).map_err(|e| e.to_string())
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

#[tauri::command]
fn list_address_book() -> Result<address_book::AddressBookResult, String> {
    address_book::list_address_book().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_address_book_entry(input: address_book::SaveAddressBookEntryInput) -> Result<address_book::AddressBookResult, String> {
    address_book::save_address_book_entry(input).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_address_book_entry(id: String) -> Result<address_book::AddressBookResult, String> {
    address_book::delete_address_book_entry(id).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            wallet: Mutex::new(None),
            bolt12_offer: Mutex::new(None),
            lightning_wallet: Mutex::new(None),
            lightning_connection: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            create_bitcoin_wallet,
            restore_bitcoin_wallet,
            get_current_wallet,
            lock_wallet_in_memory,
            generate_receive_address,
            create_send_draft,
            sync_wallet_backend,
            create_signed_send_transaction,
            review_raw_transaction,
            broadcast_raw_transaction_backend,
            export_encrypted_wallet_backup,
            restore_encrypted_wallet_backup,
            verify_encrypted_wallet_backup,
            save_wallet_to_disk,
            load_wallet_from_disk,
            get_wallet_persistence_status,
            delete_saved_wallet,
            list_address_book,
            save_address_book_entry,
            delete_address_book_entry,
            sign_message,
            create_lightning_wallet,
            get_lightning_wallet,
            connect_core_lightning_node,
            get_core_lightning_connection,
            create_ocean_bolt12_offer,
            build_ocean_setup_plan,
            save_bolt12_offer,
            get_bolt12_offer,
            create_in_app_bolt12_offer
        ])
        .run(tauri::generate_context!())
        .expect("error while running CarlosK Wallet");
}
