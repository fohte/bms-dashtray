import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import {
  aggregateLevelDistribution,
  DistributionChart,
} from '@/components/DistributionChart'
import type { PlayRecord } from '@/types'

function makeRecord(overrides: Partial<PlayRecord> = {}): PlayRecord {
  return {
    id: 'test-id',
    sha256: 'abc123',
    mode: 0,
    clear: 4,
    exScore: 1200,
    minBp: 30,
    notes: 1000,
    combo: 500,
    playedAt: '2026-03-20T12:00:00+09:00',
    title: 'Test Song',
    artist: 'Test Artist',
    level: 10,
    difficulty: 3,
    previousClear: null,
    previousExScore: null,
    previousMinBp: null,
    ...overrides,
  }
}

describe('aggregateLevelDistribution', () => {
  it('returns empty array for no records', () => {
    expect(aggregateLevelDistribution([])).toEqual([])
  })

  it('counts records by level and sorts ascending', () => {
    const records = [
      makeRecord({ level: 12 }),
      makeRecord({ level: 10 }),
      makeRecord({ level: 10 }),
      makeRecord({ level: 10 }),
      makeRecord({ level: 12 }),
    ]
    const result = aggregateLevelDistribution(records)
    expect(result).toEqual([
      { level: 10, count: 3, percentage: 60 },
      { level: 12, count: 2, percentage: 40 },
    ])
  })

  it('rounds percentage to nearest integer', () => {
    const records = [
      makeRecord({ level: 1 }),
      makeRecord({ level: 2 }),
      makeRecord({ level: 3 }),
    ]
    const result = aggregateLevelDistribution(records)
    expect(result).toEqual([
      { level: 1, count: 1, percentage: 33 },
      { level: 2, count: 1, percentage: 33 },
      { level: 3, count: 1, percentage: 33 },
    ])
  })
})

describe('DistributionChart', () => {
  it('renders "No play data" when records are empty', () => {
    render(<DistributionChart records={[]} />)
    expect(screen.getByText('No play data')).toBeInTheDocument()
  })

  it('renders level labels and counts', () => {
    const records = [
      makeRecord({ level: 10 }),
      makeRecord({ level: 10 }),
      makeRecord({ level: 11 }),
    ]
    render(<DistributionChart records={records} />)
    expect(screen.getByText('Lv.10')).toBeInTheDocument()
    expect(screen.getByText('Lv.11')).toBeInTheDocument()
    expect(screen.getByText('2')).toBeInTheDocument()
    expect(screen.getByText('1')).toBeInTheDocument()
  })

  it('renders the section header', () => {
    render(<DistributionChart records={[]} />)
    expect(screen.getByText('DIFFICULTY DISTRIBUTION')).toBeInTheDocument()
  })

  it('renders bar elements with correct aria attributes', () => {
    const records = [
      makeRecord({ level: 10 }),
      makeRecord({ level: 10 }),
      makeRecord({ level: 11 }),
    ]
    render(<DistributionChart records={records} />)
    const meters = screen.getAllByRole('meter')
    expect(meters).toHaveLength(2)

    const lv10Meter = screen.getByRole('meter', { name: 'Lv.10' })
    expect(lv10Meter).toHaveAttribute('aria-valuenow', '2')
    expect(lv10Meter).toHaveAttribute('aria-valuemax', '2')

    const lv11Meter = screen.getByRole('meter', { name: 'Lv.11' })
    expect(lv11Meter).toHaveAttribute('aria-valuenow', '1')
    expect(lv11Meter).toHaveAttribute('aria-valuemax', '2')
  })
})
