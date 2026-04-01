import { getVersion } from '@tauri-apps/api/app'
import { relaunch } from '@tauri-apps/plugin-process'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { useCallback, useEffect, useState } from 'react'

import {
  SettingsScreen,
  type UpdateCheckState,
} from '@/components/SettingsScreen'
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
  const [appVersion, setAppVersion] = useState<string | null>(null)
  const [updateCheckState, setUpdateCheckState] = useState<UpdateCheckState>({
    status: 'idle',
  })
  const [pendingUpdate, setPendingUpdate] = useState<Update | null>(null)

  useEffect(() => {
    void getVersion().then(setAppVersion)
  }, [])

  const handleCheckForUpdates = useCallback(() => {
    setUpdateCheckState({ status: 'checking' })

    const doCheck = async () => {
      try {
        const update = await check()
        if (update) {
          setPendingUpdate(update)
          setUpdateCheckState({
            status: 'available',
            version: update.version,
          })
        } else {
          setUpdateCheckState({ status: 'up-to-date' })
        }
      } catch (e) {
        setUpdateCheckState({
          status: 'error',
          message: e instanceof Error ? e.message : String(e),
        })
      }
    }

    void doCheck()
  }, [])

  const handleInstallUpdate = useCallback(() => {
    if (pendingUpdate == null) return

    const doUpdate = async () => {
      try {
        let totalLength = 0
        let downloadedLength = 0

        setUpdateCheckState({ status: 'downloading', progress: 0 })

        await pendingUpdate.downloadAndInstall((progress) => {
          if (
            progress.event === 'Started' &&
            progress.data.contentLength != null &&
            progress.data.contentLength > 0
          ) {
            totalLength = progress.data.contentLength
          } else if (progress.event === 'Progress') {
            downloadedLength += progress.data.chunkLength
            if (totalLength > 0) {
              setUpdateCheckState({
                status: 'downloading',
                progress: Math.round((downloadedLength / totalLength) * 100),
              })
            }
          }
        })

        await relaunch()
      } catch (e) {
        setUpdateCheckState({
          status: 'error',
          message: e instanceof Error ? e.message : String(e),
        })
      }
    }

    void doUpdate()
  }, [pendingUpdate])

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

    try {
      const players = await api.detectPlayers(path)
      // Use the first detected player (settings screen doesn't need a picker)
      const playerName = players[0] ?? ''
      await api.validateAndSaveConfig(path, playerName)
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
      onCheckForUpdates={handleCheckForUpdates}
      onInstallUpdate={handleInstallUpdate}
    />
  )
}
