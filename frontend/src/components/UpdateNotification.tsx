import { useEffect } from 'react'

import {
  UpdateNotificationBar,
  type UpdateNotificationState,
} from '@/components/UpdateNotificationBar'
import {
  useUpdateChecker,
  type UpdateCheckState,
} from '@/hooks/useUpdateChecker'

function toBarState(state: UpdateCheckState): UpdateNotificationState | null {
  switch (state.status) {
    case 'idle':
    case 'checking':
    case 'up-to-date':
      return null
    case 'available':
      return { status: 'available', version: state.version }
    case 'downloading':
      return { status: 'downloading', progress: state.progress }
    case 'error':
      return { status: 'error', message: state.message }
  }
}

export const UpdateNotification = () => {
  const { state, checkForUpdates, installUpdate, dismiss } = useUpdateChecker()

  useEffect(() => {
    checkForUpdates()
  }, [checkForUpdates])

  const barState = toBarState(state)
  if (!barState) return null

  return (
    <UpdateNotificationBar
      state={barState}
      onUpdate={installUpdate}
      onDismiss={dismiss}
    />
  )
}
