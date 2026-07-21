import { useState, useEffect, useRef } from 'react';

interface Recording {
  id: string;
  title: string;
  date: string;
  duration: string;
  status: 'draft' | 'transcribed' | 'summarized';
}

export function RecordingView() {
  const [isRecording, setIsRecording] = useState(false);
  const [elapsed, setElapsed] = useState(0);
  const [recordings, setRecordings] = useState<Recording[]>([]);
  const timerRef = useRef<number | null>(null);

  useEffect(() => {
    if (isRecording) {
      timerRef.current = window.setInterval(() => {
        setElapsed(prev => prev + 1);
      }, 1000);
    } else {
      if (timerRef.current) {
        clearInterval(timerRef.current);
        timerRef.current = null;
      }
    }
    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
  }, [isRecording]);

  const formatTime = (seconds: number): string => {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  };

  const handleRecord = () => {
    if (isRecording) {
      // Stop recording
      setIsRecording(false);
      setRecordings(prev => [
        {
          id: Date.now().toString(),
          title: `Recording ${prev.length + 1}`,
          date: new Date().toLocaleDateString(),
          duration: formatTime(elapsed),
          status: 'draft',
        },
        ...prev,
      ]);
      setElapsed(0);
    } else {
      // Start recording
      setElapsed(0);
      setIsRecording(true);
    }
  };

  return (
    <div className="recording-view">
      <div className="main-header">
        <h2>Recordings</h2>
        <div className="main-header-actions">
          <button className="btn btn-secondary btn-sm">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="7 10 12 15 17 10" />
              <line x1="12" y1="15" x2="12" y2="3" />
            </svg>
            Import Audio
          </button>
        </div>
      </div>

      <div className="main-body" style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '24px', paddingTop: '48px' }}>
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '16px' }}>
          <button
            className={`record-btn ${isRecording ? 'recording' : ''}`}
            onClick={handleRecord}
            aria-label={isRecording ? 'Stop recording' : 'Start recording'}
          >
            <div className="record-btn-inner" />
          </button>

          <div className="recording-timer">
            {isRecording ? formatTime(elapsed) : '00:00:00'}
          </div>

          <p style={{ color: 'var(--text-muted)', fontSize: '13px' }}>
            {isRecording ? 'Recording… tap to stop' : 'Tap to start recording'}
          </p>

          {isRecording && (
            <div style={{ display: 'flex', gap: '8px' }}>
              <button className="btn btn-secondary btn-sm" onClick={() => {}}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M10 9H6V5" /><path d="M14 5h4v4" />
                  <path d="M14 9l6-6" /><path d="M6 19l6-6" />
                </svg>
                Bookmark
              </button>
              <button className="btn btn-danger btn-sm" onClick={() => { setIsRecording(false); setElapsed(0); }}>
                Cancel
              </button>
            </div>
          )}
        </div>

        {recordings.length > 0 && (
          <div style={{ width: '100%', maxWidth: '600px' }}>
            <h3 style={{ fontSize: '14px', fontWeight: 600, marginBottom: '12px', color: 'var(--text-secondary)' }}>
              Recent Recordings
            </h3>
            <div className="recording-list">
              {recordings.map(rec => (
                <div key={rec.id} className="recording-item">
                  <div className="recording-item-icon">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <polygon points="5 3 19 12 5 21 5 3" />
                    </svg>
                  </div>
                  <div className="recording-item-info">
                    <div className="recording-item-title">{rec.title}</div>
                    <div className="recording-item-meta">{rec.date} · {rec.status}</div>
                  </div>
                  <div className="recording-item-duration">{rec.duration}</div>
                </div>
              ))}
            </div>
          </div>
        )}

        {!isRecording && recordings.length === 0 && (
          <div className="empty-state">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
              <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
              <line x1="12" y1="19" x2="12" y2="23" />
              <line x1="8" y1="23" x2="16" y2="23" />
            </svg>
            <h3>No recordings yet</h3>
            <p>Tap the record button to start capturing audio. All recordings are encrypted and saved to your Pocket USB.</p>
          </div>
        )}
      </div>
    </div>
  );
}