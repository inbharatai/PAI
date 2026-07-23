import { useState, useEffect, useCallback, Component, type ReactNode } from 'react';
import { UnlockScreen } from './components/UnlockScreen';
import { Sidebar, type ViewId } from './components/Sidebar';
import { ChatView } from './components/ChatView';
import { RecordingView } from './components/RecordingView';
import { MemoryExplorer } from './components/MemoryExplorer';
import { VaultView } from './components/VaultView';
import { SettingsView } from './components/SettingsView';
import { HardwareProfile } from './components/HardwareProfile';
import { ModelManager } from './components/ModelManager';
import { BrowserWorkspace } from './components/BrowserWorkspace';
import { DocumentsView } from './components/DocumentsView';
import { AccessibilityView } from './components/AccessibilityView';
import { tauriApi } from './lib/tauri';

type AppScreen = 'unlock' | 'main';

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100vh', gap: '16px' }}>
          <h2>Something went wrong</h2>
          <p style={{ color: 'var(--text-muted)', maxWidth: '400px', textAlign: 'center' }}>
            {this.state.error?.message || 'An unexpected error occurred.'}
          </p>
          <button onClick={() => window.location.reload()} style={{ padding: '8px 24px' }}>
            Reload
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}

function App() {
  const [screen, setScreen] = useState<AppScreen>('unlock');
  const [currentView, setCurrentView] = useState<ViewId>('chat');
  const [vaultId, setVaultId] = useState<string>('');
  const [vaultRoot, setVaultRoot] = useState<string>('');
  const [autoLockMs, setAutoLockMs] = useState<number>(300000); // default 5 min

  const handleUnlock = useCallback((id: string) => {
    setVaultId(id);
    setScreen('main');
  }, []);

  const handleLock = useCallback(() => {
    setVaultId('');
    setVaultRoot('');
    setScreen('unlock');
    setCurrentView('chat');
  }, []);

  // Load settings to get auto-lock timer; re-fetch when vaultId changes
  useEffect(() => {
    if (screen !== 'main' || !vaultId) return;
    tauriApi.getSettings(vaultRoot || '').then(settings => {
      if (settings?.auto_lock_minutes) {
        setAutoLockMs(settings.auto_lock_minutes * 60 * 1000);
      }
    }).catch(() => {});
  }, [screen, vaultId, vaultRoot]);

  // Detect vault root so settings can be loaded
  useEffect(() => {
    if (screen !== 'main') return;
    tauriApi.detectVault().then(info => {
      if (info.detected) setVaultRoot(info.vault_root);
    }).catch(() => {});
  }, [screen]);

  // Auto-lock on window blur (timer from settings)
  useEffect(() => {
    if (screen !== 'main') return;
    let timer: number | null = null;
    const handleBlur = () => {
      timer = window.setTimeout(() => {
        handleLock();
      }, autoLockMs);
    };
    const handleFocus = () => {
      if (timer) window.clearTimeout(timer);
    };
    window.addEventListener('blur', handleBlur);
    window.addEventListener('focus', handleFocus);
    return () => {
      window.removeEventListener('blur', handleBlur);
      window.removeEventListener('focus', handleFocus);
      if (timer) window.clearTimeout(timer);
    };
  }, [screen, handleLock, autoLockMs]);

  if (screen === 'unlock') {
    return (
      <ErrorBoundary>
        <UnlockScreen onUnlock={handleUnlock} />
      </ErrorBoundary>
    );
  }

  const renderView = () => {
    switch (currentView) {
      case 'chat':
        return <ChatView />;
      case 'recordings':
        return <RecordingView />;
      case 'memory':
        return <MemoryExplorer />;
      case 'vault':
        return <VaultView />;
      case 'model':
        return <ModelManager />;
      case 'browser':
        return <BrowserWorkspace />;
      case 'documents':
        return <DocumentsView />;
      case 'accessibility':
        return <AccessibilityView />;
      case 'hardware':
        return <HardwareProfile />;
      case 'settings':
        return <SettingsView vaultRoot={vaultRoot} />;
      default:
        return <ChatView />;
    }
  };

  return (
    <ErrorBoundary>
      <div className="app-shell">
        <Sidebar currentView={currentView} onNavigate={setCurrentView} onLock={handleLock} />
        <div className="main-content">
          {renderView()}
        </div>
      </div>
    </ErrorBoundary>
  );
}

export default App;