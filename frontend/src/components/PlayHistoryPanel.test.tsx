import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import {
  PlayHistoryPanel,
  type PlayHistoryPanelProps,
} from '@/components/PlayHistoryPanel'
import { makeRecord } from '@/test-helpers'

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
})
