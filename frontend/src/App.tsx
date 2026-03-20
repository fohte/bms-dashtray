import { useCallback, useEffect, useState } from 'react'

import { PlayHistoryListContainer } from '@/components/PlayHistoryListContainer'
import { SettingsScreenContainer } from '@/components/SettingsScreenContainer'
import { SetupScreenContainer } from '@/components/SetupScreenContainer'
import { tauriApi } from '@/tauri-api'
import type { AppConfig } from '@/types'

type AppState = 'loading' | 'setup' | 'main' | 'settings'

function getTodayDate(): string {
  const d = new Date()
  const yyyy = d.getFullYear()
  const mm = String(d.getMonth() + 1).padStart(2, '0')
  const dd = String(d.getDate()).padStart(2, '0')
  return `${yyyy}-${mm}-${dd}`
}

export const App = () => {
  const [appState, setAppState] = useState<AppState>('loading')
  const [config, setConfig] = useState<AppConfig | null>(null)
  const [todayDate, setTodayDate] = useState(() => getTodayDate())
  const [resetKey, setResetKey] = useState(0)

  useEffect(() => {
    const interval = setInterval(() => {
      setTodayDate(getTodayDate())
    }, 60_000)
    return () => clearInterval(interval)
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
          backgroundColor: '#000000',
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
    <div
      style={{
        minHeight: '100vh',
        display: 'flex',
        flexDirection: 'column',
        backgroundColor: '#000000',
        color: '#ffffff',
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '12px 16px',
        }}
      >
        <span
          style={{
            fontFamily: "'JetBrains Mono', monospace",
            fontSize: '14px',
            fontWeight: 700,
            letterSpacing: '1.5px',
          }}
        >
          bms-dashtray
        </span>
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '12px',
          }}
        >
          <span
            style={{
              fontFamily: "'JetBrains Mono', monospace",
              fontSize: '11px',
              fontWeight: 500,
              color: '#64748B',
            }}
          >
            {todayDate}
          </span>
          <button
            type="button"
            onClick={handleOpenSettings}
            style={{
              background: 'none',
              border: 'none',
              color: '#888888',
              fontSize: '18px',
              cursor: 'pointer',
              padding: '4px 8px',
            }}
            aria-label="Settings"
          >
            &#9881;
          </button>
        </div>
      </div>
      <div style={{ height: '1px', backgroundColor: '#1A1A1A' }} />
      <PlayHistoryListContainer key={resetKey} api={tauriApi} />
      <div style={{ flex: 1 }} />
      <div
        style={{
          display: 'flex',
          justifyContent: 'flex-end',
          padding: '12px 16px',
        }}
      >
        <button
          type="button"
          onClick={() => {
            void tauriApi.resetHistory().then(() => {
              setResetKey((k) => k + 1)
            })
          }}
          style={{
            background: 'none',
            border: '1px solid #475569',
            borderRadius: '4px',
            color: '#475569',
            fontFamily: "'JetBrains Mono', monospace",
            fontSize: '11px',
            fontWeight: 600,
            letterSpacing: '1px',
            padding: '6px 12px',
            cursor: 'pointer',
          }}
        >
          RESET
        </button>
      </div>
    </div>
  )
}
