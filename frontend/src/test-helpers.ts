import type { PlayRecord } from '@/types'

export function makeRecord(overrides: Partial<PlayRecord> = {}): PlayRecord {
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
    previousClear: null,
    previousExScore: null,
    previousMinBp: null,
    isRetired: false,
    ...overrides,
  }
}
