import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'

import type { AppConfig, PlayRecord, ScoresUpdatedPayload } from '@/types'

export interface TauriApi {
  getConfig: () => Promise<AppConfig | null>
  validateAndSaveConfig: (beatorajaRoot: string) => Promise<void>
  updateSettings: (settings: {
    resetTime?: string
    backgroundTransparent?: boolean
    fontSize?: number
  }) => Promise<void>
  resetHistory: () => Promise<void>
  openFolderDialog: () => Promise<string | null>
  getTodayRecords: () => Promise<PlayRecord[]>
  listenScoresUpdated: (
    callback: (payload: ScoresUpdatedPayload) => void,
  ) => Promise<UnlistenFn>
}

export const tauriApi: TauriApi = {
  getConfig: () => invoke<AppConfig | null>('get_config'),
  validateAndSaveConfig: (beatorajaRoot: string) =>
    invoke<void>('validate_and_save_config', { beatorajaRoot }),
  updateSettings: (settings) => invoke<void>('update_settings', settings),
  resetHistory: () => invoke<void>('reset_history'),
  openFolderDialog: () => open({ directory: true }),
  getTodayRecords: () => invoke<PlayRecord[]>('get_today_records'),
  listenScoresUpdated: (callback) =>
    listen<ScoresUpdatedPayload>('scores-updated', (event) => {
      callback(event.payload)
    }),
}
