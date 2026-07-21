import { useState, useEffect, useCallback } from 'react';
import { tauriApi } from '../lib/tauri';

interface Memory {
  id: string;
  type: string;
  title: string;
  preview: string;
  date: string;
  source: string;
}

export function MemoryExplorer() {
  const [memories, setMemories] = useState<Memory[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  const loadMemories = useCallback(async () => {
    setLoading(true);
    setError('');
    try {
      const result = await tauriApi.searchMemories({
        query: searchQuery || '*',
        memory_types: [],
        limit: 50,
        min_relevance: 0.0,
      }, 'D:\\UNOONE');
      setMemories(result.map((m: { id: string; memory_type: string; title: string; preview: string; created_at: string }) => ({
        id: m.id,
        type: m.memory_type.toLowerCase(),
        title: m.title,
        preview: m.preview,
        date: m.created_at ? new Date(m.created_at).toLocaleDateString() : '',
        source: 'Vault',
      })));
    } catch (e) {
      // Tauri not available or vault not connected — show empty
      setMemories([]);
    } finally {
      setLoading(false);
    }
  }, [searchQuery]);

  useEffect(() => {
    loadMemories();
  }, [loadMemories]);

  return (
    <div>
      <div className="main-header">
        <h2>Memory</h2>
        <div className="main-header-actions">
          <button className="btn btn-secondary btn-sm" onClick={loadMemories}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polyline points="1 4 1 10 7 10" />
              <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
            </svg>
            Refresh
          </button>
        </div>
      </div>

      <div className="main-body">
        {loading ? (
          <div style={{ display: 'flex', justifyContent: 'center', padding: '48px' }}>
            <span className="spinner" />
          </div>
        ) : memories.length > 0 ? (
          <div className="memory-grid">
            {memories.map(mem => (
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
        ) : (
          <div className="empty-state">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M12 2L2 7l10 5 10-5-10-5z" />
              <path d="M2 17l10 5 10-5" />
              <path d="M2 12l10 5 10-5" />
            </svg>
            <h3>No memories yet</h3>
            <p>Memories will appear here once you start chatting with Gemma 4 or create notes. All memories are encrypted and stored on your Pocket USB.</p>
          </div>
        )}
      </div>
    </div>
  );
}