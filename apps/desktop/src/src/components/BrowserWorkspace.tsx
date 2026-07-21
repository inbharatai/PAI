import { useState } from 'react';

interface BrowserTab {
  id: string;
  url: string;
  title: string;
}

export function BrowserWorkspace() {
  const [url, setUrl] = useState('https://');
  const [loading, setLoading] = useState(false);
  const [active, setActive] = useState(false);
  const [history, setHistory] = useState<string[]>([]);
  const [extractedText, setExtractedText] = useState<string | null>(null);
  const [tabs] = useState<BrowserTab[]>([]);

  const handleNavigate = () => {
    if (!url || url === 'https://') return;
    setLoading(true);
    setHistory(prev => [url, ...prev.slice(0, 19)]);
    // TODO: Connect to Tauri browser_execute Navigate action
    setTimeout(() => setLoading(false), 1000);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') handleNavigate();
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div className="main-header">
        <h2>Browser Workspace</h2>
        <div className="main-header-actions">
          <span className={`hw-badge ${active ? 'available' : 'unavailable'}`}>
            {active ? 'Connected' : 'Disconnected'}
          </span>
        </div>
      </div>

      {/* URL bar */}
      <div style={{
        padding: '8px 24px',
        borderBottom: '1px solid var(--border)',
        background: 'var(--bg-secondary)',
        display: 'flex',
        gap: '8px',
        alignItems: 'center',
      }}>
        <button
          className="btn btn-ghost btn-sm"
          onClick={() => { /* TODO: browser back */ }}
          title="Back"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <polyline points="15 18 9 12 15 6" />
          </svg>
        </button>
        <button
          className="btn btn-ghost btn-sm"
          onClick={() => { /* TODO: browser forward */ }}
          title="Forward"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <polyline points="9 18 15 12 9 6" />
          </svg>
        </button>
        <button
          className="btn btn-ghost btn-sm"
          onClick={handleNavigate}
          title="Refresh"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <polyline points="23 4 23 10 17 10" />
            <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
          </svg>
        </button>
        <input
          type="url"
          value={url}
          onChange={e => setUrl(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Enter URL…"
          style={{
            flex: 1,
            padding: '8px 12px',
            background: 'var(--bg-tertiary)',
            border: '1px solid var(--border)',
            borderRadius: 'var(--radius-sm)',
            color: 'var(--text-primary)',
            fontSize: '13px',
            fontFamily: 'var(--font-mono)',
            outline: 'none',
          }}
        />
        <button className="btn btn-primary btn-sm" onClick={handleNavigate} disabled={loading}>
          Go
        </button>
      </div>

      {/* Browser viewport */}
      <div style={{
        flex: 1,
        background: 'var(--bg-primary)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        position: 'relative',
      }}>
        {!active ? (
          <div className="empty-state">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10" />
              <line x1="2" y1="12" x2="22" y2="12" />
              <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
            </svg>
            <h3>Browser Workspace</h3>
            <p>Enter a URL to start browsing. All actions go through the SafetyGuard pipeline before execution.</p>
            <div style={{ display: 'flex', gap: '8px', marginTop: '8px' }}>
              <button className="btn btn-primary btn-sm" onClick={() => setActive(true)}>
                Start Session
              </button>
            </div>
          </div>
        ) : loading ? (
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '16px' }}>
            <span className="spinner" />
            <span style={{ color: 'var(--text-muted)', fontSize: '13px' }}>Loading…</span>
          </div>
        ) : (
          <div style={{
            width: '100%',
            height: '100%',
            display: 'flex',
            flexDirection: 'column',
            padding: '24px',
            overflow: 'auto',
          }}>
            {/* Placeholder viewport — actual Chromium rendering in Phase 8 production */}
            <div style={{
              width: '100%',
              aspectRatio: '16/9',
              maxWidth: '800px',
              background: 'var(--bg-tertiary)',
              border: '1px solid var(--border)',
              borderRadius: 'var(--radius-md)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              margin: '0 auto',
              color: 'var(--text-muted)',
              fontSize: '14px',
            }}>
              <div style={{ textAlign: 'center' }}>
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" style={{ marginBottom: '12px', opacity: 0.5 }}>
                  <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
                  <line x1="8" y1="21" x2="16" y2="21" />
                  <line x1="12" y1="17" x2="12" y2="21" />
                </svg>
                <p>Chromium viewport will render here</p>
                <p style={{ fontSize: '12px' }}>PageAgent manages all interactions</p>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Extracted text panel */}
      {extractedText && (
        <div style={{
          height: '200px',
          borderTop: '1px solid var(--border)',
          background: 'var(--bg-secondary)',
          overflow: 'auto',
          padding: '12px 24px',
          fontSize: '13px',
          fontFamily: 'var(--font-mono)',
          color: 'var(--text-secondary)',
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
            <span style={{ fontWeight: 600, color: 'var(--text-primary)' }}>Extracted Text</span>
            <button className="btn btn-ghost btn-sm" onClick={() => setExtractedText(null)}>
              Close
            </button>
          </div>
          <pre style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>{extractedText}</pre>
        </div>
      )}

      {/* Safety notice */}
      <div style={{
        padding: '8px 24px',
        borderTop: '1px solid var(--border)',
        background: 'var(--bg-secondary)',
        fontSize: '11px',
        color: 'var(--text-muted)',
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
      }}>
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="var(--success)" strokeWidth="2">
          <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
        </svg>
        All browser actions are reviewed by SafetyGuard before execution
      </div>
    </div>
  );
}