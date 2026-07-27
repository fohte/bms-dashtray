import { ResultAsync } from 'neverthrow'
import { useCallback, useState } from 'react'

import { SetupScreen } from '#components/SetupScreen'
import type { TauriApi } from '#tauri-api'
import type { DbFileStatus } from '#types'

function toMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e)
}

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
  const [players, setPlayers] = useState<string[]>([])
  const [selectedPlayer, setSelectedPlayer] = useState<string | null>(null)

  const validateWithPlayer = useCallback(
    async (path: string, playerName: string) => {
      setIsValidating(true)
      setError(null)
      setDbFileStatuses([])

      const result = await ResultAsync.fromPromise(
        api.validateAndSaveConfig(path, playerName),
        toMessage,
      )

      if (result.isErr()) {
        const missingFiles = parseMissingFiles(result.error)
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
        setError(result.error)
      } else {
        setDbFileStatuses([
          { name: 'songdata.db', found: true },
          { name: 'scoredatalog.db', found: true },
          { name: 'score.db', found: true },
          { name: 'scorelog.db', found: true },
        ])
      }

      setIsValidating(false)
    },
    [api],
  )

  const handleSelectFolder = useCallback(async () => {
    const path = await api.openFolderDialog()
    if (path == null) return

    setSelectedPath(path)
    setError(null)
    setPlayers([])
    setSelectedPlayer(null)
    setDbFileStatuses([])
    setIsValidating(true)

    const result = await ResultAsync.fromPromise(
      api.detectPlayers(path),
      toMessage,
    )

    if (result.isErr()) {
      setError(result.error)
      setIsValidating(false)
      return
    }

    const detected = result.value
    if (detected.length === 1) {
      // Single player: auto-select and validate
      const [player] = detected
      if (player != null) {
        setSelectedPlayer(player)
        setPlayers(detected)
        await validateWithPlayer(path, player)
      }
    } else {
      // Multiple players: let user choose
      setPlayers(detected)
      setIsValidating(false)
    }
  }, [api, validateWithPlayer])

  const handleSelectPlayer = useCallback(
    async (playerName: string) => {
      setSelectedPlayer(playerName)
      if (selectedPath != null) {
        await validateWithPlayer(selectedPath, playerName)
      }
    },
    [selectedPath, validateWithPlayer],
  )

  const handleStart = useCallback(() => {
    onSetupComplete()
  }, [onSetupComplete])

  return (
    <SetupScreen
      selectedPath={selectedPath}
      dbFileStatuses={dbFileStatuses}
      isValidating={isValidating}
      error={error}
      players={players}
      selectedPlayer={selectedPlayer}
      onSelectFolder={handleSelectFolder}
      onSelectPlayer={handleSelectPlayer}
      onStart={handleStart}
    />
  )
}

function parseMissingFiles(errorMessage: string): string[] {
  const dbFiles = ['songdata.db', 'scoredatalog.db', 'score.db', 'scorelog.db']
  return dbFiles.filter((file) => errorMessage.includes(file))
}
