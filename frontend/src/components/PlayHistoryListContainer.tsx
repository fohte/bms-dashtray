import { useCallback, useEffect, useMemo, useState } from 'react'

import { DistributionChart } from '@/components/DistributionChart'
import { filterRecords } from '@/components/filterRecords'
import { FilterTabs } from '@/components/FilterTabs'
import { PlayHistoryList } from '@/components/PlayHistoryList'
import type { TauriApi } from '@/tauri-api'
import type { FilterType, PlayRecord } from '@/types'

interface PlayHistoryListContainerProps {
  api: TauriApi
}

export function PlayHistoryListContainer({
  api,
}: PlayHistoryListContainerProps) {
  const [records, setRecords] = useState<PlayRecord[]>([])
  const [activeFilter, setActiveFilter] = useState<FilterType>('all')

  const sortByPlayedAtDesc = useCallback((records: PlayRecord[]) => {
    return [...records].sort(
      (a, b) => new Date(b.playedAt).getTime() - new Date(a.playedAt).getTime(),
    )
  }, [])

  useEffect(() => {
    api
      .getTodayRecords()
      .then((records) => {
        setRecords(sortByPlayedAtDesc(records))
      })
      .catch(() => {
        // Initial load failure is non-fatal; records will arrive via events
      })
  }, [api, sortByPlayedAtDesc])

  useEffect(() => {
    const unlistenPromise = api.listenScoresUpdated((payload) => {
      setRecords(sortByPlayedAtDesc(payload.records))
    })

    return () => {
      void unlistenPromise.then((unlisten) => {
        unlisten()
      })
    }
  }, [api, sortByPlayedAtDesc])

  const filteredRecords = useMemo(
    () => filterRecords(records, activeFilter),
    [records, activeFilter],
  )

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
          onFilterChange={setActiveFilter}
        />
      </div>
      <div style={{ padding: '0 8px' }}>
        <PlayHistoryList records={filteredRecords} />
      </div>
      <DistributionChart records={records} />
    </div>
  )
}
