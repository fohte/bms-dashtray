import { relaunch } from '@tauri-apps/plugin-process'
import { check, type Update } from '@tauri-apps/plugin-updater'
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from 'react'

export type UpdateCheckState =
  | { status: 'idle' }
  | { status: 'checking' }
  | { status: 'up-to-date' }
  | { status: 'available'; version: string }
  | { status: 'downloading'; progress: number }
  | { status: 'error'; message: string }

interface UpdateCheckerValue {
  state: UpdateCheckState
  checkForUpdates: () => void
  installUpdate: () => void
  dismiss: () => void
}

const UpdateCheckerContext = createContext<UpdateCheckerValue | null>(null)

export const UpdateCheckerProvider = UpdateCheckerContext.Provider

export function useUpdateCheckerValue(): UpdateCheckerValue {
  const [state, setState] = useState<UpdateCheckState>({ status: 'idle' })
  const [pendingUpdate, setPendingUpdate] = useState<Update | null>(null)
  const isMountedRef = useRef(true)
  const isProcessingRef = useRef(false)

  useEffect(() => {
    return () => {
      isMountedRef.current = false
    }
  }, [])

  const checkForUpdates = useCallback(() => {
    if (isProcessingRef.current) return
    isProcessingRef.current = true

    setPendingUpdate(null)
    setState({ status: 'checking' })

    const doCheck = async () => {
      try {
        const update = await check()
        if (!isMountedRef.current) return
        if (update) {
          setPendingUpdate(update)
          setState({ status: 'available', version: update.version })
        } else {
          setState({ status: 'up-to-date' })
        }
      } catch (e) {
        if (isMountedRef.current) {
          setState({
            status: 'error',
            message: e instanceof Error ? e.message : String(e),
          })
        }
      } finally {
        isProcessingRef.current = false
      }
    }

    void doCheck()
  }, [])

  const installUpdate = useCallback(() => {
    if (pendingUpdate == null || isProcessingRef.current) return
    isProcessingRef.current = true

    const doUpdate = async () => {
      try {
        let totalLength = 0
        let downloadedLength = 0

        setState({ status: 'downloading', progress: 0 })

        await pendingUpdate.downloadAndInstall((progress) => {
          if (!isMountedRef.current) return
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

        if (isMountedRef.current) {
          await relaunch()
        }
      } catch (e) {
        if (isMountedRef.current) {
          setState({
            status: 'error',
            message: e instanceof Error ? e.message : String(e),
          })
        }
      } finally {
        isProcessingRef.current = false
      }
    }

    void doUpdate()
  }, [pendingUpdate])

  const dismiss = useCallback(() => {
    setPendingUpdate(null)
    setState({ status: 'idle' })
  }, [])

  return { state, checkForUpdates, installUpdate, dismiss }
}

export function useUpdateChecker(): UpdateCheckerValue {
  const value = useContext(UpdateCheckerContext)
  if (value == null) {
    throw new Error(
      'useUpdateChecker must be used within an UpdateCheckerProvider',
    )
  }
  return value
}
