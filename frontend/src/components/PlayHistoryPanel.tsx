import { DistributionChart } from '@/components/DistributionChart'
import { FilterTabs } from '@/components/FilterTabs'
import { PlayHistoryList } from '@/components/PlayHistoryList'
import type { FilterType, PlayRecord } from '@/types'

export interface PlayHistoryPanelProps {
  records: PlayRecord[]
  filteredRecords: PlayRecord[]
  activeFilter: FilterType
  onFilterChange: (filter: FilterType) => void
}

export function PlayHistoryPanel({
  records,
  filteredRecords,
  activeFilter,
  onFilterChange,
}: PlayHistoryPanelProps) {
  return (
    <div>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '12px 16px 8px',
        }}
      >
        <span
          style={{
            fontFamily:
              "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
            fontSize: 'var(--font-size-md)',
            fontWeight: 600,
            letterSpacing: '2px',
            color: '#64748B',
          }}
        >
          PLAY HISTORY
        </span>
        <span
          style={{
            fontFamily: "'JetBrains Mono', monospace",
            fontSize: 'var(--font-size-md)',
            fontWeight: 500,
            color: '#475569',
          }}
        >
          {records.length} plays
        </span>
      </div>
      <div style={{ padding: '0 16px 8px' }}>
        <FilterTabs
          activeFilter={activeFilter}
          onFilterChange={onFilterChange}
        />
      </div>
      <div style={{ padding: '0 8px' }}>
        <PlayHistoryList records={filteredRecords} />
      </div>
      <DistributionChart records={filteredRecords} />
    </div>
  )
}
