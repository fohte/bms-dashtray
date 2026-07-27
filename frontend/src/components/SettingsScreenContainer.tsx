import { getVersion } from '@tauri-apps/api/app'
import { ResultAsync } from 'neverthrow'
import { useCallback, useEffect, useState } from 'react'

import { SettingsScreen } from '#components/SettingsScreen'
import { useUpdateChecker } from '#hooks/useUpdateChecker'
import type { TauriApi } from '#tauri-api'
import type { AppConfig } from '#types'

export interface SettingsScreenContainerProps {
  api: TauriApi
  config: AppConfig
  onBack: () => void
  onConfigChanged: (config: AppConfig) => void
}

export function SettingsScreenContainer({
  api,
  config,
  onBack,
  onConfigChanged,
}: SettingsScreenContainerProps) {
  const [currentConfig, setCurrentConfig] = useState<AppConfig>(config)
  const [appVersion, setAppVersion] = useState<string | null>(null)
  const {
    state: updateCheckState,
    checkForUpdates,
    installUpdate,
  } = useUpdateChecker()

  useEffect(() => {
    void getVersion().then(setAppVersion)
  }, [])

  const updateConfig = useCallback(
    (patch: Partial<AppConfig>) => {
      const updated = { ...currentConfig, ...patch }
      setCurrentConfig(updated)
      onConfigChanged(updated)
    },
    [currentConfig, onConfigChanged],
  )

  const handleChangeBeatorajaRoot = useCallback(async () => {
    const path = await api.openFolderDialog()
    if (path == null) return

    // Validation failed at any step — keep current config
    const result = await ResultAsync.fromPromise(
      api.detectPlayers(path),
      (e) => e,
    )
      .andThen((players) => {
        // Use the first detected player (settings screen doesn't need a picker)
        const playerName = players[0] ?? ''
        return ResultAsync.fromPromise(
          api.validateAndSaveConfig(path, playerName),
          (e) => e,
        )
      })
      .andThen(() => ResultAsync.fromPromise(api.getConfig(), (e) => e))

    if (result.isOk() && result.value != null) {
      setCurrentConfig(result.value)
      onConfigChanged(result.value)
    }
  }, [api, onConfigChanged])

  const handleToggleBackgroundTransparent = useCallback(
    async (value: boolean) => {
      updateConfig({ backgroundTransparent: value })
      await api.updateSettings({ backgroundTransparent: value })
    },
    [api, updateConfig],
  )

  const handleChangeFontSize = useCallback(
    async (delta: number) => {
      const newSize = Math.max(8, Math.min(24, currentConfig.fontSize + delta))
      if (newSize === currentConfig.fontSize) return
      updateConfig({ fontSize: newSize })
      await api.updateSettings({ fontSize: newSize })
    },
    [api, currentConfig.fontSize, updateConfig],
  )

  const handleChangeResetTime = useCallback(
    async (time: string) => {
      updateConfig({ resetTime: time })
      await api.updateSettings({ resetTime: time })
    },
    [api, updateConfig],
  )

  const handleResetHistory = useCallback(async () => {
    await api.resetHistory()
  }, [api])

  return (
    <SettingsScreen
      config={currentConfig}
      appVersion={appVersion}
      updateCheckState={updateCheckState}
      onBack={onBack}
      onChangeBeatorajaRoot={handleChangeBeatorajaRoot}
      onToggleBackgroundTransparent={handleToggleBackgroundTransparent}
      onChangeFontSize={handleChangeFontSize}
      onChangeResetTime={handleChangeResetTime}
      onResetHistory={handleResetHistory}
      onCheckForUpdates={checkForUpdates}
      onInstallUpdate={installUpdate}
    />
  )
}
