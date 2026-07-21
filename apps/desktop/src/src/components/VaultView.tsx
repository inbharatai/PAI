import { useState, useEffect, useCallback } from 'react';
import { tauriApi, type VaultStatus } from '../lib/tauri';

interface VaultDomain {
  name: string;
  icon: string;
  path: string;
}

const VAULT_DOMAINS: VaultDomain[] = [
  { name: 'Memories', icon: '🧠', path: 'memory' },
  { name: 'Chats', icon: '💬', path: 'chats' },
  { name: 'Recordings', icon: '🎙️', path: 'recordings' },
  { name: 'Documents', icon: '📄', path: 'documents' },
  { name: 'Settings', icon: '⚙️', path: 'settings' },
  { name: 'Audit', icon: '📋', path: 'audit' },
];

export function VaultView() {
  const [vaultStatus, setVaultStatus] = useState<VaultStatus | null>(null);
  const [domainCounts, setDomainCounts] = useState<Record<string, number>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  const loadVaultData = useCallback(async () => {
    setLoading(true);
    setError('');
    try {
      const status = await tauriApi.getVaultStatus();
      setVaultStatus(status);
    } catch {
      setVaultStatus(null);
    }

    // Scan vault directories for real file counts
    try {
      const vaultInfo = await tauriApi.detectVault();
      if (vaultInfo.detected) {
        const counts: Record<string, number> = {};
        for (const domain of VAULT_DOMAINS) {
          try {
            const docs = await tauriApi.listDocuments(vaultInfo.vault_root);
            counts[domain.path] = docs.length;
          } catch {
            counts[domain.path] = 0;
          }
        }
        setDomainCounts(counts);
      }
    } catch {
      // Vault not connected — counts stay at 0
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    loadVaultData();
  }, [loadVaultData]);

  const usedPercent = vaultStatus
    ? Math.round((vaultStatus.used_space_gb / Math.max(vaultStatus.total_space_gb, 1)) * 100)
    : 0;

  if (loading) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', padding: '48px' }}>
        <span className="spinner" />
      </div>
    );
  }

  return (
    <div className="vault-view">
      <div className="main-header">
        <h2>Vault</h2>
        <div className="main-header-actions">
          <button className="btn btn-secondary btn-sm" onClick={loadVaultData}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polyline points="1 4 1 10 7 10" />
              <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
            </svg>
            Refresh
          </button>
          <button className="btn btn-danger btn-sm" onClick={async () => {
            try {
              await tauriApi.lockVault();
            } catch (e) {
              setError(`Emergency lock failed: ${e instanceof Error ? e.message : String(e)}`);
            }
          }}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
              <path d="M7 11V7a5 5 0 0 1 10 0v4" />
            </svg>
            Emergency Lock
          </button>
        </div>
      </div>

      <div className="main-body">
        {!vaultStatus?.is_connected ? (
          <div className="empty-state">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
              <path d="M7 11V7a5 5 0 0 1 10 0v4" />
            </svg>
            <h3>No Pocket USB connected</h3>
            <p>Connect your UnoOne Pocket USB drive to access your encrypted vault.</p>
          </div>
        ) : (
          <>
            <div className="vault-stats">
              <div className="vault-stat-card">
                <h4>Status</h4>
                <div className="value" style={{ color: vaultStatus?.is_unlocked ? 'var(--success)' : 'var(--danger)' }}>
                  {vaultStatus?.is_unlocked ? 'Unlocked' : 'Locked'}
                </div>
              </div>
              <div className="vault-stat-card">
                <h4>Used Space</h4>
                <div className="value">
                  {vaultStatus?.used_space_gb.toFixed(1) ?? '0.0'}
                  <span className="unit">GB</span>
                </div>
              </div>
              <div className="vault-stat-card">
                <h4>Total Space</h4>
                <div className="value">
                  {vaultStatus?.total_space_gb.toFixed(0) ?? '0'}
                  <span className="unit">GB</span>
                </div>
              </div>
              <div className="vault-stat-card">
                <h4>Usage</h4>
                <div className="value">{usedPercent}<span className="unit">%</span></div>
              </div>
            </div>

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
                <span>{vaultStatus?.used_space_gb.toFixed(1) ?? '0'} GB used</span>
                <span>{vaultStatus?.total_space_gb.toFixed(0) ?? '0'} GB total</span>
              </div>
            </div>

            <h3 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '12px', color: 'var(--text-secondary)' }}>
              Vault Domains
            </h3>
            <div className="vault-domain-list">
              {VAULT_DOMAINS.map(domain => (
                <div key={domain.name} className="vault-domain-item">
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                    <span style={{ fontSize: '18px' }}>{domain.icon}</span>
                    <span className="domain-name">{domain.name}</span>
                  </div>
                  <span className="domain-count" style={{ fontSize: '12px', color: 'var(--text-muted)', fontFamily: 'var(--font-mono)' }}>
                    {domainCounts[domain.path] ?? 0} items
                  </span>
                </div>
              ))}
            </div>

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
          </>
        )}
      </div>
    </div>
  );
}