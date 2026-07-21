import { useState, useEffect } from 'react';
import { tauriApi, type ModelInfo } from '../lib/tauri';

interface Document {
  id: string;
  title: string;
  document_type: string;
  file_size_bytes: number;
  source_platform: string;
}

const TYPE_ICONS: Record<string, string> = {
  PDF: '📄',
  DOCX: '📝',
  TXT: '📃',
  MARKDOWN: '📋',
  CSV: '📊',
  XLSX: '📈',
  PPTX: '📊',
  IMAGE: '🖼️',
  AUDIO: '🎙️',
  WEB_PAGE: '🌐',
};

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function DocumentsView() {
  const [documents, setDocuments] = useState<Document[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const docs = await tauriApi.listModels('D:\\UNOONE');
        // Placeholder — document listing will use list_documents
        setDocuments([]);
      } catch {
        setDocuments([]);
      } finally {
        setLoading(false);
      }
    }
    load();
  }, []);

  return (
    <div>
      <div className="main-header">
        <h2>Documents</h2>
        <div className="main-header-actions">
          <button className="btn btn-primary btn-sm">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="17 8 12 3 7 8" />
              <line x1="12" y1="3" x2="12" y2="15" />
            </svg>
            Import Document
          </button>
        </div>
      </div>

      {/* Search bar */}
      <div style={{ padding: '0 24px 16px', borderBottom: '1px solid var(--border)' }}>
        <div style={{ display: 'flex', gap: '8px' }}>
          <input
            type="text"
            placeholder="Search documents and memories…"
            value={searchQuery}
            onChange={e => setSearchQuery(e.target.value)}
            style={{
              flex: 1,
              padding: '10px 16px',
              background: 'var(--bg-tertiary)',
              border: '1px solid var(--border)',
              borderRadius: 'var(--radius-md)',
              color: 'var(--text-primary)',
              fontSize: '14px',
              outline: 'none',
            }}
          />
          <button className="btn btn-secondary">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <circle cx="11" cy="11" r="8" />
              <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
            Search
          </button>
        </div>
      </div>

      <div className="main-body">
        {loading ? (
          <div style={{ display: 'flex', justifyContent: 'center', padding: '48px' }}>
            <span className="spinner" />
          </div>
        ) : documents.length > 0 ? (
          <div className="memory-grid">
            {documents.map(doc => (
              <div key={doc.id} className="memory-card">
                <div className="memory-card-type knowledge">
                  {doc.document_type}
                </div>
                <div style={{ fontSize: '24px', marginBottom: '8px' }}>
                  {TYPE_ICONS[doc.document_type] || '📄'}
                </div>
                <div className="memory-card-title">{doc.title}</div>
                <div className="memory-card-preview">
                  {formatFileSize(doc.file_size_bytes)} · {doc.source_platform}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="empty-state">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
              <line x1="16" y1="13" x2="8" y2="13" />
              <line x1="16" y1="17" x2="8" y2="17" />
              <polyline points="10 9 9 9 8 9" />
            </svg>
            <h3>No documents yet</h3>
            <p>Import PDFs, Word documents, text files, and more. They'll be encrypted and stored on your Pocket USB.</p>
          </div>
        )}

        {/* Memory search info */}
        <div style={{ marginTop: '24px', padding: '16px', background: 'var(--bg-secondary)', border: '1px solid var(--border)', borderRadius: 'var(--radius-md)' }}>
          <h4 style={{ fontSize: '13px', fontWeight: 600, color: 'var(--text-secondary)', marginBottom: '8px' }}>
            🔍 Memory Retrieval
          </h4>
          <div style={{ fontSize: '12px', color: 'var(--text-muted)', lineHeight: 1.6 }}>
            <p><strong>Search Pipeline:</strong> Query → Embedding → Encrypted Index → Cosine Similarity → Ranked Results</p>
            <p><strong>Supported formats:</strong> PDF, DOCX, TXT, Markdown, CSV, XLSX, PPTX, Images (OCR), Audio (STT)</p>
            <p><strong>Encryption:</strong> All embeddings and indexes are encrypted with domain keys from the vault master key</p>
          </div>
        </div>
      </div>
    </div>
  );
}