export interface AppConfig {
  beatorajaRoot: string
  playerName: string
  resetTime: string
  backgroundTransparent: boolean
  fontSize: number
}

export interface DbFileStatus {
  name: string
  found: boolean
}

export interface PlayRecord {
  id: string
  sha256: string
  mode: number
  clear: number
  exScore: number
  minBp: number
  notes: number
  combo: number
  playedAt: string
  title: string
  artist: string
  level: number
  difficulty: number
  previousClear: number | null
  previousExScore: number | null
  previousMinBp: number | null
}

export interface ScoresUpdatedPayload {
  records: PlayRecord[]
  updatedAt: string
}

export type FilterType = 'all' | 'updated'
