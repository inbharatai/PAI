interface Memory {
  id: string;
  type: 'personal' | 'preference' | 'conversation' | 'task' | 'knowledge' | 'accessibility' | 'skill';
  title: string;
  preview: string;
  date: string;
  source: string;
}

const MOCK_MEMORIES: Memory[] = [
  { id: '1', type: 'personal', title: 'User works in software development', preview: 'Prefers dark mode, uses VS Code, works with TypeScript and Kotlin', date: '2026-07-20', source: 'Android' },
  { id: '2', type: 'preference', title: 'Language preferences', preview: 'Written: English. Spoken: English, Hindi. Timezone: IST (UTC+5:30)', date: '2026-07-19', source: 'Android' },
  { id: '3', type: 'conversation', title: 'Project architecture discussion', preview: 'Discussed migration from Room to PocketMemoryVault. Key points: encrypted at rest, Argon2id KDF, USB canonical.', date: '2026-07-18', source: 'Desktop' },
  { id: '4', type: 'task', title: 'Fix recording engine crash', preview: 'AudioRecord was not released on error path. Added try-finally in stop/cancel methods.', date: '2026-07-17', source: 'Android' },
  { id: '5', type: 'knowledge', title: 'XChaCha20-Poly1305 construction', preview: 'HKDF-SHA256 derives per-message key + 24-byte nonce from domain key. AES-256-GCM hardware-accelerated on Android.', date: '2026-07-16', source: 'Desktop' },
  { id: '6', type: 'accessibility', title: 'Screen reader preference', preview: 'User prefers high contrast mode with TalkBack-compatible output on mobile.', date: '2026-07-15', source: 'Android' },
  { id: '7', type: 'skill', title: 'Email summary skill', preview: 'Custom skill: Summarize unread emails → bullet points → save to vault memory domain.', date: '2026-07-14', source: 'Desktop' },
];

export function MemoryExplorer() {
  return (
    <div>
      <div className="main-header">
        <h2>Memory</h2>
        <div className="main-header-actions">
          <button className="btn btn-secondary btn-sm">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="11" cy="11" r="8" />
              <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
            Search
          </button>
        </div>
      </div>

      <div className="main-body">
        <div className="memory-grid">
          {MOCK_MEMORIES.map(mem => (
            <div key={mem.id} className="memory-card">
              <div className={`memory-card-type ${mem.type}`}>{mem.type}</div>
              <div className="memory-card-title">{mem.title}</div>
              <div className="memory-card-preview">{mem.preview}</div>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '8px', fontSize: '11px', color: 'var(--text-muted)' }}>
                <span>{mem.date}</span>
                <span>{mem.source}</span>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}