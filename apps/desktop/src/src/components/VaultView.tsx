import { useState, useEffect } from 'react';
import { tauriApi, type VaultStatus } from '../lib/tauri';

interface VaultDomain {
  name: string;
  icon: string;
  count: number;
  size: string;
}

const MOCK_DOMAINS: VaultDomain[] = [
  { name: 'Memories', icon: '🧠', count: 47, size: '2.1 MB' },
  { name: 'Chats', icon: '💬', count: 128, size: '4.8 MB' },
  { name: 'Recordings', icon: '🎙️', count: 12, size: '840 MB' },
  { name: 'Documents', icon: '📄', count: 34, size: '156 MB' },
  { name: 'Settings', icon: '⚙️', count: 8, size: '24 KB' },
  { name: 'Audit', icon: '📋', count: 1847, size: '1.2 MB' },
];

export function VaultView() {
  const [vaultStatus, setVaultStatus] = useState<VaultStatus | null>(null);

  useEffect(() => {
    tauriApi.getVaultStatus().then(setVaultStatus).catch(() => {});
  }, []);

  const usedPercent = vaultStatus
    ? Math.round((vaultStatus.used_space_gb / vaultStatus.total_space_gb) * 100)
    : 0;

  return (
    <div className="vault-view">
      <div className="main-header">
        <h2>Vault</h2>
        <div className="main-header-actions">
          <button className="btn btn-secondary btn-sm">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polyline points="1 4 1 10 7 10" />
              <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
            </svg>
            Sync
          </button>
          <button className="btn btn-danger btn-sm">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
              <path d="M7 11V7a5 5 0 0 1 10 0v4" />
            </svg>
            Emergency Lock
          </button>
        </div>
      </div>

      <div className="main-body">
        <div className="vault-stats">
          <div className="vault-stat-card">
            <h4>Status</h4>
            <div className="value" style={{ color: vaultStatus?.is_unlocked ? 'var(--success)' : 'var(--danger)' }}>
              {vaultStatus?.is_unlocked ? 'Unlocked' : vaultStatus?.is_connected ? 'Locked' : 'Disconnected'}
            </div>
          </div>
          <div className="vault-stat-card">
            <h4>Used Space</h4>
            <div className="value">
              {vaultStatus?.used_space_gb.toFixed(1) || '0.0'}
              <span className="unit">GB</span>
            </div>
          </div>
          <div className="vault-stat-card">
            <h4>Total Space</h4>
            <div className="value">
              {vaultStatus?.total_space_gb.toFixed(0) || '460'}
              <span className="unit">GB</span>
            </div>
          </div>
          <div className="vault-stat-card">
            <h4>Usage</h4>
            <div className="value">{usedPercent}<span className="unit">%</span></div>
          </div>
        </div>

        {/* Storage bar */}
        <div style={{ marginTop: '16px', marginBottom: '24px' }}>
          <div style={{ height: '8px', background: 'var(--bg-tertiary)', borderRadius: '4px', overflow: 'hidden' }}>
            <div style={{
              height: '100%',
              width: `${usedPercent}%`,
              background: 'var(--accent)',
              borderRadius: '4px',
              transition: 'width 0.3s ease',
            }} />
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '4px', fontSize: '11px', color: 'var(--text-muted)' }}>
            <span>{vaultStatus?.used_space_gb.toFixed(1) || '0'} GB used</span>
            <span>{vaultStatus?.total_space_gb.toFixed(0) || '460'} GB total</span>
          </div>
        </div>

        {/* Vault domains */}
        <h3 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '12px', color: 'var(--text-secondary)' }}>
          Vault Domains
        </h3>
        <div className="vault-domain-list">
          {MOCK_DOMAINS.map(domain => (
            <div key={domain.name} className="vault-domain-item">
              <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                <span style={{ fontSize: '18px' }}>{domain.icon}</span>
                <span className="domain-name">{domain.name}</span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
                <span className="domain-count">{domain.count} items</span>
                <span style={{ fontSize: '12px', color: 'var(--text-muted)', fontFamily: 'var(--font-mono)' }}>{domain.size}</span>
              </div>
            </div>
          ))}
        </div>

        {/* Encryption info */}
        <div style={{ marginTop: '24px', padding: '16px', background: 'var(--bg-secondary)', border: '1px solid var(--border)', borderRadius: 'var(--radius-md)' }}>
          <h4 style={{ fontSize: '12px', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '8px' }}>
            Encryption
          </h4>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '8px', fontSize: '13px' }}>
            <div style={{ color: 'var(--text-secondary)' }}>KDF:</div>
            <div style={{ color: 'var(--text-primary)', fontFamily: 'var(--font-mono)', fontSize: '12px' }}>Argon2id (256MB, 3 iter)</div>
            <div style={{ color: 'var(--text-secondary)' }}>Cipher:</div>
            <div style={{ color: 'var(--text-primary)', fontFamily: 'var(--font-mono)', fontSize: '12px' }}>XChaCha20-Poly1305</div>
            <div style={{ color: 'var(--text-secondary)' }}>Key Derivation:</div>
            <div style={{ color: 'var(--text-primary)', fontFamily: 'var(--font-mono)', fontSize: '12px' }}>HMAC-SHA256 domain keys</div>
            <div style={{ color: 'var(--text-secondary)' }}>Journal:</div>
            <div style={{ color: 'var(--text-primary)', fontFamily: 'var(--font-mono)', fontSize: '12px' }}>Write-ahead (PENDING→COMMITTED)</div>
          </div>
        </div>
      </div>
    </div>
  );
}