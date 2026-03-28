import { describe, expect, it } from 'vitest'

import { filterRecords, isUpdatedRecord } from '@/components/filterRecords'
import type { PlayRecord } from '@/types'

function makeRecord(overrides: Partial<PlayRecord> = {}): PlayRecord {
  return {
    id: '1',
    sha256: 'abc',
    mode: 0,
    clear: 3,
    exScore: 1000,
    minBp: 50,
    notes: 500,
    combo: 200,
    playedAt: '2026-03-20T12:00:00',
    title: 'Test Song',
    subtitle: '',
    artist: 'Test Artist',
    level: 10,
    difficulty: 3,
    tableLevels: [],
    previousClear: null,
    previousExScore: null,
    previousMinBp: null,
    isRetired: false,
    ...overrides,
  }
}

describe('isUpdatedRecord', () => {
  it.each([
    {
      name: 'no previous values',
      record: makeRecord(),
      expected: false,
    },
    {
      name: 'clear improved',
      record: makeRecord({ clear: 4, previousClear: 3 }),
      expected: true,
    },
    {
      name: 'clear not improved',
      record: makeRecord({ clear: 3, previousClear: 3 }),
      expected: false,
    },
    {
      name: 'score improved',
      record: makeRecord({ exScore: 1100, previousExScore: 1000 }),
      expected: true,
    },
    {
      name: 'score not improved',
      record: makeRecord({ exScore: 1000, previousExScore: 1000 }),
      expected: false,
    },
    {
      name: 'bp improved',
      record: makeRecord({ minBp: 40, previousMinBp: 50 }),
      expected: true,
    },
    {
      name: 'bp not improved',
      record: makeRecord({ minBp: 50, previousMinBp: 50 }),
      expected: false,
    },
    {
      name: 'multiple improvements',
      record: makeRecord({
        clear: 5,
        previousClear: 3,
        exScore: 1200,
        previousExScore: 1000,
        minBp: 30,
        previousMinBp: 50,
      }),
      expected: true,
    },
    {
      name: 'score worse but clear improved',
      record: makeRecord({
        clear: 4,
        previousClear: 3,
        exScore: 900,
        previousExScore: 1000,
      }),
      expected: true,
    },
  ])('$name', ({ record, expected }) => {
    expect(isUpdatedRecord(record)).toBe(expected)
  })
})

describe('filterRecords', () => {
  const updatedRecord = makeRecord({
    id: 'updated',
    clear: 5,
    previousClear: 3,
  })
  const notUpdatedRecord = makeRecord({
    id: 'not-updated',
    clear: 3,
    previousClear: 3,
  })
  const noPreviousRecord = makeRecord({
    id: 'no-previous',
  })

  it.each([
    {
      name: 'all filter returns all records',
      filter: 'all' as const,
      records: [updatedRecord, notUpdatedRecord, noPreviousRecord],
      expected: [updatedRecord, notUpdatedRecord, noPreviousRecord],
    },
    {
      name: 'updated filter returns only updated records',
      filter: 'updated' as const,
      records: [updatedRecord, notUpdatedRecord, noPreviousRecord],
      expected: [updatedRecord],
    },
    {
      name: 'updated filter with no updated records',
      filter: 'updated' as const,
      records: [notUpdatedRecord, noPreviousRecord],
      expected: [],
    },
    {
      name: 'updated filter with empty input',
      filter: 'updated' as const,
      records: [],
      expected: [],
    },
  ])('$name', ({ records, filter, expected }) => {
    expect(filterRecords(records, filter)).toEqual(expected)
  })
})
