import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'

import { SettingsScreenContainer } from '@/components/SettingsScreenContainer'
import type { TauriApi } from '@/tauri-api'
import type { AppConfig } from '@/types'

vi.mock('@tauri-apps/api/app', () => ({
  getVersion: vi.fn().mockResolvedValue('0.1.2'),
}))

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: vi.fn().mockResolvedValue(null),
}))

vi.mock('@tauri-apps/plugin-process', () => ({
  relaunch: vi.fn().mockResolvedValue(undefined),
}))

const defaultConfig: AppConfig = {
  beatorajaRoot: '/path/to/beatoraja',
  playerName: 'default',
  resetTime: '05:00',
  backgroundTransparent: false,
  fontSize: 13,
}

function createMockApi(overrides: Partial<TauriApi> = {}): TauriApi {
  return {
    getConfig: vi.fn().mockResolvedValue(defaultConfig),
    detectPlayers: vi.fn().mockResolvedValue(['default']),
    validateAndSaveConfig: vi.fn().mockResolvedValue(undefined),
    updateSettings: vi.fn().mockResolvedValue(undefined),
    resetHistory: vi.fn().mockResolvedValue(undefined),
    openFolderDialog: vi.fn().mockResolvedValue(null),
    getTodayRecords: vi.fn().mockResolvedValue([]),
    listenScoresUpdated: vi.fn().mockResolvedValue(vi.fn()),
    ...overrides,
  }
}

describe('SettingsScreenContainer', () => {
  it('renders settings with config values', () => {
    const api = createMockApi()
    render(
      <SettingsScreenContainer
        api={api}
        config={defaultConfig}
        onBack={vi.fn()}
        onConfigChanged={vi.fn()}
      />,
    )

    expect(screen.getByText('/path/to/beatoraja')).toBeInTheDocument()
    expect(screen.getByText('default')).toBeInTheDocument()
    expect(screen.getByText('13px')).toBeInTheDocument()
  })

  it('calls onBack when back button is clicked', async () => {
    const onBack = vi.fn()
    const api = createMockApi()
    render(
      <SettingsScreenContainer
        api={api}
        config={defaultConfig}
        onBack={onBack}
        onConfigChanged={vi.fn()}
      />,
    )

    await userEvent.click(screen.getByText('< BACK'))
    expect(onBack).toHaveBeenCalledOnce()
  })

  it('toggles background transparent and calls updateSettings', async () => {
    const api = createMockApi()
    render(
      <SettingsScreenContainer
        api={api}
        config={defaultConfig}
        onBack={vi.fn()}
        onConfigChanged={vi.fn()}
      />,
    )

    await userEvent.click(
      screen.getByRole('switch', { name: 'Toggle background transparent' }),
    )

    await waitFor(() => {
      expect(api.updateSettings).toHaveBeenCalledWith({
        backgroundTransparent: true,
      })
    })
  })

  it('increases font size and calls updateSettings', async () => {
    const api = createMockApi()
    render(
      <SettingsScreenContainer
        api={api}
        config={defaultConfig}
        onBack={vi.fn()}
        onConfigChanged={vi.fn()}
      />,
    )

    await userEvent.click(
      screen.getByRole('button', { name: 'Increase font size' }),
    )

    await waitFor(() => {
      expect(api.updateSettings).toHaveBeenCalledWith({ fontSize: 14 })
    })
    expect(screen.getByText('14px')).toBeInTheDocument()
  })

  it('decreases font size and calls updateSettings', async () => {
    const api = createMockApi()
    render(
      <SettingsScreenContainer
        api={api}
        config={defaultConfig}
        onBack={vi.fn()}
        onConfigChanged={vi.fn()}
      />,
    )

    await userEvent.click(
      screen.getByRole('button', { name: 'Decrease font size' }),
    )

    await waitFor(() => {
      expect(api.updateSettings).toHaveBeenCalledWith({ fontSize: 12 })
    })
    expect(screen.getByText('12px')).toBeInTheDocument()
  })

  it('does not decrease font size below minimum', async () => {
    const api = createMockApi()
    render(
      <SettingsScreenContainer
        api={api}
        config={{ ...defaultConfig, fontSize: 8 }}
        onBack={vi.fn()}
        onConfigChanged={vi.fn()}
      />,
    )

    await userEvent.click(
      screen.getByRole('button', { name: 'Decrease font size' }),
    )

    expect(api.updateSettings).not.toHaveBeenCalled()
    expect(screen.getByText('8px')).toBeInTheDocument()
  })

  it('shows confirm dialog and resets history on confirmation', async () => {
    const api = createMockApi()
    render(
      <SettingsScreenContainer
        api={api}
        config={defaultConfig}
        onBack={vi.fn()}
        onConfigChanged={vi.fn()}
      />,
    )

    await userEvent.click(screen.getByText('RESET HISTORY NOW'))
    expect(screen.getByText('Reset History')).toBeInTheDocument()

    await userEvent.click(screen.getByText('RESET'))

    await waitFor(() => {
      expect(api.resetHistory).toHaveBeenCalledOnce()
    })
  })

  it('cancels reset history dialog', async () => {
    const api = createMockApi()
    render(
      <SettingsScreenContainer
        api={api}
        config={defaultConfig}
        onBack={vi.fn()}
        onConfigChanged={vi.fn()}
      />,
    )

    await userEvent.click(screen.getByText('RESET HISTORY NOW'))
    expect(screen.getByText('Reset History')).toBeInTheDocument()

    await userEvent.click(screen.getByText('CANCEL'))

    expect(screen.queryByText('Reset History')).not.toBeInTheDocument()
    expect(api.resetHistory).not.toHaveBeenCalled()
  })

  it('changes beatoraja root via folder dialog', async () => {
    const updatedConfig = {
      ...defaultConfig,
      beatorajaRoot: '/new/path',
      playerName: 'newplayer',
    }
    const api = createMockApi({
      openFolderDialog: vi.fn().mockResolvedValue('/new/path'),
      validateAndSaveConfig: vi.fn().mockResolvedValue(undefined),
      getConfig: vi.fn().mockResolvedValue(updatedConfig),
    })
    const onConfigChanged = vi.fn()
    render(
      <SettingsScreenContainer
        api={api}
        config={defaultConfig}
        onBack={vi.fn()}
        onConfigChanged={onConfigChanged}
      />,
    )

    await userEvent.click(screen.getByRole('button', { name: '...' }))

    await waitFor(() => {
      expect(api.validateAndSaveConfig).toHaveBeenCalledWith(
        '/new/path',
        'default',
      )
    })

    await waitFor(() => {
      expect(screen.getByText('/new/path')).toBeInTheDocument()
    })
  })

  it('notifies parent when config changes', async () => {
    const api = createMockApi()
    const onConfigChanged = vi.fn()
    render(
      <SettingsScreenContainer
        api={api}
        config={defaultConfig}
        onBack={vi.fn()}
        onConfigChanged={onConfigChanged}
      />,
    )

    await userEvent.click(
      screen.getByRole('switch', { name: 'Toggle background transparent' }),
    )

    await waitFor(() => {
      expect(onConfigChanged).toHaveBeenCalledWith(
        expect.objectContaining({ backgroundTransparent: true }),
      )
    })
  })
})
