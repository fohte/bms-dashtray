import { type CSSProperties, useCallback, useEffect, useState } from 'react'

import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'

type UpdateState =
  | { status: 'idle' }
  | { status: 'available'; version: string }
  | { status: 'downloading'; progress: number }
  | { status: 'error'; message: string }

export const UpdateNotification = () => {
  const [state, setState] = useState<UpdateState>({ status: 'idle' })

  useEffect(() => {
    let cancelled = false

    const checkForUpdate = async () => {
      try {
        const update = await check()
        if (!cancelled && update) {
          setState({ status: 'available', version: update.version })
        }
      } catch (e) {
        if (!cancelled) {
          console.error('Failed to check for updates:', e)
        }
      }
    }

    void checkForUpdate()

    return () => {
      cancelled = true
    }
  }, [])

  const handleUpdate = useCallback(() => {
    const doUpdate = async () => {
      try {
        const update = await check()
        if (!update) return

        let totalLength = 0
        let downloadedLength = 0

        await update.downloadAndInstall((progress) => {
          if (progress.event === 'Started' && progress.data.contentLength) {
            totalLength = progress.data.contentLength
          } else if (progress.event === 'Progress') {
            downloadedLength += progress.data.chunkLength
            if (totalLength > 0) {
              setState({
                status: 'downloading',
                progress: Math.round((downloadedLength / totalLength) * 100),
              })
            }
          }
        })

        await relaunch()
      } catch (e) {
        setState({
          status: 'error',
          message: e instanceof Error ? e.message : String(e),
        })
      }
    }

    setState({ status: 'downloading', progress: 0 })
    void doUpdate()
  }, [])

  const handleDismiss = useCallback(() => {
    setState({ status: 'idle' })
  }, [])

  if (state.status === 'idle') return null

  return (
    <div style={containerStyle}>
      {state.status === 'available' && (
        <>
          <span style={textStyle}>v{state.version} available</span>
          <button
            type="button"
            onClick={handleUpdate}
            style={updateButtonStyle}
          >
            UPDATE
          </button>
          <button
            type="button"
            onClick={handleDismiss}
            style={dismissButtonStyle}
          >
            DISMISS
          </button>
        </>
      )}
      {state.status === 'downloading' && (
        <span style={textStyle}>Downloading... {state.progress}%</span>
      )}
      {state.status === 'error' && (
        <>
          <span style={{ ...textStyle, color: '#EF4444' }}>
            Update failed: {state.message}
          </span>
          <button
            type="button"
            onClick={handleDismiss}
            style={dismissButtonStyle}
          >
            DISMISS
          </button>
        </>
      )}
    </div>
  )
}

const containerStyle: CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: '8px',
  padding: '8px 16px',
  backgroundColor: '#1A1A2E',
  borderBottom: '1px solid #1A1A1A',
  fontFamily: "'JetBrains Mono', monospace",
  fontSize: '11px',
}

const textStyle: CSSProperties = {
  color: '#94A3B8',
  fontWeight: 500,
}

const updateButtonStyle: CSSProperties = {
  background: 'none',
  border: '1px solid #3B82F6',
  borderRadius: '4px',
  color: '#3B82F6',
  fontFamily: "'JetBrains Mono', monospace",
  fontSize: '10px',
  fontWeight: 600,
  letterSpacing: '1px',
  padding: '3px 8px',
  cursor: 'pointer',
}

const dismissButtonStyle: CSSProperties = {
  background: 'none',
  border: 'none',
  color: '#475569',
  fontFamily: "'JetBrains Mono', monospace",
  fontSize: '10px',
  fontWeight: 600,
  padding: '3px 8px',
  cursor: 'pointer',
}
