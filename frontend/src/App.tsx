import { useCallback, useEffect, useState } from 'react'

import { PlayHistoryListContainer } from '@/components/PlayHistoryListContainer'
import { SetupScreenContainer } from '@/components/SetupScreenContainer'
import { tauriApi } from '@/tauri-api'

type AppState = 'loading' | 'setup' | 'main'

export const App = () => {
  const [appState, setAppState] = useState<AppState>('loading')

  useEffect(() => {
    tauriApi
      .getConfig()
      .then((config) => {
        setAppState(config != null ? 'main' : 'setup')
      })
      .catch(() => {
        setAppState('setup')
      })
  }, [])

  const handleSetupComplete = useCallback(() => {
    setAppState('main')
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

  return (
    <div
      style={{
        minHeight: '100vh',
        backgroundColor: '#000000',
        color: '#ffffff',
      }}
    >
      <PlayHistoryListContainer api={tauriApi} />
    </div>
  )
}
