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
    subtitle: '',
    artist: 'Test Artist',
    level: 10,
    difficulty: 3,
    tableLevels: [],
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

  it('returns empty array when no records have tableLevels', () => {
    const records = [makeRecord(), makeRecord()]
    expect(aggregateLevelDistribution(records)).toEqual([])
  })

  it('counts records by table level label and sorts alphabetically', () => {
    const records = [
      makeRecord({ tableLevels: ['★24'] }),
      makeRecord({ tableLevels: ['st3'] }),
      makeRecord({ tableLevels: ['st3'] }),
      makeRecord({ tableLevels: ['★24'] }),
      makeRecord({ tableLevels: ['★24'] }),
    ]
    const result = aggregateLevelDistribution(records)
    expect(result).toEqual([
      { label: '★24', count: 3, percentage: 60 },
      { label: 'st3', count: 2, percentage: 40 },
    ])
  })

  it('counts each table level separately for multi-table records', () => {
    const records = [
      makeRecord({ tableLevels: ['st3', 'sl5'] }),
      makeRecord({ tableLevels: ['st3'] }),
    ]
    const result = aggregateLevelDistribution(records)
    expect(result).toEqual([
      { label: 'sl5', count: 1, percentage: 33 },
      { label: 'st3', count: 2, percentage: 67 },
    ])
  })

  it('excludes records without tableLevels from aggregation', () => {
    const records = [
      makeRecord({ tableLevels: ['★12'] }),
      makeRecord({ tableLevels: [] }),
    ]
    const result = aggregateLevelDistribution(records)
    expect(result).toEqual([{ label: '★12', count: 1, percentage: 100 }])
  })
})

describe('DistributionChart', () => {
  it('renders "No table level data" when no records have tableLevels', () => {
    render(<DistributionChart records={[makeRecord()]} />)
    expect(screen.getByText('No table level data')).toBeInTheDocument()
  })

  it('renders table level labels and counts', () => {
    const records = [
      makeRecord({ tableLevels: ['st3'] }),
      makeRecord({ tableLevels: ['st3'] }),
      makeRecord({ tableLevels: ['★24'] }),
    ]
    render(<DistributionChart records={records} />)
    expect(screen.getByText('st3')).toBeInTheDocument()
    expect(screen.getByText('★24')).toBeInTheDocument()
    expect(screen.getByText('2')).toBeInTheDocument()
    expect(screen.getByText('1')).toBeInTheDocument()
  })

  it('renders the section header', () => {
    render(<DistributionChart records={[]} />)
    expect(screen.getByText('TABLE LEVEL DISTRIBUTION')).toBeInTheDocument()
  })

  it('renders bar elements with correct aria attributes', () => {
    const records = [
      makeRecord({ tableLevels: ['st3'] }),
      makeRecord({ tableLevels: ['st3'] }),
      makeRecord({ tableLevels: ['★24'] }),
    ]
    render(<DistributionChart records={records} />)
    const meters = screen.getAllByRole('meter')
    expect(meters).toHaveLength(2)

    const st3Meter = screen.getByRole('meter', { name: 'st3' })
    expect(st3Meter).toHaveAttribute('aria-valuenow', '2')
    expect(st3Meter).toHaveAttribute('aria-valuemax', '2')

    const starMeter = screen.getByRole('meter', { name: '★24' })
    expect(starMeter).toHaveAttribute('aria-valuenow', '1')
    expect(starMeter).toHaveAttribute('aria-valuemax', '2')
  })
})
