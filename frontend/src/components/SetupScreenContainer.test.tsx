import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'

import { SetupScreenContainer } from '@/components/SetupScreenContainer'
import type { TauriApi } from '@/tauri-api'

function createMockApi(overrides: Partial<TauriApi> = {}): TauriApi {
  return {
    getConfig: vi.fn().mockResolvedValue(null),
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

describe('SetupScreenContainer', () => {
  it('calls openFolderDialog when BROWSE is clicked', async () => {
    const api = createMockApi()
    render(<SetupScreenContainer api={api} onSetupComplete={vi.fn()} />)

    await userEvent.click(screen.getByRole('button', { name: '...' }))
    expect(api.openFolderDialog).toHaveBeenCalledOnce()
  })

  it('auto-selects single player and validates config', async () => {
    const api = createMockApi({
      openFolderDialog: vi.fn().mockResolvedValue('/path/to/beatoraja'),
      detectPlayers: vi.fn().mockResolvedValue(['Player1']),
      validateAndSaveConfig: vi.fn().mockResolvedValue(undefined),
    })
    render(<SetupScreenContainer api={api} onSetupComplete={vi.fn()} />)

    await userEvent.click(screen.getByRole('button', { name: '...' }))

    await waitFor(() => {
      expect(api.detectPlayers).toHaveBeenCalledWith('/path/to/beatoraja')
      expect(api.validateAndSaveConfig).toHaveBeenCalledWith(
        '/path/to/beatoraja',
        'Player1',
      )
    })
  })

  it('shows player selection when multiple players detected', async () => {
    const api = createMockApi({
      openFolderDialog: vi.fn().mockResolvedValue('/path/to/beatoraja'),
      detectPlayers: vi.fn().mockResolvedValue(['Player1', 'Player2']),
    })
    render(<SetupScreenContainer api={api} onSetupComplete={vi.fn()} />)

    await userEvent.click(screen.getByRole('button', { name: '...' }))

    await waitFor(() => {
      expect(screen.getByText('Player1')).toBeInTheDocument()
      expect(screen.getByText('Player2')).toBeInTheDocument()
    })

    // Should not have called validateAndSaveConfig yet
    expect(api.validateAndSaveConfig).not.toHaveBeenCalled()
  })

  it('validates after selecting a player from multiple choices', async () => {
    const api = createMockApi({
      openFolderDialog: vi.fn().mockResolvedValue('/path/to/beatoraja'),
      detectPlayers: vi.fn().mockResolvedValue(['Player1', 'Player2']),
      validateAndSaveConfig: vi.fn().mockResolvedValue(undefined),
    })
    render(<SetupScreenContainer api={api} onSetupComplete={vi.fn()} />)

    await userEvent.click(screen.getByRole('button', { name: '...' }))

    await waitFor(() => {
      expect(screen.getByText('Player1')).toBeInTheDocument()
    })

    await userEvent.click(screen.getByText('Player2'))

    await waitFor(() => {
      expect(api.validateAndSaveConfig).toHaveBeenCalledWith(
        '/path/to/beatoraja',
        'Player2',
      )
    })
  })

  it('shows selected path after folder selection', async () => {
    const api = createMockApi({
      openFolderDialog: vi.fn().mockResolvedValue('/path/to/beatoraja'),
    })
    render(<SetupScreenContainer api={api} onSetupComplete={vi.fn()} />)

    await userEvent.click(screen.getByRole('button', { name: '...' }))

    await waitFor(() => {
      expect(screen.getByText('/path/to/beatoraja')).toBeInTheDocument()
    })
  })

  it('enables START after successful validation', async () => {
    const api = createMockApi({
      openFolderDialog: vi.fn().mockResolvedValue('/path/to/beatoraja'),
      detectPlayers: vi.fn().mockResolvedValue(['default']),
      validateAndSaveConfig: vi.fn().mockResolvedValue(undefined),
    })
    render(<SetupScreenContainer api={api} onSetupComplete={vi.fn()} />)

    await userEvent.click(screen.getByRole('button', { name: '...' }))

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'START' })).toBeEnabled()
    })
  })

  it('shows error when detect_players fails', async () => {
    const api = createMockApi({
      openFolderDialog: vi.fn().mockResolvedValue('/wrong/path'),
      detectPlayers: vi
        .fn()
        .mockRejectedValue(
          new Error('player directory not found under /wrong/path/player/'),
        ),
    })
    render(<SetupScreenContainer api={api} onSetupComplete={vi.fn()} />)

    await userEvent.click(screen.getByRole('button', { name: '...' }))

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'START' })).toBeDisabled()
    })
  })

  it('shows not-found files on validation failure', async () => {
    const api = createMockApi({
      openFolderDialog: vi.fn().mockResolvedValue('/wrong/path'),
      detectPlayers: vi.fn().mockResolvedValue(['default']),
      validateAndSaveConfig: vi
        .fn()
        .mockRejectedValue(new Error('Missing: scoredatalog.db, score.db')),
    })
    render(<SetupScreenContainer api={api} onSetupComplete={vi.fn()} />)

    await userEvent.click(screen.getByRole('button', { name: '...' }))

    await waitFor(() => {
      expect(
        screen.getByText('scoredatalog.db が見つかりません'),
      ).toBeInTheDocument()
      expect(screen.getByText('score.db が見つかりません')).toBeInTheDocument()
    })
  })

  it('calls onSetupComplete when START is clicked', async () => {
    const onSetupComplete = vi.fn()
    const api = createMockApi({
      openFolderDialog: vi.fn().mockResolvedValue('/path/to/beatoraja'),
      detectPlayers: vi.fn().mockResolvedValue(['default']),
      validateAndSaveConfig: vi.fn().mockResolvedValue(undefined),
    })
    render(<SetupScreenContainer api={api} onSetupComplete={onSetupComplete} />)

    await userEvent.click(screen.getByRole('button', { name: '...' }))

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'START' })).toBeEnabled()
    })

    await userEvent.click(screen.getByRole('button', { name: 'START' }))
    expect(onSetupComplete).toHaveBeenCalledOnce()
  })

  it('does nothing when dialog is cancelled', async () => {
    const api = createMockApi({
      openFolderDialog: vi.fn().mockResolvedValue(null),
    })
    render(<SetupScreenContainer api={api} onSetupComplete={vi.fn()} />)

    await userEvent.click(screen.getByRole('button', { name: '...' }))

    expect(api.detectPlayers).not.toHaveBeenCalled()
    expect(api.validateAndSaveConfig).not.toHaveBeenCalled()
    expect(screen.getByText('Select folder...')).toBeInTheDocument()
  })
})
