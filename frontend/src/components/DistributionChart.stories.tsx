import type { Meta, StoryObj } from '@storybook/react-vite'

import { DistributionChart } from '@/components/DistributionChart'
import type { PlayRecord } from '@/types'

function makeRecord(overrides: Partial<PlayRecord> = {}): PlayRecord {
  return {
    id: crypto.randomUUID(),
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
    previousClear: null,
    previousExScore: null,
    previousMinBp: null,
    ...overrides,
  }
}

const meta = {
  title: 'Components/DistributionChart',
  component: DistributionChart,
} satisfies Meta<typeof DistributionChart>

export default meta
type Story = StoryObj<typeof meta>

export const FewLevels: Story = {
  args: {
    records: [
      makeRecord({ level: 10 }),
      makeRecord({ level: 10 }),
      makeRecord({ level: 10 }),
      makeRecord({ level: 11 }),
      makeRecord({ level: 11 }),
      makeRecord({ level: 12 }),
    ],
  },
}

export const ManyLevels: Story = {
  args: {
    records: [
      makeRecord({ level: 1 }),
      makeRecord({ level: 2 }),
      makeRecord({ level: 2 }),
      makeRecord({ level: 3 }),
      makeRecord({ level: 3 }),
      makeRecord({ level: 3 }),
      makeRecord({ level: 5 }),
      makeRecord({ level: 5 }),
      makeRecord({ level: 7 }),
      makeRecord({ level: 7 }),
      makeRecord({ level: 7 }),
      makeRecord({ level: 7 }),
      makeRecord({ level: 8 }),
      makeRecord({ level: 8 }),
      makeRecord({ level: 9 }),
      makeRecord({ level: 9 }),
      makeRecord({ level: 9 }),
      makeRecord({ level: 10 }),
      makeRecord({ level: 10 }),
      makeRecord({ level: 10 }),
      makeRecord({ level: 10 }),
      makeRecord({ level: 10 }),
      makeRecord({ level: 11 }),
      makeRecord({ level: 11 }),
      makeRecord({ level: 11 }),
      makeRecord({ level: 11 }),
      makeRecord({ level: 12 }),
      makeRecord({ level: 12 }),
      makeRecord({ level: 12 }),
    ],
  },
}

export const Empty: Story = {
  args: {
    records: [],
  },
}
