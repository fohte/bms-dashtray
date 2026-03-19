import { type CSSProperties, useState } from 'react'

import type { AppConfig } from '@/types'

export interface SettingsScreenProps {
  config: AppConfig
  onBack: () => void
  onChangeBeatorajaRoot: () => void
  onToggleBackgroundTransparent: (value: boolean) => void
  onChangeFontSize: (delta: number) => void
  onChangeResetTime: (time: string) => void
  onResetHistory: () => void
}

const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    minHeight: '100vh',
    backgroundColor: '#000000',
    color: '#ffffff',
    fontFamily:
      "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    padding: '12px 16px',
    borderBottom: '1px solid #222222',
  },
  backButton: {
    background: 'none',
    border: 'none',
    color: '#888888',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '12px',
    cursor: 'pointer',
    padding: '4px 8px',
    transition: 'color 0.15s',
  },
  headerTitle: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '14px',
    fontWeight: 700,
    letterSpacing: '2px',
    marginLeft: '12px',
  },
  content: {
    flex: 1,
    padding: '24px 16px',
    display: 'flex',
    flexDirection: 'column',
    gap: '32px',
  },
  sectionTitle: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '11px',
    fontWeight: 600,
    color: '#666666',
    letterSpacing: '1px',
    textTransform: 'uppercase',
    marginBottom: '12px',
  },
  row: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '8px 0',
  },
  label: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '12px',
    color: '#cccccc',
  },
  value: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '12px',
    color: '#ffffff',
  },
  pathRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '8px 0',
  },
  pathDisplay: {
    flex: 1,
    padding: '8px 10px',
    backgroundColor: '#111111',
    border: '1px solid #333333',
    borderRadius: '4px',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '11px',
    color: '#ffffff',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  changeButton: {
    padding: '8px 12px',
    backgroundColor: '#222222',
    border: '1px solid #444444',
    borderRadius: '4px',
    color: '#ffffff',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '11px',
    fontWeight: 600,
    cursor: 'pointer',
    whiteSpace: 'nowrap',
    transition: 'background-color 0.15s',
  },
  toggleTrack: {
    width: '36px',
    height: '20px',
    borderRadius: '10px',
    cursor: 'pointer',
    transition: 'background-color 0.15s',
    border: 'none',
    padding: '2px',
    display: 'flex',
    alignItems: 'center',
  },
  toggleThumb: {
    width: '16px',
    height: '16px',
    borderRadius: '50%',
    backgroundColor: '#ffffff',
    transition: 'transform 0.15s',
  },
  fontSizeControl: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  fontSizeButton: {
    width: '28px',
    height: '28px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#222222',
    border: '1px solid #444444',
    borderRadius: '4px',
    color: '#ffffff',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '14px',
    fontWeight: 600,
    cursor: 'pointer',
    transition: 'background-color 0.15s',
  },
  fontSizeValue: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '12px',
    color: '#ffffff',
    minWidth: '32px',
    textAlign: 'center',
  },
  timeInput: {
    padding: '6px 10px',
    backgroundColor: '#111111',
    border: '1px solid #333333',
    borderRadius: '4px',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '12px',
    color: '#ffffff',
    width: '80px',
    textAlign: 'center',
  },
  resetButton: {
    width: '100%',
    padding: '10px',
    backgroundColor: 'transparent',
    border: '2px solid #EF4444',
    borderRadius: '4px',
    color: '#EF4444',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '12px',
    fontWeight: 700,
    letterSpacing: '1px',
    cursor: 'pointer',
    transition: 'all 0.15s',
  },
  confirmOverlay: {
    position: 'fixed',
    inset: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.8)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000,
  },
  confirmDialog: {
    backgroundColor: '#111111',
    border: '1px solid #333333',
    borderRadius: '8px',
    padding: '24px',
    maxWidth: '320px',
    width: '100%',
    textAlign: 'center',
  },
  confirmTitle: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '14px',
    fontWeight: 700,
    marginBottom: '8px',
  },
  confirmMessage: {
    fontSize: '12px',
    color: '#888888',
    marginBottom: '20px',
    lineHeight: 1.5,
  },
  confirmButtons: {
    display: 'flex',
    gap: '8px',
  },
  confirmCancel: {
    flex: 1,
    padding: '10px',
    backgroundColor: '#222222',
    border: '1px solid #444444',
    borderRadius: '4px',
    color: '#ffffff',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '12px',
    fontWeight: 600,
    cursor: 'pointer',
  },
  confirmReset: {
    flex: 1,
    padding: '10px',
    backgroundColor: '#EF4444',
    border: 'none',
    borderRadius: '4px',
    color: '#ffffff',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '12px',
    fontWeight: 700,
    cursor: 'pointer',
  },
} satisfies Record<string, CSSProperties>

export function SettingsScreen({
  config,
  onBack,
  onChangeBeatorajaRoot,
  onToggleBackgroundTransparent,
  onChangeFontSize,
  onChangeResetTime,
  onResetHistory,
}: SettingsScreenProps) {
  const [showResetConfirm, setShowResetConfirm] = useState(false)

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <button type="button" style={styles.backButton} onClick={onBack}>
          &larr; BACK
        </button>
        <span style={styles.headerTitle}>SETTINGS</span>
      </div>

      <div style={styles.content as CSSProperties}>
        {/* BEATORAJA Section */}
        <section>
          <div style={styles.sectionTitle as CSSProperties}>BEATORAJA</div>
          <div style={styles.pathRow as CSSProperties}>
            <div style={styles.pathDisplay as CSSProperties}>
              {config.beatorajaRoot}
            </div>
            <button
              type="button"
              style={styles.changeButton as CSSProperties}
              onClick={onChangeBeatorajaRoot}
            >
              CHANGE
            </button>
          </div>
          <div style={styles.row}>
            <span style={styles.label}>Player</span>
            <span style={styles.value}>{config.playerName}</span>
          </div>
        </section>

        {/* DISPLAY Section */}
        <section>
          <div style={styles.sectionTitle as CSSProperties}>DISPLAY</div>
          <div style={styles.row}>
            <span style={styles.label}>Background Transparent</span>
            <button
              type="button"
              style={{
                ...styles.toggleTrack,
                backgroundColor: config.backgroundTransparent
                  ? '#22C55E'
                  : '#333333',
              }}
              onClick={() =>
                onToggleBackgroundTransparent(!config.backgroundTransparent)
              }
              aria-label="Toggle background transparent"
              role="switch"
              aria-checked={config.backgroundTransparent}
            >
              <div
                style={{
                  ...styles.toggleThumb,
                  transform: config.backgroundTransparent
                    ? 'translateX(16px)'
                    : 'translateX(0)',
                }}
              />
            </button>
          </div>
          <div style={styles.row}>
            <span style={styles.label}>Font Size</span>
            <div style={styles.fontSizeControl}>
              <button
                type="button"
                style={styles.fontSizeButton}
                onClick={() => onChangeFontSize(-1)}
                aria-label="Decrease font size"
              >
                -
              </button>
              <span style={styles.fontSizeValue as CSSProperties}>
                {config.fontSize}px
              </span>
              <button
                type="button"
                style={styles.fontSizeButton}
                onClick={() => onChangeFontSize(1)}
                aria-label="Increase font size"
              >
                +
              </button>
            </div>
          </div>
        </section>

        {/* DATA Section */}
        <section>
          <div style={styles.sectionTitle as CSSProperties}>DATA</div>
          <div style={styles.row}>
            <span style={styles.label}>Reset Time</span>
            <input
              type="time"
              value={config.resetTime}
              onChange={(e) => onChangeResetTime(e.target.value)}
              style={styles.timeInput as CSSProperties}
              aria-label="Reset time"
            />
          </div>
          <div style={{ padding: '8px 0' }}>
            <button
              type="button"
              style={styles.resetButton}
              onClick={() => setShowResetConfirm(true)}
            >
              RESET HISTORY NOW
            </button>
          </div>
        </section>
      </div>

      {showResetConfirm && (
        <div style={styles.confirmOverlay as CSSProperties}>
          <div style={styles.confirmDialog as CSSProperties}>
            <div style={styles.confirmTitle}>Reset History</div>
            <p style={styles.confirmMessage}>
              This will clear all play history for today. This action cannot be
              undone.
            </p>
            <div style={styles.confirmButtons}>
              <button
                type="button"
                style={styles.confirmCancel}
                onClick={() => setShowResetConfirm(false)}
              >
                CANCEL
              </button>
              <button
                type="button"
                style={styles.confirmReset}
                onClick={() => {
                  setShowResetConfirm(false)
                  onResetHistory()
                }}
              >
                RESET
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
