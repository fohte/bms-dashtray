import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'

import { SetupScreenContainer } from '@/components/SetupScreenContainer'
import type { TauriApi } from '@/tauri-api'

function createMockApi(overrides: Partial<TauriApi> = {}): TauriApi {
  return {
    getConfig: vi.fn().mockResolvedValue(null),
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

  it('validates config after folder selection', async () => {
    const api = createMockApi({
      openFolderDialog: vi.fn().mockResolvedValue('/path/to/beatoraja'),
      validateAndSaveConfig: vi.fn().mockResolvedValue(undefined),
    })
    render(<SetupScreenContainer api={api} onSetupComplete={vi.fn()} />)

    await userEvent.click(screen.getByRole('button', { name: '...' }))

    await waitFor(() => {
      expect(api.validateAndSaveConfig).toHaveBeenCalledWith(
        '/path/to/beatoraja',
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
      validateAndSaveConfig: vi.fn().mockResolvedValue(undefined),
    })
    render(<SetupScreenContainer api={api} onSetupComplete={vi.fn()} />)

    await userEvent.click(screen.getByRole('button', { name: '...' }))

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'START' })).toBeEnabled()
    })
  })

  it('shows error on validation failure', async () => {
    const api = createMockApi({
      openFolderDialog: vi.fn().mockResolvedValue('/wrong/path'),
      validateAndSaveConfig: vi
        .fn()
        .mockRejectedValue(new Error('Missing: scoredatalog.db, score.db')),
    })
    render(<SetupScreenContainer api={api} onSetupComplete={vi.fn()} />)

    await userEvent.click(screen.getByRole('button', { name: '...' }))

    await waitFor(() => {
      expect(
        screen.getByText('Missing: scoredatalog.db, score.db'),
      ).toBeInTheDocument()
    })
  })

  it('calls onSetupComplete when START is clicked', async () => {
    const onSetupComplete = vi.fn()
    const api = createMockApi({
      openFolderDialog: vi.fn().mockResolvedValue('/path/to/beatoraja'),
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

    expect(api.validateAndSaveConfig).not.toHaveBeenCalled()
    expect(screen.getByText('Select folder...')).toBeInTheDocument()
  })
})
