import '@/components/clear-lamp-animations.css'

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
  0: '#000000',
  1: '#E92F0A',
  2: '#CE01D6',
  3: '#DDA2DF',
  4: '#56CA43',
  5: '#F5C758',
  6: '#F8F7F5',
  7: '#EFFD09',
  8: '#FFFFFF',
  9: '#FFFFFF',
  10: '#FFFFFF',
}

const CLEAR_LAMP_ALT_COLORS: Record<number, string> = {
  1: '#0F0300',
  7: '#FD0909',
  8: '#09FAFD',
  9: '#3FFF4D',
  10: '#FFEB42',
}

function getClearLampName(clear: number, isRetired: boolean): string {
  if (clear === 1 && isRetired) return 'Retired'
  return CLEAR_LAMP_NAMES[clear] ?? `Unknown(${String(clear)})`
}

/** Dimmer red for mid-play retirement, distinct from full-play Failed. */
const RETIRED_LAMP_COLOR = '#8B2500'

function getClearLampColor(clear: number, isRetired: boolean): string {
  if (clear === 1 && isRetired) return RETIRED_LAMP_COLOR
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
    alignItems: 'center',
    borderRadius: '8px',
    padding: '10px 12px',
    gap: '12px',
  },
  lampBar: {
    width: '4px',
    height: '32px',
    borderRadius: '2px',
    flexShrink: 0,
  },
  content: {
    flex: 1,
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
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
    fontSize: 'var(--font-size-base)',
    fontWeight: 600,
    color: '#ffffff',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  subtitle: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: 'var(--font-size-xs)',
    fontWeight: 500,
    color: '#475569',
    flexShrink: 0,
    whiteSpace: 'nowrap',
  },
  clearRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
  },
  level: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: 'var(--font-size-md)',
    fontWeight: 600,
    color: '#94A3B8',
    flexShrink: 0,
  },
  clearName: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: 'var(--font-size-sm)',
    fontWeight: 700,
    letterSpacing: '1px',
  },
  previousClear: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: 'var(--font-size-sm)',
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
    fontSize: 'var(--font-size-lg)',
    color: '#ffffff',
    fontWeight: 700,
  },
  scoreDiff: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: 'var(--font-size-sm)',
    fontWeight: 500,
  },
  bpRow: {
    display: 'flex',
    alignItems: 'baseline',
    gap: '6px',
  },
  bpValue: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: 'var(--font-size-sm)',
    fontWeight: 500,
    color: '#64748B',
  },
  bpDiff: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: 'var(--font-size-xs)',
    fontWeight: 500,
  },
  emptyState: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '48px 24px',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: 'var(--font-size-base)',
    color: '#555555',
  },
} satisfies Record<string, CSSProperties>

function buildFlashStyle(lampClear: number, lampColor: string): CSSProperties {
  const altColor = CLEAR_LAMP_ALT_COLORS[lampClear]
  if (altColor == null) return {}
  const cycleMs = lampClear === 1 ? 50 : 100
  return {
    '--lamp-main-color': lampColor,
    '--lamp-alt-color': altColor,
    animation: `lampFlash ${cycleMs}ms step-end infinite`,
  } as CSSProperties
}

function LampBar({
  clear,
  previousClear,
  isRetired,
  currentColor,
  previousColor,
}: {
  clear: number
  previousClear: number | null
  isRetired: boolean
  currentColor: string
  previousColor: string | null
}) {
  // No flash animation for retired plays — the dimmer color is enough distinction.
  const flashStyle = isRetired ? {} : buildFlashStyle(clear, currentColor)

  if (previousColor != null && previousColor !== currentColor) {
    const previousFlashStyle =
      !isRetired && previousClear != null
        ? buildFlashStyle(previousClear, previousColor)
        : {}
    return (
      <div
        style={{
          ...styles.lampBar,
          display: 'flex',
          flexDirection: 'column',
          background: 'none',
        }}
      >
        <div
          style={{
            flex: 1,
            borderRadius: '2px 2px 0 0',
            backgroundColor: previousColor,
            ...previousFlashStyle,
          }}
        />
        <div
          style={{
            flex: 1,
            borderRadius: '0 0 2px 2px',
            backgroundColor: currentColor,
            ...flashStyle,
          }}
        />
      </div>
    )
  }

  return (
    <div
      style={{
        ...styles.lampBar,
        backgroundColor: currentColor,
        ...flashStyle,
      }}
    />
  )
}

function PlayHistoryEntry({
  record,
  index,
}: {
  record: PlayRecord
  index: number
}) {
  const lampColor = getClearLampColor(record.clear, record.isRetired)
  const clearName = getClearLampName(record.clear, record.isRetired)
  const bgColor = index % 2 === 0 ? '#111111' : '#0A0A0A'

  const clearUpdated =
    record.previousClear != null && record.clear > record.previousClear
  const previousLampColor =
    record.previousClear != null
      ? getClearLampColor(record.previousClear, false)
      : null
  const exScoreDiff = formatDiff(record.exScore, record.previousExScore, false)
  const bpDiff = formatDiff(record.minBp, record.previousMinBp, true)

  return (
    <div style={{ ...styles.entry, backgroundColor: bgColor }}>
      <LampBar
        clear={record.clear}
        previousClear={clearUpdated ? record.previousClear : null}
        isRetired={record.isRetired}
        currentColor={lampColor}
        previousColor={clearUpdated ? previousLampColor : null}
      />
      <div style={styles.content}>
        <div style={styles.left as CSSProperties}>
          <div style={styles.titleRow as CSSProperties}>
            <span style={styles.title as CSSProperties}>{record.title}</span>
            {record.subtitle !== '' && (
              <span style={styles.subtitle}>{record.subtitle}</span>
            )}
          </div>
          <div style={styles.clearRow}>
            {record.tableLevels.length > 0 ? (
              record.tableLevels.map((tl, i) => (
                <span key={i} style={styles.level}>
                  {tl}
                </span>
              ))
            ) : (
              <span style={styles.level}>Lv.{record.level}</span>
            )}
            <span style={{ ...styles.clearName, color: lampColor }}>
              {clearName}
            </span>
            {clearUpdated && (
              <span style={styles.previousClear}>
                {'< '}
                {record.previousClear != null &&
                  getClearLampName(record.previousClear, false)}
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
