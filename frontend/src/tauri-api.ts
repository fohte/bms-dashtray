import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'

import type { AppConfig, PlayRecord, ScoresUpdatedPayload } from '@/types'

export interface TauriApi {
  getConfig: () => Promise<AppConfig | null>
  detectPlayers: (beatorajaRoot: string) => Promise<string[]>
  validateAndSaveConfig: (
    beatorajaRoot: string,
    playerName: string,
  ) => Promise<undefined>
  updateSettings: (settings: {
    resetTime?: string
    backgroundTransparent?: boolean
    fontSize?: number
  }) => Promise<undefined>
  resetHistory: () => Promise<undefined>
  openFolderDialog: () => Promise<string | null>
  getTodayRecords: () => Promise<PlayRecord[]>
  listenScoresUpdated: (
    callback: (payload: ScoresUpdatedPayload) => void,
  ) => Promise<UnlistenFn>
}

export const tauriApi: TauriApi = {
  getConfig: () => invoke<AppConfig | null>('get_config'),
  detectPlayers: (beatorajaRoot: string) =>
    invoke<string[]>('detect_players', { beatorajaRoot }),
  validateAndSaveConfig: (beatorajaRoot: string, playerName: string) =>
    invoke<undefined>('validate_and_save_config', {
      beatorajaRoot,
      playerName,
    }),
  updateSettings: (settings) => invoke<undefined>('update_settings', settings),
  resetHistory: () => invoke<undefined>('reset_history'),
  openFolderDialog: () => open({ directory: true }),
  getTodayRecords: () => invoke<PlayRecord[]>('get_today_records'),
  listenScoresUpdated: (callback) =>
    listen<ScoresUpdatedPayload>('scores-updated', (event) => {
      callback(event.payload)
    }),
}
