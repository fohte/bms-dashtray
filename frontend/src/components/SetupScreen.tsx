import type { CSSProperties } from 'react'

import type { DbFileStatus } from '@/types'

export interface SetupScreenProps {
  selectedPath: string | null
  dbFileStatuses: DbFileStatus[]
  isValidating: boolean
  error: string | null
  players: string[]
  selectedPlayer: string | null
  onSelectFolder: () => void
  onSelectPlayer: (playerName: string) => void
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
  iconFrame: {
    width: '64px',
    height: '64px',
    backgroundColor: '#111111',
    borderRadius: '12px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: '16px',
  },
  iconText: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '16px',
    fontWeight: 700,
    color: '#ffffff',
  },
  title: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '20px',
    fontWeight: 700,
    letterSpacing: '2px',
    marginBottom: '8px',
  },
  description: {
    fontSize: '13px',
    color: '#94A3B8',
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
    width: '100%',
    marginBottom: '16px',
    position: 'relative',
  },
  pathDisplay: {
    flex: 1,
    padding: '12px 48px 12px 16px',
    backgroundColor: '#111111',
    border: '1px solid #222222',
    borderRadius: '8px',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '12px',
    color: '#ffffff',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  pathDisplayError: {
    borderColor: '#EF4444',
  },
  pathPlaceholder: {
    color: '#555555',
  },
  browseButton: {
    position: 'absolute',
    right: '8px',
    padding: '4px 8px',
    backgroundColor: '#FFFFFF',
    border: 'none',
    borderRadius: '4px',
    color: '#000000',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '12px',
    fontWeight: 700,
    cursor: 'pointer',
    whiteSpace: 'nowrap',
    transition: 'background-color 0.15s',
  },
  dbStatusArea: {
    width: '100%',
    marginBottom: '24px',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '12px',
  },
  dbStatusSummary: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  dbStatusItem: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    padding: '4px 0',
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
  playerSection: {
    width: '100%',
    marginBottom: '16px',
  },
  playerButton: {
    display: 'block',
    width: '100%',
    padding: '10px 16px',
    marginBottom: '4px',
    backgroundColor: '#111111',
    border: '1px solid #222222',
    borderRadius: '6px',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '12px',
    color: '#ffffff',
    cursor: 'pointer',
    textAlign: 'left' as const,
    transition: 'all 0.15s',
  },
  playerButtonSelected: {
    borderColor: '#ffffff',
    backgroundColor: '#1a1a1a',
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

export function SetupScreen({
  selectedPath,
  dbFileStatuses,
  isValidating,
  error,
  players,
  selectedPlayer,
  onSelectFolder,
  onSelectPlayer,
  onStart,
}: SetupScreenProps) {
  const allFound =
    dbFileStatuses.length === DB_FILE_NAMES.length &&
    dbFileStatuses.every((s) => s.found)
  const hasNotFound = dbFileStatuses.some((s) => !s.found)
  const canStart = allFound && !isValidating && error == null

  const foundFiles = dbFileStatuses.filter((s) => s.found).map((s) => s.name)
  const notFoundFiles = dbFileStatuses
    .filter((s) => !s.found)
    .map((s) => s.name)

  return (
    <div style={styles.container}>
      <div style={styles.iconFrame}>
        <span style={styles.iconText}>BMS</span>
      </div>
      <h1 style={styles.title}>bms-dashtray</h1>
      <p style={styles.description as CSSProperties}>
        beatoraja のプレイ履歴を
        <br />
        リアルタイムに表示します
      </p>

      <div style={styles.sectionLabel as CSSProperties}>
        BEATORAJA ROOT DIRECTORY
      </div>
      <div style={styles.pathSelector as CSSProperties}>
        <div
          style={{
            ...(styles.pathDisplay as CSSProperties),
            ...(hasNotFound ? styles.pathDisplayError : {}),
          }}
        >
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
          ...
        </button>
      </div>

      {players.length > 1 && (
        <>
          <div style={styles.sectionLabel as CSSProperties}>PLAYER</div>
          <div style={styles.playerSection}>
            {players.map((name) => (
              <button
                key={name}
                type="button"
                style={{
                  ...styles.playerButton,
                  ...(selectedPlayer === name
                    ? styles.playerButtonSelected
                    : {}),
                }}
                onClick={() => onSelectPlayer(name)}
                disabled={isValidating}
              >
                {name}
              </button>
            ))}
          </div>
        </>
      )}

      <div style={styles.dbStatusArea}>
        {dbFileStatuses.length === 0 ? null : allFound ? (
          <div style={styles.dbStatusSummary}>
            <span style={{ ...styles.statusIcon, ...styles.statusFound }}>
              &#10003;
            </span>
            <span style={{ color: '#ffffff' }}>
              {foundFiles.join(', ')} found
            </span>
          </div>
        ) : (
          <>
            {notFoundFiles.map((name) => (
              <div key={name} style={styles.dbStatusItem}>
                <span
                  style={{ ...styles.statusIcon, ...styles.statusNotFound }}
                >
                  &#10005;
                </span>
                <span style={{ color: '#EF4444' }}>
                  {name} が見つかりません
                </span>
              </div>
            ))}
            {foundFiles.map((name) => (
              <div key={name} style={styles.dbStatusItem}>
                <span style={{ ...styles.statusIcon, ...styles.statusFound }}>
                  &#10003;
                </span>
                <span style={{ color: '#ffffff' }}>{name} found</span>
              </div>
            ))}
          </>
        )}
      </div>

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
