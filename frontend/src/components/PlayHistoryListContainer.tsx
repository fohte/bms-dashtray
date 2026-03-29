import { useCallback, useEffect, useMemo, useState } from 'react'

import { filterRecords } from '@/components/filterRecords'
import { PlayHistoryPanel } from '@/components/PlayHistoryPanel'
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
    <PlayHistoryPanel
      records={records}
      filteredRecords={filteredRecords}
      activeFilter={activeFilter}
      onFilterChange={setActiveFilter}
    />
  )
}
