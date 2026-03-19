import { useCallback, useEffect, useState } from 'react'

import { SettingsScreenContainer } from '@/components/SettingsScreenContainer'
import { SetupScreenContainer } from '@/components/SetupScreenContainer'
import { tauriApi } from '@/tauri-api'
import type { AppConfig } from '@/types'

type AppState = 'loading' | 'setup' | 'main' | 'settings'

export const App = () => {
  const [appState, setAppState] = useState<AppState>('loading')
  const [config, setConfig] = useState<AppConfig | null>(null)

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
        fontFamily: "'JetBrains Mono', monospace",
        fontSize: '14px',
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '12px 16px',
          borderBottom: '1px solid #222222',
        }}
      >
        <span
          style={{
            fontWeight: 700,
            letterSpacing: '2px',
          }}
        >
          BMS DASHTRAY
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
      <div
        style={{
          flex: 1,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        BMS DASHTRAY
      </div>
    </div>
  )
}
