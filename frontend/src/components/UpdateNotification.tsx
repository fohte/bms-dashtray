import { relaunch } from '@tauri-apps/plugin-process'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { useCallback, useEffect, useState } from 'react'

import {
  UpdateNotificationBar,
  type UpdateNotificationState,
} from '@/components/UpdateNotificationBar'

type InternalState =
  | { status: 'idle' }
  | { status: 'available'; update: Update }
  | { status: 'downloading'; progress: number }
  | { status: 'error'; message: string }

function toBarState(state: InternalState): UpdateNotificationState | null {
  switch (state.status) {
    case 'idle':
      return null
    case 'available':
      return { status: 'available', version: state.update.version }
    case 'downloading':
      return { status: 'downloading', progress: state.progress }
    case 'error':
      return { status: 'error', message: state.message }
  }
}

export const UpdateNotification = () => {
  const [state, setState] = useState<InternalState>({ status: 'idle' })

  useEffect(() => {
    let cancelled = false

    const checkForUpdate = async () => {
      try {
        const update = await check()
        if (!cancelled && update) {
          setState({ status: 'available', update })
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
    if (state.status !== 'available') return
    const { update } = state

    const doUpdate = async () => {
      try {
        let totalLength = 0
        let downloadedLength = 0

        await update.downloadAndInstall((progress) => {
          if (
            progress.event === 'Started' &&
            progress.data.contentLength != null &&
            progress.data.contentLength > 0
          ) {
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
  }, [state])

  const handleDismiss = useCallback(() => {
    setState({ status: 'idle' })
  }, [])

  const barState = toBarState(state)
  if (!barState) return null

  return (
    <UpdateNotificationBar
      state={barState}
      onUpdate={handleUpdate}
      onDismiss={handleDismiss}
    />
  )
}
