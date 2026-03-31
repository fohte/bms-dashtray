import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import {
  PlayHistoryPanel,
  type PlayHistoryPanelProps,
} from '@/components/PlayHistoryPanel'
import { makeRecord } from '@/test-helpers'
import type { PlayRecord } from '@/types'

const defaultProps: PlayHistoryPanelProps = {
  records: [],
  filteredRecords: [],
  activeFilter: 'all',
  onFilterChange: vi.fn(),
}

function renderPanel(overrides: Partial<PlayHistoryPanelProps> = {}) {
  return render(<PlayHistoryPanel {...defaultProps} {...overrides} />)
}

describe('PlayHistoryPanel', () => {
  it('renders distribution chart based on filteredRecords, not all records', () => {
    const allRecords = [
      makeRecord({ tableLevels: ['★1'] }),
      makeRecord({ tableLevels: ['★2'] }),
      makeRecord({ tableLevels: ['★3'] }),
    ]
    const filtered = [makeRecord({ tableLevels: ['★2'] })]

    renderPanel({
      records: allRecords,
      filteredRecords: filtered,
      activeFilter: 'updated',
    })

    // The distribution chart should only show the filtered record's level
    const meters = screen.getAllByRole('meter')
    expect(meters).toHaveLength(1)
    expect(meters[0]).toHaveAttribute('aria-label', '★2')
  })

  it.each([
    {
      name: 'shows total plays count when filter is "all"',
      activeFilter: 'all' as const,
      filteredRecords: undefined as PlayRecord[] | undefined,
      expected: '3 plays',
    },
    {
      name: 'shows filtered updates count when filter is "updated"',
      activeFilter: 'updated' as const,
      filteredRecords: [makeRecord()],
      expected: '1 update',
    },
  ])('$name', ({ activeFilter, filteredRecords, expected }) => {
    const allRecords = [makeRecord(), makeRecord(), makeRecord()]

    renderPanel({
      records: allRecords,
      filteredRecords: filteredRecords ?? allRecords,
      activeFilter,
    })

    expect(screen.getByText(expected)).toBeInTheDocument()
  })
})
