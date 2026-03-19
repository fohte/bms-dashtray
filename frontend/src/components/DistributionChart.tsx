import type { CSSProperties } from 'react'

import type { PlayRecord } from '@/types'

export interface LevelDistribution {
  level: number
  count: number
  percentage: number
}

export function aggregateLevelDistribution(
  records: PlayRecord[],
): LevelDistribution[] {
  if (records.length === 0) return []

  const countByLevel = new Map<number, number>()
  for (const record of records) {
    countByLevel.set(record.level, (countByLevel.get(record.level) ?? 0) + 1)
  }

  const total = records.length
  return Array.from(countByLevel.entries())
    .sort(([a], [b]) => a - b)
    .map(([level, count]) => ({
      level,
      count,
      percentage: Math.round((count / total) * 100),
    }))
}

export interface DistributionChartProps {
  records: PlayRecord[]
}

const styles = {
  container: {
    width: '100%',
    padding: '12px 16px',
    fontFamily: "'JetBrains Mono', monospace",
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: '12px',
  },
  headerLabel: {
    fontFamily:
      "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
    fontSize: '11px',
    fontWeight: 600,
    color: '#64748B',
    letterSpacing: '2px',
    textTransform: 'uppercase',
  },
  headerTotal: {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '11px',
    fontWeight: 500,
    color: '#475569',
  },
  emptyMessage: {
    fontSize: '12px',
    color: '#555555',
    textAlign: 'center',
    padding: '16px 0',
  },
  row: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    marginBottom: '4px',
    height: '24px',
  },
  levelLabel: {
    fontSize: '10px',
    fontWeight: 600,
    color: '#64748B',
    width: '38px',
    textAlign: 'right',
    flexShrink: 0,
  },
  barContainer: {
    flex: 1,
    height: '16px',
    backgroundColor: '#111111',
    borderRadius: '3px',
    overflow: 'hidden',
  },
  bar: {
    height: '100%',
    backgroundColor: '#ffffff',
    borderRadius: '3px',
    transition: 'width 0.3s ease',
  },
  countLabel: {
    fontSize: '10px',
    fontWeight: 700,
    color: '#94A3B8',
    width: '16px',
    textAlign: 'right',
    flexShrink: 0,
  },
} satisfies Record<string, CSSProperties>

export function DistributionChart({ records }: DistributionChartProps) {
  const distribution = aggregateLevelDistribution(records)
  const maxCount = Math.max(0, ...distribution.map((d) => d.count))
  const total = records.length

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <span style={styles.headerLabel as CSSProperties}>
          DIFFICULTY DISTRIBUTION
        </span>
        <span style={styles.headerTotal}>{total} total</span>
      </div>
      {distribution.length === 0 ? (
        <div style={styles.emptyMessage as CSSProperties}>No play data</div>
      ) : (
        distribution.map((item) => (
          <div key={item.level} style={styles.row}>
            <div style={styles.levelLabel as CSSProperties}>
              Lv.{item.level}
            </div>
            <div style={styles.barContainer}>
              <div
                style={{
                  ...styles.bar,
                  width:
                    maxCount > 0 ? `${(item.count / maxCount) * 100}%` : '0%',
                }}
                role="meter"
                aria-label={`Lv.${item.level}`}
                aria-valuenow={item.count}
                aria-valuemin={0}
                aria-valuemax={maxCount}
              />
            </div>
            <div style={styles.countLabel as CSSProperties}>{item.count}</div>
          </div>
        ))
      )}
    </div>
  )
}
