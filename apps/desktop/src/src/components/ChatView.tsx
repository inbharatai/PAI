import { useState, useRef, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  steps?: AgentStep[];
}

interface AgentStep {
  type: 'Thinking' | 'ToolCall' | 'ToolResult' | 'SafetyBlock' | 'FinalResponse';
  tool?: string;
  args?: Record<string, unknown>;
  result?: string;
  reason?: string;
  text?: string;
  confidence?: number;
  approved?: boolean;
}

interface AgentResult {
  final_text: string;
  steps: AgentStep[];
  iterations: number;
}

interface ConversationTurn {
  role: string;
  content: string;
  tool_calls?: unknown[];
  tool_call_id?: string;
}

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

  // D1: Check model health via Tauri command (replaces raw fetch)
  const checkModelStatus = useCallback(async () => {
    try {
      const result = await invoke('check_model_health');
      // If we get here, llama-server is responding
      setModelStatus('loaded');
      setServerError('');
      return true;
    } catch {
      setModelStatus('not_loaded');
      return false;
    }
  }, []);

  useEffect(() => {
    checkModelStatus();
  }, [checkModelStatus]);

  // D2: Send message via the agentic loop (replaces direct fetch)
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
      // Build conversation history for the agent
      const conversationHistory: ConversationTurn[] = messages
        .filter(m => m.role === 'user' || m.role === 'assistant')
        .map(msg => ({
          role: msg.role,
          content: msg.content,
        }));

      // D2: Call the backend agentic loop instead of direct fetch
      const result = await invoke<AgentResult>('agent_chat', {
        message: input.trim(),
        conversationHistory,
      });

      const assistantMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: result.final_text,
        timestamp: Date.now(),
        steps: result.steps,
      };
      setMessages(prev => [...prev, assistantMessage]);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      if (errorMsg.includes('Failed to connect') || errorMsg.includes('ECONNREFUSED') || errorMsg.includes('llama-server')) {
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

  const renderSteps = (steps: AgentStep[]) => {
    if (!steps || steps.length === 0) return null;

    return (
      <div className="agent-steps" style={{ marginTop: '8px', fontSize: '12px', color: 'var(--text-muted)' }}>
        {steps.map((step, i) => {
          switch (step.type) {
            case 'ToolCall':
              return (
                <div key={i} style={{ padding: '2px 0', color: 'var(--info)' }}>
                  🔧 Calling: {step.tool}
                  {step.args && <span style={{ marginLeft: '4px', color: 'var(--text-secondary)' }}>
                    {JSON.stringify(step.args).slice(0, 80)}
                  </span>}
                </div>
              );
            case 'ToolResult':
              return (
                <div key={i} style={{ padding: '2px 0', color: 'var(--success)' }}>
                  ✅ {step.tool}: {step.result?.slice(0, 100)}
                </div>
              );
            case 'SafetyBlock':
              return (
                <div key={i} style={{ padding: '2px 0', color: 'var(--danger)' }}>
                  🛡️ Blocked: {step.tool} — {step.reason}
                </div>
              );
            case 'Thinking':
              return (
                <div key={i} style={{ padding: '2px 0', color: 'var(--text-secondary)', fontStyle: 'italic' }}>
                  💭 {step.text?.slice(0, 150)}
                </div>
              );
            default:
              return null;
          }
        })}
      </div>
    );
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
              {renderSteps(msg.steps || [])}
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