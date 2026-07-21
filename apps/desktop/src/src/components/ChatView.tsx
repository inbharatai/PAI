import { useState, useRef, useEffect, useCallback } from 'react';

interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: number;
}

// llama-server HTTP API endpoint
const LLAMA_SERVER_URL = 'http://127.0.0.1:8342';

export function ChatView() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [isGenerating, setIsGenerating] = useState(false);
  const [modelStatus, setModelStatus] = useState<'unknown' | 'loaded' | 'not_loaded' | 'error'>('unknown');
  const [serverError, setServerError] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // Check if llama-server is reachable
  const checkModelStatus = useCallback(async () => {
    try {
      const response = await fetch(`${LLAMA_SERVER_URL}/health`, { signal: AbortSignal.timeout(2000) });
      if (response.ok) {
        setModelStatus('loaded');
        setServerError('');
        return true;
      }
    } catch {
      // Server not running
    }
    setModelStatus('not_loaded');
    return false;
  }, []);

  useEffect(() => {
    checkModelStatus();
  }, [checkModelStatus]);

  const handleSend = async () => {
    if (!input.trim() || isGenerating) return;

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      role: 'user',
      content: input.trim(),
      timestamp: Date.now(),
    };

    setMessages(prev => [...prev, userMessage]);
    setInput('');
    setIsGenerating(true);
    setServerError('');

    try {
      // Build conversation history for the model
      const conversationHistory = messages
        .concat(userMessage)
        .map(msg => ({
          role: msg.role,
          content: msg.content,
        }));

      // Call llama-server completion API
      const response = await fetch(`${LLAMA_SERVER_URL}/v1/chat/completions`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          model: 'gemma-4-12b',
          messages: conversationHistory,
          max_tokens: 4096,
          temperature: 0.7,
          stream: false,
        }),
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(`Server error: ${response.status} — ${errorText}`);
      }

      const data = await response.json();
      const assistantContent = data.choices?.[0]?.message?.content || data.content || '';

      if (assistantContent) {
        const assistantMessage: ChatMessage = {
          id: (Date.now() + 1).toString(),
          role: 'assistant',
          content: assistantContent,
          timestamp: Date.now(),
        };
        setMessages(prev => [...prev, assistantMessage]);
      } else {
        setServerError('Model returned an empty response.');
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      if (errorMsg.includes('Failed to fetch') || errorMsg.includes('NetworkError') || errorMsg.includes('connect')) {
        setServerError('Cannot connect to Gemma 4. Make sure the model is loaded (Settings → Model Manager).');
      } else {
        setServerError(errorMsg);
      }
    } finally {
      setIsGenerating(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const formatTime = (ts: number) => {
    return new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  return (
    <div className="chat-view">
      <div className="chat-messages">
        {messages.length === 0 && (
          <div className="empty-state">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
            </svg>
            <h3>Chat with Gemma 4</h3>
            <p>Your private AI assistant. All conversations are encrypted and stored on your Pocket USB.</p>
            {modelStatus === 'not_loaded' && (
              <div style={{ marginTop: '12px', padding: '12px 16px', background: 'var(--bg-tertiary)', borderRadius: 'var(--radius-md)', fontSize: '13px', color: 'var(--text-secondary)' }}>
                ⚠️ No model loaded. Open <strong>Model Manager</strong> to load Gemma 4 12B.
              </div>
            )}
            {modelStatus === 'loaded' && (
              <div style={{ marginTop: '12px', padding: '12px 16px', background: 'rgba(34,197,94,0.1)', borderRadius: 'var(--radius-md)', fontSize: '13px', color: 'var(--success)' }}>
                ✅ Gemma 4 12B is loaded and ready
              </div>
            )}
          </div>
        )}
        {messages.map(msg => (
          <div key={msg.id} className={`chat-message ${msg.role}`}>
            <div className="chat-avatar">
              {msg.role === 'user' ? 'U' : 'G'}
            </div>
            <div className="chat-bubble">
              <div style={{ whiteSpace: 'pre-wrap' }}>{msg.content}</div>
              <div style={{ fontSize: '10px', color: 'var(--text-muted)', marginTop: '4px' }}>
                {formatTime(msg.timestamp)}
              </div>
            </div>
          </div>
        ))}
        {isGenerating && (
          <div className="chat-message assistant">
            <div className="chat-avatar">G</div>
            <div className="chat-bubble">
              <span className="spinner" />
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      {serverError && (
        <div style={{
          padding: '8px 16px',
          background: 'rgba(239,68,68,0.1)',
          borderTop: '1px solid rgba(239,68,68,0.3)',
          fontSize: '13px',
          color: 'var(--danger)',
        }}>
          {serverError}
        </div>
      )}

      <div className="chat-input-area">
        <div className="chat-input-row">
          <textarea
            className="chat-input"
            placeholder={
              modelStatus === 'not_loaded'
                ? 'Load a model first (Model Manager)…'
                : 'Message Gemma 4… (Enter to send, Shift+Enter for new line)'
            }
            value={input}
            onChange={e => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            rows={1}
            disabled={isGenerating || modelStatus === 'not_loaded'}
          />
          <button
            className="btn btn-primary"
            onClick={handleSend}
            disabled={!input.trim() || isGenerating || modelStatus === 'not_loaded'}
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="22" y1="2" x2="11" y2="13" />
              <polygon points="22 2 15 22 11 13 2 9 22 2" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
}