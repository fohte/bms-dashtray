import type { Meta, StoryObj } from '@storybook/react-vite'

import { PlayHistoryList } from '@/components/PlayHistoryList'
import type { PlayRecord } from '@/types'

const meta = {
  title: 'Components/PlayHistoryList',
  component: PlayHistoryList,
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof PlayHistoryList>

export default meta
type Story = StoryObj<typeof meta>

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
    title: 'Example Song',
    subtitle: '',
    artist: 'Example Artist',
    level: 12,
    difficulty: 1,
    tableLevels: [],
    previousClear: 5,
    previousExScore: 1200,
    previousMinBp: 18,
    ...overrides,
  }
}

export const Empty: Story = {
  args: {
    records: [],
  },
}

export const NormalPlay: Story = {
  args: {
    records: [
      makeRecord({
        title: 'FREEDOM DiVE',
        level: 12,
        clear: 5,
        exScore: 1834,
        minBp: 23,
        previousClear: 5,
        previousExScore: 1790,
        previousMinBp: 28,
      }),
      makeRecord({
        title: 'Groundbreaking',
        level: 11,
        clear: 6,
        exScore: 2156,
        minBp: 8,
        previousClear: 6,
        previousExScore: 2100,
        previousMinBp: 12,
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
      }),
    ],
  },
}

export const ClearLampUpdate: Story = {
  args: {
    records: [
      makeRecord({
        title: 'FREEDOM DiVE',
        level: 12,
        clear: 6,
        exScore: 1900,
        minBp: 15,
        previousClear: 5,
        previousExScore: 1790,
        previousMinBp: 28,
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
      }),
    ],
  },
}

export const ScoreDifferences: Story = {
  args: {
    records: [
      makeRecord({
        title: 'Score Improved',
        level: 10,
        clear: 5,
        exScore: 1500,
        minBp: 10,
        previousClear: 5,
        previousExScore: 1400,
        previousMinBp: 15,
      }),
      makeRecord({
        title: 'Score Decreased',
        level: 10,
        clear: 5,
        exScore: 1300,
        minBp: 25,
        previousClear: 5,
        previousExScore: 1400,
        previousMinBp: 15,
      }),
      makeRecord({
        title: 'First Play (No Previous)',
        level: 8,
        clear: 4,
        exScore: 900,
        minBp: 30,
        previousClear: null,
        previousExScore: null,
        previousMinBp: null,
      }),
    ],
  },
}

export const AllClearLamps: Story = {
  args: {
    records: [
      makeRecord({
        title: 'Max Clear',
        level: 5,
        clear: 10,
        previousClear: 10,
      }),
      makeRecord({
        title: 'Perfect Clear',
        level: 6,
        clear: 9,
        previousClear: 9,
      }),
      makeRecord({
        title: 'FullCombo Clear',
        level: 7,
        clear: 8,
        previousClear: 8,
      }),
      makeRecord({
        title: 'ExHard Clear',
        level: 10,
        clear: 7,
        previousClear: 7,
      }),
      makeRecord({
        title: 'Hard Clear',
        level: 11,
        clear: 6,
        previousClear: 6,
      }),
      makeRecord({
        title: 'Normal Clear',
        level: 12,
        clear: 5,
        previousClear: 5,
      }),
      makeRecord({
        title: 'Easy Clear',
        level: 9,
        clear: 4,
        previousClear: 4,
      }),
      makeRecord({
        title: 'LightAssistEasy Clear',
        level: 8,
        clear: 3,
        previousClear: 3,
      }),
      makeRecord({
        title: 'AssistEasy Clear',
        level: 7,
        clear: 2,
        previousClear: 2,
      }),
      makeRecord({
        title: 'Failed',
        level: 12,
        clear: 1,
        previousClear: 1,
      }),
    ],
  },
}

export const LongTitles: Story = {
  args: {
    records: [
      makeRecord({
        title: '%E3%83%96%E3%83%B3%E3%82%BF%E3%83%B3 ～Falling in "B" mix～',
        subtitle: '(EXPERT)',
        level: 12,
        difficulty: 3,
        clear: 7,
        exScore: 2456,
        minBp: 8,
        previousClear: 6,
        previousExScore: 2300,
        previousMinBp: 15,
      }),
      makeRecord({
        title: 'Ascension to Heaven -Long Version-',
        subtitle: '-ANOTHER-',
        level: 12,
        difficulty: 4,
        clear: 5,
        exScore: 1834,
        minBp: 23,
        previousClear: 5,
        previousExScore: 1790,
        previousMinBp: 28,
      }),
      makeRecord({
        title: 'FREEDOM DiVE',
        subtitle: '',
        level: 11,
        difficulty: 3,
        clear: 8,
        exScore: 3102,
        minBp: 0,
        previousClear: 7,
        previousExScore: 2900,
        previousMinBp: 3,
      }),
      makeRecord({
        title: 'Pandora',
        subtitle: '[MANIAC]',
        level: 12,
        difficulty: 3,
        clear: 1,
        exScore: 456,
        minBp: 120,
        previousClear: null,
        previousExScore: null,
        previousMinBp: null,
      }),
    ],
  },
}

export const ManyEntries: Story = {
  args: {
    records: Array.from({ length: 20 }, (_, i) =>
      makeRecord({
        title: `Song ${String(20 - i).padStart(2, '0')} - ${['FREEDOM DiVE', 'Groundbreaking', 'L9', 'Pandora', 'Ascension to Heaven'][i % 5]}`,
        level: 8 + (i % 5),
        clear: 1 + (i % 7),
        exScore: 800 + i * 50,
        minBp: Math.max(0, 40 - i * 2),
        previousClear: i % 3 === 0 ? i % 7 : 1 + (i % 7),
        previousExScore: 780 + i * 50,
        previousMinBp: Math.max(0, 42 - i * 2),
      }),
    ),
  },
}
