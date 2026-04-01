import { relaunch } from '@tauri-apps/plugin-process'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { useCallback, useState } from 'react'

export type UpdateCheckState =
  | { status: 'idle' }
  | { status: 'checking' }
  | { status: 'up-to-date' }
  | { status: 'available'; version: string }
  | { status: 'downloading'; progress: number }
  | { status: 'error'; message: string }

export function useUpdateChecker() {
  const [state, setState] = useState<UpdateCheckState>({ status: 'idle' })
  const [pendingUpdate, setPendingUpdate] = useState<Update | null>(null)

  const checkForUpdates = useCallback(() => {
    setState({ status: 'checking' })

    const doCheck = async () => {
      try {
        const update = await check()
        if (update) {
          setPendingUpdate(update)
          setState({ status: 'available', version: update.version })
        } else {
          setState({ status: 'up-to-date' })
        }
      } catch (e) {
        setState({
          status: 'error',
          message: e instanceof Error ? e.message : String(e),
        })
      }
    }

    void doCheck()
  }, [])

  const installUpdate = useCallback(() => {
    if (pendingUpdate == null) return

    const doUpdate = async () => {
      try {
        let totalLength = 0
        let downloadedLength = 0

        setState({ status: 'downloading', progress: 0 })

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

    void doUpdate()
  }, [pendingUpdate])

  const dismiss = useCallback(() => {
    setState({ status: 'idle' })
  }, [])

  return { state, checkForUpdates, installUpdate, dismiss }
}
