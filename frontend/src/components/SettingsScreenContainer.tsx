import { useCallback, useState } from 'react'

import { SettingsScreen } from '@/components/SettingsScreen'
import type { TauriApi } from '@/tauri-api'
import type { AppConfig } from '@/types'

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

  const updateConfig = useCallback(
    (patch: Partial<AppConfig>) => {
      setCurrentConfig((prev) => {
        const updated = { ...prev, ...patch }
        onConfigChanged(updated)
        return updated
      })
    },
    [onConfigChanged],
  )

  const handleChangeBeatorajaRoot = useCallback(async () => {
    const path = await api.openFolderDialog()
    if (path == null) return

    try {
      await api.validateAndSaveConfig(path)
      const refreshed = await api.getConfig()
      if (refreshed != null) {
        setCurrentConfig(refreshed)
        onConfigChanged(refreshed)
      }
    } catch {
      // Validation failed — keep current config
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
      setCurrentConfig((prev) => {
        const newSize = Math.max(8, Math.min(24, prev.fontSize + delta))
        if (newSize === prev.fontSize) return prev
        const updated = { ...prev, fontSize: newSize }
        onConfigChanged(updated)
        api.updateSettings({ fontSize: newSize })
        return updated
      })
    },
    [api, onConfigChanged],
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
      onBack={onBack}
      onChangeBeatorajaRoot={handleChangeBeatorajaRoot}
      onToggleBackgroundTransparent={handleToggleBackgroundTransparent}
      onChangeFontSize={handleChangeFontSize}
      onChangeResetTime={handleChangeResetTime}
      onResetHistory={handleResetHistory}
    />
  )
}
