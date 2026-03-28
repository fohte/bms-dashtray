import { act, render, screen, waitFor } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { PlayHistoryListContainer } from '@/components/PlayHistoryListContainer'
import type { TauriApi } from '@/tauri-api'
import type { PlayRecord, ScoresUpdatedPayload } from '@/types'

function makeRecord(overrides: Partial<PlayRecord> = {}): PlayRecord {
  return {
    id: crypto.randomUUID(),
    sha256: 'abc123',
    mode: 0,
    clear: 5,
    exScore: 1234,
    minBp: 15,
    notes: 800,
    combo: 500,
    playedAt: '2026-03-20T15:30:00+09:00',
    title: 'Test Song',
    subtitle: '',
    artist: 'Test Artist',
    level: 12,
    difficulty: 1,
    tableLevels: [],
    previousClear: 5,
    previousExScore: 1200,
    previousMinBp: 18,
    ...overrides,
  }
}

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

describe('PlayHistoryListContainer', () => {
  it('shows empty state when no records', async () => {
    const api = createMockApi()
    render(<PlayHistoryListContainer api={api} />)

    await waitFor(() => {
      expect(screen.getByText('No plays recorded today')).toBeInTheDocument()
    })
  })

  it('loads and displays initial records from getTodayRecords', async () => {
    const records = [
      makeRecord({ title: 'Song A', playedAt: '2026-03-20T15:00:00+09:00' }),
      makeRecord({ title: 'Song B', playedAt: '2026-03-20T16:00:00+09:00' }),
    ]
    const api = createMockApi({
      getTodayRecords: vi.fn().mockResolvedValue(records),
    })
    render(<PlayHistoryListContainer api={api} />)

    await waitFor(() => {
      expect(screen.getByText('Song A')).toBeInTheDocument()
      expect(screen.getByText('Song B')).toBeInTheDocument()
    })
  })

  it('sorts records by playedAt descending', async () => {
    const records = [
      makeRecord({ title: 'Older', playedAt: '2026-03-20T14:00:00+09:00' }),
      makeRecord({ title: 'Newer', playedAt: '2026-03-20T16:00:00+09:00' }),
    ]
    const api = createMockApi({
      getTodayRecords: vi.fn().mockResolvedValue(records),
    })
    const { container } = render(<PlayHistoryListContainer api={api} />)

    await waitFor(() => {
      expect(screen.getByText('Newer')).toBeInTheDocument()
    })

    const titles = container.querySelectorAll('span')
    const titleTexts = Array.from(titles)
      .map((el) => el.textContent)
      .filter((t) => t === 'Newer' || t === 'Older')
    expect(titleTexts).toEqual(['Newer', 'Older'])
  })

  it('updates records when scores-updated event is received', async () => {
    let eventCallback: ((payload: ScoresUpdatedPayload) => void) | null = null
    const api = createMockApi({
      listenScoresUpdated: vi
        .fn()
        .mockImplementation(
          (callback: (payload: ScoresUpdatedPayload) => void) => {
            eventCallback = callback
            return Promise.resolve(vi.fn())
          },
        ),
    })
    render(<PlayHistoryListContainer api={api} />)

    await waitFor(() => {
      expect(eventCallback).not.toBeNull()
    })

    const newRecords = [
      makeRecord({ title: 'New Song', playedAt: '2026-03-20T17:00:00+09:00' }),
    ]
    act(() => {
      if (eventCallback == null) throw new Error('eventCallback is null')
      eventCallback({
        records: newRecords,
        updatedAt: '2026-03-20T17:00:00+09:00',
      })
    })

    await waitFor(() => {
      expect(screen.getByText('New Song')).toBeInTheDocument()
    })
  })

  it('subscribes to scores-updated event on mount', async () => {
    const api = createMockApi()
    render(<PlayHistoryListContainer api={api} />)

    await waitFor(() => {
      expect(api.listenScoresUpdated).toHaveBeenCalledOnce()
    })
  })
})
