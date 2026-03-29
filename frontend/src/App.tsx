import { useCallback, useEffect, useState } from 'react'

import { MainScreen } from '@/components/MainScreen'
import { PlayHistoryListContainer } from '@/components/PlayHistoryListContainer'
import { SettingsScreenContainer } from '@/components/SettingsScreenContainer'
import { SetupScreenContainer } from '@/components/SetupScreenContainer'
import { UpdateNotification } from '@/components/UpdateNotification'
import { tauriApi } from '@/tauri-api'
import type { AppConfig } from '@/types'

type AppState = 'loading' | 'setup' | 'main' | 'settings'

function getTodayDate(): string {
  const d = new Date()
  const yyyy = d.getFullYear()
  const mm = String(d.getMonth() + 1).padStart(2, '0')
  const dd = String(d.getDate()).padStart(2, '0')
  return `${String(yyyy)}-${mm}-${dd}`
}

const DEFAULT_FONT_SIZE = 13

export const App = () => {
  const [appState, setAppState] = useState<AppState>('loading')
  const [config, setConfig] = useState<AppConfig | null>(null)
  const [todayDate, setTodayDate] = useState(() => getTodayDate())
  const [resetKey, setResetKey] = useState(0)

  const bgColor =
    config?.backgroundTransparent === true ? 'transparent' : '#000000'

  useEffect(() => {
    const root = document.documentElement
    root.style.fontSize = `${String(config?.fontSize ?? DEFAULT_FONT_SIZE)}px`
  }, [config?.fontSize])

  useEffect(() => {
    document.documentElement.style.backgroundColor = bgColor
    document.body.style.backgroundColor = bgColor
    const rootEl = document.getElementById('root')
    if (rootEl != null) {
      rootEl.style.backgroundColor = bgColor
    }
  }, [bgColor])

  useEffect(() => {
    const interval = setInterval(() => {
      setTodayDate(getTodayDate())
    }, 60_000)
    return () => {
      clearInterval(interval)
    }
  }, [])

  useEffect(() => {
    tauriApi
      .getConfig()
      .then((cfg) => {
        if (cfg != null) {
          setConfig(cfg)
          setAppState('main')
        } else {
          setAppState('setup')
        }
      })
      .catch(() => {
        setAppState('setup')
      })
  }, [])

  const handleSetupComplete = useCallback(() => {
    tauriApi
      .getConfig()
      .then((cfg) => {
        if (cfg != null) {
          setConfig(cfg)
          setAppState('main')
        } else {
          setAppState('setup')
        }
      })
      .catch(() => {
        setAppState('setup')
      })
  }, [])

  const handleOpenSettings = useCallback(() => {
    setAppState('settings')
  }, [])

  const handleBackFromSettings = useCallback(() => {
    setAppState('main')
  }, [])

  const handleConfigChanged = useCallback((updated: AppConfig) => {
    setConfig(updated)
  }, [])

  if (appState === 'loading') {
    return (
      <div
        style={{
          minHeight: '100vh',
          backgroundColor: bgColor,
        }}
      />
    )
  }

  if (appState === 'setup') {
    return (
      <SetupScreenContainer
        api={tauriApi}
        onSetupComplete={handleSetupComplete}
      />
    )
  }

  if (appState === 'settings' && config != null) {
    return (
      <SettingsScreenContainer
        api={tauriApi}
        config={config}
        onBack={handleBackFromSettings}
        onConfigChanged={handleConfigChanged}
      />
    )
  }

  // Main screen
  return (
    <MainScreen
      todayDate={todayDate}
      backgroundTransparent={config?.backgroundTransparent === true}
      onOpenSettings={handleOpenSettings}
      onResetHistory={() => {
        void tauriApi.resetHistory().then(() => {
          setResetKey((k) => k + 1)
        })
      }}
      banner={<UpdateNotification />}
    >
      <PlayHistoryListContainer key={resetKey} api={tauriApi} />
    </MainScreen>
  )
}
