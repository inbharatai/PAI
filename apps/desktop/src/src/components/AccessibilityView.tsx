import { useState } from 'react';

export function AccessibilityView() {
  const [highContrast, setHighContrast] = useState(false);
  const [reducedMotion, setReducedMotion] = useState(false);
  const [fontScale, setFontScale] = useState(1.0);
  const [screenReader, setScreenReader] = useState(false);

  return (
    <div>
      <div className="main-header">
        <h2>Accessibility</h2>
      </div>

      <div className="main-body">
        <div className="settings-view">
          {/* Vision */}
          <div className="settings-section">
            <div className="settings-section-header">👁️ Blind View (Vision Assist)</div>
            <div className="settings-section-body">
              <p style={{ fontSize: '13px', color: 'var(--text-secondary)', marginBottom: '16px' }}>
                Blind View uses Gemma 4 12B vision to describe images, detected objects, and screen content
                for visually impaired users. Camera feed is processed locally — nothing leaves the device.
              </p>

              <div className="settings-row">
                <div>
                  <div className="settings-row-label">Camera Blind Aid</div>
                  <div className="settings-row-desc">Use camera to describe surroundings in real-time</div>
                </div>
                <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                  <input type="checkbox" style={{ width: '16px', height: '16px' }} />
                </label>
              </div>

              <div className="settings-row">
                <div>
                  <div className="settings-row-label">Screen Reader Description</div>
                  <div className="settings-row-desc">Describe on-screen content for screen readers</div>
                </div>
                <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                  <input type="checkbox" style={{ width: '16px', height: '16px' }} />
                </label>
              </div>

              <div className="settings-row">
                <div>
                  <div className="settings-row-label">OCR Text Extraction</div>
                  <div className="settings-row-desc">Extract text from images and documents using Tesseract OCR</div>
                </div>
                <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
                  <input type="checkbox" style={{ width: '16px', height: '16px' }} />
                </label>
              </div>
            </div>
          </div>

          {/* Display */}
          <div className="settings-section">
            <div className="settings-section-header">🖥️ Display</div>
            <div className="settings-section-body">
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
                    checked={screenReader}
                    onChange={e => setScreenReader(e.target.checked)}
                    style={{ width: '16px', height: '16px' }}
                  />
                </label>
              </div>

              <div className="settings-row">
                <div>
                  <div className="settings-row-label">TTS Language</div>
                  <div className="settings-row-desc">Language for text-to-speech output</div>
                </div>
                <select defaultValue="en">
                  <option value="en">English</option>
                  <option value="hi">हिन्दी (Hindi)</option>
                  <option value="bn">বাংলা (Bengali)</option>
                  <option value="ta">தமிழ் (Tamil)</option>
                  <option value="te">తెలుగు (Telugu)</option>
                  <option value="kn">ಕನ್ನಡ (Kannada)</option>
                  <option value="ml">മലയാളം (Malayalam)</option>
                </select>
              </div>

              <div className="settings-row">
                <div>
                  <div className="settings-row-label">STT Language</div>
                  <div className="settings-row-desc">Language for speech-to-text recognition</div>
                </div>
                <select defaultValue="en">
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
                  <li><kbd style={{ padding: '2px 6px', background: 'var(--bg-tertiary)', borderRadius: '4px', fontFamily: 'var(--font-mono)', fontSize: '12px' }}>Ctrl+M</kbd> — Toggle microphone</li>
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