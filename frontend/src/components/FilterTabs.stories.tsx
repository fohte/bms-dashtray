import type { Meta, StoryObj } from '@storybook/react-vite'
import { useState } from 'react'

import { filterRecords } from '@/components/filterRecords'
import { FilterTabs } from '@/components/FilterTabs'
import type { FilterType, PlayRecord } from '@/types'

const meta = {
  title: 'Components/FilterTabs',
  component: FilterTabs,
} satisfies Meta<typeof FilterTabs>

export default meta
type Story = StoryObj<typeof meta>

export const AllActive: Story = {
  args: {
    activeFilter: 'all',
    onFilterChange: () => {},
  },
}

export const UpdatedActive: Story = {
  args: {
    activeFilter: 'updated',
    onFilterChange: () => {},
  },
}

const sampleRecords: PlayRecord[] = [
  {
    id: '1',
    sha256: 'aaa',
    mode: 0,
    clear: 5,
    exScore: 1200,
    minBp: 30,
    notes: 600,
    combo: 400,
    playedAt: '2026-03-20T14:00:00',
    title: 'FREEDOM DiVE',
    artist: 'xi',
    level: 12,
    difficulty: 3,
    previousClear: 3,
    previousExScore: 1000,
    previousMinBp: 50,
  },
  {
    id: '2',
    sha256: 'bbb',
    mode: 0,
    clear: 3,
    exScore: 800,
    minBp: 60,
    notes: 400,
    combo: 200,
    playedAt: '2026-03-20T13:30:00',
    title: 'Aleph-0',
    artist: 'LeaF',
    level: 10,
    difficulty: 2,
    previousClear: 3,
    previousExScore: 800,
    previousMinBp: 60,
  },
  {
    id: '3',
    sha256: 'ccc',
    mode: 0,
    clear: 4,
    exScore: 950,
    minBp: 45,
    notes: 500,
    combo: 350,
    playedAt: '2026-03-20T13:00:00',
    title: 'Ascension to Heaven',
    artist: 'xi',
    level: 11,
    difficulty: 3,
    previousClear: null,
    previousExScore: null,
    previousMinBp: null,
  },
]

export const Interactive: StoryObj = {
  render() {
    const [filter, setFilter] = useState<FilterType>('all')
    const filtered = filterRecords(sampleRecords, filter)

    return (
      <div
        style={{
          padding: '16px',
          fontFamily: "'JetBrains Mono', monospace",
          color: '#ffffff',
        }}
      >
        <FilterTabs activeFilter={filter} onFilterChange={setFilter} />
        <div style={{ marginTop: '16px', fontSize: '12px', color: '#888888' }}>
          {filtered.length} / {sampleRecords.length} records
        </div>
        <div style={{ marginTop: '8px', fontSize: '12px' }}>
          {filtered.map((r) => (
            <div key={r.id} style={{ padding: '4px 0' }}>
              {r.title} (Lv.{r.level})
            </div>
          ))}
        </div>
      </div>
    )
  },
}
