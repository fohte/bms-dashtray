import { useCallback, useState } from 'react'

import { SetupScreen } from '@/components/SetupScreen'
import type { TauriApi } from '@/tauri-api'
import type { DbFileStatus } from '@/types'

interface SetupScreenContainerProps {
  api: TauriApi
  onSetupComplete: () => void
}

export function SetupScreenContainer({
  api,
  onSetupComplete,
}: SetupScreenContainerProps) {
  const [selectedPath, setSelectedPath] = useState<string | null>(null)
  const [dbFileStatuses, setDbFileStatuses] = useState<DbFileStatus[]>([])
  const [isValidating, setIsValidating] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleSelectFolder = useCallback(async () => {
    const path = await api.openFolderDialog()
    if (path == null) return

    setSelectedPath(path)
    setError(null)
    setIsValidating(true)
    setDbFileStatuses([])

    try {
      await api.validateAndSaveConfig(path)
      setDbFileStatuses([
        { name: 'songdata.db', found: true },
        { name: 'scoredatalog.db', found: true },
        { name: 'score.db', found: true },
        { name: 'scorelog.db', found: true },
      ])
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e)
      const missingFiles = parseMissingFiles(message)
      const allFiles = [
        'songdata.db',
        'scoredatalog.db',
        'score.db',
        'scorelog.db',
      ]
      setDbFileStatuses(
        allFiles.map((name) => ({
          name,
          found: !missingFiles.includes(name),
        })),
      )
      setError(message)
    } finally {
      setIsValidating(false)
    }
  }, [api])

  const handleStart = useCallback(() => {
    onSetupComplete()
  }, [onSetupComplete])

  return (
    <SetupScreen
      selectedPath={selectedPath}
      dbFileStatuses={dbFileStatuses}
      isValidating={isValidating}
      error={error}
      onSelectFolder={handleSelectFolder}
      onStart={handleStart}
    />
  )
}

function parseMissingFiles(errorMessage: string): string[] {
  const dbFiles = ['songdata.db', 'scoredatalog.db', 'score.db', 'scorelog.db']
  return dbFiles.filter((file) => errorMessage.includes(file))
}
