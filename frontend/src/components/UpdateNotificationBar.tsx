import type { CSSProperties } from 'react'

export type UpdateNotificationState =
  | { status: 'available'; version: string }
  | { status: 'downloading'; progress: number }
  | { status: 'error'; message: string }

type Props = {
  state: UpdateNotificationState
  onUpdate: () => void
  onDismiss: () => void
}

export const UpdateNotificationBar = ({
  state,
  onUpdate,
  onDismiss,
}: Props) => {
  return (
    <div style={containerStyle}>
      {state.status === 'available' && (
        <>
          <span style={textStyle}>v{state.version} available</span>
          <button type="button" onClick={onUpdate} style={updateButtonStyle}>
            UPDATE
          </button>
          <button type="button" onClick={onDismiss} style={dismissButtonStyle}>
            DISMISS
          </button>
        </>
      )}
      {state.status === 'downloading' && (
        <span style={textStyle}>Downloading... {state.progress}%</span>
      )}
      {state.status === 'error' && (
        <>
          <span style={{ ...textStyle, color: '#EF4444' }}>
            Update failed: {state.message}
          </span>
          <button type="button" onClick={onDismiss} style={dismissButtonStyle}>
            DISMISS
          </button>
        </>
      )}
    </div>
  )
}

const containerStyle: CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: '8px',
  padding: '8px 16px',
  backgroundColor: '#1A1A2E',
  borderBottom: '1px solid #1A1A1A',
  fontFamily: "'JetBrains Mono', monospace",
  fontSize: '11px',
}

const textStyle: CSSProperties = {
  color: '#94A3B8',
  fontWeight: 500,
}

const updateButtonStyle: CSSProperties = {
  background: 'none',
  border: '1px solid #3B82F6',
  borderRadius: '4px',
  color: '#3B82F6',
  fontFamily: "'JetBrains Mono', monospace",
  fontSize: '10px',
  fontWeight: 600,
  letterSpacing: '1px',
  padding: '3px 8px',
  cursor: 'pointer',
}

const dismissButtonStyle: CSSProperties = {
  background: 'none',
  border: 'none',
  color: '#475569',
  fontFamily: "'JetBrains Mono', monospace",
  fontSize: '10px',
  fontWeight: 600,
  padding: '3px 8px',
  cursor: 'pointer',
}
