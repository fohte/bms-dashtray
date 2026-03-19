import type { CSSProperties } from 'react'

import type { PlayRecord } from '@/types'

export interface PlayHistoryListProps {
  records: PlayRecord[]
}

const CLEAR_LAMP_NAMES: Record<number, string> = {
  0: 'NoPlay',
  1: 'Failed',
  2: 'AssistEasy',
  3: 'LightAssistEasy',
  4: 'Easy',
  5: 'Normal',
  6: 'Hard',
  7: 'ExHard',
  8: 'FullCombo',
  9: 'Perfect',
  10: 'Max',
}

const CLEAR_LAMP_COLORS: Record<number, string> = {
  0: '#555555',
  1: '#EF4444',
  2: '#888888',
  3: '#888888',
  4: '#22C55E',
  5: '#3B82F6',
  6: '#F97316',
  7: '#FBBF24',
  8: '#ffffff',
  9: '#ffffff',
  10: '#ffffff',
}

function getClearLampName(clear: number): string {
  return CLEAR_LAMP_NAMES[clear] ?? `Unknown(${String(clear)})`
}

function getClearLampColor(clear: number): string {
  return CLEAR_LAMP_COLORS[clear] ?? '#555555'
}

function formatDiff(
  current: number,
  previous: number | null,
  invertColor: boolean,
): { text: string; color: string } | null {
  if (previous == null) return null
  const diff = current - previous
  if (diff === 0) return { text: '\u00b10', color: '#888888' }
  const sign = diff > 0 ? '+' : ''
  const isPositive = diff > 0
  const color = invertColor
    ? isPositive
      ? '#EF4444'
      : '#22C55E'
    : isPositive
      ? '#22C55E'
      : '#EF4444'
  return { text: `${sign}${String(diff)}`, color }
}

const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    gap: '2px',
  },
  entry: {
    display: 'flex',
    alignItems: 'stretch',
    minHeight: '56px',
  },
  lampBar: {
    width: '4px',
    flexShrink: 0,
  },
  content: {
    flex: 1,
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '10px 12px',
    gap: '12px',
    minWidth: 0,
  },
  left: {
    display: 'flex',
    flexDirection: 'column',
    gap: '2px',
    minWidth: 0,
  },
  titleRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    minWidth: 0,
  },
  title: {
    fontFamily:
      "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
    fontSize: '13px',
    fontWeight: 600,
    color: '#ffffff',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  upBadge: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '8px',
    fontWeight: 700,
    color: '#0A0F1C',
    backgroundColor: '#22D3EE',
    padding: '2px 6px',
    borderRadius: '4px',
    letterSpacing: '1px',
    flexShrink: 0,
  },
  clearRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
  },
  level: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '11px',
    fontWeight: 600,
    color: '#94A3B8',
    flexShrink: 0,
  },
  clearName: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '10px',
    fontWeight: 700,
    letterSpacing: '1px',
  },
  previousClear: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '10px',
    fontWeight: 500,
    color: '#475569',
  },
  right: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'flex-end',
    gap: '2px',
    flexShrink: 0,
  },
  scoreRow: {
    display: 'flex',
    alignItems: 'baseline',
    gap: '6px',
  },
  scoreValue: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '14px',
    color: '#ffffff',
    fontWeight: 700,
  },
  scoreDiff: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '10px',
    fontWeight: 500,
  },
  bpRow: {
    display: 'flex',
    alignItems: 'baseline',
    gap: '6px',
  },
  bpValue: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '10px',
    fontWeight: 500,
    color: '#64748B',
  },
  bpDiff: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '9px',
    fontWeight: 500,
  },
  emptyState: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '48px 24px',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '13px',
    color: '#555555',
  },
} satisfies Record<string, CSSProperties>

function PlayHistoryEntry({
  record,
  index,
}: {
  record: PlayRecord
  index: number
}) {
  const lampColor = getClearLampColor(record.clear)
  const clearName = getClearLampName(record.clear)
  const bgColor = index % 2 === 0 ? '#111111' : '#0A0A0A'

  const clearUpdated =
    record.previousClear != null && record.clear > record.previousClear

  const exScoreDiff = formatDiff(record.exScore, record.previousExScore, false)
  const bpDiff = formatDiff(record.minBp, record.previousMinBp, true)

  return (
    <div style={styles.entry}>
      <div style={{ ...styles.lampBar, backgroundColor: lampColor }} />
      <div style={{ ...styles.content, backgroundColor: bgColor }}>
        <div style={styles.left as CSSProperties}>
          <div style={styles.titleRow as CSSProperties}>
            <span style={styles.title as CSSProperties}>{record.title}</span>
            {clearUpdated && <span style={styles.upBadge}>UP</span>}
          </div>
          <div style={styles.clearRow}>
            <span style={styles.level}>Lv.{record.level}</span>
            <span style={{ ...styles.clearName, color: lampColor }}>
              {clearName}
            </span>
            {clearUpdated && (
              <span style={styles.previousClear}>
                {'< '}
                {getClearLampName(record.previousClear!)}
              </span>
            )}
          </div>
        </div>
        <div style={styles.right as CSSProperties}>
          <div style={styles.scoreRow}>
            <span style={styles.scoreValue}>{record.exScore}</span>
            {exScoreDiff != null && (
              <span style={{ ...styles.scoreDiff, color: exScoreDiff.color }}>
                {exScoreDiff.text}
              </span>
            )}
          </div>
          <div style={styles.bpRow}>
            <span style={styles.bpValue}>{record.minBp} bp</span>
            {bpDiff != null && (
              <span style={{ ...styles.bpDiff, color: bpDiff.color }}>
                {bpDiff.text}
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

export function PlayHistoryList({ records }: PlayHistoryListProps) {
  if (records.length === 0) {
    return <div style={styles.emptyState}>No plays recorded today</div>
  }

  return (
    <div style={styles.container as CSSProperties}>
      {records.map((record, index) => (
        <PlayHistoryEntry key={record.id} record={record} index={index} />
      ))}
    </div>
  )
}
