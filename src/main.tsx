import React, { useEffect, useMemo, useState } from 'react';
import ReactDOM from 'react-dom/client';
import { invoke } from '@tauri-apps/api/core';
import './styles.css';

type Tab = 'wallet' | 'lightning-ocean';
type BitcoinNetwork = 'bitcoin' | 'testnet' | 'signet';

type WalletInfo = {
  label: string;
  mnemonic: string;
  address: string;
  network: BitcoinNetwork | string;
  derivation: string;
  next_external_index: number;
  warning: string;
};

type ReceiveAddressInfo = { address: string; index: number; network: string; warning: string };
type PersistedWalletStatus = { exists: boolean; path: string; status: string; warning: string };
type EncryptedBackup = { version: string; kdf: string; cipher: string; salt_b64: string; nonce_b64: string; ciphertext_b64: string; warning: string };
type BackendSyncReport = { network: string; esplora_url: string; total_sats: number; confirmed_sats: number; pending_sats: number; utxo_count: number; utxo_sats: number; status: string; warning: string };
type SendDraft = { to_address: string; amount_sats: number; fee_rate_sat_vb: number; ready_to_broadcast: boolean; status: string; next_steps: string[]; warning: string };
type SignedTransactionResult = { txid: string; tx_hex: string; recipient: string; amount_sats: number; fee_sats: number; fee_rate_sat_vb: number; finalized: boolean; ready_to_broadcast: boolean; status: string; warning: string };
type RawTransactionReview = { txid: string; network: string; input_count: number; output_count: number; output_sats: number; tx_hex_len: number; explorer_url: string; ready_for_broadcast_review: boolean; status: string; warning: string };
type BackendBroadcastResult = { txid: string; network: string; esplora_url: string; explorer_url: string; status: string; warning: string };
type SignatureResponse = { signature: string; address: string; format: string };
type LightningWalletInfo = { alias: string; network: string; status: string; can_receive_bolt12_in_app: boolean; warning: string; next_steps: string[] };
type Bolt12OfferInfo = { offer: string; source: string; status: string; warning: string };
type CoreLightningConnectionStatus = { online: boolean; backend: string; node_url: string; node_alias: string; node_id: string; network: string; status: string; warning: string };
type OceanSetupPlan = { btc_address: string; bolt12_offer: string; expected_description: string; ocean_config_url: string; ocean_stats_url: string; message_to_sign_source: string; steps: string[]; warnings: string[] };

type BackupVerifyResult = { backup_address: string; current_address: string; same_address: boolean; network: string; next_external_index: number; status: string };

const networks: { id: BitcoinNetwork; label: string; hint: string }[] = [
  { id: 'signet', label: 'Signet', hint: 'Recommended while testing.' },
  { id: 'testnet', label: 'Testnet', hint: 'Also safe for testing.' },
  { id: 'bitcoin', label: 'Bitcoin mainnet', hint: 'Real BTC. Use carefully.' }
];

const tabs: { id: Tab; label: string }[] = [
  { id: 'wallet', label: 'BTC Wallet' },
  { id: 'lightning-ocean', label: 'Lightning + OCEAN' }
];

function formatSats(sats?: number) {
  return `${Number(sats || 0).toLocaleString()} sats`;
}

function explorerBase(network: string) {
  if (network === 'testnet') return 'https://mempool.space/testnet';
  if (network === 'signet') return 'https://mempool.space/signet';
  return 'https://mempool.space';
}

function short(value: string, left = 14, right = 10) {
  if (!value) return '';
  if (value.length <= left + right + 3) return value;
  return `${value.slice(0, left)}…${value.slice(-right)}`;
}

function copy(value: string) {
  navigator.clipboard?.writeText(value);
}

function App() {
  const [tab, setTab] = useState<Tab>('wallet');
  const [status, setStatus] = useState('BTC Wallet CK v0.99.6 online Lightning core is running. Keep it simple: create/import BTC wallet, hold balance, send, sign, and prepare OCEAN Lightning payouts.');

  const [wallet, setWallet] = useState<WalletInfo | null>(null);
  const [walletLabel, setWalletLabel] = useState('CarlosK');
  const [network, setNetwork] = useState<BitcoinNetwork>('signet');
  const [restoreSeed, setRestoreSeed] = useState('');
  const [showSeed, setShowSeed] = useState(false);

  const [persistPassphrase, setPersistPassphrase] = useState('');
  const [loadPassphrase, setLoadPassphrase] = useState('');
  const [persistStatus, setPersistStatus] = useState<PersistedWalletStatus | null>(null);
  const [backupPassphrase, setBackupPassphrase] = useState('');
  const [encryptedBackup, setEncryptedBackup] = useState<EncryptedBackup | null>(null);
  const [verifyBackupJson, setVerifyBackupJson] = useState('');
  const [verifyBackupPassphrase, setVerifyBackupPassphrase] = useState('');
  const [backupVerify, setBackupVerify] = useState<BackupVerifyResult | null>(null);

  const [receiveAddress, setReceiveAddress] = useState<ReceiveAddressInfo | null>(null);
  const [syncReport, setSyncReport] = useState<BackendSyncReport | null>(null);
  const [customEsploraUrl, setCustomEsploraUrl] = useState('');
  const [sendTo, setSendTo] = useState('');
  const [sendAmountSats, setSendAmountSats] = useState('1000');
  const [sendFeeRate, setSendFeeRate] = useState('5');
  const [sendDraft, setSendDraft] = useState<SendDraft | null>(null);
  const [signedTx, setSignedTx] = useState<SignedTransactionResult | null>(null);
  const [review, setReview] = useState<RawTransactionReview | null>(null);
  const [broadcastConfirm, setBroadcastConfirm] = useState('');
  const [broadcastResult, setBroadcastResult] = useState<BackendBroadcastResult | null>(null);
  const [busy, setBusy] = useState(false);

  const [lightningAlias, setLightningAlias] = useState('CarlosK Lightning');
  const [lightningWallet, setLightningWallet] = useState<LightningWalletInfo | null>(null);
  const [coreNodeUrl, setCoreNodeUrl] = useState('http://127.0.0.1:3010');
  const [coreNodeRune, setCoreNodeRune] = useState('');
  const [coreNodeConnection, setCoreNodeConnection] = useState<CoreLightningConnectionStatus | null>(null);
  const [oceanOfferLabel, setOceanOfferLabel] = useState('carlosk-ocean-payouts');
  const [oceanSetupPlan, setOceanSetupPlan] = useState<OceanSetupPlan | null>(null);
  const [bolt12Input, setBolt12Input] = useState('');
  const [bolt12Offer, setBolt12Offer] = useState<Bolt12OfferInfo | null>(null);
  const [oceanMessage, setOceanMessage] = useState('');
  const [oceanSignature, setOceanSignature] = useState<SignatureResponse | null>(null);
  const [genericMessage, setGenericMessage] = useState('');
  const [genericSignature, setGenericSignature] = useState<SignatureResponse | null>(null);

  const walletExplorer = useMemo(() => wallet ? `${explorerBase(wallet.network)}/address/${wallet.address}` : '', [wallet]);
  const effectiveNetwork = (wallet?.network as BitcoinNetwork) || network;

  useEffect(() => {
    refreshPersistenceStatus();
    invoke<WalletInfo | null>('get_current_wallet')
      .then((existing) => {
        if (existing) {
          setWallet(existing);
          if (existing.network === 'bitcoin' || existing.network === 'testnet' || existing.network === 'signet') setNetwork(existing.network);
        }
      })
      .catch(() => undefined);
    invoke<Bolt12OfferInfo | null>('get_bolt12_offer')
      .then((offer) => {
        if (offer) {
          setBolt12Offer(offer);
          setBolt12Input(offer.offer);
        }
      })
      .catch(() => undefined);
    invoke<LightningWalletInfo | null>('get_lightning_wallet').then(setLightningWallet).catch(() => undefined);
    invoke<CoreLightningConnectionStatus | null>('get_core_lightning_connection').then(setCoreNodeConnection).catch(() => undefined);
  }, []);

  function setCurrentWallet(next: WalletInfo, nextStatus: string) {
    setWallet(next);
    if (next.network === 'bitcoin' || next.network === 'testnet' || next.network === 'signet') setNetwork(next.network);
    setReceiveAddress(null);
    setSyncReport(null);
    setSendDraft(null);
    setSignedTx(null);
    setReview(null);
    setBroadcastResult(null);
    setOceanSignature(null);
    setGenericSignature(null);
    setStatus(nextStatus);
  }

  async function refreshPersistenceStatus() {
    try {
      setPersistStatus(await invoke<PersistedWalletStatus>('get_wallet_persistence_status'));
    } catch (err) {
      setStatus(String(err));
    }
  }

  async function createWallet() {
    try {
      const created = await invoke<WalletInfo>('create_bitcoin_wallet', { label: walletLabel, network });
      setCurrentWallet(created, 'New BTC wallet created locally. Back up the seed and save the encrypted wallet before receiving funds.');
    } catch (err) { setStatus(String(err)); }
  }

  async function importWallet() {
    try {
      if (!restoreSeed.trim()) return setStatus('Paste the 12-word seed phrase first.');
      const restored = await invoke<WalletInfo>('restore_bitcoin_wallet', { mnemonic: restoreSeed.trim(), label: walletLabel, network });
      setCurrentWallet(restored, 'BTC wallet imported. Confirm the address and network are correct.');
    } catch (err) { setStatus(String(err)); }
  }

  async function saveWallet() {
    try {
      const result = await invoke<PersistedWalletStatus>('save_wallet_to_disk', { passphrase: persistPassphrase });
      setPersistStatus(result);
      setStatus(result.status);
    } catch (err) { setStatus(String(err)); }
  }

  async function loadWallet() {
    try {
      const result = await invoke<WalletInfo>('load_wallet_from_disk', { passphrase: loadPassphrase, label: walletLabel });
      setCurrentWallet(result, 'Encrypted wallet unlocked from local disk.');
      await refreshPersistenceStatus();
    } catch (err) { setStatus(String(err)); }
  }

  async function lockWallet() {
    try {
      const result = await invoke<string>('lock_wallet_in_memory');
      setWallet(null);
      setReceiveAddress(null);
      setSyncReport(null);
      setStatus(result);
    } catch (err) { setStatus(String(err)); }
  }

  async function exportBackup() {
    try {
      const result = await invoke<EncryptedBackup>('export_encrypted_wallet_backup', { passphrase: backupPassphrase });
      setEncryptedBackup(result);
      setVerifyBackupJson(JSON.stringify(result, null, 2));
      setStatus('Encrypted backup exported. Keep the JSON and passphrase private.');
    } catch (err) { setStatus(String(err)); }
  }

  async function verifyBackup() {
    try {
      const result = await invoke<BackupVerifyResult>('verify_encrypted_wallet_backup', { backupJson: verifyBackupJson.trim(), passphrase: verifyBackupPassphrase });
      setBackupVerify(result);
      setStatus(result.status);
    } catch (err) { setStatus(String(err)); }
  }

  async function generateReceive() {
    try {
      const result = await invoke<ReceiveAddressInfo>('generate_receive_address');
      setReceiveAddress(result);
      setStatus('Fresh BTC receive address generated locally.');
    } catch (err) { setStatus(String(err)); }
  }

  async function syncWallet() {
    try {
      setBusy(true);
      const result = await invoke<BackendSyncReport>('sync_wallet_backend', { esploraUrl: customEsploraUrl.trim() || null });
      setSyncReport(result);
      setStatus(result.status);
    } catch (err) { setStatus(String(err)); }
    finally { setBusy(false); }
  }

  async function validateSend() {
    try {
      const result = await invoke<SendDraft>('create_send_draft', {
        input: { to_address: sendTo.trim(), amount_sats: Number(sendAmountSats), fee_rate_sat_vb: Number(sendFeeRate) }
      });
      setSendDraft(result);
      setStatus(result.status);
    } catch (err) { setStatus(String(err)); }
  }

  async function buildSignedTx() {
    try {
      setBusy(true);
      const result = await invoke<SignedTransactionResult>('create_signed_send_transaction', {
        input: { to_address: sendTo.trim(), amount_sats: Number(sendAmountSats), fee_rate_sat_vb: Number(sendFeeRate) }
      });
      setSignedTx(result);
      setReview(null);
      setBroadcastResult(null);
      setStatus(result.status);
    } catch (err) { setStatus(String(err)); }
    finally { setBusy(false); }
  }

  async function reviewSignedTx() {
    try {
      if (!signedTx?.tx_hex) return setStatus('Build and sign a transaction first.');
      const result = await invoke<RawTransactionReview>('review_raw_transaction', { txHex: signedTx.tx_hex, network: effectiveNetwork });
      setReview(result);
      setStatus(result.status);
    } catch (err) { setStatus(String(err)); }
  }

  async function broadcastSignedTx() {
    try {
      if (broadcastConfirm !== 'BROADCAST') return setStatus('Type BROADCAST exactly before sending the transaction.');
      if (!signedTx?.tx_hex) return setStatus('No signed transaction is ready.');
      setBusy(true);
      const result = await invoke<BackendBroadcastResult>('broadcast_raw_transaction_backend', {
        input: { tx_hex: signedTx.tx_hex, network: effectiveNetwork, esplora_url: customEsploraUrl.trim() || null }
      });
      setBroadcastResult(result);
      setStatus(result.status);
    } catch (err) { setStatus(String(err)); }
    finally { setBusy(false); }
  }

  async function createLightningWallet() {
    try {
      const result = await invoke<LightningWalletInfo>('create_lightning_wallet', { alias: lightningAlias });
      setLightningWallet(result);
      setStatus(result.status);
    } catch (err) { setStatus(String(err)); }
  }

  async function connectCoreLightningNode() {
    try {
      const result = await invoke<CoreLightningConnectionStatus>('connect_core_lightning_node', {
        input: {
          node_url: coreNodeUrl.trim(),
          rune: coreNodeRune.trim(),
          network: effectiveNetwork,
          alias: lightningAlias
        }
      });
      setCoreNodeConnection(result);
      setStatus(result.status);
    } catch (err) { setStatus(String(err)); }
  }

  async function createOceanBolt12FromNode() {
    try {
      if (!wallet) return setStatus('Create or unlock the BTC wallet first. OCEAN BOLT12 description must include the BTC mining address.');
      const result = await invoke<Bolt12OfferInfo>('create_ocean_bolt12_offer', {
        input: {
          node_url: coreNodeUrl.trim(),
          rune: coreNodeRune.trim(),
          label: oceanOfferLabel.trim(),
          amount: 'any'
        }
      });
      setBolt12Offer(result);
      setBolt12Input(result.offer);
      setStatus(result.status);
      await buildOceanPlan(result.offer);
    } catch (err) { setStatus(String(err)); }
  }

  async function buildOceanPlan(offerOverride?: string) {
    try {
      if (!wallet) return setStatus('Create or unlock the BTC wallet first.');
      const result = await invoke<OceanSetupPlan>('build_ocean_setup_plan', { offer: offerOverride || bolt12Input.trim() });
      setOceanSetupPlan(result);
      setStatus('OCEAN setup plan built. Open OCEAN, paste the BOLT12 offer, generate the message, then sign it here.');
    } catch (err) { setStatus(String(err)); }
  }

  async function createInAppOffer() {
    try {
      const result = await invoke<Bolt12OfferInfo>('create_in_app_bolt12_offer');
      setBolt12Offer(result);
      setBolt12Input(result.offer);
      setStatus(result.status);
    } catch (err) { setStatus(String(err)); }
  }

  async function saveBolt12() {
    try {
      const result = await invoke<Bolt12OfferInfo>('save_bolt12_offer', { offer: bolt12Input.trim() });
      setBolt12Offer(result);
      setStatus(result.status);
      if (wallet) await buildOceanPlan(result.offer);
    } catch (err) { setStatus(String(err)); }
  }

  async function signOceanMessage() {
    try {
      if (!wallet) return setStatus('Create or unlock the BTC wallet first.');
      if (!oceanMessage.trim()) return setStatus('Paste the unsigned OCEAN configuration message first.');
      const result = await invoke<SignatureResponse>('sign_message', { message: oceanMessage.trim(), address: wallet.address });
      setOceanSignature(result);
      setStatus('OCEAN configuration message signed with your BTC wallet. Copy this signature back into OCEAN.');
    } catch (err) { setStatus(String(err)); }
  }

  async function signGenericMessage() {
    try {
      if (!wallet) return setStatus('Create or unlock the BTC wallet first.');
      const result = await invoke<SignatureResponse>('sign_message', { message: genericMessage.trim(), address: wallet.address });
      setGenericSignature(result);
      setStatus('Message signed with the current BTC wallet.');
    } catch (err) { setStatus(String(err)); }
  }

  return (
    <main>
      <section className="hero">
        <div>
          <p className="eyebrow">Simple online wallet</p>
          <h1>BTC Wallet CK <span>v0.99.6</span></h1>
          <p className="subtitle">Create/import BTC, send, sign, connect to an online Lightning node, generate OCEAN BOLT12 offers, and sign OCEAN setup messages.</p>
        </div>
        <div className="status-card">
          <strong>Status</strong>
          <p>{status}</p>
          {wallet ? <code>{wallet.network} · {short(wallet.address)}</code> : <p className="muted">No wallet unlocked.</p>}
        </div>
      </section>

      <nav className="tabs">
        {tabs.map((item) => <button key={item.id} className={tab === item.id ? 'active' : ''} onClick={() => setTab(item.id)}>{item.label}</button>)}
      </nav>

      {tab === 'wallet' && (
        <section className="grid two">
          <div className="card">
            <h2>1. Create or import BTC wallet</h2>
            <label>Wallet name<input value={walletLabel} onChange={(e) => setWalletLabel(e.target.value)} /></label>
            <label>Network<select value={network} onChange={(e) => setNetwork(e.target.value as BitcoinNetwork)}>{networks.map((item) => <option key={item.id} value={item.id}>{item.label} — {item.hint}</option>)}</select></label>
            <div className="row"><button onClick={createWallet}>Create New BTC Wallet</button></div>
            <label>Import 12-word seed<textarea value={restoreSeed} onChange={(e) => setRestoreSeed(e.target.value)} placeholder="abandon abandon ..." /></label>
            <div className="row"><button className="secondary" onClick={importWallet}>Import Wallet</button></div>
            {wallet && (
              <div className="result">
                <p><strong>Current address</strong></p>
                <code>{wallet.address}</code>
                <p className="muted">Derivation: {wallet.derivation}</p>
                <a className="button-link" href={walletExplorer} target="_blank">Open Explorer</a>
                <div className="row"><button className="secondary" onClick={() => setShowSeed(!showSeed)}>{showSeed ? 'Hide Seed' : 'Show Seed Backup'}</button><button className="danger" onClick={lockWallet}>Lock Wallet</button></div>
                {showSeed && <code>{wallet.mnemonic}</code>}
              </div>
            )}
          </div>

          <div className="card">
            <h2>2. Save and unlock</h2>
            <p className="muted">This saves an encrypted wallet file locally. The passphrase is not stored.</p>
            <label>Save passphrase<input type="password" value={persistPassphrase} onChange={(e) => setPersistPassphrase(e.target.value)} /></label>
            <div className="row"><button onClick={saveWallet} disabled={!wallet}>Save Encrypted Wallet</button></div>
            <label>Unlock passphrase<input type="password" value={loadPassphrase} onChange={(e) => setLoadPassphrase(e.target.value)} /></label>
            <div className="row"><button className="secondary" onClick={loadWallet}>Unlock Saved Wallet</button><button className="secondary" onClick={refreshPersistenceStatus}>Check Saved Wallet</button></div>
            {persistStatus && <div className="result"><p>{persistStatus.status}</p><code>{persistStatus.path}</code></div>}
          </div>

          <div className="card">
            <h2>3. Receive and balance</h2>
            <div className="row"><button onClick={generateReceive} disabled={!wallet}>New Receive Address</button><button className="secondary" onClick={syncWallet} disabled={!wallet || busy}>{busy ? 'Working...' : 'Sync Balance'}</button></div>
            {receiveAddress && <div className="result"><p><strong>Receive address</strong></p><code>{receiveAddress.address}</code></div>}
            <label>Custom Esplora URL, optional<input value={customEsploraUrl} onChange={(e) => setCustomEsploraUrl(e.target.value)} placeholder="Leave blank for mempool.space" /></label>
            {syncReport && <div className="result"><p><strong>Total:</strong> {formatSats(syncReport.total_sats)}</p><p><strong>Confirmed:</strong> {formatSats(syncReport.confirmed_sats)} · <strong>Pending:</strong> {formatSats(syncReport.pending_sats)}</p><p><strong>UTXOs:</strong> {syncReport.utxo_count}</p><p className="warning">{syncReport.warning}</p></div>}
          </div>

          <div className="card">
            <h2>4. Send BTC</h2>
            <label>Recipient BTC address<input value={sendTo} onChange={(e) => setSendTo(e.target.value)} /></label>
            <label>Amount sats<input value={sendAmountSats} onChange={(e) => setSendAmountSats(e.target.value)} /></label>
            <label>Fee rate sat/vB<input value={sendFeeRate} onChange={(e) => setSendFeeRate(e.target.value)} /></label>
            <div className="row"><button className="secondary" onClick={validateSend} disabled={!wallet}>Validate</button><button onClick={buildSignedTx} disabled={!wallet || busy}>Build + Sign</button></div>
            {sendDraft && <div className="result"><p>{sendDraft.status}</p><p className="warning">{sendDraft.warning}</p></div>}
            {signedTx && <div className="result"><p><strong>Signed transaction ready</strong></p><p>Amount: {formatSats(signedTx.amount_sats)} · Fee: {formatSats(signedTx.fee_sats)}</p><code>{signedTx.txid}</code><div className="row"><button className="secondary" onClick={reviewSignedTx}>Review TX</button><button className="secondary" onClick={() => copy(signedTx.tx_hex)}>Copy Raw TX</button></div></div>}
            {review && <div className="result"><p>{review.status}</p><p>Inputs: {review.input_count} · Outputs: {review.output_count} · Output total: {formatSats(review.output_sats)}</p><a className="button-link" href={review.explorer_url} target="_blank">Preview TX</a></div>}
            {signedTx && <><label>Type BROADCAST to send<input value={broadcastConfirm} onChange={(e) => setBroadcastConfirm(e.target.value)} /></label><button className="danger" onClick={broadcastSignedTx} disabled={busy}>Broadcast Signed TX</button></>}
            {broadcastResult && <div className="result"><p>{broadcastResult.status}</p><a className="button-link" href={broadcastResult.explorer_url} target="_blank">Open Broadcast TX</a></div>}
          </div>

          <div className="card wide">
            <h2>Backup</h2>
            <p className="muted">Use this for offline backup. This is separate from the local encrypted saved wallet file.</p>
            <label>Backup passphrase<input type="password" value={backupPassphrase} onChange={(e) => setBackupPassphrase(e.target.value)} /></label>
            <div className="row"><button onClick={exportBackup} disabled={!wallet}>Export Encrypted Backup</button>{encryptedBackup && <button className="secondary" onClick={() => copy(JSON.stringify(encryptedBackup, null, 2))}>Copy Backup JSON</button>}</div>
            {encryptedBackup && <textarea value={JSON.stringify(encryptedBackup, null, 2)} readOnly />}
            <div className="divider" />
            <label>Verify backup JSON<textarea value={verifyBackupJson} onChange={(e) => setVerifyBackupJson(e.target.value)} /></label>
            <label>Verify passphrase<input type="password" value={verifyBackupPassphrase} onChange={(e) => setVerifyBackupPassphrase(e.target.value)} /></label>
            <button className="secondary" onClick={verifyBackup}>Verify Backup Restores Same Wallet</button>
            {backupVerify && <div className={backupVerify.same_address ? 'result positive' : 'result negative'}>{backupVerify.status}<code>{backupVerify.backup_address}</code></div>}
          </div>
        </section>
      )}

      {tab === 'lightning-ocean' && (
        <section className="grid two">
          <div className="card wide">
            <h2>1. Online Lightning node</h2>
            <p className="muted">This is the real next step: connect BTC Wallet CK to an online Core Lightning node so the app can generate the BOLT12 offer instead of Lexe. The node must stay online and have inbound liquidity.</p>
            <label>Lightning alias<input value={lightningAlias} onChange={(e) => setLightningAlias(e.target.value)} /></label>
            <label>Core Lightning REST / JSON-RPC URL<input value={coreNodeUrl} onChange={(e) => setCoreNodeUrl(e.target.value)} placeholder="http://127.0.0.1:3010" /></label>
            <label>Rune / API token<input type="password" value={coreNodeRune} onChange={(e) => setCoreNodeRune(e.target.value)} placeholder="Leave blank only for a local unsecured test node" /></label>
            <div className="row"><button onClick={createLightningWallet}>Create Lightning Profile</button><button className="secondary" onClick={connectCoreLightningNode}>Test Node Connection</button></div>
            {lightningWallet && <div className="result"><p>{lightningWallet.status}</p><p className="warning">{lightningWallet.warning}</p></div>}
            {coreNodeConnection && <div className={coreNodeConnection.online ? 'result positive' : 'result negative'}><p>{coreNodeConnection.status}</p><p><strong>Node:</strong> {coreNodeConnection.node_alias || 'Core Lightning'} {coreNodeConnection.node_id && <code>{short(coreNodeConnection.node_id)}</code>}</p><p className="warning">{coreNodeConnection.warning}</p></div>}
          </div>

          <div className="card wide">
            <h2>2. Generate OCEAN BOLT12 offer</h2>
            <p className="muted">This replaces Lexe for OCEAN. The app asks your online Lightning node to create a real BOLT12 offer with amount=any and the required OCEAN description for your BTC mining address.</p>
            {wallet ? <><p><strong>OCEAN BTC address / worker</strong></p><code>{wallet.address}</code><button className="secondary small" onClick={() => copy(wallet.address)}>Copy BTC Address</button></> : <p className="warning">Create or unlock a BTC wallet first.</p>}
            <label>Offer label<input value={oceanOfferLabel} onChange={(e) => setOceanOfferLabel(e.target.value)} /></label>
            <div className="row"><button onClick={createOceanBolt12FromNode} disabled={!wallet}>Generate OCEAN BOLT12 From Node</button><button className="secondary" onClick={createInAppOffer}>Embedded LDK Status</button></div>
            <label>BOLT12 offer<textarea value={bolt12Input} onChange={(e) => setBolt12Input(e.target.value)} placeholder="lno1... generated by the connected node or pasted manually" /></label>
            <div className="row"><button className="secondary" onClick={saveBolt12}>Save/Validate Offer</button>{bolt12Offer && <button className="secondary" onClick={() => copy(bolt12Offer.offer)}>Copy Offer</button>}<button className="secondary" onClick={() => buildOceanPlan()} disabled={!wallet}>Build OCEAN Plan</button></div>
            {bolt12Offer && <div className="result"><p>{bolt12Offer.status}</p><code>{short(bolt12Offer.offer, 32, 18)}</code><p className="warning">{bolt12Offer.warning}</p></div>}
          </div>

          <div className="card wide">
            <h2>3. OCEAN setup: no Lexe, no Sparrow</h2>
            <p className="muted">BTC Wallet CK now handles the two OCEAN requirements: BOLT12 offer from your online Lightning node, then BTC message signature from this wallet.</p>
            {oceanSetupPlan ? <div className="result"><p><strong>Required description</strong></p><code>{oceanSetupPlan.expected_description}</code><p><strong>OCEAN config page</strong></p><a className="button-link" href={oceanSetupPlan.ocean_config_url} target="_blank">Open OCEAN Config</a><a className="button-link" href={oceanSetupPlan.ocean_stats_url} target="_blank">Open OCEAN Stats</a><div className="divider" /><ol>{oceanSetupPlan.steps.map((item) => <li key={item}>{item}</li>)}</ol><p className="warning">{oceanSetupPlan.warnings.join(' ')}</p></div> : <p className="warning">Generate or paste a BOLT12 offer, then build the OCEAN plan.</p>}
            <label>Unsigned OCEAN configuration message<textarea value={oceanMessage} onChange={(e) => setOceanMessage(e.target.value)} placeholder="Paste the exact message generated by OCEAN after entering your BOLT12 offer..." /></label>
            <div className="row"><button onClick={signOceanMessage} disabled={!wallet}>Sign OCEAN Message With BTC Wallet</button>{oceanSignature && <button className="secondary" onClick={() => copy(oceanSignature.signature)}>Copy Signature</button>}</div>
            {oceanSignature && <div className="result"><p><strong>Signature</strong> · {oceanSignature.format}</p><code>{oceanSignature.signature}</code><p className="muted">Paste this signature into OCEAN to confirm the payout configuration.</p></div>}
          </div>

          <div className="card wide">
            <h2>Generic BTC message signing</h2>
            <p className="muted">For anything else that needs a Bitcoin address signature.</p>
            <label>Message<textarea value={genericMessage} onChange={(e) => setGenericMessage(e.target.value)} /></label>
            <div className="row"><button onClick={signGenericMessage} disabled={!wallet}>Sign With Current Wallet</button>{genericSignature && <button className="secondary" onClick={() => copy(genericSignature.signature)}>Copy Signature</button>}</div>
            {genericSignature && <div className="result"><code>{genericSignature.signature}</code></div>}
          </div>
        </section>
      )}
    </main>
  );
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
