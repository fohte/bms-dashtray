import type { Meta, StoryObj } from '@storybook/react-vite'

import { MainScreen } from '@/components/MainScreen'
import { PlayHistoryPanel } from '@/components/PlayHistoryPanel'
import { makeRecord } from '@/test-helpers'
import type { PlayRecord } from '@/types'

const meta = {
  title: 'Components/MainScreen',
  component: MainScreen,
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof MainScreen>

export default meta
type Story = StoryObj<typeof meta>

const noop = () => {}

const sampleRecords: PlayRecord[] = [
  makeRecord({
    title: 'FREEDOM DiVE',
    level: 12,
    clear: 6,
    exScore: 1900,
    minBp: 15,
    previousClear: 5,
    previousExScore: 1790,
    previousMinBp: 28,
    tableLevels: ['★24'],
  }),
  makeRecord({
    title: 'Groundbreaking',
    level: 11,
    clear: 7,
    exScore: 2200,
    minBp: 3,
    previousClear: 6,
    previousExScore: 2100,
    previousMinBp: 8,
    tableLevels: ['st3', 'sl5'],
  }),
  makeRecord({
    title: 'L9',
    level: 12,
    clear: 4,
    exScore: 1456,
    minBp: 45,
    previousClear: 4,
    previousExScore: 1500,
    previousMinBp: 40,
    tableLevels: ['★24'],
  }),
]

export const Default: Story = {
  args: {
    todayDate: '2026-03-28',
    backgroundTransparent: false,
    onOpenSettings: noop,
    onResetHistory: noop,
    children: (
      <PlayHistoryPanel
        records={sampleRecords}
        filteredRecords={sampleRecords}
        activeFilter="all"
        onFilterChange={noop}
      />
    ),
  },
}

export const BackgroundTransparent: Story = {
  args: {
    ...Default.args,
    backgroundTransparent: true,
  },
}

export const Empty: Story = {
  args: {
    todayDate: '2026-03-28',
    backgroundTransparent: false,
    onOpenSettings: noop,
    onResetHistory: noop,
    children: (
      <PlayHistoryPanel
        records={[]}
        filteredRecords={[]}
        activeFilter="all"
        onFilterChange={noop}
      />
    ),
  },
}
