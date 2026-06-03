import React, { useMemo, useState } from 'react';
import ReactDOM from 'react-dom/client';
import { invoke } from '@tauri-apps/api/core';
import './styles.css';

type Tab = 'wallet' | 'send-receive' | 'lightning' | 'signatures';

type WalletInfo = {
  mnemonic: string;
  address: string;
  network: string;
  derivation: string;
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

type SendDraft = {
  to_address: string;
  amount_sats: number;
  fee_rate_sat_vb: number;
  ready_to_broadcast: boolean;
  status: string;
  next_steps: string[];
  warning: string;
};

type SignatureResponse = {
  signature: string;
  address: string;
  format: string;
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
  { id: 'wallet', label: 'Wallet', description: 'Create, restore, and back up your BTC wallet.' },
  { id: 'send-receive', label: 'Send / Receive', description: 'Generate addresses and prepare sends.' },
  { id: 'lightning', label: 'Lightning', description: 'BOLT12 wallet work area.' },
  { id: 'signatures', label: 'Signatures', description: 'Sign messages with your BTC address.' }
];

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
  const [status, setStatus] = useState('CarlosK Wallet v0.27 Core Wallet Reset is running.');

  const [wallet, setWallet] = useState<WalletInfo | null>(null);
  const [walletLabel, setWalletLabel] = useState('CarlosK');
  const [showSeed, setShowSeed] = useState(false);
  const [restoreSeed, setRestoreSeed] = useState('');
  const [backupPassphrase, setBackupPassphrase] = useState('');
  const [encryptedBackup, setEncryptedBackup] = useState<EncryptedBackup | null>(null);
  const [restoreBackupJson, setRestoreBackupJson] = useState('');
  const [restoreBackupPassphrase, setRestoreBackupPassphrase] = useState('');

  const [receiveAddress, setReceiveAddress] = useState<ReceiveAddressInfo | null>(null);
  const [sendTo, setSendTo] = useState('');
  const [sendAmountSats, setSendAmountSats] = useState('1000');
  const [sendFeeRate, setSendFeeRate] = useState('5');
  const [sendDraft, setSendDraft] = useState<SendDraft | null>(null);

  const [lightningAlias, setLightningAlias] = useState('CarlosK Lightning');
  const [lightningWallet, setLightningWallet] = useState<LightningWalletInfo | null>(null);
  const [bolt12Input, setBolt12Input] = useState('');
  const [bolt12Offer, setBolt12Offer] = useState<Bolt12OfferInfo | null>(null);
  const [inAppBolt12Error, setInAppBolt12Error] = useState('');

  const [messageToSign, setMessageToSign] = useState('');
  const [signatureAddress, setSignatureAddress] = useState('');
  const [signature, setSignature] = useState<SignatureResponse | null>(null);

  const activeDescription = useMemo(() => tabs.find((tab) => tab.id === activeTab)?.description || '', [activeTab]);

  async function createWallet() {
    try {
      setStatus('Creating new BTC wallet locally...');
      const result = await invoke<WalletInfo>('create_bitcoin_wallet', { label: walletLabel });
      setWallet(result);
      setSignatureAddress(result.address);
      setReceiveAddress(null);
      setEncryptedBackup(null);
      setStatus('New BTC wallet created. Back up the seed phrase before receiving funds.');
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
        label: walletLabel
      });
      setWallet(result);
      setSignatureAddress(result.address);
      setReceiveAddress(null);
      setStatus('Wallet restored. Verify the address matches your backup records.');
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
      const result = await invoke<EncryptedBackup>('export_encrypted_wallet_backup', {
        passphrase: backupPassphrase
      });
      setEncryptedBackup(result);
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
      setWallet(result);
      setSignatureAddress(result.address);
      setStatus('Encrypted backup restored successfully.');
    } catch (err) {
      setStatus(String(err));
    }
  }

  async function generateReceiveAddress() {
    try {
      setStatus('Generating a new BTC receive address...');
      const result = await invoke<ReceiveAddressInfo>('generate_receive_address');
      setReceiveAddress(result);
      setStatus(`Generated receive address index ${result.index}.`);
    } catch (err) {
      setStatus(String(err));
    }
  }

  async function createSendDraft() {
    try {
      setStatus('Validating send draft...');
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

  async function createLightningWallet() {
    try {
      setStatus('Creating Lightning wallet profile...');
      const result = await invoke<LightningWalletInfo>('create_lightning_wallet', {
        alias: lightningAlias
      });
      setLightningWallet(result);
      setStatus(result.status);
    } catch (err) {
      setStatus(String(err));
    }
  }

  async function saveBolt12Offer() {
    try {
      setStatus('Validating BOLT12 offer...');
      const result = await invoke<Bolt12OfferInfo>('save_bolt12_offer', {
        offer: bolt12Input
      });
      setBolt12Offer(result);
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
      const result = await invoke<SignatureResponse>('sign_message', {
        message: messageToSign,
        address
      });
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
          <p className="eyebrow">v0.27 Core Wallet Reset</p>
          <h1>CarlosK Wallet</h1>
          <p className="subtitle">A simple self-custody Bitcoin + BOLT12 desktop wallet. The old miner dashboards and OCEAN setup clutter are removed from the main UI.</p>
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
            <button onClick={createWallet}>Create New BTC Wallet</button>
            {wallet && (
              <div className="result">
                <p><strong>Current address</strong></p>
                <code>{wallet.address}</code>
                <div className="row"><button className="secondary" onClick={() => copy(wallet.address)}>Copy Address</button><button className="secondary" onClick={() => setShowSeed(!showSeed)}>{showSeed ? 'Hide Seed' : 'Show Seed'}</button></div>
                {showSeed && <textarea readOnly value={wallet.mnemonic} />}
                <p className="warning">{wallet.warning}</p>
              </div>
            )}
          </div>

          <div className="card">
            <h2>Restore wallet</h2>
            <label>12-word seed phrase<textarea value={restoreSeed} onChange={(e) => setRestoreSeed(e.target.value)} placeholder="Paste seed phrase only on your own local machine..." /></label>
            <button onClick={restoreWalletFromSeed}>Restore From Seed</button>
          </div>

          <div className="card">
            <h2>Encrypted backup</h2>
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
                <p><strong>Address index {receiveAddress.index}</strong></p>
                <code>{receiveAddress.address}</code>
                <div className="row"><button className="secondary" onClick={() => copy(receiveAddress.address)}>Copy Receive Address</button></div>
                <p className="warning">{receiveAddress.warning}</p>
              </div>
            )}
          </div>

          <div className="card">
            <h2>Send BTC on-chain</h2>
            <p className="warning">v0.27 validates a send draft only. Real transaction creation, signing, sync, and broadcast are the next phase.</p>
            <label>Recipient BTC address<input value={sendTo} onChange={(e) => setSendTo(e.target.value)} /></label>
            <label>Amount sats<input value={sendAmountSats} onChange={(e) => setSendAmountSats(e.target.value)} /></label>
            <label>Fee rate sat/vB<input value={sendFeeRate} onChange={(e) => setSendFeeRate(e.target.value)} /></label>
            <button onClick={createSendDraft}>Validate Send Draft</button>
            {sendDraft && (
              <div className="result">
                <p><strong>{sendDraft.status}</strong></p>
                <p>To: <code>{short(sendDraft.to_address)}</code></p>
                <p>{sendDraft.amount_sats} sats · {sendDraft.fee_rate_sat_vb} sat/vB</p>
                <ul>{sendDraft.next_steps.map((step) => <li key={step}>{step}</li>)}</ul>
                <p className="warning">{sendDraft.warning}</p>
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
            <p>For now, you can save an external BOLT12 offer. In-app BOLT12 generation will be built through LDK on signet/testnet first.</p>
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
            <label>Address<input value={signatureAddress || wallet?.address || ''} onChange={(e) => setSignatureAddress(e.target.value)} placeholder="bc1..." /></label>
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
