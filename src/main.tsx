import React, { useEffect, useMemo, useState } from 'react';
import ReactDOM from 'react-dom/client';
import { invoke } from '@tauri-apps/api/core';
import './styles.css';

type Tab = 'wallet' | 'send-receive' | 'lightning' | 'signatures';
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

type ReceiveAddressInfo = {
  address: string;
  index: number;
  network: string;
  warning: string;
};

type EncryptedBackup = {
  version: string;
  kdf: string;
  cipher: string;
  salt_b64: string;
  nonce_b64: string;
  ciphertext_b64: string;
  warning: string;
};

type PersistedWalletStatus = {
  exists: boolean;
  path: string;
  status: string;
  warning: string;
};

type BackupVerifyResult = {
  backup_address: string;
  current_address: string;
  same_address: boolean;
  network: string;
  next_external_index: number;
  status: string;
};

type SendDraft = {
  to_address: string;
  amount_sats: number;
  fee_rate_sat_vb: number;
  ready_to_broadcast: boolean;
  status: string;
  next_steps: string[];
  warning: string;
};

type BackendSyncReport = {
  network: string;
  esplora_url: string;
  total_sats: number;
  confirmed_sats: number;
  pending_sats: number;
  utxo_count: number;
  utxo_sats: number;
  status: string;
  warning: string;
};

type SignedTransactionResult = {
  txid: string;
  tx_hex: string;
  recipient: string;
  amount_sats: number;
  fee_sats: number;
  fee_rate_sat_vb: number;
  finalized: boolean;
  ready_to_broadcast: boolean;
  status: string;
  warning: string;
};

type SignatureResponse = {
  signature: string;
  address: string;
  format: string;
};

type ChainBalance = {
  address: string;
  network: string;
  confirmed_sats: number;
  mempool_sats: number;
  total_sats: number;
  tx_count: number;
  utxo_count: number;
  utxo_sats: number;
  explorer_url: string;
  fetched_at: string;
};

type UtxoInfo = {
  txid: string;
  vout: number;
  value: number;
  confirmed: boolean;
  block_height?: number;
};

type TxSummary = {
  txid: string;
  confirmed: boolean;
  block_height?: number;
  block_time?: number;
  received_sats: number;
  spent_sats: number;
  net_sats: number;
  fee_sats?: number;
  url: string;
};

type FeeEstimates = {
  fastestFee?: number;
  halfHourFee?: number;
  hourFee?: number;
  economyFee?: number;
  minimumFee?: number;
};

type BroadcastResult = {
  txid: string;
  url: string;
  status: string;
};

type LightningWalletInfo = {
  alias: string;
  network: string;
  status: string;
  can_receive_bolt12_in_app: boolean;
  warning: string;
  next_steps: string[];
};

type Bolt12OfferInfo = {
  offer: string;
  source: string;
  status: string;
  warning: string;
};

const tabs: { id: Tab; label: string; description: string }[] = [
  { id: 'wallet', label: 'Wallet', description: 'Create, restore, persist, and back up your BTC wallet.' },
  { id: 'send-receive', label: 'Send / Receive', description: 'Generate on-chain receive addresses and prepare network-safe sends.' },
  { id: 'lightning', label: 'Lightning', description: 'BOLT12 receive work area. External offers now, in-app LDK later.' },
  { id: 'signatures', label: 'Signatures', description: 'Sign messages with the current wallet using BIP-322 Simple.' }
];

const networks: { id: BitcoinNetwork; label: string; hint: string }[] = [
  { id: 'bitcoin', label: 'Bitcoin mainnet', hint: 'Real BTC. Use carefully.' },
  { id: 'testnet', label: 'Testnet', hint: 'Best for testing sends.' },
  { id: 'signet', label: 'Signet', hint: 'Best for controlled wallet development.' }
];

function explorerBase(network: string) {
  if (network === 'testnet') return 'https://mempool.space/testnet';
  if (network === 'signet') return 'https://mempool.space/signet';
  return 'https://mempool.space';
}

function apiBase(network: string) {
  return `${explorerBase(network)}/api`;
}

function bitcoinUri(address: string, amountBtc?: string) {
  if (!address) return '';
  const trimmedAmount = amountBtc?.trim();
  return trimmedAmount ? `bitcoin:${address}?amount=${trimmedAmount}` : `bitcoin:${address}`;
}

async function fetchJsonWithError<T>(url: string): Promise<T> {
  const response = await fetch(url);
  if (!response.ok) {
    const body = await response.text().catch(() => '');
    throw new Error(`${response.status} ${response.statusText} from ${url}${body ? `: ${body.slice(0, 160)}` : ''}`);
  }
  return response.json() as Promise<T>;
}

function parseAddressBalance(raw: any, address: string, network: string, utxos: UtxoInfo[]): ChainBalance {
  const funded = Number(raw?.chain_stats?.funded_txo_sum || 0);
  const spent = Number(raw?.chain_stats?.spent_txo_sum || 0);
  const mempoolFunded = Number(raw?.mempool_stats?.funded_txo_sum || 0);
  const mempoolSpent = Number(raw?.mempool_stats?.spent_txo_sum || 0);
  const confirmed = funded - spent;
  const mempool = mempoolFunded - mempoolSpent;
  const txCount = Number(raw?.chain_stats?.tx_count || 0) + Number(raw?.mempool_stats?.tx_count || 0);
  return {
    address,
    network,
    confirmed_sats: confirmed,
    mempool_sats: mempool,
    total_sats: confirmed + mempool,
    tx_count: txCount,
    utxo_count: utxos.length,
    utxo_sats: utxos.reduce((sum, item) => sum + item.value, 0),
    explorer_url: `${explorerBase(network)}/address/${address}`,
    fetched_at: new Date().toLocaleString()
  };
}

function summarizeTx(raw: any, address: string, network: string): TxSummary {
  const received = (raw?.vout || [])
    .filter((output: any) => output?.scriptpubkey_address === address)
    .reduce((sum: number, output: any) => sum + Number(output?.value || 0), 0);
  const spent = (raw?.vin || [])
    .filter((input: any) => input?.prevout?.scriptpubkey_address === address)
    .reduce((sum: number, input: any) => sum + Number(input?.prevout?.value || 0), 0);
  return {
    txid: raw.txid,
    confirmed: Boolean(raw?.status?.confirmed),
    block_height: raw?.status?.block_height,
    block_time: raw?.status?.block_time,
    received_sats: received,
    spent_sats: spent,
    net_sats: received - spent,
    fee_sats: raw?.fee,
    url: `${explorerBase(network)}/tx/${raw.txid}`
  };
}

function formatSats(sats: number) {
  return `${sats.toLocaleString()} sats`;
}

function short(value: string, left = 12, right = 8) {
  if (!value) return '';
  if (value.length <= left + right + 3) return value;
  return `${value.slice(0, left)}…${value.slice(-right)}`;
}

function copy(value: string) {
  navigator.clipboard?.writeText(value);
}

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('wallet');
  const [status, setStatus] = useState('CarlosK Wallet v0.90 is running. Real features: wallet persistence, backend sync, balance lookup, UTXOs, local signed transaction creation, raw tx broadcast, BOLT12 storage, and BIP-322 signing.');

  const [wallet, setWallet] = useState<WalletInfo | null>(null);
  const [walletLabel, setWalletLabel] = useState('CarlosK');
  const [walletNetwork, setWalletNetwork] = useState<BitcoinNetwork>('signet');
  const [showSeed, setShowSeed] = useState(false);
  const [restoreSeed, setRestoreSeed] = useState('');

  const [backupPassphrase, setBackupPassphrase] = useState('');
  const [encryptedBackup, setEncryptedBackup] = useState<EncryptedBackup | null>(null);
  const [restoreBackupJson, setRestoreBackupJson] = useState('');
  const [restoreBackupPassphrase, setRestoreBackupPassphrase] = useState('');
  const [verifyBackupJson, setVerifyBackupJson] = useState('');
  const [verifyBackupPassphrase, setVerifyBackupPassphrase] = useState('');
  const [backupVerify, setBackupVerify] = useState<BackupVerifyResult | null>(null);

  const [persistPassphrase, setPersistPassphrase] = useState('');
  const [loadPersistPassphrase, setLoadPersistPassphrase] = useState('');
  const [persistStatus, setPersistStatus] = useState<PersistedWalletStatus | null>(null);

  const [receiveAddress, setReceiveAddress] = useState<ReceiveAddressInfo | null>(null);
  const [sendTo, setSendTo] = useState('');
  const [sendAmountSats, setSendAmountSats] = useState('1000');
  const [sendFeeRate, setSendFeeRate] = useState('5');
  const [sendDraft, setSendDraft] = useState<SendDraft | null>(null);
  const [backendSync, setBackendSync] = useState<BackendSyncReport | null>(null);
  const [signedTx, setSignedTx] = useState<SignedTransactionResult | null>(null);
  const [customEsploraUrl, setCustomEsploraUrl] = useState('');
  const [creatingSignedTx, setCreatingSignedTx] = useState(false);
  const [chainBalance, setChainBalance] = useState<ChainBalance | null>(null);
  const [utxos, setUtxos] = useState<UtxoInfo[]>([]);
  const [txHistory, setTxHistory] = useState<TxSummary[]>([]);
  const [feeEstimates, setFeeEstimates] = useState<FeeEstimates | null>(null);
  const [syncingChain, setSyncingChain] = useState(false);
  const [rawTxHex, setRawTxHex] = useState('');
  const [broadcastResult, setBroadcastResult] = useState<BroadcastResult | null>(null);
  const [receiveAmountBtc, setReceiveAmountBtc] = useState('');

  const [lightningAlias, setLightningAlias] = useState('CarlosK Lightning');
  const [lightningWallet, setLightningWallet] = useState<LightningWalletInfo | null>(null);
  const [bolt12Input, setBolt12Input] = useState('');
  const [bolt12Offer, setBolt12Offer] = useState<Bolt12OfferInfo | null>(null);
  const [inAppBolt12Error, setInAppBolt12Error] = useState('');

  const [messageToSign, setMessageToSign] = useState('');
  const [signatureAddress, setSignatureAddress] = useState('');
  const [signature, setSignature] = useState<SignatureResponse | null>(null);

  const activeDescription = useMemo(() => tabs.find((tab) => tab.id === activeTab)?.description || '', [activeTab]);
  const selectedNetwork = networks.find((network) => network.id === walletNetwork);

  useEffect(() => {
    refreshPersistenceStatus();
    invoke<WalletInfo | null>('get_current_wallet')
      .then((result) => {
        if (result) {
          setWallet(result);
          setSignatureAddress(result.address);
          if (result.network === 'bitcoin' || result.network === 'testnet' || result.network === 'signet') {
            setWalletNetwork(result.network);
          }
        }
      })
      .catch(() => undefined);
    const savedOffer = window.localStorage.getItem('carlosk-wallet.external-bolt12');
    if (savedOffer) {
      setBolt12Input(savedOffer);
    }
  }, []);

  async function refreshPersistenceStatus() {
    try {
      const result = await invoke<PersistedWalletStatus>('get_wallet_persistence_status');
      setPersistStatus(result);
    } catch (err) {
      setStatus(String(err));
    }
  }

  function syncWallet(result: WalletInfo, nextStatus: string) {
    setWallet(result);
    setWalletNetwork((result.network as BitcoinNetwork) || walletNetwork);
    setSignatureAddress(result.address);
    setReceiveAddress(null);
    setBackupVerify(null);
    setStatus(nextStatus);
  }

  async function createWallet() {
    try {
      setStatus('Creating new BTC wallet locally...');
      const result = await invoke<WalletInfo>('create_bitcoin_wallet', { label: walletLabel, network: walletNetwork });
      syncWallet(result, 'New BTC wallet created. Back up the seed phrase and save encrypted wallet storage before receiving funds.');
      setEncryptedBackup(null);
      setShowSeed(false);
    } catch (err) {
      setStatus(String(err));
    }
  }

  async function restoreWalletFromSeed() {
    try {
      if (!restoreSeed.trim()) {
        setStatus('Paste your 12-word seed phrase first.');
        return;
      }
      setStatus('Restoring BTC wallet locally...');
      const result = await invoke<WalletInfo>('restore_bitcoin_wallet', {
        mnemonic: restoreSeed.trim(),
        label: walletLabel,
        network: walletNetwork
      });
      syncWallet(result, 'Wallet restored. Verify the address and network match your backup records.');
    } catch (err) {
      setStatus(String(err));
    }
  }

  async function exportBackup() {
    try {
      if (!wallet) {
        setStatus('Create or restore a wallet first.');
        return;
      }
      setStatus('Encrypting wallet backup...');
      const result = await invoke<EncryptedBackup>('export_encrypted_wallet_backup', { passphrase: backupPassphrase });
      setEncryptedBackup(result);
      setVerifyBackupJson(JSON.stringify(result, null, 2));
      setStatus('Encrypted backup created. Store it offline and store the passphrase separately.');
    } catch (err) {
      setStatus(String(err));
    }
  }

  async function restoreBackup() {
    try {
      if (!restoreBackupJson.trim()) {
        setStatus('Paste encrypted backup JSON first.');
        return;
      }
      setStatus('Decrypting backup and restoring wallet...');
      const result = await invoke<WalletInfo>('restore_encrypted_wallet_backup', {
        backupJson: restoreBackupJson.trim(),
        passphrase: restoreBackupPassphrase,
        label: walletLabel
      });
      syncWallet(result, 'Encrypted backup restored successfully.');
    } catch (err) {
      setStatus(String(err));
    }
  }

  async function verifyBackup() {
    try {
      if (!verifyBackupJson.trim()) {
        setStatus('Paste encrypted backup JSON to verify first.');
        return;
      }
      setStatus('Decrypting backup for restore-proof verification...');
      const result = await invoke<BackupVerifyResult>('verify_encrypted_wallet_backup', {
        backupJson: verifyBackupJson.trim(),
        passphrase: verifyBackupPassphrase
      });
      setBackupVerify(result);
      setStatus(result.status);
    } catch (err) {
      setStatus(String(err));
    }
  }

  async function saveWalletToDisk() {
    try {
      if (!wallet) {
        setStatus('Create or restore a wallet first.');
        return;
      }
      setStatus('Saving encrypted wallet to local disk...');
      const result = await invoke<PersistedWalletStatus>('save_wallet_to_disk', { passphrase: persistPassphrase });
      setPersistStatus(result);
      setStatus(result.status);
    } catch (err) {
      setStatus(String(err));
    }
  }

  async function loadWalletFromDisk() {
    try {
      setStatus('Loading encrypted wallet from local disk...');
      const result = await invoke<WalletInfo>('load_wallet_from_disk', { passphrase: loadPersistPassphrase, label: walletLabel });
      syncWallet(result, 'Saved encrypted wallet loaded from local disk.');
      await refreshPersistenceStatus();
    } catch (err) {
      setStatus(String(err));
    }
  }

  async function deleteSavedWallet() {
    try {
      const result = await invoke<PersistedWalletStatus>('delete_saved_wallet');
      setPersistStatus(result);
      setStatus(result.status);
    } catch (err) {
      setStatus(String(err));
    }
  }

  async function generateReceiveAddress() {
    try {
      setStatus('Generating a new BTC receive address...');
      const result = await invoke<ReceiveAddressInfo>('generate_receive_address');
      setReceiveAddress(result);
      setStatus(`Generated ${result.network} receive address index ${result.index}. Save encrypted wallet storage to persist this index after restart.`);
    } catch (err) {
      setStatus(String(err));
    }
  }

  async function createSendDraft() {
    try {
      setStatus('Validating send draft against the current wallet network...');
      const result = await invoke<SendDraft>('create_send_draft', {
        input: {
          to_address: sendTo.trim(),
          amount_sats: Number(sendAmountSats),
          fee_rate_sat_vb: Number(sendFeeRate)
        }
      });
      setSendDraft(result);
      setStatus(result.status);
    } catch (err) {
      setStatus(String(err));
    }
  }

  async function syncBackendWallet() {
    try {
      if (!wallet) {
        setStatus('Create or restore a wallet first.');
        return;
      }
      setStatus('Running backend wallet sync with Esplora. This may take a moment...');
      const result = await invoke<BackendSyncReport>('sync_wallet_backend', {
        esploraUrl: customEsploraUrl.trim() || null
      });
      setBackendSync(result);
      setStatus(result.status);
    } catch (err) {
      setStatus(`Backend wallet sync failed: ${String(err)}`);
    }
  }

  async function createSignedSendTransaction() {
    try {
      if (!wallet) {
        setStatus('Create or restore a wallet first.');
        return;
      }
      if (!backendSync) {
        setStatus('Run Backend Sync first so the wallet knows its spendable UTXOs.');
        return;
      }
      setCreatingSignedTx(true);
      setStatus('Building PSBT, signing locally, and extracting signed transaction...');
      const result = await invoke<SignedTransactionResult>('create_signed_send_transaction', {
        input: {
          to_address: sendTo.trim(),
          amount_sats: Number(sendAmountSats),
          fee_rate_sat_vb: Number(sendFeeRate)
        }
      });
      setSignedTx(result);
      setRawTxHex(result.tx_hex);
      setStatus(result.status);
    } catch (err) {
      setStatus(`Signed transaction creation failed: ${String(err)}`);
    } finally {
      setCreatingSignedTx(false);
    }
  }

  async function syncChainForCurrentAddress() {
    try {
      const address = receiveAddress?.address || wallet?.address || '';
      if (!wallet || !address) {
        setStatus('Create or restore a wallet first, then generate or use a receive address.');
        return;
      }
      setSyncingChain(true);
      setStatus(`Syncing ${wallet.network} address with public Esplora API...`);
      const base = apiBase(wallet.network);
      const [stats, rawUtxos, rawTxs] = await Promise.all([
        fetchJsonWithError<any>(`${base}/address/${address}`),
        fetchJsonWithError<any[]>(`${base}/address/${address}/utxo`),
        fetchJsonWithError<any[]>(`${base}/address/${address}/txs`).catch(() => [])
      ]);
      const mappedUtxos: UtxoInfo[] = rawUtxos.map((item) => ({
        txid: item.txid,
        vout: Number(item.vout),
        value: Number(item.value),
        confirmed: Boolean(item?.status?.confirmed),
        block_height: item?.status?.block_height
      }));
      const balance = parseAddressBalance(stats, address, wallet.network, mappedUtxos);
      setChainBalance(balance);
      setUtxos(mappedUtxos);
      setTxHistory(rawTxs.slice(0, 25).map((tx) => summarizeTx(tx, address, wallet.network)));
      setStatus(`Synced ${address}. Total detected: ${formatSats(balance.total_sats)}.`);
    } catch (err) {
      setStatus(`Chain sync failed: ${String(err)}`);
    } finally {
      setSyncingChain(false);
    }
  }

  async function loadFeeEstimates() {
    try {
      const network = wallet?.network || walletNetwork;
      setStatus(`Loading ${network} fee estimates...`);
      const result = await fetchJsonWithError<FeeEstimates>(`${apiBase(network)}/v1/fees/recommended`);
      setFeeEstimates(result);
      const suggested = result.halfHourFee || result.hourFee || result.fastestFee || result.minimumFee;
      if (suggested) setSendFeeRate(String(suggested));
      setStatus('Fee estimates loaded from mempool.space. Review fee rate before signing anything.');
    } catch (err) {
      setStatus(`Fee lookup failed: ${String(err)}`);
    }
  }

  async function broadcastRawTransaction() {
    try {
      const network = wallet?.network || walletNetwork;
      const hex = rawTxHex.trim();
      if (!hex) {
        setStatus('Paste a signed raw transaction hex first.');
        return;
      }
      if (!/^[0-9a-fA-F]+$/.test(hex) || hex.length < 20) {
        setStatus('Raw transaction must be hex text. Do not paste PSBT base64 here.');
        return;
      }
      setStatus(`Broadcasting raw transaction to ${network}...`);
      const response = await fetch(`${apiBase(network)}/tx`, {
        method: 'POST',
        headers: { 'Content-Type': 'text/plain' },
        body: hex
      });
      const body = await response.text();
      if (!response.ok) {
        throw new Error(`${response.status} ${response.statusText}: ${body.slice(0, 300)}`);
      }
      const txid = body.trim();
      const result = { txid, url: `${explorerBase(network)}/tx/${txid}`, status: 'Transaction broadcast accepted by the Esplora server.' };
      setBroadcastResult(result);
      setStatus(result.status);
    } catch (err) {
      setStatus(`Broadcast failed: ${String(err)}`);
    }
  }

  async function createLightningWallet() {
    try {
      setStatus('Creating Lightning wallet profile...');
      const result = await invoke<LightningWalletInfo>('create_lightning_wallet', { alias: lightningAlias });
      setLightningWallet(result);
      setStatus(result.status);
    } catch (err) {
      setStatus(String(err));
    }
  }

  async function saveBolt12Offer() {
    try {
      setStatus('Validating BOLT12 offer...');
      const result = await invoke<Bolt12OfferInfo>('save_bolt12_offer', { offer: bolt12Input });
      setBolt12Offer(result);
      window.localStorage.setItem('carlosk-wallet.external-bolt12', result.offer);
      setStatus(result.status);
    } catch (err) {
      setStatus(String(err));
    }
  }

  async function requestInAppBolt12Offer() {
    try {
      setInAppBolt12Error('');
      setStatus('Requesting in-app BOLT12 offer...');
      const result = await invoke<Bolt12OfferInfo>('create_in_app_bolt12_offer');
      setBolt12Offer(result);
      window.localStorage.setItem('carlosk-wallet.external-bolt12', result.offer);
      setStatus(result.status);
    } catch (err) {
      setInAppBolt12Error(String(err));
      setStatus(String(err));
    }
  }

  async function signMessage() {
    try {
      if (!messageToSign.trim()) {
        setStatus('Enter a message to sign first.');
        return;
      }
      const address = signatureAddress.trim() || wallet?.address || '';
      if (!address) {
        setStatus('Create a wallet or enter an address first.');
        return;
      }
      setStatus('Signing message locally...');
      const result = await invoke<SignatureResponse>('sign_message', { message: messageToSign, address });
      setSignature(result);
      setStatus('Message signed with BIP-322 Simple.');
    } catch (err) {
      setStatus(String(err));
    }
  }

  return (
    <main>
      <section className="hero">
        <div>
          <p className="eyebrow">v0.90 Wallet MVP</p>
          <h1>CarlosK Wallet</h1>
          <p className="subtitle">A simple self-custody Bitcoin + BOLT12 desktop wallet. v0.90 adds backend wallet sync plus real local PSBT signing and signed transaction creation on top of balance lookup, UTXOs, history, and raw broadcast.</p>
        </div>
        <div className="status-card">
          <strong>Status</strong>
          <p>{status}</p>
        </div>
      </section>

      <nav className="tabs">
        {tabs.map((tab) => (
          <button key={tab.id} className={activeTab === tab.id ? 'active' : ''} onClick={() => setActiveTab(tab.id)}>
            {tab.label}
          </button>
        ))}
      </nav>

      <p className="tab-description">{activeDescription}</p>

      {activeTab === 'wallet' && (
        <section className="grid two">
          <div className="card">
            <h2>Create wallet</h2>
            <label>Wallet label<input value={walletLabel} onChange={(e) => setWalletLabel(e.target.value)} /></label>
            <label>Network
              <select value={walletNetwork} onChange={(e) => setWalletNetwork(e.target.value as BitcoinNetwork)}>
                {networks.map((network) => <option key={network.id} value={network.id}>{network.label}</option>)}
              </select>
            </label>
            <p className="muted">{selectedNetwork?.hint}</p>
            <button onClick={createWallet}>Create New BTC Wallet</button>
            {wallet && (
              <div className="result">
                <p><strong>{wallet.label}</strong> · {wallet.network} · next index {wallet.next_external_index}</p>
                <p><strong>Current receive address</strong></p>
                <code>{wallet.address}</code>
                <p className="muted">Derivation: {wallet.derivation}</p>
                <div className="row"><button className="secondary" onClick={() => copy(wallet.address)}>Copy Address</button><button className="secondary" onClick={() => setShowSeed(!showSeed)}>{showSeed ? 'Hide Seed' : 'Show Seed'}</button></div>
                {showSeed && <textarea readOnly value={wallet.mnemonic} />}
                <p className="warning">{wallet.warning}</p>
              </div>
            )}
          </div>

          <div className="card">
            <h2>Restore wallet from seed</h2>
            <p className="muted">Choose the same network that was used when the wallet was created.</p>
            <label>12-word seed phrase<textarea value={restoreSeed} onChange={(e) => setRestoreSeed(e.target.value)} placeholder="Paste seed phrase only on your own local machine..." /></label>
            <button onClick={restoreWalletFromSeed}>Restore From Seed</button>
          </div>

          <div className="card">
            <h2>Encrypted local storage</h2>
            <p className="muted">This is the first real persistence feature. It saves an encrypted wallet file on your Mac and loads it after restart using your passphrase.</p>
            <label>Save/unlock passphrase<input type="password" value={persistPassphrase} onChange={(e) => setPersistPassphrase(e.target.value)} placeholder="At least 12 characters" /></label>
            <div className="row"><button onClick={saveWalletToDisk}>Save Encrypted Wallet To Disk</button><button className="secondary" onClick={refreshPersistenceStatus}>Refresh Status</button></div>
            <label>Load passphrase<input type="password" value={loadPersistPassphrase} onChange={(e) => setLoadPersistPassphrase(e.target.value)} placeholder="Passphrase used when saving" /></label>
            <div className="row"><button onClick={loadWalletFromDisk}>Load Saved Wallet</button><button className="danger" onClick={deleteSavedWallet}>Delete Saved Wallet File</button></div>
            {persistStatus && (
              <div className={persistStatus.exists ? 'result' : 'empty'}>
                <p><strong>{persistStatus.exists ? 'Saved wallet exists' : 'No saved wallet file'}</strong></p>
                <code>{persistStatus.path}</code>
                <p className="warning">{persistStatus.warning}</p>
              </div>
            )}
          </div>

          <div className="card">
            <h2>Export encrypted backup</h2>
            <label>Backup passphrase<input type="password" value={backupPassphrase} onChange={(e) => setBackupPassphrase(e.target.value)} placeholder="At least 12 characters" /></label>
            <button onClick={exportBackup}>Export Encrypted Backup</button>
            {encryptedBackup && (
              <div className="result">
                <div className="row"><button className="secondary" onClick={() => copy(JSON.stringify(encryptedBackup, null, 2))}>Copy Backup JSON</button></div>
                <textarea readOnly value={JSON.stringify(encryptedBackup, null, 2)} />
                <p className="warning">{encryptedBackup.warning}</p>
              </div>
            )}
          </div>

          <div className="card">
            <h2>Verify encrypted backup</h2>
            <p className="muted">Proof that the backup decrypts to the same wallet address before you trust it.</p>
            <label>Backup JSON<textarea value={verifyBackupJson} onChange={(e) => setVerifyBackupJson(e.target.value)} /></label>
            <label>Passphrase<input type="password" value={verifyBackupPassphrase} onChange={(e) => setVerifyBackupPassphrase(e.target.value)} /></label>
            <button onClick={verifyBackup}>Verify Backup Matches Current Wallet</button>
            {backupVerify && (
              <div className={backupVerify.same_address ? 'result' : 'warning'}>
                <p><strong>{backupVerify.status}</strong></p>
                <p>Current: <code>{short(backupVerify.current_address)}</code></p>
                <p>Backup: <code>{short(backupVerify.backup_address)}</code></p>
                <p>Network: {backupVerify.network} · saved next index {backupVerify.next_external_index}</p>
              </div>
            )}
          </div>

          <div className="card">
            <h2>Restore encrypted backup</h2>
            <label>Backup JSON<textarea value={restoreBackupJson} onChange={(e) => setRestoreBackupJson(e.target.value)} /></label>
            <label>Passphrase<input type="password" value={restoreBackupPassphrase} onChange={(e) => setRestoreBackupPassphrase(e.target.value)} /></label>
            <button onClick={restoreBackup}>Restore Encrypted Backup</button>
          </div>
        </section>
      )}

      {activeTab === 'send-receive' && (
        <section className="grid two">
          <div className="card">
            <h2>Receive BTC on-chain</h2>
            <p>Generate a fresh native SegWit receive address from your local wallet.</p>
            <button onClick={generateReceiveAddress}>Generate Receive Address</button>
            {receiveAddress && (
              <div className="result">
                <p><strong>{receiveAddress.network} address index {receiveAddress.index}</strong></p>
                <code>{receiveAddress.address}</code>
                <div className="row"><button className="secondary" onClick={() => copy(receiveAddress.address)}>Copy Receive Address</button></div>
                <p className="warning">{receiveAddress.warning}</p>
              </div>
            )}
          </div>

          <div className="card">
            <h2>Real chain sync</h2>
            <p className="muted">Looks up the active receive address using public mempool.space Esplora APIs. This gives real balance, UTXOs, and recent transaction history for mainnet, testnet, or signet.</p>
            <p>Active address: <code>{short(receiveAddress?.address || wallet?.address || 'no wallet')}</code></p>
            <div className="row"><button onClick={syncChainForCurrentAddress} disabled={syncingChain}>{syncingChain ? 'Syncing...' : 'Sync Balance / UTXOs / Txs'}</button><button className="secondary" onClick={loadFeeEstimates}>Load Fee Estimates</button></div>
            {chainBalance && (
              <div className="result">
                <p><strong>Total:</strong> {formatSats(chainBalance.total_sats)}</p>
                <p>Confirmed: {formatSats(chainBalance.confirmed_sats)} · Mempool: {formatSats(chainBalance.mempool_sats)}</p>
                <p>UTXOs: {chainBalance.utxo_count} · UTXO value: {formatSats(chainBalance.utxo_sats)} · Txs: {chainBalance.tx_count}</p>
                <p className="muted">Fetched: {chainBalance.fetched_at}</p>
                <div className="row"><button className="secondary" onClick={() => copy(chainBalance.explorer_url)}>Copy Explorer URL</button><a className="button-link" href={chainBalance.explorer_url} target="_blank" rel="noreferrer">Open Explorer</a></div>
              </div>
            )}
            {feeEstimates && (
              <div className="result compact">
                <p><strong>Fee estimates</strong></p>
                <p>Fast: {feeEstimates.fastestFee ?? '?'} sat/vB · 30 min: {feeEstimates.halfHourFee ?? '?'} sat/vB · 1 hr: {feeEstimates.hourFee ?? '?'} sat/vB · Min: {feeEstimates.minimumFee ?? '?'} sat/vB</p>
              </div>
            )}
          </div>

          <div className="card">
            <h2>UTXOs</h2>
            {utxos.length ? (
              <div className="scroll-list">
                {utxos.map((utxo) => (
                  <div className="mini-row" key={`${utxo.txid}:${utxo.vout}`}>
                    <div><strong>{formatSats(utxo.value)}</strong><br /><span>{utxo.confirmed ? `Confirmed ${utxo.block_height || ''}` : 'Unconfirmed'}</span></div>
                    <code>{short(utxo.txid, 10, 10)}:{utxo.vout}</code>
                  </div>
                ))}
              </div>
            ) : <p className="muted">No UTXOs loaded yet. Sync the active address first.</p>}
          </div>

          <div className="card wide">
            <h2>Transaction history</h2>
            {txHistory.length ? (
              <div className="scroll-list">
                {txHistory.map((tx) => (
                  <div className="mini-row" key={tx.txid}>
                    <div><strong className={tx.net_sats >= 0 ? 'positive' : 'negative'}>{tx.net_sats >= 0 ? '+' : ''}{formatSats(tx.net_sats)}</strong><br /><span>{tx.confirmed ? `Confirmed ${tx.block_height || ''}` : 'Unconfirmed'} · fee {tx.fee_sats ?? '?'} sats</span></div>
                    <div className="row"><code>{short(tx.txid, 10, 10)}</code><button className="secondary small" onClick={() => copy(tx.url)}>Copy URL</button></div>
                  </div>
                ))}
              </div>
            ) : <p className="muted">No transactions loaded yet. Sync the active address first.</p>}
          </div>

          <div className="card">
            <h2>Receive URI</h2>
            <p className="muted">Use this for payment requests. QR rendering comes next, but the URI itself is real and copyable.</p>
            <label>Optional amount BTC<input value={receiveAmountBtc} onChange={(e) => setReceiveAmountBtc(e.target.value)} placeholder="0.0001" /></label>
            <code>{bitcoinUri(receiveAddress?.address || wallet?.address || '', receiveAmountBtc) || 'Create a wallet first'}</code>
            <div className="row"><button className="secondary" onClick={() => copy(bitcoinUri(receiveAddress?.address || wallet?.address || '', receiveAmountBtc))}>Copy Bitcoin URI</button></div>
          </div>

          <div className="card">
            <h2>Backend wallet sync</h2>
            <p className="muted">This is the real BDK/Esplora sync used by the signer. Run this after funding your wallet and before creating a signed send transaction.</p>
            <label>Optional custom Esplora API URL<input value={customEsploraUrl} onChange={(e) => setCustomEsploraUrl(e.target.value)} placeholder={apiBase(wallet?.network || walletNetwork)} /></label>
            <button onClick={syncBackendWallet}>Backend Sync Wallet</button>
            {backendSync && (
              <div className="result">
                <p><strong>{backendSync.status}</strong></p>
                <p>Confirmed: {formatSats(backendSync.confirmed_sats)} · Pending: {formatSats(backendSync.pending_sats)} · Total: {formatSats(backendSync.total_sats)}</p>
                <p>UTXOs: {backendSync.utxo_count} · UTXO value: {formatSats(backendSync.utxo_sats)}</p>
                <p>Network: {backendSync.network}</p>
                <code>{backendSync.esplora_url}</code>
                <p className="warning">{backendSync.warning}</p>
              </div>
            )}
          </div>

          <div className="card">
            <h2>Send BTC on-chain</h2>
            <p className="warning">v0.90 can create a real signed wallet transaction after Backend Sync. Test with signet/testnet first. Broadcasting spends funds.</p>
            <label>Recipient BTC address<input value={sendTo} onChange={(e) => setSendTo(e.target.value)} /></label>
            <label>Amount sats<input value={sendAmountSats} onChange={(e) => setSendAmountSats(e.target.value)} /></label>
            <label>Fee rate sat/vB<input value={sendFeeRate} onChange={(e) => setSendFeeRate(e.target.value)} /></label>
            <div className="row"><button onClick={createSendDraft}>Validate Send Draft</button><button className="danger" onClick={createSignedSendTransaction} disabled={creatingSignedTx}>{creatingSignedTx ? 'Signing...' : 'Build & Sign Transaction'}</button></div>
            {sendDraft && (
              <div className="result">
                <p><strong>{sendDraft.status}</strong></p>
                <p>To: <code>{short(sendDraft.to_address)}</code></p>
                <p>{sendDraft.amount_sats} sats · {sendDraft.fee_rate_sat_vb} sat/vB</p>
                <ul>{sendDraft.next_steps.map((step) => <li key={step}>{step}</li>)}</ul>
                <p className="warning">{sendDraft.warning}</p>
              </div>
            )}
            {signedTx && (
              <div className="result">
                <p><strong>{signedTx.status}</strong></p>
                <p>TXID: <code>{short(signedTx.txid, 16, 16)}</code></p>
                <p>Recipient: <code>{short(signedTx.recipient)}</code></p>
                <p>Amount: {formatSats(signedTx.amount_sats)} · Fee: {formatSats(signedTx.fee_sats)} · Fee rate: {signedTx.fee_rate_sat_vb} sat/vB</p>
                <p>Finalized: {signedTx.finalized ? 'Yes' : 'No'} · Ready to broadcast: {signedTx.ready_to_broadcast ? 'Yes' : 'No'}</p>
                <textarea readOnly value={signedTx.tx_hex} />
                <div className="row"><button className="secondary" onClick={() => copy(signedTx.tx_hex)}>Copy Signed TX Hex</button><button className="secondary" onClick={() => setRawTxHex(signedTx.tx_hex)}>Load Into Broadcast Box</button></div>
                <p className="warning">{signedTx.warning}</p>
              </div>
            )}
          </div>

          <div className="card wide">
            <h2>Broadcast signed raw transaction</h2>
            <p className="warning">This is real broadcast. Paste only a raw transaction hex you created and reviewed intentionally. Do not paste a PSBT here.</p>
            <label>Signed raw transaction hex<textarea value={rawTxHex} onChange={(e) => setRawTxHex(e.target.value)} placeholder="020000000001..." /></label>
            <button className="danger" onClick={broadcastRawTransaction}>Broadcast Raw Transaction</button>
            {broadcastResult && (
              <div className="result">
                <p><strong>{broadcastResult.status}</strong></p>
                <code>{broadcastResult.txid}</code>
                <div className="row"><button className="secondary" onClick={() => copy(broadcastResult.txid)}>Copy TXID</button><a className="button-link" href={broadcastResult.url} target="_blank" rel="noreferrer">Open Transaction</a></div>
              </div>
            )}
          </div>
        </section>
      )}

      {activeTab === 'lightning' && (
        <section className="grid two">
          <div className="card">
            <h2>Lightning wallet</h2>
            <label>Alias<input value={lightningAlias} onChange={(e) => setLightningAlias(e.target.value)} /></label>
            <button onClick={createLightningWallet}>Create Lightning Wallet Profile</button>
            {lightningWallet && (
              <div className="result">
                <p><strong>{lightningWallet.alias}</strong></p>
                <p>{lightningWallet.status}</p>
                <p>Network: {lightningWallet.network}</p>
                <p>In-app BOLT12 receive: {lightningWallet.can_receive_bolt12_in_app ? 'Enabled' : 'Locked'}</p>
                <ul>{lightningWallet.next_steps.map((step) => <li key={step}>{step}</li>)}</ul>
                <p className="warning">{lightningWallet.warning}</p>
              </div>
            )}
          </div>

          <div className="card">
            <h2>BOLT12 receive</h2>
            <p>For v0.90, save and persist an external BOLT12 offer. In-app BOLT12 generation remains locked until LDK signet/testnet receive and channel recovery are proven.</p>
            <label>External BOLT12 offer<textarea value={bolt12Input} onChange={(e) => setBolt12Input(e.target.value)} placeholder="lno1..." /></label>
            <div className="row"><button onClick={saveBolt12Offer}>Save External BOLT12 Offer</button><button className="secondary" onClick={requestInAppBolt12Offer}>Try In-App BOLT12</button></div>
            {bolt12Offer && (
              <div className="result">
                <p><strong>{bolt12Offer.status}</strong></p>
                <code>{short(bolt12Offer.offer, 24, 16)}</code>
                <div className="row"><button className="secondary" onClick={() => copy(bolt12Offer.offer)}>Copy BOLT12 Offer</button></div>
                <p className="warning">{bolt12Offer.warning}</p>
              </div>
            )}
            {inAppBolt12Error && <p className="warning">{inAppBolt12Error}</p>}
          </div>
        </section>
      )}

      {activeTab === 'signatures' && (
        <section className="grid two">
          <div className="card">
            <h2>Sign message</h2>
            <label>Address<input value={signatureAddress || wallet?.address || ''} onChange={(e) => setSignatureAddress(e.target.value)} placeholder="bc1 / tb1..." /></label>
            <label>Message<textarea value={messageToSign} onChange={(e) => setMessageToSign(e.target.value)} placeholder="Paste the message to sign..." /></label>
            <button onClick={signMessage}>Sign Message</button>
          </div>
          <div className="card">
            <h2>Signature output</h2>
            {signature ? (
              <div className="result">
                <p>{signature.format}</p>
                <p>Address: <code>{short(signature.address)}</code></p>
                <textarea readOnly value={signature.signature} />
                <div className="row"><button className="secondary" onClick={() => copy(signature.signature)}>Copy Signature</button></div>
              </div>
            ) : (
              <p className="muted">No signature yet.</p>
            )}
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
