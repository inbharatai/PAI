import { useState } from 'react';

export function BrowserWorkspace() {
  const [url, setUrl] = useState('https://');

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div className="main-header">
        <h2>Browser Workspace</h2>
        <div className="main-header-actions">
          <span className="hw-badge unavailable">
            Coming Soon
          </span>
        </div>
      </div>

      <div style={{
        flex: 1,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '48px',
      }}>
        <div className="empty-state">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="10" />
            <line x1="2" y1="12" x2="22" y2="12" />
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
          </svg>
          <h3>Browser Workspace</h3>
          <p style={{ maxWidth: '480px', textAlign: 'center' }}>
            The browser workspace allows PageAgent to browse the web with SafetyGuard protection.
            This feature requires Chromium/Playwright integration which is not yet available.
          </p>
          <div style={{
            marginTop: '16px',
            padding: '12px 16px',
            background: 'var(--bg-tertiary)',
            borderRadius: 'var(--radius-md)',
            fontSize: '13px',
            color: 'var(--text-secondary)',
          }}>
            <p><strong>Status:</strong> Structure defined, Playwright integration pending</p>
            <p><strong>Safety:</strong> All browser actions will go through SafetyGuard before execution</p>
          </div>
        </div>
      </div>
    </div>
  );
}