import { relaunch } from '@tauri-apps/plugin-process'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { ResultAsync } from 'neverthrow'
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from 'react'

function toMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e)
}

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
      const result = await ResultAsync.fromPromise(check(), toMessage)

      if (isMountedRef.current) {
        if (result.isErr()) {
          setState({ status: 'error', message: result.error })
        } else if (result.value) {
          setPendingUpdate(result.value)
          setState({ status: 'available', version: result.value.version })
        } else {
          setState({ status: 'up-to-date' })
        }
      }

      isProcessingRef.current = false
    }

    void doCheck()
  }, [])

  const installUpdate = useCallback(() => {
    if (pendingUpdate == null || isProcessingRef.current) return
    isProcessingRef.current = true

    const doUpdate = async () => {
      let totalLength = 0
      let downloadedLength = 0

      setState({ status: 'downloading', progress: 0 })

      const downloadResult = await ResultAsync.fromPromise(
        pendingUpdate.downloadAndInstall((progress) => {
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
        }),
        toMessage,
      )

      const result =
        downloadResult.isOk() && isMountedRef.current
          ? await ResultAsync.fromPromise(relaunch(), toMessage)
          : downloadResult

      if (isMountedRef.current && result.isErr()) {
        setState({ status: 'error', message: result.error })
      }

      isProcessingRef.current = false
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
    // eslint-disable-next-line no-restricted-syntax -- React context misuse is a programmer error; the hook's synchronous contract has no Result-consuming caller
    throw new Error(
      'useUpdateChecker must be used within an UpdateCheckerProvider',
    )
  }
  return value
}
