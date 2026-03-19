import type { FilterType, PlayRecord } from '@/types'

export function isUpdatedRecord(record: PlayRecord): boolean {
  const clearImproved =
    record.previousClear != null && record.clear > record.previousClear
  const scoreImproved =
    record.previousExScore != null && record.exScore > record.previousExScore
  const bpImproved =
    record.previousMinBp != null && record.minBp < record.previousMinBp

  return clearImproved || scoreImproved || bpImproved
}

export function filterRecords(
  records: PlayRecord[],
  filter: FilterType,
): PlayRecord[] {
  if (filter === 'all') return records
  return records.filter(isUpdatedRecord)
}
