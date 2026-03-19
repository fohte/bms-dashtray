import type { CSSProperties } from 'react'

import type { FilterType } from '@/types'

export interface FilterTabsProps {
  activeFilter: FilterType
  onFilterChange: (filter: FilterType) => void
}

const FILTERS: { type: FilterType; label: string }[] = [
  { type: 'all', label: 'ALL' },
  { type: 'updated', label: 'UPDATED' },
]

const styles = {
  container: {
    display: 'flex',
    gap: '8px',
  },
  tab: {
    padding: '6px 16px',
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: '11px',
    fontWeight: 600,
    letterSpacing: '1px',
    borderRadius: '4px',
    cursor: 'pointer',
    transition: 'all 0.15s',
    borderWidth: '1px',
    borderStyle: 'solid',
  },
  tabActive: {
    backgroundColor: '#ffffff',
    color: '#000000',
    borderColor: '#ffffff',
  },
  tabInactive: {
    backgroundColor: 'transparent',
    color: '#888888',
    borderColor: '#333333',
  },
} satisfies Record<string, CSSProperties>

export function FilterTabs({ activeFilter, onFilterChange }: FilterTabsProps) {
  return (
    <div style={styles.container}>
      {FILTERS.map(({ type, label }) => (
        <button
          key={type}
          type="button"
          style={{
            ...styles.tab,
            ...(activeFilter === type ? styles.tabActive : styles.tabInactive),
          }}
          onClick={() => onFilterChange(type)}
          aria-pressed={activeFilter === type}
        >
          {label}
        </button>
      ))}
    </div>
  )
}
