import type { CSSProperties } from 'react'

import type { DbFileStatus } from '@/types'

export interface SetupScreenProps {
  selectedPath: string | null
  dbFileStatuses: DbFileStatus[]
  isValidating: boolean
  error: string | null
  onSelectFolder: () => void
  onStart: () => void
}

const DB_FILE_NAMES = [
  'songdata.db',
  'scoredatalog.db',
  'score.db',
  'scorelog.db',
]

const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: '100vh',
    padding: '32px 24px',
    backgroundColor: '#000000',
    color: '#ffffff',
    fontFamily:
      "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
  },
  title: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '24px',
    fontWeight: 700,
    letterSpacing: '2px',
    marginBottom: '8px',
  },
  description: {
    fontSize: '13px',
    color: '#888888',
    marginBottom: '32px',
    textAlign: 'center',
    lineHeight: 1.5,
  },
  sectionLabel: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '11px',
    fontWeight: 600,
    color: '#666666',
    letterSpacing: '1px',
    textTransform: 'uppercase',
    alignSelf: 'stretch',
    marginBottom: '8px',
  },
  pathSelector: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    width: '100%',
    marginBottom: '24px',
  },
  pathDisplay: {
    flex: 1,
    padding: '10px 12px',
    backgroundColor: '#111111',
    border: '1px solid #333333',
    borderRadius: '4px',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '12px',
    color: '#ffffff',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  pathPlaceholder: {
    color: '#555555',
  },
  browseButton: {
    padding: '10px 16px',
    backgroundColor: '#222222',
    border: '1px solid #444444',
    borderRadius: '4px',
    color: '#ffffff',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '12px',
    fontWeight: 600,
    cursor: 'pointer',
    whiteSpace: 'nowrap',
    transition: 'background-color 0.15s',
  },
  dbStatusList: {
    width: '100%',
    marginBottom: '24px',
  },
  dbStatusItem: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '6px 0',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '12px',
  },
  statusIcon: {
    width: '16px',
    textAlign: 'center',
    flexShrink: 0,
  },
  statusFound: {
    color: '#22C55E',
  },
  statusNotFound: {
    color: '#EF4444',
  },
  statusPending: {
    color: '#555555',
  },
  errorMessage: {
    width: '100%',
    padding: '10px 12px',
    backgroundColor: 'rgba(239, 68, 68, 0.1)',
    border: '1px solid rgba(239, 68, 68, 0.3)',
    borderRadius: '4px',
    color: '#EF4444',
    fontSize: '12px',
    marginBottom: '24px',
  },
  startButton: {
    width: '100%',
    padding: '12px',
    borderWidth: '1px',
    borderStyle: 'solid',
    borderColor: '#ffffff',
    borderRadius: '4px',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '14px',
    fontWeight: 700,
    letterSpacing: '2px',
    cursor: 'pointer',
    transition: 'all 0.15s',
  },
  startButtonEnabled: {
    backgroundColor: '#ffffff',
    color: '#000000',
    borderColor: '#ffffff',
  },
  startButtonDisabled: {
    backgroundColor: 'transparent',
    color: '#444444',
    borderColor: '#333333',
    cursor: 'not-allowed',
  },
} satisfies Record<string, CSSProperties>

function StatusIcon({ status }: { status: 'found' | 'not-found' | 'pending' }) {
  if (status === 'found') {
    return (
      <span style={{ ...styles.statusIcon, ...styles.statusFound }}>
        &#10003;
      </span>
    )
  }
  if (status === 'not-found') {
    return (
      <span style={{ ...styles.statusIcon, ...styles.statusNotFound }}>
        &#10005;
      </span>
    )
  }
  return (
    <span style={{ ...styles.statusIcon, ...styles.statusPending }}>
      &#8212;
    </span>
  )
}

function getFileStatus(
  name: string,
  dbFileStatuses: DbFileStatus[],
): 'found' | 'not-found' | 'pending' {
  const status = dbFileStatuses.find((s) => s.name === name)
  if (!status) return 'pending'
  return status.found ? 'found' : 'not-found'
}

export function SetupScreen({
  selectedPath,
  dbFileStatuses,
  isValidating,
  error,
  onSelectFolder,
  onStart,
}: SetupScreenProps) {
  const allFound =
    dbFileStatuses.length === DB_FILE_NAMES.length &&
    dbFileStatuses.every((s) => s.found)
  const canStart = allFound && !isValidating && error == null

  return (
    <div style={styles.container}>
      <h1 style={styles.title}>BMS DASHTRAY</h1>
      <p style={styles.description as CSSProperties}>
        beatoraja のプレイ履歴をリアルタイムに表示します。
        <br />
        はじめに beatoraja のルートディレクトリを選択してください。
      </p>

      <div style={styles.sectionLabel as CSSProperties}>BEATORAJA ROOT</div>
      <div style={styles.pathSelector}>
        <div style={styles.pathDisplay as CSSProperties}>
          {selectedPath ?? (
            <span style={styles.pathPlaceholder}>Select folder...</span>
          )}
        </div>
        <button
          type="button"
          style={styles.browseButton as CSSProperties}
          onClick={onSelectFolder}
          disabled={isValidating}
        >
          BROWSE
        </button>
      </div>

      <div style={styles.sectionLabel as CSSProperties}>DATABASE FILES</div>
      <div style={styles.dbStatusList}>
        {DB_FILE_NAMES.map((name) => {
          const status = getFileStatus(name, dbFileStatuses)
          return (
            <div key={name} style={styles.dbStatusItem}>
              <StatusIcon status={status} />
              <span
                style={{
                  color:
                    status === 'not-found'
                      ? '#EF4444'
                      : status === 'found'
                        ? '#ffffff'
                        : '#555555',
                }}
              >
                {name}
              </span>
            </div>
          )
        })}
      </div>

      {error != null && <div style={styles.errorMessage}>{error}</div>}

      <button
        type="button"
        style={{
          ...styles.startButton,
          ...(canStart
            ? styles.startButtonEnabled
            : styles.startButtonDisabled),
        }}
        onClick={onStart}
        disabled={!canStart}
      >
        {isValidating ? 'VALIDATING...' : 'START'}
      </button>
    </div>
  )
}
