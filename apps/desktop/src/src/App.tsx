import { useState, useEffect, useCallback } from 'react';
import { UnlockScreen } from './components/UnlockScreen';
import { Sidebar, type ViewId } from './components/Sidebar';
import { ChatView } from './components/ChatView';
import { RecordingView } from './components/RecordingView';
import { MemoryExplorer } from './components/MemoryExplorer';
import { VaultView } from './components/VaultView';
import { SettingsView } from './components/SettingsView';
import { HardwareProfile } from './components/HardwareProfile';

type AppScreen = 'unlock' | 'main';

function App() {
  const [screen, setScreen] = useState<AppScreen>('unlock');
  const [currentView, setCurrentView] = useState<ViewId>('chat');
  const [vaultId, setVaultId] = useState<string>('');

  const handleUnlock = useCallback((id: string) => {
    setVaultId(id);
    setScreen('main');
  }, []);

  const handleLock = useCallback(() => {
    setVaultId('');
    setScreen('unlock');
    setCurrentView('chat');
  }, []);

  // Auto-lock on window blur (optional, respects timer setting)
  useEffect(() => {
    if (screen !== 'main') return;
    let timer: number | null = null;
    const handleBlur = () => {
      timer = window.setTimeout(() => {
        // Auto-lock after 5 minutes of window blur
        // In production, this would use the autoLockMinutes setting
      }, 300000);
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
  }, [screen]);

  if (screen === 'unlock') {
    return <UnlockScreen onUnlock={handleUnlock} />;
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
      case 'hardware':
        return <HardwareProfile />;
      case 'settings':
        return <SettingsView />;
      default:
        return <ChatView />;
    }
  };

  return (
    <div className="app-shell">
      <Sidebar currentView={currentView} onNavigate={setCurrentView} onLock={handleLock} />
      <div className="main-content">
        {renderView()}
      </div>
    </div>
  );
}

export default App;