import { useState, useEffect, useCallback } from 'react';
import { tauriApi, type AccessibilityStatus } from '../lib/tauri';

export function AccessibilityView() {
  const [status, setStatus] = useState<AccessibilityStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [highContrast, setHighContrast] = useState(false);
  const [reducedMotion, setReducedMotion] = useState(false);
  const [fontScale, setFontScale] = useState(1.0);

  const loadStatus = useCallback(async () => {
    setLoading(true);
    try {
      const accessibilityStatus = await tauriApi.getAccessibilityStatus();
      setStatus(accessibilityStatus);
      setHighContrast(accessibilityStatus.high_contrast);
      setReducedMotion(accessibilityStatus.reduced_motion);
      setFontScale(accessibilityStatus.font_scale);
    } catch {
      // Tauri not available — use defaults
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadStatus();
  }, [loadStatus]);

  // Apply font scale to document
  useEffect(() => {
    document.documentElement.style.fontSize = `${fontScale * 100}%`;
    return () => { document.documentElement.style.fontSize = ''; };
  }, [fontScale]);

  // Apply high contrast
  useEffect(() => {
    if (highContrast) {
      document.documentElement.classList.add('high-contrast');
    } else {
      document.documentElement.classList.remove('high-contrast');
    }
  }, [highContrast]);

  // Apply reduced motion
  useEffect(() => {
    if (reducedMotion) {
      document.documentElement.classList.add('reduced-motion');
    } else {
      document.documentElement.classList.remove('reduced-motion');
    }
  }, [reducedMotion]);

  return (
    <div>
      <div className="main-header">
        <h2>Accessibility</h2>
        <div className="main-header-actions">
          <button className="btn btn-secondary btn-sm" onClick={loadStatus}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <polyline points="1 4 1 10 7 10" />
              <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
            </svg>
            Refresh
          </button>
        </div>
      </div>

      <div className="main-body">
        <div className="settings-view">
          {/* Vision — Camera features not yet available */}
          <div className="settings-section">
            <div className="settings-section-header">👁️ Blind View (Vision Assist)</div>
            <div className="settings-section-body">
              <p style={{ fontSize: '13px', color: 'var(--text-secondary)', marginBottom: '16px' }}>
                Blind View uses Gemma 4 12B vision to describe images, detected objects, and screen content
                for visually impaired users. Camera feed is processed locally — nothing leaves the device.
              </p>

              <div className="settings-row" style={{ opacity: 0.5 }}>
                <div>
                  <div className="settings-row-label">Camera Blind Aid</div>
                  <div className="settings-row-desc">Use camera to describe surroundings in real-time (not yet available on desktop)</div>
                </div>
                <label style={{ display: 'flex', alignItems: 'center', cursor: 'not-allowed' }}>
                  <input type="checkbox" disabled style={{ width: '16px', height: '16px' }} />
                </label>
              </div>

              <div className="settings-row" style={{ opacity: 0.5 }}>
                <div>
                  <div className="settings-row-label">Screen Reader Description</div>
                  <div className="settings-row-desc">Describe on-screen content for screen readers (pending integration)</div>
                </div>
                <label style={{ display: 'flex', alignItems: 'center', cursor: 'not-allowed' }}>
                  <input type="checkbox" disabled style={{ width: '16px', height: '16px' }} />
                </label>
              </div>

              <div className="settings-row" style={{ opacity: 0.5 }}>
                <div>
                  <div className="settings-row-label">OCR Text Extraction</div>
                  <div className="settings-row-desc">Extract text from images and documents using Tesseract OCR (pending)</div>
                </div>
                <label style={{ display: 'flex', alignItems: 'center', cursor: 'not-allowed' }}>
                  <input type="checkbox" disabled style={{ width: '16px', height: '16px' }} />
                </label>
              </div>
            </div>
          </div>

          {/* Display — these settings work now */}
          <div className="settings-section">
            <div className="settings-section-header">🖥️ Display</div>
            <div className="settings-section-body">
              {status?.screen_reader_detected && (
                <div style={{ marginBottom: '12px', padding: '8px 12px', background: 'rgba(34,197,94,0.1)', borderRadius: 'var(--radius-sm)', fontSize: '13px', color: 'var(--success)' }}>
                  ✅ Screen reader detected: {status.screen_reader_name}
                </div>
              )}

              <div className="settings-row">
                <div>
                  <div className="settings-row-label">High Contrast</div>
                  <div className="settings-row-desc">Increase contrast for better visibility</div>
                </div>
                <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                  <input
                    type="checkbox"
                    checked={highContrast}
                    onChange={e => setHighContrast(e.target.checked)}
                    style={{ width: '16px', height: '16px' }}
                  />
                </label>
              </div>

              <div className="settings-row">
                <div>
                  <div className="settings-row-label">Reduced Motion</div>
                  <div className="settings-row-desc">Minimize animations and transitions</div>
                </div>
                <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                  <input
                    type="checkbox"
                    checked={reducedMotion}
                    onChange={e => setReducedMotion(e.target.checked)}
                    style={{ width: '16px', height: '16px' }}
                  />
                </label>
              </div>

              <div className="settings-row">
                <div>
                  <div className="settings-row-label">Font Scale</div>
                  <div className="settings-row-desc">Adjust text size throughout the application</div>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                  <input
                    type="range"
                    min="0.8"
                    max="2.0"
                    step="0.1"
                    value={fontScale}
                    onChange={e => setFontScale(Number(e.target.value))}
                    style={{ width: '120px' }}
                  />
                  <span style={{ fontFamily: 'var(--font-mono)', fontSize: '13px' }}>
                    {fontScale.toFixed(1)}x
                  </span>
                </div>
              </div>
            </div>
          </div>

          {/* Voice */}
          <div className="settings-section">
            <div className="settings-section-header">🔊 Voice & Audio</div>
            <div className="settings-section-body">
              <div className="settings-row">
                <div>
                  <div className="settings-row-label">Screen Reader Support</div>
                  <div className="settings-row-desc">Announce UI changes to screen readers (NVDA, JAWS, VoiceOver)</div>
                </div>
                <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                  <input
                    type="checkbox"
                    checked={status?.screen_reader_detected ?? false}
                    disabled
                    style={{ width: '16px', height: '16px' }}
                  />
                </label>
              </div>

              <div className="settings-row" style={{ opacity: 0.5 }}>
                <div>
                  <div className="settings-row-label">TTS Language</div>
                  <div className="settings-row-desc">Language for text-to-speech output (Piper integration pending)</div>
                </div>
                <select defaultValue="en" disabled>
                  <option value="en">English</option>
                  <option value="hi">हिन्दी (Hindi)</option>
                  <option value="bn">বাংলা (Bengali)</option>
                  <option value="ta">தமிழ் (Tamil)</option>
                  <option value="te">తెలుగు (Telugu)</option>
                  <option value="kn">ಕನ್ನಡ (Kannada)</option>
                  <option value="ml">മലയാളം (Malayalam)</option>
                </select>
              </div>

              <div className="settings-row" style={{ opacity: 0.5 }}>
                <div>
                  <div className="settings-row-label">STT Language</div>
                  <div className="settings-row-desc">Language for speech-to-text recognition (Whisper integration pending)</div>
                </div>
                <select defaultValue="en" disabled>
                  <option value="en">English</option>
                  <option value="hi">हिन्दी (Hindi)</option>
                  <option value="bn">বাংলা (Bengali)</option>
                  <option value="ta">தமிழ் (Tamil)</option>
                  <option value="te">తెలుగు (Telugu)</option>
                  <option value="kn">ಕನ್ನಡ (Kannada)</option>
                  <option value="ml">മലയാളം (Malayalam)</option>
                </select>
              </div>
            </div>
          </div>

          {/* Keyboard */}
          <div className="settings-section">
            <div className="settings-section-header">⌨️ Keyboard Navigation</div>
            <div className="settings-section-body">
              <div style={{ fontSize: '13px', color: 'var(--text-secondary)', lineHeight: 1.6 }}>
                <p><strong>Shortcuts:</strong></p>
                <ul style={{ paddingLeft: '20px', marginTop: '8px' }}>
                  <li><kbd style={{ padding: '2px 6px', background: 'var(--bg-tertiary)', borderRadius: '4px', fontFamily: 'var(--font-mono)', fontSize: '12px' }}>Ctrl+L</kbd> — Lock vault</li>
                  <li><kbd style={{ padding: '2px 6px', background: 'var(--bg-tertiary)', borderRadius: '4px', fontFamily: 'var(--font-mono)', fontSize: '12px' }}>Ctrl+N</kbd> — New chat</li>
                  <li><kbd style={{ padding: '2px 6px', background: 'var(--bg-tertiary)', borderRadius: '4px', fontFamily: 'var(--font-mono)', fontSize: '12px' }}>Ctrl+R</kbd> — Start/stop recording</li>
                  <li><kbd style={{ padding: '2px 6px', background: 'var(--bg-tertiary)', borderRadius: '4px', fontSize: '12px', fontFamily: 'var(--font-mono)' }}>Ctrl+M</kbd> — Toggle microphone</li>
                  <li><kbd style={{ padding: '2px 6px', background: 'var(--bg-tertiary)', borderRadius: '4px', fontFamily: 'var(--font-mono)', fontSize: '12px' }}>Ctrl+1-8</kbd> — Switch views</li>
                  <li><kbd style={{ padding: '2px 6px', background: 'var(--bg-tertiary)', borderRadius: '4px', fontFamily: 'var(--font-mono)', fontSize: '12px' }}>Escape</kbd> — Cancel current action</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}