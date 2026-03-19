import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

import type { AppConfig } from '@/types'

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
}

export const tauriApi: TauriApi = {
  getConfig: () => invoke<AppConfig | null>('get_config'),
  validateAndSaveConfig: (beatorajaRoot: string) =>
    invoke<void>('validate_and_save_config', { beatorajaRoot }),
  updateSettings: (settings) => invoke<void>('update_settings', settings),
  resetHistory: () => invoke<void>('reset_history'),
  openFolderDialog: () => open({ directory: true }),
}
